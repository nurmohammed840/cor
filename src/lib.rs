mod convert;
mod decoder;
mod encoder;
mod entry;
pub mod errors;
mod leb128;
mod utils;
mod zig_zag;

use std::io::{self, Write};

pub use cor_macro::*;
pub use encoder::FieldEncoder;

pub use decoder::{List, Value};
pub use entry::{Entries, Entry};

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub trait Encoder {
    fn encode(&self, _: &mut (impl Write + ?Sized)) -> io::Result<()>;
}

pub trait Decoder<'de>: Sized {
    fn parse(reader: &mut &'de [u8]) -> Result<Self> {
        Self::decode(&Entries::parse(reader)?)
    }

    fn decode(entries: &Entries<'de>) -> Result<Self>;
}
