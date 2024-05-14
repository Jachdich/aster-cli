extern crate dirs;
extern crate termion;
extern crate tokio;
use termion::event::Event;

use crate::termion::input::TermRead;
use std::io::stdin;

mod drawing;
mod events;
mod gui;
mod parser;
mod server;

use drawing::FmtString;
use gui::GUI;

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    NewServer,
    Messages,
    Settings,
}

#[derive(Clone, Copy)]
pub enum Focus {
    ServerList,
    ChannelList,
    Edit,
    Messages,
}

#[derive(Debug)]
pub struct User {
    nick: String,
    passwd: String,
    pfp_b64: String,
    uuid: u64,
}

pub enum Message {
    User {
        author: u64,
        content: FmtString,
        // time: chrono::DateTime,
    },
    System(FmtString),
}

pub enum LocalMessage {
    Keyboard(Event),
    Network(String, usize),
}

impl User {
    fn from_json(val: &json::JsonValue) -> Self {
        User {
            nick: val["name"].to_string(),
            passwd: "".to_string(),
            pfp_b64: val["pfp_b64"].to_string(),
            uuid: val["uuid"].as_u64().unwrap(),
        }
    }

    fn update(&mut self, val: &json::JsonValue) {
        self.nick = val["name"].to_string();
        self.pfp_b64 = val["pfp_b64"].to_string();
        self.uuid = val["uuid"].as_u64().unwrap();
    }
}

fn process_input(tx: std::sync::mpsc::Sender<LocalMessage>) {
    let stdin = stdin();

    for event in stdin.events() {
        tx.send(LocalMessage::Keyboard(event.as_ref().unwrap().clone()))
            .unwrap();
    }
}

#[tokio::main]
async fn main() {
    let (tx, rx): (
        std::sync::mpsc::Sender<LocalMessage>,
        std::sync::mpsc::Receiver<LocalMessage>,
    ) = std::sync::mpsc::channel();

    let input_tx = tx.clone();
    tokio::spawn(async move {
        process_input(input_tx);
    });

    let mut gui = GUI::new(tx, rx).await;
    gui.run_gui().await;
}
