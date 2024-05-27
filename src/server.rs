extern crate termion;
extern crate tokio;

use crate::api::{self, Channel, Request, Response, User};
use crate::tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use crate::LocalMessage;
use base64::prelude::*;
use fmtstring::FmtString;
use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::mpsc::Sender;
use tokio::io::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::broadcast::Receiver;
// use tokio_native_tls::TlsStream;

type SocketStream = TcpStream;

// for info like pfp alreadt converted to a fmtstring
pub struct Peer {
    pub uuid: i64,
    pub name: String,
    pub pfp: FmtString,
}

impl Peer {
    fn from_user(user: User) -> Self {
        let pfp_bytes = BASE64_STANDARD.decode(user.pfp).unwrap();
        let img = image::load_from_memory(&pfp_bytes)
            .unwrap()
            .resize_exact(14, 16, image::imageops::FilterType::Triangle)
            .into_rgb8();

        let matrix = dct_tiv::create_calibration_matrices(true);
        let palette = dct_tiv::get_palette();
        let pfp = dct_tiv::textify_dct(&img, &matrix, &palette)
            .into_iter()
            .next()
            .unwrap();
        Self {
            uuid: user.uuid,
            name: user.name,
            pfp, // TODO assert len == 1
        }
    }
}

// TODO: which of name, uname and uuid, if any, should be Options?
pub enum Server {
    Online {
        loaded_messages: Vec<FmtString>,
        channels: Vec<Channel>,
        curr_channel: Option<usize>,
        peers: HashMap<i64, Peer>,
        write_half: WriteHalf<SocketStream>,
        remote_addr: SocketAddr,
        ip: String,
        port: u16,
        name: Option<String>,
        uuid: Option<i64>,
        uname: Option<String>,
    },
    Offline {
        offline_reason: String,
        ip: String,
        port: u16,
        name: Option<String>,
        uuid: Option<i64>,
        uname: Option<String>,
    },
}
impl Serialize for Server {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("Server", 5)?;
        state.serialize_field("name", &self.name())?;
        state.serialize_field("ip", &self.ip())?;
        state.serialize_field("port", &self.port())?;
        state.serialize_field("uuid", &self.uuid())?;
        state.serialize_field("uname", &self.uname())?;
        state.end()
    }
}

impl Server {
    pub fn ip(&self) -> &str {
        match self {
            Self::Online { ip, .. } | Self::Offline { ip, .. } => &ip,
        }
    }
    pub fn port(&self) -> u16 {
        match self {
            Self::Online { port, .. } | Self::Offline { port, .. } => *port,
        }
    }
    pub fn uuid(&self) -> Option<i64> {
        match self {
            Self::Online { uuid, .. } | Self::Offline { uuid, .. } => *uuid,
        }
    }
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Online { name, .. } | Self::Offline { name, .. } => {
                name.as_ref().map(|x| x.as_str())
            }
        }
    }
    pub fn uname(&self) -> Option<&str> {
        match self {
            Self::Online { uname, .. } | Self::Offline { uname, .. } => {
                uname.as_ref().map(|x| x.as_str())
            }
        }
    }
}

pub enum Identification<'a> {
    Username(&'a str),
    Uuid(i64),
}

impl Server {
    pub async fn new(
        ip: String,
        port: u16,
        tx: Sender<LocalMessage>,
        mut cancel: Receiver<()>,
    ) -> Self {
        match TcpStream::connect((ip.as_str(), port)).await {
            Ok(socket) => {
                let addr = socket.peer_addr().unwrap(); // TODO figure out if this is ever gonna cause a problem

                // let cx = TlsConnector::builder()
                //     .danger_accept_invalid_certs(true)
                //     .build()?;
                // let cx = tokio_native_tls::TlsConnector::from(cx);

                // let socket = cx.connect(ip, socket).await?;
                let (read_half, write_half) = tokio::io::split(socket);

                let net_tx = tx.clone();
                // let the_ip = ip.to_owned();
                tokio::spawn(async move {
                    tokio::select! {
                        _ = Self::run_network(net_tx, read_half, addr) => {},
                        _ = cancel.recv() => {}, // we need to shut down the connection rn
                    }
                });
                Self::Online {
                    loaded_messages: Vec::new(),
                    channels: Vec::new(),
                    curr_channel: None,
                    peers: HashMap::new(),
                    write_half,
                    remote_addr: addr,
                    ip,
                    port,
                    name: None,
                    uuid: None,
                    uname: None,
                }
            }
            Err(e) => Self::Offline {
                offline_reason: format!("Failed to connect: {:?}", e),
                ip,
                port,
                name: None,
                uuid: None,
                uname: None,
            },
        }
    }

