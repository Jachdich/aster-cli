use crate::gui::GUI;
use termion::event::{Key, MouseEvent, Event, MouseButton};
use super::Focus;
use super::Message;
use super::Mode;
use crate::server::Server;

impl GUI {
    async fn focus_edit_event(&mut self, event: Event) {
        match event {
            Event::Key(Key::Char('\n')) => {
                if self.buffer.len() == 0 {
                    return;
                }

                let send_to_server: bool;
                if self.buffer.chars().nth(0).unwrap() == '/' {
                    send_to_server = self.handle_send_command(self.buffer.clone()).await;
                } else {
                    send_to_server = true;
                }

                if send_to_server {
                    let res = self.servers[self.curr_server].write(format!("{}\n", self.buffer).as_bytes()).await;
                    match res {
                        Ok(_) => {
                            self.servers[self.curr_server].loaded_messages.push(
                                Message {
                                    content: format!("{}: {}", self.config["uname"].to_string(), self.buffer)
                            });
                            self.buffer = "".to_string();
                        }
                        Err(error) => {
                            self.servers[self.curr_server].loaded_messages.push(
                                Message {
                                    content: format!("{}System{}: {}", self.theme.messages.system_message, self.theme.messages.text, error)
                            });
                        }
                    }
                } else {
                    self.buffer = "".to_string();
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

            _ => ()
        }
    }
    
    async fn focus_servers_event(&mut self, event: Event) {
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

    async fn focus_channels_event(&mut self, event: Event) {
        let s = &mut self.servers[self.curr_server];
        let reload = match event {
            Event::Key(Key::Up) => {
                if s.curr_channel > 0 {
                    s.curr_channel -= 1;
                    true
                } else {
                    false
                }
            }

            Event::Key(Key::Down) => {
                if s.curr_channel < s.channels.len() - 1 {
                    s.curr_channel += 1;
                    true
                } else {
                    false
                }
            }
            _ => false,
        };
        
        if reload {
            s.write(format!("/join {}\n", s.channels[s.curr_channel]).as_bytes()).await.unwrap();
            let cmd = format!("/join {}", s.channels[s.curr_channel]);
            self.handle_send_command(cmd).await;
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
                Focus::Edit        => self.focus_edit_event(key.clone()).await,
                Focus::ServerList  => self.focus_servers_event(key.clone()).await,
                Focus::ChannelList => self.focus_channels_event(key.clone()).await,
                Focus::Messages => (),
             }
         } else if self.mode == Mode::NewServer {
            match key.clone() {
                Event::Key(Key::Char('\n')) => {
                    if self.sel_idx == 3 {
                        //connect

                        //TODO LOTS OF ERROR HANDLING LOL
                        //this will break at the slight hint of any issue
                        self.servers.push(Server::new(self.ip_buffer.clone(), self.port_buffer.parse::<u16>().unwrap(), self.uuid_buffer.parse::<u64>().unwrap(), self.servers.len(), self.tx.clone()).await.unwrap());
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
                },
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
                Event::Key(Key::Backspace) => {
                    match self.sel_idx {
                        0 => { if self.ip_buffer.len() > 0 { self.ip_buffer.pop(); } }
                        1 => { if self.port_buffer.len() > 0 { self.port_buffer.pop(); } }
                        2 => { if self.uuid_buffer.len() > 0 { self.uuid_buffer.pop(); } }
                        _ => ()
                    }
                }

                Event::Key(Key::Char(c)) => {
                    match self.sel_idx {
                        0 => { self.ip_buffer.push(c) }
                        1 => { self.port_buffer.push(c) }
                        2 => { self.uuid_buffer.push(c) }
                        _ => ()
                    }
                }
                
                _ => ()
            }
         }
         true
    }
}
