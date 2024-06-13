extern crate termion;
extern crate tokio;

use crate::api::{self, Channel, Request, Response, User};
use crate::tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use crate::LocalMessage;
use base64::prelude::*;
use dct_tiv::Matrix;
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

pub trait WriteAsterRequest {
    async fn write_request(&mut self, command: api::Request) -> Result<usize, std::io::Error>;
}

impl WriteAsterRequest for WriteHalf<SocketStream> {
    async fn write_request(&mut self, command: api::Request) -> Result<usize, std::io::Error> {
        // Unwrap is fine because I'm pretty certain if the request can't be serialised
        // then there's something dramatically wrong
        let res =
            AsyncWriteExt::write(self, serde_json::to_string(&command).unwrap().as_bytes()).await?;
        self.write_u8(10).await?;
        Ok(res)
    }
}

// for info like pfp alreadt converted to a fmtstring
pub struct Peer {
    pub uuid: i64,
    pub name: String,
    pub pfp: FmtString,
}

// aight fuck you, global mutable variables i dont care
static mut CALIBRATION_MATRICES: Vec<Matrix<7, 16>> = Vec::new();
static mut PALETTE: Vec<char> = Vec::new();

pub fn init_calibration_matrices() {
    // SAFETY
    // This function is called once at the start of the program. A race condition should be impossible.
    unsafe {
        CALIBRATION_MATRICES = dct_tiv::create_calibration_matrices(true);
        PALETTE = dct_tiv::get_palette();
    }
}

impl Peer {
    fn from_user(user: User) -> Self {
        let pfp_bytes = BASE64_STANDARD.decode(user.pfp).unwrap();
        let img = image::load_from_memory(&pfp_bytes)
            .unwrap()
            .resize_exact(14, 16, image::imageops::FilterType::Triangle)
            .into_rgb8();

        // yea this requires unsafe, but it avoids a dependency, and is perfectly safe™ I think
        let pfp = unsafe { dct_tiv::textify_dct(&img, &CALIBRATION_MATRICES, &PALETTE) }
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

#[rustfmt::skip]
#[allow(dead_code)] // because some methods might not be used yet but are included for completeness
impl Server {
    pub fn name(&self) -> Option<&str> {
        match self { Self::Online { name, .. } | Self::Offline { name, .. } => name.as_ref().map(|x| x.as_str()) }
    }
    pub fn uname(&self) -> Option<&str> {
        match self { Self::Online { uname, .. } | Self::Offline { uname, .. } => uname.as_ref().map(|x| x.as_str()) }
    }
    pub fn ip(&self) -> &str { match self { Self::Online { ip, .. } | Self::Offline { ip, .. } => &ip }}
    pub fn port(&self) -> u16 { match self { Self::Online { port, .. } | Self::Offline { port, .. } => *port }}
    pub fn uuid(&self) -> Option<i64> { match self { Self::Online { uuid, .. } | Self::Offline { uuid, .. } => *uuid }}
    pub fn set_ip(&mut self, v: String)    { match self { Self::Online { ip, .. }    | Self::Offline { ip, .. }    => *ip = v }}
    pub fn set_port(&mut self, v: u16)     { match self { Self::Online { port, .. }  | Self::Offline { port, .. }  => *port = v }}
    pub fn set_uuid(&mut self, v: i64)     { match self { Self::Online { uuid, .. }  | Self::Offline { uuid, .. }  => *uuid = Some(v) }}
    pub fn set_name(&mut self, v: String)  { match self { Self::Online { name, .. }  | Self::Offline { name, .. }  => *name = Some(v) }}
    pub fn set_uname(&mut self, v: String) { match self { Self::Online { uname, .. } | Self::Offline { uname, .. } => *uname = Some(v) }}
}

pub enum Identification {
    Username(String),
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
                let addr = socket.peer_addr().unwrap(); // TODO figure out if this unwrap is ever gonna cause a problem

                // let cx = TlsConnector::builder()
                //     .danger_accept_invalid_certs(true)
                //     .build()?;
                // let cx = tokio_native_tls::TlsConnector::from(cx);

                // let socket = cx.connect(ip, socket).await?;
                let (read_half, write_half) = tokio::io::split(socket);

                let net_tx = tx.clone();
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

    #[rustfmt::skip]
    pub fn to_offline(self, offline_reason: String) -> Self {
        match self {
            Self::Online  { ip, port, name, uuid, uname, .. }
          | Self::Offline { ip, port, name, uuid, uname, .. } =>
                Server::Offline { offline_reason, ip, port, name, uuid, uname },
        }
    }

    pub fn is_online(&self) -> bool {
        match self {
            Self::Online { .. } => true,
            Self::Offline { .. } => false,
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

    // pub async fn update_metadata(&mut self, meta: User) -> std::result::Result<(), std::io::Error> {
    //     self.write(Request::NickRequest { nick: meta.name }).await?;
    //     //self.write(object!{"command": "passwd", "passwd": meta.passwd}).await?;
    //     self.write(Request::PfpRequest { data: meta.pfp }).await?;
    //     Ok(())
    // }

    pub async fn initialise(
        &mut self,
        id: Identification,
    ) -> std::result::Result<(), std::io::Error> {
        use Request::*;
        match self {
            Self::Online { write_half, .. } => match id {
                Identification::Uuid(uuid) => {
                    write_half
                        .write_request(LoginRequest {
                            passwd: "a".into(),
                            uname: None,
                            uuid: Some(uuid),
                        })
                        .await?;
                    write_half.write_request(GetUserRequest { uuid }).await?; // just make certain we get our own name
                    self.set_uuid(uuid); // technically this should be before, but rust complains. should be fine.
                    Ok(())
                }
                Identification::Username(username) => {
                    write_half
                        .write_request(LoginRequest {
                            passwd: "a".into(),
                            uname: Some(username.clone()),
                            uuid: None,
                        })
                        .await?;
                    self.set_uname(username);
                    Ok(())
                }
            },
            Self::Offline { .. } => Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Cannot initialise offline server!",
            )),
        }
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
            peers
                .get(&msg.author_uuid)
                .map(|x| x.name.as_str())
                .unwrap_or("Unknown User"),
            msg.content
        ));
        // let mut ks: Vec<&i64> = peers.keys().collect();
        // use rand::prelude::*;
        // let mut rng = rand::thread_rng();
        // ks.shuffle(&mut rng);

        // let pfp = peers.get(ks[0]).unwrap().pfp.clone();
        let pfp = peers
            .get(&msg.author_uuid)
            .map(|x| x.pfp.clone())
            .unwrap_or(FmtString::from_str("  "));
        FmtString::concat(pfp, formatted)
    }

