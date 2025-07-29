use anyhow::{Context, Result};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, tcp::OwnedReadHalf},
    sync::mpsc,
};

use crate::communicate::decoder::Decoder;

pub struct TcpSession {
    reader: tokio::task::JoinHandle<()>,
    writer: tokio::task::JoinHandle<()>,
    command_tx: mpsc::Sender<SessionCommand>,
}

#[derive(Debug)]
pub enum SessionCommand {
    Send(Vec<u8>),
    Close,
}

impl TcpSession {
    pub async fn new(stream: TcpStream, decoder: Decoder) -> Result<Self> {
        let (read_half, write_half) = stream.into_split();
        let (command_tx, command_rx) = mpsc::channel(32);

        let reader = Self::spawn_reader(read_half, command_tx.clone(), decoder);
        let writer = Self::spawn_writer(write_half, command_rx);

        Ok(Self {
            reader,
            writer,
            command_tx,
        })
    }

    fn spawn_reader(
        mut read_half: OwnedReadHalf,
        command_tx: mpsc::Sender<SessionCommand>,
        mut decoder: Decoder,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut buf = [0; 1024];
            loop {
                match read_half.read(&mut buf).await {
                    Ok(0) => {
                        // Connection closed by peer
                        let _ = command_tx.send(SessionCommand::Close).await;
                        break;
                    }
                    Ok(n) => {
                        let data = &buf[..n];
                        decoder.on_received(data);
                    }
                    Err(e) => {
                        eprintln!("Read error: {}", e);
                        let _ = command_tx.send(SessionCommand::Close).await;
                        break;
                    }
                }
            }
        })
    }

    fn spawn_writer(
        mut write_half: tokio::net::tcp::OwnedWriteHalf,
        mut command_rx: mpsc::Receiver<SessionCommand>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            while let Some(cmd) = command_rx.recv().await {
                match cmd {
                    SessionCommand::Send(data) => {
                        if let Err(e) = write_half.write_all(&data).await {
                            eprintln!("Write error: {}", e);
                            break;
                        }
                    }
                    SessionCommand::Close => {
                        break;
                    }
                }
            }
            // Graceful shutdown
            let _ = write_half.shutdown().await;
        })
    }

    pub async fn send(&self, data: Vec<u8>) -> Result<()> {
        self.command_tx
            .send(SessionCommand::Send(data))
            .await
            .context("Failed to send data to session")
    }

    pub async fn run(mut self) -> Result<()> {
        // Wait for either reader or writer to complete
        tokio::select! {
            _ = &mut self.reader => {
                eprintln!("Reader task finished");
            }
            _ = &mut self.writer => {
                eprintln!("Writer task finished");
            }
        }

        // Send close command if not already sent
        let _ = self.command_tx.send(SessionCommand::Close).await;

        // Wait for both tasks to complete
        tokio::try_join!(
            async { self.reader.await.context("Reader task failed") },
            async { self.writer.await.context("Writer task failed") }
        )?;

        Ok(())
    }

    pub async fn close(self) -> Result<()> {
        self.command_tx
            .send(SessionCommand::Close)
            .await
            .context("Failed to send close command")?;
        self.run().await
    }
}
