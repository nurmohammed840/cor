use crate::Value;

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

    pub fn get_and_try_into<'v, T>(&'v self, k: u32) -> Result<Option<T>, T::Error>
    where
        T: TryFrom<&'v Value<'de>>,
    {
        self.get(k).map(T::try_from).transpose()
    }
}
