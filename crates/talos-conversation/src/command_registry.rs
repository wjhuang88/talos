//! Slash command registry and completion metadata.

/// Diagnostic passthrough command reserved for model request inspection.
pub(crate) const MOCK_REQUEST_COMMAND: &str = "/mock-request";

/// Origin of a slash command — determines how metadata and execution are resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandOrigin {
    /// Command owned by a typed runtime module (Conversation, Session, TUI, etc.).
    Builtin,
    /// Command backed by a registered tool; description, schema, and nature resolve
    /// from the live [`talos_core::tool::ToolRegistry`] at runtime.
    ToolBacked { tool_name: &'static str },
}

/// Availability predicate type — returns `true` when the command's owner is ready.
pub type AvailabilityPredicate = fn() -> bool;

/// Always-available predicate for commands whose owners are unconditionally present.
pub const fn always_available() -> bool {
    true
}

/// Definition of a built-in slash command — consumed by help, completion, and the TUI-010 menu.
pub struct CommandDefinition {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub usage: &'static str,
    pub description: &'static str,
    /// Optional argument hint (e.g. `"<path>"` for `/export <path>`).
    pub arg_hint: Option<&'static str>,
    /// How the command's metadata and execution are resolved.
    pub origin: CommandOrigin,
    /// Runtime availability check — command is hidden from help/completion when this
    /// returns `false`. Tool-backed commands gate on tool presence; owner-typed commands
    /// gate on their module being active.
    pub available: AvailabilityPredicate,
}

/// How a slash command should behave when accepted from an interactive command picker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandExecutionMode {
    /// The command is complete as selected and can be submitted immediately.
    DirectExecution,
    /// The command needs or accepts inline arguments before submission.
    RequireInput,
}

impl CommandDefinition {
    /// Returns `true` when accepting the command needs the user to finish inline arguments first.
    pub fn accepts_inline_arguments(&self) -> bool {
        self.arg_hint.is_some()
    }

    /// Returns the interactive picker execution mode derived from command metadata.
    pub fn execution_mode(&self) -> CommandExecutionMode {
        if self.accepts_inline_arguments() {
            CommandExecutionMode::RequireInput
        } else {
            CommandExecutionMode::DirectExecution
        }
    }
}

/// Ordered registry of built-in slash commands.
pub struct CommandRegistry {
    commands: Vec<CommandDefinition>,
}

impl CommandRegistry {
    fn new(commands: Vec<CommandDefinition>) -> Self {
        Self { commands }
    }

    pub fn list(&self) -> &[CommandDefinition] {
        &self.commands
    }

    /// Returns only the commands whose availability predicate returns `true`.
    pub fn available_commands(&self) -> Vec<&CommandDefinition> {
        self.commands
            .iter()
            .filter(|cmd| (cmd.available)())
            .collect()
    }

    pub fn find(&self, name: &str) -> Option<&CommandDefinition> {
        self.commands
            .iter()
            .find(|cmd| cmd.name == name || cmd.aliases.contains(&name))
    }

    pub fn names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = Vec::new();
        for cmd in &self.commands {
            names.push(cmd.name);
            names.extend(cmd.aliases);
        }
        names
    }

    /// Returns only available names (filtered by availability predicates).
    pub fn available_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = Vec::new();
        for cmd in &self.commands {
            if (cmd.available)() {
                names.push(cmd.name);
                names.extend(cmd.aliases);
            }
        }
        names
    }

    pub fn complete(&self, prefix: &str) -> Vec<&str> {
        self.commands
            .iter()
            .filter(|cmd| (cmd.available)())
            .flat_map(|cmd| {
                let mut completions: Vec<&str> = Vec::new();
                if cmd.name.starts_with(prefix) {
                    completions.push(cmd.name);
                }
                for alias in cmd.aliases {
                    if alias.starts_with(prefix) {
                        completions.push(*alias);
                    }
                }
                completions
            })
            .collect()
    }
}

