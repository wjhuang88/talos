use std::collections::BTreeMap;
use std::collections::HashMap;

use talos_core::message::{SystemCacheMarker, SystemCacheType};
use talos_plugin::{HookContext, HookEvent, HookOutcome, HookRegistry};
use talos_skill::SkillIndex;

use super::assets::{DEFAULT_IDENTITY, TOOL_CALLING_STRICT};
use super::sections::{PromptSection, PromptSectionKind};
use super::types::{ActivatedSkillContext, CacheMarker, CacheType, ContextFile, ToolDescription};

/// Builder for assembling a system prompt from multiple components.
///
/// The builder uses a fluent API to configure each component of the system
/// prompt. Components are assembled in a fixed order optimized for LLM
/// provider caching: stable sections first, then semi-stable, then dynamic.
///
/// # Component Order
///
/// 1. Identity (or custom prompt if provided)
/// 2. Tool descriptions
/// 3. Skill index (Level 0)
/// 4. Context files (AGENTS.md)
/// 5. User preferences
/// 6. Append prompt (if provided)
///
/// # Example
///
/// ```
/// use talos_agent::prompt::SystemPromptBuilder;
///
/// let prompt = SystemPromptBuilder::new()
///     .with_user_preferences("Always use British English.".into())
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct SystemPromptBuilder {
    /// Agent identity and role instructions.
    identity: String,
    /// Tool names and descriptions.
    tools: Vec<ToolDescription>,
    /// Level 0 skill index (name + description).
    skill_index: Vec<SkillIndex>,
    /// Explicitly activated Level 1/2 Skill content.
    activated_skill: Option<ActivatedSkillContext>,
    /// AGENTS.md and other context file contents.
    context_files: Vec<ContextFile>,
    /// User-specific instructions.
    user_preferences: String,
    /// Overrides the default identity when provided.
    custom_prompt: Option<String>,
    /// Appended to the end of the prompt when provided.
    append_prompt: Option<String>,
    /// Bounded memory injection section (advisory, never authoritative).
    memory_section: Option<String>,
    /// Bounded session todo section (advisory orchestration context).
    todo_section: Option<String>,
    tool_call_format: &'static str,
    /// Runtime template values used for `{{slot}}` substitution.
    template_vars: HashMap<String, String>,
}

impl SystemPromptBuilder {
    /// Creates a new builder with the default identity and no other components.
    ///
    /// All optional components start empty. Use the builder methods to
    /// configure tools, skills, context files, and other components.
    #[must_use]
    pub fn new() -> Self {
        let mut template_vars = HashMap::new();
        template_vars.insert(
            "workspace_info".to_string(),
            "Workspace information unavailable.".to_string(),
        );
        template_vars.insert(
            "model_info".to_string(),
            "Provider model metadata unavailable.".to_string(),
        );

        Self {
            identity: DEFAULT_IDENTITY.to_string(),
            tools: Vec::new(),
            skill_index: Vec::new(),
            activated_skill: None,
            context_files: Vec::new(),
            user_preferences: String::new(),
            custom_prompt: None,
            append_prompt: None,
            memory_section: None,
            todo_section: None,
            tool_call_format: "",
            template_vars,
        }
    }

    pub fn with_strict_tool_format(mut self) -> Self {
        self.tool_call_format = TOOL_CALLING_STRICT;
        self
    }

    pub fn with_tool_format(mut self, format: &'static str) -> Self {
        self.tool_call_format = format;
        self
    }

