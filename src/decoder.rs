#![allow(warnings)]
use crate::{errors, leb128, utils, zig_zag};

use super::Result;
use std::{
    fmt,
    io::{self, Read},
};

#[derive(Clone)]
enum Value {
    Bool(bool),
    F32(f32),
    F64(f64),
    Int(i64),
    UInt(u64),
    Str(String),
    Bytes(Vec<u8>),
    List(List),
    Struct(Vec<(u32, Value)>),
}

#[derive(Clone)]
enum List {
    Bool(Vec<bool>),
    F32(Vec<f32>),
    F64(Vec<f64>),
    Int(Vec<i64>),
    UInt(Vec<u64>),
    Str(Vec<String>),
    Bytes(Vec<Vec<u8>>),
    List(Vec<List>),
    Struct(Vec<Vec<(u32, Value)>>),
}

// impl fmt::Debug for Value {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             Value::I(val) => val.fmt(f),
//             Value::U(val) => write!(f, "{val}u",),
//             Value::Str(val) => val.fmt(f),
//             Value::Bytes(val) => fmt::Debug::fmt(val, f),
//             Value::F32(val) => write!(f, "{val}_f32",),
//             Value::F64(val) => val.fmt(f),
//             Value::Bool(val) => val.fmt(f),
//             // Value::List(values) => f.debug_list().entries(values.iter()).finish(),
//             Value::Struct(items) => f
//                 .debug_map()
//                 .entries(items.iter().map(|(id, val)| (id, val)))
//                 .finish(),
//         }
//     }
// }

fn decode_field_ty(reader: &mut &[u8]) -> Result<(u32, u8)> {
    let byte = utils::read_byte(reader)?;

    let ty = byte & 0b00001111;
    let id = (byte >> 4) as u32;

    let id = if id == 0b1111 {
        try_into_u32(leb128::read_unsigned(reader)? + 15)?
    } else {
        id
    };
    Ok((id, ty))
}

fn try_into_u32(num: u64) -> Result<u32> {
    num.try_into()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "number out of range"))
}

fn parse_string(reader: &mut &[u8]) -> Result<String> {
    parse_bytes(reader)
        .map(String::from_utf8)?
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
}

fn parse_bytes(reader: &mut &[u8]) -> Result<Vec<u8>> {
    let len = leb128::read_unsigned(reader).map(try_into_u32)??;
    Ok(utils::read_bytes(reader, len as usize)?.to_owned())
}

fn collect<T>(len: u32, mut f: impl FnMut() -> Result<T>) -> Result<Vec<T>> {
    let mut arr = Vec::new();
    for _ in 0..len {
        arr.push(f()?);
    }
    Ok(arr)
}

fn parse_list(reader: &mut &[u8]) -> Result<List> {
    let (len, ty) = decode_field_ty(reader)?;
    match ty {
        0 | 1 => collect(len, || Ok(utils::read_byte(reader)? != 0)).map(List::Bool),
        2 => collect(len, || utils::read_buf(reader).map(f32::from_le_bytes)).map(List::F32),
        3 => collect(len, || utils::read_buf(reader).map(f64::from_le_bytes)).map(List::F64),
        4 => collect(len, || leb128::read_unsigned(reader).map(zig_zag::from)).map(List::Int),
        5 => collect(len, || leb128::read_unsigned(reader)).map(List::UInt),
        6 => collect(len, || parse_string(reader)).map(List::Str),
        7 => collect(len, || parse_bytes(reader)).map(List::Bytes),
        8 => collect(len, || parse_list(reader)).map(List::List),
        9 => collect(len, || parse_struct(reader)).map(List::Struct),
        op => Err(errors::unknown_opcode(op)),
    }
}

fn parse_struct(reader: &mut &[u8]) -> Result<Vec<(u32, Value)>> {
    let mut obj = Vec::new();

    loop {
        let (id, ty) = decode_field_ty(reader)?;
        let val = match ty {
            0 => Ok(Value::Bool(false)),
            1 => Ok(Value::Bool(true)),

            2 => utils::read_buf(reader)
                .map(f32::from_le_bytes)
                .map(Value::F32),

            3 => utils::read_buf(reader)
                .map(f64::from_le_bytes)
                .map(Value::F64),

            4 => leb128::read_unsigned(reader)
                .map(zig_zag::from)
                .map(Value::Int),

            5 => leb128::read_unsigned(reader).map(Value::UInt),
            6 => parse_string(reader).map(Value::Str),
            7 => parse_bytes(reader).map(Value::Bytes),
            8 => parse_list(reader).map(Value::List),
            9 => parse_struct(reader).map(Value::Struct),
            10 => {
                debug_assert!(id != 0);
                break; // End of struct
            }
            op => Err(errors::unknown_opcode(op)),
        };
        obj.push((id, val?));
    }
    Ok(obj)
}
