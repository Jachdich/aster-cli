extern crate termion;
extern crate tokio;

use std::collections::HashMap;
use crate::tokio::io::AsyncWriteExt;

use super::Message;
use super::User;
use crate::servernetwork::ServerNetwork;
use super::LocalMessage;


pub struct Server {
    pub loaded_messages: Vec<Message>,
    pub ip: String,
    pub port: u16,
    pub name: String,
    pub channels: Vec<String>,
    pub curr_channel: usize,
    pub peers: HashMap<u64, User>,
    pub uuid: u64,
    pub net: std::option::Option<ServerNetwork>,
}

impl Server {
    pub async fn new(ip: String, port: u16, uuid: u64, idx: usize, tx: std::sync::mpsc::Sender<LocalMessage>) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let net: ServerNetwork = ServerNetwork::new(&ip, port, tx, idx).await?;
        Ok(Server{
            loaded_messages: Vec::new(),
            ip,
            port,
            name: "".to_string(),
            channels: Vec::new(),
            curr_channel: 0,
            peers: HashMap::new(),
            uuid,
            net: Some(net),
        })
    }

    pub fn add_message(&mut self, content: String, nick: String) {

        //todo compile regex once and use it mulyiple times, this is slow as fuck
        let url_regex = r#"^https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)$"#;
        let compiled_regex = regex::Regex::new(url_regex).unwrap();
        if compiled_regex.is_match(&content) {
            //this is gonna be fuckin rough... brace yourselves
        }

        self.loaded_messages.push(
            Message{
                content: format!("{}: {}", nick, content),
            }
        );
    }

    pub fn offline(ip: String, port: u16, name: String, uuid: u64) -> Self {
        Server{
            loaded_messages: Vec::new(),
            ip,
            port,
            name: name,
            channels: Vec::new(),
            curr_channel: 0,
            peers: HashMap::new(),
            uuid,
            net: None,
        }
    }

    pub async fn write(&mut self, data: &[u8]) -> std::result::Result<(), Box<dyn std::error::Error>> {
        match self.net.as_mut() {
            Some(net) => {
                net.write_half.write(data).await?;
                Ok(())
            },
            None => Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotConnected, "Server is offline"))),
        }
    }

    pub async fn update_metadata(&mut self, meta: User) -> std::result::Result<(), Box<dyn std::error::Error>> {
        self.write(format!("/nick {}\n", meta.nick).as_bytes()).await?;
        self.write(format!("/passwd {}\n", meta.passwd).as_bytes()).await?;
        self.write(format!("/pfp {}\n", meta.pfp_b64).as_bytes()).await?;
        Ok(())
    }

    pub async fn initialise(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        if self.uuid == 0 {
            self.write(b"/register\n").await?;
        } else {
            self.write(format!("/login {}\n", self.uuid).as_bytes()).await?;
        }
        self.write(b"/get_icon\n").await?;
        self.write(b"/get_name\n").await?;
        self.write(b"/get_all_metadata\n").await?;
        self.write(b"/get_channels\n").await?;
        self.write(b"/online\n").await?;
        self.write(b"/history 100\n").await?;
        Ok(())
    }

    pub fn to_json(&self) -> json::JsonValue {
        json::object!{
            name: self.name.clone(),
            ip: self.ip.clone(),
            port: self.port,
            uuid: self.uuid,
        }
    }
}
