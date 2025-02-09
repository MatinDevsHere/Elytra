use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::collections::HashMap;
use std::io::{self, Read, Write};

#[derive(Debug, Clone, PartialEq)]
pub enum Tag {
    End,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<i8>),
    String(String),
    List(Vec<Tag>),
    Compound(HashMap<String, Tag>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl Tag {
    pub fn get_type_id(&self) -> u8 {
        match self {
            Tag::End => 0,
            Tag::Byte(_) => 1,
            Tag::Short(_) => 2,
            Tag::Int(_) => 3,
            Tag::Long(_) => 4,
            Tag::Float(_) => 5,
            Tag::Double(_) => 6,
            Tag::ByteArray(_) => 7,
            Tag::String(_) => 8,
            Tag::List(_) => 9,
            Tag::Compound(_) => 10,
            Tag::IntArray(_) => 11,
            Tag::LongArray(_) => 12,
        }
    }

    pub fn read<R: Read>(reader: &mut R) -> io::Result<(String, Tag)> {
        let type_id = reader.read_u8()?;
        if type_id == 0 {
            return Ok(("".to_string(), Tag::End));
        }

        let name_length = reader.read_u16::<BigEndian>()?;
        let mut name_bytes = vec![0u8; name_length as usize];
        reader.read_exact(&mut name_bytes)?;
        let name = String::from_utf8(name_bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let tag = Tag::read_payload(reader, type_id)?;
        Ok((name, tag))
    }

    fn read_payload<R: Read>(reader: &mut R, type_id: u8) -> io::Result<Tag> {
        match type_id {
            0 => Ok(Tag::End),
            1 => Ok(Tag::Byte(reader.read_i8()?)),
            2 => Ok(Tag::Short(reader.read_i16::<BigEndian>()?)),
            3 => Ok(Tag::Int(reader.read_i32::<BigEndian>()?)),
            4 => Ok(Tag::Long(reader.read_i64::<BigEndian>()?)),
            5 => Ok(Tag::Float(reader.read_f32::<BigEndian>()?)),
            6 => Ok(Tag::Double(reader.read_f64::<BigEndian>()?)),
            7 => {
                let length = reader.read_i32::<BigEndian>()?;
                let mut bytes = Vec::with_capacity(length as usize);
                for _ in 0..length {
                    bytes.push(reader.read_i8()?);
                }
                Ok(Tag::ByteArray(bytes))
            }
            8 => {
                let length = reader.read_u16::<BigEndian>()?;
                let mut bytes = vec![0u8; length as usize];
                reader.read_exact(&mut bytes)?;
                String::from_utf8(bytes)
                    .map(Tag::String)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
            }
            9 => {
                let list_type = reader.read_u8()?;
                let length = reader.read_i32::<BigEndian>()?;
                let mut list = Vec::with_capacity(length as usize);
                for _ in 0..length {
                    list.push(Tag::read_payload(reader, list_type)?);
                }
                Ok(Tag::List(list))
            }
            10 => {
                let mut compound = HashMap::new();
                loop {
                    let (name, tag) = Tag::read(reader)?;
                    if let Tag::End = tag {
                        break;
                    }
                    compound.insert(name, tag);
                }
                Ok(Tag::Compound(compound))
            }
            11 => {
                let length = reader.read_i32::<BigEndian>()?;
                let mut ints = Vec::with_capacity(length as usize);
                for _ in 0..length {
                    ints.push(reader.read_i32::<BigEndian>()?);
                }
                Ok(Tag::IntArray(ints))
            }
            12 => {
                let length = reader.read_i32::<BigEndian>()?;
                let mut longs = Vec::with_capacity(length as usize);
                for _ in 0..length {
                    longs.push(reader.read_i64::<BigEndian>()?);
                }
                Ok(Tag::LongArray(longs))
            }
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid tag type: {}", type_id),
            )),
        }
    }

    pub fn write<W: Write>(&self, writer: &mut W, name: &str) -> io::Result<()> {
        writer.write_u8(self.get_type_id())?;

        if !matches!(self, Tag::End) {
            writer.write_u16::<BigEndian>(name.len() as u16)?;
            writer.write_all(name.as_bytes())?;
        }

        self.write_payload(writer)
    }

    fn write_payload<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            Tag::End => Ok(()),
            Tag::Byte(v) => writer.write_i8(*v),
            Tag::Short(v) => writer.write_i16::<BigEndian>(*v),
            Tag::Int(v) => writer.write_i32::<BigEndian>(*v),
            Tag::Long(v) => writer.write_i64::<BigEndian>(*v),
            Tag::Float(v) => writer.write_f32::<BigEndian>(*v),
            Tag::Double(v) => writer.write_f64::<BigEndian>(*v),
            Tag::ByteArray(v) => {
                writer.write_i32::<BigEndian>(v.len() as i32)?;
                for &b in v {
                    writer.write_i8(b)?;
                }
                Ok(())
            }
            Tag::String(v) => {
                writer.write_u16::<BigEndian>(v.len() as u16)?;
                writer.write_all(v.as_bytes())
            }
            Tag::List(v) => {
                if v.is_empty() {
                    writer.write_u8(0)?; // TAG_End for empty lists
                } else {
                    writer.write_u8(v[0].get_type_id())?;
                }
                writer.write_i32::<BigEndian>(v.len() as i32)?;
                for tag in v {
                    tag.write_payload(writer)?;
                }
                Ok(())
            }
            Tag::Compound(v) => {
                for (name, tag) in v {
                    tag.write(writer, name)?;
                }
                Tag::End.write(writer, "")?;
                Ok(())
            }
            Tag::IntArray(v) => {
                writer.write_i32::<BigEndian>(v.len() as i32)?;
                for &i in v {
                    writer.write_i32::<BigEndian>(i)?;
                }
                Ok(())
            }
            Tag::LongArray(v) => {
                writer.write_i32::<BigEndian>(v.len() as i32)?;
                for &l in v {
                    writer.write_i64::<BigEndian>(l)?;
                }
                Ok(())
            }
        }
    }

    pub fn as_compound(&self) -> Option<&HashMap<String, Tag>> {
        match self {
            Tag::Compound(map) => Some(map),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&Vec<Tag>> {
        match self {
            Tag::List(list) => Some(list),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&String> {
        match self {
            Tag::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Tag::Long(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_i32(&self) -> Option<i32> {
        match self {
            Tag::Int(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_i16(&self) -> Option<i16> {
        match self {
            Tag::Short(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_i8(&self) -> Option<i8> {
        match self {
            Tag::Byte(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Tag::Double(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_f32(&self) -> Option<f32> {
        match self {
            Tag::Float(n) => Some(*n),
            _ => None,
        }
    }
}

// NBTFile represents a complete NBT file with compression support
pub struct NBTFile {
    pub root: Tag,
    pub name: String,
}

impl NBTFile {
    pub fn new(name: String, root: Tag) -> Self {
        NBTFile { root, name }
    }

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let (name, root) = Tag::read(reader)?;
        Ok(NBTFile { root, name })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.root.write(writer, &self.name)
    }

    pub fn read_gzip<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut decoder = GzDecoder::new(reader);
        Self::read(&mut decoder)
    }

    pub fn write_gzip<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let mut encoder = GzEncoder::new(writer, Compression::default());
        self.write(&mut encoder)?;
        encoder.finish()?;
        Ok(())
    }
}
