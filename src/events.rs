use super::DisplayMessage;
use super::Focus;
use super::Mode;
use crate::gui::GUI;
use crate::prompt::{Prompt, PromptEvent, PromptField};
use crate::server::Server;
use termion::event::{Event, Key, MouseButton, MouseEvent};

impl GUI {
    async fn focus_edit_event(&mut self, event: Event) {
        match event {
            Event::Key(Key::Char('\n')) => {
                if self.buffer.len() == 0 {
                    return;
                }

                if self.buffer.chars().nth(0).unwrap() == '/' {
                    self.handle_send_command(self.buffer.clone()).await.unwrap(); // TODO set this as system message
                    self.buffer = "".to_string();
                } else {
                    let curr_channel_uuid = if let Server::Online {
                        channels,
                        curr_channel,
                        ..
                    } = &self.servers[self.curr_server]
                    {
                        channels[curr_channel.unwrap()].uuid //TODO fix this!!!
                    } else {
                        panic!("Cannot send anything to a nonexistant server!")
                    };

                    let res = self.servers[self.curr_server]
                        .write(crate::api::Request::SendRequest {
                            content: self.buffer.clone(),
                            channel: curr_channel_uuid,
                        })
                        .await;
                    match res {
                        Ok(_) => {
                            // self.servers[self.curr_server]
                            //     .loaded_messages
                            //     .push(Message::System(
                            //         format!(
                            //             "{}: {}",
                            //             self.config["uname"].to_string(),
                            //             self.buffer
                            //         )
                            //         .into(),
                            //     ));
                            self.buffer = "".to_string();
                        }
                        Err(error) => {
                            self.send_system(
                                format!(
                                    "{}System{}: {}",
                                    self.theme.messages.system_message,
                                    self.theme.messages.text,
                                    error
                                )
                                .as_str(),
                            );
                        }
                    }
                }
            }

            Event::Key(Key::Char(ch)) => {
                self.buffer.push(ch);
            }

            Event::Key(Key::Backspace) => {
                self.buffer.pop();
            }

            Event::Mouse(MouseEvent::Press(MouseButton::WheelUp, _, _)) => {
                self.scroll -= 1;
            }

            Event::Mouse(MouseEvent::Press(MouseButton::WheelDown, _, _)) => {
                self.scroll += 1;
            }

            _ => (),
        }
    }

    fn focus_servers_event(&mut self, event: Event) {
        match event {
            Event::Key(Key::Up) => {
                if self.curr_server > 0 {
                    self.curr_server -= 1;
                }
            }

            Event::Key(Key::Down) => {
                if self.curr_server < self.servers.len() - 1 {
                    self.curr_server += 1;
                }
            }
            _ => (),
        }
    }

    fn focus_channels_event(&mut self, event: Event) {
        let s = &mut self.servers[self.curr_server];
        if let Server::Online {
            curr_channel,
            channels,
            ..
        } = s
        {
            let reload = match event {
                Event::Key(Key::Up) => {
                    if curr_channel.is_some_and(|x| x > 0) {
                        *curr_channel.as_mut().unwrap() -= 1;
                        true
                    } else {
                        false
                    }
                }

                Event::Key(Key::Down) => {
                    if curr_channel.is_some_and(|x| x < channels.len() - 1) {
                        *curr_channel.as_mut().unwrap() += 1;
                        true
                    } else if curr_channel.is_none() && channels.len() > 0 {
                        *curr_channel = Some(0);
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            };

            if reload {
                // s.write(format!("/join {}\n", s.channels[s.curr_channel]).as_bytes())
                //     .await
                //     .unwrap();
                // let cmd = format!("/join {}", s.channels[s.curr_channel]);
                // self.handle_send_command(cmd).await;
            }
        } else {
            panic!("Offline server somehow changed their chanel")
        }
    }

    pub async fn handle_keyboard(&mut self, key: Event) -> bool {
        match key.clone() {
            Event::Key(Key::Ctrl('c')) => return false,
            Event::Key(Key::Ctrl('n')) => {
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
                        PromptField::I64 {
                            name: "UUID",
                            default: None,
                        },
                    ],
                    vec!["Connect", "Cancel"],
                ));
            }
            Event::Key(Key::Alt('c')) => {
                self.focus = Focus::ChannelList;
            }
            Event::Key(Key::Alt('s')) => {
                self.focus = Focus::ServerList;
            }
            Event::Key(Key::Alt('e')) => {
                self.focus = Focus::Edit;
            }
            _ => (),
        }
        if self.mode == Mode::Messages {
            match self.focus {
                Focus::Edit => self.focus_edit_event(key.clone()).await,
                Focus::ServerList => self.focus_servers_event(key.clone()),
                Focus::ChannelList => self.focus_channels_event(key.clone()),
                Focus::Messages => (),
            }
        } else if self.mode == Mode::NewServer {
            let p = self.prompt.as_mut().unwrap();
            match p.handle_event(key) {
                Some(PromptEvent::ButtonPressed("Connect")) => {
                    // TODO LOTS OF ERROR HANDLING LOL
                    // this will break at the slight hint of any issue
                    // The unwraps here are ugly (TODO) but they should be ok, since we define the right types earlier on
                    self.servers.push(
                        Server::new(
                            p.get_str("IP").unwrap().to_owned(),
                            p.get_u16("Port").unwrap(),
                            p.get_i64("UUID").unwrap(),
                            self.tx.clone(),
                        )
                        .await,
                    );
                    self.servers.last_mut().unwrap().initialise().await.unwrap();
                    self.mode = Mode::Messages;
                }
                Some(PromptEvent::ButtonPressed("Cancel")) => {
                    self.mode = Mode::Messages;
                    self.prompt = None;
                }
                Some(PromptEvent::ButtonPressed(_)) => unreachable!(), // no idea
                None => (),
            }
        }
        true
    }
}
