use crate::{Value, convert::ConvertFrom, errors};

#[derive(Clone, Debug)]
pub struct Entry<'de> {
    pub key: u32,
    pub value: Value<'de>,
}

#[derive(Clone, Default)]
pub struct Entries<'de>(pub(crate) Vec<Entry<'de>>);

impl<'de> Entries<'de> {
    pub fn get(&self, k: u32) -> Option<&Value<'de>> {
        self.0
            .iter()
            .find_map(|Entry { key, value }| (*key == k).then_some(value))
    }

    pub fn get_and_convert<'v, T>(&'v self, k: u32) -> Result<T, errors::ConvertError>
    where
        T: ConvertFrom<Option<&'v Value<'de>>>,
    {
        T::convert_from(self.get(k)).map_err(|mut err| {
            err.key = Some(k);
            err
        })
    }

    pub(crate) fn iter(&self) -> &[Entry<'de>] {
        &self.0
    }
}
