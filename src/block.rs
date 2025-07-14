use crate::error::Result;
use crate::header::{Endianness, Header};
use byteorder::{LittleEndian, ReadBytesExt};
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