    /// Sets a template slot value for `{{slot}}` substitution.
    #[must_use]
    pub fn with_template_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.template_vars.insert(key.into(), value.into());
        self
    }

    /// Sets workspace information for the default identity template.
    #[must_use]
    pub fn with_workspace_info(self, value: impl Into<String>) -> Self {
        self.with_template_var("workspace_info", value)
    }

    /// Sets model information for the default identity template.
    #[must_use]
    pub fn with_model_info(self, value: impl Into<String>) -> Self {
        self.with_template_var("model_info", value)
    }

    /// Sets the tool descriptions for inclusion in the system prompt.
    ///
    /// Tools are sorted alphabetically by name to ensure stable ordering
    /// across turns, maximizing cache hit rates.
    #[must_use]
    pub fn with_tools(mut self, tools: Vec<ToolDescription>) -> Self {
        self.tools = tools;
        self
    }

    /// Sets the skill index for inclusion in the system prompt.
    ///
    /// Only Level 0 metadata (name, description, triggers) is included.
    /// Full skill bodies are not loaded at this stage.
    #[must_use]
    pub fn with_skill_index(mut self, skills: Vec<SkillIndex>) -> Self {
        self.skill_index = skills;
        self
    }

    /// Sets the explicitly activated Skill context.
    ///
    /// This content is cacheable after activation. The owning [`Agent`](crate::Agent)
    /// invalidates the stable-prefix cache whenever activation changes.
    #[must_use]
    pub fn with_activated_skill(mut self, skill: Option<ActivatedSkillContext>) -> Self {
        self.activated_skill = skill;
        self
    }

    /// Sets the context files for inclusion in the system prompt.
    ///
    /// Typically loaded from `AGENTS.md` files via [`ContextLoader`].
    ///
    /// [`ContextLoader`]: crate::context::ContextLoader
    #[must_use]
    pub fn with_context_files(mut self, files: Vec<ContextFile>) -> Self {
        self.context_files = files;
        self
    }

    /// Sets user-specific instructions for inclusion in the system prompt.
    #[must_use]
    pub fn with_user_preferences(mut self, prefs: String) -> Self {
        self.user_preferences = prefs;
        self
    }

    /// Sets a bounded memory section for inclusion in the system prompt.
    ///
    /// Memory is advisory only — never authoritative over session context.
    /// When `None`, no memory section is injected.
    #[must_use]
    pub fn with_memory_section(mut self, section: Option<String>) -> Self {
        self.memory_section = section;
        self
    }

    /// Sets a bounded session todo section for inclusion in the dynamic prompt suffix.
    ///
    /// Todo context is advisory orchestration state. It is intentionally not part of the
    /// stable cached prefix because it can change between turns.
    #[must_use]
    pub fn with_todo_section(mut self, section: Option<String>) -> Self {
        self.todo_section = section;
        self
    }

    /// Sets a custom prompt that replaces the default identity.
    ///
    /// When provided, the custom prompt is used instead of the default
    /// identity. The rest of the prompt (tools, skills, etc.) is still
    /// assembled normally.
    #[must_use]
    pub fn with_custom_prompt(mut self, prompt: String) -> Self {
        self.custom_prompt = Some(prompt);
        self
    }

    /// Sets an append prompt that is added at the end of the system prompt.
    ///
    /// The append prompt is always placed last, after all other components.
    #[must_use]
    pub fn with_append_prompt(mut self, prompt: String) -> Self {
        self.append_prompt = Some(prompt);
        self
    }

    /// Clears the append prompt, removing any previously set value.
    pub fn clear_append_prompt(&mut self) {
        self.append_prompt = None;
    }

    /// Sets the append prompt to an optional value.
    ///
    /// Use `None` to clear the append prompt, or `Some(prompt)` to set it.
    pub fn set_append_prompt_opt(&mut self, prompt: Option<String>) {
        self.append_prompt = prompt;
    }

    fn render_template(&self, template: &str, extra_vars: &[(&str, String)]) -> String {
        let mut rendered = template.to_string();
        let mut vars = self.template_vars.clone();
        for (key, value) in extra_vars {
            vars.insert((*key).to_string(), value.clone());
        }

        for (key, value) in vars {
            rendered = rendered.replace(&format!("{{{{{key}}}}}"), &value);
        }
        rendered
    }

    fn current_datetime() -> String {
        let seconds = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0);
        format!("unix_seconds={seconds}")
    }

    fn tool_protocol_hint(&self) -> String {
        if self.tool_call_format.is_empty() {
            "Native tool calling is enabled. Use provider-native tool calls; do not emit textual tool-call JSON unless the provider requires a fallback.".to_string()
        } else {
            self.tool_call_format.trim().to_string()
        }
    }

    fn prompt_sections(&self) -> Vec<PromptSection> {
        let mut sections: Vec<PromptSection> = Vec::new();

        let stable_vars = [("tool_protocol_hint", self.tool_protocol_hint())];

        let identity = if let Some(ref custom) = self.custom_prompt {
            self.render_template(custom, &stable_vars)
        } else {
            self.render_template(&self.identity, &stable_vars)
        };
        sections.push(PromptSection {
            text: format!("# Identity\n{identity}\n"),
            kind: PromptSectionKind::Cacheable,
        });

        if self.tools.is_empty() {
            sections.push(PromptSection {
                text: String::from("# Tools\nNo tools available.\n"),
                kind: PromptSectionKind::Cacheable,
            });
        } else {
            let mut families: BTreeMap<_, Vec<&ToolDescription>> = BTreeMap::new();
            for tool in &self.tools {
                families.entry(tool.family).or_default().push(tool);
            }

            sections.push(PromptSection {
                text: String::from("# Tools\nTool definitions are grouped by stable family.\n"),
                kind: PromptSectionKind::Cacheable,
            });

            for (family, mut sorted_tools) in families {
                sorted_tools.sort_by(|a, b| a.name.cmp(&b.name));
                let mut tools_section = format!("# Tool Family: {family:?}\n");
                for tool in sorted_tools {
                    tools_section.push_str(&format!("## {}\n{}\n", tool.name, tool.description));
                    if let Some(props) = tool.parameters.get("properties")
                        && let Some(required) = tool.parameters.get("required")
                    {
                        let req_list: Vec<&str> = required
                            .as_array()
                            .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
                            .unwrap_or_default();
                        let mut param_parts = Vec::new();
                        for (key, val) in props.as_object().unwrap_or(&serde_json::Map::new()) {
                            let desc = val
                                .get("description")
                                .and_then(|d| d.as_str())
                                .unwrap_or("");
                            let ptype = val.get("type").and_then(|t| t.as_str()).unwrap_or("any");
                            let req = if req_list.contains(&key.as_str()) {
                                "required"
                            } else {
                                "optional"
                            };
                            param_parts
                                .push(format!("  - {} ({}): {} [{}]", key, ptype, desc, req));
                        }
                        if !param_parts.is_empty() {
                            tools_section.push_str("Parameters:\n");
                            tools_section.push_str(&param_parts.join("\n"));
                            tools_section.push_str("\n\n");
                        }
                    }
                    tools_section.push('\n');
                }
                sections.push(PromptSection {
                    text: tools_section,
                    kind: PromptSectionKind::Cacheable,
                });
            }
        }

        if self.skill_index.is_empty() {
            sections.push(PromptSection {
                text: String::from("# Skills\nNo skills available.\n"),
                kind: PromptSectionKind::Cacheable,
            });
        } else {
            let mut skills_section = String::from("# Skills\n");
            for skill in &self.skill_index {
                skills_section.push_str(&format!("- **{}**: {}\n", skill.name, skill.description));
            }
            skills_section.push('\n');
            sections.push(PromptSection {
                text: skills_section,
                kind: PromptSectionKind::Cacheable,
            });
        }

        if let Some(ref skill) = self.activated_skill {
            sections.push(PromptSection {
                text: format!(
                    "# Activated Skill: {}\n{}\n",
                    skill.name.trim(),
                    skill.content.trim()
                ),
                kind: PromptSectionKind::Cacheable,
            });
        }

        if self.context_files.is_empty() {
            sections.push(PromptSection {
                text: String::from("# Context\nNo context files loaded.\n"),
                kind: PromptSectionKind::Dynamic,
            });
        } else {
            let mut context_section = String::from("# Context\n");
            for file in &self.context_files {
                context_section.push_str(&format!("--- {} ---\n{}\n\n", file.path, file.content));
            }
            sections.push(PromptSection {
                text: context_section,
                kind: PromptSectionKind::Dynamic,
            });
        }

        if let Some(ref memory) = self.memory_section {
            sections.push(PromptSection {
                text: format!("# Memory\n{memory}\n"),
                kind: PromptSectionKind::Dynamic,
            });
        }

        if let Some(ref todos) = self.todo_section {
            sections.push(PromptSection {
                text: format!("# Session Todos\n{todos}\n"),
                kind: PromptSectionKind::Dynamic,
            });
        }

        if !self.user_preferences.is_empty() {
            sections.push(PromptSection {
                text: format!("# User Preferences\n{}\n", self.user_preferences),
                kind: PromptSectionKind::Dynamic,
            });
        }

        let runtime_section = self.render_template(
            "# Runtime Context\nCurrent datetime: {{datetime}}\n",
            &[("datetime", Self::current_datetime())],
        );
        sections.push(PromptSection {
            text: runtime_section,
            kind: PromptSectionKind::Dynamic,
        });

        if let Some(ref append) = self.append_prompt {
            sections.push(PromptSection {
                text: format!("# Additional Instructions\n{append}\n"),
                kind: PromptSectionKind::Dynamic,
            });
        }

        sections
    }

    /// Assembles and returns the final system prompt as a string.
    ///
    /// Components are assembled in the optimal order for caching:
    /// 1. Identity (or custom prompt if provided)
    /// 2. Tools (sorted by name)
    /// 3. Skill index
    /// 4. Context files
    /// 5. User preferences
    /// 6. Append prompt (if provided)
    ///
    /// Empty components are omitted from the output.
    #[must_use]
    pub fn build(&self) -> String {
        self.prompt_sections()
            .into_iter()
            .map(|section| section.text)
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Assembles the system prompt and emits the `OnSystemPromptBuilt` hook.
    ///
    /// Handlers may replace the final prompt by returning
    /// [`talos_plugin::HookResult::Modify`]. `Skip` leaves the prompt unchanged.
    #[allow(dead_code)]
    pub(crate) async fn build_with_hooks(
        &self,
        hook_registry: &HookRegistry,
        ctx: &HookContext,
    ) -> Result<(String, Vec<SystemCacheMarker>), String> {
        let (prompt, markers) = self.build_with_cache_markers();
        let original_prompt = prompt.clone();
        let outcome = hook_registry
            .dispatch(ctx, HookEvent::OnSystemPromptBuilt { prompt: &prompt })
            .await;

        match outcome {
            HookOutcome::Continue(HookEvent::OnSystemPromptBuilt { prompt })
            | HookOutcome::Skip(HookEvent::OnSystemPromptBuilt { prompt }) => {
                let prompt = prompt.to_string();
                let markers = if prompt == original_prompt {
                    markers.into_iter().map(Into::into).collect()
                } else {
                    Vec::new()
                };
                Ok((prompt, markers))
            }
            HookOutcome::Deny { reason, .. } => Err(reason),
            HookOutcome::Continue(_) | HookOutcome::Skip(_) => {
                Ok((prompt, markers.into_iter().map(Into::into).collect()))
            }
        }
    }

    /// Runs a pre-assembled prompt through the `OnSystemPromptBuilt` hook.
    ///
    /// The `stable_prefix_len` indicates the byte length of the stable prefix
    /// (Identity + Tools + Skills) within the combined prompt. A cache marker
    /// is emitted for this range if the hook does not modify the prompt.
    pub(crate) async fn build_with_hooks_from_prompt(
        &self,
        hook_registry: &HookRegistry,
        ctx: &HookContext,
        prompt: &str,
        stable_prefix_len: usize,
    ) -> Result<(String, Vec<SystemCacheMarker>), String> {
        let original_prompt = prompt.to_string();
        let outcome = hook_registry
            .dispatch(ctx, HookEvent::OnSystemPromptBuilt { prompt })
            .await;

        match outcome {
            HookOutcome::Continue(HookEvent::OnSystemPromptBuilt { prompt })
            | HookOutcome::Skip(HookEvent::OnSystemPromptBuilt { prompt }) => {
                let prompt = prompt.to_string();
                let markers = if prompt == original_prompt && stable_prefix_len > 0 {
                    vec![SystemCacheMarker {
                        offset: 0,
                        length: stable_prefix_len,
                        cache_type: SystemCacheType::Ephemeral,
                    }]
                } else {
                    Vec::new()
                };
                Ok((prompt, markers))
            }
            HookOutcome::Deny { reason, .. } => Err(reason),
            HookOutcome::Continue(_) | HookOutcome::Skip(_) => {
                let markers = if stable_prefix_len > 0 {
                    vec![SystemCacheMarker {
                        offset: 0,
                        length: stable_prefix_len,
                        cache_type: SystemCacheType::Ephemeral,
                    }]
                } else {
                    Vec::new()
                };
                Ok((prompt.to_string(), markers))
            }
        }
    }

    /// Builds only the stable prefix (Identity + Tools + Skills).
    ///
    /// These sections are cacheable and do not change between turns unless
    /// tools, skills, or the identity/custom prompt are modified. The result
    /// can be cached by the caller and reused across turns.
    ///
    /// Returns `None` if there are no stable sections (should not happen in
    /// practice since Identity is always present).
    #[must_use]
    pub fn build_stable_prefix(&self) -> String {
        let sections = self.prompt_sections();
        let mut prefix = String::new();
        let mut first = true;
        for section in &sections {
            if section.kind != PromptSectionKind::Cacheable {
                break;
            }
            if !first {
                prefix.push('\n');
            }
            prefix.push_str(&section.text);
            first = false;
        }
        prefix
    }

    /// Builds only the dynamic suffix (Context + User Preferences + Runtime + Append).
    ///
    /// These sections change every turn (e.g., datetime) or are semi-stable
    /// (context files, user preferences). Combined with a cached stable prefix,
    /// they form the complete system prompt.
    #[must_use]
    pub fn build_dynamic_suffix(&self) -> String {
        let sections = self.prompt_sections();
        let mut suffix = String::new();
        let mut first = true;
        for section in &sections {
            if section.kind == PromptSectionKind::Cacheable {
                continue;
            }
            if !first {
                suffix.push('\n');
            }
            suffix.push_str(&section.text);
            first = false;
        }
        suffix
    }

    /// Assembles the system prompt with cache control markers.
    ///
    /// Returns the prompt string and a list of [`CacheMarker`]s indicating
    /// which byte ranges are stable and suitable for provider caching.
    ///
    /// Cacheable sections:
    /// - Identity (or custom prompt)
    /// - Tools
    /// - Skill index
    ///
    /// Semi-stable sections (context files, user preferences) and the
    /// append prompt are not marked for caching.
    #[must_use]
    pub fn build_with_cache_markers(&self) -> (String, Vec<CacheMarker>) {
        let mut markers: Vec<CacheMarker> = Vec::new();
        let sections = self.prompt_sections();
        let mut prompt = String::new();

        for (index, section) in sections.iter().enumerate() {
            if index > 0 {
                prompt.push('\n');
            }
            let offset = prompt.len();
            prompt.push_str(&section.text);
            if section.kind == PromptSectionKind::Cacheable {
                markers.push(CacheMarker {
                    offset,
                    length: section.text.len(),
                    cache_type: CacheType::Ephemeral,
                });
            }
        }

        (prompt, markers)
    }

    /// Estimates the total token count of the assembled prompt.
    ///
    /// Uses a heuristic of 1 token per 4 characters, which is a reasonable
    /// approximation for English text. This is not exact and should not be
    /// used for billing purposes.
    #[must_use]
    pub fn total_tokens(&self) -> usize {
        let prompt = self.build();
        // Heuristic: ~1 token per 4 characters for English text
        prompt.chars().count().div_ceil(4)
    }

    /// Logs the prompt size for debugging purposes.
    ///
    /// Prints the character count and estimated token count to stderr.
    /// This is useful for monitoring prompt size during development.
    pub fn log_size(&self) {
        let prompt = self.build();
        let char_count = prompt.chars().count();
        let token_estimate = self.total_tokens();
        eprintln!("System prompt: {char_count} characters, ~{token_estimate} tokens");
    }
}

impl Default for SystemPromptBuilder {
    fn default() -> Self {
        Self::new()
    }
}
