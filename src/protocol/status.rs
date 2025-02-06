use super::packet::*;
use crate::protocol::packet::{MinecraftPacketBuffer, Packet};

/// The packet sent by the client to request the server status.
pub struct StatusRequestPacket;

/// Packet implementation for the status request packet.
impl Packet for StatusRequestPacket {
    /// Reads the packet from the buffer.
    fn read(_buffer: &mut MinecraftPacketBuffer) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        Ok(StatusRequestPacket)
    }
}

/// The packet sent by the server to respond to the status request.
/// The response is a JSON string.
pub struct StatusResponsePacket {
    pub response_json: String,
}

/// Packet implementation for the status response packet.
impl Packet for StatusResponsePacket {
    /// Writes the packet to the buffer.
    /// The response is a JSON string.
    fn write(&self, buffer: &mut MinecraftPacketBuffer) -> std::io::Result<()> {
        buffer.write_varint(Self::packet_id());
        buffer.write_string(&self.response_json);
        Ok(())
    }
}
