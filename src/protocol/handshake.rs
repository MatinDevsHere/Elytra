use super::packet::*;
use tokio::io;

#[derive(Debug)]
pub struct HandshakePacket {
    pub protocol_version: i32,
    pub server_address: String,
    pub server_port: u16,
    pub next_state: i32,
}

impl Packet for HandshakePacket {
    fn packet_id() -> i32 {
        0x00
    }

    fn write(&self, buffer: &mut MinecraftPacketBuffer) -> io::Result<()> {
        buffer.write_varint(self.protocol_version);
        buffer.write_string(&self.server_address);
        buffer.write_varint(self.server_port as i32);
        buffer.write_varint(self.next_state);
        Ok(())
    }

    fn read(buffer: &mut MinecraftPacketBuffer) -> io::Result<Self> {
        Ok(HandshakePacket {
            protocol_version: buffer.read_varint()?,
            server_address: buffer.read_string()?,
            server_port: buffer.read_varint()? as u16,
            next_state: buffer.read_varint()?,
        })
    }
}
