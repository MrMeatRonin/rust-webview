use bytes::Bytes;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::mpsc::{self, Receiver, Sender, channel},
    task::JoinHandle,
};

use std::sync::atomic::Ordering::Relaxed;

// size of reader buffer
const BUFFER_SIZE: usize = 1024;

// how many frame (Bytes) can be stored in sync channel
const FRAME_BOUND: usize = 8;

pub enum Input {
    EOF,
    Data(Bytes),
}

pub enum Output {
    SHUTDOWN,
    Data(Bytes),
}

pub struct Session {
    writer_tx: Sender<Output>,
}

impl Session {
    pub fn link(stream: TcpStream) -> (Session, Receiver<Input>, Sender<Output>) {
        let (mut read_hf, mut write_hf) = stream.into_split();

        //define reader and writer channels, each channel regard None as the terminating singal
        let (reader_tx, reader_rx) = mpsc::channel::<Input>(FRAME_BOUND);
        let (writer_tx, mut writer_rx) = mpsc::channel::<Output>(FRAME_BOUND);

        // create reader thread to send message to read_rx
        let reader = tokio::spawn(async move {
            let mut buffer = [0; BUFFER_SIZE];

            loop {
                match read_hf.read(&mut buffer).await {
                    Ok(0) => {
                        //reach end of stream, send a elegant terminating signal (use None)
                        reader_tx.send(Input::EOF);
                        break; // will automatically drop reader channel
                    }
                    Ok(n) => {
                        let bytes = Bytes::copy_from_slice(&buffer[..n]);
                        if let Err(e) = reader_tx.send(Input::Data(bytes)).await {
                            eprint!("Send read bytes error: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Read error: {}", e);
                        break;
                    }
                }
            }
        });

        // create writer thread
        let writer = tokio::spawn(async move {
            while let Some(bytes) = writer_rx.recv().await {
                match bytes {
                    Output::SHUTDOWN => {
                        break;
                    }
                    Output::Data(bytes) => {
                        if let Err(e) = write_hf.write_all(&bytes).await {
                            eprintln!("Write error: {}", e);
                            break;
                        }
                    }
                }
            }
        });

        (
            Session {
                writer_tx: writer_tx.clone(),
            },
            reader_rx,
            writer_tx,
        )
    }

    pub fn shutdown(&self) {
        self.writer_tx.send(Output::SHUTDOWN);
    }
}
