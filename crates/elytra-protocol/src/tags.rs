use crate::packet::{MinecraftPacketBuffer, Packet};

pub struct TagsPacket {
    block_tags: Vec<Tag>,
    item_tags: Vec<Tag>,
    fluid_tags: Vec<Tag>,
    entity_tags: Vec<Tag>,
}

struct Tag {
    name: String,
    entries: Vec<i32>,
}

impl Packet for TagsPacket {
    fn packet_id() -> i32
    where
        Self: Sized,
    {
        0x5B
    }

    fn write_to_buffer(&self, buffer: &mut MinecraftPacketBuffer) -> std::io::Result<()> {
        buffer.write_varint(Self::packet_id());

        // Write block tags
        buffer.write_varint(self.block_tags.len() as i32);
        for tag in &self.block_tags {
            buffer.write_string(&tag.name);
            buffer.write_varint(tag.entries.len() as i32);
            for entry in &tag.entries {
                buffer.write_varint(*entry);
            }
        }

        // Write item tags
        buffer.write_varint(self.item_tags.len() as i32);
        for tag in &self.item_tags {
            buffer.write_string(&tag.name);
            buffer.write_varint(tag.entries.len() as i32);
            for entry in &tag.entries {
                buffer.write_varint(*entry);
            }
        }

        // Write fluid tags
        buffer.write_varint(self.fluid_tags.len() as i32);
        for tag in &self.fluid_tags {
            buffer.write_string(&tag.name);
            buffer.write_varint(tag.entries.len() as i32);
            for entry in &tag.entries {
                buffer.write_varint(*entry);
            }
        }

        // Write entity tags
        buffer.write_varint(self.entity_tags.len() as i32);
        for tag in &self.entity_tags {
            buffer.write_string(&tag.name);
            buffer.write_varint(tag.entries.len() as i32);
            for entry in &tag.entries {
                buffer.write_varint(*entry);
            }
        }

        Ok(())
    }
}

impl TagsPacket {
    pub fn new() -> Self {
        // Block tags
        let block_tags = vec![
            Tag {
                name: "minecraft:mineable/pickaxe".to_string(),
                entries: vec![1, 2, 3], // Example block IDs that can be mined with pickaxe
            },
            Tag {
                name: "minecraft:mineable/axe".to_string(),
                entries: vec![4, 5, 6], // Example block IDs that can be mined with axe
            },
            Tag {
                name: "minecraft:mineable/shovel".to_string(),
                entries: vec![7, 8, 9], // Example block IDs that can be mined with shovel
            },
        ];

        // Item tags
        let item_tags = vec![
            Tag {
                name: "minecraft:tools".to_string(),
                entries: vec![256, 257, 258], // Example item IDs for tools
            },
            Tag {
                name: "minecraft:weapons".to_string(),
                entries: vec![267, 268, 269], // Example item IDs for weapons
            },
        ];

        // Fluid tags
        let fluid_tags = vec![
            Tag {
                name: "minecraft:water".to_string(),
                entries: vec![8, 9], // Water and flowing water
            },
            Tag {
                name: "minecraft:lava".to_string(),
                entries: vec![10, 11], // Lava and flowing lava
            },
        ];

        // Entity tags
        let entity_tags = vec![
            Tag {
                name: "minecraft:raiders".to_string(),
                entries: vec![36, 37, 38], // Example entity IDs for raiders
            },
            Tag {
                name: "minecraft:skeletons".to_string(),
                entries: vec![51, 52], // Example entity IDs for skeleton types
            },
        ];

        Self {
            block_tags,
            item_tags,
            fluid_tags,
            entity_tags,
        }
    }
}
