use openssl::ssl::{SslMethod, SslConnector, SslVerifyMode};
use std::net::TcpStream;
use std::str::from_utf8;
extern crate termion;

use termion::raw::IntoRawMode;
use termion::event::Key;
use std::io::{Write, Read, stdout, stdin};
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
    author: User,
    content: String,
//    time: chrono::DateTime,
}

fn draw_messages<W: Write>(screen: &mut W, messages: &Vec<Message>) {
    write!(screen, "{}#general", termion::cursor::Goto(2, 5)).unwrap();
    
    let mut line = 2;
    for message in messages.iter() {
        write!(screen, "{}{}: {}{}", termion::cursor::Goto(28, line), message.author.nick, message.content, "").unwrap();
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

fn draw_screen<W: Write>(screen: &mut W, mode: u8, messages: &Vec<Message>) {
    let (width, height) = termion::terminal_size().unwrap();

    if width < 32 || height < 8 {
        write!(screen, "Terminal size is too small lol").unwrap();
        return;
    }
    
    if mode == SERVER_MODE {
        draw_border(screen);
        draw_messages(screen, messages);
    }
}

fn process_input<R: Read, W: Write>(screen: &mut W, stdin: &mut R) -> bool {
    if let Some(c) = stdin.keys().next() {
        match c.unwrap() {
            Key::Ctrl('c') => return true,
            Key::Up        => write!(screen, "<up>").unwrap(),
            Key::Down      => write!(screen, "<down>").unwrap(),
            _              => (),
        }
    }
    return false;
}

fn main() {
    let mut loaded_messages: Vec<Message> = Vec::new();

    let child = std::thread::spawn(move || {
        let stdout = stdout().into_raw_mode().unwrap();
        let mut stdin = stdin();
    
    	let mut screen = termion::screen::AlternateScreen::from(stdout).into_raw_mode().unwrap();
        draw_screen(&mut screen, SERVER_MODE, &loaded_messages);
        screen.flush().unwrap();
        loop {
            write!(screen, "{}{}", termion::cursor::Goto(1, 1), termion::clear::CurrentLine).unwrap();
    	    if process_input(&mut screen, &mut stdin) {
    	        break;
    	    }
    	    draw_screen(&mut screen, SERVER_MODE, &loaded_messages);
    	    screen.flush().unwrap();
    	}
    });

    match TcpStream::connect("127.0.0.1:2345") {
        Ok(mut stream) => {
            //println!("Connected to server port 42069");

            let mut connector = SslConnector::builder(SslMethod::tls())
                .unwrap();
            connector.set_verify(SslVerifyMode::NONE);
            let connector = connector.build();
            
            //let mut sslstream = connector.connect("127.0.0.1", stream).unwrap();

            let msg = b"hello, world";
            stream.write(msg).unwrap(); //todo handle error
            let mut data = [0 as u8; 12];
            match stream.read_exact(&mut data) {
                Ok(_) => {
                    let text = from_utf8(&data).unwrap();
                    loaded_messages.push(Message{author: User{nick: "Jams lol".to_string(), uuid: 1}, content: text.to_string()});
                    //println!("{}", text);
                },
                Err(e) => {
                    //println!("Failed to recv data: {}", e);
                }
            }
        },
        Err(e) => {
            //println!("Failed to connect: {}", e);
        }
    }

    let res = child.join();
}
