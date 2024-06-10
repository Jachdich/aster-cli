use super::Focus;
use super::Mode;
use crate::api;
use crate::gui::GUI;
use crate::prompt::EditBuffer;
use crate::prompt::{Prompt, PromptEvent, PromptField};
use crate::server::Server;
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

                    let res = self.servers[curr_server]
                        .write(crate::api::Request::SendRequest {
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
        let s = &mut self.servers[self.curr_server.unwrap()];
        if let Server::Online {
            curr_channel,
            channels,
            loaded_messages,
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
                s.write(api::Request::HistoryRequest {
                    num: 1000,
                    channel,
                    before_message: None,
                })
                .await;
            }
        } else {
            panic!("Offline server somehow changed their chanel")
        }
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
                    let id =
                        crate::server::Identification::Username(p.get_str("Username").unwrap());

                    // TODO its own function so it can also be called inside the `/connect` command handler
                    self.servers.push(
                        Server::new(
                            p.get_str("IP").unwrap().to_owned(),
                            p.get_u16("Port").unwrap(),
                            self.tx.clone(),
                            self.cancel.subscribe(),
                        )
                        .await,
                    );
                    let conn = self.servers.last_mut().unwrap();
                    if conn.is_online() {
                        conn.initialise(id).await.unwrap();
                    }
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
