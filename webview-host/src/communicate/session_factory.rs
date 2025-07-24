use std::io::{Read, Write};
use std::net::TcpStream;

use crate::communicate::Session;

pub struct SessionFactory {}

impl SessionFactory {
    pub fn new() -> Self {
        return SessionFactory {};
    }

    /**
     * Create session upon TcpStream.
     * Define packet transfering structure (protocol).
     */
    pub fn create_on(&self, mut stream: TcpStream) -> std::io::Result<()> {
         let session = Session::new();
        let mut len_buffer = [0u8; size_of::<u32>()];

        loop {
            // Read head
            match stream.read_exact(&mut len_buffer) {
                Ok(_) => (),
                Err(_) => {
                    break;
                }
            }

            // Read data
            let msg_len = u32::from_be_bytes(len_buffer) as usize;
            let mut buffer = vec![0; msg_len];
            match stream.read_exact(&mut buffer) {
                Ok(_) => (),
                Err(_) => {
                    break;
                }
            }

            // Process message
            let received = String::from_utf8_lossy(&buffer);
            println!("Received ({} bytes): {}", msg_len, received);

           session.on_received(&received);

            // Send response (same protocol)// Send response (same protocol)
            let response = format!("Echo: {}", received);
            let response_len = response.len() as u32;
            stream.write_all(&response_len.to_be_bytes())?;
            stream.write_all(response.as_bytes())?;
            stream.flush()?;
        }

        stream.shutdown(std::net::Shutdown::Both)?;
        Ok(())
    }
}
