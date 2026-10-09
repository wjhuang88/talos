//! Raw ingress validation. No parsed-Value constructor is exposed.

use std::{collections::BTreeMap, fmt};

use serde_json::Value;
use talos_core::tool::{BrowserRawArguments, BrowserRawArgumentsError};

use super::{BrowserCommand, BrowserOperation, BrowserSessionRef};

/// Bounded validation failure; never includes supplied arguments or parser diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum BrowserRequestError {
    /// The raw request exceeds the protocol budget.
    #[error("browser request exceeds 16 KiB")]
    TooLarge,
    /// An integer protocol version other than v2 was supplied.
    #[error("unsupported browser protocol version")]
    UnsupportedVersion,
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
    command: BrowserCommand,
}

impl fmt::Debug for BrowserRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BrowserRequest")
            .field("operation", &self.operation)
            .finish_non_exhaustive()
    }
}

impl BrowserRequest {
    /// Publishes the closed v2 wire schema, including optional normalized defaults.
    /// UTF-8 byte budgets and URL semantics are additionally checked during raw admission.
    /// An unexpected generator layout yields the deny-all schema, never a permissive fallback.
    pub fn schema() -> Value {
        let mut schema = schemars::schema_for!(BrowserCommand).to_value();
        if let Some(branches) = schema.get_mut("oneOf").and_then(Value::as_array_mut) {
            for branch in branches {
                if !Self::wire_branch(branch) {
                    return Value::Bool(false);
                }
            }
        } else if let Some(branches) = schema.get_mut("anyOf").and_then(Value::as_array_mut) {
            for branch in branches {
                if !Self::wire_branch(branch) {
                    return Value::Bool(false);
                }
            }
        } else {
            return Value::Bool(false);
        }
        schema
    }

    fn wire_branch(branch: &mut Value) -> bool {
        let Some(properties) = branch.get_mut("properties").and_then(Value::as_object_mut) else {
            return false;
        };
        properties.insert(
            "protocolVersion".into(),
            serde_json::json!({"type":"integer", "const":2}),
        );
        for (field, min, max) in [("text", 0, 4096), ("option", 1, 1024), ("url", 1, 4096)] {
            if let Some(value) = properties.get_mut(field) {
                value["minLength"] = Value::from(min);
                value["maxLength"] = Value::from(max);
                value["pattern"] = Value::from("^[^\\u0000]*$");
            }
        }
        for (field, max) in [("amount", 5000), ("milliseconds", 30000)] {
            if let Some(value) = properties.get_mut(field) {
                value["minimum"] = Value::from(1);
                value["maximum"] = Value::from(max);
            }
        }
        if let Some(url) = properties.get_mut("url") {
            url["pattern"] = Value::from(r"^[hH][tT][tT][pP][sS]?://[^/?#@\s\\]+([/?#][^\s\\]*)?$");
            url["not"] = serde_json::json!({"pattern":r"[\u0000-\u0020\u007f\\]"});
            url["format"] = Value::from("uri");
        }
        if let Some(value) = properties.get_mut("amount") {
            value["default"] = Value::from(500);
        }
        if let Some(value) = properties.get_mut("visibility") {
            value["default"] = Value::from("background");
        }
        if let Some(required) = branch.get_mut("required").and_then(Value::as_array_mut) {
            required.retain(|field| field != "amount" && field != "visibility");
            required.push(Value::from("protocolVersion"));
        } else {
            return false;
        }
        true
    }

    /// Parses original tool-argument JSON before any lossy conversion to Value.
    ///
    /// The caller must supply the original bytes, never reserialized parsed JSON.
    /// This validates syntax only; trusted host context and permission remain required.
    pub fn parse_raw(raw: &str) -> Result<Self, BrowserRequestError> {
        let arguments = BrowserRawArguments::parse_original(raw).map_err(|error| match error {
            BrowserRawArgumentsError::TooLarge => BrowserRequestError::TooLarge,
            BrowserRawArgumentsError::Invalid => BrowserRequestError::Invalid,
        })?;
        Self::from_original(&arguments)
    }

