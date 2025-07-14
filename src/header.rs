use crate::error::{BlendFileError, Result};
use std::io::{Read, Seek};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PointerSize {
    Bits32,
    Bits64,
}

impl PointerSize {
    pub fn bytes(&self) -> usize {
        match self {
            PointerSize::Bits32 => 4,
            PointerSize::Bits64 => 8,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Endianness {
    Little,
    Big,
}

#[derive(Debug, Clone)]
pub struct Header {
    pub magic: [u8; 7],
    pub pointer_size: PointerSize,
    pub endianness: Endianness,
    pub version: u32,
}

impl Header {
    pub fn from_reader<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        let mut magic = [0u8; 7];
        reader.read_exact(&mut magic)?;

        if &magic != b"BLENDER" {
            return Err(BlendFileError::InvalidFormat(format!(
                "Invalid magic: {:?}",
                String::from_utf8_lossy(&magic)
            )));
        }

        let mut pointer_size_byte = [0u8; 1];
        reader.read_exact(&mut pointer_size_byte)?;

        let pointer_size = match pointer_size_byte[0] {
            b'_' => PointerSize::Bits32,
            b'-' => PointerSize::Bits64,
            _ => {
                return Err(BlendFileError::InvalidFormat(format!(
                    "Invalid pointer size indicator: {}",
                    pointer_size_byte[0] as char
                )))
            }
        };

        let mut endianness_byte = [0u8; 1];
        reader.read_exact(&mut endianness_byte)?;

        let endianness = match endianness_byte[0] {
            b'v' => Endianness::Little,
            b'V' => Endianness::Big,
            _ => {
                return Err(BlendFileError::InvalidFormat(format!(
                    "Invalid endianness indicator: {}",
                    endianness_byte[0] as char
                )))
            }
        };

        let mut version_bytes = [0u8; 3];
        reader.read_exact(&mut version_bytes)?;

        let version_str = String::from_utf8_lossy(&version_bytes);
        let version = version_str.parse::<u32>().map_err(|_| {
            BlendFileError::InvalidFormat(format!("Invalid version: {version_str}"))
        })?;

        Ok(Header {
            magic,
            pointer_size,
            endianness,
            version,
        })
    }

    pub fn write_to_writer<W: std::io::Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.magic)?;

        let pointer_size_byte = match self.pointer_size {
            PointerSize::Bits32 => b'_',
            PointerSize::Bits64 => b'-',
        };
        writer.write_all(&[pointer_size_byte])?;

        let endianness_byte = match self.endianness {
            Endianness::Little => b'v',
            Endianness::Big => b'V',
        };
        writer.write_all(&[endianness_byte])?;

        let version_str = format!("{:03}", self.version);
        writer.write_all(version_str.as_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_valid_header() {
        let data = b"BLENDER-v279";
        let mut cursor = Cursor::new(data);
        let header = Header::from_reader(&mut cursor).unwrap();

        assert_eq!(header.magic, *b"BLENDER");
        assert_eq!(header.pointer_size, PointerSize::Bits64);
        assert_eq!(header.endianness, Endianness::Little);
        assert_eq!(header.version, 279);
    }

    #[test]
    fn test_invalid_magic() {
        let data = b"INVALID-v279";
        let mut cursor = Cursor::new(data);
        let result = Header::from_reader(&mut cursor);

        assert!(result.is_err());
    }

    #[test]
    fn test_32bit_header() {
        let data = b"BLENDER_v279";
        let mut cursor = Cursor::new(data);
        let header = Header::from_reader(&mut cursor).unwrap();

        assert_eq!(header.pointer_size, PointerSize::Bits32);
    }

    #[test]
    fn test_big_endian_header() {
        let data = b"BLENDER-V279";
        let mut cursor = Cursor::new(data);
        let header = Header::from_reader(&mut cursor).unwrap();

        assert_eq!(header.endianness, Endianness::Big);
    }
}
