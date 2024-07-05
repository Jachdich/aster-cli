use crate::api::{self, Channel, Request, Response, User};
use crate::LocalMessage;
use base64::prelude::*;
use fmtstring::FmtString;
use native_tls::TlsConnector;
use notify_rust::{Notification, Timeout};
use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::collections::HashMap;
use std::io::Write;
use std::net::SocketAddr;
use std::sync::mpsc::Sender;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::io::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::broadcast::Receiver;
use tokio_native_tls::TlsStream;

type SocketStream = TlsStream<TcpStream>;

pub trait WriteAsterRequestAsync {
    async fn write_request(&mut self, command: api::Request) -> Result<usize, std::io::Error>;
}

pub trait WriteAsterRequest {
    fn write_request(&mut self, command: &api::Request) -> Result<(), std::io::Error>;
}

impl WriteAsterRequest for native_tls::TlsStream<std::net::TcpStream> {
    fn write_request(&mut self, command: &api::Request) -> Result<(), std::io::Error> {
        write!(self, "{}\n", serde_json::to_string(command).unwrap())
    }
}

impl WriteAsterRequestAsync for WriteHalf<SocketStream> {
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
impl Peer {
    fn from_user(user: User) -> Self {
        let pfp_bytes = BASE64_STANDARD.decode(user.pfp).unwrap();
        let img = image::load_from_memory(&pfp_bytes)
            .unwrap()
            .resize_exact(14, 16, image::imageops::FilterType::Triangle)
            .into_rgb8();

        let pfp = dct_tiv::textify_dct(
            &img,
            &dct_tiv::DEFAULT_DCT_MATRICIES,
            &dct_tiv::DEFAULT_PALETTE,
        )
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

pub struct LoadedMessage {
    pub lines: Vec<FmtString>,
    pub message: api::Message,
}

pub struct OnlineServer {
    pub loaded_messages: Vec<LoadedMessage>,
    pub channels: Vec<Channel>,
    pub curr_channel: Option<usize>,
    pub peers: HashMap<i64, Peer>,
    pub write_half: WriteHalf<SocketStream>,
    pub remote_addr: SocketAddr,
}

pub struct Server {
    pub ip: String,
    pub port: u16,
    pub name: Option<String>,
    pub uuid: Option<i64>,
    pub uname: Option<String>,
    pub network: Result<OnlineServer, String>,
}

impl LoadedMessage {
    pub fn from_message(
        message: api::Message,
        peers: &HashMap<i64, Peer>,
        width: usize,
    ) -> LoadedMessage {
        let mut this = LoadedMessage {
            lines: Vec::new(),
            message,
        };
        this.rebuild(peers, width);
        this
    }

    pub fn rebuild(&mut self, peers: &HashMap<i64, Peer>, width: usize) {
        let uname_str = peers
            .get(&self.message.author_uuid)
            .map(|x| x.name.as_str())
            .unwrap_or("Unknown User");
        let formatted = FmtString::from_str(&format!(" {}: {}", uname_str, self.message.content));

        let pfp = peers
            .get(&self.message.author_uuid)
            .map(|x| x.pfp.clone())
            .unwrap_or(FmtString::from_str("  "));

        let left_margin = pfp.len() + 1;
        self.lines = vec![pfp];
        for c in formatted.into_iter() {
            // unwraps are ok because we start with at least 1 element
            let curr = self.lines.last().unwrap();
            if c.ch == '\n' || curr.len() >= width - 1 {
                self.lines
                    .push(FmtString::from_str(&" ".repeat(left_margin)))
            }
            let curr = self.lines.last_mut().unwrap();
            if c.ch != '\n' {
                curr.push(c);
            }
        }
    }
}

impl Serialize for Server {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("Server", 6)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("ip", &self.ip)?;
        state.serialize_field("port", &self.port)?;
        state.serialize_field("uuid", &self.uuid)?;
        state.serialize_field("uname", &self.uname)?;
        state.serialize_field("idx", &0)?; // TODO temp
        state.end()
    }
}

#[derive(Clone)]
pub enum Identification {
    Username(String),
    Uuid(i64),
}

impl OnlineServer {
    pub async fn initialise(
        &mut self,
        id: Identification,
        passwd: String,
    ) -> std::result::Result<(), std::io::Error> {
        use Request::*;
        match id {
            Identification::Uuid(uuid) => {
                self.write_half
                    .write_request(LoginRequest {
                        passwd,
                        uname: None,
                        uuid: Some(uuid),
                    })
                    .await?;
                self.write_half
                    .write_request(GetUserRequest { uuid })
                    .await?; // just make certain we get our own name
                Ok(())
            }
            Identification::Username(username) => {
                self.write_half
                    .write_request(LoginRequest {
                        passwd,
                        uname: Some(username),
                        uuid: None,
                    })
                    .await?;
                Ok(())
            }
        }
    }

