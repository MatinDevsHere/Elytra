use super::packet::*;
use crate::protocol::packet::{MinecraftPacketBuffer, Packet};

pub struct StatusRequestPacket;

impl Packet for StatusRequestPacket {
    fn read(_buffer: &mut MinecraftPacketBuffer) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        Ok(StatusRequestPacket)
    }
}

pub struct StatusResponsePacket {
    pub response: String,
}

impl Packet for StatusResponsePacket {
    fn packet_id() -> i32 {
        0x00
    }

    fn write(&self, buffer: &mut MinecraftPacketBuffer) -> std::io::Result<()> {
        buffer.write_varint(Self::packet_id());
        buffer.write_string(&self.response);
        Ok(())
    }
}
