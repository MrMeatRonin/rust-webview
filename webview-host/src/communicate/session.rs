use std::sync::{
    Arc,
    atomic::AtomicBool,
    mpsc::{Receiver, SyncSender, sync_channel},
};

use bytes::Bytes;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    task::JoinHandle,
};

use std::sync::atomic::Ordering::Relaxed;

// size of reader buffer
const BUFFER_SIZE: usize = 1024;

// how many frame (Bytes) can be stored in sync channel
const FRAME_BOUND: usize = 8;

pub struct Session {
    writer_tx: SyncSender<Option<Bytes>>,
}

impl Session {
    pub fn link(
        stream: TcpStream,
    ) -> (Session, Receiver<Option<Bytes>>, SyncSender<Option<Bytes>>) {
        let (mut read_hf, mut write_hf) = stream.into_split();

        //define reader and writer channels, each channel regard None as the terminating singal
        let (reader_tx, reader_rx) = sync_channel::<Option<Bytes>>(FRAME_BOUND);
        let (writer_tx, writer_rx) = sync_channel::<Option<Bytes>>(FRAME_BOUND);

        // create reader thread to send message to read_rx
        let reader = tokio::spawn(async move {
            let mut buffer = [0; BUFFER_SIZE];

            loop {
                match read_hf.read(&mut buffer).await {
                    Ok(0) => {
                        //reach end of stream, send a elegant terminating signal (use None)
                        reader_tx.send(None);

                        break; // will automatically drop reader channel
                    }
                    Ok(n) => {
                        let bytes = Bytes::copy_from_slice(&buffer[..n]);
                        if let Err(e) = reader_tx.send(Some(bytes)) {
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
            while let Ok(bytes) = writer_rx.recv() {
                if let Some(bytes) = bytes {
                    //send message
                    if let Err(e) = write_hf.write_all(&bytes).await {
                        eprintln!("Write error: {}", e);
                        break;
                    }
                } else {
                    // received elegant termianting signal
                    break;
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
        self.writer_tx.send(None);
    }
}
