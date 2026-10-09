//! Original browser argument admission shared by provider and host boundaries.

use std::{collections::BTreeMap, fmt};

use serde::{
    Deserialize,
    de::{MapAccess, Visitor},
};
use serde_json::Value;

/// Bounded original JSON parsed before duplicate keys can be lost.
///
/// This is syntax evidence only, never permission or semantic admission. Trusted
/// transports must supply original argument bytes, not reserialized `Value`.
/// There is deliberately no Deserialize or Value constructor for this type.
pub struct BrowserRawArguments {
    fields: BTreeMap<String, Value>,
}

impl fmt::Debug for BrowserRawArguments {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("BrowserRawArguments { .. }")
    }
}

/// Content-free raw browser argument failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum BrowserRawArgumentsError {
    /// Original argument bytes exceed the v2 request budget.
    #[error("browser request exceeds 16 KiB")]
    TooLarge,
    /// Invalid JSON, duplicate keys, non-object input or non-scalar fields.
    #[error("invalid browser argument JSON")]
    Invalid,
}

impl BrowserRawArguments {
    /// Checks original argument JSON, including escaped duplicate field names.
    /// Call only at trusted raw ingress; constructing this from reserialized JSON
    /// does not establish the integrity of the original transport.
    pub fn parse_original(raw: &str) -> Result<Self, BrowserRawArgumentsError> {
        if raw.len() > 16 * 1024 {
            return Err(BrowserRawArgumentsError::TooLarge);
        }
        let mut parser = serde_json::Deserializer::from_str(raw);
        let Fields(fields) =
            Fields::deserialize(&mut parser).map_err(|_| BrowserRawArgumentsError::Invalid)?;
        parser
            .end()
            .map_err(|_| BrowserRawArgumentsError::Invalid)?;
        Ok(Self { fields })
    }

    /// Returns immutable fields for full semantic admission. May contain secrets;
    /// never send this value to display, logs, or durable replay.
    pub fn fields(&self) -> &BTreeMap<String, Value> {
        &self.fields
    }
}

struct Fields(BTreeMap<String, Value>);

impl<'de> Deserialize<'de> for Fields {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct FieldsVisitor;
        impl<'de> Visitor<'de> for FieldsVisitor {
            type Value = Fields;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("scalar browser argument object")
            }
            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Fields, M::Error> {
                let mut fields = BTreeMap::new();
                while let Some(key) = map.next_key::<String>()? {
                    if fields.contains_key(&key) {
                        return Err(serde::de::Error::custom("duplicate field"));
                    }
                    let value = map.next_value::<Value>()?;
                    if value.is_object() || value.is_array() || value.is_null() {
                        return Err(serde::de::Error::custom("invalid field type"));
                    }
                    fields.insert(key, value);
                }
                Ok(Fields(fields))
            }
        }
        deserializer.deserialize_map(FieldsVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_lossy_ingress_without_exposing_arguments() {
        for raw in [
            r#"{"text":"secret","text":"other"}"#,
            r#"{"text":"secret","te\u0078t":"other"}"#,
            r#"{"text":null}"#,
            r#"{"text":{}}"#,
            r#"{"text":[]}"#,
            "[]",
            "{} {}",
        ] {
            assert_eq!(
                BrowserRawArguments::parse_original(raw).expect_err("reject"),
                BrowserRawArgumentsError::Invalid
            );
        }
        let input = BrowserRawArguments::parse_original(r#"{"text":"secret"}"#).expect("valid");
        assert_eq!(input.fields()["text"], "secret");
        assert!(!format!("{input:?}").contains("secret"));
        assert_eq!(
            BrowserRawArguments::parse_original(&" ".repeat(16385)).expect_err("size"),
            BrowserRawArgumentsError::TooLarge
        );
    }
}
