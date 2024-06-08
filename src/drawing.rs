use crate::gui::GUI;
use crate::server::Server;
use crate::Mode;
use fmtstring::{Colour, FmtChar, FmtString};
use std::fmt;
use std::io::Write;

fn centred(text: &str, width: usize) -> String {
    format!("{: ^1$}", text, width)
}
fn fmtchar_from_json_impl(val: &json::JsonValue) -> Option<FmtChar> {
    Some(FmtChar {
        ch: val[0].as_str()?.chars().next()?,
        fg: parse_colour(&val[1].as_str()?),
        bg: parse_colour(&val[2].as_str()?),
    })
}
pub fn fmtchar_from_json(val: &json::JsonValue) -> OptionalFmtChar {
    OptionalFmtChar(fmtchar_from_json_impl(val))
}

fn parse_colour(inp: &str) -> Colour {
    if inp.starts_with("#") {
        panic!("RGB colours not currently supported");
    }

    match inp {
        "black" => Colour::Black,
        "blue" => Colour::Blue,
        "cyan" => Colour::Cyan,
        "green" => Colour::Green,
        "light black" => Colour::LightBlack,
        "light blue" => Colour::LightBlue,
        "light green" => Colour::LightGreen,
        "light magenta" => Colour::LightMagenta,
        "light red" => Colour::LightRed,
        "light white" => Colour::LightWhite,
        "light yellow" => Colour::LightYellow,
        "magenta" => Colour::Magenta,
        "red" => Colour::Red,
        "reset" => Colour::Default,
        "white" => Colour::White,
        "yellow" => Colour::Yellow,
        _ => todo!(),
    }
}

#[derive(Clone, Debug)]
pub struct OptionalFmtChar(Option<FmtChar>);

impl OptionalFmtChar {
    pub fn width(&self) -> u16 {
        if self.0.is_some() {
            1
        } else {
            0
        }
    }
}

impl fmt::Display for OptionalFmtChar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(c) => write!(f, "{}", c),
            None => Ok(()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Colour2 {
    pub fg: Colour,
    pub bg: Colour,
}

impl fmt::Display for Colour2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            self.fg.to_string(fmtstring::Ground::Foreground),
            self.bg.to_string(fmtstring::Ground::Background)
        )
    }
}

#[derive(Clone, Debug)]
pub struct ThemedBorder {
    pub tl: OptionalFmtChar,
    pub tr: OptionalFmtChar,
    pub bl: OptionalFmtChar,
    pub br: OptionalFmtChar,
    pub top: OptionalFmtChar,
    pub bottom: OptionalFmtChar,
    pub left: OptionalFmtChar,
    pub right: OptionalFmtChar,
    pub bottom_split: OptionalFmtChar,
    pub top_split: OptionalFmtChar,
    pub left_split: OptionalFmtChar,
    pub right_split: OptionalFmtChar,
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

fn get_or<'a>(
    name: &str,
    main: &'a json::JsonValue,
    aux: &'a json::JsonValue,
) -> &'a json::JsonValue {
    if main[name].is_null() {
        &aux[name]
    } else {
        &main[name]
    }
}

