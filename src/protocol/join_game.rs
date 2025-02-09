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
            dimension_codec: default_dimension_codec(),
            dimension: default_dimension(),
            world_name,
            hashed_seed: 0,
            max_players: 100,
            view_distance: 10,
            reduced_debug_info: false,
            enable_respawn_screen: true,
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

        // Write NBT data
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
fn default_dimension_codec() -> Tag {
    let mut root = HashMap::new();

    // Create the dimension type registry
    let mut dimension_type = HashMap::new();
    dimension_type.insert(
        "type".to_owned(),
        Tag::String("minecraft:dimension_type".to_owned()),
    );

    let mut dimension_type_value = HashMap::new();
    {
        let mut overworld = HashMap::new();
        overworld.insert("piglin_safe".to_owned(), Tag::Byte(0));
        overworld.insert("natural".to_owned(), Tag::Byte(1));
        overworld.insert("ambient_light".to_owned(), Tag::Float(0.0));
        overworld.insert(
            "infiniburn".to_owned(),
            Tag::String("minecraft:infiniburn_overworld".to_owned()),
        );
        overworld.insert("respawn_anchor_works".to_owned(), Tag::Byte(0));
        overworld.insert("has_skylight".to_owned(), Tag::Byte(1));
        overworld.insert("bed_works".to_owned(), Tag::Byte(1));
        overworld.insert(
            "effects".to_owned(),
            Tag::String("minecraft:overworld".to_owned()),
        );
        overworld.insert("has_raids".to_owned(), Tag::Byte(1));
        overworld.insert("logical_height".to_owned(), Tag::Int(256));
        overworld.insert("coordinate_scale".to_owned(), Tag::Float(1.0));
        overworld.insert("ultrawarm".to_owned(), Tag::Byte(0));
        overworld.insert("has_ceiling".to_owned(), Tag::Byte(0));

        let mut value = HashMap::new();
        value.insert(
            "name".to_owned(),
            Tag::String("minecraft:overworld".to_owned()),
        );
        value.insert("id".to_owned(), Tag::Int(0));
        value.insert("element".to_owned(), Tag::Compound(overworld));

        dimension_type_value.insert("value".to_owned(), Tag::List(vec![Tag::Compound(value)]));
    }
    dimension_type.insert("value".to_owned(), Tag::Compound(dimension_type_value));
    root.insert(
        "minecraft:dimension_type".to_owned(),
        Tag::Compound(dimension_type),
    );

    // Create the biome registry
    let mut biome_registry = HashMap::new();
    biome_registry.insert(
        "type".to_owned(),
        Tag::String("minecraft:worldgen/biome".to_owned()),
    );

    let mut biome_value = HashMap::new();
    {
        let mut plains = HashMap::new();
        plains.insert("precipitation".to_owned(), Tag::String("rain".to_owned()));
        plains.insert("depth".to_owned(), Tag::Float(0.125));
        plains.insert("temperature".to_owned(), Tag::Float(0.8));
        plains.insert("scale".to_owned(), Tag::Float(0.05));
        plains.insert("downfall".to_owned(), Tag::Float(0.4));
        plains.insert("category".to_owned(), Tag::String("plains".to_owned()));

        let mut effects = HashMap::new();
        effects.insert("sky_color".to_owned(), Tag::Int(7907327));
        effects.insert("water_fog_color".to_owned(), Tag::Int(329011));
        effects.insert("fog_color".to_owned(), Tag::Int(12638463));
        effects.insert("water_color".to_owned(), Tag::Int(4159204));
        plains.insert("effects".to_owned(), Tag::Compound(effects));

        let mut value = HashMap::new();
        value.insert(
            "name".to_owned(),
            Tag::String("minecraft:plains".to_owned()),
        );
        value.insert("id".to_owned(), Tag::Int(0));
        value.insert("element".to_owned(), Tag::Compound(plains));

        biome_value.insert("value".to_owned(), Tag::List(vec![Tag::Compound(value)]));
    }
    biome_registry.insert("value".to_owned(), Tag::Compound(biome_value));
    root.insert(
        "minecraft:worldgen/biome".to_owned(),
        Tag::Compound(biome_registry),
    );

    Tag::Compound(root)
}

/// Constructs a default dimension NBT compound tag for the world being joined
fn default_dimension() -> Tag {
    let mut dimension = HashMap::new();
    dimension.insert("piglin_safe".to_owned(), Tag::Byte(0));
    dimension.insert("natural".to_owned(), Tag::Byte(1));
    dimension.insert("ambient_light".to_owned(), Tag::Float(0.0));
    dimension.insert(
        "infiniburn".to_owned(),
        Tag::String("minecraft:infiniburn_overworld".to_owned()),
    );
    dimension.insert("respawn_anchor_works".to_owned(), Tag::Byte(0));
    dimension.insert("has_skylight".to_owned(), Tag::Byte(1));
    dimension.insert("bed_works".to_owned(), Tag::Byte(1));
    dimension.insert(
        "effects".to_owned(),
        Tag::String("minecraft:overworld".to_owned()),
    );
    dimension.insert("has_raids".to_owned(), Tag::Byte(1));
    dimension.insert("logical_height".to_owned(), Tag::Int(256));
    dimension.insert("coordinate_scale".to_owned(), Tag::Float(1.0));
    dimension.insert("ultrawarm".to_owned(), Tag::Byte(0));
    dimension.insert("has_ceiling".to_owned(), Tag::Byte(0));
    Tag::Compound(dimension)
}
