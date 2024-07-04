use super::Focus;
use super::Mode;
use crate::gui::GUI;
use crate::prompt::EditBuffer;
use crate::prompt::PromptEvent;
use crate::server::Identification;
use crate::server::WriteAsterRequestAsync;
use termion::event::{Event, Key, MouseButton, MouseEvent};

impl GUI {
    async fn send_message_to_server(&mut self, server: usize) -> Result<(), String> {
        let net = self.servers[server]
            .network
            .as_mut()
            .map_err(|e| e.to_owned())?;
        let ch = net
            .curr_channel
            .ok_or("No channel is selected you silly goose!")?;

        let uuid = net.channels[ch].uuid;
        let content = self.buffer.data.clone();
        net.write_half
            .write_request(crate::api::Request::SendRequest {
                content,
                channel: uuid,
            })
            .await
            .map_err(|e| e.to_string())?;

        self.buffer = EditBuffer::new("".to_string());
        Ok(())
    }

    async fn handle_send_message(&mut self) {
        if self.mode == Mode::EditMessage {
            if self.buffer.data.len() == 0 {
                // delete message
            } else {
                if let Err(e) = self.edit_message(self.buffer.data.clone()).await {
                    self.send_system(&e.0);
                }
            }
            self.mode = Mode::Messages;
            self.buffer = EditBuffer::new("".to_string());
            self.selected_message = None;
        } else {
            if self.buffer.data.len() == 0 {
                return;
            }

            if self.buffer.data.chars().nth(0).unwrap() == '/' {
                // clear edit buffer before executing command, in case command modifies the buffer
                let command = self.buffer.data.clone();
                self.buffer = EditBuffer::new("".to_string());

                if let Err(e) = self.handle_send_command(command).await {
                    self.send_system(e.0.as_str());
                }
            } else if let Some(curr_server) = self.curr_server {
                if let Err(e) = self.send_message_to_server(curr_server).await {
                    self.send_system(&e);
                }
            } else {
                self.send_system("No server is selected you silly goose!");
            }
        }
    }

    fn select_message_up(&mut self) {
        // TODO modify scroll if goes offscreen
        self.selected_message = match self.selected_message {
            None => Some(1),
            Some(n) => Some(n + 1),
        }
    }
    fn select_message_down(&mut self) {
        self.selected_message = match self.selected_message {
            Some(1) => None,
            Some(n) => Some(n - 1),
            None => None,
        }
    }

    async fn focus_edit_event(&mut self, event: Event) {
        match event {
            Event::Key(Key::Char('\n')) => self.handle_send_message().await,
            Event::Key(Key::Char(ch)) => self.buffer.push(ch),
            Event::Key(Key::Backspace) => self.buffer.pop(),
            Event::Key(Key::Ctrl('h')) => self.buffer.pop_word(),
            Event::Key(Key::Left) => self.buffer.left(),
            Event::Key(Key::Right) => self.buffer.right(),

            Event::Mouse(MouseEvent::Press(MouseButton::WheelUp, _, _)) => {
                self.scroll -= 1;
            }

            Event::Mouse(MouseEvent::Press(MouseButton::WheelDown, _, _)) => {
                self.scroll += 1;
            }

            Event::Key(Key::Up) => self.select_message_up(),
            Event::Key(Key::Down) => self.select_message_down(),
            Event::Key(Key::Esc) if self.mode == Mode::EditMessage => {
                self.mode = Mode::Messages;
                self.buffer = EditBuffer::new("".to_string());
                self.selected_message = None;
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
        let net = s
            .network
            .as_mut()
            .expect("Offline server somehow changed their channel");
        let switching_channel = match event {
            Event::Key(Key::Up) => {
                if net.curr_channel.is_some_and(|x| x > 0) {
                    Some(net.curr_channel.unwrap() - 1)
                } else {
                    None
                }
            }

            Event::Key(Key::Down) => {
                if net.curr_channel.is_some_and(|x| x < net.channels.len() - 1) {
                    Some(net.curr_channel.unwrap() + 1)
                } else if net.curr_channel.is_none() && net.channels.len() > 0 {
                    Some(0)
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some(idx) = switching_channel {
            net.switch_channel(idx).await;
        }
    }

    pub async fn focus_any_event(&mut self, event: Event) {
        match event {
            Event::Mouse(MouseEvent::Press(MouseButton::Left, x, y)) => {
                if x >= self.theme.sidebar_width as u16
                    + self.theme.channels.border.left.width()
                    + self.theme.channels.border.right.width()
                    || x <= self.theme.channels.border.left.width()
                {
                    return;
                }

                if y >= self.theme.get_servers_start_pos() as u16
                    && y < self.theme.get_channels_start_pos(self.height) as u16 - 1
                {
                    let idx = y as usize - self.theme.get_servers_start_pos();
                    let Some(curr_server) = self.curr_server.map(|idx| &mut self.servers[idx])
                    else {
                        return ();
                    };
                    if let Ok(ref mut net) = curr_server.network {
                        if idx < net.channels.len() && !net.curr_channel.is_some_and(|c| c == idx) {
                            net.switch_channel(idx).await;
                        }
                    }
                } else if y >= self.theme.get_channels_start_pos(self.height) as u16
                    && y < self.height - self.theme.servers.border.bottom.width()
                {
                    let idx = y as usize - self.theme.get_channels_start_pos(self.height);
                    if idx < self.servers.len() {
                        self.curr_server = Some(idx);
                    }
                }
            }
            _ => (),
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
        if self.mode == Mode::Messages || self.mode == Mode::EditMessage {
            match self.focus {
                Focus::Edit => self.focus_edit_event(key.clone()).await,
                Focus::ServerList => self.focus_servers_event(key.clone()),
                Focus::ChannelList => self.focus_channels_event(key.clone()).await,
                Focus::Messages => (),
            }
            self.focus_any_event(key.clone()).await;
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
