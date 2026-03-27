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
        if let Some(inner) = as_transparent_object(self) {
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::Field;

    type Result = std::result::Result<(), anyhow::Error>;

    #[test]
    fn transparent_object_to_json() -> Result {
        let actual = Value::Object(Rc::new(Object {
            name: "Example".to_string(),
            fields: vec![Field {
                name: "Field".to_string(),
                value: Value::Array(Values(vec![Value::Boolean(true)])),
            }],
        }));

        assert_eq!(serde_json::to_string(&actual)?, "[true]");

        let actual = Value::Object(Rc::new(Object {
            name: "Top Level".to_string(),
            fields: vec![Field {
                name: "Top Field".to_string(),
                value: Value::Object(Rc::new(Object {
                    name: "Inner".to_string(),
                    fields: vec![Field {
                        name: "Inner Field".to_string(),
                        value: Value::S32(-100),
                    }],
                })),
            }],
        }));

        assert_eq!(serde_json::to_string(&actual)?, "-100");

        let actual = Value::Object(Rc::new(Object {
            name: "Example".to_string(),
            fields: vec![
                Field {
                    name: "Field 1".to_string(),
                    value: Value::Array(Values(vec![Value::Boolean(true)])),
                },
                Field {
                    name: "Field 2".to_string(),
                    value: Value::Array(Values(vec![Value::Boolean(false)])),
                },
            ],
        }));

        assert_eq!(
            serde_json::to_string(&actual)?,
            r#"{"Field 1":[true],"Field 2":[false]}"#
        );

        let actual = Value::Object(Rc::new(Object {
            name: "Example 2".to_string(),
            fields: vec![
                Field {
                    name: "Field 1".to_string(),
                    value: Value::Boolean(true),
                },
                Field {
                    name: "Field 2".to_string(),
                    value: Value::Boolean(false),
                },
            ],
        }));

        assert_eq!(
            serde_json::to_string(&actual)?,
            r#"{"Field 1":true,"Field 2":false}"#
        );

        Ok(())
    }
}
