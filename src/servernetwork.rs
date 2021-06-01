extern crate tokio;
use native_tls::TlsConnector;
use tokio_native_tls::TlsStream;
use std::net::ToSocketAddrs;
use tokio::net::TcpStream;
use tokio::io::{ReadHalf, WriteHalf};
use crate::tokio::io::AsyncBufReadExt;

use super::LocalMessage;

pub struct ServerNetwork {
    pub write_half: WriteHalf<TlsStream<TcpStream>>,
}

impl ServerNetwork {
    pub async fn new(ip: &str, port: u16, tx: std::sync::mpsc::Sender<LocalMessage>, idx: usize) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let addr = format!("{}:{}", ip, port)
            .to_socket_addrs()?
            .next()
            .ok_or("failed to resolve hostname")?;

        let socket = TcpStream::connect(&addr).await?;
        let cx = TlsConnector::builder().danger_accept_invalid_certs(true).build()?;
        let cx = tokio_native_tls::TlsConnector::from(cx);

        let socket = cx.connect(ip, socket).await?;
        let (read_half, write_half) = tokio::io::split(socket);

        let net_tx = tx.clone();
        std::thread::spawn(move || {
            futures::executor::block_on(ServerNetwork::run_network(net_tx, read_half, idx));
        });

        Ok(ServerNetwork {
            write_half,
        })
    }

    pub async fn run_network(tx: std::sync::mpsc::Sender<LocalMessage>, stream: ReadHalf<TlsStream<TcpStream>>, idx: usize) {
        let mut reader = tokio::io::BufReader::new(stream);
    
        loop {
            let mut result: String = "".to_string();
            match reader.read_line(&mut result).await {
            	Ok(_len) => {
      	            tx.send(LocalMessage::Network(result, idx)).unwrap();
      	        }
    
            	Err(..) => {
            		return;
            	}
            }
        }
    }
}
