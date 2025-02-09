use std::io::{self};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

/// Packet trait. Contains the packet ID and the functions to write and read the packet.
pub trait Packet {
    /// Packet ID
    fn packet_id() -> i32
    where
        Self: Sized,
    {
        0x00
    }

    /// Reads the packet from the buffer. Default implementation is used for server-only packets, as
    /// they don't need to be read from the buffer.
    fn read_from_buffer(_buffer: &mut MinecraftPacketBuffer) -> io::Result<Self>
    where
        Self: Sized,
    {
        unimplemented!("Client-bound packets don't need read")
    }

    /// Writes the packet to the buffer. Default implementation is used for client-only packets, as
    /// they don't need to be written to the buffer.
    fn write_to_buffer(&self, _buffer: &mut MinecraftPacketBuffer) -> io::Result<()> {
        unimplemented!("Server-bound packets don't need write")
    }
}

/// Sends a packet to the client
pub async fn send_packet<T: Packet>(packet: T, socket: &mut TcpStream) -> io::Result<()> {
    let mut response_buffer = MinecraftPacketBuffer::new();
    packet.write_to_buffer(&mut response_buffer)?;

    let mut packet_with_length = MinecraftPacketBuffer::new();
    packet_with_length.write_varint(response_buffer.buffer.len() as i32);
    packet_with_length
        .buffer
        .extend_from_slice(&response_buffer.buffer);

    socket.write_all(&packet_with_length.buffer).await?;

    Ok(())
}

/// Minecraft packet buffer. Contains the buffer and the cursor.
/// The cursor is used to keep track of the current position in the buffer.
/// The buffer is used to store the packet data.
#[derive(Debug)]
pub struct MinecraftPacketBuffer {
    pub buffer: Vec<u8>,
    cursor: usize,
}

/// Minecraft packet buffer impl.
impl MinecraftPacketBuffer {
    /// Creates a new Minecraft packet buffer.
    /// The buffer is initialized with a capacity of 1024 bytes.
    /// The cursor is initialized to 0.
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            cursor: 0,
        }
    }

    /// Creates a new Minecraft packet buffer from a byte array.
    /// The buffer is initialized with the given byte array.
    /// The cursor is initialized to 0.
    /// The buffer is not copied, so the caller must ensure that the byte array is valid for
    /// the lifetime of the Minecraft packet buffer.
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            buffer: bytes,
            cursor: 0,
        }
    }

    /// Returns the current cursor position for the buffer.
    pub fn peek_byte(&self) -> Option<u8> {
        if self.cursor < self.buffer.len() {
            Some(self.buffer[self.cursor])
        } else {
            None
        }
    }

    /// Writes a VarInt to the buffer.
    /// A VarInt is a variable-length integer. It is encoded using 7 bits per byte, with the most
    /// significant bit of each byte set to 1 unless it is the final byte in the encoded
    /// representation.
    /// The VarInt is written to the buffer in network (big-endian) order.
    pub fn write_varint(&mut self, mut value: i32) {
        while (value & !0x7F) != 0 {
            self.buffer.push(((value & 0x7F) as u8) | 0x80);
            value >>= 7;
        }
        self.buffer.push((value & 0x7F) as u8);
    }

    /// Reads a VarInt from the buffer
    /// A VarInt is a variable-length integer. It is encoded using 7 bits per byte, with the most
    /// significant bit of each byte set to 1 unless it is the final byte in the encoded
    /// representation.
    /// The VarInt is read from the buffer in network (big-endian) order.
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

    /// Writes a u16 to the buffer.
    /// The u16 is written to the buffer in network (big-endian) order.
    pub fn write_string(&mut self, value: &str) {
        let bytes = value.as_bytes();
        self.write_varint(bytes.len() as i32);
        self.buffer.extend_from_slice(bytes);
    }

    /// Reads a string from the buffer.
    /// The string is read from the buffer in network (big-endian) order.
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

    /// Writes a UUID to the buffer.
    /// The UUID is written as two longs in big-endian order.
    pub fn write_uuid(&mut self, value: uuid::Uuid) {
        self.buffer.extend_from_slice(value.as_bytes());
    }

    /// Reads a UUID from the buffer.
    /// The UUID is read as two longs in big-endian order.
    pub fn read_uuid(&mut self) -> io::Result<uuid::Uuid> {
        if self.cursor + 16 > self.buffer.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Not enough bytes to read UUID",
            ));
        }
        let bytes = &self.buffer[self.cursor..self.cursor + 16];
        self.cursor += 16;
        Ok(uuid::Uuid::from_slice(bytes).unwrap())
    }

    // Write an u16 in network (big-endian) order.
    pub fn write_u16(&mut self, value: u16) {
        self.buffer.push((value >> 8) as u8);
        self.buffer.push((value & 0xFF) as u8);
    }

    // Read an u16 in network (big-endian) order.
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

    pub fn write_bool(&mut self, value: bool) {
        self.buffer.push(if value { 1 } else { 0 });
    }

    pub fn read_bool(&mut self) -> io::Result<bool> {
        if self.cursor >= self.buffer.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Not enough bytes to read bool",
            ));
        }
        let value = self.buffer[self.cursor] != 0;
        self.cursor += 1;
        Ok(value)
    }

    pub fn write_i8(&mut self, value: i8) {
        self.buffer.push(value as u8);
    }

    pub fn read_i8(&mut self) -> io::Result<i8> {
        if self.cursor >= self.buffer.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Not enough bytes to read i8",
            ));
        }
        let value = self.buffer[self.cursor] as i8;
        self.cursor += 1;
        Ok(value)
    }

    pub fn write_u8(&mut self, value: u8) {
        self.buffer.push(value);
    }

    pub fn read_u8(&mut self) -> io::Result<u8> {
        if self.cursor >= self.buffer.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Not enough bytes to read u8",
            ));
        }
        let value = self.buffer[self.cursor];
        self.cursor += 1;
        Ok(value)
    }

    pub fn write_i32(&mut self, value: i32) {
        self.buffer.extend_from_slice(&value.to_be_bytes());
    }

    pub fn write_i64(&mut self, value: i64) {
        self.buffer.extend_from_slice(&value.to_be_bytes());
    }
}

impl std::io::Write for MinecraftPacketBuffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