    async fn post_init(&mut self) -> Result<(), std::io::Error> {
        use Request::*;
        self.write_half.write_request(GetIconRequest).await?;
        self.write_half.write_request(GetNameRequest).await?;
        self.write_half.write_request(GetMetadataRequest).await?;
        self.write_half.write_request(ListChannelsRequest).await?;
        self.write_half.write_request(OnlineRequest).await?;
        Ok(())
    }
    pub async fn write(&mut self, request: Request) -> Result<usize, std::io::Error> {
        self.write_half.write_request(request).await
    }

    pub fn get_channel(&self, uuid: i64) -> Option<&Channel> {
        self.channels.iter().find(|c| c.uuid == uuid)
    }

    pub async fn switch_channel(&mut self, idx: usize) {
        self.loaded_messages.clear();
        self.curr_channel = Some(idx);
        let channel = self.channels[idx].uuid;
        let res = self
            .write_half
            .write_request(api::Request::HistoryRequest {
                num: 100,
                channel,
                before_message: None,
            })
            .await;
        if let Err(_) = res {
            // *s = (*s).to_offline(e.to_string());
            // TODO make the server offline
        }
    }
}

impl Server {
    pub async fn new(
        ip: String,
        port: u16,
        id: Identification,
        tx: Sender<LocalMessage>,
        mut cancel: Receiver<()>,
    ) -> Self {
        let network = match TcpStream::connect((ip.as_str(), port)).await {
            Ok(socket) => {
                let addr = socket.peer_addr().unwrap(); // TODO figure out if this unwrap is ever gonna cause a problem

                let cx = TlsConnector::builder()
                    .danger_accept_invalid_certs(true)
                    .build()
                    .expect("Couldn't initialise a TLS connection");
                let cx = tokio_native_tls::TlsConnector::from(cx);

                match cx.connect(ip.as_str(), socket).await {
                    Ok(socket) => {
                        let (read_half, write_half) = tokio::io::split(socket);

                        let net_tx = tx.clone();
                        tokio::spawn(async move {
                            tokio::select! {
                                _ = Self::run_network(net_tx, read_half, addr) => {},
                                _ = cancel.recv() => {}, // we need to shut down the connection rn
                            }
                        });
                        Ok(OnlineServer {
                            loaded_messages: Vec::new(),
                            channels: Vec::new(),
                            curr_channel: None,
                            peers: HashMap::new(),
                            write_half,
                            remote_addr: addr,
                        })
                    }
                    Err(e) => Err(format!("Failed to init TLS encryption: {:?}", e)),
                }
            }
            Err(e) => Err(format!("Failed to connect: {:?}", e)),
        };

        let (uname, uuid) = match id {
            Identification::Username(uname) => (Some(uname), None),
            Identification::Uuid(uuid) => (None, Some(uuid)),
        };

        Self {
            ip,
            port,
            name: None,
            uuid,
            uname,
            network,
        }
    }

    pub fn to_offline(&mut self, offline_reason: String) {
        self.network = Err(offline_reason);
    }

    pub fn is_online(&self) -> bool {
        self.network.is_ok()
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
                Ok(len) => {
                    if len == 0 {
                        tx.send(LocalMessage::NetError(
                            "Server sent zero-length response (likely shut down)".into(),
                        ))
                        .unwrap();
                        return;
                    }
                    tx.send(LocalMessage::Network(result, addr)).unwrap();
                }
                Err(e) => {
                    tx.send(LocalMessage::NetError(format!(
                        "Error occurred in network thread, ip: {}: {:?}",
                        addr, e
                    )))
                    .unwrap();
                    return;
                }
            }
        }
    }

    fn format_message(
        msg: api::Message,
        peers: &HashMap<i64, Peer>,
        message_width: usize,
    ) -> LoadedMessage {
        LoadedMessage::from_message(msg, peers, message_width)
    }

