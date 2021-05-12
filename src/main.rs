extern crate termion;
extern crate tokio;
use native_tls::TlsConnector;
use tokio_native_tls::TlsStream;
use std::net::ToSocketAddrs;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::io::{ReadHalf, WriteHalf};
use crate::tokio::io::AsyncBufReadExt;

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
    passwd: String,
    pfp_b64: String,
}

struct Message {
//    author: u64,
    content: String,
//    time: chrono::DateTime,
}

fn draw_servers<W: Write>(screen: &mut W, servers: &Vec<Server>, curr_server: usize) {
    let (_width, height) = termion::terminal_size().unwrap();
    let list_height: u16 = height as u16 - 5;
    let servers_height: u16;
    if list_height % 2 == 0 {
        servers_height = list_height / 2 - 1;
    } else {
        servers_height = list_height / 2;
    }

    let mut vert_pos = 5;
    for channel in &servers[curr_server].channels {
        write!(screen, "{}#{}", termion::cursor::Goto(2, vert_pos), channel).unwrap();
        vert_pos += 1;
        //TODO scrolling
    }
    vert_pos = servers_height + 6;
    for server in servers {
        write!(screen, "{}{}", termion::cursor::Goto(2, vert_pos), server.name).unwrap();
        vert_pos += 1;
    }
}

fn draw_messages<W: Write>(screen: &mut W, messages: &Vec<Message>, mut scroll: isize) -> isize {
    let (width, height) = termion::terminal_size().unwrap();
    let max_messages = height as usize - 3;
    let len = messages.len();
    let start_idx = len as isize - max_messages as isize + scroll as isize;
    let start_idx = if start_idx < 0 { 0 } else { start_idx as usize };

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
    let list_height: usize = height as usize - 5;
    let channels_height: usize = list_height / 2;
    let servers_height: usize;
    if list_height % 2 == 0 {
        servers_height = list_height / 2 - 1;
    } else {
        servers_height = list_height / 2;
    }
    let server_string = centred("cospox.com", left_margin);
    let space_padding = " ".repeat(width as usize - left_margin - 3);
    write!(screen, "{}{}", termion::cursor::Goto(1, 1), termion::clear::All).unwrap();
    write!(screen, "┏{}┳{}┓\r\n", "━".repeat(left_margin), "━".repeat(width as usize - left_margin - 3)).unwrap();
    write!(screen, "┃{}┃{}┃\r\n", centred("Connected to", left_margin), space_padding).unwrap();
    write!(screen, "┃{}┃{}┃\r\n", server_string, space_padding).unwrap();
    write!(screen, "┣{}┫{}┃\r\n", "━".repeat(left_margin), space_padding).unwrap();
    write!(screen, "{}", format!("┃{}┃{}┃\r\n", " ".repeat(left_margin), space_padding).repeat(channels_height)).unwrap();
    write!(screen, "┣{}┫{}┃\r\n", "━".repeat(left_margin), space_padding).unwrap();
    write!(screen, "{}", format!("┃{}┃{}┃\r\n", " ".repeat(left_margin), space_padding).repeat(servers_height)).unwrap();
    write!(screen, "┗{}┻{}┛", "━".repeat(left_margin), "━".repeat(width as usize - left_margin - 3)).unwrap();

}

enum LocalMessage {
    Keyboard(Event),
    Network(String, usize),
}

fn draw_screen<W: Write>(screen: &mut W, mode: u8, servers: &Vec<Server>, curr_server: usize, buffer: &String, mut scroll: isize) -> isize {
    let (width, height) = termion::terminal_size().unwrap();

    if width < 32 || height < 8 {
        write!(screen, "Terminal size is too small lol").unwrap();
        return scroll;
    }
    
    if mode == SERVER_MODE {
        draw_border(screen);
        if servers.len() > 0 {
            scroll = draw_messages(screen, &servers[curr_server].loaded_messages, scroll);
            draw_servers(screen, servers, curr_server);
        }
        write!(screen, "{}{}", termion::cursor::Goto(28, height - 1), buffer);
    }
    scroll
}

fn process_input(tx: std::sync::mpsc::Sender<LocalMessage>) {
    let stdin = stdin();

    for event in stdin.events() {
        tx.send(LocalMessage::Keyboard(event.as_ref().unwrap().clone()));
    }
}

