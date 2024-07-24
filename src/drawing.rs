use crate::gui::Gui;
use crate::{Focus, Mode};
use fmtstring::{Colour, FmtChar};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fmt;
use std::io::Write;

static BUILTIN_THEMES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("default", include_str!("../themes/default.json"));
    m.insert("cmus", include_str!("../themes/cmus.json"));
    m
});

fn centred(text: &str, width: usize) -> String {
    format!("{: ^1$}", text, width)
}

fn fmtchar_from_json_impl(val: &serde_json::Value) -> Option<FmtChar> {
    Some(FmtChar {
        ch: val[0].as_str()?.chars().next()?,
        fg: parse_colour(val[1].as_str()?),
        bg: parse_colour(val[2].as_str()?),
    })
}

pub fn fmtchar_from_json(val: &serde_json::Value) -> OptionalFmtChar {
    OptionalFmtChar(fmtchar_from_json_impl(val))
}

fn parse_colour(inp: &str) -> Colour {
    if inp.starts_with('#') {
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
    pub sidebar_width: usize,
}

fn get_or<'a>(
    name: &str,
    main: &'a serde_json::Value,
    aux: &'a serde_json::Value,
) -> &'a serde_json::Value {
    if main[name].is_null() {
        &aux[name]
    } else {
        &main[name]
    }
}

impl ThemedArea {
    pub fn new(cfg: &serde_json::Value, fallback: &serde_json::Value) -> Self {
        ThemedArea {
            text: Colour2 {
                fg: parse_colour(get_or("text-foreground", cfg, fallback).as_str().unwrap()),
                bg: parse_colour(get_or("text-background", cfg, fallback).as_str().unwrap()),
            },
            selected_text: Colour2 {
                fg: parse_colour(
                    get_or("selected-text-foreground", cfg, fallback)
                        .as_str()
                        .unwrap(),
                ),
                bg: parse_colour(
                    get_or("selected-text-background", cfg, fallback)
                        .as_str()
                        .unwrap(),
                ),
            },
            unfocussed_selected_text: Colour2 {
                fg: parse_colour(
                    get_or("unfocussed-selected-text-foreground", cfg, fallback)
                        .as_str()
                        .unwrap(),
                ),
                bg: parse_colour(
                    get_or("unfocussed-selected-text-background", cfg, fallback)
                        .as_str()
                        .unwrap(),
                ),
            },
            error_text: Colour2 {
                fg: parse_colour(
                    get_or("error-text-foreground", cfg, fallback)
                        .as_str()
                        .unwrap(),
                ),
                bg: parse_colour(
                    get_or("error-text-background", cfg, fallback)
                        .as_str()
                        .unwrap(),
                ),
            },
            system_message: Colour2 {
                fg: parse_colour(
                    get_or("system-message-foreground", cfg, fallback)
                        .as_str()
                        .unwrap(),
                ),
                bg: parse_colour(
                    get_or("system-message-background", cfg, fallback)
                        .as_str()
                        .unwrap(),
                ),
            },

            border: ThemedBorder {
                tl: fmtchar_from_json(get_or("border-tl", cfg, fallback)),
                tr: fmtchar_from_json(get_or("border-tr", cfg, fallback)),
                bl: fmtchar_from_json(get_or("border-bl", cfg, fallback)),
                br: fmtchar_from_json(get_or("border-br", cfg, fallback)),
                top: fmtchar_from_json(get_or("border-top", cfg, fallback)),
                bottom: fmtchar_from_json(get_or("border-bottom", cfg, fallback)),
                left: fmtchar_from_json(get_or("border-left", cfg, fallback)),
                right: fmtchar_from_json(get_or("border-right", cfg, fallback)),
                bottom_split: fmtchar_from_json(get_or("border-bottom-split", cfg, fallback)),
                top_split: fmtchar_from_json(get_or("border-top-split", cfg, fallback)),
                left_split: fmtchar_from_json(get_or("border-left-split", cfg, fallback)),
                right_split: fmtchar_from_json(get_or("border-right-split", cfg, fallback)),
            },
        }
    }
}

impl Theme {
    pub fn new(name: &str) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        // TODO when rust 1.79 becomes old enough, we can drop this .to_owned() and borrow the std::fs::read
        let file_contents = if let Some(c) = BUILTIN_THEMES.get(name) {
            (*c).to_owned()
        } else {
            std::fs::read_to_string(format!("themes/{}.json", name))?
        };

        let totalcfg: serde_json::Value = serde_json::from_str(&file_contents)?;

