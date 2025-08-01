use crate::{errors, leb128, utils, zig_zag};

use super::Result;
use std::{
    fmt::{self, Debug, Write},
    io,
};

#[derive(Clone)]
pub enum Value<'de> {
    Bool(bool),
    F32(f32),
    F64(f64),
    Int(i64),
    UInt(u64),
    Str(&'de str),
    Bytes(&'de [u8]),
    List(List<'de>),
    Struct(Vec<(u32, Value<'de>)>),
}

#[derive(Clone)]
pub enum List<'de> {
    Bool(Vec<bool>),
    F32(Vec<f32>),
    F64(Vec<f64>),
    Int(Vec<i64>),
    UInt(Vec<u64>),
    Str(Vec<&'de str>),
    Bytes(Vec<&'de [u8]>),
    List(Vec<List<'de>>),
    Struct(Vec<Vec<(u32, Value<'de>)>>),
}

impl Debug for List<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool(val) => Debug::fmt(val, f),
            Self::F32(val) => Debug::fmt(val, f),
            Self::F64(val) => Debug::fmt(val, f),
            Self::Int(val) => Debug::fmt(val, f),
            Self::UInt(val) => Debug::fmt(val, f),
            Self::Str(val) => Debug::fmt(val, f),
            Self::Bytes(val) => Debug::fmt(val, f),
            Self::List(val) => Debug::fmt(val, f),
            Self::Struct(val) => Debug::fmt(val, f),
        }
    }
}

impl Debug for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(val) => val.fmt(f),
            Value::UInt(val) => write!(f, "{val}u",),
            Value::Str(val) => val.fmt(f),
            Value::Bytes(bytes) => {
                f.write_char('(')?;
                let mut bytes = bytes.iter().peekable();
                while let Some(byte) = bytes.next() {
                    if bytes.peek().is_some() {
                        write!(f, "{byte} ")?;
                    } else {
                        write!(f, "{byte}")?;
                    }
                }
                f.write_char(')')
            }
            Value::F32(val) => write!(f, "{val:#?}f"),
            Value::F64(val) => val.fmt(f),
            Value::Bool(val) => val.fmt(f),
            Value::List(list) => list.fmt(f),
            Value::Struct(items) => f
                .debug_map()
                .entries(items.iter().map(|(id, val)| (id, val)))
                .finish(),
        }
    }
}

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

fn parse_string<'de>(reader: &mut &'de [u8]) -> Result<&'de str> {
    parse_bytes(reader)
        .map(str::from_utf8)?
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
}

fn parse_bytes<'de>(reader: &mut &'de [u8]) -> Result<&'de [u8]> {
    let len = leb128::read_unsigned(reader).map(try_into_u32)??;
    utils::read_bytes(reader, len as usize)
}

fn collect<T>(len: u32, mut f: impl FnMut() -> Result<T>) -> Result<Vec<T>> {
    let mut arr = Vec::new();
    for _ in 0..len {
        arr.push(f()?);
    }
    Ok(arr)
}

fn parse_list<'de>(reader: &mut &'de [u8]) -> Result<List<'de>> {
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

pub fn parse_struct<'de>(reader: &mut &'de [u8]) -> Result<Vec<(u32, Value<'de>)>> {
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
                debug_assert!(id == 0);
                break; // End of struct
            }
            op => Err(errors::unknown_opcode(op)),
        };
        obj.push((id, val?));
    }
    Ok(obj)
}

macro_rules! convert {
    [$($name:ident($ty:ty))*] => [$(
        impl<'de> TryFrom<&Value<'de>> for $ty {
            type Error = io::Error;
            fn try_from(val: &Value<'de>) -> Result<Self> {
                match val {
                    Value::$name(val) => Ok(*val),
                    val => Err(val.invalid_type(std::any::type_name::<$ty>())),
                }
            }
        }
    )*];
    [$($name:ident => $ty:ty)*] => [$(
        impl TryFrom<&Value<'_>> for $ty {
            type Error = io::Error;
            fn try_from(val: &Value) -> Result<Self> {
                match val {
                    Value::$name(val) => <$ty>::try_from(*val).map_err(errors::invalid_input),
                    val => Err(val.invalid_type(std::any::type_name::<$ty>())),
                }
            }
        }
    )*]
}

convert! {
    Bool(bool)
    F32(f32)
    F64(f64)
    Int(i64)
    UInt(u64)
    Str(&'de str)
    Bytes(&'de [u8])
}

convert! {
    Int => i8
    Int => i16
    Int => i32

    UInt => u8
    UInt => u16
    UInt => u32
}

impl Value<'_> {
    pub fn try_get<'e, T>(key: u32, entries: &'e Vec<(u32, Self)>) -> Result<Option<T>, T::Error>
    where
        T: TryFrom<&'e Self>,
    {
        entries
            .iter()
            .find_map(|(k, v)| (*k == key).then_some(v))
            .map(T::try_from)
            .transpose()
    }
}