struct GUI {
    message_scroll: isize,
    buffer: String,
    tx: std::sync::mpsc::Sender<LocalMessage>,
    rx: std::sync::mpsc::Receiver<LocalMessage>,
    config: json::JsonValue,
    servers: Vec<Server>,
    curr_server: usize,
}

impl GUI {
    
    fn new(tx: std::sync::mpsc::Sender<LocalMessage>, rx: std::sync::mpsc::Receiver<LocalMessage>) -> Self {
        let default_config: json::JsonValue = json::object!{
            servers: [],
            uname: "",
            passwd: "",
            pfp: "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",
        };
        let contents = std::fs::read_to_string("preferences.json");
        let config: json::JsonValue;
        match contents {
            Ok(contents) => {
                match json::parse(&contents) {
                    Ok(value) => {
                        config = value;
                    }
                    Err(_) => {
                        config = default_config;
                    }
                }
            }
            Err(_) => {
                config = default_config;
            }
        }
        GUI {
            message_scroll: 0,
            buffer: "".to_string(),
            tx,
            rx,
            config,
            servers: Vec::new(),
            curr_server: 0,
        }
    }
    
    async fn handle_keyboard(&mut self, key: Event) -> bool {
        match key {
             Event::Key(Key::Ctrl('c')) => return false,
             Event::Key(Key::Ctrl('n')) => {
                //self.servers.push(Server::new());
                    
             }
             Event::Key(Key::Char('\n')) => {
                if self.buffer.len() == 0 {
                    return true;
                }
                if self.buffer.chars().nth(0).unwrap() == '/' {
                    let split = self.buffer.split(" ");
                    let argv = split.collect::<Vec<&str>>();
                    match argv[0] {
                        "/nick" => {
                            self.config["uname"] = argv[1].into();
                        }

                        "/join" => {
                            self.servers[self.curr_server].loaded_messages.clear();
                        }
                        _ => {}
                    }
                }

                self.servers[self.curr_server].net.write_half.write(self.buffer.as_bytes()).await;
                self.servers[self.curr_server].net.write_half.write(b"\n").await;
                self.servers[self.curr_server].loaded_messages.push(Message{content: format!("{}: {}", self.config["uname"].to_string(), self.buffer)});
                self.buffer = "".to_string();
            }

            Event::Key(Key::Char(ch)) => {
                self.buffer.push(ch); 
            }

            Event::Key(Key::Backspace) => {
                self.buffer.pop();
            }

            Event::Mouse(MouseEvent::Press(MouseButton::WheelUp, _, _)) => {
                self.message_scroll -= 1;
            }

            Event::Mouse(MouseEvent::Press(MouseButton::WheelDown, _, _)) => {
                self.message_scroll += 1;
            }
         
            _ => (),
         }
         true
    }

    fn handle_network_packet(&mut self, obj: json::JsonValue, serv: usize) {
        if !obj["content"].is_null() {
            self.servers[serv].loaded_messages.push(
            Message{
                content: "".to_string(),//format!("{}: {}", obj["content"].to_string(), self.servers[serv].peers[,
            });
        }
    }

    fn save_config(&self) {
        //TODO unwraps BADE
        let mut file = std::fs::File::create("preferences.json").unwrap();
        file.write_all(self.config.dump().as_bytes()).unwrap();
    }

    async fn run_gui(&mut self) {
        let stdout = stdout().into_raw_mode().unwrap();
        //let mut screen = stdout();
    	let mut screen = termion::input::MouseTerminal::from(stdout).into_raw_mode().unwrap();
        draw_screen(&mut screen, SERVER_MODE, &Vec::new(), 0, &self.buffer, 0);
        screen.flush().unwrap();

        loop {
            write!(screen, "{}{}", termion::cursor::Goto(1, 1), termion::clear::All).unwrap();
    	      
    	    match self.rx.recv().unwrap() {
    	        LocalMessage::Keyboard(key) => {
    	            if !self.handle_keyboard(key).await {
    	                self.save_config();
                        return;
    	            }
    	        }

    	        LocalMessage::Network(msg, idx) => {
    	            let obj = json::parse(&msg);
       	            match obj {
       	                Ok(obj) => {
       	                    self.handle_network_packet(obj, idx);
       	                }
       	                Err(_) => {
       	                    //ignore for now
       	                }
       	            }
    	        }
    	    }
    	    self.message_scroll = draw_screen(&mut screen, SERVER_MODE, &self.servers, self.curr_server, &self.buffer, self.message_scroll);
    	    screen.flush().unwrap();
    	}
    }
}

