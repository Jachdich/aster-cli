extern crate termion;

use crate::api::{Request, Response};
use crate::drawing::Theme;
use crate::prompt::{EditBuffer, Prompt, PromptField};
use crate::server::{Identification, Server};
use crate::Focus;
use crate::LocalMessage;
use crate::Mode;
use fmtstring::FmtString;
use serde::{Deserialize, Serialize};
use std::io::{stdout, Write};
use std::net::SocketAddr;
use std::sync::mpsc::{Receiver, Sender};
use tokio::sync::broadcast;

#[derive(Deserialize, Serialize)]
pub struct Settings {
    pub uname: String,
    pub passwd: String,
    pub pfp: String,
    pub sync_ip: String,
    pub sync_port: u16,
}

pub struct GUI {
    pub scroll: isize,
    pub buffer: EditBuffer,
    pub tx: Sender<LocalMessage>,
    pub servers: Vec<Server>,
    pub curr_server: Option<usize>,
    pub mode: Mode,
    pub focus: Focus,
    pub theme: Theme,
    pub system_message: FmtString,
    pub prompt: Option<Prompt>,
    pub redraw: bool,
    pub width: u16,
    pub height: u16,
    pub cancel: broadcast::Sender<()>,
    pub settings: Settings,
}

#[derive(Debug)]
pub struct CommandError(pub String);

impl GUI {
    pub async fn new(
        tx: std::sync::mpsc::Sender<LocalMessage>,
        cancel: broadcast::Sender<()>,
        settings: Settings,
        servers: Vec<Server>,
    ) -> Self {
        GUI {
            scroll: 0,
            buffer: EditBuffer::new("".into()),
            tx,
            servers,
            curr_server: None,
            mode: Mode::Messages,
            focus: Focus::Edit,
            theme: Theme::new("themes/default.json").unwrap(),
            system_message: FmtString::from_str(""),

            prompt: None,
            redraw: true,
            width: 0,
            height: 0,

            cancel,
            settings,
        }
    }

    pub fn curr_server(&self) -> Option<&Server> {
        self.curr_server.map(|s| &self.servers[s])
    }
    pub fn curr_server_mut(&mut self) -> Option<&mut Server> {
        match self.curr_server {
            Some(server) => Some(&mut self.servers[server]),
            None => None,
        }
    }

    pub fn send_system(&mut self, message: &str) {
        self.system_message = format!("System: {}", message).into();
    }

    pub async fn handle_send_command(&mut self, cmd: String) -> Result<(), CommandError> {
        let argv = cmd.split(" ").collect::<Vec<&str>>();
        match argv[0] {
            "/nick" => {
                self.settings.uname = argv[1].to_owned();
                for server in &mut self.servers {
                    if let Ok(ref mut net) = server.network {
                        net.write(Request::NickRequest {
                            nick: argv[1].to_owned(),
                        })
                        .await
                        .unwrap(); // TODO UNWRAP REEE
                    } else {
                        // TODO: cache this to send later
                    }
                }
                Ok(())
            }

            "/join" => {
                let Some(curr_server) = self.curr_server else {
                    return Err(CommandError(
                        "No server is selected you silly goose!".into(),
                    ));
                };
                let Ok(ref mut net) = &mut self.servers[curr_server].network else {
                    return Err(CommandError(
                        "This server is offline, cannot join any channels!".into(),
                    ));
                };

                net.loaded_messages.clear();

                net.curr_channel =
                    Some(net.channels.iter().position(|r| r.name == argv[1]).ok_or(
                        CommandError(format!(
                            "Channel '{}' does not exist in this server",
                            argv[1]
                        )),
                    )?);

                //It is possible that this unwrap fails due to the time interval since it was last checked. fuck it I cba
                net.write(Request::HistoryRequest {
                    num: 100,
                    channel: net.channels[net.curr_channel.unwrap()].uuid,
                    before_message: None,
                })
                .await
                .unwrap();
                // self.draw_messages();

                Ok(())
            }
            "/theme" => {
                if argv.len() != 2 {
                    self.send_system("Expected exactly one argument");
                }

                match Theme::new(&format!("themes/{}.json", argv[1])) {
                    Ok(theme) => {
                        self.theme = theme;
                        self.send_system(&format!("Changed theme to {}", argv[1]));
                        // all of these potentially need drawing
                        // self.draw_border();
                        // self.draw_messages();
                        // self.draw_servers();
                        // self.draw_status_line();
                        // self.draw_input_buffer();
                    }
                    Err(_e) => self.send_system("Nonexistant or invalid theme provided"),
                }

                Ok(())
            }

            "/connect" => {
                // possibility to connect using [username@]hostname[:port] instead of interactive menu
                if argv.len() == 2 {
                    let (rest, username) = if let Some(splits) = argv[1].rsplit_once('@') {
                        splits
                    } else {
                        (argv[1], self.settings.uname.as_str())
                    };
                    let (port, ip) = if let Some((port, ip)) = rest.rsplit_once(':') {
                        (
                            port.parse::<u16>().map_err(|_| {
                                CommandError("Unable to parse port number".to_owned())
                            })?,
                            ip,
                        )
                    } else {
                        (2345, rest)
                    };
                    let id = Identification::Username(username.to_owned());
                    self.connect_to_server(ip.to_owned(), port, id).await;
                } else {
                    self.mode = Mode::NewServer;
                    self.prompt = Some(Prompt::new(
                        "Add a server",
                        vec![
                            PromptField::String {
                                name: "IP",
                                default: None,
                            },
                            PromptField::U16 {
                                name: "Port",
                                default: Some(2345),
                            },
                            PromptField::String {
                                name: "Username",
                                default: Some(self.settings.uname.clone()),
                            },
                        ],
                        vec!["Connect", "Cancel"],
                    ));
                }

                Ok(())
            }
            _ => Err(CommandError(format!("Unknown command '{}'", argv[0]))),
        }
    }

    pub async fn connect_to_server(&mut self, ip: String, port: u16, id: Identification) {
        let mut conn = Server::new(ip, port, self.tx.clone(), self.cancel.subscribe()).await;
        if let Ok(ref mut net) = conn.network {
            net.initialise(id).await.unwrap();
        }
        self.servers.push(conn);
    }

    pub fn save_config(&mut self) {
        // TODO unwrap bade
        let mut pref_dir = dirs::preference_dir().unwrap();
        pref_dir.push("aster-cli");
        std::fs::create_dir_all(pref_dir.as_path());
        pref_dir.push("preferences.json");
        let mut file = std::fs::File::create(pref_dir).unwrap();
        let server_list = serde_json::to_value(&self.servers).unwrap();
        let mut prefs = serde_json::to_value(&self.settings).unwrap();
        prefs["servers"] = server_list;
        file.write_all(prefs.to_string().as_bytes()).unwrap();
    }

    pub fn get_server_by_addr(&mut self, addr: SocketAddr) -> Option<&mut Server> {
        self.servers.iter_mut().find(|server| match server.network {
            Ok(ref net) => net.remote_addr == addr,
            Err(_) => false,
        })
    }
}
