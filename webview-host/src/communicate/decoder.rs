use crate::communicate::{
    tcp_session::SessionCommand,
    traits::{Request, RequestHandler},
};
use anyhow::Error;
use mini_redis::buffer;
use std::{
    cmp,
    collections::HashMap,
    sync::{Arc, mpsc},
    usize,
};

const HEAD_LEN: i32 = 4;
const MAX_PACKET_SIZE: usize = 1024 * 1024;

#[derive(Debug)]
enum DecodeState {
    /// Represent waiting for reading of head.
    /// # Parameters
    /// - `read`: how many bytes already read
    ExpectHead(usize),

    ///Represent waiting for reading of body.
    /// # Parameters
    /// - `read`: how many bytes already read
    ExpectBody(usize),
}

/**
 * Manage data receiving in bytes level and jobs schedularing. (to handler)
 */
pub struct Decoder {
    len_buffer: [u8; 4],
    packet_buffer: Box<Vec<u8>>,
    state: DecodeState,
    packet_handler: Box<dyn FnMut(String) -> anyhow::Result<()> + Send>,
}

impl Decoder {
    pub fn new<F>(packet_handler: F) -> Self
    where
        F: FnMut(String) -> anyhow::Result<()> + 'static + Send,
    {
        Decoder {
            len_buffer: [0u8; size_of::<u32>()],
            packet_buffer: Box::new(Vec::with_capacity(0)),
            state: DecodeState::ExpectHead(0),
            packet_handler: Box::new(packet_handler),
        }
    }

    pub fn on_received(&mut self, data: &[u8]) -> anyhow::Result<()> {
        if (data.is_empty()) {
            return Ok(());
        }

        match self.state {
            DecodeState::ExpectHead(mut read) => {
                // read available bytes into buffer
                let len_buffer = &mut self.len_buffer;
                let available = cmp::min(data.len(), len_buffer.len() - read);
                len_buffer[read..read + available].copy_from_slice(&data[..available]);

                // update and check if reading completed
                read += available;
                if (read == len_buffer.len()) {
                    let packet_len = u32::from_be_bytes(*len_buffer) as usize;

                    //verify packet len
                    if packet_len > MAX_PACKET_SIZE {
                        return Err(anyhow::Error::msg("Packet is too large."));
                    }

                    //init paket buffer
                    self.packet_buffer = Box::new(Vec::with_capacity(packet_len));

                    //change state and recursive call
                    self.state = DecodeState::ExpectBody(0);
                    self.on_received(&data[read..])
                } else {
                    self.state = DecodeState::ExpectHead(read);
                    Ok(())
                }
            }
            DecodeState::ExpectBody(mut read) => {
                // read available bytes into buffer
                let packet_buffer = &mut self.packet_buffer;
                let available = cmp::min(data.len(), packet_buffer.capacity() - read);
                packet_buffer.extend_from_slice(&data[..available]);

                // update and check if reading completed
                read += available;
                if (read == packet_buffer.len()) {
                    let packet = String::from_utf8_lossy(&packet_buffer);

                    //for consumers
                    match (self.packet_handler)(packet.into_owned()) {
                        Ok(_) => {}
                        Err(error) => {
                            println!("{:?}", error)
                        }
                    }

                    //change state and recursive call (use original len buffer)
                    self.state = DecodeState::ExpectHead(0);
                    self.on_received(&data[read..])
                } else {
                    self.state = DecodeState::ExpectBody(read);
                    Ok(())
                }
            }
        }
    }
}
