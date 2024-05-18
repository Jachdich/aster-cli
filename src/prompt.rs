use crate::drawing::Theme;
use termion::event::{Event, Key, MouseButton, MouseEvent};

enum Selection {
    Field(usize),
    Button(usize),
}

struct EditBuffer {
    data: String,
    edit_position: usize,
}

impl EditBuffer {
    fn new(default: String) -> Self {
        Self {
            edit_position: default.len(),
            data: default,
        }
    }
    fn push(&mut self, c: char) {
        let mut new_data = self.data.chars().collect::<Vec<char>>();
        new_data.insert(self.edit_position, c);
        self.data = new_data.iter().collect();
        self.edit_position += 1;
    }
    fn pop(&mut self) {
        if self.edit_position > 0 {
            let mut new_data = self.data.chars().collect::<Vec<char>>();
            new_data.remove(self.edit_position - 1);
            self.data = new_data.iter().collect();
            self.edit_position -= 1;
        }
    }
    fn left(&mut self) {
        if self.edit_position > 0 {
            self.edit_position -= 1;
        }
    }
    fn right(&mut self) {
        if self.edit_position < self.data.len() {
            self.edit_position += 1;
        }
    }
}

pub struct Prompt {
    name: &'static str,
    fields: Vec<PromptField>,
    buttons: Vec<&'static str>,
    buffers: Vec<EditBuffer>,
    selected: Selection,
}

pub enum PromptEvent {
    ButtonPressed(&'static str),
}

#[derive(Debug)]
pub enum FieldError {
    WrongType,
    NoSuchField,
}

impl Prompt {
    pub fn new(name: &'static str, fields: Vec<PromptField>, buttons: Vec<&'static str>) -> Self {
        let num_fields = fields.len();
        let buffers = fields
            .iter()
            .map(|field| EditBuffer::new(field.default_string()))
            .collect();
        Self {
            name,
            fields,
            buttons,
            buffers,
            selected: Selection::Field(0),
        }
    }

    fn increment_selection(&mut self) {
        match self.selected {
            Selection::Field(idx) => {
                if idx + 1 < self.fields.len() {
                    self.selected = Selection::Field(idx + 1)
                } else {
                    self.selected = Selection::Button(0)
                }
            }
            Selection::Button(idx) => {
                if idx + 1 < self.buttons.len() {
                    self.selected = Selection::Button(idx + 1);
                }
            }
        }
    }
    fn decrement_selection(&mut self) {
        match self.selected {
            Selection::Button(idx) => {
                if idx > 0 {
                    self.selected = Selection::Button(idx - 1)
                } else {
                    self.selected = Selection::Field(self.fields.len() - 1);
                }
            }
            Selection::Field(idx) => {
                if idx > 0 {
                    self.selected = Selection::Field(idx - 1);
                }
            }
        }
    }

    pub fn handle_event(&mut self, event: Event) -> Option<PromptEvent> {
        match event.clone() {
            Event::Key(Key::Char('\n')) => match self.selected {
                Selection::Button(idx) => {
                    return Some(PromptEvent::ButtonPressed(self.buttons[idx]))
                }
                Selection::Field(idx) => self.increment_selection(),
            },
            Event::Key(Key::Down) => self.increment_selection(),
            Event::Key(Key::Up) => self.decrement_selection(),

            Event::Key(Key::Right) => match self.selected {
                Selection::Field(idx) => self.buffers[idx].right(),
                Selection::Button(_) => self.increment_selection(),
            },
            Event::Key(Key::Left) => match self.selected {
                Selection::Field(idx) => self.buffers[idx].left(),
                Selection::Button(_) => self.decrement_selection(),
            },

            Event::Key(Key::Backspace) => {
                if let Selection::Field(idx) = self.selected {
                    self.buffers[idx].pop();
                }
            }

            Event::Key(Key::Char(c)) => {
                if let Selection::Field(idx) = self.selected {
                    self.buffers[idx].push(c);
                }
            }

            _ => (),
        }
        None
    }

    pub fn height(&self) -> u16 {
        self.fields.len() as u16 + 1
    }