    pub async fn handle_network_packet(
        &mut self,
        response: Response,
        message_width: usize,
        inactivity_time: Duration,
        we_are_the_selected_server: bool,
    ) -> Result<(), String> {
        use api::Status::{self, *};
        use Response::*;
        let net = self
            .network
            .as_mut()
            .expect("Network packet recv'd for offline server??");
        match response {
            GetMetadataResponse { data, .. } => {
                for elem in data.unwrap() {
                    let peer = Peer::from_user(elem);
                    if self.uuid.is_some_and(|uuid| uuid == peer.uuid) {
                        // info about ourselves that we may not know yet!
                        if self.uname.is_none() {
                            self.uname = Some(peer.name.clone());
                        }
                    }
                    net.peers.insert(peer.uuid, peer);
                }
            }
            RegisterResponse {
                uuid: new_uuid,
                status: Ok,
            } => self.uuid = Some(new_uuid.unwrap()),
            RegisterResponse {
                status: Conflict, ..
            } => {
                return Err(format!(
                    "Cannot register with username '{}' as it is already in use",
                    self.uname.as_ref().unwrap()
                ));
            }
            LoginResponse {
                uuid: Some(new_uuid),
                status: Ok,
            } => {
                self.uuid = Some(new_uuid);
                net.post_init().await.unwrap(); // TODO get rid of this unwrap
            }
            LoginResponse {
                status: NotFound, ..
            } => {
                // Try register instead
                let uname = self
                    .uname
                    .as_ref()
                    .ok_or("No username to register with!".to_owned())?
                    .to_owned();
                net.write_half
                    .write_request(Request::RegisterRequest {
                        passwd: "a".into(),
                        uname,
                    })
                    .await
                    .unwrap(); // TODO get rid of this
            }
            LoginResponse {
                status: Forbidden, ..
            } => {
                return Err(format!(
                    "Invalid password for {}@{}:{}",
                    self.uname.as_ref().map(|s| s.as_str()).unwrap_or(
                        self.uuid
                            .map(|uuid| format!("{}", uuid))
                            .unwrap_or("{unknown}".to_owned()) // this should never happen, but just in case
                            .as_str()
                    ),
                    self.ip,
                    self.port
                ));
            }
            GetNameResponse { data, status: Ok } => self.name = Some(data.unwrap()),
            ListChannelsResponse { data, status: Ok } => net.channels = data.unwrap(),
            HistoryResponse { data, status: Ok } => {
                let new_msgs = data
                    .unwrap()
                    .into_iter()
                    .map(|message| Self::format_message(message, &net.peers, message_width))
                    .collect::<Vec<_>>(); // TODO get rid of this collect: borrow checker complains, tho
                net.loaded_messages.extend(new_msgs);
            }
            ContentResponse { message, .. } => {
                let in_current_channel = net
                    .curr_channel
                    .is_some_and(|c| net.channels[c].uuid == message.channel_uuid);
                if in_current_channel {
                    net.loaded_messages.push(Self::format_message(
                        message.clone(),
                        &net.peers,
                        message_width,
                    ));
                }
                if !we_are_the_selected_server
                    || !in_current_channel
                    || inactivity_time > Duration::from_secs(10)
                {
                    Notification::new()
                        .summary(&format!(
                            "{} #{}",
                            self.name.as_ref().map(|s| s.as_str()).unwrap_or(" "),
                            net.get_channel(message.channel_uuid)
                                .map(|c| c.name.as_str())
                                .unwrap_or("Unknown Channel"),
                        ))
                        .body(&format!(
                            "{}: {}",
                            net.peers
                                .get(&message.author_uuid)
                                .map(|p| p.name.as_str())
                                .unwrap_or("Unknown User"),
                            message.content
                        ))
                        .timeout(Timeout::Milliseconds(6000))
                        .show()
                        .unwrap();
                }
            }

            MessageEditedResponse {
                status: Ok,
                message,
                new_content,
            } => {
                for msg in &mut net.loaded_messages {
                    if msg.message.uuid == message {
                        msg.message.content = new_content;
                        msg.message.edited = true;
                        msg.rebuild(&net.peers, message_width);
                        break;
                    }
                }
            }

            MessageDeletedResponse {
                status: Ok,
                message,
            } => {
                net.loaded_messages
                    .retain(|msg| msg.message.uuid != message);
            }

            _ => {
                if response.status() != Status::Ok {
                    return Err(format!(
                        "Non-OK status from {}:{}: {}: {}",
                        self.ip,
                        self.port,
                        response.name(),
                        response.status(),
                    ));
                }
            }
        }
        Result::Ok(())
    }
}
