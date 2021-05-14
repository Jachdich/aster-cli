use super::Mode;
use super::Focus;
use super::LocalMessage;
use crate::server::Server;
use super::Message;
use super::User;

extern crate termion;
use termion::raw::IntoRawMode;
use std::io::{Write, stdout};

use crate::drawing::Theme;

pub struct GUIBounds {
    pub left_margin: usize,
}

pub struct GUI {
    pub scroll: isize,
    pub buffer: String,
    pub tx: std::sync::mpsc::Sender<LocalMessage>,
    pub rx: std::sync::mpsc::Receiver<LocalMessage>,
    pub config: json::JsonValue,
    pub servers: Vec<Server>,
    pub curr_server: usize,
    pub mode: Mode,
    pub focus: Focus,
    pub screen: termion::raw::RawTerminal<termion::input::MouseTerminal<std::io::Stdout>>,
    pub bounds: GUIBounds,
    pub theme: Theme,

    pub sel_idx: usize,
    pub ip_buffer: String,
    pub port_buffer: String,
    pub uuid_buffer: String,
}

impl GUI {
    pub async fn new(tx: std::sync::mpsc::Sender<LocalMessage>, rx: std::sync::mpsc::Receiver<LocalMessage>) -> Self {
        let default_config: json::JsonValue = json::object!{
            servers: [],
            uname: "",
            passwd: "",
            pfp: "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",
        };
        let contents = std::fs::read_to_string("preferences.json");
        let config: json::JsonValue;
        match contents {
            Ok(contents) => {
                match json::parse(&contents) {
                    Ok(value) => {
                        config = value;
                    }
                    Err(_) => {
                        config = default_config;
                    }
                }
            }
            Err(_) => {
                config = default_config;
            }
        }

        let mut servers: Vec<Server> = Vec::new();

        for serv in config["servers"].members() {
            let conn = Server::new(serv["ip"].to_string(), serv["port"].as_u16().unwrap(), serv["uuid"].as_u64().unwrap(), servers.len(), tx.clone()).await;
            match conn {
                Ok(mut conn) => {
                    let res = conn.initialise().await;
                    match res {
                        Ok(()) => {
                            servers.push(conn);
                        },
                        Err(_error) => {
                            servers.push(Server::offline(serv["ip"].to_string(), serv["port"].as_u16().unwrap(), serv["name"].to_string(), serv["uuid"].as_u64().unwrap()));
                        }
                    }
                },
                Err(_error) => {
                    servers.push(Server::offline(serv["ip"].to_string(), serv["port"].as_u16().unwrap(), serv["name"].to_string(), serv["uuid"].as_u64().unwrap()));
                }
            }
        }

        let stdout = stdout();
        //let mut screen = stdout();
    	let screen = termion::input::MouseTerminal::from(stdout).into_raw_mode().unwrap();
    	
        GUI {
            scroll: 0,
            buffer: "".to_string(),
            tx,
            rx,
            config,
            servers,
            curr_server: 0,
            mode: Mode::Messages,
            focus: Focus::Edit,
            screen,
            bounds: GUIBounds { left_margin: 24 },
            theme: Theme::new(),

            sel_idx: 0,
            ip_buffer: "".to_string(),
            port_buffer: "".to_string(),
            uuid_buffer: "".to_string(),
        }
    }

    pub async fn handle_send_command(&mut self, cmd: String) {
        let argv = cmd.split(" ").collect::<Vec<&str>>();
        match argv[0] {
            "/nick" => {
                self.config["uname"] = argv[1].into();
            }

            "/join" => {
                self.servers[self.curr_server].loaded_messages.clear();
                //It is possible that this unwrap fails due to the time interval since it was last checked. fuck it I cba
                self.servers[self.curr_server].write(b"/history 100\n").await.unwrap();
                self.servers[self.curr_server].curr_channel = 
                    self.servers[self.curr_server].channels.iter().position(|r| r == argv[1]).unwrap();
                    //shitty inefficient code lol
            }
            _ => {}
        }
    }

    fn handle_network_packet(&mut self, obj: json::JsonValue, serv: usize) {
        let s = &mut self.servers[serv];
        if !obj["content"].is_null() {
            let nick = s.peers[&obj["author_uuid"].as_u64().unwrap()].nick.clone();
            //let nick = format!("{:?}", s.peers[&obj["author_uuid"].as_u64().unwrap()]);
            s.loaded_messages.push(
            Message{
                content: format!("{}: {}",
                    nick,
                    obj["content"].to_string()),
            });
        } else if !obj["command"].is_null() {
            match obj["command"].to_string().as_str() {
                "metadata" => {
                    for elem in obj["data"].members() {
                        let elem_uuid = elem["uuid"].as_u64().unwrap();
                        if !s.peers.contains_key(&elem_uuid) {
                            s.peers.insert(elem_uuid, User::from_json(elem));
                        } else {
                            s.peers.get_mut(&elem_uuid).unwrap().update(elem);
                        }
                    }
                },
                "set" => {
                    match obj["key"].to_string().as_str() {
                        "self_uuid" => {
                            s.uuid = obj["value"].as_u64().unwrap();
                        }
                        _ => ()
                    }
                },/*
                "get_icon" => {
                    s.icon*/
                "get_name" => {
                    s.name = obj["data"].to_string();
                },
                
                "get_channels" => {
                    s.channels.clear();
                    for elem in obj["data"].members() {
                        s.channels.push(elem.to_string());
                    }
                },
                _ => ()
            }
        } else if !obj["history"].is_null() {
            for elem in obj["history"].members() {
                s.loaded_messages.push(Message{
                    content: format!("{}: {}", 
                        s.peers[&elem["author_uuid"].as_u64().unwrap()].nick,
                        elem["content"].to_string())});
            }
        } else {
            s.loaded_messages.push(
            Message{
                content: format!("DEBUG: {}", obj.dump()),
            });
        }
    }

    fn save_config(&mut self) {
        //TODO unwraps BADE
        self.config["servers"] = json::array![];
        for server in &self.servers {
            self.config["servers"].push(server.to_json()).unwrap();
        }
        let mut file = std::fs::File::create("preferences.json").unwrap();
        file.write_all(self.config.dump().as_bytes()).unwrap();
    }

    pub async fn run_gui(&mut self) {
        self.draw_screen();
        self.screen.flush().unwrap();

        loop {   	      
    	    match self.rx.recv().unwrap() {
    	        LocalMessage::Keyboard(key) => {
    	            if !self.handle_keyboard(key).await {
    	                self.save_config();
                        return;
    	            }
    	        }

    	        LocalMessage::Network(msg, idx) => {
    	            let obj = json::parse(&msg);
       	            match obj {
       	                Ok(obj) => {
       	                    self.handle_network_packet(obj, idx);
       	                }
       	                Err(_) => {
       	                    //ignore for now
       	                }
       	            }
    	        }
    	    }
    	    self.draw_screen();
    	    self.screen.flush().unwrap();
    	}
    }
}
