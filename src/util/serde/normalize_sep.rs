use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    for<'a> Serde<&'a T>: Serialize,
    S: Serializer,
{
    Serde(value).serialize(serializer)
}

pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    Serde<T>: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Serde::deserialize(deserializer).map(|v| v.0)
}

pub struct Serde<T>(T);

impl Serialize for Serde<&'_ String> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Serde<String> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let path = String::deserialize(deserializer)?;
        Ok(Serde(crate::util::path::normalize_sep(&path).into_owned()))
    }
}
