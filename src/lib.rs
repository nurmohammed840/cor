mod decoder;
mod encoder;
mod errors;
mod leb128;
mod utils;
mod zig_zag;

use std::io::Write;

pub use cor_macro::*;
pub use decoder::*;
pub use encoder::FieldEncoder;

pub type Result<T, E = std::io::Error> = std::result::Result<T, E>;

pub trait Encoder {
    fn encode(&self, _: &mut (impl Write + ?Sized)) -> Result<()>;
}

pub trait Decoder<'de>: Sized {
    fn decode(_: &mut &'de [u8]) -> Result<Self>;
}
