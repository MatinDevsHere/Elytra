use super::packet::*;
use crate::protocol::status::StatusResponsePacket;
use serde_json::json;
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// Handshake packet
#[derive(Debug)]
pub struct HandshakePacket {
    pub protocol_version: i32,
    pub server_address: String,
    pub server_port: u16,
    pub next_state: i32,
}

/// Handshake packet impl
impl Packet for HandshakePacket {
    /// Packet ID
    fn packet_id() -> i32 {
        0x00
    }

    /// Writes the packet to the buffer
    fn write(&self, buffer: &mut MinecraftPacketBuffer) -> io::Result<()> {
        buffer.write_varint(self.protocol_version);
        buffer.write_string(&self.server_address);
        buffer.write_u16(self.server_port);
        buffer.write_varint(self.next_state);
        Ok(())
    }

    /// Reads the packet from the buffer
    fn read(buffer: &mut MinecraftPacketBuffer) -> io::Result<Self> {
        let _packet_length = buffer.read_varint()?;
        let packet_id = buffer.read_varint()?;

        if packet_id != 0x00 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid packet ID",
            ));
        }

        Ok(HandshakePacket {
            protocol_version: buffer.read_varint()?,
            server_address: buffer.read_string()?,
            server_port: buffer.read_u16()?,
            next_state: buffer.read_varint()?,
        })
    }
}

/// Handles the handshake packet next state
pub async fn handle_handshake(mut socket: TcpStream, handshake: HandshakePacket) -> io::Result<()> {
    match handshake.next_state {
        1 => {
            // Wait for the status request packet
            let mut buffer = [0u8; 1024];
            socket.read(&mut buffer).await?;

            // Build and send status response
            let status_json = json!({
                "version": {
                    "name": "1.16.5",
                    "protocol": 754
                },
                "players": {
                    "max": 100,
                    "online": 0,
                    "sample": []
                },
                "description": {
                    "text": "Elytra Server"
                }
            });

            let response = StatusResponsePacket {
                response_json: status_json.to_string(),
            };

            let mut packet_buffer = MinecraftPacketBuffer::new();
            response.write(&mut packet_buffer)?;

            // Write length prefix and packet data
            let mut final_buffer = MinecraftPacketBuffer::new();
            final_buffer.write_varint(packet_buffer.buffer.len() as i32);
            final_buffer.buffer.extend_from_slice(&packet_buffer.buffer);

            socket.write_all(&final_buffer.buffer).await?;
        }
        2 => {
            // Handle login state
            let mut response = MinecraftPacketBuffer::new();
            response.write_varint(0x00);
            socket.write_all(&response.buffer).await?;
        }
        _ => {
            panic!("Unknown next state: {}", handshake.next_state);
        }
    }
    Ok(())
}
