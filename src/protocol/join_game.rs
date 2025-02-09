use super::nbt::Tag;
use super::packet::*;
use std::collections::HashMap;
use tokio::io::Result;

pub struct JoinGamePacket {
    pub entity_id: i32,
    pub is_hardcore: bool,
    pub gamemode: u8,
    pub previous_gamemode: i8,    // Byte, -1 if no previous gamemode
    pub world_names: Vec<String>, // Array of Identifier
    pub dimension_codec: Tag,     // NBT Tag Compound
    pub dimension: Tag,           // NBT Tag Compound
    pub world_name: String,       // Identifier
    pub hashed_seed: i64,         // Long
    pub max_players: i32,         // VarInt, but ignored by client
    pub view_distance: i32,       // VarInt (2-32)
    pub reduced_debug_info: bool,
    pub enable_respawn_screen: bool,
    pub is_debug: bool,
    pub is_flat: bool,
}

impl JoinGamePacket {
    // A helper constructor that builds the NBT data with default values.
    pub fn new(entity_id: i32, world_names: Vec<String>, world_name: String) -> Self {
        Self {
            entity_id,
            is_hardcore: false,
            gamemode: 0,
            previous_gamemode: -1,
            world_names,
            // Build an NBT compound with keys like "minecraft:dimension_type" and "minecraft:worldgen/biome".
            dimension_codec: default_dimension_codec(),
            // Build the dimension compound with required keys.
            dimension: default_dimension(),
            world_name,
            hashed_seed: 0,
            max_players: 100,
            view_distance: 10,
            reduced_debug_info: false,
            enable_respawn_screen: false,
            is_debug: false,
            is_flat: false,
        }
    }
}

impl Packet for JoinGamePacket {
    fn packet_id() -> i32 {
        0x24
    }

    fn write_to_buffer(&self, buffer: &mut MinecraftPacketBuffer) -> Result<()> {
        buffer.write_varint(Self::packet_id());
        buffer.write_i32(self.entity_id);
        buffer.write_bool(self.is_hardcore);
        buffer.write_u8(self.gamemode);
        buffer.write_i8(self.previous_gamemode);

        // Write world names array
        buffer.write_varint(self.world_names.len() as i32);
        for world_name in &self.world_names {
            buffer.write_string(world_name);
        }

        // Write NBT data—in this case our default compounds include keys like
        // "minecraft:dimension_type" and "minecraft:worldgen/biome".
        self.dimension_codec.write(buffer, "dimension_codec")?;
        self.dimension.write(buffer, "dimension")?;

        buffer.write_string(&self.world_name);
        buffer.write_i64(self.hashed_seed);
        buffer.write_varint(self.max_players);
        buffer.write_varint(self.view_distance);
        buffer.write_bool(self.reduced_debug_info);
        buffer.write_bool(self.enable_respawn_screen);
        buffer.write_bool(self.is_debug);
        buffer.write_bool(self.is_flat);

        Ok(())
    }
}

/// Constructs a default dimension codec NBT compound tag that includes the keys
/// required by the protocol, such as "minecraft:dimension_type" and "minecraft:worldgen/biome".
///
/// TODO: Has to be read from config
fn default_dimension_codec() -> Tag {
    let mut compound = HashMap::new();

    // Create the "minecraft:dimension_type" entry.
    let mut dimension_type_value = HashMap::new();
    {
        let mut overworld_details = HashMap::new();
        // Minimal set of properties; expand as needed.
        overworld_details.insert("piglin_safe".to_string(), Tag::Byte(1));
        overworld_details.insert("natural".to_string(), Tag::Byte(0));
        overworld_details.insert("ambient_light".to_string(), Tag::Float(1.0));
        overworld_details.insert("fixed_time".to_string(), Tag::Long(0));
        overworld_details.insert("infiniburn".to_string(), Tag::String("".to_string()));
        overworld_details.insert("respawn_anchor_works".to_string(), Tag::Byte(0));
        overworld_details.insert("has_skylight".to_string(), Tag::Byte(1));
        overworld_details.insert("bed_works".to_string(), Tag::Byte(0));
        overworld_details.insert(
            "effects".to_string(),
            Tag::String("minecraft:overworld".to_string()),
        );
        overworld_details.insert("has_raids".to_string(), Tag::Byte(0));
        overworld_details.insert("logical_height".to_string(), Tag::Int(256));
        overworld_details.insert("coordinate_scale".to_string(), Tag::Float(1.0));
        overworld_details.insert("ultrawarm".to_string(), Tag::Byte(0));
        overworld_details.insert("has_ceiling".to_string(), Tag::Byte(0));
        dimension_type_value.insert(
            "minecraft:overworld".to_string(),
            Tag::Compound(overworld_details),
        );
    }
    compound.insert(
        "minecraft:dimension_type".to_string(),
        Tag::Compound(dimension_type_value),
    );

    // Create the "minecraft:worldgen/biome" entry.
    let mut biome_value = HashMap::new();
    {
        let mut plains_biome = HashMap::new();
        // Minimal biome properties.
        plains_biome.insert("precipitation".to_string(), Tag::String("none".to_string()));
        let mut effects = HashMap::new();
        effects.insert("sky_color".to_string(), Tag::Int(7842047));
        effects.insert("water_fog_color".to_string(), Tag::Int(329011));
        effects.insert("water_color".to_string(), Tag::Int(4159204));
        effects.insert("fog_color".to_string(), Tag::Int(12638463));
        plains_biome.insert("effects".to_string(), Tag::Compound(effects));
        biome_value.insert("minecraft:plains".to_string(), Tag::Compound(plains_biome));
    }
    compound.insert(
        "minecraft:worldgen/biome".to_string(),
        Tag::Compound(biome_value),
    );

    Tag::Compound(compound)
}

/// Constructs a default dimension NBT compound tag for the world you are joining.
/// This example includes keys such as "min_y", "height", and "logical_height".
fn default_dimension() -> Tag {
    let mut compound = HashMap::new();
    compound.insert("min_y".to_string(), Tag::Int(0));
    compound.insert("height".to_string(), Tag::Int(256));
    compound.insert("logical_height".to_string(), Tag::Int(256));
    Tag::Compound(compound)
}
