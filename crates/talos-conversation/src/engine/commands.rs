use super::*;

fn parse_todo_command(arg: &str) -> Result<TodoCommandRequest, String> {
    let mut tokens = arg.split_whitespace();
    let subcommand = tokens.next().unwrap_or("list");
    let mut request = TodoCommandRequest {
        action: match subcommand {
            "" | "list" => TodoCommandAction::List,
            "show" => {
                let id = tokens
                    .next()
                    .ok_or_else(|| "Usage: /todo show <id>".to_string())?;
                TodoCommandAction::Show { id: id.to_string() }
            }
            "stats" => TodoCommandAction::Stats,
            "delete" => {
                let id = tokens
                    .next()
                    .ok_or_else(|| "Usage: /todo delete <id> --confirm".to_string())?;
                let mut confirm = false;
                let mut pending = tokens.next();
                while let Some(flag) = pending.take() {
                    match flag {
                        "--confirm" | "--yes" | "-y" => confirm = true,
                        other => {
                            return Err(format!(
                                "Unknown /todo delete option: {other}. Usage: /todo delete <id> --confirm"
                            ));
                        }
                    }
                    pending = tokens.next();
                }
                TodoCommandAction::Delete {
                    id: id.to_string(),
                    confirm,
                }
            }
            "export" => {
                let format = match tokens.next() {
                    None | Some("markdown") | Some("md") => TodoExportFormat::Markdown,
                    Some("json") => TodoExportFormat::Json,
                    Some(other) => {
                        return Err(format!("Unknown todo export format: {other}"));
                    }
                };
                TodoCommandAction::Export { format }
            }
            other if other.starts_with("--") => TodoCommandAction::List,
            other => {
                return Err(format!(
                    "Unknown todo command: {other}. Usage: /todo [list|show|stats|delete|export]"
                ));
            }
        },
        status_filter: None,
        priority_filter: None,
        tag_filter: None,
        sort: None,
    };

    let mut pending = if subcommand.starts_with("--") {
        Some(subcommand)
    } else {
        None
    };
    while let Some(token) = pending.take().or_else(|| tokens.next()) {
        match token {
            "--status" => {
                request.status_filter = Some(
                    tokens
                        .next()
                        .ok_or_else(|| "Missing value for --status".to_string())?
                        .to_string(),
                );
            }
            "--priority" => {
                request.priority_filter = Some(
                    tokens
                        .next()
                        .ok_or_else(|| "Missing value for --priority".to_string())?
                        .to_string(),
                );
            }
            "--tag" => {
                request.tag_filter = Some(
                    tokens
                        .next()
                        .ok_or_else(|| "Missing value for --tag".to_string())?
                        .to_string(),
                );
            }
            "--sort" => {
                request.sort = Some(
                    tokens
                        .next()
                        .ok_or_else(|| "Missing value for --sort".to_string())?
                        .to_string(),
                );
            }
            other => return Err(format!("Unknown todo option: {other}")),
        }
    }

    Ok(request)
}

