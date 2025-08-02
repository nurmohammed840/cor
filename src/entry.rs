use crate::{Value, convert::ConvertFrom};

#[derive(Clone, Debug)]
pub struct Entry<'de> {
    pub key: u32,
    pub value: Value<'de>,
}

#[derive(Clone, Debug, Default)]
pub struct Entries<'de>(pub(crate) Vec<Entry<'de>>);

impl<'de> Entries<'de> {
    pub fn get(&self, k: u32) -> Option<&Value<'de>> {
        self.0
            .iter()
            .find_map(|Entry { key, value }| (*key == k).then_some(value))
    }

    #[allow(private_bounds, private_interfaces)]
    pub fn get_and_convert<'v, T>(&'v self, k: u32) -> Result<T, T::Error>
    where
        T: ConvertFrom<Option<&'v Value<'de>>>,
    {
        T::convert_from(self.get(k))
    }
}
