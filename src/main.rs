use openssl::ssl::{SslMethod, SslConnector, SslVerifyMode};
use std::io::BufReader;
use std::str::from_utf8;
use std::net::TcpStream;
extern crate termion;
extern crate tokio;

use termion::raw::IntoRawMode;
use termion::event::Key;
use std::io::{Write, stdout, stdin};
use crate::termion::input::TermRead;

const SERVER_MODE: u8 = 0;

fn centred(text: &str, width: usize) -> String {
    format!("{: ^1$}", text, width)
}

struct User {
    nick: String,
    uuid: u128,
}

struct Message {
//    author: User,
    content: String,
//    time: chrono::DateTime,
}

fn draw_messages<W: Write>(screen: &mut W, messages: &Vec<Message>) {
    write!(screen, "{}#general", termion::cursor::Goto(2, 5)).unwrap();
    
    let mut line = 2;
    for message in messages.iter() {
        write!(screen, "{}{}{}", termion::cursor::Goto(28, line), message.content, "").unwrap();
        line += 1;
    }
}

fn draw_border<W: Write>(screen: &mut W) {
    let (width, height) = termion::terminal_size().unwrap();
    let left_margin: usize = 24;
    let server_string = centred("cospox.com", left_margin);
    let space_padding = " ".repeat(width as usize - left_margin - 3);
    write!(screen, "{}{}", termion::cursor::Goto(1, 1), termion::clear::All).unwrap();
    write!(screen, "┏{}┳{}┓\r\n", "━".repeat(left_margin), "━".repeat(width as usize - left_margin - 3)).unwrap();
    write!(screen, "┃{}┃{}┃\r\n", centred("Connected to", left_margin), space_padding).unwrap();
    write!(screen, "┃{}┃{}┃\r\n", server_string, space_padding).unwrap();
    write!(screen, "┣{}┫{}┃\r\n", "━".repeat(left_margin), space_padding).unwrap();
    write!(screen, "{}", format!("┃{}┃{}┃\r\n", " ".repeat(left_margin), space_padding).repeat(height as usize - 5)).unwrap();
    write!(screen, "┗{}┻{}┛", "━".repeat(left_margin), "━".repeat(width as usize - left_margin - 3)).unwrap();

}

enum LocalMessage {
    Keyboard(Key),
    Network(String),
}

fn draw_screen<W: Write>(screen: &mut W, mode: u8, messages: &Vec<Message>, buffer: &String) {
    let (width, height) = termion::terminal_size().unwrap();

    if width < 32 || height < 8 {
        write!(screen, "Terminal size is too small lol").unwrap();
        return;
    }
    
    if mode == SERVER_MODE {
        draw_border(screen);
        draw_messages(screen, messages);
        write!(screen, "{}{}", termion::cursor::Goto(28, height - 1), buffer);
    }
}

fn process_input(tx: std::sync::mpsc::Sender<LocalMessage>) {
    let mut stdin = stdin();

    for c in stdin.keys() {
        tx.send(LocalMessage::Keyboard(c.as_ref().unwrap().clone()));
    }
}

fn run_gui(rx: std::sync::mpsc::Receiver<LocalMessage>, mut stream: TcpStream) {
    let stdout = stdout().into_raw_mode().unwrap();

    let mut loaded_messages: Vec<Message> = Vec::new();
    let mut buffer: String = "".to_string();
    
	let mut screen = termion::screen::AlternateScreen::from(stdout).into_raw_mode().unwrap();
    draw_screen(&mut screen, SERVER_MODE, &loaded_messages, &buffer);
    screen.flush().unwrap();
    loop {
        write!(screen, "{}{}", termion::cursor::Goto(1, 1), termion::clear::CurrentLine).unwrap();
	    
	    match rx.recv().unwrap() {
	        LocalMessage::Keyboard(key) => {
	            match key {
	                Key::Ctrl('c') => return,
	                Key::Char('\n') => {
	                    stream.write(buffer.as_bytes());
	                    stream.write(b"\n");
	                    loaded_messages.push(Message{content: format!("Jams: {}", buffer)});
	                    buffer = "".to_string();
	                }
	                Key::Char(ch) => {
	                   buffer.push(ch); 
	                },
	                Key::Backspace => {
	                    buffer.pop();
	                }
	                _ => (),
	            }
	        },

	        LocalMessage::Network(msg) => {
	            loaded_messages.push(Message{content: msg});
	        },
	    }
	    draw_screen(&mut screen, SERVER_MODE, &loaded_messages, &buffer);
	    screen.flush().unwrap();
	}
}

fn run_network(tx: std::sync::mpsc::Sender<LocalMessage>, stream: TcpStream) {

    let mut connector = SslConnector::builder(SslMethod::tls())
        .unwrap();
    connector.set_verify(SslVerifyMode::NONE);
    let connector = connector.build();

    let mut reader = BufReader::new(stream);
    
    //let mut sslstream = connector.connect("127.0.0.1", stream).unwrap();

    //let msg = b"hello, world";
    //stream.write(msg).unwrap(); //todo handle error
    loop {
        let result = reader.read_line();
        match result {
            Ok(data) => {
                let text = data;
                tx.send(LocalMessage::Network(text.unwrap()));
            },
            Err(e) => {
                //println!("Failed to recv data: {}", e);
            }
        }
    }
}

fn main() {
    let stream = TcpStream::connect("127.0.0.1:2345").unwrap();
    let other_stream = stream.try_clone().unwrap();
    
    let (tx, rx): (std::sync::mpsc::Sender<LocalMessage>, std::sync::mpsc::Receiver<LocalMessage>) = std::sync::mpsc::channel();
    
    //run_network_thread(tx.clone(), stream);
    let net_tx = tx.clone();
    let input_tx = tx.clone();
    std::thread::spawn(move || {
        run_network(net_tx, stream);
    });
    std::thread::spawn(move || {
        process_input(input_tx);
    });
    run_gui(rx, other_stream);
    
}
