use super::Focus;
use super::Mode;
use crate::api;
use crate::gui::GUI;
use crate::prompt::EditBuffer;
use crate::prompt::PromptEvent;
use crate::server::Identification;
use crate::server::Server;
use crate::server::WriteAsterRequest;
use termion::event::{Event, Key, MouseButton, MouseEvent};

impl GUI {
    async fn focus_edit_event(&mut self, event: Event) {
        match event {
            Event::Key(Key::Char('\n')) => {
                if self.buffer.data.len() == 0 {
                    return;
                }

                if self.buffer.data.chars().nth(0).unwrap() == '/' {
                    if let Err(e) = self.handle_send_command(self.buffer.data.clone()).await {
                        self.send_system(e.0.as_str());
                    }
                    self.buffer = EditBuffer::new("".to_string());
                } else if let Some(curr_server) = self.curr_server {
                    let curr_channel_uuid = if let Server::Online {
                        channels,
                        curr_channel: Some(curr_channel),
                        ..
                    } = &self.servers[curr_server]
                    {
                        channels[*curr_channel].uuid
                    } else {
                        self.send_system("Cannot send anything to a nonexistant server!");
                        //TODO fix this!!!
                        return;
                    };

                    if let Server::Online { write_half, .. } = &mut self.servers[curr_server] {
                        let res = write_half
                            .write_request(crate::api::Request::SendRequest {
                                content: self.buffer.data.clone(),
                                channel: curr_channel_uuid,
                            })
                            .await;
                        match res {
                            Ok(_) => {
                                self.buffer = EditBuffer::new("".to_string());
                            }
                            Err(error) => {
                                self.send_system(error.to_string().as_str());
                            }
                        }
                    } else {
                        self.send_system("Cannot send anything to an offline server!");
                    }
                } else {
                    self.send_system("No server is selected you silly goose!");
                }
            }

            Event::Key(Key::Char(ch)) => self.buffer.push(ch),
            Event::Key(Key::Backspace) => self.buffer.pop(),
            Event::Key(Key::Left) => self.buffer.left(),
            Event::Key(Key::Right) => self.buffer.right(),

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
        if let Some(curr_server) = self.curr_server.as_mut() {
            match event {
                Event::Key(Key::Up) => {
                    if *curr_server > 0 {
                        *curr_server -= 1;
                    }
                }

                Event::Key(Key::Down) => {
                    if *curr_server < self.servers.len() - 1 {
                        *curr_server += 1;
                    }
                }
                _ => (),
            }
        } else if self.servers.len() > 0 {
            self.curr_server = Some(0);
        }
    }

    async fn focus_channels_event(&mut self, event: Event) {
        let Some(curr_server) = self.curr_server else {
            return;
        };
        let s = &mut self.servers[curr_server];
        if let Server::Online {
            curr_channel,
            channels,
            loaded_messages,
            write_half,
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
                loaded_messages.clear();
                let channel = channels[curr_channel.unwrap()].uuid;
                let res = write_half
                    .write_request(api::Request::HistoryRequest {
                        num: 1000,
                        channel,
                        before_message: None,
                    })
                    .await;
                if let Err(_) = res {
                    // *s = (*s).to_offline(e.to_string());
                    // TODO make the server offline
                }
            }
        } else {
            panic!("Offline server somehow changed their chanel")
        }
        // unsafe {
        //     let server_ptr = &mut self.servers[curr_server] as *mut Server;
        //     let new_server = (*server_ptr).to_offline("a".into());
        // }
        // self.servers[curr_server] = self.servers[curr_server].to_offline("a".to_string());
    }

    pub async fn handle_keyboard(&mut self, key: Event) -> bool {
        match key.clone() {
            Event::Key(Key::Ctrl('c')) => return false,
            Event::Key(Key::Ctrl('n')) => {}
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
                Focus::ChannelList => self.focus_channels_event(key.clone()).await,
                Focus::Messages => (),
            }
        } else if self.mode == Mode::NewServer {
            let p = self.prompt.as_mut().unwrap();
            match p.handle_event(key) {
                Some(PromptEvent::ButtonPressed("Connect")) => {
                    // TODO LOTS OF ERROR HANDLING LOL
                    // this will break at the slight hint of any issue

                    // The unwraps here are ugly (TODO) but they should be ok, since we define the right types earlier on
                    let id = Identification::Username(p.get_str("Username").unwrap().into());
                    let ip = p.get_str("IP").unwrap().to_owned();
                    let port = p.get_u16("Port").unwrap();
                    self.connect_to_server(ip, port, id).await;
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