    /// Applies the full v2 semantic contract to original-ingress syntax evidence.
    /// The evidence carries no lifecycle identity or execution permission.
    pub fn from_original(arguments: &BrowserRawArguments) -> Result<Self, BrowserRequestError> {
        if arguments
            .fields()
            .get("protocolVersion")
            .and_then(Value::as_u64)
            .is_some_and(|version| version != 2)
        {
            return Err(BrowserRequestError::UnsupportedVersion);
        }
        Self::validate(arguments.fields().clone()).ok_or(BrowserRequestError::Invalid)
    }

    /// Returns the exact admitted operation vocabulary.
    pub fn operation(&self) -> BrowserOperation {
        self.operation
    }

    /// Returns validated fields for trusted host dispatch. These contain sensitive input.
    pub fn fields(&self) -> &BTreeMap<String, Value> {
        &self.fields
    }

    /// Returns the typed normalized command. This is not a permission capability.
    pub fn command(&self) -> &BrowserCommand {
        &self.command
    }

    fn validate(mut fields: BTreeMap<String, Value>) -> Option<Self> {
        // JSON Schema integer describes mathematical values, including 2.0 and 2e0.
        // Normalize bounded integral numbers once before constructing the typed command.
        for field in ["protocolVersion", "amount", "milliseconds"] {
            if let Some(value) = fields.get_mut(field) {
                let number = value.as_f64()?;
                if !number.is_finite()
                    || number.fract() != 0.0
                    || !(0.0..=30000.0).contains(&number)
                {
                    return None;
                }
                *value = Value::from(number as u64);
            }
        }
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
        match operation {
            Open => {
                fields
                    .entry("visibility".into())
                    .or_insert(Value::String("background".into()));
            }
            Scroll => {
                fields.entry("amount".into()).or_insert(Value::from(500));
            }
            _ => (),
        }
        let mut payload: serde_json::Map<String, Value> = fields.clone().into_iter().collect();
        payload.remove("protocolVersion");
        let command = serde_json::from_value(Value::Object(payload)).ok()?;
        Some(Self {
            operation,
            fields,
            command,
        })
    }
}

fn bounded_text(value: &Value, min: usize, max: usize) -> Option<&str> {
    let text = value.as_str()?;
    ((min..=max).contains(&text.len()) && !text.contains('\0')).then_some(text)
}

