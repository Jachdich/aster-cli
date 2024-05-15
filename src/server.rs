extern crate termion;
extern crate tokio;

use crate::tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use crate::FmtString;
use crate::LocalMessage;
use enum_common_fields::EnumCommonFields;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::mpsc::Sender;
use tokio::io::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;
// use tokio_native_tls::TlsStream;

use super::Message;
use super::User;
use json::{object, JsonValue};

type SocketStream = TcpStream;

// TODO: which of name and uuid, if any, should be Options?
pub enum Server {
    Online {
        loaded_messages: Vec<Message>,
        channels: Vec<String>,
        curr_channel: Option<usize>,
        peers: HashMap<u64, User>,
        write_half: WriteHalf<SocketStream>,
        remote_addr: SocketAddr,
        ip: String,
        port: u16,
        name: Option<String>,
        uuid: Option<u64>,
    },
    Offline {
        offline_reason: String,
        ip: String,
        port: u16,
        name: Option<String>,
        uuid: Option<u64>,
    },
}

impl Server {
    fn ip(&self) -> &str {
        match self {
            Self::Online { ip, .. } | Self::Offline { ip, .. } => &ip,
        }
    }
    fn port(&self) -> u16 {
        match self {
            Self::Online { port, .. } | Self::Offline { port, .. } => *port,
        }
    }
    fn uuid(&self) -> Option<u64> {
        match self {
            Self::Online { uuid, .. } | Self::Offline { uuid, .. } => *uuid,
        }
    }
    fn name(&self) -> Option<&str> {
        match self {
            Self::Online { name, .. } | Self::Offline { name, .. } => name.map(|x| x.as_str()),
        }
    }
}

impl Server {
    pub async fn new(ip: String, port: u16, uuid: u64, tx: Sender<LocalMessage>) -> Self {
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
                    Self::run_network(net_tx, read_half, addr);
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
                }
            }
            Err(e) => Self::Offline {
                offline_reason: format!("Failed to connect: {:?}", e),
                ip,
                port,
                name: None,
                uuid: Some(uuid),
            },
        }
    }

    pub fn add_message(&mut self, content: FmtString, author: u64) {
        //todo compile regex once and use it mulyiple times, this is slow as fuck
        //let url_regex = r#"^https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)$"#;
        //let compiled_regex = regex::Regex::new(url_regex).unwrap();
        //if compiled_regex.is_match(&content) {
        //this is gonna be fuckin rough... brace yourselves
        //}

        match self {
            Self::Online {
                loaded_messages, ..
            } => loaded_messages.push(Message::User { content, author }),
            Self::Offline { .. } => panic!("Try to add a message to an offline server!"),
        }
    }

    pub async fn write(&mut self, value: JsonValue) -> std::result::Result<usize, std::io::Error> {
        match self {
            Self::Online { write_half, .. } => write_half.write(&value.dump().into_bytes()).await,
            Self::Offline { .. } => Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Server is offline",
            )),
        }
    }

    pub async fn update_metadata(&mut self, meta: User) -> std::result::Result<(), std::io::Error> {
        self.write(object! {"command": "nick", "nick": meta.nick})
            .await?;
        //self.write(object!{"command": "passwd", "passwd": meta.passwd}).await?;
        self.write(object! {"command": "pfp", "data": meta.pfp_b64})
            .await?;
        Ok(())
    }

    pub async fn initialise(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        if let Some(uuid) = self.uuid() {
            self.write(object! {"command": "login", "uuid": uuid, "passwd": "a"})
                .await?;
        } else {
            self.write(object! {"command": "register"}).await?;
        }
        self.write(object! {"command": "get_icon"}).await?;
        self.write(object! {"command": "get_name"}).await?;
        self.write(object! {"command": "get_all_metadata"}).await?;
        self.write(object! {"command": "get_channels"}).await?;
        self.write(object! {"command": "online"}).await?;
        self.write(object! {"command": "history", "num": 100, "channel": 0})
            .await?;
        Ok(())
    }

    // pub fn to_json(&self) -> json::JsonValue {
    //     json::object! {
    //         name: self.name().into(),
    //         ip: self.ip().clone(),
    //         port: self.port(),
    //         uuid: self.uuid(),
    //     }
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
                Ok(_len) => {
                    tx.send(LocalMessage::Network(result, addr));
                }
                Err(e) => {
                    println!(
                        "Error occurred in network thread, ip: {}:{}: {:?}",
                        addr.0, addr.1, e
                    );
                    return;
                }
            }
        }
    }

    pub fn handle_network_packet(&mut self, obj: json::JsonValue) {
        if let Self::Online {
            peers,
            uuid,
            name,
            loaded_messages,
            channels,
            ..
        } = self
        {
            if !obj["command"].is_null() {
                match obj["command"].to_string().as_str() {
                    "metadata" => {
                        for elem in obj["data"].members() {
                            let elem_uuid = elem["uuid"].as_u64().unwrap();
                            if !peers.contains_key(&elem_uuid) {
                                peers.insert(elem_uuid, User::from_json(elem));
                            } else {
                                peers.get_mut(&elem_uuid).unwrap().update(elem);
                            }
                        }
                    }
                    "set" => match obj["key"].to_string().as_str() {
                        "uuid" => {
                            *uuid = Some(obj["value"].as_u64().unwrap());
                        }
                        _ => (),
                    },
                    "get_name" => {
                        *name = Some(obj["data"].to_string());
                    }

                    "get_channels" => {
                        channels.clear();
                        for elem in obj["data"].members() {
                            channels.push(elem["name"].to_string());
                        }
                    }
                    "history" => {
                        for elem in obj["data"].members() {
                            let author = elem["author_uuid"].as_u64().unwrap();
                            loaded_messages.push(Message::User {
                                content: elem["content"].to_string().into(),
                                author,
                            });
                        }
                    }
                    "content" => {
                        let author = obj["author_uuid"].as_u64().unwrap();
                        self.add_message(obj["content"].to_string().into(), author);
                    }
                    _ => (),
                }
            } else {
                loaded_messages.push(Message::System(
                    format!("DEBUG (no command): {}", obj.dump()).into(),
                ));
            }
        }
    }
}
