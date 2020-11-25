use bytes::Bytes;
use itertools::Itertools;

use std::path::Path;

use crate::address::Address;
use crate::entry::Entry;
use crate::error::Error;
use crate::reader::*;

pub const INDEX_HEADER_SIZE: usize = 32;
pub const INDEX_HEADER_PADDING: i64 = 4 + 4 + 8 + (4*52) + ((4*2) + 4 + (4*5)*3 + 4 + 4 + 4 +(4*7));

#[derive(Debug)]
pub struct IndexHeader {
    pub version: String,
    pub num_entry: u32,
    pub num_byte: u32,
    pub last_file_created: String,
    pub table_len: u32,
}

pub struct IndexTable {
    pub table: Vec<Address>
}

impl IndexHeader {
    pub fn parse(input: &mut Bytes) -> crate::Result<Self> {
        if read_u32(input) != 0xC103CAC3 {
            return Err(Error::MagicError)
        }

        let version_raw = read_u32(input);
        let version = format!("{}.{}", version_raw >> 16, version_raw & 0x0000FFFF);

        let num_entry = read_u32(input);

        // Total size of the stored data
        let num_byte = read_u32(input);
        let last_file_created = format!("f_{:>06x}", read_u32(input));
        
        // Dirty flag and usage data adderss (each u32)
        let _ = input.split_to(8);

        let table_len = read_u32(input);

        // ignore the rest of header

        Ok(
            IndexHeader {
                version,
                num_entry,
                num_byte,
                last_file_created,
                table_len,
            }
        )
    }
}

impl IndexTable {
    pub fn parse(input: &mut Bytes, table_len: u32) -> crate::Result<IndexTable> {
        let result = try_count(Address::parse, table_len as usize)(input)?.into_iter().filter_map(|x| x).collect();
        Ok(IndexTable{table: result})
    }

    pub fn analyze<P: AsRef<Path>>(self, base_path: P) -> crate::Result<Vec<Entry>> {
        let result: Vec<Vec<Entry>> = self.table.into_iter().map(|addr| Entry::new(addr, base_path.as_ref())).try_collect()?;
        Ok(result.into_iter().flatten().collect())
    }
}