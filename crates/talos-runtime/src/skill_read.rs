//! Opt-in model-initiated Skill reader (ADR-092).

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{Value, json};
use talos_core::tool::{AgentTool, ToolResult};
use talos_skill::{Skill, SkillReadLimits, chunk_text, list_skill_files, read_skill_file};

/// Name of the built-in skill reader tool.
pub(crate) const SKILL_READ_TOOL_NAME: &str = "skill_read";

/// Read-only tool exposing bound Skill bodies and reference files.
pub(crate) struct SkillReadTool {
    skills: Arc<BTreeMap<String, Skill>>,
    limits: SkillReadLimits,
    description: String,
}

impl SkillReadTool {
    pub(crate) fn new(skills: Vec<Skill>, limits: SkillReadLimits) -> Self {
        let skills: BTreeMap<String, Skill> = skills
            .into_iter()
            .map(|skill| (skill.name.clone(), skill))
            .collect();
        let names = skills.keys().cloned().collect::<Vec<_>>().join(", ");
        let description = format!(
            "Read a bound Skill. Without `reference`, returns the full SKILL.md instructions and \
             the Skill's file list; read this before following a Skill. With `reference` (a path \
             relative to the Skill directory), returns that file. Long output is chunked; repeat \
             with the returned `offset` to continue. Available skills: {names}."
        );
        Self {
            skills: Arc::new(skills),
            limits,
            description,
        }
    }
}

fn render(
    skill: &Skill,
    reference: Option<&str>,
    offset: usize,
    limits: &SkillReadLimits,
) -> Result<String, String> {
    let mut out = format!("# Skill: {}\n", skill.name);
    let (title, text) = match reference {
        None => ("SKILL.md".to_string(), skill.body.clone()),
        Some(path) => (
            path.trim().to_string(),
            read_skill_file(skill, path, limits).map_err(|error| error.to_string())?,
        ),
    };
    let chunk = chunk_text(&text, offset, limits.max_chunk_bytes);
    let _ = writeln!(
        out,
        "## {title} (bytes {}-{} of {})\n{}",
        chunk.offset,
        chunk.offset + chunk.text.len(),
        chunk.total_bytes,
        chunk.text
    );
    if let Some(next) = chunk.next_offset {
        let _ = writeln!(
            out,
            "\n[truncated: call again with offset={next} to continue]"
        );
    }
    if reference.is_none() && chunk.offset == 0 {
        let listing = list_skill_files(skill, limits).map_err(|error| error.to_string())?;
        out.push_str("\n## Files\n");
        for file in &listing.files {
            let _ = writeln!(out, "- {file}");
        }
        if listing.truncated {
            out.push_str("- [listing truncated]\n");
        }
    }
    Ok(out)
}

#[async_trait]
impl AgentTool for SkillReadTool {
    fn name(&self) -> &str {
        SKILL_READ_TOOL_NAME
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "skill": {
                    "type": "string",
                    "enum": self.skills.keys().collect::<Vec<_>>(),
                    "description": "Bound Skill name."
                },
                "reference": {
                    "type": "string",
                    "description": "Optional file path relative to the Skill directory."
                },
                "offset": {
                    "type": "integer",
                    "minimum": 0,
                    "description": "Byte offset returned by a previous truncated call."
                }
            },
            "required": ["skill"],
            "additionalProperties": false
        })
    }

    fn is_read_only(&self) -> bool {
        true
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["skill", "reference"]
    }

    async fn execute(&self, input: Value) -> ToolResult {
        let Some(name) = input.get("skill").and_then(Value::as_str) else {
            return ToolResult::error("`skill` is required");
        };
        let Some(skill) = self.skills.get(name).cloned() else {
            return ToolResult::error(format!("skill '{name}' is not bound to this runtime"));
        };
        let reference = match input.get("reference") {
            None | Some(Value::Null) => None,
            Some(Value::String(path)) => Some(path.clone()),
            Some(_) => return ToolResult::error("`reference` must be a string"),
        };
        let offset = match input.get("offset") {
            None | Some(Value::Null) => 0,
            Some(value) => match value.as_u64().and_then(|v| usize::try_from(v).ok()) {
                Some(offset) => offset,
                None => return ToolResult::error("`offset` must be a non-negative integer"),
            },
        };
        let limits = self.limits;
        let outcome = tokio::task::spawn_blocking(move || {
            render(&skill, reference.as_deref(), offset, &limits)
        })
        .await;
        match outcome {
            Ok(Ok(text)) => ToolResult::success(text),
            Ok(Err(error)) => ToolResult::error(format!("skill_read failed: {error}")),
            Err(_) => ToolResult::error("skill_read failed: reader task did not complete"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    pub(crate) fn disk_skill(root: &Path, name: &str, body: &str) -> Skill {
        let dir = root.join(name);
        fs::create_dir_all(&dir).expect("skill dir");
        fs::write(dir.join("SKILL.md"), body).expect("write SKILL.md");
        Skill {
            name: name.to_string(),
            description: format!("{name} skill"),
            triggers: Vec::new(),
            body: body.to_string(),
            source_path: dir.join("SKILL.md"),
            source: Default::default(),
        }
    }

    #[tokio::test]
    async fn reads_bodies_references_and_continues_chunks() {
        let dir = tempfile::tempdir().expect("tempdir");
        let a = disk_skill(dir.path(), "alpha", "Alpha body");
        let b = disk_skill(dir.path(), "beta", &"b".repeat(30));
        fs::create_dir_all(dir.path().join("alpha/recipes")).expect("mkdir");
        fs::write(dir.path().join("alpha/recipes/r.md"), "recipe text").expect("write");
        fs::write(dir.path().join("secret.txt"), "secret").expect("write");
        let limits = SkillReadLimits {
            max_chunk_bytes: 16,
            ..SkillReadLimits::default()
        };
        let tool = SkillReadTool::new(vec![a, b], limits);

        let body = tool.execute(json!({"skill": "alpha"})).await;
        assert!(!body.is_error, "{}", body.content);
        assert!(body.content.contains("Alpha body"));
        assert!(body.content.contains("- recipes/r.md"));

        let reference = tool
            .execute(json!({"skill": "alpha", "reference": "recipes/r.md"}))
            .await;
        assert!(
            reference.content.contains("recipe text"),
            "{}",
            reference.content
        );

        let first = tool.execute(json!({"skill": "beta"})).await;
        assert!(first.content.contains("offset=16"), "{}", first.content);
        let rest = tool.execute(json!({"skill": "beta", "offset": 16})).await;
        assert!(
            rest.content.contains("bytes 16-30 of 30"),
            "{}",
            rest.content
        );
        assert!(!rest.content.contains("offset="));

        for input in [
            json!({"skill": "alpha", "reference": "../secret.txt"}),
            json!({"skill": "gamma"}),
            json!({"skill": "alpha", "offset": -1}),
            json!({}),
        ] {
            assert!(tool.execute(input).await.is_error);
        }
    }
}
