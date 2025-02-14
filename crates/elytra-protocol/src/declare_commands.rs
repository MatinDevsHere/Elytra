use crate::packet::MinecraftPacketBuffer;
use crate::packet::Packet;
use std::io;

// Node type flags
const NODE_TYPE_ROOT: u8 = 0;
const NODE_TYPE_LITERAL: u8 = 1;
const NODE_TYPE_ARGUMENT: u8 = 2;

// Flag masks
#[allow(dead_code)]
const FLAG_NODE_TYPE: u8 = 0x03;
const FLAG_EXECUTABLE: u8 = 0x04;
const FLAG_REDIRECT: u8 = 0x08;
const FLAG_SUGGESTIONS: u8 = 0x10;

#[derive(Debug, Clone)]
pub enum NodeType {
    Root,
    Literal { name: String },
    Argument { name: String, parser: Parser },
}

#[derive(Debug, Clone)]
pub struct CommandNode {
    node_type: NodeType,
    children: Vec<i32>,
    redirect_node: Option<i32>,
    is_executable: bool,
    suggestions_type: Option<String>,
}

impl CommandNode {
    pub fn new_root() -> Self {
        Self {
            node_type: NodeType::Root,
            children: Vec::new(),
            redirect_node: None,
            is_executable: false,
            suggestions_type: None,
        }
    }

    pub fn new_literal(name: impl Into<String>, is_executable: bool) -> Self {
        Self {
            node_type: NodeType::Literal { name: name.into() },
            children: Vec::new(),
            redirect_node: None,
            is_executable,
            suggestions_type: None,
        }
    }

    pub fn new_argument(name: impl Into<String>, parser: Parser, is_executable: bool) -> Self {
        Self {
            node_type: NodeType::Argument {
                name: name.into(),
                parser,
            },
            children: Vec::new(),
            redirect_node: None,
            is_executable,
            suggestions_type: None,
        }
    }

    pub fn add_child(&mut self, child_index: i32) {
        self.children.push(child_index);
    }

    pub fn set_redirect(&mut self, redirect_index: i32) {
        self.redirect_node = Some(redirect_index);
    }

    pub fn set_suggestions(&mut self, suggestions_type: impl Into<String>) {
        self.suggestions_type = Some(suggestions_type.into());
    }
}

#[derive(Debug, Clone)]
pub enum Parser {
    Bool,
    Double { min: Option<f64>, max: Option<f64> },
    Float { min: Option<f32>, max: Option<f32> },
    Integer { min: Option<i32>, max: Option<i32> },
    Long { min: Option<i64>, max: Option<i64> },
    String(StringType),
    Entity { single: bool, only_players: bool },
    GameProfile,
    BlockPos,
    ColumnPos,
    Vec3,
    Vec2,
    BlockState,
    BlockPredicate,
    ItemStack,
    ItemPredicate,
    Color,
    Component,
    Message,
    Nbt,
    NbtPath,
    Objective,
    ObjectiveCriteria,
    Operation,
    Particle,
    Rotation,
    Angle,
    ScoreboardSlot,
    ScoreHolder { allow_multiple: bool },
    Swizzle,
    Team,
    ItemSlot,
    ResourceLocation,
    MobEffect,
    Function,
    EntityAnchor,
    Range { allow_decimals: bool },
    IntRange,
    FloatRange,
    ItemEnchantment,
    EntitySummon,
    Dimension,
    Uuid,
    NbtTag,
    NbtCompoundTag,
    Time,
}

#[derive(Debug, Clone)]
pub enum StringType {
    SingleWord,
    QuotablePhrase,
    GreedyPhrase,
}

pub struct DeclareCommandsPacket {
    nodes: Vec<CommandNode>,
    root_index: i32,
}

impl DeclareCommandsPacket {
    pub fn new() -> Self {
        Self {
            nodes: vec![CommandNode::new_root()],
            root_index: 0,
        }
    }

    pub fn add_node(&mut self, node: CommandNode) -> i32 {
        let index = self.nodes.len() as i32;
        self.nodes.push(node);
        index
    }

