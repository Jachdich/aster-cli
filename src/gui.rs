extern crate termion;

use super::api::{Message, Request, Response, Status, User};
use super::DisplayMessage;
use super::Focus;
use super::LocalMessage;
use super::Mode;
use crate::drawing::Theme;
use crate::prompt::{Prompt, PromptField};
use crate::server::Server;
use fmtstring::FmtString;
use std::io::{stdout, Write};
use std::net::SocketAddr;
use std::sync::mpsc::{Receiver, Sender};
use termion::raw::IntoRawMode;
use tokio::sync::broadcast;

pub struct GUI {
    pub scroll: isize,
    pub buffer: String,
    pub tx: Sender<LocalMessage>,
    pub rx: Receiver<LocalMessage>,
    pub servers: Vec<Server>, // FEAT server folders
    pub curr_server: Option<usize>,
    pub mode: Mode,
    pub focus: Focus,
    pub screen: termion::raw::RawTerminal<termion::input::MouseTerminal<std::io::Stdout>>,
    pub theme: Theme,
    pub border_buffer: String,
    pub draw_border: bool,
    pub system_message: FmtString,
    pub prompt: Option<Prompt>,
    pub redraw: bool,
    pub last_w: u16,
    pub last_h: u16,
    pub cancel: broadcast::Sender<()>,
}

#[derive(Debug)]
pub struct CommandError(pub String);

impl GUI {
    pub async fn new(
        tx: std::sync::mpsc::Sender<LocalMessage>,
        rx: std::sync::mpsc::Receiver<LocalMessage>,
        cancel: broadcast::Sender<()>,
    ) -> Self {
        // let default_config: json::JsonValue = json::object! {
        //     servers: [],
        //     uname: "",
        //     passwd: "",
        //     pfp: "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",
        // };
        // let preferences_path = "preferences.json"; //dirs::preference_dir();
        // let contents = std::fs::read_to_string(preferences_path);
        // let config: json::JsonValue;
        // match contents {
        //     Ok(contents) => match json::parse(&contents) {
        //         Ok(value) => {
        //             config = value;
        //         }
        //         Err(_) => {
        //             config = default_config;
        //         }
        //     },
        //     Err(_) => {
        //         config = default_config;
        //     }
        // }

        let mut servers: Vec<Server> = Vec::new();

        // for serv in config["servers"].members() {
        //     let conn = Server::new(
        //         serv["ip"].to_string(),
        //         serv["port"].as_u16().unwrap(),
        //         serv["uuid"].as_u64().unwrap(),
        //         tx.clone(),
        //     )
        //     .await;
        //     servers.push(conn);
        //     // conn.initialise();
        // }

        let stdout = stdout();
        //let mut screen = stdout();
        let screen = termion::input::MouseTerminal::from(stdout)
            .into_raw_mode()
            .unwrap();

        GUI {
            scroll: 0,
            buffer: "".to_string(),
            tx,
            rx,
            servers,
            curr_server: None,
            mode: Mode::Messages,
            focus: Focus::Edit,
            screen,
            theme: Theme::new("themes/default.json").unwrap(),
            border_buffer: String::new(),
            draw_border: true,

            system_message: FmtString::from_str(""),

            prompt: None,
            redraw: true,
            last_w: 0,
            last_h: 0,

            cancel,
        }
    }

    pub fn send_system(&mut self, message: &str) {
        let mut coloured_string: FmtString = format!("System: {}", message).into();
        // for n in 0..6 {
        //     coloured_string[n].fg = self.theme.messages.system_message.fg.clone();
        //     coloured_string[n].bg = self.theme.messages.system_message.bg.clone();
        // }
        // coloured_string[7].fg = self.theme.messages.text.fg.clone();
        // coloured_string[7].bg = self.theme.messages.text.bg.clone();

        self.system_message = coloured_string;
    }

    pub async fn handle_send_command(&mut self, cmd: String) -> Result<(), CommandError> {
        let argv = cmd.split(" ").collect::<Vec<&str>>();
        match argv[0] {
            "/nick" => {
                // self.config["uname"] = argv[1].into();
                // TODO send to all servers
                Ok(())
            }

            "/join" => {
                // TODO this is awful. Fix it.
                if let Some(curr_server) = self.curr_server {
                    let channel_uuid = if let Server::Online {
                        loaded_messages,
                        channels,
                        curr_channel,
                        ..
                    } = &mut self.servers[curr_server]
                    {
                        loaded_messages.clear();

                        *curr_channel =
                            Some(channels.iter().position(|r| r.name == argv[1]).ok_or(
                                CommandError(format!(
                                    "Channel '{}' does not exist in this server",
                                    argv[1]
                                )),
                            )?);
                        channels[curr_channel.unwrap()].uuid
                    } else {
                        return Err(CommandError(
                            "This server is offline, cannot join any channels!".into(),
                        ));
                    };

                    //It is possible that this unwrap fails due to the time interval since it was last checked. fuck it I cba
                    self.servers[curr_server]
                        .write(Request::HistoryRequest {
                            num: 100,
                            channel: channel_uuid,
                            before_message: None,
                        })
                        .await
                        .unwrap();
                } else {
                    return Err(CommandError(
                        "No server is selected you silly goose!".into(),
                    ));
                }

                Ok(())
            }
            "/theme" => {
                if argv.len() != 2 {
                    self.send_system("Expected exactly one argument");
                }

                match Theme::new(&format!("themes/{}.json", argv[1])) {
                    Ok(theme) => {
                        self.theme = theme;
                        // self.send_system(&format!("Changed theme to {}", argv[1]));
                    }
                    Err(_e) => self.send_system("Nonexistant or invalid theme provided"),
                }

                Ok(())
            }
            _ => Err(CommandError(format!("Unknown command '{}'", argv[0]))),
        }
    }

    fn save_config(&mut self) {
        // TODO unwrap bade
        let mut file = std::fs::File::create("preferences.json").unwrap();
        file.write_all(serde_json::to_string(&self.servers).unwrap().as_bytes())
            .unwrap();
    }

    fn get_server_by_addr(&mut self, addr: SocketAddr) -> Option<&mut Server> {
        self.servers.iter_mut().find(|server| match server {
            Server::Online { remote_addr, .. } => *remote_addr == addr,
            Server::Offline { .. } => false,
        })
    }

    pub async fn run_gui(&mut self) {
        self.draw_screen();
        self.screen.flush().unwrap();

        loop {
            match self.rx.recv().unwrap() {
                LocalMessage::Keyboard(key) => {
                    if !self.handle_keyboard(key).await {
                        self.save_config();
                        self.cancel.send(());
                        return;
                    }
                }

                LocalMessage::Network(msg, addr) => {
                    let obj: Result<serde_json::Value, serde_json::Error> =
                        serde_json::from_str(&msg);
                    match obj {
                        Ok(obj) => {
                            let status: Status =
                                serde_json::from_value(obj["status"].clone()).unwrap();
                            if status != Status::Ok {
                                panic!("We likely did something wrong, non-ok status: {:?}", obj)
                            }
                            let response: Response = serde_json::from_value(obj).unwrap();
                            self.get_server_by_addr(addr)
                                .expect("Network packet recv'd for offline server")
                                .handle_network_packet(response);
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