static COMMAND_REGISTRY: std::sync::LazyLock<CommandRegistry> = std::sync::LazyLock::new(|| {
    CommandRegistry::new(vec![
        CommandDefinition {
            name: "/help",
            aliases: &[],
            usage: "/help",
            description: "Show this help",
            arg_hint: None,
            origin: CommandOrigin::Builtin,
            available: always_available,
        },
        CommandDefinition {
            name: "/quit",
            aliases: &["/exit"],
            usage: "/quit | /exit",
            description: "Exit Talos",
            arg_hint: None,
            origin: CommandOrigin::Builtin,
            available: always_available,
        },
        CommandDefinition {
            name: "/status",
            aliases: &[],
            usage: "/status",
            description: "Show session info",
            arg_hint: None,
            origin: CommandOrigin::Builtin,
            available: always_available,
        },
        CommandDefinition {
            name: "/plugins",
            aliases: &[],
            usage: "/plugins",
            description: "Plugin packages (not yet available — use /mcp for MCP status)",
            arg_hint: None,
            origin: CommandOrigin::Builtin,
            available: always_available,
        },
        CommandDefinition {
            name: "/mcp",
            aliases: &[],
            usage: "/mcp",
            description: "Show MCP server status and tool provenance",
            arg_hint: None,
            origin: CommandOrigin::Builtin,
            available: always_available,
        },
        CommandDefinition {
            name: "/hooks",
            aliases: &[],
            usage: "/hooks",
            description: "Show hook diagnostics without executing hooks",
            arg_hint: None,
            origin: CommandOrigin::Builtin,
            available: always_available,
        },
        CommandDefinition {
            name: "/skills",
            aliases: &[],
            usage: "/skills [activate <name> | reference <path>]",
            description: "List or activate runtime skills",
            arg_hint: Some("[activate <name> | reference <path>]"),
            origin: CommandOrigin::Builtin,
            available: always_available,
        },
        CommandDefinition {
            name: "/copy",
            aliases: &[],
            usage: "/copy last | /copy all",
            description: "Copy transcript to clipboard",
            arg_hint: Some("last | all"),
            origin: CommandOrigin::Builtin,
            available: always_available,
        },
        CommandDefinition {
            name: "/export",
            aliases: &[],
            usage: "/export <path>",
            description: "Export transcript to file",
            arg_hint: Some("<path>"),
            origin: CommandOrigin::Builtin,
            available: always_available,
        },
        CommandDefinition {
            name: "/new",
            aliases: &[],
            usage: "/new",
            description: "Start a fresh session",
            arg_hint: None,
            origin: CommandOrigin::Builtin,
            available: always_available,
        },
        CommandDefinition {
            name: "/resume",
            aliases: &[],
            usage: "/resume [session-id]",
            description: "Resume a workspace session",
            arg_hint: Some("[session-id]"),
            origin: CommandOrigin::Builtin,
            available: always_available,
        },
        CommandDefinition {
            name: "/fork",
            aliases: &[],
            usage: "/fork",
            description: "Fork the active session",
            arg_hint: None,
            origin: CommandOrigin::Builtin,
            available: always_available,
        },
        CommandDefinition {
            name: "/delete",
            aliases: &[],
            usage: "/delete [N]",
            description: "Delete a workspace session via the picker",
            arg_hint: Some("[N]"),
            origin: CommandOrigin::Builtin,
            available: always_available,
        },
        CommandDefinition {
            name: "/model",
            aliases: &[],
            usage: "/model",
            description: "Browse and switch models (opens picker)",
            arg_hint: None,
            origin: CommandOrigin::Builtin,
            available: always_available,
        },
        CommandDefinition {
            name: "/connect",
            aliases: &[],
            usage: "/connect",
            description: "Connect a provider (opens picker)",
            arg_hint: None,
            origin: CommandOrigin::Builtin,
            available: always_available,
        },
        CommandDefinition {
            name: "/todo",
            aliases: &[],
            usage: "/todo [list|show|stats|export]",
            description: "Show session todos",
            arg_hint: Some("[list|show|stats|export]"),
            origin: CommandOrigin::Builtin,
            available: always_available,
        },
        CommandDefinition {
            name: "/agile",
            aliases: &[],
            usage: "/agile [status]",
            description: "Show governance board/iteration status",
            arg_hint: Some("[status]"),
            origin: CommandOrigin::Builtin,
            available: always_available,
        },
        CommandDefinition {
            name: "/validate",
            aliases: &[],
            usage: "/validate [governance]",
            description: "Run internal validation evidence without host tools",
            arg_hint: Some("[governance]"),
            origin: CommandOrigin::Builtin,
            available: always_available,
        },
        CommandDefinition {
            name: "/attach",
            aliases: &[],
            usage: "/attach <path>",
            description: "Attach a local image to the next message (vision models only)",
            arg_hint: Some("<path>"),
            origin: CommandOrigin::Builtin,
            available: always_available,
        },
    ])
});

/// Returns the shared static command registry for TUI-010 menu and help rendering.
pub fn command_registry() -> &'static CommandRegistry {
    &COMMAND_REGISTRY
}
