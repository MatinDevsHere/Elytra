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

    // Create the dimension registry
    let mut dimension_registry = HashMap::new();
    dimension_registry.insert(
        "type".to_string(),
        Tag::String("minecraft:dimension_type".to_string()),
    );

    let mut overworld_details = HashMap::new();
    overworld_details.insert("piglin_safe".to_string(), Tag::Byte(0));
    overworld_details.insert("natural".to_string(), Tag::Byte(1));
    overworld_details.insert("ambient_light".to_string(), Tag::Float(0.0));
    overworld_details.insert(
        "infiniburn".to_string(),
        Tag::String("minecraft:infiniburn_overworld".to_string()),
    );
    overworld_details.insert("respawn_anchor_works".to_string(), Tag::Byte(0));
    overworld_details.insert("has_skylight".to_string(), Tag::Byte(1));
    overworld_details.insert("bed_works".to_string(), Tag::Byte(1));
    overworld_details.insert(
        "effects".to_string(),
        Tag::String("minecraft:overworld".to_string()),
    );
    overworld_details.insert("has_raids".to_string(), Tag::Byte(1));
    overworld_details.insert("logical_height".to_string(), Tag::Int(256));
    overworld_details.insert("coordinate_scale".to_string(), Tag::Float(1.0));
    overworld_details.insert("ultrawarm".to_string(), Tag::Byte(0));
    overworld_details.insert("has_ceiling".to_string(), Tag::Byte(0));

    let mut overworld_entry = HashMap::new();
    overworld_entry.insert(
        "name".to_string(),
        Tag::String("minecraft:overworld".to_string()),
    );
    overworld_entry.insert("id".to_string(), Tag::Int(0));
    overworld_entry.insert("element".to_string(), Tag::Compound(overworld_details));

    // Create direct list of dimension entries
    dimension_registry.insert(
        "value".to_string(),
        Tag::List(vec![Tag::Compound(overworld_entry)]),
    );
    compound.insert(
        "minecraft:dimension_type".to_string(),
        Tag::Compound(dimension_registry),
    );

    // Create the biome registry
    let mut biome_registry = HashMap::new();
    biome_registry.insert(
        "type".to_string(),
        Tag::String("minecraft:worldgen/biome".to_string()),
    );

    let mut plains_details = HashMap::new();
    plains_details.insert("precipitation".to_string(), Tag::String("rain".to_string()));
    plains_details.insert("temperature".to_string(), Tag::Float(0.8));
    plains_details.insert(
        "temperature_modifier".to_string(),
        Tag::String("none".to_string()),
    );
    plains_details.insert("downfall".to_string(), Tag::Float(0.4));
    plains_details.insert("scale".to_string(), Tag::Float(0.1));
    plains_details.insert("depth".to_string(), Tag::Float(0.125));
    plains_details.insert("category".to_string(), Tag::String("plains".to_string()));

    let mut effects = HashMap::new();
    effects.insert("sky_color".to_string(), Tag::Int(7907327));
    effects.insert("water_fog_color".to_string(), Tag::Int(329011));
    effects.insert("fog_color".to_string(), Tag::Int(12638463));
    effects.insert("water_color".to_string(), Tag::Int(4159204));
    effects.insert(
        "mood_sound".to_string(),
        Tag::Compound({
            let mut mood = HashMap::new();
            mood.insert("tick_delay".to_string(), Tag::Int(6000));
            mood.insert("offset".to_string(), Tag::Double(2.0));
            mood.insert(
                "sound".to_string(),
                Tag::String("minecraft:ambient.cave".to_string()),
            );
            mood.insert("block_search_extent".to_string(), Tag::Int(8));
            mood
        }),
    );
    plains_details.insert("effects".to_string(), Tag::Compound(effects));

    let mut plains_entry = HashMap::new();
    plains_entry.insert(
        "name".to_string(),
        Tag::String("minecraft:plains".to_string()),
    );
    plains_entry.insert("id".to_string(), Tag::Int(1));
    plains_entry.insert("element".to_string(), Tag::Compound(plains_details));

    // Create direct list of biome entries
    biome_registry.insert(
        "value".to_string(),
        Tag::List(vec![Tag::Compound(plains_entry)]),
    );
    compound.insert(
        "minecraft:worldgen/biome".to_string(),
        Tag::Compound(biome_registry),
    );

    Tag::Compound(compound)
}

/// Constructs a default dimension NBT compound tag for the world you are joining.
/// This example includes keys such as "min_y", "height", and "logical_height".
fn default_dimension() -> Tag {
    let mut compound = HashMap::new();

    // Add the required dimension properties
    compound.insert("piglin_safe".to_string(), Tag::Byte(0));
    compound.insert("natural".to_string(), Tag::Byte(1));
    compound.insert("ambient_light".to_string(), Tag::Float(0.0));
    compound.insert(
        "infiniburn".to_string(),
        Tag::String("minecraft:infiniburn_overworld".to_string()),
    );
    compound.insert("respawn_anchor_works".to_string(), Tag::Byte(0));
    compound.insert("has_skylight".to_string(), Tag::Byte(1));
    compound.insert("bed_works".to_string(), Tag::Byte(1));
    compound.insert(
        "effects".to_string(),
        Tag::String("minecraft:overworld".to_string()),
    );
    compound.insert("has_raids".to_string(), Tag::Byte(1));
    compound.insert("min_y".to_string(), Tag::Int(0));
    compound.insert("height".to_string(), Tag::Int(256));
    compound.insert("logical_height".to_string(), Tag::Int(256));
    compound.insert("coordinate_scale".to_string(), Tag::Float(1.0));
    compound.insert("ultrawarm".to_string(), Tag::Byte(0));
    compound.insert("has_ceiling".to_string(), Tag::Byte(0));

    Tag::Compound(compound)
}
