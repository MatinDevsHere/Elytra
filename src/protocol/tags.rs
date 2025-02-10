use crate::protocol::packet::{MinecraftPacketBuffer, Packet};

pub struct TagsPacket {}

impl Packet for TagsPacket {
    fn packet_id() -> i32
    where
        Self: Sized,
    {
        0x5B
    }

    fn write_to_buffer(&self, buffer: &mut MinecraftPacketBuffer) -> std::io::Result<()> {
        buffer.write_varint(Self::packet_id());

        // Length of Block Tags Array
        buffer.write_varint(0);

        // Length of Item Tags Array
        buffer.write_varint(0);

        // Length of Fluid Tags Array
        buffer.write_varint(0);

        // Length of Entity Tags Array
        buffer.write_varint(0);

        Ok(())
    }
}

impl TagsPacket {
    pub fn new() -> TagsPacket {
        TagsPacket {}
    }
}
