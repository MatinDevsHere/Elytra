use super::packet::*;
use tokio::io;

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

    /// Reads the packet from the buffer
    fn read_from_buffer(buffer: &mut MinecraftPacketBuffer) -> io::Result<Self> {
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

    /// Writes the packet to the buffer
    fn write_to_buffer(&self, buffer: &mut MinecraftPacketBuffer) -> io::Result<()> {
        buffer.write_varint(self.protocol_version);
        buffer.write_string(&self.server_address);
        buffer.write_u16(self.server_port);
        buffer.write_varint(self.next_state);
        Ok(())
    }
}
