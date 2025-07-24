use serde::{Deserialize, Serialize};
use std::net::TcpStream;

#[derive(Serialize, Deserialize, Debug)]
struct Request {
    handler: String,
}

pub struct Session {}

impl Session {
    pub fn new() -> Session {
        Session {}
    }

    pub fn on_received(self: &Self, raw: &str) {
        let request: Request = serde_json::from_str(raw).unwrap();
        println!("{:?}", request);
    }

    pub fn send(msg: &str) {}
}
