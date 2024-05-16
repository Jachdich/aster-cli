extern crate dirs;
extern crate termion;
extern crate tokio;
mod api;
use termion::event::Event;

use crate::termion::input::TermRead;
use std::io::stdin;
use std::net::SocketAddr;

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

pub enum DisplayMessage {
    User(api::Message),
    System(FmtString),
}

pub enum LocalMessage {
    Keyboard(Event),
    Network(String, SocketAddr),
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