    pub fn get_node_mut(&mut self, index: i32) -> Option<&mut CommandNode> {
        self.nodes.get_mut(index as usize)
    }

    pub fn get_root_mut(&mut self) -> &mut CommandNode {
        &mut self.nodes[self.root_index as usize]
    }
}

impl Parser {
    fn write(&self, buffer: &mut MinecraftPacketBuffer) -> io::Result<()> {
        match self {
            Parser::Bool => buffer.write_string("brigadier:bool"),
            Parser::Double { min, max } => {
                buffer.write_string("brigadier:double");
                let mut flags: u8 = 0;
                if min.is_some() {
                    flags |= 0x01;
                }
                if max.is_some() {
                    flags |= 0x02;
                }
                buffer.write_u8(flags);
                if let Some(min) = min {
                    buffer.write_i64(*min as i64);
                }
                if let Some(max) = max {
                    buffer.write_i64(*max as i64);
                }
            }
            Parser::Float { min, max } => {
                buffer.write_string("brigadier:float");
                let mut flags: u8 = 0;
                if min.is_some() {
                    flags |= 0x01;
                }
                if max.is_some() {
                    flags |= 0x02;
                }
                buffer.write_u8(flags);
                if let Some(min) = min {
                    buffer.write_i32(*min as i32);
                }
                if let Some(max) = max {
                    buffer.write_i32(*max as i32);
                }
            }
            Parser::String(string_type) => {
                buffer.write_string("brigadier:string");
                buffer.write_varint(match string_type {
                    StringType::SingleWord => 0,
                    StringType::QuotablePhrase => 1,
                    StringType::GreedyPhrase => 2,
                });
            }
            Parser::Entity {
                single,
                only_players,
            } => {
                buffer.write_string("minecraft:entity");
                let mut flags: u8 = 0;
                if *single {
                    flags |= 0x01;
                }
                if *only_players {
                    flags |= 0x02;
                }
                buffer.write_u8(flags);
            }
            // Add other parser implementations as needed
            _ => buffer.write_string("minecraft:entity"), // Default case
        }
        Ok(())
    }
}

impl Packet for DeclareCommandsPacket {
    fn packet_id() -> i32 {
        0x10
    }

    fn write_to_buffer(&self, buffer: &mut MinecraftPacketBuffer) -> io::Result<()> {
        buffer.write_varint(Self::packet_id());

        // Write number of nodes
        buffer.write_varint(self.nodes.len() as i32);

        // Write each node
        for node in &self.nodes {
            // Calculate flags
            let mut flags = match node.node_type {
                NodeType::Root => NODE_TYPE_ROOT,
                NodeType::Literal { .. } => NODE_TYPE_LITERAL,
                NodeType::Argument { .. } => NODE_TYPE_ARGUMENT,
            };
            if node.is_executable {
                flags |= FLAG_EXECUTABLE;
            }
            if node.redirect_node.is_some() {
                flags |= FLAG_REDIRECT;
            }
            if node.suggestions_type.is_some() {
                flags |= FLAG_SUGGESTIONS;
            }
            buffer.write_u8(flags);

            // Write children count and indices
            buffer.write_varint(node.children.len() as i32);
            for child in &node.children {
                buffer.write_varint(*child);
            }

            // Write redirect node if present
            if let Some(redirect) = node.redirect_node {
                buffer.write_varint(redirect);
            }

            // Write name for literal and argument nodes
            match &node.node_type {
                NodeType::Root => {}
                NodeType::Literal { name } | NodeType::Argument { name, .. } => {
                    buffer.write_string(name);
                }
            }

            // Write parser info for argument nodes
            if let NodeType::Argument { parser, .. } = &node.node_type {
                parser.write(buffer)?;
            }

            // Write suggestions type if present
            if let Some(suggestions) = &node.suggestions_type {
                buffer.write_string(suggestions);
            }
        }

        // Write root node index
        buffer.write_varint(self.root_index);

        Ok(())
    }
}