    async fn post_init(&mut self) -> Result<(), std::io::Error> {
        use Request::*;
        match self {
            Self::Online { write_half, .. } => {
                write_half.write_request(GetIconRequest).await?;
                write_half.write_request(GetNameRequest).await?;
                write_half.write_request(GetMetadataRequest).await?;
                write_half.write_request(ListChannelsRequest).await?;
                write_half.write_request(OnlineRequest).await?;
                Ok(())
            }
            Self::Offline { .. } => Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Cannot initialise offline server!",
            )),
        }
    }

    pub async fn handle_network_packet(&mut self, response: Response) -> Result<(), String> {
        if let Self::Online {
            peers,
            uuid,
            name,
            loaded_messages,
            channels,
            uname,
            write_half,
            ..
        } = self
        {
            use api::Status;
            use Response::*;
            match response {
                GetMetadataResponse { data, .. } => {
                    for elem in data.unwrap() {
                        let peer_uuid = elem.uuid;
                        let peer = Peer::from_user(elem);
                        if uuid.is_some_and(|uuid| uuid == peer_uuid) {
                            // info about ourselves that we may not know yet!
                            if uname.is_none() {
                                *uname = Some(peer.name.clone());
                            }
                        }
                        peers.insert(peer_uuid, peer);
                    }
                }
                RegisterResponse { uuid: new_uuid, .. } => *uuid = Some(new_uuid.unwrap()),
                LoginResponse {
                    uuid: Some(new_uuid),
                    status: Status::Ok,
                } => {
                    *uuid = Some(new_uuid);
                    self.post_init().await.unwrap(); // TODO get rid of this unwrap
                }
                LoginResponse {
                    status: Status::NotFound,
                    ..
                } => {
                    // Try register instead
                    let uname = uname
                        .as_ref()
                        .ok_or("No username to register with!".to_owned())?
                        .to_owned();
                    write_half
                        .write_request(Request::RegisterRequest {
                            passwd: "a".into(),
                            uname,
                        })
                        .await
                        .unwrap(); // TODO get rid of this
                }
                LoginResponse {
                    status: Status::Forbidden,
                    ..
                } => {
                    return Err(format!(
                        "Invalid password for {}@{}:{}",
                        self.uname().unwrap_or(
                            self.uuid()
                                .map(|uuid| format!("{}", uuid))
                                .unwrap_or("{unknown}".to_owned()) // this should never happen, but just in case
                                .as_str()
                        ),
                        self.ip(),
                        self.port()
                    ));
                }
                GetNameResponse { data, .. } => *name = Some(data.unwrap()),
                ListChannelsResponse { data, .. } => *channels = data.unwrap(),
                HistoryResponse { data, .. } => loaded_messages.extend(
                    data.unwrap()
                        .into_iter()
                        .map(|message| Self::format_message(&message, peers)),
                ),
                ContentResponse { message, .. } => {
                    loaded_messages.push(Self::format_message(&message, peers))
                }
                _ => {
                    if response.status() != Status::Ok {
                        return Err(format!(
                            "Non-OK status from {}:{}: {}: {}",
                            self.ip(),
                            self.port(),
                            response.name(),
                            response.status(),
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}
