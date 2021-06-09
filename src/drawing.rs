extern crate termion;
use std::io::Write;
use std::fmt;

use crate::gui::GUI;
use super::Mode;

fn centred(text: &str, width: usize) -> String {
    format!("{: ^1$}", text, width)
}
/*
#[derive(Copy, Clone, Debug)]
pub struct RGB {
	pub r: u8,
	pub g: u8,
	pub b: u8,
	pub default: bool,
}*/

#[derive(Clone, Debug)]
pub struct FmtChar {
	pub ch: String,
	pub fg: String,
	pub bg: String,
}

impl fmt::Display for FmtChar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}", self.fg, self.bg, self.ch)
    }
}

impl FmtChar {
    pub fn from_json(val: &json::JsonValue) -> Self {
        FmtChar {
            ch: val[0].to_string(),
            fg: parse_colour(&val[1].to_string(), false).to_string(),
            bg: parse_colour(&val[2].to_string(), true).to_string(),
        }
    }

    pub fn width(&self) -> u16 {
        self.ch.chars().count() as u16
    }
}
/*
impl RGB {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        RGB {
            r: r, g: g, b: b,
            default: false
        }
    }
    
    pub fn from_html(n: u32) -> Self {
    	let r: u8 = ((n >> 16) & 0xFF) as u8;
    	let g: u8 = ((n >> 8)  & 0xFF) as u8;
    	let b: u8 = ((n >> 0)  & 0xFF) as u8;
    	RGB {
    		r:r, g:g, b:b, default:false
    	}
    }
    pub fn to_fg(&self) -> termion::color::Fg<termion::color::Rgb> {
    	return termion::color::Fg(termion::color::Rgb(self.r, self.g, self.b));
    }
    pub fn to_bg(&self) -> termion::color::Bg<termion::color::Rgb> {
    	return termion::color::Bg(termion::color::Rgb(self.r, self.g, self.b));
    }
    pub fn get_inverted(&self) -> RGB {
    	let txt_col: RGB;
        if self.r as u16 + self.g as u16 + self.b as u16 > 384 {
        	txt_col = RGB::new(0, 0, 0);
        } else {
        	txt_col = RGB::new(255, 255, 255);
        }
        return txt_col;
    }

    pub fn to_html_string(&self) -> String {
    	if self.default {
    		return "default".to_string();
    	}
    	format!("#{:02X?}{:02X?}{:02X?}", self.r, self.g, self.b)
    }
}

impl std::cmp::PartialEq for RGB {
    fn eq(&self, other: &Self) -> bool {
        self.r == other.r && self.g == other.g && self.b == other.b
    }
}*/

fn parse_colour(inp: &str, bg: bool) -> &'static str {
    if inp.starts_with("#") { panic!("RGB colours not currently supported"); }

    match inp {
        "black" => if bg { termion::color::Black.bg_str() } else { termion::color::Black.fg_str() },
        "blue" => if bg { termion::color::Blue.bg_str() } else { termion::color::Blue.fg_str() },
        "cyan" => if bg { termion::color::Cyan.bg_str() } else { termion::color::Cyan.fg_str() },
        "green" => if bg { termion::color::Green.bg_str() } else { termion::color::Green.fg_str() },
        "light black" => if bg { termion::color::LightBlack.bg_str() } else { termion::color::LightBlack.fg_str() },
        "light blue" => if bg { termion::color::LightBlue.bg_str() } else { termion::color::LightBlue.fg_str() },
        "light green" => if bg { termion::color::LightGreen.bg_str() } else { termion::color::LightGreen.fg_str() },
        "light magenta" => if bg { termion::color::LightMagenta.bg_str() } else { termion::color::LightMagenta.fg_str() },
        "light red" => if bg { termion::color::LightRed.bg_str() } else { termion::color::LightRed.fg_str() },
        "light white" => if bg { termion::color::LightWhite.bg_str() } else { termion::color::LightWhite.fg_str() },
        "light yellow" => if bg { termion::color::LightYellow.bg_str() } else { termion::color::LightYellow.fg_str() },
        "magenta" => if bg { termion::color::Magenta.bg_str() } else { termion::color::Magenta.fg_str() },
        "red" => if bg { termion::color::Red.bg_str() } else { termion::color::Red.fg_str() },
        "reset" => if bg { termion::color::Reset.bg_str() } else { termion::color::Reset.fg_str() },
        "white" => if bg { termion::color::White.bg_str() } else { termion::color::White.fg_str() },
        "yellow" => if bg { termion::color::Yellow.bg_str() } else { termion::color::Yellow.fg_str() },
        _ => "",
    }
}

