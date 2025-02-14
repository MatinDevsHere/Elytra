use crate::packet::{MinecraftPacketBuffer, Packet};
use std::io;

#[derive(Debug, Clone)]
pub struct KeepAlivePacket {
    pub keep_alive_id: i64,
}

impl Packet for KeepAlivePacket {
    fn packet_id() -> i32 {
        0x1F
    }

    fn read_from_buffer(buffer: &mut MinecraftPacketBuffer) -> io::Result<Self> {
        let _packet_id = buffer.read_varint()?;
        Ok(KeepAlivePacket {
            keep_alive_id: buffer.read_i64()?,
        })
    }

    fn write_to_buffer(&self, buffer: &mut MinecraftPacketBuffer) -> io::Result<()> {
        buffer.write_varint(Self::packet_id());
        buffer.write_i64(self.keep_alive_id);
        Ok(())
    }
}

impl KeepAlivePacket {
    pub fn new(keep_alive_id: i64) -> Self {
        Self { keep_alive_id }
    }
}
