use std::io::{Seek, SeekFrom};
use std::fs::File;
use std::path::Path;

use bytes::Buf;
use chrono::prelude::*;

use crate::address::{Address, BlockType};
use crate::reader::*;
use crate::error::Error;

#[derive(Debug)]
pub struct HttpHeader {
    pub server_responce: String,
    pub content_type: String,
    pub content_length: u32,
    pub server_time: DateTime<Utc>,
    pub server_last_modified: DateTime<Utc>,
    pub etag: String,
    pub server_name: String
}

impl HttpHeader {
    pub fn new() -> Self {
        HttpHeader {server_responce: String::new(), content_type: String::new(), content_length: 0, server_time: Utc::now(), server_last_modified: Utc::now(), etag: String::new(), server_name: String::new() }
    }
}

pub fn parse_http_header<P: AsRef<Path>>(addr: &Address, base_url: P) -> crate::Result<Option<HttpHeader>> {
    if addr.blocktype != BlockType::SeparateFile {
        let mut file = File::open(base_url.as_ref().join(&addr.file_name))?;

        file.seek(SeekFrom::Start((8192 + addr.block_number * (addr.blocktype.get_size() as u32)).into()))?;
        let byte = read_exact(&mut file, addr.blocktype.get_size())?;

        let size = byte.remaining();
        let mut start = 0;
        let mut end = 0;

        for pos in 0..(size-4) {
            if byte[pos] == b'H' && byte[pos+1] == b'T' && byte[pos+2] == b'T' && byte[pos+3] == b'P' {
                start = pos;
                break;
            }
        }

        if start == 0 { return Ok(None) }

        for pos in start..(size-2) {
            if byte[pos] == b'\x00' && byte[pos+1] == b'\x00' {
                end = pos;
                break;
            }
        }

        if end == 0 { Ok(None) }
        else {
            let header = std::str::from_utf8(&byte[start..end]).map_err(|_| Error::StringParseError)?;
            let mut lines = header.split('\x00');
            let resp = lines.next().unwrap();

            let mut header = HttpHeader::new();
            header.server_responce = resp.into();

            for line in lines {
                let pair: Vec<&str> = line.splitn(2, ": ").collect();

                match pair[0] {
                    "Date" => header.server_time = DateTime::from_utc(DateTime::parse_from_rfc2822(pair[1]).unwrap().naive_utc(), Utc),
                    "Content-Type" => header.content_type = pair[1].into(),
                    "Content-Length" => header.content_length = pair[1].parse().unwrap(),
                    "Server" => header.server_name = pair[1].into(),
                    "Last-Modified" => header.server_last_modified = DateTime::from_utc(DateTime::parse_from_rfc2822(pair[1]).unwrap().naive_utc(), Utc),
                    "ETag" => {
                        let tmp = &pair[1][2..];
                        header.etag = tmp[..(tmp.len() - 2)].into();
                    },
                    _ => continue,
                }
            }
            Ok(Some(header))
        }
    } else {
        Ok(None)
    }
}