#[derive(Clone, Debug)]
pub struct ThemedBorder {
    pub tl: FmtChar,
    pub tr: FmtChar,
    pub bl: FmtChar,
    pub br: FmtChar,
    pub top: FmtChar,
    pub bottom: FmtChar,
    pub left: FmtChar,
    pub right: FmtChar,
    pub bottom_split: FmtChar,
    pub top_split: FmtChar,
    pub left_split: FmtChar,
    pub right_split: FmtChar,
}

#[derive(Clone, Debug)]
pub struct ThemedArea {
    pub text: String,
    pub selected_text: String,
    pub unfocussed_selected_text: String,
    pub error_text: String,
    pub system_message: String,
    pub border: ThemedBorder,
}

#[derive(Clone, Debug)]
pub struct Theme {
    pub channels: ThemedArea,
    pub servers: ThemedArea,
    pub edit: ThemedArea,
    pub messages: ThemedArea,
    pub status: ThemedArea,
    pub left_margin: usize,
}

fn get_or<'a>(name: &str, main: &'a json::JsonValue, aux: &'a json::JsonValue) -> &'a json::JsonValue {
    if main[name].is_null() {
        &aux[name]
    } else {
        &main[name]
    }
}

impl ThemedArea {
    pub fn new(cfg: &json::JsonValue, fallback: &json::JsonValue) -> Self {
        ThemedArea {
            text: parse_colour(&get_or("text-foreground", cfg, fallback).to_string(), false).to_string() + parse_colour(&get_or("text-background", cfg, fallback).to_string(), true),
            selected_text: parse_colour(&get_or("selected-text-foreground", cfg, fallback).to_string(), false).to_string() + parse_colour(&get_or("selected-text-background", cfg, fallback).to_string(), true),
            unfocussed_selected_text: parse_colour(&get_or("unfocussed-selected-text-foreground", cfg, fallback).to_string(), false).to_string() + parse_colour(&get_or("unfocussed-selected-text-background", cfg, fallback).to_string(), true),
            error_text: parse_colour(&get_or("error-text-foreground", cfg, fallback).to_string(), false).to_string() + parse_colour(&get_or("error-text-background", cfg, fallback).to_string(), true),
            system_message: parse_colour(&get_or("system-message-foreground", cfg, fallback).to_string(), false).to_string() + parse_colour(&get_or("system-message-background", cfg, fallback).to_string(), true),

            border: ThemedBorder {
                tl:           FmtChar::from_json(get_or("border-tl",           &cfg, &fallback)),
                tr:           FmtChar::from_json(get_or("border-tr",           &cfg, &fallback)),
                bl:           FmtChar::from_json(get_or("border-bl",           &cfg, &fallback)),
                br:           FmtChar::from_json(get_or("border-br",           &cfg, &fallback)),
                top:          FmtChar::from_json(get_or("border-top",          &cfg, &fallback)),
                bottom:       FmtChar::from_json(get_or("border-bottom",       &cfg, &fallback)),
                left:         FmtChar::from_json(get_or("border-left",         &cfg, &fallback)),
                right:        FmtChar::from_json(get_or("border-right",        &cfg, &fallback)),
                bottom_split: FmtChar::from_json(get_or("border-bottom-split", &cfg, &fallback)),
                top_split:    FmtChar::from_json(get_or("border-top-split",    &cfg, &fallback)),
                left_split:   FmtChar::from_json(get_or("border-left-split",   &cfg, &fallback)),
                right_split:  FmtChar::from_json(get_or("border-right-split",  &cfg, &fallback)),
            }
        }
    }
}

impl Theme {
    pub fn new(filename: &str) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let totalcfg = json::parse(&std::fs::read_to_string(filename)?)?;

        let servers = ThemedArea::new(&totalcfg["servers"], &totalcfg["global"]);
        let channels = ThemedArea::new(&totalcfg["channels"], &totalcfg["global"]);
        let edit = ThemedArea::new(&totalcfg["edit"], &totalcfg["global"]);
        let messages = ThemedArea::new(&totalcfg["messages"], &totalcfg["global"]);
        let status = ThemedArea::new(&totalcfg["status"], &totalcfg["global"]);