impl ConversationEngine {
    /// Slash command names currently exposed by help and completion.
    ///
    /// Derived from the shared `CommandRegistry` so help, completion, and TUI-010
    /// always reflect the same executable command set.
    pub fn slash_commands() -> Vec<&'static str> {
        command_registry().available_names()
    }

    pub fn handle_slash_command(&mut self, input: &str) -> Vec<UiOutput> {
        let parts: Vec<&str> = input.splitn(2, ' ').collect();
        let cmd = parts[0];
        let arg = parts.get(1).copied().unwrap_or("");
        let mut outputs = Vec::new();

        match cmd {
            "/help" => {
                let mut text = String::from("[System] Available commands:\n");
                for command in command_registry().available_commands() {
                    text.push_str(&format!(
                        "[System]   {:<20} — {}\n",
                        command.usage, command.description
                    ));
                }
                outputs.push(content_block(MessageSource::System, text));
            }
            "/quit" | "/exit" => {
                outputs.push(UiOutput::Exit);
            }
            "/status" => {
                let text = format!(
                    "[System] Model: {} | Input: {} | Output: {} tokens\n",
                    self.model_name, self.usage.input_tokens, self.usage.output_tokens,
                );
                outputs.push(content_block(MessageSource::System, text));
            }
            "/auto" => {
                let argument = arg.trim().to_ascii_lowercase();
                match argument.as_str() {
                    "" => {
                        let source = if self.auto_override.is_some() {
                            "session override"
                        } else {
                            "config/default"
                        };
                        let state = if self.auto_enabled() {
                            "enabled"
                        } else {
                            "disabled"
                        };
                        outputs.push(content_block(MessageSource::System, format!("[System] Auto mode: {state} (source: {source}; evaluator: unavailable in this slice; deadline: 8s; circuit: closed).\n")));
                    }
                    "on" | "off" => {
                        let enabled = argument == "on";
                        self.auto_override = Some(enabled);
                        if let Some(callback) = &self.auto_mode_callback {
                            callback(enabled);
                        }
                        let state = if enabled { "enabled" } else { "disabled" };
                        outputs.push(content_block(MessageSource::System, format!("[System] Auto mode {state} for this session only; configuration and transcript were not changed.\n")));
                    }
                    _ => outputs.push(content_block(
                        MessageSource::Error,
                        "[Error] Usage: /auto [on | off]. State unchanged.\n".to_string(),
                    )),
                }
            }
            "/plugins" => {
                outputs.extend(self.handle_plugins_command());
            }
            "/mcp" => {
                outputs.extend(self.handle_mcp_command());
            }
            "/hooks" => {
                outputs.extend(self.handle_hooks_command());
            }
            "/skills" => {
                outputs.extend(self.handle_skills_command(arg));
            }
            "/copy" => {
                outputs.extend(self.handle_copy_command(arg));
            }
            "/export" => {
                outputs.extend(self.handle_export_command(arg));
            }
            "/new" => {
                if self.is_processing {
                    let text = "[System] Cannot start a new session while a turn is active. Wait for the current turn to finish.\n".to_string();
                    outputs.push(content_block(MessageSource::System, text));
                } else {
                    outputs.push(UiOutput::SessionNew(SessionNewRequest));
                }
            }
            "/resume" => {
                if self.is_processing {
                    let text = "[System] Cannot resume a session while a turn is active. Wait for the current turn to finish.\n".to_string();
                    outputs.push(content_block(MessageSource::System, text));
                } else {
                    let session_id = if arg.is_empty() {
                        None
                    } else {
                        Some(arg.to_string())
                    };
                    outputs.push(UiOutput::SessionResume(SessionResumeRequest { session_id }));
                }
            }
            "/fork" => {
                if self.is_processing {
                    let text = "[System] Cannot fork a session while a turn is active. Wait for the current turn to finish.\n".to_string();
                    outputs.push(content_block(MessageSource::System, text));
                } else {
                    outputs.push(UiOutput::SessionFork(SessionForkRequest));
                }
            }
            "/delete" => {
                if self.is_processing {
                    let text = "[System] Cannot delete a session while a turn is active. Wait for the current turn to finish.\n".to_string();
                    outputs.push(content_block(MessageSource::System, text));
                } else if arg.is_empty() {
                    outputs.push(UiOutput::SessionDelete(SessionDeleteRequest {
                        selection: None,
                    }));
                } else {
                    outputs.push(UiOutput::SessionDelete(SessionDeleteRequest {
                        selection: Some(arg.to_string()),
                    }));
                }
            }
            "/model" => {
                if self.is_processing {
                    let text = "[System] Cannot switch models while a turn is active. Wait for the current turn to finish.\n".to_string();
                    outputs.push(content_block(MessageSource::System, text));
                } else {
                    outputs.push(UiOutput::ModelSwitchRequest(ModelSwitchRequest {
                        model_id: arg.to_string(),
                        provider_needs_credential: false,
                        provider_hint: None,
                    }));
                }
            }
            "/connect" => {
                outputs.push(UiOutput::ConnectProviderRequest {
                    provider: arg.to_string(),
                });
            }
            "/todo" => {
                outputs.extend(self.handle_todo_command(arg));
            }
            "/agile" => {
                outputs.extend(self.handle_agile_command(arg));
            }
            "/validate" => {
                outputs.extend(self.handle_validate_command(arg));
            }
            "/attach" => {
                if arg.trim().is_empty() {
                    let text =
                        "[Error] /attach requires a file path. Usage: /attach <path>\n".to_string();
                    outputs.push(content_block(MessageSource::Error, text));
                } else {
                    outputs.push(UiOutput::AttachImageRequest {
                        path: arg.trim().to_string(),
                    });
                }
            }
            "/attachments" | "/imgs" => {
                if self.pending_image_attachments.is_empty() {
                    let text =
                        "[System] No pending image attachments. Use /attach <path> to add one.\n"
                            .to_string();
                    outputs.push(content_block(MessageSource::System, text));
                } else {
                    let mut text = format!(
                        "[System] Pending image attachments ({}):\n",
                        self.pending_image_attachments.len()
                    );
                    for (idx, part) in self.pending_image_attachments.iter().enumerate() {
                        match part {
                            talos_core::message::ContentPart::Image {
                                path,
                                mime,
                                byte_count,
                                content_digest: _,
                            } => {
                                let filename = path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("(unknown)");
                                text.push_str(&format!(
                                    "[System]   [{}] {filename} ({byte_count} bytes, {mime})\n",
                                    idx + 1
                                ));
                            }
                            _ => {
                                text.push_str(&format!(
                                    "[System]   [{}] (non-image part)\n",
                                    idx + 1
                                ));
                            }
                        }
                    }
                    text.push_str(
                        "[System] These will be sent with your next message. Use /detach <index|all> to remove.\n",
                    );
                    outputs.push(content_block(MessageSource::System, text));
                }
            }
            "/detach" => {
                let arg_trimmed = arg.trim();
                if arg_trimmed.is_empty() {
                    let hint = "[Error] Usage: /detach <index|all>\nExample: /detach 1\n          /detach all\n".to_string();
                    outputs.push(content_block(MessageSource::Error, hint));
                } else if arg_trimmed == "all" {
                    let count = self.pending_image_attachments.len();
                    if count == 0 {
                        let text = "[System] No pending attachments to remove.\n".to_string();
                        outputs.push(content_block(MessageSource::System, text));
                    } else {
                        self.pending_image_attachments.clear();
                        let text = format!("[System] Removed {count} pending attachment(s).\n");
                        outputs.push(content_block(MessageSource::System, text));
                        outputs.push(UiOutput::Status(self.status_snapshot()));
                    }
                } else {
                    match arg_trimmed.parse::<usize>() {
                        Ok(n) if n >= 1 && n <= self.pending_image_attachments.len() => {
                            self.pending_image_attachments.remove(n - 1);
                            let text = format!("[System] Removed attachment at index {n}.\n");
                            outputs.push(content_block(MessageSource::System, text));
                            outputs.push(UiOutput::Status(self.status_snapshot()));
                        }
                        Ok(n) => {
                            let text = format!(
                                "[Error] Index {n} out of range. Run /attachments to see valid indices (1..={}).\n",
                                self.pending_image_attachments.len()
                            );
                            outputs.push(content_block(MessageSource::Error, text));
                        }
                        Err(_) => {
                            let text = format!(
                                "[Error] '/detach {arg_trimmed}' is not a valid index. Use a positive number or 'all'.\n"
                            );
                            outputs.push(content_block(MessageSource::Error, text));
                        }
                    }
                }
            }
            _ => {
                let text =
                    format!("[Error] Unknown command: {cmd}. Type /help for available commands.\n");
                outputs.push(content_block(MessageSource::Error, text));
            }
        }

        outputs
    }

    fn handle_todo_command(&self, arg: &str) -> Vec<UiOutput> {
        match parse_todo_command(arg) {
            Ok(request) => vec![UiOutput::TodoCommand(request)],
            Err(message) => vec![content_block(
                MessageSource::Error,
                format!("[Error] {message}\n"),
            )],
        }
    }

    fn handle_agile_command(&self, _arg: &str) -> Vec<UiOutput> {
        let Some(ref ws) = self.workspace_root else {
            return vec![content_block(
                MessageSource::System,
                "[System] /agile is unavailable — no workspace path set.\n".to_string(),
            )];
        };
        let text = crate::governance_summary::format_governance_summary(ws);
        vec![content_block(MessageSource::System, text)]
    }

    fn handle_validate_command(&self, arg: &str) -> Vec<UiOutput> {
        let profile = match arg.trim() {
            "" | "governance" => crate::ValidationProfile::Governance,
            other => {
                let text = format!(
                    "[Error] Unsupported internal validation profile: {other}. Usage: /validate [governance]\n"
                );
                return vec![content_block(MessageSource::Error, text)];
            }
        };
        let Some(ref ws) = self.workspace_root else {
            return vec![content_block(
                MessageSource::System,
                "[System] /validate is unavailable — no workspace path set.\n".to_string(),
            )];
        };

        let plan = crate::collect_validation_plan(ws, profile);
        let evidence = crate::run_validation_plan(ws, plan);
        let text = crate::render_text_evidence(&evidence);
        vec![content_block(MessageSource::System, text)]
    }

    fn handle_copy_command(&self, scope: &str) -> Vec<UiOutput> {
        let (text, scope_enum, label) = match scope {
            "last" => {
                let content = self
                    .last_assistant_text()
                    .unwrap_or_else(|| "(no assistant messages yet)".to_string());
                (content, CopyScope::Last, "last assistant message")
            }
            "all" => {
                let content = self.transcript_plain_text();
                if content.is_empty() {
                    ("(empty transcript)".to_string(), CopyScope::All, "all")
                } else {
                    (content, CopyScope::All, "full transcript")
                }
            }
            _ => {
                let hint = "[Error] Usage: /copy last | /copy all\n".to_string();
                return vec![content_block(MessageSource::Error, hint)];
            }
        };

        let confirm = format!("[System] Copying {label} to clipboard…\n");
        let mut outputs = vec![content_block(MessageSource::System, confirm)];
        outputs.push(UiOutput::CopyToClipboard {
            text,
            scope: scope_enum,
        });
        outputs
    }

    fn handle_export_command(&self, path_arg: &str) -> Vec<UiOutput> {
        let path = path_arg.trim();
        if path.is_empty() {
            let hint =
                "[Error] Usage: /export <path> [--include-thinking]\nExample: /export transcript.md\n".to_string();
            return vec![content_block(MessageSource::Error, hint)];
        }

        let include_thinking = path.contains("--include-thinking");
        let clean_path = path.replace("--include-thinking", "").trim().to_string();

        let content = if include_thinking {
            self.transcript_plain_text_with_thinking()
        } else {
            self.transcript_plain_text()
        };
        if content.is_empty() {
            let msg = "[System] Transcript is empty — nothing to export.\n".to_string();
            return vec![content_block(MessageSource::System, msg)];
        }

        let confirm = format!("[System] Exporting transcript to {clean_path}…\n");
        let mut outputs = vec![content_block(MessageSource::System, confirm)];
        outputs.push(UiOutput::ExportToFile {
            path: PathBuf::from(clean_path),
            content,
        });
        outputs
    }

    fn handle_mcp_command(&mut self) -> Vec<UiOutput> {
        let snap = self.extension_snapshot();
        if snap.mcp_servers.is_empty() && snap.provenance.is_empty() {
            let text = "[System] No MCP servers configured and no tool provenance observed yet.\n"
                .to_string();
            return vec![content_block(MessageSource::System, text)];
        }
        let mut text = String::new();
        if !snap.mcp_servers.is_empty() {
            text.push_str("[System] MCP servers (startup snapshot):\n");
            for server in &snap.mcp_servers {
                if server.connected {
                    text.push_str(&format!(
                        "[System]   {} (connected, {} tool{})\n",
                        server.name,
                        server.tool_count,
                        if server.tool_count == 1 { "" } else { "s" },
                    ));
                } else {
                    let error = server.error.as_deref().unwrap_or("unavailable");
                    text.push_str(&format!(
                        "[System]   {} (unavailable: {error})\n",
                        server.name
                    ));
                }
            }
        }
        if !snap.provenance.is_empty() {
            text.push_str("[System] Observed tool provenance (this session):\n");
            for entry in &snap.provenance {
                text.push_str(&format!(
                    "[System]   {} ({} call{})\n",
                    entry.key,
                    entry.count,
                    if entry.count == 1 { "" } else { "s" },
                ));
            }
        }
        let mcp_collisions: Vec<_> = snap
            .collisions
            .iter()
            .filter(|c| c.starts_with("mcp:"))
            .collect();
        if !mcp_collisions.is_empty() {
            text.push_str(&format!(
                "[System]   collisions: {}\n",
                mcp_collisions
                    .iter()
                    .map(|c| c.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        vec![content_block(MessageSource::System, text)]
    }

    fn handle_hooks_command(&self) -> Vec<UiOutput> {
        let snap = self.extension_snapshot();
        let mut text = String::new();
        text.push_str("[System] Hooks diagnostics:\n");

        if snap.hooks.declarations.is_empty() {
            text.push_str("[System]   config-introduced hooks: none declared\n");
        } else {
            text.push_str(&format!(
                "[System]   config-introduced hooks: {} declared\n",
                snap.hooks.declarations.len()
            ));
            for d in &snap.hooks.declarations {
                let status = if d.enabled { "enabled" } else { "disabled" };
                text.push_str(&format!(
                    "[System]     {} ({}) [{status}]\n",
                    d.name, d.event
                ));
            }
        }
        text.push_str(&format!(
            "[System]   executable hook carriers: {}\n",
            if snap.hooks.executable_carriers_enabled {
                "enabled"
            } else {
                "disabled"
            }
        ));
        text.push_str("[System]   builtin hook event catalog:\n");
        for kind in &snap.hooks.event_catalog {
            text.push_str(&format!("[System]     {kind}\n"));
        }
        let hook_collisions: Vec<_> = snap
            .collisions
            .iter()
            .filter(|c| c.starts_with("hook:"))
            .collect();
        if !hook_collisions.is_empty() {
            text.push_str(&format!(
                "[System]   collisions: {}\n",
                hook_collisions
                    .iter()
                    .map(|c| c.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        vec![content_block(MessageSource::System, text)]
    }

    fn handle_plugins_command(&self) -> Vec<UiOutput> {
        let snap = self.extension_snapshot();
        let mut text = String::new();
        text.push_str("[System] Extension diagnostics:\n");
        text.push_str(&format!(
            "[System]   MCP servers: {} ({} connected)\n",
            snap.mcp_servers.len(),
            snap.mcp_servers.iter().filter(|s| s.connected).count()
        ));
        text.push_str(&format!(
            "[System]   Hook declarations: {}\n",
            snap.hooks.declarations.len()
        ));
        text.push_str(&format!(
            "[System]   Provenance observations: {}\n",
            snap.provenance.len()
        ));
        if snap.loaded_plugins.is_empty() {
            text.push_str("[System]   WASM plugin packages: none loaded\n");
        } else {
            text.push_str(&format!(
                "[System]   WASM plugin packages: {} loaded\n",
                snap.loaded_plugins.len()
            ));
            for plugin in &snap.loaded_plugins {
                text.push_str(&format!(
                    "[System]     {}@{}/{} — capabilities: {}\n",
                    plugin.name,
                    plugin.version,
                    plugin.carrier,
                    plugin.capabilities.join(", ")
                ));
            }
        }
        text.push_str("[System] Use /mcp for MCP detail, /hooks for hook detail.\n");
        if !snap.collisions.is_empty() {
            text.push_str(&format!(
                "[System]   collisions: {}\n",
                snap.collisions.join(", ")
            ));
        }
        vec![content_block(MessageSource::System, text)]
    }

    fn handle_skills_command(&mut self, arg: &str) -> Vec<UiOutput> {
        let mut parts = arg.split_whitespace();
        match parts.next() {
            Some("activate") => {
                if self.is_processing {
                    let text = "[System] Cannot activate a skill while a turn is active. Wait for the current turn to finish.\n".to_string();
                    return vec![content_block(MessageSource::System, text)];
                }
                let name = parts.collect::<Vec<_>>().join(" ");
                if name.trim().is_empty() {
                    let text = "[Error] Usage: /skills activate <name>\n".to_string();
                    return vec![content_block(MessageSource::Error, text)];
                }
                return vec![UiOutput::SkillCommand(SkillCommandRequest::Activate {
                    name,
                })];
            }
            Some("reference") => {
                if self.is_processing {
                    let text = "[System] Cannot load a skill reference while a turn is active. Wait for the current turn to finish.\n".to_string();
                    return vec![content_block(MessageSource::System, text)];
                }
                let path = parts.collect::<Vec<_>>().join(" ");
                if path.trim().is_empty() {
                    let text = "[Error] Usage: /skills reference <relative-path>\n".to_string();
                    return vec![content_block(MessageSource::Error, text)];
                }
                return vec![UiOutput::SkillCommand(SkillCommandRequest::Reference {
                    path,
                })];
            }
            Some(other) => {
                let text = format!(
                    "[Error] Unknown /skills action: {other}. Usage: /skills [activate <name> | reference <path>]\n"
                );
                return vec![content_block(MessageSource::Error, text)];
            }
            None => {}
        }

        if self.skills.is_empty() {
            let text = "[System] No skills available.\n".to_string();
            return vec![content_block(MessageSource::System, text)];
        }

        let mut text = String::from("[System] Available skills (Level 0 metadata):\n");
        for skill in &self.skills {
            let state = if skill.active { "active" } else { "available" };
            text.push_str(&format!(
                "[System]   {} ({source}) ({state}) — {}\n",
                skill.name,
                skill.description,
                source = skill.source,
            ));
        }
        text.push_str(
            "[System] Use /skills activate <name> to load one Skill body, then /skills reference <relative-path> for bounded references.\n",
        );
        vec![content_block(MessageSource::System, text)]
    }

    pub fn is_model_passthrough_slash_command(input: &str) -> bool {
        let trimmed = input.trim_start();
        if trimmed == MOCK_REQUEST_COMMAND {
            return true;
        }

        trimmed
            .strip_prefix(MOCK_REQUEST_COMMAND)
            .and_then(|rest| rest.chars().next())
            .is_some_and(char::is_whitespace)
    }

    pub fn complete_slash_command(&self, input: &str) -> Vec<&str> {
        command_registry().complete(input)
    }
}
