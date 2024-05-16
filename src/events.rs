use super::DisplayMessage;
use super::Focus;
use super::Mode;
use crate::gui::GUI;
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
                self.ip_buffer = "".to_string();
                self.port_buffer = "2345".to_string();
                self.uuid_buffer = "0".to_string();
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
            match key.clone() {
                Event::Key(Key::Char('\n')) => {
                    if self.sel_idx == 3 {
                        //connect

                        //TODO LOTS OF ERROR HANDLING LOL
                        //this will break at the slight hint of any issue
                        self.servers.push(
                            Server::new(
                                self.ip_buffer.clone(),
                                self.port_buffer.parse::<u16>().unwrap(),
                                self.uuid_buffer.parse::<i64>().unwrap(),
                                self.tx.clone(),
                            )
                            .await,
                        );
                        self.servers.last_mut().unwrap().initialise().await.unwrap();
                        self.mode = Mode::Messages;
                    } else if self.sel_idx == 4 {
                        //cancel
                        self.mode = Mode::Messages;
                    } else {
                        if self.sel_idx < 4 {
                            self.sel_idx += 1;
                        }
                    }
                }
                Event::Key(Key::Down) => {
                    if self.sel_idx < 4 {
                        self.sel_idx += 1;
                    }
                }
                Event::Key(Key::Up) => {
                    if self.sel_idx > 0 {
                        self.sel_idx -= 1;
                    }
                }
                Event::Key(Key::Right) => {
                    if self.sel_idx < 4 {
                        self.sel_idx += 1;
                    }
                }
                Event::Key(Key::Left) => {
                    if self.sel_idx > 0 {
                        self.sel_idx -= 1;
                    }
                }
                Event::Key(Key::Backspace) => match self.sel_idx {
                    0 => {
                        if self.ip_buffer.len() > 0 {
                            self.ip_buffer.pop();
                        }
                    }
                    1 => {
                        if self.port_buffer.len() > 0 {
                            self.port_buffer.pop();
                        }
                    }
                    2 => {
                        if self.uuid_buffer.len() > 0 {
                            self.uuid_buffer.pop();
                        }
                    }
                    _ => (),
                },

                Event::Key(Key::Char(c)) => match self.sel_idx {
                    0 => self.ip_buffer.push(c),
                    1 => self.port_buffer.push(c),
                    2 => self.uuid_buffer.push(c),
                    _ => (),
                },

                _ => (),
            }
        }
        true
    }
}
