pub mod error;
pub mod index;
pub mod address;
pub mod entry;
pub mod data;
pub mod reader;

use std::io::{Seek, SeekFrom};
use std::fs::File;
use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, error::Error>;

pub struct Cache {
    pub header: index::IndexHeader,
    pub entries: Vec<entry::Entry>,
    pub base_path: PathBuf,
}

impl Cache {
    pub fn copy_data<P: AsRef<Path>>(&self, target: &entry::Entry, dst: P) -> crate::Result<()> {
        if let entry::Key::LocalKey(key) = &target.key {
            let fname = key.split('/').last().unwrap();

            let target_addr = &target.data[0].0;
            if target_addr.blocktype == address::BlockType::SeparateFile {
                std::fs::copy(self.base_path.join(&target_addr.file_name), dst.as_ref().join(fname))?;
            } else {
                let mut data = File::open(self.base_path.join(&target_addr.file_name))?;
                data.seek(SeekFrom::Start((8192 + target_addr.block_number * (target_addr.blocktype.get_size() as u32)).into()))?;

                let byte = reader::read_exact(&mut data, target.data[0].1 as usize)?;
                std::fs::write(dst.as_ref().join(fname), byte)?;
            }
        }
        Ok(())
    }
}

pub fn parse<P: AsRef<Path>>(index: P) -> Result<Cache> {
    let path = index.as_ref();
    let base_path = path.parent().unwrap();
    let mut index_f = File::open(path)?;

    let mut buf = reader::read_exact(&mut index_f, index::INDEX_HEADER_SIZE).map_err(|err| err.add_eof_location("IndexHeader"))?;
    let header = index::IndexHeader::parse(&mut buf)?;

    index_f.seek(SeekFrom::Current(index::INDEX_HEADER_PADDING))?;

    let mut buf = reader::read_exact(&mut index_f, (header.table_len * 4) as usize).map_err(|err| err.add_eof_location("Index AddressTable"))?;
    let table = index::IndexTable::parse(&mut buf, header.table_len)?;
    let entries = table.analyze(base_path)?;

    Ok(Cache {header, entries, base_path: base_path.into()})
}

