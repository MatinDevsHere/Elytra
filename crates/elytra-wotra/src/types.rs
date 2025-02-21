use serde::{Deserialize, Serialize};

// TODO: What are these?

pub type Result<T> = std::result::Result<T, crate::error::ElytraError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rotation {
    pub yaw: f32,
    pub pitch: f32,
}
