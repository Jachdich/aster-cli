use openssl::ssl::{SslMethod, SslConnector, SslVerifyMode};
use std::net::TcpStream;
use std::io::{Read, Write};
use std::str::from_utf8;

fn main() {
    match TcpStream::connect("127.0.0.1:42069") {
        Ok(stream) => {
            println!("Connected to server port 42069");

            let mut connector = SslConnector::builder(SslMethod::tls())
                .unwrap();
            connector.set_verify(SslVerifyMode::NONE);
            let connector = connector.build();
            
            let mut sslstream = connector.connect("127.0.0.1", stream).unwrap();

            let msg = b"hello, world";
            sslstream.write(msg).unwrap(); //todo handle error
            let mut data = [0 as u8; 12];
            match sslstream.read_exact(&mut data) {
                Ok(_) => {
                    let text = from_utf8(&data).unwrap();
                    println!("{}", text);
                },
                Err(e) => {
                    println!("Failed to recv data: {}", e);
                }
            }
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
}
