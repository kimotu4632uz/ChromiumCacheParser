use chrono::prelude::*;
use bytes::Bytes;

use std::io::{Seek, SeekFrom};
use std::fs::File;
use std::path::Path;

use crate::address::Address;
use crate::data::HttpHeader;
use crate::error::Error;
use crate::reader::*;

#[derive(Debug, PartialEq)]
pub enum Key {
    LocalKey(String),
    DataKey(Address, u32),
}

impl Key {
    pub const fn is_local_key(&self) -> bool {
        match self {
            Key::LocalKey(_) => true,
            _ => false,
        }
    }

    pub const fn is_datakey(&self) -> bool {
        !self.is_local_key()
    }
}

#[derive(Debug)]
pub struct Entry {
    pub hash: u32,
    pub ranking_node: u32,
    pub usage_count: u32,
    pub reuse_count: u32,
    pub state: u32,
    pub creation_time: DateTime<Utc>,
    pub flags: u32,
    pub key: Key,
    pub data: Vec<(Address, u32)>,
    pub header: Option<HttpHeader>,
}

impl Entry {
    pub fn parse(input: &mut Bytes) -> crate::Result<(Self, Option<Address>)> {
        let hash = read_u32(input);
        let next = Address::parse(input)?;
        let ranking_node = read_u32(input);
        let usage_count = read_u32(input);
        let reuse_count = read_u32(input);
        let state = read_u32(input);

        let creation_time = Utc.ymd(1601, 1, 1).and_hms(0,0,0) + chrono::Duration::microseconds(read_i64(input));

        let key_length = read_u32(input);
        let kay_address = Address::parse(input)?;

        let data_size = count(read_u32, 4usize)(input);
        let addrs = try_count(Address::parse, 4)(input)?;

        let data = addrs.into_iter().zip(data_size.into_iter()).filter_map(|(x, y)| x.map(|a| (a,y)) ).collect();

        let flags = read_u32(input);

        let key = if let Some(addr) = kay_address {
            Key::DataKey(addr, key_length)
        } else {
            if input.len() < (20 + key_length) as usize {
                println!("Warning: couldn't parse string key of index...");
                Key::LocalKey("".into())
            } else {
                let key = String::from_utf8(input.split_off(20usize).split_to(key_length as usize).to_vec()).map_err(|_| Error::StringParseError)?;
                Key::LocalKey(key.to_owned())
            }
        };

        Ok((Entry {
                hash, ranking_node, usage_count, reuse_count, state, creation_time, flags, key, data, header: None
            },
            next
        ))
    }

    fn get_all<P: AsRef<Path>>(addr: Address, base_path: P) -> crate::Result<Vec<Self>> {
        let mut file = File::open(base_path.as_ref().join(addr.file_name))?;

        file.seek(SeekFrom::Start((8192 + addr.block_number * (addr.blocktype.get_size() as u32)).into()))?;

        let mut buf = read_exact(&mut file, addr.blocktype.get_size()*((addr.contiguous_block + 1) as usize))?;
        let (entry, next) = Self::parse(&mut buf)?;

        if let Some(addr) = next {
            let mut result = Self::get_all(addr, base_path)?;
            result.push(entry);
            Ok(result)
        } else {
            Ok(vec![entry])
        }
    }

    pub fn analyze_header<P: AsRef<Path>>(self, base_path: P) -> crate::Result<Self> {
        let (header, data) = 
            if let Key::DataKey(addr, _) = &self.key {
                (crate::data::parse_http_header(&addr, base_path.as_ref())?, self.data)
            } else {
                let mut header = None;
                let mut data = Vec::new();

                for (addr, size) in self.data {
                    let result = crate::data::parse_http_header(&addr, base_path.as_ref())?;
                    match result {
                        Some(_) => header = result,
                        None => data.push((addr, size)),
                    }
                }

                (header, data)
            };

        Ok(Self { header: header, data: data, ..self })
    }

    pub fn new<P: AsRef<Path>>(addr: Address, base_path: P) -> crate::Result<Vec<Self>> {
        let mut entries = Self::get_all(addr, base_path.as_ref())?;
        entries.reverse();

        entries.into_iter().map(|x| x.analyze_header(base_path.as_ref())).collect()
    }
}
