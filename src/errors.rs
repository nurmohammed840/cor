use crate::{List, Value};
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

pub fn invalid_input(err: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Error {
    Error::new(ErrorKind::InvalidInput, err)
}

impl Value<'_> {
    fn type_name(&self) -> &str {
        match self {
            Value::Bool(_) => "boolean",
            Value::F32(_) => "f32",
            Value::F64(_) => "f64",
            Value::Int(_) => "integer",
            Value::UInt(_) => "unsigned integer",
            Value::Str(_) => "string",
            Value::Bytes(_) => "bytes",
            Value::Struct(_) => "struct",
            Value::List(list) => match list {
                List::Bool(_) => "[boolean]",
                List::F32(_) => "[f32]",
                List::F64(_) => "[f64]",
                List::Int(_) => "[integer]",
                List::UInt(_) => "[unsigned integer]",
                List::Str(_) => "[string]",
                List::Bytes(_) => "[bytes]",
                List::Struct(_) => "[struct]",
                List::List(_) => "[...]",
            },
        }
    }

    pub(crate) fn invalid_type(&self, expected: &str) -> Error {
        let error = format!("expected `{expected}`, found `{}`", self.type_name());
        Error::new(ErrorKind::InvalidInput, error)
    }
}
