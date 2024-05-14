extern crate termion;
extern crate tokio;

use crate::tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use crate::FmtString;
use std::collections::HashMap;
use tokio::io::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio_native_tls::TlsStream;

use super::Message;
use super::User;
use json::{object, JsonValue};

pub enum Server {
    Online {
        loaded_messages: Vec<Message>,
        ip: String,
        port: u16,
        name: String,
        channels: Vec<String>,
        curr_channel: usize,
        peers: HashMap<u64, User>,
        uuid: u64,
        write_half: WriteHalf<TlsStream<TcpStream>>,
    },
    Offline {
        ip: String,
        port: u16,
        name: String,
        uuid: u64,
        offline_reason: String,
    },
}

impl Server {
    pub async fn new(ip: String, port: u16, uuid: u64, idx: usize) -> Self {
        Server {
            loaded_messages: Vec::new(),
            ip,
            port,
            name: "".to_string(),
            channels: Vec::new(),
            curr_channel: 0,
            peers: HashMap::new(),
            uuid,
            write_half: None,
        }
    }

    pub fn add_message(&mut self, content: FmtString, author: u64) {
        //todo compile regex once and use it mulyiple times, this is slow as fuck
        //let url_regex = r#"^https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)$"#;
        //let compiled_regex = regex::Regex::new(url_regex).unwrap();
        //if compiled_regex.is_match(&content) {
        //this is gonna be fuckin rough... brace yourselves
        //}

        self.loaded_messages.push(Message::User { content, author });
    }

    pub async fn write(&mut self, value: JsonValue) -> std::result::Result<usize, std::io::Error> {
        match self.write_half.as_mut() {
            Some(write_half) => write_half.write(&value.dump().into_bytes()).await,
            None => Err(std::io::Error::new(
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
        if self.uuid == 0 {
            self.write(object! {"command": "register"}).await?;
        } else {
            self.write(object! {"command": "login", "uuid": self.uuid, "passwd": "a"})
                .await?;
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

    pub fn to_json(&self) -> json::JsonValue {
        json::object! {
            name: self.name.clone(),
            ip: self.ip.clone(),
            port: self.port,
            uuid: self.uuid,
        }
    }

    pub async fn init_network(
        &self,
        ip: &str,
        port: u16,
    ) -> std::result::Result<(), std::io::Error> {
        let addr = format!("{}:{}", ip, port);

        let socket = TcpStream::connect(&addr).await?;
        // let cx = TlsConnector::builder()
        //     .danger_accept_invalid_certs(true)
        //     .build()?;
        // let cx = tokio_native_tls::TlsConnector::from(cx);

        // let socket = cx.connect(ip, socket).await?;
        let (read_half, write_half) = tokio::io::split(socket);

        let net_tx = tx.clone();
        std::thread::spawn(move || {
            futures::executor::block_on(Self::run_network(net_tx, read_half, idx));
        });
        Ok(())
    }

    async fn run_network(tx: stream: ReadHalf<TlsStream<TcpStream>>) {
        let mut reader = tokio::io::BufReader::new(stream);

        loop {
            let mut result: String = "".to_string();
            match reader.read_line(&mut result).await {
                Ok(_len) => {
                    self.handle_network_packet(result);
                }
                Err(e) => {
                    println!(
                        "Error occurred in network thread, ip: {}:{}: {:?}",
                        self.ip, self.port, e
                    );
                    return;
                }
            }
        }
    }

    pub fn handle_network_packet(&mut self, obj: json::JsonValue) {
        if let Self::Online { peers, uuid, name, loaded_messages, channels, .. } = self {
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
                            *uuid = obj["value"].as_u64().unwrap();
                        }
                        _ => (),
                    }, /*
                    "get_icon" => {
                    s.icon*/
                    "get_name" => {
                        *name = obj["data"].to_string();
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
