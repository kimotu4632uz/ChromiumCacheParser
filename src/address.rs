use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use bytes::Bytes;

use crate::error::AddressErrorKind;
use crate::reader::read_u32;

#[derive(FromPrimitive, PartialEq, Debug)]
pub enum BlockType {
    SeparateFile,
    RankingBlock,
    Block256,
    Block1024,
    Block4096
}

impl BlockType {
    pub fn get_size(&self) -> usize {
        match *self {
            Self::SeparateFile => 0,
            Self::RankingBlock => 36,
            Self::Block256 => 256,
            Self::Block1024 => 1024,
            Self::Block4096 => 4096,
        }
    }
}

//impl fmt::Display for BlockType {
//    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//        match *self {
//            Self::SeparateFile => write!(f, "SeparateFile"),
//            Self::RankingBlock => write!(f, "RankingBlock"),
//            Self::Block256 => write!(f, "Block256"),
//            Self::Block1024 => write!(f, "Block1024"),
//            Self::Block4096 => write!(f, "Block4096"),
//        }
//    }
//}

#[derive(Debug, PartialEq)]
pub struct Address {
    pub addr: u32,
    pub blocktype: BlockType,
    pub file_name: String,
    pub contiguous_block: u32,
    pub block_number: u32,
}

impl Address {
    fn new(addr: u32, blocktype: BlockType, file_name: String, contiguous_block: u32 , block_number: u32) -> Self {
        Address {addr, blocktype, file_name, contiguous_block, block_number}
    }

    pub fn parse(input: &mut Bytes) -> crate::Result<Option<Self>> {
        let addr = read_u32(input);

        if addr == 0 {
            Ok(None)
        } else if addr < 2^31 {
            Err(AddressErrorKind::UnInitialized.into())
        } else {
            let blocktype = FromPrimitive::from_u32((addr & 0x70000000) >> 28);
        
            match blocktype {
                Some(BlockType::SeparateFile) => {
                    let file_name = format!("f_{:>06x}", (addr & 0x0FFFFFFF));

                    Ok(
                        Some(Address::new(addr, blocktype.unwrap(), file_name, 0,0)),
                    )
                },

                Some(BlockType::RankingBlock) => {
                    let file_name = format!("data_{}", (addr & 0x00FF0000) >> 16);

                    Ok(
                        Some(Address::new(addr, blocktype.unwrap(), file_name, 0,0)),
                    )
                },

                Some(BlockType::Block256) | Some(BlockType::Block1024) | Some(BlockType::Block4096) => {
                    let contiguous_block = (addr & 0x03000000) >> 24;
                    let file_name = format!("data_{}", (addr & 0x00FF0000) >> 16);
                    let block_number = addr & 0x0000FFFF;

                    Ok(
                        Some(Address::new(addr, blocktype.unwrap(), file_name, contiguous_block, block_number)),
                    )
                },

                None => {
                    Err(AddressErrorKind::InvalidBlockType.into())
                }
            }
        }
    }

    pub fn parse_null_to_error(input: &mut Bytes) -> crate::Result<Self> {
        Address::parse(input).and_then(|x| x.ok_or(AddressErrorKind::NullAddress.into()))
    }
}

