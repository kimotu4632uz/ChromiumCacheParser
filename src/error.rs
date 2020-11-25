use thiserror::Error;
use std::io;

//#[derive(Debug, Error)]
//pub enum Err<T: fmt::Debug> {
//    #[error("error caused by address parse")]
//    Address(AddressError<T>),
//
//    #[error("error caused by parse")]
//    Nom(NErr<NError<T>>),
//}
//
//impl<T: fmt::Debug> From<NErr<NError<T>>> for Err<T> {
//    fn from(err: NErr<NError<T>>) -> Err<T> {
//        Err::Nom(err)
//    }
//}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Address Parse Error: {0}")]
    AddressError(AddressErrorKind),

    #[error("Index file magic is not satisfied")]
    MagicError,

    #[error("Unexpected EOF while parsing {0}")]
    UnexpectedEOF(String),

    #[error("Error while parsing string")]
    StringParseError,

    #[error("IO Error: {0}")]
    IOError(io::Error),
}

impl From<AddressErrorKind> for Error {
    fn from(kind: AddressErrorKind) -> Self {
        Error::AddressError(kind)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IOError(err)
    }
}

impl Error {
    pub fn add_eof_location(self, location: &str) -> Self {
        match self {
            Error::UnexpectedEOF(_) => Error::UnexpectedEOF(location.into()),
            _ => self,
        }
    }
}

#[derive(Debug, Error)]
pub enum AddressErrorKind {
    #[error("address is null")]
    NullAddress,
    
    #[error("address is not initialized")]
    UnInitialized,

    #[error("block type of address is invalid")]
    InvalidBlockType,
}

