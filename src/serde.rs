use crate::layout::TypeId;
use serde::{Deserialize, Deserializer};
use std::borrow::Cow;
use std::collections::HashMap;

pub(crate) fn deserialize_hex_map<'de, D, V>(
    deserializer: D,
) -> Result<HashMap<TypeId, V>, D::Error>
where
    D: Deserializer<'de>,
    V: Deserialize<'de>,
{
    let map: HashMap<Cow<'de, str>, V> = HashMap::deserialize(deserializer)?;

    map.into_iter()
        .map(|(k, v)| {
            TypeId::from_str_radix(&k, 16)
                .map(|key| (key, v))
                .map_err(serde::de::Error::custom)
        })
        .collect()
}

pub(crate) fn deserialize_hex<'de, D>(deserializer: D) -> Result<usize, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    usize::from_str_radix(s, 16).map_err(serde::de::Error::custom)
}
