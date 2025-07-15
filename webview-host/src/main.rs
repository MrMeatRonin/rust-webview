use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
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

        // Send response (same protocol)// Send response (same protocol)
        let response = format!("Echo: {}", received);
        let response_len = response.len() as u32;
        stream.write_all(&response_len.to_be_bytes())?;
        stream.write_all(response.as_bytes())?;
        stream.flush()?;
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:39174")?;
    println!("Server listening on port 39174...");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr()?);
                thread::spawn(move || {
                    handle_client(stream).unwrap_or_else(|e| eprintln!("Error: {}", e));
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }

    Ok(())
}
