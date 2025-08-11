use crate::{Value, convert::ConvertFrom, errors};

#[derive(Clone, Default)]
pub struct Entries<'de>(Vec<(u16, Value<'de>)>);

impl<'de> Entries<'de> {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn get(&self, k: u16) -> Option<&Value<'de>> {
        self.0
            .iter()
            .find_map(|(key, value)| (*key == k).then_some(value))
    }

    #[inline]
    pub fn insert(&mut self, key: u16, value: Value<'de>) {
        self.0.push((key, value));
    }

    pub fn get_and_convert<'v, T>(&'v self, k: u16) -> Result<T, errors::ConvertError>
    where
        T: ConvertFrom<Option<&'v Value<'de>>>,
    {
        T::convert_from(self.get(k)).map_err(|mut err| {
            err.key = Some(k);
            err
        })
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn iter(&self) -> std::slice::Iter<'_, (u16, Value<'de>)> {
        self.0.iter()
    }
}
