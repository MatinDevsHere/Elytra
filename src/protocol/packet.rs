use std::io::{self};

pub trait Packet {
    fn packet_id() -> i32 {
        0x00
    }
    fn write(&self, _buffer: &mut MinecraftPacketBuffer) -> io::Result<()> {
        unimplemented!("Server-bound packets don't need write")
    }
    fn read(_buffer: &mut MinecraftPacketBuffer) -> io::Result<Self>
    where
        Self: Sized,
    {
        unimplemented!("Client-bound packets don't need read")
    }
}

#[derive(Debug)]
pub struct MinecraftPacketBuffer {
    pub buffer: Vec<u8>, // Made public
    cursor: usize,
}

impl MinecraftPacketBuffer {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            cursor: 0,
        }
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            buffer: bytes,
            cursor: 0,
        }
    }

    pub fn write_varint(&mut self, mut value: i32) {
        loop {
            let mut temp = (value & 0b0111_1111) as u8;
            value = (value >> 7) & (i32::max_value() >> 6);
            if value != 0 {
                temp |= 0b1000_0000;
            }
            self.buffer.push(temp);
            if value == 0 {
                break;
            }
        }
    }

    pub fn read_varint(&mut self) -> io::Result<i32> {
        let mut result = 0;
        let mut shift = 0;

        loop {
            let byte = self.buffer[self.cursor];
            self.cursor += 1;

            result |= ((byte & 0b0111_1111) as i32) << shift;
            if byte & 0b1000_0000 == 0 {
                break;
            }
            shift += 7;

            if shift >= 32 {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "VarInt too big"));
            }
        }

        Ok(result)
    }

    pub fn write_string(&mut self, value: &str) {
        self.write_varint(value.len() as i32);
        self.buffer.extend_from_slice(value.as_bytes());
    }

    pub fn read_string(&mut self) -> io::Result<String> {
        let length = self.read_varint()? as usize;
        // For Minecraft protocol, empty strings are valid
        if length == 0 {
            return Ok(String::new());
        }

        // Make sure we don't read past buffer
        if self.cursor + length > self.buffer.len() {
            return Ok(String::new()); // Protocol allows empty string fallback
        }

        let string_bytes = &self.buffer[self.cursor..self.cursor + length];
        self.cursor += length;

        // Minecraft protocol allows invalid UTF-8 in some cases
        Ok(String::from_utf8(string_bytes.to_vec()).unwrap_or_else(|_| String::new()))
    }
}
