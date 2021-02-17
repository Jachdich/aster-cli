use openssl::ssl::{SslMethod, SslConnector, SslVerifyMode};
use std::io::BufReader;
use std::str::from_utf8;
use std::net::TcpStream;
extern crate termion;
extern crate tokio;

use termion::raw::IntoRawMode;
use termion::event::{Key, MouseEvent, Event, MouseButton};
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

fn draw_messages<W: Write>(screen: &mut W, messages: &Vec<Message>, mut scroll: isize) -> isize {
    let (width, height) = termion::terminal_size().unwrap();
    let max_messages = height as usize - 3;
    let len = messages.len();
    let start_idx = len as isize - max_messages as isize + scroll as isize;
    let start_idx = if start_idx < 0 { 0 } else { start_idx as usize };
    write!(screen, "{}#general", termion::cursor::Goto(2, 5)).unwrap();

    if scroll > 0 { scroll = 0; }
    if (scroll + start_idx as isize) < 0 { scroll = 0 - start_idx as isize; }
    
    let mut line = 2;
    for message in messages[(start_idx as isize + scroll) as usize..(len as isize + scroll) as usize].iter() {

        let max_chars: usize = width as usize - 28;
        let num_lines: usize = (message.content.len() as f64 / max_chars as f64).ceil() as usize;
        for i in 0..num_lines {
            let e = if (i + 1) * max_chars >= message.content.len() { message.content.len() } else { (i + 1) * max_chars };
            write!(screen, "{}{}{}", termion::cursor::Goto(28, line), &message.content[i * max_chars..e], "").unwrap();
            line += 1;
        }
    }

    scroll
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
    Keyboard(Event),
    Network(String),
}

fn draw_screen<W: Write>(screen: &mut W, mode: u8, messages: &Vec<Message>, buffer: &String, mut scroll: isize) -> isize {
    let (width, height) = termion::terminal_size().unwrap();

    if width < 32 || height < 8 {
        write!(screen, "Terminal size is too small lol").unwrap();
        return scroll;
    }
    
    if mode == SERVER_MODE {
        draw_border(screen);
        scroll = draw_messages(screen, messages, scroll);
        write!(screen, "{}{}", termion::cursor::Goto(28, height - 1), buffer);
    }
    scroll
}

fn process_input(tx: std::sync::mpsc::Sender<LocalMessage>) {
    let mut stdin = stdin();

    for event in stdin.events() {
        tx.send(LocalMessage::Keyboard(event.as_ref().unwrap().clone()));
    }
}

fn run_gui(rx: std::sync::mpsc::Receiver<LocalMessage>, mut stream: TcpStream) {
    let stdout = stdout().into_raw_mode().unwrap();

    let mut loaded_messages: Vec<Message> = Vec::new();
    let mut buffer: String = "".to_string();
    
	let mut screen = termion::input::MouseTerminal::from(stdout).into_raw_mode().unwrap();
    //let mut screen = stdout;
    draw_screen(&mut screen, SERVER_MODE, &loaded_messages, &buffer, 0);
    screen.flush().unwrap();
    let mut waiting_for_messages = 50;
    let mut requested_messages = false;
    let mut scroll: isize = 0;
    let mut uname = format!("{}", stream.local_addr().unwrap());
    stream.write(format!("/nick {}\n", uname).as_bytes()).unwrap();
    loop {
        write!(screen, "{}{}", termion::cursor::Goto(1, 1), termion::clear::CurrentLine).unwrap();

        if waiting_for_messages > 0 && !requested_messages {
            stream.write(format!("/history {} {}\n", 0, waiting_for_messages).as_bytes()).unwrap();
            requested_messages = true;
        }
	      
	    match rx.recv().unwrap() {
	        LocalMessage::Keyboard(key) => {
	            match key {
	                
	                Event::Key(Key::Ctrl('c')) => return,
	                Event::Key(Key::Char('\n')) => {
                        if buffer.chars().nth(0).unwrap() == '/' {
                            let split = buffer.split(" ");
                            let argv = split.collect::<Vec<&str>>();
                            match argv[0] {
                                "/nick" => {
                                    uname = argv[1].to_string();
                                }

                                "/join" => {
                                    requested_messages = false;
                                    waiting_for_messages = 50;
                                    loaded_messages.clear();
                                }
                                _ => {}
                            }
                        }

	                    stream.write(buffer.as_bytes());
	                    stream.write(b"\n");
	                    loaded_messages.push(Message{content: format!("{}: {}", uname, buffer)});
	                    buffer = "".to_string();
	                }
	                
	                Event::Key(Key::Char(ch)) => {
	                   buffer.push(ch); 
	                }
	                
	                Event::Key(Key::Backspace) => {
	                    buffer.pop();
	                }

                    Event::Mouse(MouseEvent::Press(MouseButton::WheelUp, _, _)) => {
                        scroll -= 1;
                    }

                    Event::Mouse(MouseEvent::Press(MouseButton::WheelDown, _, _)) => {
                        scroll += 1;
                    }
	                
	                _ => (),
	            }
	        },

	        LocalMessage::Network(msg) => {
	            if waiting_for_messages > 0 {
	                loaded_messages.insert(loaded_messages.len(), Message{content: msg});
	                waiting_for_messages -= 1;
	                if waiting_for_messages == 0 { requested_messages = false; }
	            } else {
	                loaded_messages.push(Message{content: msg});
	            }
	        },
	    }
	    scroll = draw_screen(&mut screen, SERVER_MODE, &loaded_messages, &buffer, scroll);
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
                match data {
                    Some(text) => {
                        //tell GUI that message has been recv'd
                        tx.send(LocalMessage::Network(text));
                    }

                    None => {}
                }
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
