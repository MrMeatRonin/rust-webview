#![allow(warnings)]

mod communicate;
mod webview;

use std::collections::HashMap;
use std::thread;

use mini_redis::{Connection, Frame};
use tokio::net::{TcpListener, TcpStream};

use crate::communicate::Decoder;
use crate::communicate::TcpSession;

#[tokio::main]
async fn main() {
    // Bind the listener to the address
    let listener = TcpListener::bind("127.0.0.1:39174").await.unwrap();

    loop {
        // The second item contains the IP and port of the new connection.
        let (stream, _) = listener.accept().await.unwrap();

        tokio::spawn(async move {
            let decoder = Decoder::new(vec![]);
            TcpSession::new(stream, decoder).await.unwrap().run().await;
        });
    }
}
