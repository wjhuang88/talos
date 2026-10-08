//! Raw ingress validation. No parsed-Value constructor is exposed.

use std::{collections::BTreeMap, fmt};

use serde::{
    Deserialize,
    de::{MapAccess, Visitor},
};
use serde_json::Value;

use super::{BrowserOperation, BrowserSessionRef};

/// Bounded validation failure; never includes supplied arguments or parser diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum BrowserRequestError {
    /// The raw request exceeds the protocol budget.
    #[error("browser request exceeds 16 KiB")]
    TooLarge,
    /// JSON, fields, version, or semantic bounds are invalid.
    #[error("invalid browser v2 request")]
    Invalid,
}

/// Validated wire request, not a permission grant or lifecycle admission ticket.
///
/// Debug deliberately omits arguments, including fill values and URLs.
pub struct BrowserRequest {
    operation: BrowserOperation,
    fields: BTreeMap<String, Value>,
}

impl fmt::Debug for BrowserRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BrowserRequest")
            .field("operation", &self.operation)
            .finish_non_exhaustive()
    }
}

impl BrowserRequest {
    /// Parses original tool-argument JSON before any lossy conversion to Value.
    ///
    /// The caller must supply the original bytes, never reserialized parsed JSON.
    /// This validates syntax only; trusted host context and permission remain required.
    pub fn parse_raw(raw: &str) -> Result<Self, BrowserRequestError> {
        if raw.len() > 16 * 1024 {
            return Err(BrowserRequestError::TooLarge);
        }
        let mut parser = serde_json::Deserializer::from_str(raw);
        let Fields(fields) =
            Fields::deserialize(&mut parser).map_err(|_| BrowserRequestError::Invalid)?;
        parser.end().map_err(|_| BrowserRequestError::Invalid)?;
        Self::validate(fields).ok_or(BrowserRequestError::Invalid)
    }

    /// Returns the exact admitted operation vocabulary.
    pub fn operation(&self) -> BrowserOperation {
        self.operation
    }

    /// Returns validated fields for trusted host dispatch. These contain sensitive input.
    pub fn fields(&self) -> &BTreeMap<String, Value> {
        &self.fields
    }

    fn validate(fields: BTreeMap<String, Value>) -> Option<Self> {
        if fields.get("protocolVersion")?.as_u64()? != 2 {
            return None;
        }
        let operation: BrowserOperation =
            serde_json::from_value(fields.get("operation")?.clone()).ok()?;
        use BrowserOperation::*;
        let (required, optional): (&[&str], &[&str]) = match operation {
            Open => (&["tabRef", "url"], &["visibility"]),
            Read | Snapshot | CurrentUrl | Screenshot => (&["tabRef", "frameRef"], &[]),
            Click | Hover | Check | Uncheck | WaitForElement => {
                (&["tabRef", "frameRef", "snapshotRef", "elementRef"], &[])
            }
            Fill => (
                &["tabRef", "frameRef", "snapshotRef", "elementRef", "text"],
                &[],
            ),
            Select => (
                &["tabRef", "frameRef", "snapshotRef", "elementRef", "option"],
                &[],
            ),
            Press => (&["tabRef", "frameRef", "key"], &[]),
            Scroll => (&["tabRef", "frameRef", "direction"], &["amount"]),
            WaitMilliseconds => (&["milliseconds"], &[]),
            TabClose | TabSwitch | FrameTree => (&["tabRef"], &[]),
            TabNew | TabList | WindowClose => (&[], &[]),
        };
        if required.iter().any(|name| !fields.contains_key(*name)) {
            return None;
        }
        for (name, value) in &fields {
            if !["protocolVersion", "operation"].contains(&name.as_str())
                && !required.contains(&name.as_str())
                && !optional.contains(&name.as_str())
            {
                return None;
            }
            match name.as_str() {
                "protocolVersion" | "operation" => (),
                "tabRef" | "frameRef" | "snapshotRef" | "elementRef" => {
                    BrowserSessionRef::try_new(value.as_str()?).ok()?;
                }
                "text" => {
                    bounded_text(value, 0, 4096)?;
                }
                "option" => {
                    bounded_text(value, 1, 1024)?;
                }
                "url" => {
                    let text = bounded_text(value, 1, 4096)?;
                    // Reject URL-parser normalization of whitespace and backslashes.
                    if text
                        .chars()
                        .any(|c| c.is_control() || c.is_whitespace() || c == '\\')
                    {
                        return None;
                    }
                    let url = reqwest::Url::parse(text).ok()?;
                    if !matches!(url.scheme(), "http" | "https")
                        || url.host_str().is_none()
                        || !url.username().is_empty()
                        || url.password().is_some()
                    {
                        return None;
                    }
                    // Empty userinfo is still credential syntax.
                    if text
                        .split_once("://")?
                        .1
                        .split(['/', '?', '#'])
                        .next()?
                        .contains('@')
                    {
                        return None;
                    }
                }
                "visibility" => {
                    member(value, &["background", "foreground"])?;
                }
                "direction" => {
                    member(value, &["up", "down", "left", "right"])?;
                }
                "key" => {
                    member(
                        value,
                        &[
                            "Enter",
                            "Tab",
                            "Escape",
                            "Space",
                            "Backspace",
                            "Delete",
                            "ArrowUp",
                            "ArrowDown",
                            "ArrowLeft",
                            "ArrowRight",
                            "Home",
                            "End",
                            "PageUp",
                            "PageDown",
                        ],
                    )?;
                }
                "amount" => {
                    if !(1..=5000).contains(&value.as_u64()?) {
                        return None;
                    }
                }
                "milliseconds" => {
                    if !(1..=30000).contains(&value.as_u64()?) {
                        return None;
                    }
                }
                _ => return None,
            }
        }
        Some(Self { operation, fields })
    }
}

