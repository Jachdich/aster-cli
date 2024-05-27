extern crate termion;
use std::io::Write;
use std::fmt;
use crate::DisplayMessage;
use crate::server::Server;
use crate::gui::GUI;
use super::Mode;
use fmtstring::{Colour, FmtString, FmtChar};

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

// #[derive(Clone, Debug)]
// pub struct FmtChar {
// 	pub ch: String,
// 	pub fg: String,
// 	pub bg: String,
// }

// #[derive(Clone)]
// pub struct FmtString {
//     pub cont: Vec<FmtChar>,
// 	dirty: bool,
// 	cache: String,
// }


/*
impl fmt::Display for FmtString {
    fn fmt(&mut self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.dirty {
            self.rebuild_cache();
        }
        
        write!(f, "{}", self.cache)
    }
}*/

// impl From<String> for FmtString {
//     fn from(item: String) -> Self {
//         FmtString::from_str(&item)
//     }
// }

// use core::ops::Index;
// use core::ops::IndexMut;
// use core::ops::Range;

// impl Index<Range<usize>> for FmtString {
//     type Output = [FmtChar];
//     fn index(&self, range: Range<usize>) -> &Self::Output {
//         &self.cont[range]
//     }
// }

// impl Index<usize> for FmtString {
//     type Output = FmtChar;
//     fn index(&self, idx: usize) -> &Self::Output {
//         &self.cont[idx]
//     }
// }

// impl IndexMut<usize> for FmtString {
//     fn index_mut(&mut self, idx: usize) -> &mut FmtChar {
//         self.dirty = true;
//         &mut self.cont[idx]
//     }
// }

// impl FmtString {
//     pub fn from_str(data: &str) -> Self {
//         let mut buf: Vec<FmtChar> = Vec::new();
//         for ch in data.chars() {
//             buf.push(FmtChar { ch: ch.to_string(), fg: String::new(), bg: String::new() });
//         }
//         FmtString {
//             cont: buf,
//             dirty: true,
//             cache: String::new()
//         }
//     }

//     pub fn from_buffer(data: Vec<FmtChar>) -> Self {
//         FmtString {
//             cont: data,
//             dirty: true,
//             cache: String::new()
//         }
//     }

//     pub fn from_slice(data: &[FmtChar]) -> Self {
//         FmtString {
//             cont: data.to_vec(),
//             dirty: true,
//             cache: String::new()
//         }
//     }
    
//     pub fn to_optimised_string(&self) -> String {
//         let mut buf = String::new();
//         let mut last_fg = String::new();
//         let mut last_bg = String::new();
//         for ch in &self.cont {
//             if last_fg != ch.fg {
//                 buf.push_str(&ch.fg);
//                 last_fg = ch.fg.clone();
//             }
//             if last_bg != ch.bg {
//                 buf.push_str(&ch.bg);
//                 last_bg = ch.bg.clone();
//             }
//             buf.push_str(&ch.ch);
//         }
//         buf
//     }

//     pub fn as_str(&mut self) -> &str {
//         if self.dirty {
//             self.rebuild_cache();
//         }
        
//         &self.cache
//     }

//     pub fn len(&self) -> usize {
//         self.cont.len()
//     }

//     fn rebuild_cache(&mut self) {
//         self.cache = self.to_optimised_string();
//         self.dirty = false;
//     }
// }

// impl FmtChar {
    pub fn fmtchar_from_json(val: &json::JsonValue) -> FmtChar {
        FmtChar {
            ch: val[0].as_str().unwrap().chars().next().unwrap(),
            fg: parse_colour(&val[1].to_string()),
            bg: parse_colour(&val[2].to_string()),
        }
    }

//     pub fn width(&self) -> u16 {
//         self.ch.chars().count() as u16
//     }
// }
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

fn parse_colour(inp: &str) -> Colour {
    if inp.starts_with("#") { panic!("RGB colours not currently supported"); }

    match inp {
        "black"         => Colour::Black,
        "blue"          => Colour::Blue,
        "cyan"          => Colour::Cyan,
        "green"         => Colour::Green,
        "light black"   => Colour::LightBlack,
        "light blue"    => Colour::LightBlue,
        "light green"   => Colour::LightGreen,
        "light magenta" => Colour::LightMagenta,
        "light red"     => Colour::LightRed,
        "light white"   => Colour::LightWhite,
        "light yellow"  => Colour::LightYellow,
        "magenta"       => Colour::Magenta,
        "red"           => Colour::Red,
        "reset"         => Colour::Default,
        "white"         => Colour::White,
        "yellow"        => Colour::Yellow,
        _ => todo!(),
    }
}

