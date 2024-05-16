extern crate termion;
extern crate tokio;

use crate::api::{self, Channel, Request, Response, User};
use crate::tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use crate::LocalMessage;
use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::mpsc::Sender;
use tokio::io::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;
// use tokio_native_tls::TlsStream;

use super::DisplayMessage;

type SocketStream = TcpStream;

// TODO: which of name, uname and uuid, if any, should be Options?
pub enum Server {
    Online {
        loaded_messages: Vec<DisplayMessage>,
        channels: Vec<Channel>,
        curr_channel: Option<usize>,
        peers: HashMap<i64, User>,
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

// pub fn to_json(&self) -> json::JsonValue {
//     json::object! {
//         name: self.name().into(),
//         ip: self.ip().clone(),
//         port: self.port(),
//         uuid: self.uuid(),
//     }
// }

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

impl Server {
    pub async fn new(ip: String, port: u16, uuid: i64, tx: Sender<LocalMessage>) -> Self {
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
                    Self::run_network(net_tx, read_half, addr).await;
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
                    uuid: Some(uuid),
                    uname: None,
                }
            }
            Err(e) => Self::Offline {
                offline_reason: format!("Failed to connect: {:?}", e),
                ip,
                port,
                name: None,
                uuid: Some(uuid),
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

    pub async fn initialise(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        use Request::*;
        if let Some(uuid) = self.uuid() {
            self.write(LoginRequest {
                passwd: "a".into(),
                uname: None,
                uuid: Some(uuid),
            })
            .await?;
        } else {
            self.write(RegisterRequest {
                passwd: "a".into(),
                uname: self.uname().unwrap().into(),
            })
            .await?;
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
                        peers.insert(elem.uuid, elem);
                    }
                }
                RegisterResponse { uuid: new_uuid } => *uuid = Some(new_uuid),
                GetNameResponse { data } => *name = Some(data),
                ListChannelsResponse { data } => *channels = data,
                HistoryResponse { data } => loaded_messages.extend(
                    data.into_iter()
                        .map(|message| DisplayMessage::User(message)),
                ),
                ContentResponse(message) => loaded_messages.push(DisplayMessage::User(message)),
                _ => (),
            }
        }
    }
}
