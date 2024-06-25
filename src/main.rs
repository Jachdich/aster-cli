extern crate dirs;
extern crate termion;
extern crate tokio;
mod api;

use crate::api::Response;
use crate::drawing::draw_border;
use crate::gui::Settings;
use crate::prompt::*;
use crate::server::Server;
use api::{Status, SyncData, SyncServer};
use drawing::Theme;
use fmtstring::FmtString;
use server::WriteAsterRequest;
use std::io::{stdin, stdout, BufRead, BufReader, Write};
use std::net::SocketAddr;
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
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

async fn init_server_from_syncserver(
    serv: &SyncServer,
    tx: &std::sync::mpsc::Sender<LocalMessage>,
    cancel: &broadcast::Sender<()>,
) -> Option<Server> {
    let mut conn = Server::new(
        serv.ip.clone(),
        serv.port as u16,
        tx.clone(),
        cancel.subscribe(),
    )
    .await;
    if conn.is_online() {
        let id = if let Some(uuid) = serv.uuid {
            crate::server::Identification::Uuid(uuid)
        } else {
            crate::server::Identification::Username(serv.uname.clone())
        };
        match conn.network.as_mut().unwrap().initialise(id).await {
            Ok(()) => (),
            Err(e) => conn.to_offline(e.to_string()),
        }
    }

    if !conn.is_online() {
        conn.name = serv.name.clone(); // TODO get rid of this clone()?
        conn.uuid = serv.uuid;
        conn.uname = Some(serv.uname.clone());
    }
    Some(conn)
}

fn load_config_json() -> serde_json::Value {
    let default_config: serde_json::Value = serde_json::json!({
        "servers": [],
        "uname": Option::<&str>::None,
        "passwd": Option::<&str>::None,
        "pfp": "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",
    });
    let mut preferences_path = dirs::preference_dir().unwrap();
    preferences_path.push("aster-cli");
    preferences_path.push("preferences.json");
    let contents = std::fs::read_to_string(preferences_path);
    match contents {
        Ok(contents) => match serde_json::from_str(&contents) {
            Ok(value) => value,
            Err(_) => default_config,
        },
        Err(_) => default_config,
    }
}

async fn load_servers(
    server_info: &[api::SyncServer],
    tx: std::sync::mpsc::Sender<LocalMessage>,
    cancel: broadcast::Sender<()>,
) -> Vec<Server> {
    let mut servers: Vec<Server> = Vec::new();

    for serv in server_info {
        let conn = init_server_from_syncserver(serv, &tx, &cancel).await;
        if let Some(conn) = conn {
            servers.push(conn);
        } else {
            // ???, server decode failed
        }
    }
    servers
}

enum AuthMode {
    Login,
    Register,
}

fn load_sync_data(
    ip: &str,
    port: u16,
    uname: &str,
    passwd: &str,
    auth: AuthMode,
) -> Result<(Option<SyncData>, Vec<SyncServer>), std::io::Error> {
    let mut conn = std::net::TcpStream::connect((ip, port))?;

    match auth {
        AuthMode::Login => conn.write_request(&api::Request::LoginRequest {
            passwd: passwd.to_owned(),
            uname: Some(uname.to_owned()),
            uuid: None,
        })?,
        AuthMode::Register => conn.write_request(&api::Request::RegisterRequest {
            passwd: passwd.to_owned(),
            uname: uname.to_owned(),
        })?,
    }
    conn.write_request(&api::Request::SyncGetRequest)?;
    conn.write_request(&api::Request::SyncGetServersRequest)?;

    let mut reader = BufReader::new(conn);
    let mut syncdata: Option<SyncData> = None;
    let mut syncservers: Vec<SyncServer> = Vec::new();
    let mut got_data = false;
    let mut got_servers = false;
    loop {
        let mut buf = String::new();

        BufRead::read_line(&mut reader, &mut buf)?;
        let response: Response = serde_json::from_str(&buf).unwrap();
        use std::io::{Error, ErrorKind};
        use Response::*;
        use Status::*;
        match response {
            LoginResponse { status: Ok, .. } | RegisterResponse { status: Ok, .. } => (),
            LoginResponse {
                status: Forbidden, ..
            } => return Err(Error::new(ErrorKind::PermissionDenied, "")),
            LoginResponse {
                status: NotFound, ..
            } => return Err(Error::new(ErrorKind::NotFound, "")),
            SyncGetResponse {
                status: Ok,
                data: Some(data),
            } => {
                syncdata = Some(data);
                got_data = true
            }
            SyncGetResponse {
                status: NotFound, ..
            } => got_data = true,
            SyncGetServersResponse {
                status: Ok,
                servers: Some(servers),
            } => {
                syncservers = servers;
                got_servers = true;
            }
            APIVersion { .. } => (),
            res => {
                // error on non-OK status. it's fine if the server sends us data we don't care about, as long as it's Ok
                if res.status() != Ok {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!(
                            "Unexpected result from the server: {}: {}",
                            res.name(),
                            res.status()
                        ),
                    ));
                }
            }
        }
        if got_data && got_servers {
            return Result::Ok((syncdata, syncservers));
        }
    }
}

