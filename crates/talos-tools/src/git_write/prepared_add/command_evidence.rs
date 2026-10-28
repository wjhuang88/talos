//! Redacted, operation-specific evidence for retained Git mutation commands.
//! All six effect classes remain possible through configured native programs;
//! this deliberately promises neither confinement nor exhaustive target discovery.

use super::*;

impl AddPlan {
    pub(super) fn git_command_evidence(&self) -> Option<talos_core::tool::PreparedCommandEvidence> {
        use talos_core::tool::{
            PreparedCommandDescription, PreparedCommandEffect as Effect, PreparedCommandEvidence,
            PreparedCommandTargets,
        };
        let (operation, arguments, remote_selection, explicit_remote) = match &self.operation {
            Operation::Add
                if self.phase == Phase::Mutation && self.native_add_command.is_some() =>
            {
                (
                    "git:add-command/v1",
                    vec![
                        "add".to_owned(),
                        format!("<retained-paths:{}>", self.paths.len()),
                    ],
                    "local repository operation",
                    None,
                )
            }
            Operation::Commit(input)
                if self.phase == Phase::Mutation && self.native_commit_command.is_some() =>
            {
                (
                    "git:commit-command/v1",
                    vec![
                        "commit".to_owned(),
                        "-m".to_owned(),
                        "<retained-message>".to_owned(),
                        if input.all { "--all" } else { "<staged-only>" }.to_owned(),
                    ],
                    "local repository operation",
                    None,
                )
            }
            Operation::Checkout(input)
                if self.phase == Phase::Mutation && self.native_checkout_command.is_some() =>
            {
                (
                    "git:checkout-command/v1",
                    vec![
                        "checkout".to_owned(),
                        if input.create {
                            "-b"
                        } else {
                            "<existing-branch>"
                        }
                        .to_owned(),
                        "<retained-branch>".to_owned(),
                    ],
                    "local repository operation",
                    None,
                )
            }
            Operation::Push(input)
                if self.phase == Phase::Mutation && self.native_push_command.is_some() =>
            {
                (
                    "git:push-command/v1",
                    vec![
                        "push".to_owned(),
                        if input.force {
                            "--force"
                        } else {
                            "<normal-update>"
                        }
                        .to_owned(),
                        "<retained-remote>".to_owned(),
                        "<retained-refspec>".to_owned(),
                    ],
                    if input.remote.is_some() {
                        "explicit retained remote; value withheld"
                    } else {
                        "configured default remote"
                    },
                    input.remote.as_deref().and_then(|remote| {
                        super::super::config_state::command_remote_host(remote.as_bytes())
                    }),
                )
            }
            Operation::Pull(input)
                if self.phase == Phase::PullCommandMutation
                    && self.native_pull_command.is_some() =>
            {
                (
                    "git:pull-command/v1",
                    [
                        "-c",
                        "maintenance.autoDetach=false",
                        "-c",
                        "gc.autoDetach=false",
                        "--no-pager",
                        "pull",
                    ]
                    .map(str::to_owned)
                    .into_iter()
                    .chain(if input.remote.is_some() {
                        vec!["--".into(), "<retained-remote>".into()]
                    } else {
                        Vec::new()
                    })
                    .collect(),
                    if input.remote.is_some() {
                        "explicit retained remote; name/location withheld"
                    } else {
                        "configured upstream"
                    },
                    input.remote.as_deref().and_then(|remote| {
                        super::super::config_state::command_remote_host(remote.as_bytes())
                    }),
                )
            }
            _ => return None,
        };
        let config = self.config.as_ref()?;
        let label = |path: &Path| {
            path.strip_prefix(&self.workspace_review_root)
                .ok()
                .map(|path| {
                    if path.as_os_str().is_empty() {
                        ".".into()
                    } else {
                        path.to_string_lossy().replace('\\', "/")
                    }
                })
                .unwrap_or_else(|| "<outside-workspace>".into())
        };
        Some(PreparedCommandEvidence::new(
            self.identity.clone(),
            PreparedCommandDescription {
                operation: operation.into(),
                arguments,
                effects: vec![
                    Effect::Execute,
                    Effect::Network,
                    Effect::WorktreeMutation,
                    Effect::MetadataMutation,
                    Effect::Recursion,
                    Effect::ConfiguredPrograms,
                ],
                confined: false,
                targets: PreparedCommandTargets {
                    repository: label(&self.repository.workdir),
                    metadata: [&self.repository.git_dir, &self.repository.common_dir]
                        .map(|path| label(path))
                        .to_vec(),
                    remote_selection: remote_selection.into(),
                    configured_remote_hosts: config.pull_command_remote_hosts(),
                    invocation_remote_host: explicit_remote,
                    configured_program_categories: config
                        .pull_command_program_categories()
                        .into_iter()
                        .map(str::to_owned)
                        .collect(),
                    dynamic_effects_unknown: true,
                },
            },
        ))
    }
}