fn member(value: &Value, choices: &[&str]) -> Option<()> {
    choices.contains(&value.as_str()?).then_some(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_operation_has_closed_wire_shape_and_typed_payload() {
        let schema = BrowserRequest::schema();
        let validator = jsonschema::validator_for(&schema).expect("valid browser wire schema");
        let branches = schema
            .get("oneOf")
            .or_else(|| schema.get("anyOf"))
            .expect("valid test fixture")
            .as_array()
            .expect("valid test fixture");
        assert_eq!(branches.len(), 21);
        for branch in branches {
            assert_eq!(branch["additionalProperties"], false);
            let props = branch["properties"]
                .as_object()
                .expect("valid test fixture");
            let mut input = serde_json::Map::new();
            for field in branch["required"].as_array().expect("valid test fixture") {
                let field = field.as_str().expect("valid test fixture");
                let value = match field {
                    "protocolVersion" => Value::from(2),
                    "operation" => props[field]["const"].clone(),
                    "url" => Value::from("https://example.com/path"),
                    "key" => Value::from("Enter"),
                    "direction" => Value::from("down"),
                    "milliseconds" => Value::from(1),
                    _ => Value::from("ref_1"),
                };
                input.insert(field.into(), value);
            }
            let valid = Value::Object(input);
            assert!(validator.is_valid(&valid), "{valid}");
            let request =
                BrowserRequest::parse_raw(&valid.to_string()).expect("valid test fixture");
            assert_eq!(request.command().operation(), request.operation());
            for field in branch["required"].as_array().expect("valid test fixture") {
                let mut missing = valid.clone();
                missing
                    .as_object_mut()
                    .expect("valid test fixture")
                    .remove(field.as_str().expect("valid test fixture"));
                assert!(BrowserRequest::parse_raw(&missing.to_string()).is_err());
                assert!(!validator.is_valid(&missing));
            }
            let mut extra = valid.clone();
            extra["unexpected"] = Value::from("forbidden");
            assert!(BrowserRequest::parse_raw(&extra.to_string()).is_err());
            assert!(!validator.is_valid(&extra));
            for field in props.keys() {
                let mut null = valid.clone();
                null[field] = Value::Null;
                assert!(BrowserRequest::parse_raw(&null.to_string()).is_err());
                assert!(!validator.is_valid(&null));
            }
        }
    }

    #[test]
    fn defaults_have_one_normalized_representation() {
        let implicit = BrowserRequest::parse_raw(r#"{"protocolVersion":2,"operation":"scroll","tabRef":"t","frameRef":"f","direction":"down"}"#).expect("valid test fixture");
        let explicit = BrowserRequest::parse_raw(r#"{"protocolVersion":2,"operation":"scroll","tabRef":"t","frameRef":"f","direction":"down","amount":500}"#).expect("valid test fixture");
        assert_eq!(implicit.fields(), explicit.fields());
        assert!(matches!(
            implicit.command(),
            BrowserCommand::Scroll { amount: 500, .. }
        ));
    }

    #[test]
    fn schema_and_admission_reject_abusive_scalar_values() {
        let schema = BrowserRequest::schema();
        let validator = jsonschema::options()
            .should_validate_formats(true)
            .build(&schema)
            .expect("valid schema");
        for url in [
            "javascript:alert(1)",
            "https://user:secret@example.com",
            "https://@example.com",
            "https://example.com\\other",
            "https://example.com/has space",
            "https://example.com/\n",
        ] {
            let input =
                serde_json::json!({"protocolVersion":2,"operation":"open","tabRef":"t","url":url});
            assert!(!validator.is_valid(&input), "schema accepted {input}");
            assert!(BrowserRequest::parse_raw(&input.to_string()).is_err());
        }
        for token in ["", "ref\n", "has space", "é", "x/y"] {
            let input =
                serde_json::json!({"protocolVersion":2,"operation":"tab-close","tabRef":token});
            assert!(!validator.is_valid(&input));
            assert!(BrowserRequest::parse_raw(&input.to_string()).is_err());
        }
        for number in [
            serde_json::json!(0),
            serde_json::json!(30001),
            serde_json::json!(-1),
            serde_json::json!(1.5),
            serde_json::json!("1"),
        ] {
            let input = serde_json::json!({"protocolVersion":2,"operation":"wait-milliseconds","milliseconds":number});
            assert!(!validator.is_valid(&input));
            assert!(BrowserRequest::parse_raw(&input.to_string()).is_err());
        }
        for number in [
            serde_json::json!(1),
            serde_json::json!(1.0),
            serde_json::json!(30000.0),
        ] {
            let input = serde_json::json!({"protocolVersion":2.0,"operation":"wait-milliseconds","milliseconds":number});
            assert!(validator.is_valid(&input));
            assert!(BrowserRequest::parse_raw(&input.to_string()).is_ok());
        }
    }

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
        for value in ["0", "30001", "1.5", "-1", "null", "\"1\""] {
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
        let request = BrowserRequest::parse_raw(&input.to_string()).expect("valid test fixture");
        assert!(!format!("{request:?}").contains("secret-value"));
        input["text"] = Value::String("界".repeat(1366));
        assert!(BrowserRequest::parse_raw(&input.to_string()).is_err());
        assert_eq!(
            BrowserRequest::parse_raw(&" ".repeat(16385))
                .expect_err("invalid test fixture must be rejected"),
            BrowserRequestError::TooLarge
        );
    }
}
