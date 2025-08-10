use crate::{Value, convert::ConvertFrom, errors};

#[derive(Clone, Default)]
pub struct Entries<'de>(pub(crate) Vec<(u32, Value<'de>)>);

impl<'de> Entries<'de> {
    pub fn get(&self, k: u32) -> Option<&Value<'de>> {
        self.0
            .iter()
            .find_map(|(key, value)| (*key == k).then_some(value))
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

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(crate) fn iter(&self) -> &[(u32, Value<'de>)] {
        &self.0
    }
}
