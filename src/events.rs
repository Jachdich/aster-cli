use super::Focus;
use super::Mode;
use crate::api;
use crate::gui::GUI;
use crate::prompt::EditBuffer;
use crate::prompt::PromptEvent;
use crate::server::Identification;
use crate::server::WriteAsterRequest;
use termion::event::{Event, Key, MouseButton, MouseEvent};

impl GUI {
    async fn send_message_to_server(&mut self, server: usize) {
        let Ok(ref mut net) = self.servers[server].network else {
            self.send_system("This server is offline!");
            return;
        };

        let Some(ch) = net.curr_channel else {
            self.send_system("No channel is selected you silly goose!");
            return;
        };


        let uuid = net.channels[ch].uuid;
        let content = self.buffer.data.clone();
        let res = net
            .write_half
            .write_request(crate::api::Request::SendRequest {
                content,
                channel: uuid,
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
    }

    async fn handle_send_message(&mut self) {
        if self.buffer.data.len() == 0 {
            return;
        }

        if self.buffer.data.chars().nth(0).unwrap() == '/' {
            if let Err(e) = self.handle_send_command(self.buffer.data.clone()).await {
                self.send_system(e.0.as_str());
            }
            self.buffer = EditBuffer::new("".to_string());
        } else if let Some(curr_server) = self.curr_server {
            self.send_message_to_server(curr_server).await;
        } else {
            self.send_system("No server is selected you silly goose!");
        }
    }

    async fn focus_edit_event(&mut self, event: Event) {
        match event {
            Event::Key(Key::Char('\n')) => self.handle_send_message().await,
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

            Event::Mouse(MouseEvent::Press(MouseButton::Left, x, y)) => {
                if x < self.theme.left_margin as u16 + self.theme.channels.border.left.width() // ??
                    && x > self.theme.channels.border.left.width()
                    && y >= self.theme.get_servers_start_pos() as u16
                    && y < self.theme.get_channels_start_pos(self.height) as u16 - 1
                {
                    let idx = y as usize - self.theme.get_servers_start_pos();
                    self.send_system(idx.to_string().as_str());
                    let Some(curr_server) = self.curr_server.map(|idx| &mut self.servers[idx])
                    else {
                        return ();
                    };
                    let reload = if let Ok(ref net) = curr_server.network {
                        idx < net.channels.len() && !net.curr_channel.is_some_and(|c| c == idx)
                    } else {
                        false
                    };
                }
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
        if let Ok(ref mut net) = s.network {
            let reload = match event {
                Event::Key(Key::Up) => {
                    if net.curr_channel.is_some_and(|x| x > 0) {
                        *net.curr_channel.as_mut().unwrap() -= 1;
                        true
                    } else {
                        false
                    }
                }

                Event::Key(Key::Down) => {
                    if net.curr_channel.is_some_and(|x| x < net.channels.len() - 1) {
                        *net.curr_channel.as_mut().unwrap() += 1;
                        true
                    } else if net.curr_channel.is_none() && net.channels.len() > 0 {
                        net.curr_channel = Some(0);
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            };

            if reload {
                net.loaded_messages.clear();
                let channel = net.channels[net.curr_channel.unwrap()].uuid;
                let res = net
                    .write_half
                    .write_request(api::Request::HistoryRequest {
                        num: 100,
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
