use openssl::ssl::{SslMethod, SslConnector, SslVerifyMode};
use std::net::TcpStream;
use std::io::{Read, Write};
use std::str::from_utf8;
use std::io;

fn main() {
    match TcpStream::connect("127.0.0.1:42069") {
        Ok(stream) => {
            println!("Connected to server port 42069");

            let mut connector = SslConnector::builder(SslMethod::tls())
                .unwrap();
            connector.set_verify(SslVerifyMode::NONE);
            let connector = connector.build();
            
            let mut sslstream = connector.connect("127.0.0.1", stream).unwrap();

            loop{
                let mut input = String::new();
                let mut stdin = io::stdin(); // We get `Stdin` here.
                stdin.read_line(&mut input);

                let msg = input.as_bytes();

                sslstream.write(msg).unwrap(); //todo handle error
                let mut data = vec![0 as u8; msg.len()];
                match sslstream.read_exact(&mut data) {
                    Ok(_) => {
                        let text = from_utf8(&data).unwrap();
                        println!("{}", text);
                    },
                    Err(e) => {
                        println!("Failed to recv data: {}", e);
                    }
                }
            }

            
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
}