        Ok(Theme {
            left_margin: 24,
            servers,
            channels,
            edit,
            messages,
            status,
        })
    }

    fn get_list_height(&self, height: u16) -> usize {
        (height - 1 - self.channels.border.top.width()
                    - self.channels.border.bottom.width() * 2
                    - self.servers.border.top.width()
                    - self.servers.border.bottom.width()) as usize
    }

    fn get_servers_start_pos(&self) -> usize {
        (self.channels.border.top.width()
            + 2 + self.channels.border.bottom.width() * 2) as usize
    }

    fn get_channels_start_pos(&self, height: u16) -> usize {
        self.get_servers_start_pos() as usize
            + self.get_servers_height(height) as usize
            + self.channels.border.bottom.width() as usize
            + self.servers.border.top.width() as usize
    }
    
    fn get_servers_height(&self, height: u16) -> usize {
        let list_height = self.get_list_height(height);
        if list_height % 2 == 0 {
            list_height / 2 - 1
        } else {
            list_height / 2
        }
    }
    fn get_channels_height(&self, height: u16) -> usize {
        let list_height = self.get_list_height(height);
        list_height / 2
    }
}

fn border_rep(c: &FmtChar, n: usize) -> String {
    format!("{}{}{}{}{}", c.fg, c.bg, (&format!("{}", c.ch)).repeat(n), termion::color::Fg(termion::color::Reset), termion::color::Bg(termion::color::Reset))
}

