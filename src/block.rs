use crate::error::Result;
use crate::header::{Endianness, Header};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Seek};

#[derive(Debug, Clone)]
pub struct Block {
    pub code: [u8; 4],
    pub size: u32,
    pub old_memory_address: u64,
    pub sdna_index: u32,
    pub count: u32,
    pub data_offset: u64,
    pub data: Vec<u8>,
}

impl Block {
    pub fn from_reader<R: Read + Seek>(reader: &mut R, header: &Header) -> Result<Option<Self>> {
        let mut code = [0u8; 4];
        match reader.read_exact(&mut code) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e.into()),
        }

        // Check for DNA1 block which indicates end of file blocks
        if &code == b"DNA1" {
            return Ok(None);
        }

        // Read size
        let size = match header.endianness {
            Endianness::Little => reader.read_u32::<LittleEndian>()?,
            Endianness::Big => reader.read_u32::<byteorder::BigEndian>()?,
        };

        // Read old memory address
        let old_memory_address = match header.pointer_size {
            crate::header::PointerSize::Bits32 => {
                let addr = match header.endianness {
                    Endianness::Little => reader.read_u32::<LittleEndian>()?,
                    Endianness::Big => reader.read_u32::<byteorder::BigEndian>()?,
                };
                addr as u64
            }
            crate::header::PointerSize::Bits64 => match header.endianness {
                Endianness::Little => reader.read_u64::<LittleEndian>()?,
                Endianness::Big => reader.read_u64::<byteorder::BigEndian>()?,
            },
        };

        // Read SDNA index
        let sdna_index = match header.endianness {
            Endianness::Little => reader.read_u32::<LittleEndian>()?,
            Endianness::Big => reader.read_u32::<byteorder::BigEndian>()?,
        };

        // Read count
        let count = match header.endianness {
            Endianness::Little => reader.read_u32::<LittleEndian>()?,
            Endianness::Big => reader.read_u32::<byteorder::BigEndian>()?,
        };

        // Read the actual block data
        let mut data = vec![0u8; size as usize];
        reader.read_exact(&mut data)?;

        let data_offset = reader.stream_position()?;

        Ok(Some(Block {
            code,
            size,
            old_memory_address,
            sdna_index,
            count,
            data_offset,
            data,
        }))
    }

    pub fn is_library(&self) -> bool {
        &self.code[..2] == b"LI"
    }

    pub fn is_image(&self) -> bool {
        &self.code[..2] == b"IM"
    }

    pub fn is_sound(&self) -> bool {
        &self.code[..2] == b"SO"
    }

    pub fn is_movie_clip(&self) -> bool {
        &self.code[..2] == b"MC"
    }

    pub fn get_type_name(&self) -> String {
        String::from_utf8_lossy(&self.code).into_owned()
    }

    pub fn get_string_field(&self, field_name: &str) -> Result<String> {
        // This is a simplified implementation - in a real scenario, you'd use DNA info
        // For now, we'll search for null-terminated strings in the data
        let null_pos = self
            .data
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(self.data.len());
        let string_data = &self.data[..null_pos];
        Ok(String::from_utf8_lossy(string_data).into_owned())
    }

    pub fn set_string_field(&mut self, field_name: &str, value: &str) -> Result<()> {
        // This is a simplified implementation - in a real scenario, you'd use DNA info
        let bytes = value.as_bytes();
        let len = bytes.len().min(self.data.len());

        // Copy the string bytes
        self.data[..len].copy_from_slice(&bytes[..len]);

        // Null-terminate if there's space
        if len < self.data.len() {
            self.data[len] = 0;
        }

        Ok(())
    }

    pub fn get_float_array_field(&self, field_name: &str, count: usize) -> Result<Vec<f32>> {
        // This is a simplified implementation - in a real scenario, you'd use DNA info
        let mut result = Vec::with_capacity(count);

        for i in 0..count {
            if i * 4 + 4 <= self.data.len() {
                let bytes = &self.data[i * 4..i * 4 + 4];
                let value = f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                result.push(value);
            } else {
                result.push(0.0);
            }
        }

        Ok(result)
    }

    pub fn set_float_array_field(&mut self, field_name: &str, values: &[f32]) -> Result<()> {
        // This is a simplified implementation - in a real scenario, you'd use DNA info
        for (i, &value) in values.iter().enumerate() {
            let start = i * 4;
            let end = start + 4;

            if end <= self.data.len() {
                let bytes = value.to_le_bytes();
                self.data[start..end].copy_from_slice(&bytes);
            }
        }

        Ok(())
    }

    pub fn write_to_writer<W: std::io::Write>(
        &self,
        writer: &mut W,
        header: &crate::header::Header,
    ) -> Result<()> {
        use byteorder::{LittleEndian, WriteBytesExt};

        // Write block code
        writer.write_all(&self.code)?;

        // Write size
        match header.endianness {
            Endianness::Little => writer.write_u32::<LittleEndian>(self.size)?,
            Endianness::Big => writer.write_u32::<byteorder::BigEndian>(self.size)?,
        }

        // Write old memory address
        match header.pointer_size {
            crate::header::PointerSize::Bits32 => {
                let addr = self.old_memory_address as u32;
                match header.endianness {
                    Endianness::Little => writer.write_u32::<LittleEndian>(addr)?,
                    Endianness::Big => writer.write_u32::<byteorder::BigEndian>(addr)?,
                }
            }
            crate::header::PointerSize::Bits64 => match header.endianness {
                Endianness::Little => writer.write_u64::<LittleEndian>(self.old_memory_address)?,
                Endianness::Big => {
                    writer.write_u64::<byteorder::BigEndian>(self.old_memory_address)?
                }
            },
        }

        // Write SDNA index
        match header.endianness {
            Endianness::Little => writer.write_u32::<LittleEndian>(self.sdna_index)?,
            Endianness::Big => writer.write_u32::<byteorder::BigEndian>(self.sdna_index)?,
        }

        // Write count
        match header.endianness {
            Endianness::Little => writer.write_u32::<LittleEndian>(self.count)?,
            Endianness::Big => writer.write_u32::<byteorder::BigEndian>(self.count)?,
        }

        // Write data
        writer.write_all(&self.data)?;

        Ok(())
    }
}

