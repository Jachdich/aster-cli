extern crate termion;
use std::io::Write;

use crate::gui::GUI;
use super::Mode;

fn centred(text: &str, width: usize) -> String {
    format!("{: ^1$}", text, width)
}

pub struct Theme {
    pub selected: String,
}

impl Theme {
    pub fn new() -> Self {
        Theme {
            selected: termion::color::Bg(termion::color::Blue).to_string() + &termion::color::Fg(termion::color::LightYellow).to_string(),
        }
    }
}

impl GUI {
    fn draw_servers(&mut self) {
        let (_width, height) = termion::terminal_size().unwrap();
        let list_height: u16 = height as u16 - 5;
    
        let mut vert_pos = 5;
        let mut idx = 0;
        for channel in &self.servers[self.curr_server].channels {
            write!(self.screen, "{}{}{}{}{}",
                termion::cursor::Goto(2, vert_pos),
    
                if idx == self.servers[self.curr_server].curr_channel { termion::color::Bg(termion::color::Blue).to_string() }
                else { termion::color::Bg(termion::color::Reset).to_string() },
                
                channel,
                " ".repeat(self.bounds.left_margin - channel.len()),
                
                termion::color::Bg(termion::color::Reset),
            ).unwrap();
            vert_pos += 1;
            idx += 1;
            //TODO scrolling
        }
        vert_pos = list_height / 2 + 6;
        idx = 0;
        for server in &self.servers {
            write!(self.screen, "{}{}{}{}{}{}{}",
                termion::cursor::Goto(2, vert_pos),
    
                if idx == self.curr_server { self.theme.selected.clone() }
                else { "".to_string() },
                
                if server.net.is_none() { termion::color::Fg(termion::color::Red).to_string() }
                else { "".to_string() },
                
                server.name,
                " ".repeat(self.bounds.left_margin - server.name.len()),
                
                termion::color::Fg(termion::color::Reset),
                termion::color::Bg(termion::color::Reset),
            ).unwrap();
            vert_pos += 1;
            idx += 1;
        }
    }
    
    fn draw_messages(&mut self) {
        let messages = &self.servers[self.curr_server].loaded_messages;
        let (width, height) = termion::terminal_size().unwrap();
        let max_messages = height as usize - 3;
        let len = messages.len();
    
        let start_idx = len as isize - max_messages as isize + self.scroll as isize;
        let start_idx = if start_idx < 0 { 0 } else { start_idx as usize };
    
        if self.scroll > 0 { self.scroll = 0; }
        if (self.scroll + start_idx as isize) <= 0 { self.scroll = 0 - start_idx as isize; }
    
        //LOL not a good idea but it works
        let start_idx = len as isize - max_messages as isize + self.scroll as isize;
        let start_idx = if start_idx < 0 { 0 } else { start_idx as usize };
        
    
        let mut total_lines = 0;
        let max_chars: usize = width as usize - self.bounds.left_margin - 4;
        let max_lines = height - 2;
        for msg in messages[(start_idx as isize + self.scroll) as usize..(len as isize + self.scroll) as usize].iter() {
            total_lines += (msg.content.len() as f64 / max_chars as f64).ceil() as usize;
        }
    
        let mut line = total_lines as u16;
    
        let mut buffer: String = "".to_string();
    
        for message in messages[(start_idx as isize + self.scroll) as usize..(len as isize + self.scroll) as usize].iter() {
    
            let num_lines: usize = (message.content.len() as f64 / max_chars as f64).ceil() as usize;
            for i in 0..num_lines {
                if line >= max_lines {
                    line -= 1;
                    continue;
                }
                let e = if (i + 1) * max_chars >= message.content.len() { message.content.len() } else { (i + 1) * max_chars };
                buffer.push_str(&format!("{}{}{}", termion::cursor::Goto(self.bounds.left_margin as u16 + 4, height - line - 1), &message.content[i * max_chars..e], ""));
                line -= 1;
            }
        }
        write!(self.screen, "{}", buffer).unwrap();
    }
    
    
    fn draw_border(&mut self) {
        let (width, height) = termion::terminal_size().unwrap();
        let list_height: usize = height as usize - 5;
        let channels_height: usize = list_height / 2;
        let servers_height: usize;
        if list_height % 2 == 0 {
            servers_height = list_height / 2 - 1;
        } else {
            servers_height = list_height / 2;
        }
        
        let server_string = centred("cospox.com", self.bounds.left_margin);
        let space_padding = " ".repeat(width as usize - self.bounds.left_margin - 3);
        write!(self.screen, "{}{}┏{}┳{}┓\r\n┃{}┃{}┃\r\n┃{}┃{}┃\r\n┣{}┫{}┃\r\n{}┣{}┫{}┃\r\n{}┗{}┻{}┛",
            termion::cursor::Goto(1, 1), termion::clear::All,
            "━".repeat(self.bounds.left_margin), "━".repeat(width as usize - self.bounds.left_margin - 3),
            centred("Connected to", self.bounds.left_margin), space_padding,
            server_string, space_padding,
            "━".repeat(self.bounds.left_margin), space_padding,
            format!("┃{}┃{}┃\r\n", " ".repeat(self.bounds.left_margin), space_padding).repeat(channels_height),
            "━".repeat(self.bounds.left_margin), space_padding,
            format!("┃{}┃{}┃\r\n", " ".repeat(self.bounds.left_margin), space_padding).repeat(servers_height),
            "━".repeat(self.bounds.left_margin), "━".repeat(width as usize - self.bounds.left_margin - 3),
        ).unwrap();
    
    }