impl GUI {
    fn draw_servers(&mut self) {
        let (_width, height) = termion::terminal_size().unwrap();
        let list_height: u16 = self.theme.get_list_height(height) as u16;
    
        let mut vert_pos = self.theme.get_servers_start_pos() as u16;
        let mut idx = 0;
        for channel in &self.servers[self.curr_server].channels {
            write!(self.screen, "{}{}{}{}{}{}{}{}",
                termion::cursor::Goto(1 + self.theme.servers.border.left.width(), vert_pos),
                termion::color::Fg(termion::color::Reset), termion::color::Bg(termion::color::Reset),
    
                if idx == self.servers[self.curr_server].curr_channel { self.theme.servers.selected_text.clone() }
                else { self.theme.servers.text.clone() },
                
                channel,
                " ".repeat(self.theme.left_margin - channel.len()),
                
                termion::color::Bg(termion::color::Reset),
                termion::color::Fg(termion::color::Reset),
            ).unwrap();
            vert_pos += 1;
            idx += 1;
            //TODO scrolling
        }
        vert_pos = self.theme.get_channels_start_pos(height) as u16;
        idx = 0;
        for server in &self.servers {
            write!(self.screen, "{}{}{}{}{}{}{}{}{}",
                termion::cursor::Goto(1 + self.theme.servers.border.left.width(), vert_pos),
                termion::color::Fg(termion::color::Reset), termion::color::Bg(termion::color::Reset),
    
                if idx == self.curr_server { self.theme.servers.selected_text.clone() }
                else { self.theme.servers.text.clone() },
                
                if server.net.is_none() { self.theme.servers.error_text.clone() }
                else { "".to_string() },
                
                server.name,
                " ".repeat(self.theme.left_margin - server.name.len()),
                
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
        let max_messages = height as usize - (self.theme.messages.border.top.width() + self.theme.messages.border.bottom.width() + self.theme.edit.border.bottom.width() + self.theme.edit.border.top.width()) as usize;
        let len = messages.len();
    
        let start_idx = len as isize - max_messages as isize + self.scroll as isize;
        let start_idx = if start_idx < 0 { 0 } else { start_idx as usize };
    
        if self.scroll > 0 { self.scroll = 0; }
        if (self.scroll + start_idx as isize) <= 0 { self.scroll = 0 - start_idx as isize; }
    
        //LOL not a good idea but it works
        let start_idx = len as isize - max_messages as isize + self.scroll as isize;
        let start_idx = if start_idx < 0 { 0 } else { start_idx as usize };
        
    
        let mut total_lines = 0;
        let max_chars: usize = width as usize - self.theme.left_margin - 4;
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
                
                buffer.push_str(&format!("{}{}{}", termion::cursor::Goto(
                    self.theme.left_margin as u16 + self.theme.servers.border.left.width() + self.theme.servers.border.right.width() + 2,
                    height - line - 1), &message.content[i * max_chars..e], " ".repeat(max_chars - e)));
                line -= 1;
            }
        }
        write!(self.screen, "{}{}{}", termion::color::Fg(termion::color::Reset), termion::color::Bg(termion::color::Reset), buffer).unwrap();
    }
    
    
    fn draw_border(&mut self) {
        let (width, height) = termion::terminal_size().unwrap();
        let channels_height = self.theme.get_channels_height(height);
        let servers_height  = self.theme.get_servers_height(height);
        
        let left_margin = self.theme.left_margin;

        let total_border_width = (self.theme.servers.border.left.width() + self.theme.servers.border.right.width()
                               + self.theme.messages.border.left.width() + self.theme.messages.border.right.width()) as usize;
        let space_padding = " ".repeat(width as usize - left_margin - total_border_width);
        let rs = termion::color::Fg(termion::color::Reset).to_string() + (&termion::color::Bg(termion::color::Reset).to_string());

//                            0 1      2 3            4                5                    6                7             8 91011     1213
        write!(self.screen, "{}{}{sttl}{}{}\r\n{stleft}{}{stright}{mleft}{}{mright}\r\n{stleft}{}{stright}{mleft}{}{mright}\r\n{}{}{}{}{sbl}{}{}",
        
/*0*/       termion::cursor::Goto(1, 1),
/*1*/       "",
/*2*/       border_rep(&self.theme.status.border.top, left_margin),

/*3*/       if self.theme.messages.border.left.width() == 0 || self.theme.channels.border.right.width() == 0 {
                format!("{sttop_split}{}{mtr}", 
                    border_rep(&self.theme.messages.border.top, width as usize - left_margin - total_border_width),

                    mtr = self.theme.messages.border.tr,
                    sttop_split = self.theme.status.border.top_split,
                )
            } else {
                format!("{sttr}{mtl}{}{mtr}", 
                    border_rep(&self.theme.messages.border.top, width as usize - left_margin - total_border_width),

                    mtr = self.theme.messages.border.tr,
                    mtl = self.theme.messages.border.tl,
                    sttr = self.theme.status.border.tr,
                )
            },
            
/*4*/       centred("Connected to", left_margin),
/*5*/       space_padding,

/*6*/       centred("cospox.com", self.theme.left_margin),
/*7*/       space_padding,

/*8*/       if self.theme.channels.border.bottom.width() > 0 {
                format!("{stleft_split}{}{stright_split}{mleft}{}{mright}\r\n",
                    border_rep(&self.theme.channels.border.bottom, left_margin),
                    space_padding,
                    stleft_split = self.theme.status.border.left_split,
                    stright_split = self.theme.status.border.right_split,
                    mright = self.theme.messages.border.right,
                    mleft = self.theme.messages.border.left,
                )
            } else {
                "".to_string()
            },

/*9*/       format!("{cleft}{rs}{}{cright}{rs}{mleft}{}{mright}\r\n",
                " ".repeat(left_margin),
                space_padding,
                rs = rs,
                cleft = self.theme.channels.border.left, 
                cright = self.theme.channels.border.right, 
                mright = self.theme.messages.border.right,
                mleft = self.theme.messages.border.left,
            ).repeat(channels_height),

/*10*/      if self.theme.channels.border.bottom.width() > 0 && self.theme.servers.border.top.width() > 0 {
                format!("{cbl}{}{cbr}{mleft}{}{mright}\r\n{stl}{}{str}{mleft}{}{mright}\r\n", 
                    border_rep(&self.theme.channels.border.bottom, left_margin), 
                    space_padding,
                    border_rep(&self.theme.servers.border.top, left_margin), 
                    space_padding,
                    cbl = self.theme.channels.border.bl,
                    cbr = self.theme.channels.border.br,
                    str = self.theme.servers.border.tr,
                    stl = self.theme.servers.border.tl,
                    mright = self.theme.messages.border.right,
                    mleft = self.theme.messages.border.left,
                )
            } else if self.theme.channels.border.bottom.width() > 0 {
                format!("{cleft_split}{}{cright_split}{mleft}{}{mright}\r\n", 
                    border_rep(&self.theme.channels.border.bottom, left_margin),
                    space_padding,
                    cleft_split = self.theme.channels.border.left_split,
                    cright_split = self.theme.channels.border.right_split,
                    mright = self.theme.messages.border.right,
                    mleft = self.theme.messages.border.left,
                )
            } else if self.theme.servers.border.top.width() > 0 { 
                format!("{sleft_split}{}{sright_split}{mleft}{}{mright}\r\n", 
                    border_rep(&self.theme.servers.border.top, left_margin),
                    space_padding,
                    sleft_split = self.theme.servers.border.left_split,
                    sright_split = self.theme.servers.border.right_split,
                    mright = self.theme.messages.border.right,
                    mleft = self.theme.messages.border.left,
                )
            } else {
                "".to_string()
            },
            
/*11*/      format!("{sleft}{rs}{}{sright}{rs}{mleft}{}{mright}\r\n", 
                " ".repeat(self.theme.left_margin), 
                space_padding,
                rs = rs,
                sleft = self.theme.servers.border.left,
                sright = self.theme.servers.border.right,
                mright = self.theme.messages.border.right,
                mleft = self.theme.messages.border.left,
            ).repeat(servers_height),

/*12*/      border_rep(&self.theme.servers.border.bottom, left_margin),

/*13*/      if self.theme.messages.border.left.width() == 0 || self.theme.servers.border.right.width() == 0 {
                format!("{sbottom_split}{}{mbr}", 
                    border_rep(&self.theme.messages.border.bottom, width as usize - left_margin - total_border_width),

                    mbr = self.theme.messages.border.br,
                    sbottom_split = self.theme.servers.border.bottom_split,
                )
            } else {
                format!("{sbr}{mbl}{}{mbr}", 
                    border_rep(&self.theme.messages.border.bottom, width as usize - left_margin - total_border_width),

                    mbr = self.theme.messages.border.br,
                    mbl = self.theme.messages.border.bl,
                    sbr = self.theme.servers.border.br,
                )
            },

            //stl = self.theme.servers.border.tl,
            sttl = self.theme.status.border.tl,
            sbl = self.theme.servers.border.bl,
            //ctop_split = self.theme.channels.border.top_split,
            //sleft = self.theme.servers.border.left,
            //cleft = self.theme.channels.border.left,
            stleft = self.theme.status.border.left,
            //sright = self.theme.servers.border.right,
            //sleft_split = self.theme.servers.border.left_split,
            //sright_split = self.theme.servers.border.right_split,
            //sbottom_split = self.theme.servers.border.bottom_split,
            mright = self.theme.messages.border.right,
            //cright = self.theme.channels.border.right,
            stright = self.theme.status.border.right,
            mleft = self.theme.messages.border.left,
            //mbr = self.theme.messages.border.br,
            //mtr = self.theme.messages.border.tr,

            
        ).unwrap();
    
    }

