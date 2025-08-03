use crate::*;
use std::io;

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

    Str => String
    Bytes => Vec<u8>
}

pub trait ConvertFrom<T>: Sized {
    type Error;
    fn convert_from(value: T) -> Result<Self, Self::Error>;
}

impl<'v, 'de, T> ConvertFrom<Option<&'v Value<'de>>> for Option<T>
where
    T: TryFrom<&'v Value<'de>, Error = io::Error>,
{
    type Error = T::Error;

    fn convert_from(value: Option<&'v Value<'de>>) -> Result<Self> {
        value.map(T::try_from).transpose()
    }
}

impl<'v, 'de, T> ConvertFrom<Option<&'v Value<'de>>> for T
where
    T: TryFrom<&'v Value<'de>, Error = io::Error>,
{
    type Error = T::Error;

    fn convert_from(value: Option<&'v Value<'de>>) -> Result<Self> {
        match value {
            Some(val) => T::try_from(val),
            None => {
                let error = format!("expected `{}`, found `None`", std::any::type_name::<T>());
                Err(io::Error::new(io::ErrorKind::InvalidInput, error))
            }
        }
    }
}

#[doc(hidden)]
pub fn convert_into_struct<'de, T>(val: &Value<'de>) -> Result<T, std::io::Error>
where
    T: Decoder<'de>,
{
    match val {
        Value::Struct(entries) => T::decode(entries),
        _ => {
            let error = format!("expected `{}`, found `None`", std::any::type_name::<T>());
            Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, error))
        }
    }
}
