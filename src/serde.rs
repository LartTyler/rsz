use crate::layout::TypeId;
use crate::{Object, Objects, Value, Values};
use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Cow;
use std::collections::HashMap;
use std::rc::Rc;

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

impl Serialize for Objects {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.len() == 1 {
            self[0].serialize(serializer)
        } else {
            (*self).serialize(serializer)
        }
    }
}

impl Serialize for Object {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(inner) = as_transparent_collection(self) {
            inner.serialize(serializer)
        } else if let Some(inner) = as_transparent_object(self) {
            inner.serialize(serializer)
        } else {
            let mut map = serializer.serialize_map(Some(self.fields.len()))?;

            for field in &self.fields {
                if let Some(inner) = as_transparent_value(&field.value) {
                    map.serialize_entry(&field.name, &inner)?;
                } else {
                    map.serialize_entry(&field.name, &field.value)?;
                }
            }

            map.end()
        }
    }
}

fn as_transparent_collection(item: &Object) -> Option<&Values> {
    if let Value::Array(inner) = &item.fields.first_only()?.value {
        Some(inner)
    } else {
        None
    }
}

fn as_transparent_object(item: &Object) -> Option<&Value> {
    item.fields.first_only().map(|v| &v.value)
}

fn as_transparent_value(value: &Value) -> Option<Rc<Object>> {
    if let Value::Object(inner) = value
        && inner.fields.len() == 1
    {
        Some(inner.clone())
    } else {
        None
    }
}

trait FirstOnly<T> {
    fn first_only(&self) -> Option<&T>;
}

impl<T> FirstOnly<T> for Vec<T> {
    fn first_only(&self) -> Option<&T> {
        if self.len() == 1 { self.first() } else { None }
    }
}

impl Serialize for Values {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;

        for value in self.iter() {
            if let Some(inner) = as_transparent_value(value) {
                seq.serialize_element(&inner)?;
            } else {
                seq.serialize_element(value)?;
            }
        }

        seq.end()
    }
}