fn get_sync_details<W: Write>(
    config: &serde_json::Value,
    screen: &mut W,
) -> Result<(String, u16, String, String, AuthMode), ()> {
    if config["uname"].is_null()
        || config["passwd"].is_null()
        || config["sync_ip"].is_null()
        || config["sync_port"].is_null()
    {
        let theme = Theme::new("themes/default.json").unwrap(); // TODO get this from a legitimate source, rn its validity is questionable

        let mut prompt = Prompt::new(
            "Enter login details",
            vec![
                PromptField::String {
                    name: "Username",
                    default: config["uname"].as_str().map(|s| s.to_owned()),
                },
                PromptField::String {
                    name: "Password",
                    default: config["passwd"].as_str().map(|s| s.to_owned()),
                },
                PromptField::String {
                    name: "Sync server IP",
                    default: config["sync_ip"].as_str().map(|s| s.to_owned()),
                },
                PromptField::U16 {
                    name: "Sync server port",
                    default: Some(
                        config["sync_port"]
                            .as_u64()
                            .map(|u| u as u16)
                            .unwrap_or(2345),
                    ),
                },
            ],
            vec!["Login", "Register", "Quit"],
        );
        let (w, h) = termion::terminal_size().unwrap();
        let ph = prompt.height();
        let pw = 18 + 15; // approx
        let x = (w - pw) / 2;
        let y = (h - ph) / 2;
        write!(screen, "{}", termion::clear::All).unwrap();
        prompt.draw(screen, x, y, &theme);
        screen.flush().unwrap();
        let mut events = stdin().events();
        loop {
            let event = events.next();
            let prompt_event = prompt.handle_event(event.unwrap().unwrap());
            match prompt_event {
                Some(PromptEvent::ButtonPressed("Login"))
                | Some(PromptEvent::ButtonPressed("Register")) => {
                    let sync_ip = prompt.get_str("Sync server IP").unwrap().to_owned();
                    let sync_port = prompt.get_u16("Sync server port").unwrap();
                    let uname = prompt.get_str("Username").unwrap().to_owned();
                    let passwd = prompt.get_str("Password").unwrap().to_owned();

                    // TODO this is a bit of a hack
                    let PromptEvent::ButtonPressed(mode) = prompt_event.unwrap() else {
                        unreachable!()
                    };
                    return Ok((
                        sync_ip,
                        sync_port,
                        uname,
                        passwd,
                        if mode == "login" {
                            AuthMode::Login
                        } else {
                            AuthMode::Register
                        },
                    ));
                }
                Some(PromptEvent::ButtonPressed("Quit")) => return Err(()),
                Some(PromptEvent::ButtonPressed(_)) => unreachable!(),
                None => (),
            }
            prompt.draw(screen, x, y, &theme);
            screen.flush().unwrap();
        }
    } else {
        // yea we already have all the data. no need to ask for it!!
        let uname = config["uname"].as_str().unwrap().to_owned(); // yea i think this unwrap is O.K. rn
        let passwd = config["passwd"].as_str().unwrap().to_owned(); // yea i think this unwrap is O.K. rn
        let sync_ip = config["sync_ip"].as_str().unwrap().to_owned(); // yea i think this unwrap is O.K. rn
        let sync_port = config["sync_port"].as_u64().unwrap() as u16; // yea i think this unwrap is O.K. rn
        return Ok((sync_ip, sync_port, uname, passwd, AuthMode::Login));
    }
}