        let servers = ThemedArea::new(&totalcfg["servers"], &totalcfg["global"]);
        let channels = ThemedArea::new(&totalcfg["channels"], &totalcfg["global"]);
        let edit = ThemedArea::new(&totalcfg["edit"], &totalcfg["global"]);
        let messages = ThemedArea::new(&totalcfg["messages"], &totalcfg["global"]);
        let status = ThemedArea::new(&totalcfg["status"], &totalcfg["global"]);

        let sidebar_width = 32;

        Ok(Theme {
            sidebar_width,
            servers,
            channels,
            edit,
            messages,
            status,
        })
    }

    pub fn get_left_margin(&self) -> usize {
        self.sidebar_width
            + u16::max(
                self.servers.border.left.width(),
                self.channels.border.right.width(),
            ) as usize
            + u16::max(
                self.servers.border.right.width(),
                self.channels.border.right.width(),
            ) as usize
    }

    pub fn get_list_height(&self, height: u16) -> usize {
        (height
            - 1
            - self.channels.border.top.width()
            - self.channels.border.bottom.width() * 2
            - self.servers.border.top.width()
            - self.servers.border.bottom.width()) as usize
    }

    pub fn get_servers_start_pos(&self) -> usize {
        (self.channels.border.top.width() + 2 + self.channels.border.bottom.width() * 2) as usize
    }

    pub fn get_channels_start_pos(&self, height: u16) -> usize {
        self.get_servers_start_pos()
            + self.get_servers_height(height)
            + self.channels.border.bottom.width() as usize
            + self.servers.border.top.width() as usize
    }

    pub fn get_servers_height(&self, height: u16) -> usize {
        let list_height = self.get_list_height(height);
        if list_height % 2 == 0 {
            list_height / 2 - 1
        } else {
            list_height / 2
        }
    }
    pub fn get_channels_height(&self, height: u16) -> usize {
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
            (format!("{}", c.ch)).repeat(n),
            termion::color::Fg(termion::color::Reset),
            termion::color::Bg(termion::color::Reset)
        )
    } else {
        "".into()
    }
}

use std::fmt::Write as _;

