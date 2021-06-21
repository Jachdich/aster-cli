extern crate termion;
extern crate tokio;
extern crate dirs;
use termion::event::Event;

use std::io::stdin;
use crate::termion::input::TermRead;

mod drawing;
mod events;
mod gui;
mod server;
mod servernetwork;
mod parser;

use gui::GUI;
use drawing::FmtString;

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

pub struct Message {
//    author: u64,
    content: FmtString,
//    time: chrono::DateTime,
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
        tx.send(LocalMessage::Keyboard(event.as_ref().unwrap().clone())).unwrap();
    }
}

#[tokio::main]
async fn main() {
    let (tx, rx): (std::sync::mpsc::Sender<LocalMessage>, std::sync::mpsc::Receiver<LocalMessage>) = std::sync::mpsc::channel();

    let input_tx = tx.clone();
    std::thread::spawn(move || {
        process_input(input_tx);
    });
    let mut gui = GUI::new(tx, rx).await;
    gui.run_gui().await;
}