    fn draw_new_server(&mut self) {
        let (_width, height) = termion::terminal_size().unwrap();

        let cur_x: u16;
        let cur_y: u16;
        match self.sel_idx {
            0 => { cur_x = self.bounds.left_margin as u16 + 11 + self.ip_buffer.len() as u16;   cur_y = height - 4; }
            1 => { cur_x = self.bounds.left_margin as u16 + 11 + self.port_buffer.len() as u16; cur_y = height - 3; }
            2 => { cur_x = self.bounds.left_margin as u16 + 11 + self.uuid_buffer.len() as u16; cur_y = height - 2; }
            _ => { cur_x = 1; cur_y = 1; }
        }
        
        write!(self.screen, "{}{}ip   : {}{}{}{}{}port : {}{}{}{}{}uuid : {}{}{}{}{}[connect]{}{} {}[cancel]{}{}{}",
            if self.sel_idx == 0 { self.theme.selected.clone() } else { "".to_string() },
            termion::cursor::Goto(self.bounds.left_margin as u16 + 4, height - 4),
            termion::color::Fg(termion::color::Reset), termion::color::Bg(termion::color::Reset),
            self.ip_buffer,

            if self.sel_idx == 1 { self.theme.selected.clone() } else { "".to_string() },
            termion::cursor::Goto(self.bounds.left_margin as u16 + 4, height - 3),
            termion::color::Fg(termion::color::Reset), termion::color::Bg(termion::color::Reset),
            self.port_buffer,

            if self.sel_idx == 2 { self.theme.selected.clone() } else { "".to_string() },
            termion::cursor::Goto(self.bounds.left_margin as u16 + 4, height - 2),
            termion::color::Fg(termion::color::Reset), termion::color::Bg(termion::color::Reset),
            self.uuid_buffer,
            
            termion::cursor::Goto(self.bounds.left_margin as u16 + 4, height - 1),
            if self.sel_idx == 3 { self.theme.selected.clone() } else { "".to_string() }, 
            termion::color::Bg(termion::color::Reset),
            termion::color::Fg(termion::color::Reset),
            if self.sel_idx == 4 { self.theme.selected.clone() } else { "".to_string() },
            termion::color::Bg(termion::color::Reset),
            termion::color::Fg(termion::color::Reset),
            termion::cursor::Goto(cur_x, cur_y),
        ).unwrap();
        
    }
    
    pub fn draw_screen(&mut self)  {
        let (width, height) = termion::terminal_size().unwrap();
    
        if width < 32 || height < 8 {
            write!(self.screen, "Terminal size is too small lol").unwrap();
            return;
        }
    
        match self.mode {
            Mode::Messages => {
                self.draw_border();
                if self.servers.len() > 0 {
                    self.draw_messages();
                    self.draw_servers();
                }
                write!(self.screen, "{}{}", termion::cursor::Goto(self.bounds.left_margin as u16 + 4, height - 1), self.buffer).unwrap();
            }
            Mode::NewServer => {
                self.draw_border();
                if self.servers.len() > 0 {
                    self.draw_servers();
                }
                self.draw_new_server();
            }
            Mode::Settings => {
                
            }
        }
    }
}

