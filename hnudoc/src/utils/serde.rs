// serde 辅助

use std::str::FromStr;

use serde::{Deserialize, Deserializer};
use serde_json::Value;

// query 中空串视为 None
pub fn empty_string_as_none<'de, D, T>(
    deserializer: D,
) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: for<'a> Deserialize<'a> + FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::String(s) if s.is_empty() => Ok(None),
        Value::String(s) => {
            s.parse().map(Some).map_err(serde::de::Error::custom)
        }
        Value::Null => Ok(None),
        other => serde_json::from_value(other)
            .map(Some)
            .map_err(serde::de::Error::custom),
    }
}
