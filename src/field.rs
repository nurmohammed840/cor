use leb128::*;

use super::*;

// #[doc(hidden)]
pub trait FieldEncoder {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u32) -> Result<()>;
}

impl<T: FieldEncoder> FieldEncoder for Option<T> {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u32) -> Result<()> {
        match self {
            Some(val) => FieldEncoder::encode(val, writer, id),
            None => Ok(()),
        }
    }
}

fn encode_field_ty(writer: &mut (impl Write + ?Sized), id: u32, ty: u8) -> Result<()> {
    if id < 15 {
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

impl FieldEncoder for f32 {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u32) -> Result<()> {
        encode_field_ty(writer, id, 2)?;
        writer.write_all(&self.to_le_bytes())
    }
}

impl FieldEncoder for f64 {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u32) -> Result<()> {
        encode_field_ty(writer, id, 3)?;
        writer.write_all(&self.to_le_bytes())
    }
}

fn encode_sign(writer: &mut (impl Write + ?Sized), id: u32, num: impl Into<i64>) -> Result<()> {
    encode_field_ty(writer, id, 4)?;

    let mut buf = Leb128Buf::<10>::new();
    buf.write_u64(zig_zag::into(num.into()));
    writer.write_all(buf.as_bytes())
}

fn encode_unsign(writer: &mut (impl Write + ?Sized), id: u32, num: impl Into<u64>) -> Result<()> {
    encode_field_ty(writer, id, 5)?;

    let mut buf = Leb128Buf::<10>::new();
    buf.write_u64(num.into());
    writer.write_all(buf.as_bytes())
}

impl FieldEncoder for str {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u32) -> Result<()> {
        let bytes = self.as_bytes();

        encode_field_ty(writer, id, 6)?;
        encode_len_u32(writer, bytes.len())?;
        writer.write_all(bytes)
    }
}

impl FieldEncoder for [u8] {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u32) -> Result<()> {
        encode_field_ty(writer, id, 7)?;
        encode_len_u32(writer, self.len())?;
        writer.write_all(self)
    }
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

// ----------------------------------------------

trait Item {
    fn ty() -> u8;
    fn encode(&self, writer: &mut (impl Write + ?Sized)) -> Result<()>;
}

impl Item for bool {
    fn ty() -> u8 {
        1
    }

    fn encode(&self, writer: &mut (impl Write + ?Sized)) -> Result<()> {
        writer.write_all(&[match self {
            false => 1,
            true => 2,
        }])
    }
}

impl Item for f32 {
    fn ty() -> u8 {
        2
    }

    fn encode(&self, writer: &mut (impl Write + ?Sized)) -> Result<()> {
        writer.write_all(&self.to_le_bytes())
    }
}

impl Item for f64 {
    fn ty() -> u8 {
        3
    }
    fn encode(&self, writer: &mut (impl Write + ?Sized)) -> Result<()> {
        writer.write_all(&self.to_le_bytes())
    }
}

macro_rules! impl_item {
    (@sign: $($ty: ty),*) => {$(
        impl Item for $ty {
            fn ty() -> u8 { 4 }
            fn encode(&self, writer: &mut (impl Write + ?Sized)) -> Result<()> {
                let mut buf = Leb128Buf::<10>::new();
                buf.write_u64(zig_zag::into((*self).into()));
                writer.write_all(buf.as_bytes())
            }
        }
    )*};
    (@unsign: $($ty: ty),*) => {$(
        impl Item for $ty {
            fn ty() -> u8 { 5 }
            fn encode(&self, writer: &mut (impl Write + ?Sized)) -> Result<()> {
                let mut buf = Leb128Buf::<10>::new();
                buf.write_u64((*self).into());
                writer.write_all(buf.as_bytes())
            }
        }
    )*};
}

impl_item! { @sign: i16, i32, i64 }
impl_item! { @unsign: u16, u32, u64 }

impl Item for str {
    fn ty() -> u8 {
        6
    }

    fn encode(&self, writer: &mut (impl Write + ?Sized)) -> Result<()> {
        let bytes = self.as_bytes();
        encode_len_u32(writer, bytes.len())?;
        writer.write_all(bytes)
    }
}

impl Item for [u8] {
    fn ty() -> u8 {
        7
    }
    fn encode(&self, writer: &mut (impl Write + ?Sized)) -> Result<()> {
        encode_len_u32(writer, self.len())?;
        writer.write_all(self)
    }
}

impl<T: Item> Item for [T] {
    fn ty() -> u8 {
        8
    }
    fn encode(&self, writer: &mut (impl Write + ?Sized)) -> Result<()> {
        self.iter().try_for_each(|el| T::encode(el, writer))
    }
}

impl<T: Encoder> Item for T {
    fn ty() -> u8 {
        9
    }
    fn encode(&self, writer: &mut (impl Write + ?Sized)) -> Result<()> {
        T::encode(self, writer)
    }
}

impl<T: Item> FieldEncoder for [T] {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u32) -> Result<()> {
        encode_field_ty(writer, id, 8)?;

        // +--------+--------+...+--------+
        // |sssstttt| elements            |
        // +--------+--------+...+--------+
        // Compact protocol list header (2+ bytes, long form) and elements:
        // +--------+--------+...+--------+--------+...+--------+
        // |1111tttt| size                | elements            |
        // +--------+--------+...+--------+--------+...+--------+
        encode_field_ty(writer, self.len().try_into().unwrap(), T::ty())?;
        self.iter().try_for_each(|el| T::encode(el, writer))
    }
}

impl<T: Encoder> FieldEncoder for T {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u32) -> Result<()> {
        encode_field_ty(writer, id, 9)?;
        T::encode(self, writer)
    }
}

// ------------------------------------------------------------------------------------

impl FieldEncoder for String {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u32) -> Result<()> {
        FieldEncoder::encode(self.as_str(), writer, id)
    }
}

impl FieldEncoder for Vec<u8> {
    fn encode(&self, writer: &mut (impl Write + ?Sized), id: u32) -> Result<()> {
        FieldEncoder::encode(self.as_slice(), writer, id)
    }
}
