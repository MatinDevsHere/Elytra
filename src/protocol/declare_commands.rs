use crate::protocol::packet::MinecraftPacketBuffer;
use crate::Packet;

pub struct DeclareCommandsPacket {
    commands_count: u8,
}

impl Packet for DeclareCommandsPacket {
    fn packet_id() -> i32
    where
        Self: Sized,
    {
        0x10
    }

    fn write_to_buffer(&self, buffer: &mut MinecraftPacketBuffer) -> std::io::Result<()> {
        buffer.write_varint(Self::packet_id());
        buffer.write_varint(self.commands_count as i32);

        Ok(())
    }
}

impl DeclareCommandsPacket {
    pub fn new(commands_count: u8) -> DeclareCommandsPacket {
        DeclareCommandsPacket { commands_count }
    }
}
