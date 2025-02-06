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
    fn write(&self, buffer: &mut MinecraftPacketBuffer) -> std::io::Result<()> {
        buffer.write_string(&self.response);
        Ok(())
    }
}
