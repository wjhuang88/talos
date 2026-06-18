use serde_json::Value;

pub(crate) fn normalize_tool_input(name: &str, input: Value) -> Value {
    if let Value::Object(mut map) = input {
        if (name == "write" || name == "edit")
            && let Some(Value::String(path)) = map.get("path")
        {
            let cleaned = path.trim().to_string();
            let safe = cleaned.replace("..", "");
            map.insert("path".into(), Value::String(safe));
        }
        if let Some(Value::String(content)) = map.get("content") {
            let cleaned = content.trim().to_string();
            map.insert("content".into(), Value::String(cleaned));
        }
        if let Some(Value::String(cmd)) = map.get("command") {
            map.insert("command".into(), Value::String(cmd.trim().to_string()));
        }
        Value::Object(map)
    } else {
        input
    }
}
