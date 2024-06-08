use crate::drawing::Theme;
use termion::event::{Event, Key, MouseButton, MouseEvent};

enum Selection {
    Field(usize),
    Button(usize),
}

pub struct EditBuffer {
    pub data: String,
    pub edit_position: usize,
}

impl EditBuffer {
    pub fn new(default: String) -> Self {
        Self {
            edit_position: default.len(),
            data: default,
        }
    }
    pub fn push(&mut self, c: char) {
        let i = if self.edit_position == 0 {
            0
        } else {
            self.data
                .char_indices()
                .nth(self.edit_position - 1)
                .unwrap()
                .0
                + 1 // edit position should always be in range...
        };
        self.data.insert(i, c);
        self.edit_position += 1;
    }
    pub fn pop(&mut self) {
        if self.edit_position > 0 {
            let (i, _) = self
                .data
                .char_indices()
                .nth(self.edit_position - 1)
                .unwrap(); // edit position should always be in range...
            self.data.remove(i);
            self.edit_position -= 1;
        }
    }
    pub fn left(&mut self) {
        if self.edit_position > 0 {
            self.edit_position -= 1;
        }
    }

    pub fn right(&mut self) {
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

    fn validate_field(&self, idx: usize) -> bool {
        let data = &self.buffers[idx].data;
        match self.fields[idx] {
            PromptField::String { .. } => true,
            PromptField::U16 { .. } => data.parse::<u16>().is_ok(),
            PromptField::I64 { .. } => data.parse::<i64>().is_ok(),
        }
    }

    pub fn handle_event(&mut self, event: Event) -> Option<PromptEvent> {
        match event.clone() {
            Event::Key(Key::Char('\n')) => match self.selected {
                Selection::Button(idx) => {
                    return Some(PromptEvent::ButtonPressed(self.buttons[idx]))
                }
                Selection::Field(_) => self.increment_selection(),
            },
            Event::Key(Key::Down) | Event::Key(Key::Char('\t')) => self.increment_selection(),
            Event::Key(Key::Up) | Event::Key(Key::BackTab) => self.decrement_selection(),

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

                    // if the user inputted something wrong, it's best to check after adding and just remove the lsat char if necessary
                    if !self.validate_field(idx) {
                        self.buffers[idx].pop();
                    }
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
        let align = self
            .fields
            .iter()
            .map(|field| field.name().len())
            .max()
            .unwrap(); // unwrap: we must have at least one field
        let mut idx = 0;
        for (field, buffer) in std::iter::zip(self.fields.iter(), self.buffers.iter()) {
            let idx2 = if let Selection::Field(idx2) = self.selected {
                idx2
            } else {
                usize::MAX // I hope this will never equal idx
            };

            if idx2 == idx {
                write!(
                    screen,
                    "{}{}{}{}{}{}: {}",
                    termion::cursor::Goto(x, y + idx as u16),
                    theme.servers.selected_text,
                    field.name(),
                    " ".repeat(align - field.name().len()),
                    termion::color::Fg(termion::color::Reset),
                    termion::color::Bg(termion::color::Reset),
                    buffer.data,
                )
                .unwrap();
            } else {
                write!(
                    screen,
                    "{}{}{}: {}",
                    termion::cursor::Goto(x, y + idx as u16),
                    field.name(),
                    " ".repeat(align - field.name().len()),
                    buffer.data,
                )
                .unwrap();
            }
            idx += 1;
        }

        write!(
            screen,
            "{}",
            termion::cursor::Goto(x, y + self.fields.len() as u16)
        )
        .unwrap();

        idx = 0;
        for button in &self.buttons {
            let idx2 = if let Selection::Button(idx2) = self.selected {
                idx2
            } else {
                usize::MAX // I hope this will never equal idx
            };
            if idx2 == idx {
                write!(
                    screen,
                    "[{}{}{}{}] ",
                    theme.servers.selected_text,
                    button,
                    termion::color::Fg(termion::color::Reset),
                    termion::color::Bg(termion::color::Reset),
                )
                .unwrap();
            } else {
                write!(screen, "[{}] ", button).unwrap();
            }
            idx += 1;
        }

        write!(
            screen,
            "{}",
            if let Selection::Field(idx) = self.selected {
                let sel_x = x + align as u16 + 2 + self.buffers[idx].edit_position as u16;
                let sel_y = y + idx as u16;
                termion::cursor::Goto(sel_x, sel_y)
            } else {
                termion::cursor::Goto(1, 1)
            }
        )
        .unwrap();
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
