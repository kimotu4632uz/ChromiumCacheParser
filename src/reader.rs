use bytes::{Buf, Bytes};

use std::io;
use crate::error::Error;

pub fn read_u32(input: &mut Bytes) -> u32 {
    if cfg!(target_endian = "big") {
        input.get_u32()
    } else {
        input.get_u32_le()
    }
}

pub fn read_i64(input: &mut Bytes) -> i64 {
    if cfg!(target_endian = "big") {
        input.get_i64()
    } else {
        input.get_i64_le()
    }
}

pub fn count<T: Sized>(f: fn(_: &mut Bytes) -> T, count: usize) -> impl FnMut(&mut Bytes) -> Vec<T> {
    move |input| {
        let mut vec: Vec<T> = Vec::new();
        for _ in 0..count {
            let result = f(input);
            vec.push(result);
        }
        vec
    }
}

pub fn try_count<T: Sized>(f: fn(_: &mut Bytes) -> crate::Result<T>, count: usize) -> impl FnMut(&mut Bytes) -> crate::Result<Vec<T>> {
    move |input| {
        let mut vec: Vec<T> = Vec::with_capacity(count);
        for _ in 0..count {
            let result = f(input)?;
            vec.push(result);
        }
        Ok(vec)
    }
}

pub fn read_exact<I: io::Read>(mut input: I, size: usize) -> crate::Result<Bytes> {
    let mut result = Vec::with_capacity(size);
    result.resize(size, 0);
    match input.read(&mut result) {
        Ok(len) => {
            if len == size {
                Ok(Bytes::from(result))
            } else {
                Err(Error::UnexpectedEOF("".into()))
            }
        },
        Err(err) => Err(Error::IOError(err))
    }
}