// #[derive(Clone, Debug)]
// pub struct Colour {
//     pub fg: String,
//     pub bg: String,
// }

// impl Colour {
//     pub fn new() -> Self {
//         Colour {
//             fg: "".to_string(),
//             bg: "".to_string(),
//         }
//     }

//     pub fn from_strs(fg: &str, bg: &str) -> Colour {
//         Colour { fg: fg.to_string(), bg: bg.to_string() }
//     }
// }

#[derive(Clone, Debug)]
pub struct Colour2 {
    pub fg: Colour,
    pub bg: Colour,
}

impl fmt::Display for Colour2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.fg.to_string(fmtstring::Ground::Foreground), self.bg.to_string(fmtstring::Ground::Background))
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
    pub text: Colour2,
    pub selected_text: Colour2,
    pub unfocussed_selected_text: Colour2,
    pub error_text: Colour2,
    pub system_message: Colour2,
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
            text:                     Colour2 { fg: parse_colour(&get_or("text-foreground",                     cfg, fallback).to_string()), bg: parse_colour(&get_or("text-background", cfg, fallback).to_string())},
            selected_text:            Colour2 { fg: parse_colour(&get_or("selected-text-foreground",            cfg, fallback).to_string()), bg: parse_colour(&get_or("selected-text-background", cfg, fallback).to_string())},
            unfocussed_selected_text: Colour2 { fg: parse_colour(&get_or("unfocussed-selected-text-foreground", cfg, fallback).to_string()), bg: parse_colour(&get_or("unfocussed-selected-text-background", cfg, fallback).to_string())},
            error_text:               Colour2 { fg: parse_colour(&get_or("error-text-foreground",               cfg, fallback).to_string()), bg: parse_colour(&get_or("error-text-background", cfg, fallback).to_string())},
            system_message:           Colour2 { fg: parse_colour(&get_or("system-message-foreground",           cfg, fallback).to_string()), bg: parse_colour(&get_or("system-message-background", cfg, fallback).to_string())},

            border: ThemedBorder {
                tl:           fmtchar_from_json(get_or("border-tl",           &cfg, &fallback)),
                tr:           fmtchar_from_json(get_or("border-tr",           &cfg, &fallback)),
                bl:           fmtchar_from_json(get_or("border-bl",           &cfg, &fallback)),
                br:           fmtchar_from_json(get_or("border-br",           &cfg, &fallback)),
                top:          fmtchar_from_json(get_or("border-top",          &cfg, &fallback)),
                bottom:       fmtchar_from_json(get_or("border-bottom",       &cfg, &fallback)),
                left:         fmtchar_from_json(get_or("border-left",         &cfg, &fallback)),
                right:        fmtchar_from_json(get_or("border-right",        &cfg, &fallback)),
                bottom_split: fmtchar_from_json(get_or("border-bottom-split", &cfg, &fallback)),
                top_split:    fmtchar_from_json(get_or("border-top-split",    &cfg, &fallback)),
                left_split:   fmtchar_from_json(get_or("border-left-split",   &cfg, &fallback)),
                right_split:  fmtchar_from_json(get_or("border-right-split",  &cfg, &fallback)),
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
            left_margin: 32,
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
    format!("{}{}{}{}{}", c.fg.to_string(fmtstring::Ground::Foreground), c.bg.to_string(fmtstring::Ground::Background), (&format!("{}", c.ch)).repeat(n), termion::color::Fg(termion::color::Reset), termion::color::Bg(termion::color::Reset))
}

