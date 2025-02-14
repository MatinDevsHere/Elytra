use crate::packet::{MinecraftPacketBuffer, Packet};

#[derive(Debug)]
pub struct ClientSettingsPacket {
    locale: String,
    view_distance: u8,
    chat_mode: u8, // VarInt enum: 0 = enabled, 1 = commands only, 2 = hidden
    chat_colors: bool,
    displayed_skin_parts: u8, // Bitmask for skin parts
    main_hand: u8,            // VarInt enum: 0 = Left, 1 = Right
}

impl ClientSettingsPacket {
    pub fn new(
        locale: String,
        view_distance: u8,
        chat_mode: u8,
        chat_colors: bool,
        displayed_skin_parts: u8,
        main_hand: u8,
    ) -> Self {
        Self {
            locale,
            view_distance,
            chat_mode,
            chat_colors,
            displayed_skin_parts,
            main_hand,
        }
    }
}

impl Packet for ClientSettingsPacket {
    fn packet_id() -> i32 {
        0x05
    }

    fn read_from_buffer(buffer: &mut MinecraftPacketBuffer) -> std::io::Result<Self> {
        Ok(Self {
            locale: buffer.read_string()?,
            view_distance: buffer.read_u8()?,
            chat_mode: buffer.read_varint()? as u8,
            chat_colors: buffer.read_bool()?,
            displayed_skin_parts: buffer.read_u8()?,
            main_hand: buffer.read_varint()? as u8,
        })
    }

    fn write_to_buffer(&self, buffer: &mut MinecraftPacketBuffer) -> std::io::Result<()> {
        buffer.write_string(&self.locale);
        buffer.write_u8(self.view_distance);
        buffer.write_varint(self.chat_mode as i32);
        buffer.write_bool(self.chat_colors);
        buffer.write_u8(self.displayed_skin_parts);
        buffer.write_varint(self.main_hand as i32);
        Ok(())
    }
}