fn bounded_text(value: &Value, min: usize, max: usize) -> Option<&str> {
    let text = value.as_str()?;
    ((min..=max).contains(&text.len()) && !text.contains('\0')).then_some(text)
}

fn member(value: &Value, choices: &[&str]) -> Option<()> {
    choices.contains(&value.as_str()?).then_some(())
}

struct Fields(BTreeMap<String, Value>);

impl<'de> Deserialize<'de> for Fields {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct FieldsVisitor;
        impl<'de> Visitor<'de> for FieldsVisitor {
            type Value = Fields;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("closed scalar browser request")
            }
            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Fields, M::Error> {
                let mut fields = BTreeMap::new();
                while let Some(key) = map.next_key::<String>()? {
                    if fields.contains_key(&key) {
                        return Err(serde::de::Error::custom("duplicate field"));
                    }
                    let value = map.next_value::<Value>()?;
                    // All v2 request fields are scalar; reject every nested object/array.
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
    fn rejects_lossy_ingress_and_non_closed_requests() {
        for raw in [
            r#"{"protocolVersion":2,"operation":"tab-new","operation":"tab-list"}"#,
            r#"{"protocolVersion":2,"operation":"tab-new","oper\u0061tion":"tab-list"}"#,
            r#"{"protocolVersion":2,"operation":"tab-new","script":"secret"}"#,
            r#"{"protocolVersion":1,"operation":"tab-new"}"#,
            r#"{"protocolVersion":2,"operation":"tab-new"} {}"#,
            r#"{"protocolVersion":2,"operation":"open","tabRef":"t","url":null}"#,
        ] {
            assert!(BrowserRequest::parse_raw(raw).is_err(), "{raw}");
        }
    }

    #[test]
    fn rejects_numeric_coercion_and_credential_urls() {
        for value in ["0", "30001", "1.0", "-1", "null", "\"1\""] {
            assert!(BrowserRequest::parse_raw(&format!(r#"{{"protocolVersion":2,"operation":"wait-milliseconds","milliseconds":{value}}}"#)).is_err());
        }
        for url in [
            "https://user:secret@example.com",
            "https://@example.com",
            "javascript:alert(1)",
            "https://example.com\\other",
        ] {
            let raw =
                serde_json::json!({"protocolVersion":2,"operation":"open","tabRef":"t","url":url})
                    .to_string();
            assert!(BrowserRequest::parse_raw(&raw).is_err());
        }
    }

    #[test]
    fn enforces_utf8_byte_budget_and_redacts_debug() {
        let mut input = serde_json::json!({"protocolVersion":2,"operation":"fill","tabRef":"t","frameRef":"f","snapshotRef":"s","elementRef":"e","text":"secret-value"});
        let request = BrowserRequest::parse_raw(&input.to_string()).unwrap();
        assert!(!format!("{request:?}").contains("secret-value"));
        input["text"] = Value::String("界".repeat(1366));
        assert!(BrowserRequest::parse_raw(&input.to_string()).is_err());
        assert_eq!(
            BrowserRequest::parse_raw(&" ".repeat(16385)).unwrap_err(),
            BrowserRequestError::TooLarge
        );
    }
}