    // pub fn add_message(&mut self, content: FmtString, author: u64) {
    //     //todo compile regex once and use it mulyiple times, this is slow as fuck
    //     //let url_regex = r#"^https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)$"#;
    //     //let compiled_regex = regex::Regex::new(url_regex).unwrap();
    //     //if compiled_regex.is_match(&content) {
    //     //this is gonna be fuckin rough... brace yourselves
    //     //}

    //     match self {
    //         Self::Online {
    //             loaded_messages, ..
    //         } => loaded_messages.push(DisplayMessage::User { content, author }),
    //         Self::Offline { .. } => panic!("Try to add a message to an offline server!"),
    //     }
    // }

    pub async fn write(
        &mut self,
        command: api::Request,
    ) -> std::result::Result<usize, std::io::Error> {
        match self {
            Self::Online { write_half, .. } => {
                let res = write_half
                    .write(serde_json::to_string(&command)?.as_bytes())
                    .await?;
                write_half.write_u8(10).await?;
                Ok(res)
            }
            Self::Offline { .. } => Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Server is offline",
            )),
        }
    }

    pub async fn update_metadata(&mut self, meta: User) -> std::result::Result<(), std::io::Error> {
        self.write(Request::NickRequest { nick: meta.name }).await?;
        //self.write(object!{"command": "passwd", "passwd": meta.passwd}).await?;
        self.write(Request::PfpRequest { data: meta.pfp }).await?;
        Ok(())
    }

    pub async fn initialise(
        &mut self,
        id: Identification<'_>,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        use Request::*;
        match id {
            Identification::Uuid(uuid) => {
                self.write(LoginRequest {
                    passwd: "a".into(),
                    uname: None,
                    uuid: Some(uuid),
                })
                .await?;
            }
            Identification::Username(username) => {
                self.write(LoginRequest {
                    passwd: "a".into(),
                    uname: Some(username.into()),
                    uuid: None,
                })
                .await?;
            }
        }
        self.write(GetIconRequest).await?;
        self.write(GetNameRequest).await?;
        self.write(GetMetadataRequest).await?;
        self.write(ListChannelsRequest).await?;
        self.write(OnlineRequest).await?;
        // self.write(HistoryRequest {
        //     num: 100,
        //     channel: 0,
        //     before_message: None,
        // })
        // .await?;
        Ok(())
    }

    async fn run_network(
        tx: std::sync::mpsc::Sender<LocalMessage>,
        stream: ReadHalf<SocketStream>,
        // addr: (String, u16),
        addr: SocketAddr,
    ) {
        let mut reader = tokio::io::BufReader::new(stream);
        // let addr_hash = addr.hash();

        loop {
            let mut result: String = "".to_string();
            match reader.read_line(&mut result).await {
                Ok(_len) => {
                    tx.send(LocalMessage::Network(result, addr)).unwrap();
                }
                Err(e) => {
                    println!("Error occurred in network thread, ip: {}: {:?}", addr, e);
                    return;
                }
            }
        }
    }

    fn format_message(msg: &api::Message, peers: &HashMap<i64, Peer>) -> FmtString {
        let formatted = FmtString::from_str(&format!(
            " {}: {}",
            peers.get(&msg.author_uuid).unwrap().name,
            msg.content
        ));
        // let mut ks: Vec<&i64> = peers.keys().collect();
        // use rand::prelude::*;
        // let mut rng = rand::thread_rng();
        // ks.shuffle(&mut rng);

        // let pfp = peers.get(ks[0]).unwrap().pfp.clone();
        let pfp = peers.get(&msg.author_uuid).unwrap().pfp.clone();
        FmtString::concat(pfp, formatted)
    }

    pub fn handle_network_packet(&mut self, response: Response) {
        if let Self::Online {
            peers,
            uuid,
            name,
            loaded_messages,
            channels,
            ..
        } = self
        {
            use Response::*;
            match response {
                GetMetadataResponse { data } => {
                    for elem in data {
                        let uuid = elem.uuid;
                        let peer = Peer::from_user(elem);
                        peers.insert(uuid, peer);
                    }
                }
                RegisterResponse { uuid: new_uuid } => *uuid = Some(new_uuid),
                GetNameResponse { data } => *name = Some(data),
                ListChannelsResponse { data } => *channels = data,
                HistoryResponse { data } => loaded_messages.extend(
                    data.into_iter()
                        .map(|message| Self::format_message(&message, peers)),
                ),
                ContentResponse(message) => {
                    loaded_messages.push(Self::format_message(&message, peers))
                }
                _ => (),
            }
        }
    }
}
