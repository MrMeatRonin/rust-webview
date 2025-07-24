use std::thread;
use std::{net::TcpListener, sync::Arc};

use crate::communicate::SessionFactory;

/**
 * Socker server for accepting incoming tcp conncetions.
 */
pub struct SocketServer {
    session_factory: SessionFactory,
}

impl SocketServer {
    pub fn new(session_factory: SessionFactory) -> Arc<SocketServer> {
        return Arc::new(SocketServer { session_factory });
    }

    pub fn start(self: Arc<Self>, port: i32) -> std::io::Result<()> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port))?;
        let addr = listener.local_addr()?;
        println!("{}", format!("Server listening on port {}...", addr.port()));

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    println!("New connection: {}", stream.peer_addr()?);
                    let socket_server = Arc::clone(&self);

                    //provide threads to process stream
                    thread::spawn(
                        move || match socket_server.session_factory.create_on(stream) {
                            Ok(_) => todo!(),
                            Err(_) => todo!(),
                        },
                    );
                }
                Err(e) => {
                    eprintln!("Connection failed: {}", e);
                }
            }
        }
        Ok(())
    }
}