pub struct BlockIterator<'a, R: Read + Seek> {
    reader: &'a mut R,
    header: &'a Header,
    finished: bool,
}

impl<'a, R: Read + Seek> BlockIterator<'a, R> {
    pub fn new(reader: &'a mut R, header: &'a Header) -> Self {
        BlockIterator {
            reader,
            header,
            finished: false,
        }
    }
}

impl<'a, R: Read + Seek> Iterator for BlockIterator<'a, R> {
    type Item = Result<Block>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        match Block::from_reader(self.reader, self.header) {
            Ok(Some(block)) => Some(Ok(block)),
            Ok(None) => {
                self.finished = true;
                None
            }
            Err(e) => {
                self.finished = true;
                Some(Err(e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_block_creation() {
        let mut data = vec![
            b'L', b'I', b'\0', b'\0', // code
            100, 0, 0, 0, // size (little-endian)
            0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // address (64-bit)
            0, 0, 0, 0, // sdna_index
            1, 0, 0, 0, // count
        ];
        data.extend(vec![0u8; 100]); // Add 100 bytes of data

        let header = crate::header::Header {
            magic: *b"BLENDER",
            pointer_size: crate::header::PointerSize::Bits64,
            endianness: Endianness::Little,
            version: 279,
        };

        let mut cursor = Cursor::new(data);
        let block = Block::from_reader(&mut cursor, &header).unwrap().unwrap();

        assert_eq!(block.code, *b"LI\0\0");
        assert_eq!(block.size, 100);
        assert_eq!(block.old_memory_address, 0x10);
        assert_eq!(block.sdna_index, 0);
        assert_eq!(block.count, 1);
        assert_eq!(block.data.len(), 100);
    }

    #[test]
    fn test_block_type_checking() {
        let block = Block {
            code: *b"LI\0\0",
            size: 100,
            old_memory_address: 0x1000,
            sdna_index: 0,
            count: 1,
            data_offset: 100,
            data: vec![0; 100],
        };

        assert!(block.is_library());
        assert!(!block.is_image());
        assert!(!block.is_sound());
        assert!(!block.is_movie_clip());

        let block = Block {
            code: *b"IM\0\0",
            size: 200,
            old_memory_address: 0x2000,
            sdna_index: 1,
            count: 1,
            data_offset: 200,
            data: vec![0; 200],
        };

        assert!(!block.is_library());
        assert!(block.is_image());
        assert!(!block.is_sound());
        assert!(!block.is_movie_clip());
    }
}