impl GUI {
    fn draw_servers(&mut self) {
        let (_width, height) = termion::terminal_size().unwrap();
        let height = height - 1;
        let list_height: u16 = self.theme.get_list_height(height) as u16;
    
        let mut vert_pos = self.theme.get_servers_start_pos() as u16;
        let mut idx = 0;
        if let Some(curr_server) = self.curr_server {
            if let Server::Online { channels, curr_channel, .. } = &self.servers[curr_server] {
                for channel in channels {
                    write!(self.screen, "{}{}{}{}{}{}{}{}",
                        termion::cursor::Goto(1 + self.theme.servers.border.left.width(), vert_pos),
                        termion::color::Fg(termion::color::Reset), termion::color::Bg(termion::color::Reset),
    
                        if curr_channel.is_some_and(|cc| idx == cc) { self.theme.servers.selected_text.clone() }
                        else { self.theme.servers.text.clone() },
                
                        channel.name,
                        " ".repeat(self.theme.left_margin - channel.name.len()),
                
                        termion::color::Bg(termion::color::Reset),
                        termion::color::Fg(termion::color::Reset),
                    ).unwrap();
                    vert_pos += 1;
                    idx += 1;
                    //TODO scrolling
                }
            }
        }
        vert_pos = self.theme.get_channels_start_pos(height) as u16 + 1;
        idx = 0;
        for server in &self.servers {
            write!(self.screen, "{}{}{}{}{}{}{}{}{}",
                termion::cursor::Goto(1 + self.theme.servers.border.left.width(), vert_pos),
                termion::color::Fg(termion::color::Reset), termion::color::Bg(termion::color::Reset),
    
                if Some(idx) == self.curr_server { self.theme.servers.selected_text.clone() }
                else { self.theme.servers.text.clone() },
                
                if let Server::Offline { .. } = server { self.theme.servers.error_text.clone() }
                else { Colour2 { fg: Colour::Default, bg: Colour::Default } },
                
                server.name().unwrap_or("Unknown Server"),
                " ".repeat(self.theme.left_margin - server.name().unwrap_or("Unknown Server").len()),
                
                termion::color::Fg(termion::color::Reset),
                termion::color::Bg(termion::color::Reset),
            ).unwrap();
            vert_pos += 1;
            idx += 1;
        }
    }
    
