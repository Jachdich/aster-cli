use crate::gui::GUI;
use termion::event::{Key, MouseEvent, Event, MouseButton};
use super::Focus;
use super::Message;

impl GUI {
    async fn focus_edit_event(&mut self, event: Event) {
        match event {
            Event::Key(Key::Char('\n')) => {
                if self.buffer.len() == 0 {
                    return;
                }

                let res = self.servers[self.curr_server].write(format!("{}\n", self.buffer).as_bytes()).await;
                match res {
                    Ok(_) => {
                        self.servers[self.curr_server].loaded_messages.push(
                            Message {
                                content: format!("{}: {}", self.config["uname"].to_string(), self.buffer)
                        });
                        if self.buffer.chars().nth(0).unwrap() == '/' {
                            self.handle_send_command(self.buffer.clone()).await;
                        }
                        self.buffer = "".to_string();
                    }
                    Err(error) => {
                        self.servers[self.curr_server].loaded_messages.push(
                            Message {
                                content: format!("ERROR: {:?}", error)
                        });
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
        match event {
            Event::Key(Key::Up) => {
                if s.curr_channel > 0 {
                    s.curr_channel -= 1;
                }
            }

            Event::Key(Key::Down) => {
                if s.curr_channel < s.channels.len() - 1 {
                    s.curr_channel += 1;
                }
            }
            _ => (),
        }
        s.write(format!("/join {}\n", s.channels[s.curr_channel]).as_bytes()).await.unwrap();
        let cmd = format!("/join {}", s.channels[s.curr_channel]);
        self.handle_send_command(cmd).await;
    }

    pub async fn handle_keyboard(&mut self, key: Event) -> bool {
        match key.clone() {
             Event::Key(Key::Ctrl('c')) => return false,
             Event::Key(Key::Ctrl('n')) => {
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
         match self.focus {
            Focus::Edit        => self.focus_edit_event(key.clone()).await,
            Focus::ServerList  => self.focus_servers_event(key.clone()).await,
            Focus::ChannelList => self.focus_channels_event(key.clone()).await,
            Focus::Messages => (),
         }
         true
    }
}
