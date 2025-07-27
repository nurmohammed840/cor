use std::io::{Error, ErrorKind};

pub fn eof() -> Error {
    Error::new(ErrorKind::UnexpectedEof, "Failed to read field type")
}

pub fn leb128_err() -> Error {
    Error::new(ErrorKind::InvalidData, "LEB128 overflow")
}

pub fn unknown_opcode(code: u8) -> Error {
    Error::new(ErrorKind::InvalidData, format!("Unknown opcode: {code}",))
}
