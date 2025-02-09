use super::nbt::Tag;

pub struct JoinGamePacket {
    pub entity_id: i32,
    pub is_hardcore: bool,
    pub gamemode: u8,
    pub previous_gamemode: i8,    // Byte, -1 if no previous gamemode
    pub world_count: i32,         // VarInt
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
