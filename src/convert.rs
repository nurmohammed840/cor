use crate::{errors::ConvertError, *};

macro_rules! convert {
    [$($name:ident($ty:ty))*] => [$(
        impl<'de> TryFrom<&Value<'de>> for $ty {
            type Error = ConvertError;
            fn try_from(val: &Value<'de>) -> Result<Self, Self::Error> {
                match val {
                    Value::$name(val) => Ok(*val),
                    val => Err(val.invalid_type(std::any::type_name::<$ty>())),
                }
            }
        }
    )*];
    [$($name:ident => $ty:ty)*] => [$(
        impl TryFrom<&Value<'_>> for $ty {
            type Error = ConvertError;
            fn try_from(val: &Value) -> Result<Self, Self::Error> {
                match val {
                    Value::$name(val) => <$ty>::try_from(*val).map_err(ConvertError::from),
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
    fn convert_from(value: T) -> Result<Self, errors::ConvertError>;
}

impl<'v, 'de, T> ConvertFrom<Option<&'v Value<'de>>> for Option<T>
where
    T: TryFrom<&'v Value<'de>, Error = ConvertError>,
{
    fn convert_from(value: Option<&'v Value<'de>>) -> Result<Self, errors::ConvertError> {
        value.map(T::try_from).transpose()
    }
}

impl<'v, 'de, T> ConvertFrom<Option<&'v Value<'de>>> for T
where
    T: TryFrom<&'v Value<'de>, Error = ConvertError>,
{
    fn convert_from(value: Option<&'v Value<'de>>) -> Result<Self, errors::ConvertError> {
        match value {
            Some(val) => T::try_from(val),
            None => Err(ConvertError::new(format!(
                "expected `{}`, found `None`",
                std::any::type_name::<T>()
            ))),
        }
    }
}

#[doc(hidden)]
pub fn convert_into_struct<'de, T>(val: &Value<'de>) -> Result<T, ConvertError>
where
    T: Decoder<'de>,
{
    match val {
        Value::Struct(entries) => T::decode(entries).map_err(ConvertError::from),
        _ => Err(ConvertError::new(format!(
            "expected `{}`, found `None`",
            std::any::type_name::<T>()
        ))),
    }
}