impl Gui {
    pub fn draw_servers<W: Write>(&self, screen: &mut W) {
        let a = std::time::Instant::now();
        let (_width, height) = termion::terminal_size().unwrap();
        let height = height - 1;

        let mut vert_pos = self.theme.get_servers_start_pos() as u16;
        let mut idx = 0;
        let mut buffer = String::new();
        if let Some(curr_server) = self.curr_server {
            if let Ok(ref net) = &self.servers[curr_server].network {
                for channel in &net.channels {
                    write!(
                        buffer,
                        "{}{}{}{}{}{}{}{}",
                        termion::cursor::Goto(
                            1 + self.theme.channels.border.left.width(),
                            vert_pos
                        ),
                        termion::color::Fg(termion::color::Reset),
                        termion::color::Bg(termion::color::Reset),
                        if net.curr_channel.is_some_and(|cc| idx == cc) {
                            if self.focus == Focus::ChannelList {
                                &self.theme.channels.selected_text
                            } else {
                                &self.theme.channels.unfocussed_selected_text
                            }
                        } else {
                            &self.theme.channels.text
                        },
                        channel.name,
                        " ".repeat(self.theme.sidebar_width - channel.name.len()),
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
                buffer,
                "{}{}",
                termion::cursor::Goto(1 + self.theme.servers.border.left.width(), vert_pos),
                " ".repeat(self.theme.sidebar_width),
            )
            .unwrap();
            vert_pos += 1;
        }
        vert_pos += 1;
        // vert_pos = self.theme.get_channels_start_pos(height) as u16 + 1;
        idx = 0;
        for server in &self.servers {
            let backup_name = format!("<{}:{}>", server.ip, server.port);
            let display_name = server.name.as_ref().unwrap_or(&backup_name);
            write!(
                buffer,
                "{}{}{}{}{}{}{}{}",
                termion::cursor::Goto(1 + self.theme.servers.border.left.width(), vert_pos),
                termion::color::Fg(termion::color::Reset),
                termion::color::Bg(termion::color::Reset),
                if Some(idx) == self.curr_server {
                    if self.focus == Focus::ServerList {
                        &self.theme.servers.selected_text
                    } else {
                        &self.theme.servers.unfocussed_selected_text
                    }
                } else if !server.is_online() {
                    &self.theme.servers.error_text
                } else {
                    &self.theme.servers.text
                },
                display_name,
                " ".repeat(self.theme.sidebar_width - display_name.len()),
                termion::color::Fg(termion::color::Reset),
                termion::color::Bg(termion::color::Reset),
            )
            .unwrap();
            vert_pos += 1;
            idx += 1;
        }
        log_time(a.elapsed(), "draw_channels logic");
        let a = std::time::Instant::now();
        write!(screen, "{}", buffer).unwrap();
        log_time(a.elapsed(), "draw_channels write");
    }

    pub fn draw_messages<W: Write>(&mut self, screen: &mut W, input_lines: u16) {
        let a = std::time::Instant::now();
        let mut nothing = Vec::new();
        let messages = if let Some(curr_server) = self.curr_server {
            self.servers[curr_server]
                .network
                .as_mut()
                .map(|net| &mut net.loaded_messages)
                .unwrap_or(&mut nothing)
        } else {
            &mut nothing
        };

        // the actual height we have to work with, which is the window height
        // minus the number of lines the input buffer is taking up.
        // minus the various borders
        let height = self.height - input_lines;
        let max_messages = height as usize
            - (self.theme.messages.border.top.width()
                + self.theme.messages.border.bottom.width()
                + self.theme.edit.border.bottom.width()
                + self.theme.edit.border.top.width()) as usize;
        let len = messages.len();

        //LOL not a good idea but it works
        let start_idx = len as isize - max_messages as isize + self.scroll;
        let start_idx = if start_idx < 0 { 0 } else { start_idx as usize };

        if self.scroll > 0 {
            self.scroll = 0;
        }
        if (self.scroll + start_idx as isize) <= 0 {
            self.scroll = 0 - start_idx as isize;
        }

        let max_chars: usize = self.width as usize
            - self.theme.get_left_margin()
            - self.theme.messages.border.left.width() as usize
            - self.theme.messages.border.right.width() as usize
            - 1; // 1 space of padding on the left

        // + 1 because zero-based indexing
        // + 1 because 1 space of padding on the left
        let message_start_x = self.theme.get_left_margin() as u16 + 1 + 1;

        let max_lines = height - 2;
        let total_lines = messages
            [(start_idx as isize + self.scroll) as usize..(len as isize + self.scroll) as usize]
            .iter()
            .fold(0, |acc, msg| acc + msg.lines.len()); // lovely functional goodness

        let mut line = total_lines as u16;

        let mut buffer: String = "".to_string();

        for (i, message) in messages
            [(start_idx as isize + self.scroll) as usize..(len as isize + self.scroll) as usize]
            .iter_mut()
            .enumerate()
        {
            let total_idx = i as isize + start_idx as isize + self.scroll;
            let highlight = self
                .selected_message
                .is_some_and(|x| x as isize == len as isize - total_idx);

            let num_lines: usize = message.lines.len();
            for i in 0..num_lines {
                if line >= max_lines {
                    line -= 1;
                    continue;
                }

                if highlight {
                    buffer.push_str(termion::style::Bold.as_ref());
                }

                buffer.push_str(
                    &termion::cursor::Goto(message_start_x, height - line - 1).to_string(),
                );
                buffer.push_str(message.lines[i].to_str());
                buffer.push_str(&" ".repeat(max_chars - message.lines[i].len()));

                if highlight {
                    buffer.push_str(termion::style::NoBold.as_ref());
                }
                line -= 1;
            }
        }
        // Fill any remaining space at the top with spaces, so that messages don't stick around in channels without a full history
        let spaces = " ".repeat(max_chars);
        line = max_lines - 1;
        while line > total_lines as u16 {
            buffer.push_str(&termion::cursor::Goto(message_start_x, height - line - 1).to_string());
            buffer.push_str(&spaces);
            line -= 1;
        }

        log_time(a.elapsed(), "draw_messages logic");
        let a = std::time::Instant::now();
        write!(
            screen,
            "{}{}{}",
            termion::color::Fg(termion::color::Reset),
            termion::color::Bg(termion::color::Reset),
            buffer
        )
        .unwrap();
        log_time(a.elapsed(), "draw_messages write");
    }

    pub fn draw_status_line<W: Write>(&self, screen: &mut W) {
        let a = std::time::Instant::now();
        let mut buffer = String::new();
        write!(
            buffer,
            "{}{}{}{}{}{}",
            termion::cursor::Goto(1, self.height),
            self.theme.status.system_message,
            self.system_message,
            " ".repeat(self.width as usize - self.system_message.len()),
            termion::color::Bg(termion::color::Reset),
            termion::color::Fg(termion::color::Reset),
        )
        .unwrap();
        log_time(a.elapsed(), "draw_status_line logic");
        let a = std::time::Instant::now();
        write!(screen, "{}", buffer).unwrap();
        log_time(a.elapsed(), "draw_status_line write");
    }

    pub fn draw_input_buffer<W: Write>(&self, screen: &mut W) -> (u16, u16) {
        let a = std::time::Instant::now();
        let mut buffer = String::new();
        let begin_x = self.theme.sidebar_width as u16
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
                buffer,
                "{}{}",
                termion::cursor::Goto(begin_x, self.height - 1 - num_lines + curr_line),
                line
            )
            .unwrap();

            // fill the rest of the line with spaces if it doesn't fill the whole line, to make sure nothing is left behind
            let curr_line_len = line.len();
            if curr_line_len < max_drawing_width as usize {
                write!(
                    buffer,
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
                screen,
                "{}{}",
                termion::cursor::Goto(begin_x, self.height - 2),
                " ".repeat(max_drawing_width as usize)
            )
            .unwrap();
        }
        log_time(a.elapsed(), "draw_input_buffer logic");
        let a = std::time::Instant::now();
        write!(screen, "{}", buffer).unwrap();
        log_time(a.elapsed(), "draw_input_buffer write");
        (num_lines, max_drawing_width)
    }

    pub fn draw_prompt<W: Write>(&self, screen: &mut W) {
        let y = self.height - self.prompt.as_ref().unwrap().height() - 1;
        self.prompt.as_ref().unwrap().draw(
            screen,
            self.theme.sidebar_width as u16 + 4,
            y,
            &self.theme,
        );
    }

    pub fn draw_all<W: Write>(&mut self, screen: &mut W) {
        self.draw_status_line(screen);

        match self.mode {
            Mode::Messages | Mode::EditMessage => {
                let (num_input_lines, max_drawing_width) = self.draw_input_buffer(screen);
                if !self.servers.is_empty() {
                    self.draw_messages(screen, num_input_lines);
                    self.draw_servers(screen);
                }
                let cursor_x_pos = self.buffer.edit_position % max_drawing_width as usize;
                let cursor_y_pos = self.buffer.edit_position / max_drawing_width as usize;

                write!(
                    screen,
                    "{}",
                    termion::cursor::Goto(
                        self.theme.sidebar_width as u16
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
                if !self.servers.is_empty() {
                    self.draw_servers(screen);
                }
                self.draw_prompt(screen);
            }
            Mode::Settings => {}
        }
    }
}

pub fn draw_border(theme: &Theme) -> String {
    let a = std::time::Instant::now();
    let (width, height) = termion::terminal_size().unwrap();
    let height = height - 1;
    let channels_height = theme.get_channels_height(height);
    let servers_height = theme.get_servers_height(height);

    let left_margin = theme.sidebar_width;

    let total_border_width = (theme.servers.border.left.width()
        + theme.servers.border.right.width()
        + theme.messages.border.left.width()
        + theme.messages.border.right.width()) as usize;
    let space_padding = " ".repeat(width as usize - left_margin - total_border_width);
    let rs = termion::color::Fg(termion::color::Reset).to_string()
        + (&termion::color::Bg(termion::color::Reset).to_string());

    let res = format!("{0}{1}{sttl}{2}{3}\r\n{stleft}{rs}{4}{stright}{mleft}{space_padding}{mright}\r\n{stleft}{rs}{5}{stright}{mleft}{space_padding}{mright}\r\n{6}{7}{8}{9}{sbl}{10}{11}",
/*0*/       termion::cursor::Goto(1, 1),
/*1*/       "",
/*2*/       border_rep(&theme.status.border.top, left_margin),

/*3*/       if theme.messages.border.left.width() == 0 || theme.channels.border.right.width() == 0 {
            format!("{sttop_split}{}{mtr}", 
                border_rep(&theme.messages.border.top, width as usize - left_margin - total_border_width),

                mtr = theme.messages.border.tr,
                sttop_split = theme.status.border.top_split,
            )
        } else {
            format!("{sttr}{mtl}{}{mtr}", 
                border_rep(&theme.messages.border.top, width as usize - left_margin - total_border_width),

                mtr = theme.messages.border.tr,
                mtl = theme.messages.border.tl,
                sttr = theme.status.border.tr,
            )
        },

/*4*/       centred("Connected to", left_margin),

/*5*/       centred("cospox.com", theme.sidebar_width),

/*6*/       if theme.channels.border.bottom.width() > 0 {
            format!("{stleft_split}{}{stright_split}{mleft}{}{mright}\r\n",
                border_rep(&theme.channels.border.bottom, left_margin),
                space_padding,
                stleft_split = theme.status.border.left_split,
                stright_split = theme.status.border.right_split,
                mright = theme.messages.border.right,
                mleft = theme.messages.border.left,
            )
        } else {
            "".to_string()
        },

/*7*/       format!("{cleft}{rs}{}{cright}{rs}{mleft}{}{mright}\r\n",
            " ".repeat(left_margin),
            space_padding,
            rs = rs,
            cleft = theme.channels.border.left,
            cright = theme.channels.border.right,
            mright = theme.messages.border.right,
            mleft = theme.messages.border.left,
        ).repeat(channels_height),

/*8*/       if theme.channels.border.bottom.width() > 0 && theme.servers.border.top.width() > 0 {
            format!("{cbl}{}{cbr}{mleft}{}{mright}\r\n{stl}{}{str}{mleft}{}{mright}\r\n",
                border_rep(&theme.channels.border.bottom, left_margin),
                space_padding,
                border_rep(&theme.servers.border.top, left_margin),
                space_padding,
                cbl = theme.channels.border.bl,
                cbr = theme.channels.border.br,
                str = theme.servers.border.tr,
                stl = theme.servers.border.tl,
                mright = theme.messages.border.right,
                mleft = theme.messages.border.left,
            )
        } else if theme.channels.border.bottom.width() > 0 {
            format!("{cleft_split}{}{cright_split}{mleft}{}{mright}\r\n", 
                border_rep(&theme.channels.border.bottom, left_margin),
                space_padding,
                cleft_split = theme.channels.border.left_split,
                cright_split = theme.channels.border.right_split,
                mright = theme.messages.border.right,
                mleft = theme.messages.border.left,
            )
        } else if theme.servers.border.top.width() > 0 {
            format!("{sleft_split}{}{sright_split}{mleft}{}{mright}\r\n", 
                border_rep(&theme.servers.border.top, left_margin),
                space_padding,
                sleft_split = theme.servers.border.left_split,
                sright_split = theme.servers.border.right_split,
                mright = theme.messages.border.right,
                mleft = theme.messages.border.left,
            )
        } else {
            "".to_string()
        },

/*9*/       format!("{sleft}{rs}{}{sright}{rs}{mleft}{}{mright}\r\n", 
            " ".repeat(theme.sidebar_width), 
            space_padding,
            rs = rs,
            sleft = theme.servers.border.left,
            sright = theme.servers.border.right,
            mright = theme.messages.border.right,
            mleft = theme.messages.border.left,
        ).repeat(servers_height),

/*10*/      border_rep(&theme.servers.border.bottom, left_margin),

/*11*/      if theme.messages.border.left.width() == 0 || theme.servers.border.right.width() == 0 {
            format!("{sbottom_split}{}{mbr}", 
                border_rep(&theme.messages.border.bottom, width as usize - left_margin - total_border_width),

                mbr = theme.messages.border.br,
                sbottom_split = theme.servers.border.bottom_split,
            )
        } else {
            format!("{sbr}{mbl}{}{mbr}", 
                border_rep(&theme.messages.border.bottom, width as usize - left_margin - total_border_width),

                mbr = theme.messages.border.br,
                mbl = theme.messages.border.bl,
                sbr = theme.servers.border.br,
            )
        },

        //stl = theme.servers.border.tl,
        sttl = theme.status.border.tl,
        sbl = theme.servers.border.bl,
        //ctop_split = theme.channels.border.top_split,
        //sleft = theme.servers.border.left,
        //cleft = theme.channels.border.left,
        stleft = theme.status.border.left,
        //sright = theme.servers.border.right,
        //sleft_split = theme.servers.border.left_split,
        //sright_split = theme.servers.border.right_split,
        //sbottom_split = theme.servers.border.bottom_split,
        mright = theme.messages.border.right,
        //cright = theme.channels.border.right,
        stright = theme.status.border.right,
        mleft = theme.messages.border.left,
        rs = rs,
        //mbr = theme.messages.border.br,
        //mtr = theme.messages.border.tr,
    );
    log_time(a.elapsed(), "draw_border logic");
    res
}

pub fn log_time(t: std::time::Duration, s: &'static str) {
    std::fs::OpenOptions::new().write(true).append(true).open("times.txt").unwrap().write_fmt(format_args!("{}: {:?}\n", s, t)).unwrap();
}
