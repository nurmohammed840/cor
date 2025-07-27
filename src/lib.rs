mod decoder;
mod errors;
mod field;
mod leb128;
mod utils;
mod zig_zag;

use std::io::{Result, Write};

pub trait Encoder {
    fn encode(&self, _: &mut (impl Write + ?Sized)) -> Result<()>;
}

pub trait Decoder<'de>: Sized {
    fn decode(_: &mut &'de [u8]) -> Result<Self>;
}
