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
    pub buffer: Vec<u8>,
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

    pub fn peek_byte(&self) -> Option<u8> {
        if self.cursor < self.buffer.len() {
            Some(self.buffer[self.cursor])
        } else {
            None
        }
    }

    pub fn write_varint(&mut self, mut value: i32) {
        while (value & !0x7F) != 0 {
            self.buffer.push(((value & 0x7F) as u8) | 0x80);
            value >>= 7;
        }
        self.buffer.push((value & 0x7F) as u8);
    }
    pub fn read_varint(&mut self) -> io::Result<i32> {
        let mut result = 0;
        let mut shift = 0;

        loop {
            if self.cursor >= self.buffer.len() {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "EOF while reading VarInt",
                ));
            }

            let byte = self.buffer[self.cursor];
            self.cursor += 1;

            result |= ((byte & 0x7F) as i32) << shift;
            shift += 7;

            if (byte & 0x80) == 0 {
                break;
            }

            if shift >= 32 {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "VarInt too big"));
            }
        }

        Ok(result)
    }
    pub fn write_string(&mut self, value: &str) {
        let bytes = value.as_bytes();
        self.write_varint(bytes.len() as i32);
        self.buffer.extend_from_slice(bytes);
    }

    pub fn read_string(&mut self) -> io::Result<String> {
        let length = self.read_varint()? as usize;
        if self.cursor + length > self.buffer.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Not enough bytes to read the full string",
            ));
        }
        let bytes = &self.buffer[self.cursor..self.cursor + length];
        self.cursor += length;
        String::from_utf8(bytes.to_vec()).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Failed to convert bytes to UTF-8 string",
            )
        })
    }

    // Write a u16 in network (big-endian) order.
    pub fn write_u16(&mut self, value: u16) {
        self.buffer.push((value >> 8) as u8);
        self.buffer.push((value & 0xFF) as u8);
    }

    // Read a u16 in network (big-endian) order.
    pub fn read_u16(&mut self) -> io::Result<u16> {
        if self.cursor + 2 > self.buffer.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Not enough bytes to read u16",
            ));
        }
        let hi = self.buffer[self.cursor] as u16;
        let lo = self.buffer[self.cursor + 1] as u16;
        self.cursor += 2;
        Ok((hi << 8) | lo)
    }
}
