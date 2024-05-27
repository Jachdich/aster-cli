extern crate dirs;
extern crate termion;
extern crate tokio;
mod api;

use crate::termion::input::TermRead;
use fmtstring::FmtString;
use std::io::stdin;
use std::net::SocketAddr;
use termion::event::{Event, Key};
use tokio::sync::broadcast;

mod drawing;
mod events;
mod gui;
mod prompt;
mod server;

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
    User(FmtString),
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
        if let Event::Key(Key::Ctrl('c')) = event.unwrap() {
            return;
        }
    }
}

#[tokio::main]
async fn main() {
    let (tx, rx): (
        std::sync::mpsc::Sender<LocalMessage>,
        std::sync::mpsc::Receiver<LocalMessage>,
    ) = std::sync::mpsc::channel();

    let (cancel_tx, cancel_rx) = broadcast::channel(1);
    drop(cancel_rx); // bruh why does it give me a rx, I just want a tx for now

    let input_tx = tx.clone();
    tokio::spawn(async move {
        process_input(input_tx);
    });

    let mut gui = GUI::new(tx, rx, cancel_tx).await;
    gui.run_gui().await;
}
