extern crate termion;

use crate::api::Request;
use crate::drawing::Theme;
use crate::prompt::{EditBuffer, Prompt, PromptField};
use crate::server::{Identification, LoadedMessage, Server};
use crate::Focus;
use crate::LocalMessage;
use crate::Mode;
use crate::Settings;
use std::io::Write;
use std::net::SocketAddr;
use std::sync::mpsc::Sender;
use tokio::sync::broadcast;

pub struct Gui {
    pub scroll: isize,
    pub buffer: EditBuffer,
    pub tx: Sender<LocalMessage>,
    pub servers: Vec<Server>,
    pub curr_server: Option<usize>,
    pub mode: Mode,
    pub focus: Focus,
    pub theme: Theme,
    pub system_message: String,
    pub prompt: Option<Prompt>,
    pub width: u16,
    pub height: u16,
    pub cancel: broadcast::Sender<()>,
    pub settings: Settings,
    pub selected_message: Option<usize>,
}

#[derive(Debug)]
pub struct CommandError(pub String);

impl Gui {
    pub async fn new(
        tx: std::sync::mpsc::Sender<LocalMessage>,
        cancel: broadcast::Sender<()>,
        settings: Settings,
        servers: Vec<Server>,
    ) -> Self {
        Gui {
            scroll: 0,
            buffer: EditBuffer::new("".into()),
            tx,
            servers,
            curr_server: None,
            mode: Mode::Messages,
            focus: Focus::Edit,
            theme: Theme::new(&settings.theme).unwrap(),
            system_message: "".into(),

            prompt: None,
            width: 0,
            height: 0,

            cancel,
            settings,
            selected_message: None,
        }
    }

    pub fn send_system(&mut self, message: &str) {
        self.system_message = format!("System: {}", message);
    }

    fn get_selected_message(&self) -> Result<&LoadedMessage, CommandError> {
        let Some(selected_message) = self.selected_message else {
            return Err(CommandError("No message selected to edit!".to_string()));
        };
        let Some(server) = self.curr_server.map(|x| &self.servers[x]) else {
            return Err(CommandError("No server selected!".to_string()));
        };
        let Ok(ref net) = server.network else {
            return Err(CommandError("This server is offline!".to_string()));
        };
        Ok(&net.loaded_messages[net.loaded_messages.len() - selected_message])
    }

    pub async fn edit_message(&mut self, new_content: String) -> Result<(), CommandError> {
        let uuid = self.get_selected_message()?.message.uuid;
        let packet = Request::Edit {
            message: uuid,
            new_content,
        };
        // TODO code dupe...
        let Some(server) = self.curr_server.map(|x| &mut self.servers[x]) else {
            return Err(CommandError("No server selected!".to_string()));
        };
        let Ok(ref mut net) = server.network else {
            return Err(CommandError("This server is offline!".to_string()));
        };
        net.write(packet)
            .await
            .map_err(|e| CommandError(format!("Error sending edit packet: {}", e)))?;
        self.buffer = EditBuffer::new("".to_string());
        self.selected_message = None;
        Ok(())
    }

    pub async fn delete_message(&mut self) -> Result<(), CommandError> {
        // TODO MASSIVE code dupe...
        let uuid = self.get_selected_message()?.message.uuid;
        let packet = Request::Delete { message: uuid };
        let Some(server) = self.curr_server.map(|x| &mut self.servers[x]) else {
            return Err(CommandError("No server selected!".to_string()));
        };
        let Ok(ref mut net) = server.network else {
            return Err(CommandError("This server is offline!".to_string()));
        };
        net.write(packet)
            .await
            .map_err(|e| CommandError(format!("Error sending delete packet: {}", e)))?;
        self.selected_message = None;
        Ok(())
    }

    pub async fn handle_send_command(&mut self, cmd: String) -> Result<(), CommandError> {
        let argv = cmd.split(' ').collect::<Vec<&str>>();
        match argv[0] {
            "/e" | "/edit" => match cmd.split_once(' ') {
                Some((_, content)) => self.edit_message(content.to_owned()).await,
                None => {
                    self.mode = Mode::EditMessage;
                    self.buffer = EditBuffer::new(
                        self.get_selected_message().unwrap().message.content.clone(),
                    ); // TODO save buffer
                    self.send_system(&self.buffer.data.clone());
                    Ok(())
                }
            },
            "/d" | "/delete" => self.delete_message().await,
            "/nick" => {
                argv[1].clone_into(&mut self.settings.uname);
                for server in &mut self.servers {
                    if let Ok(ref mut net) = server.network {
                        net.write(Request::Nick {
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
                net.write(Request::History {
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

                match Theme::new(argv[1]) {
                    Ok(theme) => {
                        self.theme = theme;
                        argv[1].clone_into(&mut self.settings.theme);
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
                    let (port, ip) = if let Some((ip, port)) = rest.rsplit_once(':') {
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
        let mut conn = Server::new(
            ip,
            port,
            id.clone(),
            self.settings.passwd.clone(),
            self.tx.clone(),
            self.cancel.subscribe(),
        )
        .await;
        if let Ok(ref mut net) = conn.network {
            net.initialise(id, self.settings.passwd.clone())
                .await
                .unwrap();
        }
        self.servers.push(conn);
    }

    pub fn save_config(&mut self) {
        // TODO unwrap bade
        let mut pref_dir = dirs::preference_dir().unwrap();
        pref_dir.push("aster-cli");
        std::fs::create_dir_all(pref_dir.as_path()).expect(
            "Unable to create the directory to put the config file in! Do you have permission?",
        );
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
