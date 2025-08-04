use crate::{errors::ConvertError, *};
use std::any::type_name;

macro_rules! convert {
    [$($name:ident($ty:ty))*] => [$(
        impl<'de> ConvertFrom<&Value<'de>> for $ty {
            fn convert_from(val: &Value<'de>) -> Result<Self, ConvertError> {
                match val {
                    Value::$name(val) => Ok(*val),
                    val => Err(val.invalid_type(type_name::<$ty>())),
                }
            }
        }
    )*];
    [$($name:ident => $ty:ty)*] => [$(
        impl ConvertFrom<&Value<'_>> for $ty {
            fn convert_from(val: &Value) -> Result<Self, ConvertError> {
                match val {
                    Value::$name(val) => <$ty>::try_from(*val).map_err(ConvertError::from),
                    val => Err(val.invalid_type(type_name::<$ty>())),
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
    fn convert_from(value: T) -> Result<Self, errors::ConvertError>;
}

impl<'v, 'de, T> ConvertFrom<Option<&'v Value<'de>>> for Option<T>
where
    T: ConvertFrom<&'v Value<'de>>,
{
    fn convert_from(value: Option<&'v Value<'de>>) -> Result<Self, errors::ConvertError> {
        value.map(T::convert_from).transpose()
    }
}

impl<'v, 'de, T> ConvertFrom<Option<&'v Value<'de>>> for T
where
    T: ConvertFrom<&'v Value<'de>>,
{
    fn convert_from(value: Option<&'v Value<'de>>) -> Result<Self, errors::ConvertError> {
        match value {
            Some(val) => T::convert_from(val),
            None => Err(ConvertError::new(format!(
                "expected `{}`, found `None`",
                type_name::<T>()
            ))),
        }
    }
}

impl<'v, 'de, T> ConvertFrom<&'v Value<'de>> for T
where
    T: Decoder<'de>,
{
    fn convert_from(value: &'v Value<'de>) -> Result<Self, errors::ConvertError> {
        match value {
            Value::Struct(entries) => T::decode(entries).map_err(ConvertError::from),
            _ => Err(ConvertError::new(format!(
                "expected `{}`, found `None`",
                type_name::<T>()
            ))),
        }
    }
}