impl ThemedArea {
    pub fn new(cfg: &json::JsonValue, fallback: &json::JsonValue) -> Self {
        ThemedArea {
            text: Colour2 {
                fg: parse_colour(&get_or("text-foreground", cfg, fallback).to_string()),
                bg: parse_colour(&get_or("text-background", cfg, fallback).to_string()),
            },
            selected_text: Colour2 {
                fg: parse_colour(&get_or("selected-text-foreground", cfg, fallback).to_string()),
                bg: parse_colour(&get_or("selected-text-background", cfg, fallback).to_string()),
            },
            unfocussed_selected_text: Colour2 {
                fg: parse_colour(
                    &get_or("unfocussed-selected-text-foreground", cfg, fallback).to_string(),
                ),
                bg: parse_colour(
                    &get_or("unfocussed-selected-text-background", cfg, fallback).to_string(),
                ),
            },
            error_text: Colour2 {
                fg: parse_colour(&get_or("error-text-foreground", cfg, fallback).to_string()),
                bg: parse_colour(&get_or("error-text-background", cfg, fallback).to_string()),
            },
            system_message: Colour2 {
                fg: parse_colour(&get_or("system-message-foreground", cfg, fallback).to_string()),
                bg: parse_colour(&get_or("system-message-background", cfg, fallback).to_string()),
            },

            border: ThemedBorder {
                tl: fmtchar_from_json(get_or("border-tl", &cfg, &fallback)),
                tr: fmtchar_from_json(get_or("border-tr", &cfg, &fallback)),
                bl: fmtchar_from_json(get_or("border-bl", &cfg, &fallback)),
                br: fmtchar_from_json(get_or("border-br", &cfg, &fallback)),
                top: fmtchar_from_json(get_or("border-top", &cfg, &fallback)),
                bottom: fmtchar_from_json(get_or("border-bottom", &cfg, &fallback)),
                left: fmtchar_from_json(get_or("border-left", &cfg, &fallback)),
                right: fmtchar_from_json(get_or("border-right", &cfg, &fallback)),
                bottom_split: fmtchar_from_json(get_or("border-bottom-split", &cfg, &fallback)),
                top_split: fmtchar_from_json(get_or("border-top-split", &cfg, &fallback)),
                left_split: fmtchar_from_json(get_or("border-left-split", &cfg, &fallback)),
                right_split: fmtchar_from_json(get_or("border-right-split", &cfg, &fallback)),
            },
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
        (height
            - 1
            - self.channels.border.top.width()
            - self.channels.border.bottom.width() * 2
            - self.servers.border.top.width()
            - self.servers.border.bottom.width()) as usize
    }

    fn get_servers_start_pos(&self) -> usize {
        (self.channels.border.top.width() + 2 + self.channels.border.bottom.width() * 2) as usize
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

fn border_rep(c: &OptionalFmtChar, n: usize) -> String {
    if let Some(c) = c.0 {
        format!(
            "{}{}{}{}{}",
            c.fg.to_string(fmtstring::Ground::Foreground),
            c.bg.to_string(fmtstring::Ground::Background),
            (&format!("{}", c.ch)).repeat(n),
            termion::color::Fg(termion::color::Reset),
            termion::color::Bg(termion::color::Reset)
        )
    } else {
        "".into()
    }
}

impl GUI {
    pub fn draw_servers(&mut self) {
        let (_width, height) = termion::terminal_size().unwrap();
        let height = height - 1;
        let list_height: u16 = self.theme.get_list_height(height) as u16;

        let mut vert_pos = self.theme.get_servers_start_pos() as u16;
        let mut idx = 0;
        if let Some(curr_server) = self.curr_server {
            if let Server::Online {
                channels,
                curr_channel,
                ..
            } = &self.servers[curr_server]
            {
                for channel in channels {
                    write!(
                        self.screen,
                        "{}{}{}{}{}{}{}{}",
                        termion::cursor::Goto(1 + self.theme.servers.border.left.width(), vert_pos),
                        termion::color::Fg(termion::color::Reset),
                        termion::color::Bg(termion::color::Reset),
                        if curr_channel.is_some_and(|cc| idx == cc) {
                            self.theme.servers.selected_text.clone()
                        } else {
                            self.theme.servers.text.clone()
                        },
                        channel.name,
                        " ".repeat(self.theme.left_margin - channel.name.len()),
                        termion::color::Bg(termion::color::Reset),
                        termion::color::Fg(termion::color::Reset),
                    )
                    .unwrap();
                    vert_pos += 1;
                    idx += 1;
                    //TODO scrolling
                }
            }
        }
        while vert_pos < self.theme.get_channels_start_pos(height) as u16 {
            write!(
                self.screen,
                "{}{}",
                termion::cursor::Goto(1 + self.theme.servers.border.left.width(), vert_pos),
                " ".repeat(self.theme.left_margin),
            )
            .unwrap();
            vert_pos += 1;
        }
        vert_pos += 1;
        // vert_pos = self.theme.get_channels_start_pos(height) as u16 + 1;
        idx = 0;
        for server in &self.servers {
            write!(
                self.screen,
                "{}{}{}{}{}{}{}{}{}",
                termion::cursor::Goto(1 + self.theme.servers.border.left.width(), vert_pos),
                termion::color::Fg(termion::color::Reset),
                termion::color::Bg(termion::color::Reset),
                if Some(idx) == self.curr_server {
                    self.theme.servers.selected_text.clone()
                } else {
                    self.theme.servers.text.clone()
                },
                if let Server::Offline { .. } = server {
                    self.theme.servers.error_text.clone()
                } else {
                    Colour2 {
                        fg: Colour::Default,
                        bg: Colour::Default,
                    }
                },
                server.name().unwrap_or("Unknown Server"),
                " ".repeat(
                    self.theme.left_margin - server.name().unwrap_or("Unknown Server").len()
                ),
                termion::color::Fg(termion::color::Reset),
                termion::color::Bg(termion::color::Reset),
            )
            .unwrap();
            vert_pos += 1;
            idx += 1;
        }
    }

    pub fn draw_messages(&mut self, input_lines: u16) {
        let nothing = Vec::new();
        let messages = if let Some(curr_server) = self.curr_server {
            match &self.servers[curr_server] {
                Server::Online {
                    loaded_messages, ..
                } => loaded_messages,
                _ => &nothing,
            }
        } else {
            &nothing
        };

        // the actual height we have to work with, which is the window height
        // minus the number of lines the input buffer is taking up.
        let height = self.height - input_lines;
        let max_messages = height as usize
            - (self.theme.messages.border.top.width()
                + self.theme.messages.border.bottom.width()
                + self.theme.edit.border.bottom.width()
                + self.theme.edit.border.top.width()) as usize;
        let len = messages.len();

        //LOL not a good idea but it works
        let start_idx = len as isize - max_messages as isize + self.scroll as isize;
        let start_idx = if start_idx < 0 { 0 } else { start_idx as usize };

        if self.scroll > 0 {
            self.scroll = 0;
        }
        if (self.scroll + start_idx as isize) <= 0 {
            self.scroll = 0 - start_idx as isize;
        }

        let mut total_lines = 0;
        let max_chars: usize = self.width as usize - self.theme.left_margin - 4;
        let max_lines = height - 2;
        for msg in messages
            [(start_idx as isize + self.scroll) as usize..(len as isize + self.scroll) as usize]
            .iter()
        {
            total_lines += (msg.len() as f64 / max_chars as f64).ceil() as usize;
        }

        let mut line = total_lines as u16;

        let mut buffer: String = "".to_string();

        for message in messages
            [(start_idx as isize + self.scroll) as usize..(len as isize + self.scroll) as usize]
            .iter()
        {
            let num_lines: usize = (message.len() as f64 / max_chars as f64).ceil() as usize;
            for i in 0..num_lines {
                if line >= max_lines {
                    line -= 1;
                    continue;
                }
                let e = if (i + 1) * max_chars >= message.len() {
                    message.len()
                } else {
                    (i + 1) * max_chars
                };

                buffer.push_str(
                    &termion::cursor::Goto(
                        self.theme.left_margin as u16
                            + self.theme.servers.border.left.width()
                            + self.theme.servers.border.right.width()
                            + 2,
                        height - line - 1,
                    )
                    .to_string(),
                );
                buffer.push_str(Into::<FmtString>::into(&message[i * max_chars..e]).to_str());
                buffer.push_str(&" ".repeat(max_chars - message[i * max_chars..e].len()));
                line -= 1;
            }
        }
        // Fill any remaining space at the top with spaces, so that messages don't stick around in channels without a full history
        let spaces = " ".repeat(max_chars);
        line = max_lines - 1;
        while line > total_lines as u16 {
            buffer.push_str(
                &termion::cursor::Goto(
                    self.theme.left_margin as u16
                        + self.theme.servers.border.left.width()
                        + self.theme.servers.border.right.width()
                        + 2,
                    height - line - 1,
                )
                .to_string(),
            );
            buffer.push_str(&spaces);
            line -= 1;
        }
        write!(
            self.screen,
            "{}{}{}",
            termion::color::Fg(termion::color::Reset),
            termion::color::Bg(termion::color::Reset),
            buffer
        )
        .unwrap();
    }

    pub fn draw_border(&mut self) {
        if self.draw_border {
            self.draw_border = false;
            let (width, height) = termion::terminal_size().unwrap();
            let height = height - 1;
            let channels_height = self.theme.get_channels_height(height);
            let servers_height = self.theme.get_servers_height(height);

            let left_margin = self.theme.left_margin;

            let total_border_width = (self.theme.servers.border.left.width()
                + self.theme.servers.border.right.width()
                + self.theme.messages.border.left.width()
                + self.theme.messages.border.right.width())
                as usize;
            let space_padding = " ".repeat(width as usize - left_margin - total_border_width);
            let rs = termion::color::Fg(termion::color::Reset).to_string()
                + (&termion::color::Bg(termion::color::Reset).to_string());

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

    pub fn update_term_size(&mut self) {
        let (width, height) = termion::terminal_size().unwrap();

        if width < 32 || height < 8 {
            write!(self.screen, "Terminal size is too small lol").unwrap();
            return;
        }

        if self.width != width || self.height != height {
            self.draw_border = true; // actually redraw, don't use cached
            self.draw_border();
        }

        self.width = width;
        self.height = height;
    }

    pub fn draw_status_line(&mut self) {
        write!(
            self.screen,
            "{}{}{}",
            termion::cursor::Goto(1, self.height),
            termion::clear::CurrentLine,
            self.system_message.to_optimised_string()
        )
        .unwrap();
    }

    pub fn draw_input_buffer(&mut self) -> (u16, u16) {
        let begin_x = self.theme.left_margin as u16
            + self.theme.servers.border.left.width()
            + self.theme.servers.border.right.width()
            + 2; // ?
        let max_drawing_width = self.width
            - (begin_x
                + self.theme.messages.border.right.width()
                + self.theme.messages.border.left.width());

        // + 1 so that it adds a new line for the cursor to go on (as if the cursor is 1 extra char)
        let num_lines = ((self.buffer.data.chars().count() + 1) as f64 / max_drawing_width as f64)
            .ceil() as u16; // TODO kinda inefficient if you think about it

        let mut iter = self.buffer.data.chars();
        let mut pos = 0;
        let mut curr_line = 0;

        while pos < self.buffer.data.len() {
            let mut len = 0;
            for ch in iter.by_ref().take(max_drawing_width as usize) {
                len += ch.len_utf8();
            }
            let line = &self.buffer.data[pos..pos + len];
            write!(
                self.screen,
                "{}{}",
                termion::cursor::Goto(begin_x, self.height - 1 - num_lines + curr_line),
                line
            )
            .unwrap();

            // fill the rest of the line with spaces if it doesn't fill the whole line, to make sure nothing is left behind
            let curr_line_len = line.len();
            if curr_line_len < max_drawing_width as usize {
                write!(
                    self.screen,
                    "{}",
                    " ".repeat(max_drawing_width as usize - curr_line_len)
                )
                .unwrap();
            }

            pos += len;
            curr_line += 1;
        }

        // special case: if we just started a new line & it has nothing on it,
        // clear the line, because there isn't any bit of the line to trigger the
        // previous line clearing thingie
        if curr_line < num_lines {
            write!(
                self.screen,
                "{}{}",
                termion::cursor::Goto(begin_x, self.height - 2),
                " ".repeat(max_drawing_width as usize)
            )
            .unwrap();
        }
        (num_lines, max_drawing_width)
    }

    pub fn draw_prompt(&mut self) {
        let y = self.height - self.prompt.as_ref().unwrap().height() - 1;
        self.prompt.as_ref().unwrap().draw(
            &mut self.screen,
            self.theme.left_margin as u16 + 4,
            y,
            &self.theme,
        );
    }

    pub fn draw_all(&mut self) {
        self.update_term_size();
        write!(
            self.screen,
            "{}{}{}",
            termion::cursor::Goto(1, self.height),
            termion::clear::CurrentLine,
            self.system_message.to_optimised_string()
        )
        .unwrap();

        match self.mode {
            Mode::Messages => {
                let (num_input_lines, max_drawing_width) = self.draw_input_buffer();
                if self.servers.len() > 0 {
                    self.draw_messages(num_input_lines);
                    self.draw_servers();
                }
                let cursor_x_pos = self.buffer.edit_position % max_drawing_width as usize;
                let cursor_y_pos = self.buffer.edit_position / max_drawing_width as usize;

                write!(
                    self.screen,
                    "{}",
                    termion::cursor::Goto(
                        self.theme.left_margin as u16
                            + self.theme.servers.border.left.width()
                            + self.theme.servers.border.right.width()
                            + 2
                            + cursor_x_pos as u16,
                        self.height - 1 - num_input_lines.max(1) + cursor_y_pos as u16
                    )
                )
                .unwrap();
            }
            Mode::NewServer => {
                if self.servers.len() > 0 {
                    self.draw_servers();
                }
                self.draw_prompt();
            }
            Mode::Settings => {}
        }
    }
}
