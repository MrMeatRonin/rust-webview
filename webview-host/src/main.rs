#![allow(warnings)]

mod communicate;
mod webview;

use std::collections::HashMap;
use std::thread;

use mini_redis::{Connection, Frame};
use tokio::net::{TcpListener, TcpStream};

use crate::communicate::Decoder;
use crate::communicate::Request;
use crate::communicate::Session;

#[tokio::main]
async fn main() {
    // Bind the listener to the address
    let listener = TcpListener::bind("127.0.0.1:39174").await.unwrap();

    loop {
        // The second item contains the IP and port of the new connection.
        let (stream, _) = listener.accept().await.unwrap();

        //make socket accept channels instead of providing them
        //manage tcp session more carefully
        //ToDo: create an invocation manager to manage sessions
        //can derive sub session in session (like browser scope)

        tokio::spawn(async move {
            let (mut session, mut reader_rx, mut writer_tx) = Session::link(stream);

            let mut decoder = Decoder::new(|packet| {
                let request: Request = serde_json::from_str(&packet)?;
                println!("Received request {:?}", request);
                Ok(())
            });

            while let Some(bytes) = reader_rx.recv().await {
                match bytes {
                    communicate::Input::EOF => {
                        break;
                    }
                    communicate::Input::Data(bytes) => {
                        decoder.on_received(&bytes);
                    }
                }
            }
            session.shutdown();
        });
    }
}