    pub fn draw<W: std::io::Write>(&self, screen: &mut W, x: u16, y: u16, theme: &Theme) {
        // let cursor_x: u16;
        // let cursor_y: u16;

        let mut idx = 0;
        for (field, buffer) in std::iter::zip(self.fields.iter(), self.buffers.iter()) {
            if let Selection::Field(idx) = self.selected {
                write!(
                    screen,
                    "{}{}{} : {}{}{}",
                    termion::cursor::Goto(x, y + idx as u16),
                    theme.servers.selected_text,
                    field.name(),
                    buffer.data,
                    termion::color::Fg(termion::color::Reset),
                    termion::color::Bg(termion::color::Reset),
                )
                .unwrap();
            } else {
                write!(
                    screen,
                    "{}{} : {}",
                    termion::cursor::Goto(x, y + idx as u16),
                    field.name(),
                    buffer.data,
                )
                .unwrap();
            }
            idx += 1;
        }

        // write!(
        //     screen,
        //     "{}{}ip   : {}{}{}{}{}port : {}{}{}{}{}uuid : {}{}{}{}{}[connect]{}{} {}[cancel]{}{}{}",
        //     if self.sel_idx == 0 {
        //         self.theme.servers.selected_text.clone()
        //     } else {
        //         Colour::new()
        //     },
        //     termion::cursor::Goto(self.theme.left_margin as u16 + 4, height - 4),
        //     termion::color::Fg(termion::color::Reset),
        //     termion::color::Bg(termion::color::Reset),
        //     self.ip_buffer,
        //     if self.sel_idx == 1 {
        //         self.theme.servers.selected_text.clone()
        //     } else {
        //         Colour::new()
        //     },
        //     termion::cursor::Goto(self.theme.left_margin as u16 + 4, height - 3),
        //     termion::color::Fg(termion::color::Reset),
        //     termion::color::Bg(termion::color::Reset),
        //     self.port_buffer,
        //     if self.sel_idx == 2 {
        //         self.theme.servers.selected_text.clone()
        //     } else {
        //         Colour::new()
        //     },
        //     termion::cursor::Goto(self.theme.left_margin as u16 + 4, height - 2),
        //     termion::color::Fg(termion::color::Reset),
        //     termion::color::Bg(termion::color::Reset),
        //     self.uuid_buffer,
        //     termion::cursor::Goto(self.theme.left_margin as u16 + 4, height - 1),
        //     if self.sel_idx == 3 {
        //         self.theme.servers.selected_text.clone()
        //     } else {
        //         Colour::new()
        //     },
        //     termion::color::Bg(termion::color::Reset),
        //     termion::color::Fg(termion::color::Reset),
        //     if self.sel_idx == 4 {
        //         self.theme.servers.selected_text.clone()
        //     } else {
        //         Colour::new()
        //     },
        //     termion::color::Bg(termion::color::Reset),
        //     termion::color::Fg(termion::color::Reset),
        //     termion::cursor::Goto(cur_x, cur_y),
        // )
        // .unwrap();
    }

    fn index_from_str(&self, val: &str) -> Option<usize> {
        self.fields.iter().position(|field| field.name() == val)
    }

    pub fn get_str(&self, key: &str) -> Result<&str, FieldError> {
        let idx = self.index_from_str(key).ok_or(FieldError::NoSuchField)?;
        if let PromptField::String { .. } = self.fields[idx] {
            Ok(&self.buffers[idx].data)
        } else {
            Err(FieldError::WrongType)
        }
    }
    pub fn get_u16(&self, key: &str) -> Result<u16, FieldError> {
        let idx = self.index_from_str(key).ok_or(FieldError::NoSuchField)?;
        if let PromptField::U16 { .. } = self.fields[idx] {
            Ok(self.buffers[idx].data.parse().unwrap()) // This unwrap **SHOULD** be okay, as we should have prevented anything invalid from being entered in the first place
        } else {
            Err(FieldError::WrongType)
        }
    }
    pub fn get_i64(&self, key: &str) -> Result<i64, FieldError> {
        let idx = self.index_from_str(key).ok_or(FieldError::NoSuchField)?;
        if let PromptField::I64 { .. } = self.fields[idx] {
            Ok(self.buffers[idx].data.parse().unwrap()) // Likewise
        } else {
            Err(FieldError::WrongType)
        }
    }
}

pub enum PromptField {
    String {
        name: &'static str,
        default: Option<String>,
    },
    U16 {
        name: &'static str,
        default: Option<u16>,
    },
    I64 {
        name: &'static str,
        default: Option<i64>,
    },
}

impl PromptField {
    #[rustfmt::skip]
    fn default_string(&self) -> String {
        match self {
            Self::String { default: Some(d), .. } => d.clone(),
            Self::U16 { default: Some(d), .. } => d.to_string(),
            Self::I64 { default: Some(d), .. } => d.to_string(),
            _ => "".to_owned(),
        }
    }
    fn name(&self) -> &str {
        match self {
            Self::String { name, .. } | Self::U16 { name, .. } | Self::I64 { name, .. } => name,
        }
    }
}