fn load_settings(config: &serde_json::Value, sync_data: Option<SyncData>) -> Settings {
    let pfp = config["pfp"].as_str().unwrap().to_owned(); // unwrap ok: json will always contain default pfp, even if none in file
    let sync_ip = config["sync_ip"].as_str().unwrap().to_owned(); // unwrap ok: json will always contain default sync_ip, even if none in file
    let sync_port = config["sync_port"].as_u64().unwrap() as u16; // unwrap ok: json will always contain default sync_port, even if none in file
    if let Some(sync_data) = sync_data {
        Settings {
            uname: sync_data.uname,
            passwd: config["passwd"].as_str().unwrap().to_owned(), // yea i think this unwrap is O.K. rn
            pfp: sync_data.pfp,
            sync_ip,
            sync_port,
        }
    } else {
        let uname = config["uname"].as_str().unwrap().to_owned(); // yea i think this unwrap is O.K. rn
        let passwd = config["passwd"].as_str().unwrap().to_owned(); // yea i think this unwrap is O.K. rn
        Settings {
            uname,
            passwd,
            pfp,
            sync_ip,
            sync_port,
        }
    }
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

    let mut screen = termion::input::MouseTerminal::from(stdout().into_raw_mode().unwrap());
    let mut conf = load_config_json();

    let (sync_data, sync_servers) = loop {
        let Ok((sync_ip, sync_port, uname, passwd, auth)) = get_sync_details(&conf, &mut screen)
        else {
            return; // quit because the only error state is if the user decides to exit
        };

        // update the json to reflect our new-found data
        // TODO this is not the most elegant way of doing it
        conf["uname"] = uname.clone().into();
        conf["passwd"] = passwd.clone().into();
        conf["sync_ip"] = sync_ip.clone().into();
        conf["sync_port"] = sync_port.into();

        match load_sync_data(&sync_ip, sync_port, &uname, &passwd, auth) {
            Ok((sync_data, sync_servers)) => break (sync_data, sync_servers),
            Err(e) => println!(
                "{}A network error occurred while logging in. Is the server offline? Details: {:?}",
                termion::cursor::Goto(1, 1),
                e
            ),
        }
    };

    println!("{:?}", conf["servers"]);

    let a: Vec<SyncServer> = serde_json::from_value(conf["servers"].clone()).unwrap(); // TODO temp

    let servers = load_servers(&a, tx.clone(), cancel_tx.clone()).await;
    let settings = load_settings(&conf, sync_data);

    let mut gui = GUI::new(tx.clone(), cancel_tx.clone(), settings, servers).await;
    screen.flush().unwrap();

    let mut last_width = 0;
    let mut last_height = 0;

    let input_tx = tx.clone();
    tokio::spawn(async move {
        process_input(input_tx);
    });

    loop {
        let (width, height) = termion::terminal_size().unwrap();

        if width < 32 || height < 8 {
            write!(screen, "Terminal size is too small lol").unwrap();
            return;
        }

        if last_width != width || last_height != height {
            let border = draw_border(&gui.theme);
            write!(screen, "{}", border).unwrap();
        }

        gui.width = width;
        gui.height = height; // TODO get rid of this and do it properly ffs

        match rx.recv().unwrap() {
            LocalMessage::Keyboard(key) => {
                if !gui.handle_keyboard(key).await {
                    gui.save_config();
                    cancel_tx.send(());
                    return;
                }
            }

            LocalMessage::Network(msg, addr) => {
                let obj: Result<serde_json::Value, serde_json::Error> = serde_json::from_str(&msg);
                match obj {
                    Ok(obj) => {
                        let response: Response = serde_json::from_value(obj).unwrap();
                        // for formatting the messages
                        let max_message_width = width as usize - gui.theme.left_margin - 4; // TODO why 4???
                        match gui
                            .get_server_by_addr(addr)
                            .expect("Network packet recv'd for offline server")
                            .handle_network_packet(response, max_message_width)
                            .await
                        {
                            Ok(()) => (),
                            Err(e) => gui.send_system(&e),
                        }
                    }
                    Err(_) => {
                        //ignore for now
                    }
                }
            }
        }
        gui.draw_all(&mut screen);
        screen.flush().unwrap();
        last_width = width;
        last_height = height;
    }
}