struct ServerNetwork {
    tx: std::sync::mpsc::Sender<LocalMessage>,
    write_half: WriteHalf<TlsStream<TcpStream>>,
}

struct Server {
    loaded_messages: Vec<Message>,
    ip: String,
    name: String,
    channels: Vec<String>,
    peers: Vec<User>,
    curr_uuid: u64,
    net: ServerNetwork,
    idx: usize,
}

impl Server {
    fn from_known(ip: String, port: u16, name: String, uuid: u64, idx: usize, tx: std::sync::mpsc::Sender<LocalMessage>) -> Self {
        let mut net: ServerNetwork = futures::executor::block_on(ServerNetwork::new(&ip, port, tx, idx));
        net.write_half.write(format!("/login {}", uuid).to_bytes());
        net.write_half.write(b"/get_icon");
        net.write_half.write(b"/get_name");
        net.write_half.write(b"/get_all_metadata");
        net.write_half.write(b"/get_channels");
        net.write_half.write(b"/online");
        
        Server{
            loaded_messages: Vec::new(),
            ip,
            name,
            channels: Vec::new(),
            curr_uuid: uuid,
            net,
            idx,
        }
    }

    fn from_unknown(ip: String, port: u16, idx: usize, tx: std::sync::mpsc::Sender<LocalMessage>) -> Self {
        let mut net: ServerNetwork = futures::executor::block_on(ServerNetwork::new(&ip, port, tx, idx));
        net.write_half.write(b"/register");
        net.write_half.write(b"/get_icon");
        net.write_half.write(b"/get_name");
        net.write_half.write(b"/get_all_metadata");
        net.write_half.write(b"/get_channels");
        net.write_half.write(b"/online");
        
        Server{
            loaded_messages: Vec::new(),
            ip,
            name: "".to_string(),
            channels: Vec::new(),
            curr_uuid: 0,
            net,
            idx,
        }
    }

    fn from_uuid(ip: String, port: u16, uuid: u64, idx: usize, tx: std::sync::mpsc::Sender<LocalMessage>) -> Self {
        let mut net: ServerNetwork = futures::executor::block_on(ServerNetwork::new(&ip, port, tx, idx));
        net.write_half.write(format!("/login {}", uuid).to_bytes());
        net.write_half.write(b"/get_icon");
        net.write_half.write(b"/get_name");
        net.write_half.write(b"/get_all_metadata");
        net.write_half.write(b"/get_channels");
        net.write_half.write(b"/online");
        
        Server{
            loaded_messages: Vec::new(),
            ip,
            name: "".to_string(),
            channels: Vec::new(),
            curr_uuid: uuid,
            net,
            idx,
        }
    }

    fn update_metadata(&self, meta: User) {
        
    }
}

impl ServerNetwork {
    async fn new(ip: &str, port: u16, tx: std::sync::mpsc::Sender<LocalMessage>, idx: usize) -> Self {
        let addr = format!("{}:{}", ip, port)
            .to_socket_addrs().unwrap()
            .next()
            .ok_or("failed to resolve hostname").unwrap();

        let socket = TcpStream::connect(&addr).await.unwrap();
        let cx = TlsConnector::builder().danger_accept_invalid_certs(true).build().unwrap();
        let cx = tokio_native_tls::TlsConnector::from(cx);

        let socket = cx.connect(ip, socket).await.unwrap();
        let (read_half, write_half) = tokio::io::split(socket);

        let net_tx = tx.clone();
        std::thread::spawn(move || {
            futures::executor::block_on(ServerNetwork::run_network(net_tx, read_half, idx));
        });

        ServerNetwork {
            tx,
            write_half,
        }
    }

    async fn run_network(tx: std::sync::mpsc::Sender<LocalMessage>, stream: ReadHalf<TlsStream<TcpStream>>, idx: usize) {
        let mut reader = tokio::io::BufReader::new(stream);
    
        loop {
            let mut result: String = "".to_string();
            match reader.read_line(&mut result).await {
            	Ok(_len) => {
      	            tx.send(LocalMessage::Network(result, idx));
      	        }
    
            	Err(..) => {
            		return;
            	}
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let (tx, rx): (std::sync::mpsc::Sender<LocalMessage>, std::sync::mpsc::Receiver<LocalMessage>) = std::sync::mpsc::channel();

    let input_tx = tx.clone();
    std::thread::spawn(move || {
        process_input(input_tx);
    });
    let mut gui = GUI::new(tx, rx);
    gui.run_gui().await;
}