    fn draw_messages(&mut self) {
        // TODO: Possibly more efficient way of doing this without copying?
        let nothing = Vec::new();
        let messages = if let Some(curr_server) = self.curr_server {
            match &self.servers[curr_server] {
                Server::Online { loaded_messages, peers, .. } => loaded_messages,/*.iter().map(|message| match message {
                    DisplayMessage::User(message) => FmtString::from_str(&format!("{}: {}", peers.get(&message.author_uuid).unwrap().name, message.content)),
                    DisplayMessage::System(s) => s.clone(),
                }).collect::<Vec<FmtString>>(),*/
                _ => &nothing,
            }
        } else { &nothing };

        let (width, height) = termion::terminal_size().unwrap();
        let height = height - 1;
        let max_messages = height as usize - (self.theme.messages.border.top.width() + self.theme.messages.border.bottom.width() + self.theme.edit.border.bottom.width() + self.theme.edit.border.top.width()) as usize;
        let len = messages.len();
    
        let start_idx = len as isize - max_messages as isize + self.scroll as isize;
        let start_idx = if start_idx < 0 { 0 } else { start_idx as usize };
    
        if self.scroll > 0 { self.scroll = 0; }
        if (self.scroll + start_idx as isize) <= 0 { self.scroll = 0 - start_idx as isize; }
    
        //LOL not a good idea but it works
        //let start_idx = len as isize - max_messages as isize + self.scroll as isize;
        //let start_idx = if start_idx < 0 { 0 } else { start_idx as usize };
       
    
        let mut total_lines = 0;
        let max_chars: usize = width as usize - self.theme.left_margin - 4;
        let max_lines = height - 2;
        for msg in messages[(start_idx as isize + self.scroll) as usize..(len as isize + self.scroll) as usize].iter() {
            total_lines += (msg.len() as f64 / max_chars as f64).ceil() as usize;
        }
    
        let mut line = total_lines as u16;
    
        let mut buffer: String = "".to_string();
    
        for message in messages[(start_idx as isize + self.scroll) as usize..(len as isize + self.scroll) as usize].iter() {
    
            let num_lines: usize = (message.len() as f64 / max_chars as f64).ceil() as usize;
            for i in 0..num_lines {
                if line >= max_lines {
                    line -= 1;
                    continue;
                }
                let e = if (i + 1) * max_chars >= message.len() { message.len() } else { (i + 1) * max_chars };
                
                buffer.push_str(&format!("{}{}{}",
                    termion::cursor::Goto(
                        self.theme.left_margin as u16 + self.theme.servers.border.left.width() + self.theme.servers.border.right.width() + 2,
                        height - line - 1
                    ),
                    Into::<FmtString>::into(&message[i * max_chars..e]).to_str(), " ".repeat(max_chars - message[i * max_chars..e].len())
                ));
                line -= 1;
            }
        }
        write!(self.screen, "{}{}{}", termion::color::Fg(termion::color::Reset), termion::color::Bg(termion::color::Reset), buffer).unwrap();
    }
    
    
    fn draw_border(&mut self) {
        if self.draw_border {
            self.draw_border = false;
            let (width, height) = termion::terminal_size().unwrap();
            let height = height - 1;
            let channels_height = self.theme.get_channels_height(height);
            let servers_height  = self.theme.get_servers_height(height);
            
            let left_margin = self.theme.left_margin;

            let total_border_width = (self.theme.servers.border.left.width() + self.theme.servers.border.right.width()
                                   + self.theme.messages.border.left.width() + self.theme.messages.border.right.width()) as usize;
            let space_padding = " ".repeat(width as usize - left_margin - total_border_width);
            let rs = termion::color::Fg(termion::color::Reset).to_string() + (&termion::color::Bg(termion::color::Reset).to_string());

            self.border_buffer = format!("{0}{1}{sttl}{2}{3}\r\n{stleft}{4}{stright}{mleft}{space_padding}{mright}\r\n{stleft}{5}{stright}{mleft}{space_padding}{mright}\r\n{6}{7}{8}{9}{sbl}{10}{11}",
            
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

    /*5*/       centred("cospox.com", self.theme.left_margin),

    /*6*/       if self.theme.channels.border.bottom.width() > 0 {
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

    /*7*/       format!("{cleft}{rs}{}{cright}{rs}{mleft}{}{mright}\r\n",
                    " ".repeat(left_margin),
                    space_padding,
                    rs = rs,
                    cleft = self.theme.channels.border.left, 
                    cright = self.theme.channels.border.right, 
                    mright = self.theme.messages.border.right,
                    mleft = self.theme.messages.border.left,
                ).repeat(channels_height),

    /*8*/       if self.theme.channels.border.bottom.width() > 0 && self.theme.servers.border.top.width() > 0 {
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
                
    /*9*/       format!("{sleft}{rs}{}{sright}{rs}{mleft}{}{mright}\r\n", 
                    " ".repeat(self.theme.left_margin), 
                    space_padding,
                    rs = rs,
                    sleft = self.theme.servers.border.left,
                    sright = self.theme.servers.border.right,
                    mright = self.theme.messages.border.right,
                    mleft = self.theme.messages.border.left,
                ).repeat(servers_height),

    /*10*/      border_rep(&self.theme.servers.border.bottom, left_margin),

    /*11*/      if self.theme.messages.border.left.width() == 0 || self.theme.servers.border.right.width() == 0 {
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

            );
        }
        write!(self.screen, "{}", self.border_buffer).unwrap();
    
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
            self.redraw = true;
        }

        write!(self.screen, "{}{}{}", termion::cursor::Goto(1, height), termion::clear::CurrentLine, self.system_message.to_optimised_string()).unwrap();
    
        match self.mode {
            Mode::Messages => {
                if self.servers.len() > 0 {
                    self.draw_messages();
                    self.draw_servers();
                }
                write!(self.screen, "{}{}", termion::cursor::Goto(self.theme.left_margin as u16 + self.theme.servers.border.left.width() + self.theme.servers.border.right.width() + 2, height - 2), self.buffer).unwrap();
            }
            Mode::NewServer => {
                if self.servers.len() > 0 {
                    self.draw_servers();
                }
                let (_width, height) = termion::terminal_size().unwrap();
                let y = height - self.prompt.as_ref().unwrap().height() - 1;
                self.prompt.as_ref().unwrap().draw(&mut self.screen, self.theme.left_margin as u16 + 4, y, &self.theme);
            }
            Mode::Settings => {
                
            }
        }
        self.last_w = width;
        self.last_h = height;
    }
}

