use leb128::*;

use super::*;

pub trait FieldEncoder {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u32) -> Result<()>;
}

// pub trait FieldDecoder<'de>: Sized {
//     fn decode(_: &mut &'de [u8]) -> Result<Self>;
// }

impl<T: FieldEncoder> FieldEncoder for Option<T> {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u32) -> Result<()> {
        match self {
            Some(val) => FieldEncoder::encode(val, writer, id),
            None => Ok(()),
        }
    }
}

fn encode_field_ty(writer: &mut (impl Write + ?Sized), id: u32, ty: u8) -> Result<()> {
    if id >= 15 {
        let header = (id as u8) << 4;
        writer.write_all(&[header | ty])
    } else {
        let header = 0b1111 << 4;
        let mut buf: Leb128Buf<8> = Leb128Buf::<8>::new();
        buf.write_byte(header | ty);
        buf.write_u32(id - 15);
        writer.write_all(buf.as_bytes())
    }
}

fn encode_len_u32(writer: &mut (impl Write + ?Sized), len: usize) -> Result<()> {
    let len: u32 = len.try_into().unwrap();
    let mut buf = Leb128Buf::<8>::new();
    buf.write_u32(len);
    writer.write_all(buf.as_bytes())
}

impl FieldEncoder for bool {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u32) -> Result<()> {
        let ty = match self {
            false => 0,
            true => 1,
        };
        encode_field_ty(writer, id, ty)
    }
}

impl FieldEncoder for str {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u32) -> Result<()> {
        encode_field_ty(writer, id, 2)?;
        encode_len_u32(writer, self.as_bytes().len())?;
        writer.write_all(self.as_bytes())
    }
}

impl FieldEncoder for [u8] {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u32) -> Result<()> {
        encode_field_ty(writer, id, 3)?;
        encode_len_u32(writer, self.len())?;
        writer.write_all(self)
    }
}

impl FieldEncoder for f32 {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u32) -> Result<()> {
        encode_field_ty(writer, id, 4)?;
        writer.write_all(&self.to_le_bytes())
    }
}

impl FieldEncoder for f64 {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u32) -> Result<()> {
        encode_field_ty(writer, id, 5)?;
        writer.write_all(&self.to_le_bytes())
    }
}

fn encode_unsign(writer: &mut (impl Write + ?Sized), id: u32, num: impl Into<u64>) -> Result<()> {
    encode_field_ty(writer, id, 6)?;

    let mut buf = Leb128Buf::<10>::new();
    buf.write_u64(num.into());
    writer.write_all(buf.as_bytes())
}

fn to_zig_zag(num: i64) -> u64 {
    ((num << 1) ^ (num >> 63)) as u64
}

// fn from_zig_zag(num: u64) -> i64 {
//     ((num >> 1) as i64) ^ -((num & 1) as i64)
// }

fn encode_sign(writer: &mut (impl Write + ?Sized), id: u32, num: impl Into<i64>) -> Result<()> {
    encode_field_ty(writer, id, 7)?;

    let mut buf = Leb128Buf::<10>::new();
    buf.write_u64(to_zig_zag(num.into()));
    writer.write_all(buf.as_bytes())
}

macro_rules! impl_for {
    (unsign: $($ty: ty)*) => {$(
        impl FieldEncoder for $ty {
            fn encode(&self, writer: &mut (impl Write + ?Sized), id: u32) -> Result<()> {
                encode_unsign(writer, id, *self)
            }
        }
    )*};
    (sign: $($ty: ty)*) => {$(
        impl FieldEncoder for $ty {
            fn encode(&self, writer: &mut (impl Write + ?Sized), id: u32) -> Result<()> {
                encode_sign(writer, id, *self)
            }
        }
    )*};
}

impl_for! {
    unsign: u8 u16 u32 u64
}

impl_for! {
    sign: i8 i16 i32 i64
}

impl<T: Encoder> FieldEncoder for T {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u32) -> Result<()> {
        encode_field_ty(writer, id, 8)?;
        T::encode(self, writer)
    }
}

