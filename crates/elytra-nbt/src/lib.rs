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
            return Ok(("".to_owned(), Tag::End));
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_tag_type_ids() {
        assert_eq!(Tag::End.get_type_id(), 0);
        assert_eq!(Tag::Byte(0).get_type_id(), 1);
        assert_eq!(Tag::Short(0).get_type_id(), 2);
        assert_eq!(Tag::Int(0).get_type_id(), 3);
        assert_eq!(Tag::Long(0).get_type_id(), 4);
        assert_eq!(Tag::Float(0.0).get_type_id(), 5);
        assert_eq!(Tag::Double(0.0).get_type_id(), 6);
        assert_eq!(Tag::ByteArray(vec![]).get_type_id(), 7);
        assert_eq!(Tag::String("".to_string()).get_type_id(), 8);
        assert_eq!(Tag::List(vec![]).get_type_id(), 9);
        assert_eq!(Tag::Compound(HashMap::new()).get_type_id(), 10);
        assert_eq!(Tag::IntArray(vec![]).get_type_id(), 11);
        assert_eq!(Tag::LongArray(vec![]).get_type_id(), 12);
    }

    #[test]
    fn test_tag_as_methods() {
        // Test as_compound
        let mut map = HashMap::new();
        map.insert("test".to_string(), Tag::Int(42));
        let compound = Tag::Compound(map);
        assert!(compound.as_compound().is_some());
        assert_eq!(
            compound.as_compound().unwrap().get("test"),
            Some(&Tag::Int(42))
        );
        assert!(Tag::Int(0).as_compound().is_none());

        // Test as_list
        let list = Tag::List(vec![Tag::Int(1), Tag::Int(2)]);
        assert!(list.as_list().is_some());
        assert_eq!(list.as_list().unwrap().len(), 2);
        assert!(Tag::Int(0).as_list().is_none());

        // Test as_string
        let string = Tag::String("test".to_string());
        assert!(string.as_string().is_some());
        assert_eq!(string.as_string().unwrap(), "test");
        assert!(Tag::Int(0).as_string().is_none());

        // Test numeric conversions
        assert_eq!(Tag::Byte(42).as_i8(), Some(42));
        assert_eq!(Tag::Short(42).as_i16(), Some(42));
        assert_eq!(Tag::Int(42).as_i32(), Some(42));
        assert_eq!(Tag::Long(42).as_i64(), Some(42));
        assert_eq!(Tag::Float(42.0).as_f32(), Some(42.0));
        assert_eq!(Tag::Double(42.0).as_f64(), Some(42.0));
    }

    #[test]
    fn test_tag_read_write() {
        let test_cases = vec![
            (Tag::Byte(42), "byte"),
            (Tag::Short(1234), "short"),
            (Tag::Int(12345678), "int"),
            (Tag::Long(123456789012), "long"),
            (Tag::Float(3.14), "float"),
            (Tag::Double(3.14159), "double"),
            (Tag::ByteArray(vec![1, 2, 3]), "bytearray"),
            (Tag::String("Hello, World!".to_string()), "string"),
            (
                Tag::List(vec![Tag::Int(1), Tag::Int(2), Tag::Int(3)]),
                "list",
            ),
            (Tag::IntArray(vec![1, 2, 3]), "intarray"),
            (Tag::LongArray(vec![1, 2, 3]), "longarray"),
        ];

        for (tag, name) in test_cases {
            let mut buffer = Vec::new();
            tag.write(&mut buffer, name).unwrap();

            let mut cursor = Cursor::new(buffer);
            let (read_name, read_tag) = Tag::read(&mut cursor).unwrap();

            assert_eq!(read_name, name);
            assert_eq!(read_tag, tag);
        }
    }

    #[test]
    fn test_compound_tag_read_write() {
        let mut compound = HashMap::new();
        compound.insert("byte".to_string(), Tag::Byte(42));
        compound.insert("string".to_string(), Tag::String("test".to_string()));
        compound.insert(
            "list".to_string(),
            Tag::List(vec![Tag::Int(1), Tag::Int(2)]),
        );

        let tag = Tag::Compound(compound);

        let mut buffer = Vec::new();
        tag.write(&mut buffer, "root").unwrap();

        let mut cursor = Cursor::new(buffer);
        let (name, read_tag) = Tag::read(&mut cursor).unwrap();

        assert_eq!(name, "root");
        assert_eq!(read_tag, tag);
    }

    #[test]
    fn test_nbt_file() {
        let mut compound = HashMap::new();
        compound.insert("name".to_string(), Tag::String("Test".to_string()));
        compound.insert("value".to_string(), Tag::Int(42));

        let original = NBTFile::new("test".to_string(), Tag::Compound(compound));

        // Test regular write/read
        let mut buffer = Vec::new();
        original.write(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let read = NBTFile::read(&mut cursor).unwrap();

        assert_eq!(read.name, original.name);
        assert_eq!(read.root, original.root);

        // Test gzip write/read
        let mut gzip_buffer = Vec::new();
        original.write_gzip(&mut gzip_buffer).unwrap();

        let mut gzip_cursor = Cursor::new(gzip_buffer);
        let gzip_read = NBTFile::read_gzip(&mut gzip_cursor).unwrap();

        assert_eq!(gzip_read.name, original.name);
        assert_eq!(gzip_read.root, original.root);
    }

    #[test]
    fn test_invalid_tag_type() {
        let mut buffer = vec![255]; // Invalid tag type
        let result = Tag::read_payload(&mut Cursor::new(buffer), 255);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_list() {
        let tag = Tag::List(vec![]);
        let mut buffer = Vec::new();
        tag.write(&mut buffer, "empty").unwrap();

        let mut cursor = Cursor::new(buffer);
        let (name, read_tag) = Tag::read(&mut cursor).unwrap();

        assert_eq!(name, "empty");
        assert_eq!(read_tag, tag);
    }
}