    fn draw_new_server(&mut self) {
        let (_width, height) = termion::terminal_size().unwrap();

        let cur_x: u16;
        let cur_y: u16;
        match self.sel_idx {
            0 => { cur_x = self.theme.left_margin as u16 + 11 + self.ip_buffer.len() as u16;   cur_y = height - 4; }
            1 => { cur_x = self.theme.left_margin as u16 + 11 + self.port_buffer.len() as u16; cur_y = height - 3; }
            2 => { cur_x = self.theme.left_margin as u16 + 11 + self.uuid_buffer.len() as u16; cur_y = height - 2; }
            _ => { cur_x = 1; cur_y = 1; }
        }
        
        write!(self.screen, "{}{}ip   : {}{}{}{}{}port : {}{}{}{}{}uuid : {}{}{}{}{}[connect]{}{} {}[cancel]{}{}{}",
            if self.sel_idx == 0 { self.theme.servers.selected_text.clone() } else { "".to_string() },
            termion::cursor::Goto(self.theme.left_margin as u16 + 4, height - 4),
            termion::color::Fg(termion::color::Reset), termion::color::Bg(termion::color::Reset),
            self.ip_buffer,

            if self.sel_idx == 1 { self.theme.servers.selected_text.clone() } else { "".to_string() },
            termion::cursor::Goto(self.theme.left_margin as u16 + 4, height - 3),
            termion::color::Fg(termion::color::Reset), termion::color::Bg(termion::color::Reset),
            self.port_buffer,

            if self.sel_idx == 2 { self.theme.servers.selected_text.clone() } else { "".to_string() },
            termion::cursor::Goto(self.theme.left_margin as u16 + 4, height - 2),
            termion::color::Fg(termion::color::Reset), termion::color::Bg(termion::color::Reset),
            self.uuid_buffer,
            
            termion::cursor::Goto(self.theme.left_margin as u16 + 4, height - 1),
            if self.sel_idx == 3 { self.theme.servers.selected_text.clone() } else { "".to_string() }, 
            termion::color::Bg(termion::color::Reset),
            termion::color::Fg(termion::color::Reset),
            if self.sel_idx == 4 { self.theme.servers.selected_text.clone() } else { "".to_string() },
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

        if self.last_w != width || self.last_h != height { 
            self.redraw = true;
        }

        if self.redraw {
            self.draw_border();
            self.redraw = false;
        }
    
        match self.mode {
            Mode::Messages => {
                if self.servers.len() > 0 {
                    self.draw_messages();
                    self.draw_servers();
                }
                write!(self.screen, "{}{}", termion::cursor::Goto(self.theme.left_margin as u16 + self.theme.servers.border.left.width() + self.theme.servers.border.right.width() + 2, height - 1), self.buffer).unwrap();
            }
            Mode::NewServer => {
                if self.servers.len() > 0 {
                    self.draw_servers();
                }
                self.draw_new_server();
            }
            Mode::Settings => {
                
            }
        }
        self.last_w = width;
        self.last_h = height;
    }
}

