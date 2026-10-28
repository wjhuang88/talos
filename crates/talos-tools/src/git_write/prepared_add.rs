//! One-use staging plan. Configuration and process ownership validation remain
//! I300 publication gates; this module does not claim OS containment.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use serde_json::Value;
use talos_core::tool::{
    PreparedNormalStage, PreparedNormalToolInvocation, PreparedPlanIdentity, ToolExecutionOutput,
    ToolNature, ToolPermissionFacet, ToolProvenance, ToolResourceKind,
};
use talos_permission::{AdmittedInvocation, PermissionRequest};

use super::{
    GitAddInput, GitCheckoutInput, GitCommitInput, GitPullInput, GitPushInput, GitToolError,
    capture_add_sources, literal_add_paths, repository_state::RepositoryState, snapshot,
};

#[path = "prepared_add/command.rs"]
mod command;
#[path = "prepared_add/host_path.rs"]
mod host_path;
#[path = "prepared_add/pull_command.rs"]
mod pull_command;

struct ChildInspectionEvidence {
    attributes: Vec<PathBuf>,
    sources: Vec<(PathBuf, PathBuf, Vec<String>, Vec<snapshot::Entry>)>,
    status: Vec<(PathBuf, super::status_state::StatusState)>,
}

// Git for Windows converts the executable's retained verbatim spelling to
// forward slashes. Compare that representation without resolving a new path,
// stripping prefixes, folding case, or interpreting dot components.
#[cfg(any(windows, test))]
fn windows_bound_program_spelling(reported: &Path, bound: &Path) -> bool {
    match (reported.to_str(), bound.to_str()) {
        (Some(reported), Some(bound)) => reported.replace('\\', "/") == bound.replace('\\', "/"),
        _ => false,
    }
}

fn validate_internal_program(
    reported: &Path,
    admitted: &Path,
    bound: &Path,
) -> Result<(), GitToolError> {
    // Preserve the spelling check before opening the newly reported path: a
    // different exec-path has no inherited Read authority, even if it might
    // resolve to the same file. Diagnostics must not canonicalize it first.
    let same_spelling = reported == admitted;
    #[cfg(windows)]
    let same_spelling = same_spelling || windows_bound_program_spelling(reported, bound);
    if !same_spelling {
        return Err(GitToolError::InvalidInput(format!(
            "Internal Git exec-path spelling changed after executable admission: admitted={}, reported={}",
            admitted.display(),
            reported.display(),
        )));
    }
    // Reopen only the admitted spelling, never the reported alternative. The
    // caller also retains executable bytes/identity and revalidates them.
    let canonical = admitted.canonicalize().map_err(|error| {
        GitToolError::InvalidInput(format!(
            "Internal Git exec-path cannot be resolved after executable admission: {error}"
        ))
    })?;
    if canonical != bound {
        return Err(GitToolError::InvalidInput(
            "Internal Git exec-path canonical target changed after executable admission".into(),
        ));
    }
    Ok(())
}

#[derive(PartialEq, Eq)]
struct ReceiverChildExecutable {
    spelling: PathBuf,
    canonical: PathBuf,
    identity: snapshot::PathIdentity,
    program: ProgramEvidence,
}

struct SenderProgramBinding {
    spelling: PathBuf,
    canonical: PathBuf,
    identity: snapshot::PathIdentity,
}

impl ReceiverChildExecutable {
    fn capture(spelling: &Path, admitted: &Path) -> Result<Self, GitToolError> {
        let canonical = spelling.canonicalize().map_err(|_| {
            GitToolError::InvalidInput("Receiver child executable is unavailable".into())
        })?;
        if canonical != admitted {
            return Err(GitToolError::InvalidInput(
                "Receiver child executable changed after Read admission".into(),
            ));
        }
        let identity = snapshot::PathIdentity::capture(spelling).map_err(|_| {
            GitToolError::InvalidInput("Receiver child executable path is unavailable".into())
        })?;
        let program = capture_program(&canonical)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if !std::fs::metadata(&canonical)
                .is_ok_and(|metadata| metadata.permissions().mode() & 0o111 != 0)
            {
                return Err(GitToolError::InvalidInput(
                    "Receiver child executable is not executable".into(),
                ));
            }
        }
        if spelling.canonicalize().ok().as_deref() != Some(&canonical)
            || snapshot::PathIdentity::capture(spelling).ok().as_ref() != Some(&identity)
            || capture_program(&canonical)? != program
        {
            return Err(GitToolError::InvalidInput(
                "Receiver child executable changed during inspection".into(),
            ));
        }
        Ok(Self {
            spelling: spelling.to_owned(),
            canonical,
            identity,
            program,
        })
    }

    fn revalidate(&self, admitted: &Path) -> Result<(), GitToolError> {
        if &Self::capture(&self.spelling, admitted)? != self {
            return Err(GitToolError::InvalidInput(
                "Receiver child executable changed after discovery".into(),
            ));
        }
        Ok(())
    }
}

impl ChildInspectionEvidence {
    fn revalidate(&self) -> Result<(), GitToolError> {
        for (workdir, expected) in &self.status {
            if &super::status_state::StatusState::capture(workdir, &[])? != expected {
                return Err(GitToolError::InvalidInput(
                    "Submodule status inputs changed during inspection".into(),
                ));
            }
        }
        for (discovery, workdir, paths, expected) in &self.sources {
            let (current, nested) = tracked_commit_sources_at(discovery, workdir)?;
            if &current != paths
                || !nested.is_empty()
                || &capture_tracked_sources(workdir, paths)? != expected
            {
                return Err(GitToolError::InvalidInput(
                    "Submodule tracked sources changed after inspection".into(),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
enum Operation {
    Add,
    Commit(GitCommitInput),
    Checkout(GitCheckoutInput),
    Push(GitPushInput),
    Pull(GitPullInput),
}

impl Operation {
    fn name(&self) -> &'static str {
        match self {
            Self::Add => "git_add",
            Self::Commit(_) => "git_commit",
            Self::Checkout(_) => "git_checkout",
            Self::Push(_) => "git_push",
            Self::Pull(_) => "git_pull",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    SenderConfiguration,
    ReceiverRoots,
    PushProgram,
    ReceiverMetadata,
    ReceiverCommon,
    ReceiverConfiguration,
    ReceiverAuxiliary,
    ReceiverMutation,
    Mutation,
    CheckoutDeletion,
    CheckoutNewGitlinks,
    CheckoutChildMarkers,
    CheckoutChildMetadata,
    CheckoutChildConfiguration,
    CheckoutChildWorktree,
    TargetConfiguration,
    PullCommandMutation,
}

pub(super) struct AddPlan {
    phase: Phase,
    command_budget: std::sync::Mutex<CommandBudget>,
    #[cfg(any(unix, windows))]
    execution: super::OwnedCommandOperation,
    operation: Operation,
    commit_state: Option<super::repository_state::CommitState>,
    push_destinations: Vec<String>,
    receiver_roots: Vec<PathBuf>,
    receiver_child_paths: Vec<PathBuf>,
    receiver_child_executables: Vec<ReceiverChildExecutable>,
    receiver_layouts: Vec<super::receiver_state::Root>,
    receiver_metadata: Vec<super::receiver_state::Metadata>,
    receiver_common: Vec<super::receiver_state::Common>,
    receiver_worktree_inventories: Vec<super::receiver_state::WorktreeInventory>,
    receiver_siblings: Vec<super::receiver_state::Metadata>,
    receiver_configs: Vec<(PathBuf, super::config_state::ChildConfig)>,
    receiver_effects: Vec<super::config_state::ReceiverEffects>,
    receiver_auxiliary: Vec<Vec<PathBuf>>,
    receiver_rebase: Vec<(PathBuf, bool, super::receiver_state::RebaseState)>,
    receiver_sibling_refs: Vec<(PathBuf, super::receiver_state::ReferenceTree)>,
    receiver_sibling_targets: Vec<super::receiver_state::SiblingTargets>,
    receiver_storage: Vec<super::receiver_storage::Storage>,
    auxiliary_paths: Option<super::auxiliary_paths::AuxiliaryPaths>,
    sender_mutation_facets: Vec<ToolPermissionFacet>,
    checkout_target: Option<super::checkout_state::TargetTree>,
    checkout_guessed: bool,
    checkout_tracking_source: Option<(String, String)>,
    checkout_tracking_writes: Vec<super::target_config::Record>,
    checkout_new_branch: Option<String>,
    checkout_parents: Option<super::checkout_state::TargetParents>,
    checkout_empty_gitlinks: Vec<(PathBuf, snapshot::PathIdentity)>,
    checkout_new_gitlinks: Vec<(PathBuf, NewGitlinkDirectory)>,
    checkout_child_layouts: Vec<super::receiver_state::Root>,
    checkout_child_metadata: Vec<super::receiver_state::Metadata>,
    checkout_child_bootstrap: Vec<(PathBuf, super::config_state::ChildConfig)>,
    checkout_child_worktrees: Vec<(PathBuf, PathBuf, snapshot::PathIdentity)>,
    checkout_child_startup: Vec<super::config_state::ChildStartupLayout>,
    checkout_pending_new_gitlinks: Vec<(PathBuf, snapshot::PathIdentity)>,
    checkout_deletion: Option<super::checkout_deletion::Inventory>,
    target_config_sources: Option<super::target_config::Sources>,
    target_config_roots: Vec<PathBuf>,
    target_hook_paths: Option<Vec<PathBuf>>,
    target_hook_evidence: Vec<(PathBuf, snapshot::RegularEvidence)>,
    native_pull_command: Option<command::FrozenCommand>,
    native_add_command: Option<command::FrozenCommand>,
    native_commit_command: Option<command::FrozenCommand>,
    identity: PreparedPlanIdentity,
    workspace: PathBuf,
    workspace_review_root: PathBuf,
    input: Value,
    provenance: ToolProvenance,
    repository: RepositoryState,
    paths: Vec<String>,
    sources: Vec<snapshot::Entry>,
    gitlinks: Vec<GitlinkEvidence>,
    facets: Vec<ToolPermissionFacet>,
    environment: BTreeMap<OsString, OsString>,
    program: PathBuf,
    program_entry: ProgramEvidence,
    maintenance_program: Option<PathBuf>,
    sender_program_candidate: Option<PathBuf>,
    sender_entry_program: Option<(PathBuf, ProgramEvidence)>,
    sender_program_binding: Option<SenderProgramBinding>,
    config: Option<super::config_state::ConfigState>,
    child_configs: Vec<(PathBuf, super::config_state::ChildConfig)>,
    child_locations: Vec<(PathBuf, super::repository_state::ChildLocation)>,
    child_attributes: std::collections::BTreeSet<PathBuf>,
    child_attributes_discovered: bool,
}

// Retained by the consumed plan across discovery/continuation. Reserve worst-case
// stdout + stderr capacity before spawning; failures and cancellation do not
// refund it. This counts Talos-spawned commands, not Git's internal descendants.
struct CommandBudget {
    commands: usize,
    output_bytes: u64,
}

impl Default for CommandBudget {
    fn default() -> Self {
        Self {
            commands: 128,
            output_bytes: 64 * 1024 * 1024,
        }
    }
}

impl CommandBudget {
    fn reserve(&mut self, per_pipe_limit: u64) -> Result<(), GitToolError> {
        let exhausted = || {
            GitToolError::InvalidInput(
                "Git cumulative command/output budget exhausted; no further child started".into(),
            )
        };
        let commands = self.commands.checked_sub(1).ok_or_else(exhausted)?;
        let reserved_bytes = per_pipe_limit.checked_mul(2).ok_or_else(exhausted)?;
        let output_bytes = self
            .output_bytes
            .checked_sub(reserved_bytes)
            .ok_or_else(exhausted)?;
        self.commands = commands;
        self.output_bytes = output_bytes;
        Ok(())
    }
}

impl AddPlan {
    #[cfg(any(unix, windows))]
    pub(super) fn bind_operation(mut self, execution: super::OwnedCommandOperation) -> Self {
        self.execution = execution;
        self
    }

    async fn run_command(
        &self,
        command: tokio::process::Command,
        deadline: std::time::Duration,
        limit: u64,
    ) -> Result<Vec<u8>, GitToolError> {
        let output = self.run_command_output(command, deadline, limit).await?;
        if !output.status.success() {
            let bytes = if output.stderr.is_empty() {
                &output.stdout
            } else {
                &output.stderr
            };
            return Err(GitToolError::Git(
                String::from_utf8_lossy(bytes).trim().to_owned(),
            ));
        }
        Ok(output.stdout)
    }

    // A probe's documented nonzero exit is data, but spawning, timeout,
    // cancellation and collection failures remain errors from the owned runner.
    async fn run_command_output(
        &self,
        command: tokio::process::Command,
        deadline: std::time::Duration,
        limit: u64,
    ) -> Result<std::process::Output, GitToolError> {
        self.run_command_output_with_input(command, deadline, limit, None)
            .await
    }

    async fn run_command_output_with_input(
        &self,
        command: tokio::process::Command,
        deadline: std::time::Duration,
        limit: u64,
        input: Option<Vec<u8>>,
    ) -> Result<std::process::Output, GitToolError> {
        self.command_budget
            .lock()
            .map_err(|_| GitToolError::Git("Git command budget unavailable".into()))?
            .reserve(limit)?;
        #[cfg(unix)]
        {
            let mut command = command;
            crate::process_boundary::apply_process_hardening(&mut command);
            let limit = usize::try_from(limit).unwrap_or(usize::MAX);
            let result = match input {
                Some(input) => {
                    self.execution
                        .execute_with_input(command.into_std(), deadline, limit, input)
                        .await
                }
                None => {
                    self.execution
                        .execute(command.into_std(), deadline, limit)
                        .await
                }
            };
            result
                .map_err(|error| GitToolError::Git(format!("host Git execution failed: {error}; effects may be partial; inspect repository before retrying")))
        }
        #[cfg(windows)]
        {
            let limit = usize::try_from(limit).unwrap_or(usize::MAX);
            let result = match input {
                Some(input) => {
                    self.execution
                        .execute_with_input(command.into_std(), deadline, limit, input)
                        .await
                }
                None => {
                    self.execution
                        .execute(command.into_std(), deadline, limit)
                        .await
                }
            };
            result.map_err(|error| GitToolError::Git(format!("host Git execution failed: {error}; effects may be partial; inspect repository before retrying")))
        }
        #[cfg(not(any(unix, windows)))]
        {
            let _ = (command, deadline, limit, input);
            Err(GitToolError::InvalidInput(
                "Structured Git process-tree supervision is unsupported on this platform".into(),
            ))
        }
    }

    pub(super) fn prepare(
        workspace: &Path,
        input: &Value,
        provenance: ToolProvenance,
    ) -> Result<Self, GitToolError> {
        let parsed: GitAddInput = serde_json::from_value(input.clone())
            .map_err(|error| GitToolError::InvalidInput(error.to_string()))?;
        Self::prepare_operation(
            workspace,
            input,
            provenance,
            Operation::Add,
            Some(parsed.paths),
        )
    }

    pub(super) fn prepare_commit(
        workspace: &Path,
        input: &Value,
        provenance: ToolProvenance,
    ) -> Result<Self, GitToolError> {
        let parsed: GitCommitInput = serde_json::from_value(input.clone())
            .map_err(|error| GitToolError::InvalidInput(error.to_string()))?;
        if parsed.message.contains('\0') {
            return Err(GitToolError::InvalidInput(
                "Git commit message contains NUL".into(),
            ));
        }
        Self::prepare_operation(
            workspace,
            input,
            provenance,
            Operation::Commit(parsed),
            None,
        )
    }

    pub(super) fn prepare_push(
        workspace: &Path,
        input: &Value,
        provenance: ToolProvenance,
    ) -> Result<Self, GitToolError> {
        let parsed: GitPushInput = serde_json::from_value(input.clone())
            .map_err(|error| GitToolError::InvalidInput(error.to_string()))?;
        super::validate_git_positional(parsed.remote.as_deref().unwrap_or("origin"))?;
        if let Some(branch) = &parsed.branch {
            super::validate_git_positional(branch)?;
        }
        Self::prepare_operation(workspace, input, provenance, Operation::Push(parsed), None)
    }

    pub(super) fn prepare_pull(
        workspace: &Path,
        input: &Value,
        provenance: ToolProvenance,
    ) -> Result<Self, GitToolError> {
        let parsed: GitPullInput = serde_json::from_value(input.clone())
            .map_err(|error| GitToolError::InvalidInput(error.to_string()))?;
        if let Some(remote) = &parsed.remote {
            super::validate_git_positional(remote)?;
        }
        Self::prepare_operation(workspace, input, provenance, Operation::Pull(parsed), None)
    }

    pub(super) fn prepare_checkout(
        workspace: &Path,
        input: &Value,
        provenance: ToolProvenance,
    ) -> Result<Self, GitToolError> {
        let parsed: GitCheckoutInput = serde_json::from_value(input.clone())
            .map_err(|e| GitToolError::InvalidInput(e.to_string()))?;
        super::validate_git_positional(if parsed.branch == "-" && !parsed.create {
            "@{-1}"
        } else {
            &parsed.branch
        })?;
        Self::prepare_operation(
            workspace,
            input,
            provenance,
            Operation::Checkout(parsed),
            None,
        )
    }

    fn prepare_operation(
        workspace: &Path,
        input: &Value,
        provenance: ToolProvenance,
        operation: Operation,
        selected: Option<Vec<String>>,
    ) -> Result<Self, GitToolError> {
        if !cfg!(any(unix, windows)) {
            return Err(GitToolError::InvalidInput(
                "Structured Git process-tree supervision is unsupported on this platform".into(),
            ));
        }
        let environment = execution_environment();
        validate_environment(&environment)?;
        let environment = trace_suppressed_environment(environment);
        let program = resolve_git(&environment)?;
        let program_entry = capture_program(&program)?;
        let repository = RepositoryState::capture(workspace)?;
        let commit_state = if matches!(operation, Operation::Commit(_) | Operation::Checkout(_)) {
            Some(super::repository_state::CommitState::capture(&repository)?)
        } else {
            None
        };
        let config = None;
        let mut gitlinks = Vec::new();
        let paths = match &operation {
            Operation::Push(_) | Operation::Pull(_) => Vec::new(),
            Operation::Add => literal_add_paths(
                workspace,
                &repository.workdir,
                &selected.unwrap_or_default(),
            )?,
            Operation::Commit(_) | Operation::Checkout(_) => {
                let tracked = tracked_commit_sources(&repository.workdir)?;
                gitlinks = tracked.1;
                tracked.0
            }
        };
        let sources = match &operation {
            Operation::Push(_) | Operation::Pull(_) => Vec::new(),
            Operation::Add => capture_add_sources(&repository.workdir, &paths)?,
            Operation::Commit(_) | Operation::Checkout(_) => {
                capture_tracked_sources(&repository.workdir, &paths)?
            }
        };
        let mut facets = Vec::new();
        if matches!(
            operation,
            Operation::Checkout(_) | Operation::Push(_) | Operation::Pull(_)
        ) {
            for path in [
                &repository.workdir,
                &repository.git_dir,
                &repository.common_dir,
            ] {
                facets.push(path_facet(
                    ToolNature::Read,
                    path,
                    if matches!(operation, Operation::Pull(_)) {
                        "Pull sender repository roots; configuration discovery only, with no fetch or integration authority"
                    } else if matches!(operation, Operation::Push(_)) {
                        "push sender references and objects; worktree directory enumeration and attribute rules for pack delta selection"
                    } else {
                        "checkout target discovery and worktree evidence"
                    },
                )?);
            }
            if matches!(operation, Operation::Pull(_)) {
                facets.push(ToolPermissionFacet::with_resource(
                    ToolNature::Read,
                    "git:pull-bounded-discovery",
                    ToolResourceKind::Remote,
                ).with_description("bounded native Pull discovery within displayed repository roots; 4096 entries / 4 MiB shared; no writes, hooks or network"));
                facets.push(ToolPermissionFacet::with_resource(
                    ToolNature::Read,
                    "git:pull-repository-metadata",
                    ToolResourceKind::Remote,
                ));
            }
        }
        for (path, entry) in paths.iter().zip(&sources) {
            let source = repository.workdir.join(path);
            // Git records the final link entry, not its target. The permission
            // engine canonicalizes Path resources, so admit its containing
            // directory rather than pretending to read the target.
            let resource = if matches!(entry, snapshot::Entry::Link(_)) {
                source.parent().ok_or_else(|| {
                    GitToolError::InvalidInput("staging entry has no parent".into())
                })?
            } else {
                &source
            };
            facets.push(path_facet(
                ToolNature::Read,
                resource,
                "selected staging source entries",
            )?);
        }
        for link in &gitlinks {
            let source = repository.workdir.join(&link.path);
            let resource = if matches!(link.entry, snapshot::Entry::Link(_)) {
                source.parent().ok_or_else(|| {
                    GitToolError::InvalidInput("gitlink entry has no parent".into())
                })?
            } else {
                &source
            };
            facets.push(path_facet(
                ToolNature::Read,
                resource,
                "tracked submodule entry and checkout metadata",
            )?);
            if let Some(child) = &link.metadata_path {
                facets.push(path_facet(
                    ToolNature::Read,
                    child,
                    "submodule HEAD and index metadata",
                )?);
            }
        }
        facets.push(path_facet(
            ToolNature::Write,
            &repository.git_dir,
            "index and worktree Git metadata",
        )?);
        if repository.common_dir != repository.git_dir {
            facets.push(path_facet(
                ToolNature::Write,
                &repository.common_dir,
                "shared Git object metadata",
            )?);
        }
        facets.push(
            ToolPermissionFacet::with_resource(
                ToolNature::Execute,
                program.to_str().ok_or_else(|| {
                    GitToolError::InvalidInput("Git program path is not UTF-8".into())
                })?,
                ToolResourceKind::Command,
            )
            .with_description(format!("{}; diagnostic trace2 file/socket output disabled in structured children", match &operation {
                Operation::Push(_) => "host Git push with resolved remote destinations",
                Operation::Pull(_) => "host Git Pull configuration discovery; fetch and integration require subsequent admissions",
                Operation::Add => "host Git literal-path staging",
                Operation::Checkout(_) => "host Git branch checkout with repository-wide worktree effects",
                Operation::Commit(_) => "host Git commit; maintenance.autoDetach=false and gc.autoDetach=false keep maintenance in the foreground",
            })),
        );
        // Closed named capability: only the fixed discovery stage consumes it.
        // It is not Path authority and cannot be used by generic file tools.
        facets.retain(|facet| facet.nature != ToolNature::Write);
        facets.push(ToolPermissionFacet::with_resource(
            ToolNature::Read, "git:configuration-chain", ToolResourceKind::Remote,
        ).with_description("discover Git system/global/include configuration internally; no general file access"));
        facets.push(ToolPermissionFacet::with_resource(
            ToolNature::Read,
            "git:auxiliary-path-resolution",
            ToolResourceKind::Remote,
        ).with_description("resolve frozen local Git auxiliary paths; file contents require separate exact Read admission"));
        facets.push(path_facet(
            ToolNature::Read,
            &repository.git_dir,
            "source anchor for the fixed Git auxiliary path resolver",
        )?);
        if matches!(operation, Operation::Pull(_)) {
            facets.push(ToolPermissionFacet::with_resource(
                ToolNature::Read,
                "git:pull-worktree-resolution",
                ToolResourceKind::Remote,
            ).with_description("resolve frozen Pull gitdir/commondir and startup core.worktree paths; actual external targets require separate Read admission"));
        }
        let native_pull_command = match &operation {
            Operation::Pull(pull) => Some(command::FrozenCommand::pull(
                &program,
                &repository.workdir,
                pull,
                &environment,
            )?),
            _ => None,
        };
        let native_add_command = if matches!(operation, Operation::Add) {
            Some(command::FrozenCommand::add(
                &program,
                &repository.workdir,
                &paths,
                &environment,
            )?)
        } else {
            None
        };
        let native_commit_command = match &operation {
            Operation::Commit(input) => Some(command::FrozenCommand::commit(
                &program,
                &repository.workdir,
                input,
                &environment,
            )?),
            _ => None,
        };
        Ok(Self {
            phase: Phase::SenderConfiguration,
            command_budget: std::sync::Mutex::new(CommandBudget::default()),
            #[cfg(any(unix, windows))]
            execution: super::OwnedCommandExecutor::default().operation(),
            operation,
            commit_state,
            push_destinations: Vec::new(),
            receiver_roots: Vec::new(),
            receiver_child_paths: Vec::new(),
            receiver_child_executables: Vec::new(),
            receiver_layouts: Vec::new(),
            receiver_metadata: Vec::new(),
            receiver_common: Vec::new(),
            receiver_worktree_inventories: Vec::new(),
            receiver_siblings: Vec::new(),
            receiver_configs: Vec::new(),
            receiver_effects: Vec::new(),
            receiver_auxiliary: Vec::new(),
            receiver_rebase: Vec::new(),
            receiver_sibling_refs: Vec::new(),
            receiver_sibling_targets: Vec::new(),
            receiver_storage: Vec::new(),
            auxiliary_paths: None,
            sender_mutation_facets: Vec::new(),
            checkout_target: None,
            checkout_guessed: false,
            checkout_tracking_source: None,
            checkout_tracking_writes: Vec::new(),
            checkout_new_branch: None,
            checkout_parents: None,
            checkout_empty_gitlinks: Vec::new(),
            checkout_new_gitlinks: Vec::new(),
            checkout_child_layouts: Vec::new(),
            checkout_child_metadata: Vec::new(),
            checkout_child_bootstrap: Vec::new(),
            checkout_child_worktrees: Vec::new(),
            checkout_child_startup: Vec::new(),
            checkout_pending_new_gitlinks: Vec::new(),
            checkout_deletion: None,
            target_config_sources: None,
            target_config_roots: Vec::new(),
            target_hook_paths: None,
            target_hook_evidence: Vec::new(),
            native_pull_command,
            native_add_command,
            native_commit_command,
            identity: PreparedPlanIdentity::new(),
            workspace: workspace.to_owned(),
            workspace_review_root: workspace
                .canonicalize()
                .map_err(|error| GitToolError::InvalidInput(error.to_string()))?,
            input: input.clone(),
            provenance,
            repository,
            paths,
            sources,
            gitlinks,
            facets,
            environment,
            program,
            program_entry,
            maintenance_program: None,
            sender_program_candidate: None,
            sender_entry_program: None,
            sender_program_binding: None,
            config,
            child_configs: Vec::new(),
            child_locations: Vec::new(),
            child_attributes: std::collections::BTreeSet::new(),
            child_attributes_discovered: false,
        })
    }

    fn request(&self) -> PermissionRequest<'_> {
        PermissionRequest::new(
            self.operation.name(),
            self.provenance.clone(),
            &self.facets,
            &self.input,
        )
    }

    async fn inspect_maintenance_program(&self) -> Result<PathBuf, GitToolError> {
        let mut command = tokio::process::Command::new(&self.program);
        command
            .current_dir(&self.repository.workdir)
            .arg("--exec-path")
            .env_clear()
            .envs(&self.environment);
        let output = self
            .run_command(command, std::time::Duration::from_secs(10), 16 * 1024)
            .await?;
        let path = output
            .strip_suffix(b"\n")
            .map(|path| path.strip_suffix(b"\r").unwrap_or(path))
            .and_then(|path| std::str::from_utf8(path).ok())
            .map(PathBuf::from)
            .filter(|path| path.is_absolute() && !path.as_os_str().as_encoded_bytes().contains(&0))
            .ok_or_else(|| {
                GitToolError::InvalidInput("Git child executable directory is unavailable".into())
            })?;
        Ok(path.join(if cfg!(windows) { "git.exe" } else { "git" }))
    }

    async fn request_sender_program(mut self: Box<Self>) -> Result<Box<Self>, GitToolError> {
        self.revalidate()?;
        let candidate = self.inspect_maintenance_program().await?;
        // Keep all original Read frontiers; first full sender discovery still
        // needs the original sources and repository evidence.
        self.facets
            .retain(|facet| facet.nature != ToolNature::Execute);
        self.facets.push(path_facet(
            ToolNature::Read,
            &self.program,
            "entry Git executable drift evidence; no further execution",
        )?);
        self.facets.push(path_facet(
            ToolNature::Read,
            &candidate,
            "internal Git executable identity before first sender discovery",
        )?);
        self.facets.push(ToolPermissionFacet::with_resource(
            ToolNature::Execute,
            candidate.to_str().ok_or_else(|| {
                GitToolError::InvalidInput("Internal Git executable path is not UTF-8".into())
            })?,
            ToolResourceKind::Command,
        ));
        self.sender_program_candidate = Some(candidate);
        self.identity = PreparedPlanIdentity::new();
        Ok(self)
    }

    async fn discover_sender(
        mut self: Box<Self>,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<Box<Self>, GitToolError> {
        if let Some(candidate) = self.sender_program_candidate.take() {
            self.revalidate()?;
            let admitted = self.receiver_read_path(&candidate, authorizations)?;
            let bound = ReceiverChildExecutable::capture(&candidate, admitted)?;
            let invocation = if cfg!(windows) {
                bound.spelling.clone()
            } else {
                bound.canonical.clone()
            };
            let old_path = std::mem::replace(&mut self.program, invocation);
            let old_entry = std::mem::replace(&mut self.program_entry, bound.program);
            self.sender_entry_program = Some((old_path, old_entry));
            self.sender_program_binding = Some(SenderProgramBinding {
                spelling: bound.spelling,
                canonical: bound.canonical,
                identity: bound.identity,
            });
            // A second migration is never allowed. Do this before configuration,
            // target or child discovery through the newly admitted executable.
            let maintenance = self.inspect_maintenance_program().await?;
            validate_internal_program(&maintenance, &candidate, self.sender_program_target())?;
            self.revalidate()?;
        } else if cfg!(windows)
            && self.sender_entry_program.is_none()
            && matches!(
                self.operation,
                Operation::Commit(_) | Operation::Checkout(_)
            )
        {
            return self.request_sender_program().await;
        }
        self.discover_admitted(authorizations).await
    }

    fn add_push_program_admission(&mut self) -> Result<(), GitToolError> {
        let candidate = self.maintenance_program.as_ref().ok_or_else(|| {
            GitToolError::InvalidInput("Push internal Git candidate missing".into())
        })?;
        self.facets.push(path_facet(
            ToolNature::Read,
            &self.program,
            "entry Git executable retained for drift checks",
        )?);
        self.facets.push(path_facet(
            ToolNature::Read,
            candidate,
            "internal Git executable before Push re-discovery",
        )?);
        self.facets.push(ToolPermissionFacet::with_resource(
            ToolNature::Execute,
            candidate.to_str().ok_or_else(|| {
                GitToolError::InvalidInput("Internal Git path is not UTF-8".into())
            })?,
            ToolResourceKind::Command,
        ));
        self.facets.push(
            ToolPermissionFacet::with_resource(
                ToolNature::Read,
                "git:configuration-chain",
                ToolResourceKind::Remote,
            )
            .with_description("fresh internal Git configuration discovery before receiver reads"),
        );
        self.facets.push(ToolPermissionFacet::with_resource(
            ToolNature::Read,
            "git:auxiliary-path-resolution",
            ToolResourceKind::Remote,
        ));
        self.facets.push(path_facet(
            ToolNature::Read,
            &self.repository.git_dir,
            "source anchor for the fixed Git auxiliary path resolver",
        )?);
        Ok(())
    }

    async fn rebind_push_program(
        mut self: Box<Self>,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<Box<Self>, GitToolError> {
        let Operation::Push(push) = &self.operation else {
            return Err(GitToolError::InvalidInput(
                "Push re-discovery requires Push".into(),
            ));
        };
        if !self.paths.is_empty()
            || !self.child_attributes.is_empty()
            || !self.child_configs.is_empty()
            || self.sender_entry_program.is_some()
            || !self.facets.iter().any(|facet| {
                facet.resource.as_deref() == Some("git:configuration-chain")
                    && facet.nature == ToolNature::Read
            })
        {
            return Err(GitToolError::InvalidInput(
                "Invalid Push re-discovery state".into(),
            ));
        }
        let remote = push.remote.as_deref().unwrap_or("origin").to_owned();
        self.revalidate()?;
        let candidate = self.maintenance_program.clone().ok_or_else(|| {
            GitToolError::InvalidInput("Push internal Git candidate missing".into())
        })?;
        let admitted = self.receiver_read_path(&candidate, authorizations)?;
        let bound = ReceiverChildExecutable::capture(&candidate, admitted)?;
        let invocation = if cfg!(windows) {
            bound.spelling.clone()
        } else {
            bound.canonical.clone()
        };
        let old_path = std::mem::replace(&mut self.program, invocation);
        let old_entry = std::mem::replace(&mut self.program_entry, bound.program);
        self.sender_entry_program = Some((old_path.clone(), old_entry));
        self.sender_program_binding = Some(SenderProgramBinding {
            spelling: bound.spelling,
            canonical: bound.canonical,
            identity: bound.identity,
        });
        let maintenance = self.inspect_maintenance_program().await?;
        validate_internal_program(&maintenance, &candidate, self.sender_program_target())?;
        // Only the fresh configuration-chain proof covers newly reported config
        // files. Other newly reported paths are compared, never opened here.
        let output = self.inspect_config().await?;
        let mut config = super::config_state::ConfigState::capture_host(
            &output,
            &self.repository,
            &self.environment,
        )?;
        super::remote_state::reject_legacy_remotes(&self.repository.common_dir)?;
        config.prepare_push(&self.repository, &self.environment, &remote)?;
        let destinations = super::remote_state::destinations(
            &super::remote_state::records(&output)?,
            &remote,
            true,
        )?;
        let mut roots = Vec::new();
        for destination in &destinations {
            if let Some(path) =
                super::remote_state::local_destination(destination, &self.repository.workdir)?
            {
                roots.push(path);
            }
        }
        if destinations != self.push_destinations || roots != self.receiver_roots {
            return Err(GitToolError::InvalidInput(
                "Push destinations changed with internal Git".into(),
            ));
        }
        for variable in ["GIT_ATTR_SYSTEM", "GIT_ATTR_GLOBAL"] {
            let path = self
                .inspect_attribute_path_at(&self.repository.workdir, variable)
                .await?;
            config.stable_auxiliary.insert(path.clone());
            config.auxiliary.insert(path);
        }
        for variable in ["GIT_CONFIG_SYSTEM", "GIT_CONFIG_GLOBAL"] {
            let paths = self.inspect_config_paths(variable).await?;
            config.stable_auxiliary.extend(paths.iter().cloned());
            config.auxiliary.extend(paths);
        }
        config.verify_host(&self.inspect_config().await?)?;
        if Some(&config) != self.config.as_ref() {
            return Err(GitToolError::InvalidInput(
                "Push configuration or auxiliary sources changed with internal Git".into(),
            ));
        }
        let mut children = Vec::new();
        if !roots.is_empty() {
            children.push(self.inspect_git_shell_path().await?);
            if !children.contains(&candidate) {
                children.push(candidate);
            }
        }
        if children != self.receiver_child_paths {
            return Err(GitToolError::InvalidInput(
                "Push child executable paths changed with internal Git".into(),
            ));
        }
        self.config = Some(config);
        let entry_read = path_facet(
            ToolNature::Read,
            &old_path,
            "entry Git executable retained for drift checks",
        )?;
        let new_program = self
            .program
            .to_str()
            .ok_or_else(|| GitToolError::InvalidInput("Internal Git path is not UTF-8".into()))?;
        for facets in [&mut self.facets, &mut self.sender_mutation_facets] {
            facets.retain(|facet| facet.resource.as_deref() != Some("git:configuration-chain"));
            for facet in facets.iter_mut() {
                if facet.nature == ToolNature::Execute
                    && facet.resource.as_deref() == old_path.to_str()
                {
                    facet.resource = Some(new_program.to_owned());
                }
            }
            facets.push(entry_read.clone());
        }
        self.revalidate()?;
        Ok(self)
    }

    async fn discover_admitted(
        mut self: Box<Self>,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<Box<Self>, GitToolError> {
        self.revalidate()?;
        if matches!(
            &self.operation,
            Operation::Commit(_) | Operation::Checkout(_)
        ) {
            for link in &self.gitlinks {
                let Some(metadata_path) = &link.metadata_path else {
                    continue;
                };
                let location = super::repository_state::ChildLocation::capture(metadata_path)?;
                self.child_attributes
                    .insert(location.common_dir.join("config"));
                self.child_attributes
                    .insert(location.git_dir.join("config.worktree"));
                let workdir = self.repository.workdir.join(&link.path);
                let output = self.inspect_config_at(&workdir).await?;
                self.child_attributes
                    .extend(super::config_state::declared_include_paths(
                        &output,
                        &workdir,
                        &self.environment,
                    )?);
                let config = super::config_state::ChildConfig::capture(&output, &workdir)?;
                config.verify_host(&self.inspect_config_at(&workdir).await?)?;
                self.child_attributes.extend(config.attribute_paths(
                    &workdir,
                    &location.common_dir,
                    &self.environment,
                )?);
                for variable in ["GIT_ATTR_SYSTEM", "GIT_ATTR_GLOBAL"] {
                    self.child_attributes
                        .insert(self.inspect_attribute_path_at(&workdir, variable).await?);
                }
                if super::repository_state::ChildLocation::capture(metadata_path)? != location {
                    return Err(GitToolError::InvalidInput(
                        "Submodule location changed during configuration discovery".into(),
                    ));
                }
                self.child_configs.push((workdir, config));
                self.child_locations.push((metadata_path.clone(), location));
            }
        }
        let output = self.inspect_config().await?;
        let mut config = super::config_state::ConfigState::capture_host(
            &output,
            &self.repository,
            &self.environment,
        )?;
        if let Operation::Push(push) = &self.operation {
            super::remote_state::reject_legacy_remotes(&self.repository.common_dir)?;
            config.prepare_push(
                &self.repository,
                &self.environment,
                push.remote.as_deref().unwrap_or("origin"),
            )?;
            self.push_destinations = super::remote_state::destinations(
                &super::remote_state::records(&output)?,
                push.remote.as_deref().unwrap_or("origin"),
                true,
            )?;
        }
        if matches!(self.operation, Operation::Commit(_)) {
            config.prepare_commit(
                &self.repository,
                &self.environment,
                self.commit_state
                    .as_ref()
                    .ok_or_else(|| GitToolError::Git("missing commit state evidence".into()))?,
            )?;
        }
        if matches!(
            self.operation,
            Operation::Commit(_) | Operation::Checkout(_) | Operation::Push(_)
        ) {
            self.maintenance_program = Some(self.inspect_maintenance_program().await?);
        }
        if let Operation::Checkout(checkout) = &self.operation {
            let (resolved, tracking_source) =
                self.resolve_checkout_target(checkout, &config).await?;
            let guessed = tracking_source.is_some();
            self.checkout_guessed = guessed;
            self.checkout_tracking_source = tracking_source;
            if checkout.create || guessed {
                self.checkout_new_branch =
                    Some(self.inspect_checkout_branch(&checkout.branch).await?);
                let start = self.inspect_checkout_start(checkout.create).await?;
                self.checkout_tracking_writes = config.checkout_tracking_writes(
                    self.checkout_new_branch
                        .as_deref()
                        .ok_or_else(|| GitToolError::Git("Missing checkout branch name".into()))?,
                    start.as_deref(),
                )?;
            }
            let oid = resolved.as_str();
            let symbolic_context = checkout.branch == "HEAD"
                || checkout.branch == "@"
                || checkout.branch == "-"
                || checkout
                    .branch
                    .strip_prefix("@{-")
                    .and_then(|value| value.strip_suffix('}'))
                    .is_some_and(|number| {
                        !number.is_empty() && number.bytes().all(|byte| byte.is_ascii_digit())
                    });
            let previous_branch = if !checkout.create && symbolic_context {
                let mut command = tokio::process::Command::new(&self.program);
                command
                    .current_dir(&self.repository.workdir)
                    .args([
                        "rev-parse",
                        "--symbolic-full-name",
                        "--verify",
                        "--end-of-options",
                        if checkout.branch == "-" {
                            "@{-1}"
                        } else {
                            &checkout.branch
                        },
                    ])
                    .env_clear()
                    .envs(&self.environment);
                let output = self
                    .run_command(command, std::time::Duration::from_secs(10), 4096)
                    .await?;
                let reference = std::str::from_utf8(&output)
                    .map_err(|_| {
                        GitToolError::InvalidInput("Invalid previous checkout reference".into())
                    })?
                    .trim();
                reference.strip_prefix("refs/heads/").map(str::to_owned)
            } else {
                None
            };
            let local_name = previous_branch.as_deref().unwrap_or(&checkout.branch);
            let local = if !checkout.create && local_name != "-" {
                super::checkout_state::LocalTarget::capture(&self.repository.workdir, local_name)?
            } else {
                None
            };
            if local.as_ref().is_some_and(|local| local.commit != oid) {
                return Err(GitToolError::InvalidInput(
                    "Checkout local target changed during discovery".into(),
                ));
            }
            let target_discovery = local.is_none() && config.has_branch_conditionals();
            if target_discovery {
                let mut roots = config
                    .files
                    .iter()
                    .map(|(path, _)| path.clone())
                    .collect::<Vec<_>>();
                // git-var reports system/global roots in native read order.
                // Included sources remain capture dependencies, never roots.
                let mut ordered_roots = Vec::new();
                for variable in ["GIT_CONFIG_SYSTEM", "GIT_CONFIG_GLOBAL"] {
                    ordered_roots.extend(self.inspect_config_paths(variable).await?);
                }
                ordered_roots.push(self.repository.common_dir.join("config"));
                roots.extend(ordered_roots.iter().cloned());
                self.target_config_roots = ordered_roots;
                let branch = if checkout.create || guessed {
                    self.checkout_new_branch.as_deref().map(str::as_bytes)
                } else {
                    local
                        .as_ref()
                        .and_then(|local| local.reference.strip_prefix("refs/heads/"))
                        .map(str::as_bytes)
                };
                let mut sources = super::target_config::Sources::new(branch, roots)?;
                sources.add_directory_context(&self.repository.git_dir)?;
                self.target_config_sources = Some(sources);
            }
            // Source hooks are still selected from the invocation config. A
            // pending target chain cannot proceed to mutation until its effects
            // gate has independently completed.
            config.prepare_checkout(
                &self.repository,
                &self.environment,
                local.is_some() || target_discovery,
            )?;
            let target = super::checkout_state::TargetTree::capture(&self.repository.workdir, oid)?;
            let target_links: std::collections::BTreeSet<_> = target
                .entries
                .iter()
                .filter(|(_, (mode, _))| *mode == 0o160000)
                .map(|(path, _)| path.as_str())
                .collect();
            let current_links: std::collections::BTreeSet<_> = self
                .gitlinks
                .iter()
                .map(|link| link.path.as_str())
                .collect();
            for added in target_links.difference(&current_links) {
                // A missing new gitlink becomes an uninitialized directory:
                // there is no child repository whose configuration or status
                // could be inspected by checkout. Its absence is retained in
                // the target source evidence and checked again before spawn.
                match std::fs::symlink_metadata(self.repository.workdir.join(added)) {
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => return Err(GitToolError::InvalidInput(error.to_string())),
                    Ok(metadata) if metadata.is_dir() => {
                        let path = self.repository.workdir.join(added);
                        let identity = snapshot::PathIdentity::capture(&path)
                            .map_err(|error| GitToolError::InvalidInput(error.to_string()))?;
                        self.checkout_pending_new_gitlinks.push((path, identity));
                    }
                    Ok(metadata) if metadata.is_file() && self.paths.iter().any(|path| path == added) => {}
                    #[cfg(unix)]
                    Ok(metadata) if metadata.file_type().is_symlink()
                        && self.paths.iter().any(|path| path == added)
                        && tracked_symlink(&self.repository.workdir, added)? => {}
                    Ok(_) => return Err(GitToolError::InvalidInput("Checkout introduces a submodule at an existing path; child discovery requires a separately authorized invocation".into())),
                }
            }
            let mut deletion_roots = Vec::new();
            for removed in current_links.difference(&target_links) {
                // A missing, uninitialized gitlink has no child state to
                // preserve. Its absence is bound by retained gitlink evidence.
                if self.gitlinks.iter().any(|link| {
                    link.path == *removed
                        && link.metadata_path.is_none()
                        && matches!(link.entry, snapshot::Entry::Missing)
                }) {
                    continue;
                }
                if target.entries.keys().any(|path| {
                    path == removed
                        || path
                            .strip_prefix(*removed)
                            .is_some_and(|suffix| suffix.starts_with('/'))
                }) {
                    let path = self.repository.workdir.join(removed);
                    if self.gitlinks.iter().any(|link| {
                        link.path == *removed
                            && link.metadata_path.is_none()
                            && matches!(link.entry, snapshot::Entry::Directory(_))
                    }) {
                        if let Ok(identity) = capture_empty_gitlink(&path) {
                            self.checkout_empty_gitlinks.push((path, identity));
                        } else {
                            deletion_roots.push((*removed).to_owned());
                        }
                        continue;
                    }
                    if self.gitlinks.iter().any(|link| {
                        link.path == *removed && matches!(link.entry, snapshot::Entry::Directory(_))
                    }) {
                        deletion_roots.push((*removed).to_owned());
                        continue;
                    }
                    return Err(GitToolError::InvalidInput("Checkout changes a submodule path kind; use a separately authorized invocation".into()));
                }
            }
            if !deletion_roots.is_empty() {
                // These canonical child metadata locations were retained during
                // admitted configuration discovery. Reject a known overlap
                // without enumerating the metadata tree or spending its stages.
                // Aliased/indirect sources still use the final deletion check.
                for root in &deletion_roots {
                    let root = self.repository.workdir.join(root);
                    if self.child_locations.iter().any(|(_, location)| {
                        location.common_dir.join("config").starts_with(&root)
                            || location.git_dir.join("config.worktree").starts_with(&root)
                    }) {
                        return Err(GitToolError::InvalidInput(
                            "Checkout deletion overlaps an effective configuration or auxiliary source"
                                .into(),
                        ));
                    }
                }
                self.checkout_deletion = Some(super::checkout_deletion::Inventory::new(
                    &self.repository.workdir,
                    deletion_roots,
                )?);
            }
            let mut paths: std::collections::BTreeSet<_> = self.paths.iter().cloned().collect();
            paths.extend(target.entries.keys().cloned());
            self.paths = paths.into_iter().collect();
            self.checkout_parents = Some(super::checkout_state::TargetParents::capture(
                &self.repository.workdir,
                &self.paths,
            )?);
            self.sources = capture_tracked_sources(&self.repository.workdir, &self.paths)?;
            self.checkout_target = Some(target);
        }
        // Git reports its compiled installation prefix and global attribute
        // fallback without reading attribute contents. Do not guess either path
        // from the executable location; their reads require the next admission.
        for variable in ["GIT_ATTR_SYSTEM", "GIT_ATTR_GLOBAL"] {
            let path = self
                .inspect_attribute_path_at(&self.repository.workdir, variable)
                .await?;
            config.stable_auxiliary.insert(path.clone());
            config.auxiliary.insert(path);
        }
        config
            .stable_auxiliary
            .extend(self.child_attributes.iter().cloned());
        for (_, child) in &self.child_configs {
            config
                .stable_auxiliary
                .extend(child.files.iter().map(|(path, _)| path.clone()));
        }
        for variable in ["GIT_CONFIG_SYSTEM", "GIT_CONFIG_GLOBAL"] {
            let paths = self.inspect_config_paths(variable).await?;
            config.stable_auxiliary.extend(paths.iter().cloned());
            config.auxiliary.extend(paths);
        }
        let mut leaves = Vec::new();
        for (path, source) in self.paths.iter().zip(&self.sources) {
            selected_leaves(Path::new(path), source, &mut leaves)?;
            selected_directory_rules(
                &self.repository.workdir.join(path),
                source,
                &mut config.auxiliary,
            );
        }
        // Attribute/ignore rules in every source ancestor affect selected files.
        // This only records paths; contents wait for the continuation's admission.
        for path in self.paths.iter().chain(leaves.iter()) {
            let selected = self.repository.workdir.join(path);
            let mut ancestor = selected.parent();
            while let Some(directory) = ancestor {
                if !directory.starts_with(&self.repository.workdir) {
                    break;
                }
                config.auxiliary.insert(directory.join(".gitattributes"));
                config.auxiliary.insert(directory.join(".gitignore"));
                ancestor = directory.parent();
            }
        }
        // Capture and then inspect again: do not accept a mixed configuration snapshot.
        config.verify_host(&self.inspect_config().await?)?;
        let mut auxiliary_sources = config.auxiliary.clone();
        auxiliary_sources.extend(config.files.iter().map(|(path, _)| path.clone()));
        auxiliary_sources.extend(self.child_attributes.iter().cloned());
        auxiliary_sources.extend(
            self.child_configs
                .iter()
                .flat_map(|(_, child)| child.files.iter().map(|(path, _)| path.clone())),
        );
        if let Some(paths) = &mut self.auxiliary_paths {
            paths.extend(self.operation.name(), auxiliary_sources, authorizations)?;
        } else {
            self.auxiliary_paths = Some(super::auxiliary_paths::AuxiliaryPaths::resolve(
                self.operation.name(),
                &self.workspace,
                &self.repository.git_dir,
                auxiliary_sources,
                authorizations,
            )?);
        }
        self.config = Some(config);
        self.revalidate()?;
        self.facets
            .retain(|facet| facet.resource.as_deref() != Some("git:configuration-chain"));
        self.auxiliary_paths
            .as_ref()
            .ok_or_else(|| GitToolError::Git("missing auxiliary path evidence".into()))?
            .add_read_frontier(&mut self.facets)?;
        self.facets.push(path_facet(
            ToolNature::Write,
            &self.repository.git_dir,
            "index and worktree Git metadata",
        )?);
        if self.repository.common_dir != self.repository.git_dir {
            self.facets.push(path_facet(
                ToolNature::Write,
                &self.repository.common_dir,
                "shared Git object metadata",
            )?);
        }
        if matches!(self.operation, Operation::Checkout(_)) {
            self.facets.push(path_facet(
                ToolNature::Write,
                &self.repository.workdir,
                "checkout repository-wide worktree mutation",
            )?);
        }
        self.identity = PreparedPlanIdentity::new();
        if matches!(self.operation, Operation::Commit(_)) {
            self.facets.push(path_facet(ToolNature::Read, &self.repository.workdir,
                "repository-wide status directory enumeration and ignore rules, including untracked directories")?);
        }
        for path in &self.child_attributes {
            self.facets.push(path_facet(
                ToolNature::Read,
                path,
                "submodule effective attribute and exclude sources",
            )?);
        }
        for (_, location) in &self.child_locations {
            if matches!(self.operation, Operation::Checkout(_)) {
                self.facets.push(path_facet(
                    ToolNature::Write,
                    &location.git_dir,
                    "checkout child status may refresh its index",
                )?);
            }
            self.facets.push(path_facet(
                ToolNature::Read,
                &location.common_dir,
                "discovered shared submodule metadata",
            )?);
        }
        for (_, config) in &self.child_configs {
            for (path, _) in &config.files {
                self.facets.push(path_facet(
                    ToolNature::Read,
                    path,
                    "submodule configuration evidence",
                )?);
            }
        }
        if let Some(program) = &self.maintenance_program {
            self.facets.push(path_facet(
                ToolNature::Read,
                program,
                "Git child executable identity for maintenance, submodule status or object transfer",
            )?);
            self.facets.push(ToolPermissionFacet::with_resource(
                ToolNature::Execute, program.to_str().ok_or_else(|| GitToolError::InvalidInput("Git child executable path is not UTF-8".into()))?, ToolResourceKind::Command,
            ).with_description("Git maintenance, submodule status or object transfer; executable must match the bound Git binary"));
        }
        for destination in &self.push_destinations {
            if let Some(path) =
                super::remote_state::local_destination(destination, &self.repository.workdir)?
            {
                self.facets.push(path_facet(
                    ToolNature::Read,
                    &path,
                    "local Git receiver layout discovery; no receiver mutation",
                )?);
                self.receiver_roots.push(path);
                if self.receiver_roots.len() > 64 {
                    return Err(GitToolError::InvalidInput(
                        "Git receiver layout discovery exceeds 64 destinations".into(),
                    ));
                }
                continue;
            }
            self.facets.push(
                ToolPermissionFacet::with_resource(
                    ToolNature::Network,
                    destination.clone(),
                    ToolResourceKind::Remote,
                )
                .with_description(
                    "Git push object transfer and remote reference updates; non-direct transports require additional effect validation",
                ),
            );
        }
        if !self.receiver_roots.is_empty() {
            let shell = self.inspect_git_shell_path().await?;
            self.facets.push(path_facet(
                ToolNature::Read,
                &shell,
                "local Git transport shell executable identity",
            )?);
            self.receiver_child_paths.push(shell);
            let maintenance = self.maintenance_program.as_ref().ok_or_else(|| {
                GitToolError::InvalidInput("Receiver Git child executable missing".into())
            })?;
            if !self.receiver_child_paths.contains(maintenance) {
                self.receiver_child_paths.push(maintenance.clone());
            }
        }
        self.phase = if !self.checkout_pending_new_gitlinks.is_empty() {
            self.sender_mutation_facets = self.facets.clone();
            self.facets.retain(|facet| facet.nature == ToolNature::Read);
            for (path, _) in &self.checkout_pending_new_gitlinks {
                self.facets.push(path_facet(
                    ToolNature::Read,
                    path,
                    "new gitlink top-level name discovery; no mutation authority",
                )?);
            }
            Phase::CheckoutNewGitlinks
        } else if self.target_config_sources.is_some() {
            self.sender_mutation_facets = self.facets.clone();
            self.facets.retain(|facet| facet.nature == ToolNature::Read);
            self.add_target_configuration_reads()?;
            Phase::TargetConfiguration
        } else if self.checkout_deletion.is_some() {
            self.sender_mutation_facets = self.facets.clone();
            self.facets.retain(|facet| facet.nature == ToolNature::Read);
            self.add_checkout_deletion_reads()?;
            Phase::CheckoutDeletion
        } else if self.receiver_roots.is_empty() {
            Phase::Mutation
        } else {
            self.sender_mutation_facets = self.facets.clone();
            self.facets.retain(|facet| facet.nature == ToolNature::Read);
            Phase::ReceiverRoots
        };
        if cfg!(windows) && matches!(self.operation, Operation::Push(_)) {
            if self.phase == Phase::Mutation {
                self.sender_mutation_facets = self.facets.clone();
                self.facets.retain(|facet| facet.nature == ToolNature::Read);
                self.phase = Phase::PushProgram;
            }
            self.add_push_program_admission()?;
        }
        Ok(self)
    }

    /// Direct discovery fixture for legacy unit tests. Production discovery
    /// always receives tokens from the prepared-invocation admission fence.
    #[cfg(test)]
    async fn discover(self: Box<Self>) -> Result<Box<Self>, GitToolError> {
        let proof = tests::proof(&self);
        let authorizations = proof
            .consume_for_prepared(&self.request(), &self.identity)
            .map_err(|error| GitToolError::InvalidInput(error.to_string()))?;
        self.discover_admitted(&authorizations).await
    }

    fn discover_new_gitlinks(
        mut self: Box<Self>,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<Box<Self>, GitToolError> {
        self.revalidate()?;
        for (path, _) in &self.checkout_pending_new_gitlinks {
            if !authorizations.iter().any(|token| {
                token.normalized_path() == path
                    && token.authorizes_path(
                        "git_checkout",
                        ToolNature::Read,
                        &self.workspace,
                        path.to_str().unwrap_or_default(),
                    )
            }) {
                return Err(GitToolError::InvalidInput(
                    "New gitlink directory requires exact Read admission".into(),
                ));
            }
        }
        for (path, expected) in &self.checkout_pending_new_gitlinks {
            if &snapshot::PathIdentity::capture(path)
                .map_err(|error| GitToolError::InvalidInput(error.to_string()))?
                != expected
            {
                return Err(GitToolError::InvalidInput(
                    "New gitlink directory changed during discovery".into(),
                ));
            }
            let evidence = capture_new_gitlink_directory(path, true)?;
            if &evidence.identity != expected {
                return Err(GitToolError::InvalidInput(
                    "New gitlink directory changed during discovery".into(),
                ));
            }
            self.checkout_new_gitlinks.push((path.clone(), evidence));
        }
        self.checkout_pending_new_gitlinks.clear();
        let markers: Vec<_> = self
            .checkout_new_gitlinks
            .iter()
            .filter(|(_, evidence)| evidence.names.iter().any(|name| name == ".git"))
            .map(|(path, _)| path.join(".git"))
            .collect();
        if !markers.is_empty() {
            for marker in markers {
                self.facets.push(path_facet(
                    ToolNature::Read,
                    &marker,
                    "new child Git marker discovery; target metadata remains unapproved",
                )?);
            }
            self.phase = Phase::CheckoutChildMarkers;
            self.identity = PreparedPlanIdentity::new();
            return Ok(self);
        }
        self.finish_new_gitlink_discovery()
    }

    fn finish_new_gitlink_discovery(mut self: Box<Self>) -> Result<Box<Self>, GitToolError> {
        if self.target_config_sources.is_some() {
            self.add_target_configuration_reads()?;
            self.phase = Phase::TargetConfiguration;
        } else if self.checkout_deletion.is_some() {
            self.add_checkout_deletion_reads()?;
            self.phase = Phase::CheckoutDeletion;
        } else {
            self.facets.append(&mut self.sender_mutation_facets);
            self.phase = Phase::Mutation;
        }
        self.identity = PreparedPlanIdentity::new();
        Ok(self)
    }

    fn child_read<'a>(
        &self,
        path: &Path,
        authorizations: &'a [talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<&'a Path, GitToolError> {
        authorizations
            .iter()
            .find(|token| {
                token.authorizes_path(
                    self.operation.name(),
                    ToolNature::Read,
                    &self.workspace,
                    path.to_str().unwrap_or_default(),
                )
            })
            .map(|token| token.normalized_path())
            .ok_or_else(|| {
                GitToolError::InvalidInput("Child path requires separate Read admission".into())
            })
    }

    fn child_backlink_matches(
        &self,
        layout: &super::receiver_state::Root,
        marker: Option<&Path>,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<bool, GitToolError> {
        let Some(marker) = marker else {
            return Ok(true);
        };
        if marker == layout.canonical.join(".git") {
            return Ok(true);
        }
        #[cfg(not(windows))]
        {
            let _ = authorizations;
            Ok(false)
        }
        #[cfg(windows)]
        {
            let admitted = self.child_read(&layout.spelling, authorizations)?;
            self.child_read(&layout.spelling.join(".git"), authorizations)?;
            layout.revalidate(admitted)?;
            // Convert only the already admitted root. Never resolve or open
            // the backlink declaration, which may name an unrelated path.
            let spelling = host_path::directory_argument(&layout.canonical)?;
            let matches = marker == spelling.join(".git");
            layout.revalidate(admitted)?;
            Ok(matches)
        }
    }

    fn discover_checkout_child_markers(
        mut self: Box<Self>,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<Box<Self>, GitToolError> {
        self.revalidate()?;
        let roots: Vec<_> = self
            .checkout_new_gitlinks
            .iter()
            .filter(|(_, evidence)| evidence.names.iter().any(|name| name == ".git"))
            .map(|(path, _)| path.clone())
            .collect();
        // Check the complete frontier before any marker contents are inspected.
        for root in &roots {
            self.child_read(root, authorizations)?;
            self.child_read(&root.join(".git"), authorizations)?;
        }
        for root in roots {
            let layout = super::receiver_state::Root::capture(
                &root,
                self.child_read(&root, authorizations)?,
            )?;
            for path in std::iter::once(layout.metadata_path.clone())
                .chain(["commondir", "gitdir", "HEAD"].map(|leaf| layout.metadata_path.join(leaf)))
            {
                self.facets.push(path_facet(ToolNature::Read, &path,
                    "new child Git directory and fixed layout pointer discovery; no target contents")?);
            }
            self.checkout_child_layouts.push(layout);
        }
        self.identity = PreparedPlanIdentity::new();
        self.phase = Phase::CheckoutChildMetadata;
        Ok(self)
    }

    fn discover_checkout_child_metadata(
        mut self: Box<Self>,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<Box<Self>, GitToolError> {
        self.revalidate()?;
        for layout in &self.checkout_child_layouts {
            self.child_read(&layout.metadata_path, authorizations)?;
            for leaf in ["commondir", "gitdir", "HEAD"] {
                self.child_read(&layout.metadata_path.join(leaf), authorizations)?;
            }
        }
        for layout in &self.checkout_child_layouts {
            let metadata = super::receiver_state::Metadata::capture(
                &layout.metadata_path,
                self.child_read(&layout.metadata_path, authorizations)?,
            )?;
            // A linked child's backlink must name the already admitted marker.
            // Do not inspect a newly declared unrelated worktree marker.
            if !self.child_backlink_matches(
                layout,
                metadata.worktree_marker.as_deref(),
                authorizations,
            )? {
                return Err(GitToolError::InvalidInput(
                    "New child backlink requires independent worktree discovery".into(),
                ));
            }
            self.facets.push(path_facet(
                ToolNature::Read,
                &metadata.common_path,
                "new child shared Git metadata; no mutation authority",
            )?);
            for path in [
                metadata.common_path.join("config"),
                metadata.canonical.join("config.worktree"),
            ] {
                self.facets.push(path_facet(
                    ToolNature::Read,
                    &path,
                    "new child raw startup layout configuration; includes not expanded",
                )?);
            }
            self.checkout_child_metadata.push(metadata);
        }
        self.facets.push(
            ToolPermissionFacet::with_resource(
                ToolNature::Execute,
                self.program
                    .to_str()
                    .ok_or_else(|| GitToolError::InvalidInput("Git path is not UTF-8".into()))?,
                ToolResourceKind::Command,
            )
            .with_description("fixed new-child configuration discovery; Trace2 outputs disabled"),
        );
        self.facets.push(ToolPermissionFacet::with_resource(
            ToolNature::Read, "git:configuration-chain", ToolResourceKind::Remote,
        ).with_description("discover new-child system/global/include configuration internally; no general file access"));
        self.identity = PreparedPlanIdentity::new();
        self.phase = Phase::CheckoutChildConfiguration;
        Ok(self)
    }

    async fn discover_checkout_child_configuration(
        mut self: Box<Self>,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<Box<Self>, GitToolError> {
        self.revalidate()?;
        if self.checkout_child_layouts.len() != self.checkout_child_metadata.len() {
            return Err(GitToolError::Git(
                "Incomplete new child location evidence".into(),
            ));
        }
        // No command runs until every newly discovered shared root is admitted.
        for metadata in &self.checkout_child_metadata {
            self.child_read(&metadata.common_path, authorizations)?;
            self.child_read(&metadata.common_path.join("config"), authorizations)?;
            self.child_read(&metadata.canonical.join("config.worktree"), authorizations)?;
        }
        for (layout, metadata) in self
            .checkout_child_layouts
            .iter()
            .zip(&self.checkout_child_metadata)
        {
            let startup = super::config_state::ChildStartupLayout::capture(
                &metadata.canonical,
                &metadata.common_path,
                metadata.has_common(),
            )?;
            for target in &startup.targets {
                self.facets.push(path_facet(
                    ToolNature::Read,
                    target,
                    "new child raw startup core.worktree target; independent of include overrides",
                )?);
            }
            let worktree = startup
                .targets
                .iter()
                .next()
                .cloned()
                .unwrap_or_else(|| layout.canonical.clone());
            self.checkout_child_startup.push(startup);
            // Bind setup to the already admitted directory: --git-dir alone
            // still resolves core.worktree. This is a discovery-only override;
            // the configured value remains in the inventory for fresh admission.
            let output = self
                .inspect_child_bootstrap_config(&metadata.canonical, &layout.canonical)
                .await?;
            let config =
                super::config_state::ChildConfig::capture_receiver(&output, &metadata.canonical)?;
            config.verify_host(
                &self
                    .inspect_child_bootstrap_config(&metadata.canonical, &layout.canonical)
                    .await?,
            )?;
            self.facets.push(path_facet(
                ToolNature::Read,
                &worktree,
                "new child effective worktree layout and index-selected attribute path discovery; no attribute contents",
            )?);
            for path in [&metadata.canonical, &metadata.common_path] {
                self.facets.push(path_facet(
                    ToolNature::Read,
                    path,
                    "new child HEAD, index and object metadata for attribute path discovery; no mutation",
                )?);
            }
            for (path, _) in &config.files {
                self.facets.push(path_facet(
                    ToolNature::Read,
                    path,
                    "new child bootstrap configuration evidence retained for continuation",
                )?);
            }
            self.checkout_child_bootstrap.push((worktree, config));
        }
        self.identity = PreparedPlanIdentity::new();
        self.phase = Phase::CheckoutChildWorktree;
        Ok(self)
    }

    async fn discover_checkout_child_worktree(
        mut self: Box<Self>,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<Box<Self>, GitToolError> {
        self.revalidate()?;
        if self.checkout_child_bootstrap.len() != self.checkout_child_layouts.len() {
            return Err(GitToolError::Git(
                "Incomplete child bootstrap evidence".into(),
            ));
        }
        // Admit the entire worktree frontier before any ordinary Git setup.
        for startup in &self.checkout_child_startup {
            for target in &startup.targets {
                self.child_read(target, authorizations)?;
            }
        }
        for (worktree, config) in &self.checkout_child_bootstrap {
            let admitted = self.child_read(worktree, authorizations)?;
            let identity = snapshot::PathIdentity::capture(worktree).map_err(|_| {
                GitToolError::InvalidInput("Child worktree identity unavailable".into())
            })?;
            if worktree.canonicalize().ok().as_deref() != Some(admitted) {
                return Err(GitToolError::InvalidInput(
                    "Child worktree changed after admission".into(),
                ));
            }
            config.revalidate()?;
            self.checkout_child_worktrees
                .push((worktree.clone(), admitted.to_owned(), identity));
        }
        for startup in &self.checkout_child_startup {
            for target in &startup.targets {
                let admitted = self.child_read(target, authorizations)?;
                let identity = snapshot::PathIdentity::capture(target).map_err(|_| {
                    GitToolError::InvalidInput("Child startup target unavailable".into())
                })?;
                if target.canonicalize().ok().as_deref() != Some(admitted) {
                    return Err(GitToolError::InvalidInput(
                        "Child startup target changed after admission".into(),
                    ));
                }
                self.checkout_child_worktrees
                    .push((target.clone(), admitted.to_owned(), identity));
            }
        }
        for (layout, metadata) in self
            .checkout_child_layouts
            .iter()
            .zip(&self.checkout_child_metadata)
        {
            let location = super::repository_state::ChildLocation::capture(&layout.metadata_path)?;
            if location.git_dir != metadata.canonical
                || location.common_dir != self.child_read(&metadata.common_path, authorizations)?
            {
                return Err(GitToolError::InvalidInput(
                    "New child shared directory changed after admission".into(),
                ));
            }
            let workdir = &layout.canonical;
            let output = self.inspect_config_at(workdir).await?;
            let config = super::config_state::ChildConfig::capture(&output, workdir)?;
            config.verify_host(&self.inspect_config_at(workdir).await?)?;
            self.child_attributes
                .extend(super::config_state::declared_include_paths(
                    &output,
                    workdir,
                    &self.environment,
                )?);
            self.child_attributes
                .insert(location.common_dir.join("config"));
            self.child_attributes
                .insert(location.git_dir.join("config.worktree"));
            let effective_worktree = self.checkout_child_effective_worktree(workdir).to_owned();
            self.child_attributes.extend(config.attribute_paths(
                &effective_worktree,
                &location.common_dir,
                &self.environment,
            )?);
            for variable in ["GIT_ATTR_SYSTEM", "GIT_ATTR_GLOBAL"] {
                self.child_attributes
                    .insert(self.inspect_attribute_path_at(workdir, variable).await?);
            }
            layout.revalidate(&layout.canonical)?;
            metadata.revalidate(&metadata.canonical)?;
            if super::repository_state::ChildLocation::capture(&layout.metadata_path)? != location {
                return Err(GitToolError::InvalidInput(
                    "New child location changed during configuration discovery".into(),
                ));
            }
            // These are the existing child-status scopes, freshly exposed for
            // new children. They are not inferred from parent-directory Read.
            self.sender_mutation_facets.push(path_facet(
                ToolNature::Read,
                self.checkout_child_effective_worktree(workdir),
                "new child status directory enumeration, ignore rules and tracked sources",
            )?);
            self.sender_mutation_facets.push(path_facet(
                ToolNature::Read,
                &location.git_dir,
                "new child HEAD, index and object metadata",
            )?);
            self.sender_mutation_facets.push(path_facet(
                ToolNature::Read,
                &location.common_dir,
                "new child shared object and reference metadata",
            )?);
            self.sender_mutation_facets.push(path_facet(
                ToolNature::Write,
                &location.git_dir,
                "checkout child status may refresh its index",
            )?);
            self.child_configs.push((workdir.clone(), config));
            self.child_locations
                .push((layout.metadata_path.clone(), location));
        }
        let config = self
            .config
            .as_mut()
            .ok_or_else(|| GitToolError::Git("Missing source configuration".into()))?;
        config
            .stable_auxiliary
            .extend(self.child_attributes.iter().cloned());
        for path in &self.child_attributes {
            self.sender_mutation_facets.push(path_facet(
                ToolNature::Read,
                path,
                "new child effective attribute, exclude and include sources",
            )?);
        }
        for (_, child) in &self.child_configs {
            for (path, _) in &child.files {
                config.stable_auxiliary.insert(path.clone());
                let read = path_facet(ToolNature::Read, path, "new child configuration evidence")?;
                // Intermediate target-config/deletion stages also revalidate
                // ChildConfig after the discovery capability has expired.
                self.facets.push(read.clone());
                self.sender_mutation_facets.push(read);
            }
        }
        let sources = self.child_attributes.iter().cloned().chain(
            self.child_configs
                .iter()
                .flat_map(|(_, child)| child.files.iter().map(|(path, _)| path.clone())),
        );
        let auxiliary = self
            .auxiliary_paths
            .as_mut()
            .ok_or_else(|| GitToolError::Git("Missing auxiliary path discovery evidence".into()))?;
        auxiliary.extend(self.operation.name(), sources, authorizations)?;
        auxiliary.add_read_frontier(&mut self.facets)?;
        auxiliary.add_read_frontier(&mut self.sender_mutation_facets)?;
        // This stage explicitly admits child metadata/index and worktree reads.
        // Discover selected rule paths here, co-publishing their actual-target
        // frontier with the other auxiliary sources. Bodies still wait for the
        // next admission; mutation need not spend a stage discovering them.
        self = self.discover_child_attribute_targets(authorizations)?;
        self.facets.retain(|facet| {
            facet.nature == ToolNature::Read
                && facet.resource.as_deref() != Some("git:configuration-chain")
        });
        self.revalidate()?;
        self.finish_new_gitlink_discovery()
    }

    fn add_target_configuration_reads(&mut self) -> Result<(), GitToolError> {
        for path in self
            .target_config_sources
            .as_ref()
            .ok_or_else(|| GitToolError::Git("Missing target configuration sources".into()))?
            .paths()
        {
            if !self.facets.iter().any(|facet| {
                facet.nature == ToolNature::Read && facet.resource.as_deref() == path.to_str()
            }) {
                self.facets.push(path_facet(
                    ToolNature::Read,
                    &path,
                    "target configuration source or ancestor identity; no mutation authority",
                )?);
            }
        }
        Ok(())
    }

    fn discover_target_configuration(
        mut self: Box<Self>,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<Box<Self>, GitToolError> {
        self.revalidate()?;
        if let Some(paths) = &self.target_hook_paths {
            let sources = self
                .target_config_sources
                .as_mut()
                .ok_or_else(|| GitToolError::Git("Missing target hook ancestor evidence".into()))?;
            sources.advance(
                |path| {
                    authorizations.iter().any(|token| {
                        token.authorizes_path(
                            "git_checkout",
                            ToolNature::Read,
                            &self.workspace,
                            path.to_str().unwrap_or_default(),
                        )
                    })
                },
                &self.environment,
            )?;
            if !sources.complete() {
                self.add_target_configuration_reads()?;
                self.identity = PreparedPlanIdentity::new();
                return Ok(self);
            }
            if paths.iter().any(|path| {
                !authorizations.iter().any(|token| {
                    token.authorizes_path(
                        "git_checkout",
                        ToolNature::Read,
                        &self.workspace,
                        path.to_str().unwrap_or_default(),
                    )
                })
            }) {
                return Err(GitToolError::InvalidInput(
                    "Target checkout hooks require exact Read admission before capture".into(),
                ));
            }
            self.target_hook_evidence =
                super::config_state::ConfigState::capture_checkout_reload_hooks(paths)?;
            self.revalidate()?;
            // Preserve every discovered Read in the final profile. A deletion
            // inventory still needs its own read-only frontier before any Write
            // authority is restored. Neither continuation reuses this proof.
            if self.checkout_deletion.is_some() {
                self.add_checkout_deletion_reads()?;
                self.phase = Phase::CheckoutDeletion;
            } else {
                self.facets
                    .extend(self.sender_mutation_facets.iter().cloned());
                self.phase = Phase::Mutation;
            }
            self.identity = PreparedPlanIdentity::new();
            return Ok(self);
        }
        let sources = self
            .target_config_sources
            .as_mut()
            .ok_or_else(|| GitToolError::Git("Missing target configuration sources".into()))?;
        sources.advance(
            |path| {
                authorizations.iter().any(|token| {
                    token.authorizes_path(
                        "git_checkout",
                        ToolNature::Read,
                        &self.workspace,
                        path.to_str().unwrap_or_default(),
                    )
                })
            },
            &self.environment,
        )?;
        if sources.complete()
            && sources.worktree_config_enabled(&self.repository.common_dir.join("config"))?
        {
            let worktree = self.repository.git_dir.join("config.worktree");
            // The extension is now bound to admitted main-config bytes. A new
            // worktree root waits for its own exact Read frontier if necessary.
            if self.target_config_roots.last() != Some(&worktree) {
                self.target_config_roots.push(worktree.clone());
                sources.add([worktree])?;
            }
        }
        if sources.complete() {
            sources.predict_tracking(
                &self.repository.common_dir.join("config"),
                &self.checkout_tracking_writes,
            )?;
            // Potential source discovery is not ordered conditional evaluation.
            // Do not manufacture final effects from this intermediate evidence.
            let records = sources.ordered_records(
                &self.target_config_roots,
                &self.environment,
                Some(&self.repository.git_dir),
            )?;
            super::config_state::ConfigState::validate_checkout_reload(&records)?;
            let paths = self
                .config
                .as_ref()
                .ok_or_else(|| GitToolError::Git("Missing source configuration".into()))?
                .checkout_reload_hook_paths(&records, &self.repository, &self.environment)?;
            for path in &paths {
                sources.add_optional_directory_context(path.parent().ok_or_else(|| {
                    GitToolError::InvalidInput("Target hook parent is unavailable".into())
                })?)?;
            }
            for path in &paths {
                self.facets.push(path_facet(
                    ToolNature::Read,
                    path,
                    "late checkout traditional hook candidate; contents require fresh Read admission",
                )?);
            }
            self.target_hook_paths = Some(paths);
            // Target Sources keeps its own exact-frontier/link-refusal rules.
            // Retain these newly known file spellings in the invocation-wide
            // auxiliary chain as well, for final overlap and mutation checks.
            let paths = sources
                .captured_paths()
                .cloned()
                .chain(self.target_hook_paths.iter().flatten().cloned());
            let auxiliary = self.auxiliary_paths.as_mut().ok_or_else(|| {
                GitToolError::Git("Missing auxiliary path discovery evidence".into())
            })?;
            auxiliary.extend(self.operation.name(), paths, authorizations)?;
            auxiliary.add_read_frontier(&mut self.facets)?;
            self.add_target_configuration_reads()?;
            self.identity = PreparedPlanIdentity::new();
            return Ok(self);
        }
        self.add_target_configuration_reads()?;
        self.identity = PreparedPlanIdentity::new();
        Ok(self)
    }

    fn add_checkout_deletion_reads(&mut self) -> Result<(), GitToolError> {
        let inventory = self
            .checkout_deletion
            .as_ref()
            .ok_or_else(|| GitToolError::Git("Missing checkout deletion inventory".into()))?;
        for path in inventory.paths() {
            if !self.facets.iter().any(|facet| {
                facet.nature == ToolNature::Read && facet.resource.as_deref() == path.to_str()
            }) {
                self.facets.push(path_facet(ToolNature::Read, &path,
                    "checkout deletion inventory: directory names or exact file evidence; no deletion authority")?);
            }
        }
        Ok(())
    }

    fn discover_checkout_deletion(
        mut self: Box<Self>,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<Box<Self>, GitToolError> {
        self.revalidate()?;
        let mut inventory = self
            .checkout_deletion
            .take()
            .ok_or_else(|| GitToolError::Git("Missing checkout deletion inventory".into()))?;
        inventory.advance(|path| {
            authorizations.iter().any(|token| {
                token.normalized_path() == path
                    && token.authorizes_path(
                        "git_checkout",
                        ToolNature::Read,
                        &self.workspace,
                        path.to_str().unwrap_or_default(),
                    )
            })
        })?;
        let complete = inventory.complete();
        self.checkout_deletion = Some(inventory);
        self.add_checkout_deletion_reads()?;
        if complete {
            self.facets
                .extend(self.sender_mutation_facets.iter().cloned());
            for path in self
                .checkout_deletion
                .as_ref()
                .ok_or_else(|| GitToolError::Git("Missing checkout deletion inventory".into()))?
                .paths()
            {
                self.facets.push(path_facet(ToolNature::Write, &path,
                    "checkout may recursively delete this child entry, including untracked files and child Git metadata")?);
            }
            self.phase = Phase::Mutation;
        }
        self.identity = PreparedPlanIdentity::new();
        Ok(self)
    }

    async fn inspect_git_shell_path(&self) -> Result<PathBuf, GitToolError> {
        let mut command = tokio::process::Command::new(&self.program);
        command
            .current_dir(&self.repository.workdir)
            .args(["var", "GIT_SHELL_PATH"])
            .env_clear()
            .envs(&self.environment);
        let output = self
            .run_command(command, std::time::Duration::from_secs(10), 16 * 1024)
            .await?;
        let path = output
            .strip_suffix(b"\n")
            .and_then(|path| std::str::from_utf8(path).ok())
            .map(PathBuf::from)
            .filter(|path| {
                path.is_absolute()
                    && !path.as_os_str().as_encoded_bytes().contains(&0)
                    && !path
                        .components()
                        .any(|component| component == std::path::Component::ParentDir)
            })
            .ok_or_else(|| {
                GitToolError::InvalidInput("Git transport shell path is unavailable".into())
            })?;
        Ok(path)
    }

    async fn revalidate_receiver_shell_selection(&self) -> Result<(), GitToolError> {
        // Git for Windows resolves sh through PATH for each invocation.
        // Compare spelling before capturing or executing a newly selected shell.
        let shell = self.inspect_git_shell_path().await?;
        if self.receiver_child_paths.first() != Some(&shell) {
            return Err(GitToolError::InvalidInput(
                "Receiver shell selection changed after approval".into(),
            ));
        }
        Ok(())
    }

    fn discover_receiver_roots(
        mut self: Box<Self>,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<Box<Self>, GitToolError> {
        // No Git command or automatic repository discovery is allowed here:
        // a gitfile may name a directory outside the admitted receiver root.
        let mut metadata_paths = Vec::new();
        for path in &self.receiver_child_paths {
            let admitted = self.receiver_read_path(path, authorizations)?;
            self.receiver_child_executables
                .push(ReceiverChildExecutable::capture(path, admitted)?);
        }
        self.validate_receiver_git_programs()?;
        for root in &self.receiver_roots {
            let spelling = root
                .to_str()
                .ok_or_else(|| GitToolError::InvalidInput("Receiver path is not UTF-8".into()))?;
            let authorization = authorizations
                .iter()
                .find(|authorization| {
                    authorization.authorizes_path(
                        self.operation.name(),
                        ToolNature::Read,
                        &self.workspace,
                        spelling,
                    )
                })
                .ok_or_else(|| {
                    GitToolError::InvalidInput("Receiver Read authorization missing".into())
                })?;
            let layout =
                super::receiver_state::Root::capture(root, authorization.normalized_path())?;
            metadata_paths.push(layout.metadata_path.clone());
            self.receiver_layouts.push(layout);
        }
        for path in metadata_paths {
            self.facets.push(path_facet(
                ToolNature::Read,
                &path,
                "receiver Git metadata layout; configuration and writes remain unapproved",
            )?);
        }
        self.identity = PreparedPlanIdentity::new();
        self.phase = Phase::ReceiverMetadata;
        Ok(self)
    }

    fn receiver_read_path<'a>(
        &self,
        path: &Path,
        authorizations: &'a [talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<&'a Path, GitToolError> {
        let spelling = path
            .to_str()
            .ok_or_else(|| GitToolError::InvalidInput("Receiver path is not UTF-8".into()))?;
        authorizations
            .iter()
            .find(|authorization| {
                authorization.authorizes_path(
                    self.operation.name(),
                    ToolNature::Read,
                    &self.workspace,
                    spelling,
                )
            })
            .map(|authorization| authorization.normalized_path())
            .ok_or_else(|| GitToolError::InvalidInput("Receiver Read authorization missing".into()))
    }

    fn discover_receiver_metadata(
        mut self: Box<Self>,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<Box<Self>, GitToolError> {
        self.revalidate_receiver_roots(authorizations)?;
        for layout in &self.receiver_layouts {
            let admitted = self.receiver_read_path(&layout.metadata_path, authorizations)?;
            let metadata =
                super::receiver_state::Metadata::capture(&layout.metadata_path, admitted)?;
            self.facets.push(path_facet(
                ToolNature::Read,
                &metadata.common_path,
                "receiver shared metadata layout; configuration and writes remain unapproved",
            )?);
            for path in [
                metadata.common_path.join("refs"),
                metadata.canonical.join("refs"),
            ] {
                self.facets.push(path_facet(
                    ToolNature::Read,
                    &path,
                    "receiver reference directory and bounded descendant tree; no mutation",
                )?);
            }
            self.facets.push(path_facet(
                ToolNature::Read,
                &metadata.common_path.join("packed-refs"),
                "receiver packed reference file evidence; no mutation",
            )?);
            self.facets.push(path_facet(
                ToolNature::Read,
                &metadata.common_path.join("worktrees"),
                "receiver worktree registration names only; no child content reads",
            )?);
            // Path authorizations are exact. The separately reviewed closed
            // capability covers bounded descendant reads under the admitted
            // refs roots; the roots alone cannot authorize the tree walk.
            self.facets.push(
                ToolPermissionFacet::with_resource(
                    ToolNature::Read,
                    "git:receiver-reference-tree",
                    ToolResourceKind::Remote,
                )
                .with_description(
                    "bounded receiver refs subtree inspection at the separately admitted paths; push only, no general file access or mutation",
                ),
            );
            if let Some(marker) = &metadata.worktree_marker {
                self.facets.push(path_facet(
                    ToolNature::Read,
                    marker,
                    "receiver linked-worktree backpointer target; no worktree mutation",
                )?);
            }
            self.receiver_metadata.push(metadata);
        }
        self.revalidate_receiver_roots(authorizations)?;
        self.identity = PreparedPlanIdentity::new();
        self.phase = Phase::ReceiverCommon;
        Ok(self)
    }

    fn discover_receiver_common(
        mut self: Box<Self>,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<Box<Self>, GitToolError> {
        self.require_receiver_reference_capability()?;
        self.revalidate_receiver_metadata(authorizations)?;
        let mut reference_budget = snapshot::Budget {
            entries: 65_536,
            bytes: 16 * 1024 * 1024,
        };
        for metadata in &self.receiver_metadata {
            let common = self.receiver_read_path(&metadata.common_path, authorizations)?;
            let marker = metadata
                .worktree_marker
                .as_ref()
                .map(|path| self.receiver_read_path(path, authorizations))
                .transpose()?;
            for path in [
                metadata.common_path.join("refs"),
                metadata.canonical.join("refs"),
                metadata.common_path.join("packed-refs"),
            ] {
                self.receiver_read_path(&path, authorizations)?;
            }
            let evidence = super::receiver_state::Common::capture_with_budget(
                metadata,
                common,
                marker,
                &mut reference_budget,
            )?;
            let registrations = metadata.common_path.join("worktrees");
            let inventory = super::receiver_state::WorktreeInventory::capture(
                &registrations,
                self.receiver_read_path(&registrations, authorizations)?,
            )?;
            for name in &inventory.names {
                let sibling = registrations.join(name);
                self.facets.push(path_facet(
                    ToolNature::Read,
                    &sibling,
                    "receiver sibling worktree metadata candidate; contents deferred",
                )?);
                for leaf in ["HEAD", "gitdir", "commondir"] {
                    self.facets.push(path_facet(
                        ToolNature::Read,
                        &sibling.join(leaf),
                        "receiver sibling HEAD and layout pointer evidence; targets deferred",
                    )?);
                }
            }
            self.receiver_worktree_inventories.push(inventory);
            for (base, declared) in &evidence.includes {
                let text = declared.to_str().ok_or_else(|| {
                    GitToolError::InvalidInput("Receiver include path is not UTF-8".into())
                })?;
                let path = if let Some(relative) = text.strip_prefix("~/") {
                    let home = self
                        .environment
                        .get(std::ffi::OsStr::new("HOME"))
                        .ok_or_else(|| {
                            GitToolError::InvalidInput(
                                "Receiver include requires a captured home directory".into(),
                            )
                        })?;
                    let home = Path::new(home);
                    if !home.is_absolute() {
                        return Err(GitToolError::InvalidInput(
                            "Receiver include home must be absolute".into(),
                        ));
                    }
                    super::remote_state::validate_local_path_syntax(home)?;
                    if Path::new(relative).has_root() {
                        return Err(GitToolError::InvalidInput(
                            "Receiver home include suffix must be relative".into(),
                        ));
                    }
                    super::remote_state::validate_local_path_syntax(Path::new(relative))?;
                    if Path::new(relative)
                        .components()
                        .any(|part| part == std::path::Component::ParentDir)
                    {
                        return Err(GitToolError::InvalidInput(
                            "Receiver home include has ambiguous parent traversal".into(),
                        ));
                    }
                    home.join(relative)
                } else {
                    if text.starts_with('~') || text.starts_with("%(") {
                        return Err(GitToolError::InvalidInput(
                            "Receiver include interpolation requires additional discovery".into(),
                        ));
                    }
                    super::remote_state::validate_local_path_syntax(declared)?;
                    base.join(declared)
                };
                self.facets.push(path_facet(ToolNature::Read, &path,
                    "receiver declared include candidate, including inactive conditions; contents not yet inspected")?);
            }
            for path in &evidence.local_worktrees {
                self.facets.push(path_facet(ToolNature::Read, path,
                    "receiver local core.worktree layout hint; effective configuration remains unapproved")?);
            }
            if let Some(target) = &evidence.backlink_target {
                self.facets.push(path_facet(
                    ToolNature::Read,
                    target,
                    "receiver worktree backlink verification; no mutation",
                )?);
            }
            self.receiver_common.push(evidence);
        }
        self.revalidate_receiver_metadata(authorizations)?;
        self.facets.push(
            ToolPermissionFacet::with_resource(
                ToolNature::Execute,
                self.program.to_str().ok_or_else(|| {
                    GitToolError::InvalidInput("Git program path is not UTF-8".into())
                })?,
                ToolResourceKind::Command,
            )
            .with_description(
                "fixed receiver configuration inventory; Trace2 file/socket output disabled",
            ),
        );
        self.facets.push(ToolPermissionFacet::with_resource(
            ToolNature::Read, "git:configuration-chain", ToolResourceKind::Remote,
        ).with_description("discover receiver system/global/include configuration internally; no mutation or general file access"));
        self.identity = PreparedPlanIdentity::new();
        self.phase = Phase::ReceiverConfiguration;
        Ok(self)
    }

    async fn discover_receiver_configuration(
        mut self: Box<Self>,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<Box<Self>, GitToolError> {
        self.revalidate()?;
        self.revalidate_receiver_common(authorizations)?;
        // Ordinary `git config` setup can resolve these direct-file layout
        // hints even though receive-pack's enter_repo does not. Their access
        // must be admitted before the fixed configuration-chain command.
        for (metadata, inventory) in self
            .receiver_metadata
            .iter()
            .zip(&self.receiver_worktree_inventories)
        {
            for name in &inventory.names {
                let path = metadata.common_path.join("worktrees").join(name);
                let admitted = self.receiver_read_path(&path, authorizations)?;
                for leaf in ["HEAD", "gitdir", "commondir"] {
                    self.receiver_read_path(&path.join(leaf), authorizations)?;
                }
                let sibling = super::receiver_state::Metadata::capture(&path, admitted)?;
                self.facets.push(path_facet(
                    ToolNature::Read,
                    &sibling.canonical.join("refs"),
                    "sibling private references under dedicated receiver reference-tree admission",
                )?);
                self.facets.push(path_facet(
                    ToolNature::Read,
                    &sibling.common_path,
                    "sibling shared directory target; verification deferred",
                )?);
                if let Some(marker) = &sibling.worktree_marker {
                    self.facets.push(path_facet(
                        ToolNature::Read,
                        marker,
                        "sibling worktree marker target; verification deferred",
                    )?);
                }
                self.receiver_siblings.push(sibling);
            }
        }
        for common in &self.receiver_common {
            for path in &common.local_worktrees {
                let admitted = self.receiver_read_path(path, authorizations)?;
                if path.canonicalize().map_err(|_| {
                    GitToolError::InvalidInput("Receiver setup worktree is unavailable".into())
                })? != admitted
                {
                    return Err(GitToolError::InvalidInput(
                        "Receiver setup worktree changed after admission".into(),
                    ));
                }
            }
        }
        for (metadata, common) in self.receiver_metadata.iter().zip(&self.receiver_common) {
            let output = self.inspect_receiver_config_at(&metadata.canonical).await?;
            let config =
                super::config_state::ChildConfig::capture_receiver(&output, &metadata.canonical)?;
            if output != self.inspect_receiver_config_at(&metadata.canonical).await? {
                return Err(GitToolError::InvalidInput(
                    "Receiver configuration changed during discovery".into(),
                ));
            }
            let mut effects = config.receiver_effects(
                &metadata.canonical,
                &common.canonical,
                &self.environment,
            )?;
            // get_worktrees reads the main worktree HEAD and separately checks
            // common/config.worktree for core.bare when invoked from a linked
            // worktree. These are not necessarily in that receiver's config chain.
            effects.paths.insert(common.canonical.join("HEAD"));
            effects
                .paths
                .insert(common.canonical.join("config.worktree"));
            // receive-pack checks detached worktrees for an active bisect.
            // Inventory names only here; auxiliary admission precedes reads.
            for gitdir in std::iter::once(&metadata.canonical)
                .chain(std::iter::once(&common.canonical))
                .chain(
                    self.receiver_siblings
                        .iter()
                        .filter(|sibling| {
                            sibling.spelling.parent()
                                == Some(metadata.common_path.join("worktrees").as_path())
                        })
                        .map(|sibling| &sibling.canonical),
                )
            {
                for leaf in ["BISECT_LOG", "BISECT_START"] {
                    effects.paths.insert(gitdir.join(leaf));
                }
                for (directory, merge) in [("rebase-apply", false), ("rebase-merge", true)] {
                    let path = gitdir.join(directory);
                    self.facets.push(path_facet(
                        ToolNature::Read,
                        &path,
                        "receiver rebase directory existence",
                    )?);
                    for leaf in super::receiver_state::RebaseState::leaves(merge) {
                        self.facets.push(path_facet(
                            ToolNature::Read,
                            &path.join(leaf),
                            "receiver rebase occupancy evidence",
                        )?);
                    }
                }
            }
            effects
                .paths
                .extend(config.files.iter().map(|(path, _)| path.clone()));
            for (path, _) in &config.files {
                self.facets.push(path_facet(
                    ToolNature::Read,
                    path,
                    "receiver configuration evidence",
                )?);
            }
            for path in super::config_state::declared_include_paths(
                &output,
                &metadata.canonical,
                &self.environment,
            )? {
                effects.paths.insert(path.clone());
                self.facets.push(path_facet(
                    ToolNature::Read,
                    &path,
                    "receiver declared include dependency",
                )?);
            }
            self.receiver_effects.push(effects);
            self.receiver_configs
                .push((metadata.canonical.clone(), config));
        }
        self.revalidate_receiver_siblings(authorizations)?;
        self.revalidate_receiver_common(authorizations)?;
        self.revalidate()?;
        self.facets.retain(|facet| {
            facet.nature != ToolNature::Execute
                && facet.resource.as_deref() != Some("git:configuration-chain")
        });
        let receiver_sources = self
            .receiver_effects
            .iter()
            .flat_map(|effects| effects.paths.iter().cloned());
        let auxiliary = self.auxiliary_paths.as_mut().ok_or_else(|| {
            GitToolError::InvalidInput("Missing invocation auxiliary path evidence".into())
        })?;
        auxiliary.extend(self.operation.name(), receiver_sources, authorizations)?;
        auxiliary.add_read_frontier(&mut self.facets)?;
        self.identity = PreparedPlanIdentity::new();
        self.phase = Phase::ReceiverAuxiliary;
        Ok(self)
    }

    async fn inspect_receiver_config_at(&self, gitdir: &Path) -> Result<Vec<u8>, GitToolError> {
        let mut command = tokio::process::Command::new(&self.program);
        // An absolute gitdir keeps local --show-origin paths independent of
        // any cwd change performed by ordinary Git repository setup.
        command
            .current_dir(gitdir)
            .arg("--no-pager")
            .arg("--git-dir")
            .arg(host_path::directory_argument(gitdir)?.as_os_str())
            .args(["config", "--show-origin", "--null", "--list", "--includes"])
            .env_clear()
            .envs(&self.environment);
        self.run_command(command, std::time::Duration::from_secs(10), 256 * 1024)
            .await
            .map_err(|error| match error {
                GitToolError::InvalidInput(_) => error,
                _ => GitToolError::Git(
                    "Receiver configuration inspection failed; configuration values withheld"
                        .into(),
                ),
            })
    }

    async fn inspect_child_bootstrap_config(
        &self,
        gitdir: &Path,
        worktree: &Path,
    ) -> Result<Vec<u8>, GitToolError> {
        let mut command = tokio::process::Command::new(&self.program);
        command
            .current_dir(worktree)
            .arg("--no-pager")
            .arg("--git-dir")
            .arg(host_path::directory_argument(gitdir)?.as_os_str())
            .arg("--work-tree")
            .arg(host_path::directory_argument(worktree)?.as_os_str())
            .args(["config", "--show-origin", "--null", "--list", "--includes"])
            .env_clear()
            .envs(&self.environment);
        self.run_command(command, std::time::Duration::from_secs(10), 256 * 1024)
            .await
            .map_err(|_| {
                GitToolError::Git(
                    "Child bootstrap configuration inspection failed; values withheld".into(),
                )
            })
    }

    fn discover_receiver_auxiliary(
        mut self: Box<Self>,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<Box<Self>, GitToolError> {
        self.revalidate()?;
        self.revalidate_receiver_common(authorizations)?;
        self.revalidate_receiver_siblings(authorizations)?;
        let hooks: Vec<_> = self
            .config
            .iter()
            .flat_map(|config| config.hooks.iter().cloned())
            .collect();
        let auxiliary = self.auxiliary_paths.as_mut().ok_or_else(|| {
            GitToolError::InvalidInput("Missing invocation auxiliary path evidence".into())
        })?;
        auxiliary.capture_admitted(authorizations, &hooks)?;
        for (_, config) in &self.receiver_configs {
            config.validate_auxiliary_snapshot(auxiliary)?;
        }
        for (effects, common) in self.receiver_effects.iter().zip(&self.receiver_common) {
            effects.validate_admitted(auxiliary)?;
            for path in &effects.paths {
                let entry = auxiliary.leaf_evidence(path).ok_or_else(|| {
                    GitToolError::InvalidInput("Receiver auxiliary evidence is missing".into())
                })?;
                if path == &common.canonical.join("HEAD")
                    && let snapshot::Entry::File(_, bytes) = &entry.entry
                {
                    super::receiver_state::validate_symbolic_target(bytes)?;
                }
            }
            self.receiver_auxiliary
                .push(effects.paths.iter().cloned().collect());
        }
        self.require_receiver_reference_capability()?;
        let mut reference_budget = self.remaining_receiver_reference_budget()?;
        for sibling in &self.receiver_siblings {
            let targets = self.capture_receiver_sibling_targets(sibling, authorizations)?;
            if let Some(target) = &targets.backlink_target {
                self.facets.push(path_facet(
                    ToolNature::Read,
                    target,
                    "sibling marker backlink target; verification after fresh admission",
                )?);
            }
            self.receiver_sibling_targets.push(targets);
            let path = sibling.canonical.join("refs");
            let admitted = self.receiver_read_path(&path, authorizations)?;
            let evidence = super::receiver_state::ReferenceTree::capture_admitted(
                &path,
                admitted,
                &mut reference_budget,
            )?;
            self.receiver_sibling_refs.push((path, evidence));
        }
        for root in self
            .receiver_metadata
            .iter()
            .map(|metadata| &metadata.canonical)
            .chain(self.receiver_common.iter().map(|common| &common.canonical))
            .chain(
                self.receiver_siblings
                    .iter()
                    .map(|sibling| &sibling.canonical),
            )
        {
            for (directory, merge) in [("rebase-apply", false), ("rebase-merge", true)] {
                let path = root.join(directory);
                if self
                    .receiver_rebase
                    .iter()
                    .any(|(existing, _, _)| existing == &path)
                {
                    continue;
                }
                let admitted = self.receiver_read_path(&path, authorizations)?;
                for leaf in super::receiver_state::RebaseState::leaves(merge) {
                    self.receiver_read_path(&path.join(leaf), authorizations)?;
                }
                let state = super::receiver_state::RebaseState::capture(&path, admitted, merge)?;
                self.receiver_rebase.push((path, merge, state));
            }
        }
        self.revalidate_receiver_siblings(authorizations)?;
        self.revalidate_receiver_common(authorizations)?;
        self.identity = PreparedPlanIdentity::new();
        self.phase = Phase::ReceiverMutation;
        self.prepare_receiver_storage()?;
        Ok(self)
    }

    fn revalidate_receiver_auxiliary(
        &self,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<(), GitToolError> {
        self.revalidate()?;
        self.revalidate_receiver_common(authorizations)?;
        self.revalidate_receiver_siblings(authorizations)?;
        for (path, merge, expected) in &self.receiver_rebase {
            let admitted = self.receiver_read_path(path, authorizations)?;
            for leaf in super::receiver_state::RebaseState::leaves(*merge) {
                self.receiver_read_path(&path.join(leaf), authorizations)?;
            }
            if &super::receiver_state::RebaseState::capture(path, admitted, *merge)? != expected {
                return Err(GitToolError::InvalidInput(
                    "Receiver rebase state changed after admission".into(),
                ));
            }
        }
        self.require_receiver_reference_capability()?;
        if self.receiver_sibling_refs.len() != self.receiver_siblings.len() {
            return Err(GitToolError::InvalidInput(
                "Receiver sibling reference evidence missing".into(),
            ));
        }
        if self.receiver_sibling_targets.len() != self.receiver_siblings.len() {
            return Err(GitToolError::InvalidInput(
                "Receiver sibling target evidence missing".into(),
            ));
        }
        for (sibling, expected) in self
            .receiver_siblings
            .iter()
            .zip(&self.receiver_sibling_targets)
        {
            if &self.capture_receiver_sibling_targets(sibling, authorizations)? != expected {
                return Err(GitToolError::InvalidInput(
                    "Receiver sibling targets changed after admission".into(),
                ));
            }
            let admitted = expected
                .backlink_target
                .as_ref()
                .map(|path| self.receiver_read_path(path, authorizations))
                .transpose()?;
            expected.verify_backlink(sibling, admitted)?;
        }
        let mut reference_budget = self.remaining_receiver_reference_budget()?;
        for (path, expected) in &self.receiver_sibling_refs {
            let admitted = self.receiver_read_path(path, authorizations)?;
            if &super::receiver_state::ReferenceTree::capture_admitted(
                path,
                admitted,
                &mut reference_budget,
            )? != expected
            {
                return Err(GitToolError::InvalidInput(
                    "Receiver sibling references changed after admission".into(),
                ));
            }
        }
        let auxiliary = self.auxiliary_paths.as_ref().ok_or_else(|| {
            GitToolError::InvalidInput("Missing invocation auxiliary path evidence".into())
        })?;
        auxiliary.revalidate(authorizations)?;
        for (_, config) in &self.receiver_configs {
            config.validate_auxiliary_snapshot(auxiliary)?;
        }
        if self.receiver_effects.len() != self.receiver_auxiliary.len() {
            return Err(GitToolError::InvalidInput(
                "Receiver auxiliary source inventory is incomplete".into(),
            ));
        }
        for (effects, expected) in self.receiver_effects.iter().zip(&self.receiver_auxiliary) {
            if &effects.paths.iter().cloned().collect::<Vec<_>>() != expected {
                return Err(GitToolError::InvalidInput(
                    "Receiver auxiliary source set changed after admission".into(),
                ));
            }
            effects.validate_admitted(auxiliary)?;
        }
        Ok(())
    }

    fn remaining_receiver_reference_budget(&self) -> Result<snapshot::Budget, GitToolError> {
        let mut budget = snapshot::Budget {
            entries: 65_536,
            bytes: 16 * 1024 * 1024,
        };
        for common in &self.receiver_common {
            budget.entries = budget
                .entries
                .checked_sub(common.reference_entries)
                .ok_or_else(|| {
                    GitToolError::InvalidInput("Receiver reference entry limit exceeded".into())
                })?;
            budget.bytes = budget
                .bytes
                .checked_sub(common.reference_bytes)
                .ok_or_else(|| {
                    GitToolError::InvalidInput("Receiver reference byte limit exceeded".into())
                })?;
        }
        Ok(budget)
    }

    fn prepare_receiver_storage(&mut self) -> Result<(), GitToolError> {
        for root in self
            .receiver_metadata
            .iter()
            .map(|metadata| &metadata.canonical)
            .chain(self.receiver_common.iter().map(|common| &common.canonical))
        {
            self.facets.push(path_facet(
                ToolNature::Read,
                root,
                "receiver metadata structure and object-store inspection",
            )?);
        }
        self.facets.push(ToolPermissionFacet::with_resource(ToolNature::Read, "git:receiver-metadata", ToolResourceKind::Remote)
            .with_description("receiver metadata descendant inspection at separately admitted roots; no mutation in this stage"));
        self.facets.push(
            ToolPermissionFacet::with_resource(
                ToolNature::Execute,
                self.program
                    .to_str()
                    .ok_or_else(|| GitToolError::InvalidInput("Git path is not UTF-8".into()))?,
                ToolResourceKind::Command,
            )
            .with_description(
                "bound Git receiver executable; metadata inspection stage does not start it",
            ),
        );
        Ok(())
    }

    fn inspect_receiver_storage(
        &self,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<Vec<super::receiver_storage::Storage>, GitToolError> {
        self.revalidate_receiver_auxiliary(authorizations)?;
        if !self.facets.iter().any(|facet| {
            facet.nature == ToolNature::Read
                && facet.resource_kind == Some(ToolResourceKind::Remote)
                && facet.resource.as_deref() == Some("git:receiver-metadata")
        }) {
            return Err(GitToolError::InvalidInput(
                "Receiver metadata capability missing".into(),
            ));
        }
        let mut remaining = 65_536;
        let mut storage = Vec::new();
        for root in self
            .receiver_metadata
            .iter()
            .map(|metadata| &metadata.canonical)
            .chain(self.receiver_common.iter().map(|common| &common.canonical))
        {
            storage.push(super::receiver_storage::Storage::capture(
                root,
                self.receiver_read_path(root, authorizations)?,
                &mut remaining,
            )?);
        }
        Ok(storage)
    }

    fn prepare_receiver_mutation(
        mut self: Box<Self>,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<Box<Self>, GitToolError> {
        self.validate_receiver_shell_baseline()?;
        self.receiver_storage = self.inspect_receiver_storage(authorizations)?;
        self.facets
            .extend(self.sender_mutation_facets.iter().cloned());
        for root in self
            .receiver_metadata
            .iter()
            .map(|entry| &entry.canonical)
            .chain(self.receiver_common.iter().map(|entry| &entry.canonical))
        {
            self.facets.push(path_facet(ToolNature::Write, root,
                "receiver Git metadata for push and foreground maintenance; excludes worktrees and external stores")?);
        }
        self.facets.push(ToolPermissionFacet::with_resource(
            ToolNature::Write, "git:receiver-metadata", ToolResourceKind::Remote,
        ).with_description("push and foreground maintenance metadata within separately admitted receiver roots"));
        for child in &self.receiver_child_executables {
            self.facets.push(
                ToolPermissionFacet::with_resource(
                    ToolNature::Execute,
                    child.spelling.to_str().ok_or_else(|| {
                        GitToolError::InvalidInput("Receiver executable path is not UTF-8".into())
                    })?,
                    ToolResourceKind::Command,
                )
                .with_description("bound receiver transport or Git maintenance executable"),
            );
        }
        self.identity = PreparedPlanIdentity::new();
        self.phase = Phase::Mutation;
        Ok(self)
    }

    fn capture_receiver_sibling_targets(
        &self,
        sibling: &super::receiver_state::Metadata,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<super::receiver_state::SiblingTargets, GitToolError> {
        let owner = self
            .receiver_metadata
            .iter()
            .zip(&self.receiver_common)
            .find(|(metadata, _)| {
                sibling.spelling.parent() == Some(metadata.common_path.join("worktrees").as_path())
            })
            .ok_or_else(|| GitToolError::InvalidInput("Receiver sibling owner missing".into()))?;
        let common = self.receiver_read_path(&sibling.common_path, authorizations)?;
        let marker = sibling
            .worktree_marker
            .as_ref()
            .map(|path| self.receiver_read_path(path, authorizations))
            .transpose()?;
        super::receiver_state::SiblingTargets::capture(sibling, common, &owner.1.canonical, marker)
    }

    fn revalidate_receiver_siblings(
        &self,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<(), GitToolError> {
        let expected: usize = self
            .receiver_worktree_inventories
            .iter()
            .map(|inventory| inventory.names.len())
            .sum();
        if self.receiver_siblings.len() != expected {
            return Err(GitToolError::InvalidInput(
                "Receiver sibling metadata evidence missing".into(),
            ));
        }
        for sibling in &self.receiver_siblings {
            for leaf in ["HEAD", "gitdir", "commondir"] {
                self.receiver_read_path(&sibling.spelling.join(leaf), authorizations)?;
            }
            sibling.revalidate(self.receiver_read_path(&sibling.spelling, authorizations)?)?;
        }
        Ok(())
    }

    fn revalidate_receiver_common(
        &self,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<(), GitToolError> {
        self.require_receiver_reference_capability()?;
        self.revalidate_receiver_metadata(authorizations)?;
        if self.receiver_worktree_inventories.len() != self.receiver_common.len() {
            return Err(GitToolError::InvalidInput(
                "Receiver worktree inventory missing".into(),
            ));
        }
        for (metadata, inventory) in self
            .receiver_metadata
            .iter()
            .zip(&self.receiver_worktree_inventories)
        {
            inventory.revalidate(
                self.receiver_read_path(&metadata.common_path.join("worktrees"), authorizations)?,
            )?;
        }
        let mut reference_budget = snapshot::Budget {
            entries: 65_536,
            bytes: 16 * 1024 * 1024,
        };
        for (metadata, evidence) in self.receiver_metadata.iter().zip(&self.receiver_common) {
            let common = self.receiver_read_path(&metadata.common_path, authorizations)?;
            let marker = metadata
                .worktree_marker
                .as_ref()
                .map(|path| self.receiver_read_path(path, authorizations))
                .transpose()?;
            for path in [
                metadata.common_path.join("refs"),
                metadata.canonical.join("refs"),
                metadata.common_path.join("packed-refs"),
            ] {
                self.receiver_read_path(&path, authorizations)?;
            }
            evidence.revalidate_with_budget(metadata, common, marker, &mut reference_budget)?;
            let target = evidence
                .backlink_target
                .as_ref()
                .map(|path| self.receiver_read_path(path, authorizations))
                .transpose()?;
            evidence.verify_backlink(metadata, target)?;
        }
        Ok(())
    }

    // The dispatcher has consumed the full prepared proof before entering
    // these methods. Exact Path tokens alone do not cover descendant reads.
    fn require_receiver_reference_capability(&self) -> Result<(), GitToolError> {
        if !self.facets.iter().any(|facet| {
            facet.nature == ToolNature::Read
                && facet.resource_kind == Some(ToolResourceKind::Remote)
                && facet.resource.as_deref() == Some("git:receiver-reference-tree")
        }) {
            return Err(GitToolError::InvalidInput(
                "Receiver reference tree discovery capability missing".into(),
            ));
        }
        Ok(())
    }

    fn revalidate_receiver_metadata(
        &self,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<(), GitToolError> {
        self.revalidate_receiver_roots(authorizations)?;
        for metadata in &self.receiver_metadata {
            metadata.revalidate(self.receiver_read_path(&metadata.spelling, authorizations)?)?;
        }
        Ok(())
    }

    fn revalidate_receiver_roots(
        &self,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<(), GitToolError> {
        if self.receiver_child_paths.len() != self.receiver_child_executables.len() {
            return Err(GitToolError::InvalidInput(
                "Receiver child executable evidence is incomplete".into(),
            ));
        }
        for (path, evidence) in self
            .receiver_child_paths
            .iter()
            .zip(&self.receiver_child_executables)
        {
            evidence.revalidate(self.receiver_read_path(path, authorizations)?)?;
        }
        self.validate_receiver_git_programs()?;
        for layout in &self.receiver_layouts {
            let spelling = layout
                .spelling
                .to_str()
                .ok_or_else(|| GitToolError::InvalidInput("Receiver path is not UTF-8".into()))?;
            let authorization = authorizations
                .iter()
                .find(|authorization| {
                    authorization.authorizes_path(
                        self.operation.name(),
                        ToolNature::Read,
                        &self.workspace,
                        spelling,
                    )
                })
                .ok_or_else(|| {
                    GitToolError::InvalidInput("Receiver Read authorization missing".into())
                })?;
            layout.revalidate(authorization.normalized_path())?;
        }
        Ok(())
    }

    fn validate_receiver_shell_baseline(&self) -> Result<(), GitToolError> {
        let path = self
            .receiver_child_paths
            .first()
            .ok_or_else(|| GitToolError::InvalidInput("Receiver shell missing".into()))?;
        let evidence = self
            .receiver_child_executables
            .iter()
            .find(|child| &child.spelling == path)
            .ok_or_else(|| GitToolError::InvalidInput("Receiver shell identity missing".into()))?;
        #[cfg(windows)]
        {
            host_path::validate_receiver_shell(self.sender_program_target(), &evidence.canonical)
        }
        #[cfg(not(windows))]
        validate_receiver_shell_path(path, &evidence.canonical)
    }

    fn validate_receiver_git_programs(&self) -> Result<(), GitToolError> {
        // Validate representability now; dispatch will use this exact fixed
        // command once the remaining mutation effects are bound.
        receiver_command(&self.program)?;
        validate_receiver_shell_environment(&self.environment)?;
        let maintenance = self.maintenance_program.as_ref().ok_or_else(|| {
            GitToolError::InvalidInput("Receiver Git child executable missing".into())
        })?;
        // The fixed receiver command invokes this Git's builtin receive-pack;
        // no dashed git-receive-pack executable participates in dispatch.
        let evidence = self
            .receiver_child_executables
            .iter()
            .find(|evidence| &evidence.spelling == maintenance)
            .ok_or_else(|| {
                GitToolError::InvalidInput("Receiver Git executable evidence missing".into())
            })?;
        if evidence.program != self.program_entry {
            return Err(GitToolError::InvalidInput("Receiver Git children differ from the configuration-discovery executable; use a separately authorized invocation".into()));
        }
        Ok(())
    }

    async fn inspect_config(&self) -> Result<Vec<u8>, GitToolError> {
        self.inspect_config_at(&self.repository.workdir).await
    }

    async fn resolve_checkout_target(
        &self,
        checkout: &GitCheckoutInput,
        config: &super::config_state::ConfigState,
    ) -> Result<(String, Option<(String, String)>), GitToolError> {
        let target = if checkout.create {
            "HEAD"
        } else if checkout.branch == "-" {
            "@{-1}"
        } else {
            &checkout.branch
        };
        let mut tracking_source = None;
        let output = match self.inspect_commit_target(target).await? {
            Some(output) => output,
            None if !checkout.create && checkout.branch != "-" => {
                let (candidates, default) = config.checkout_candidates(&checkout.branch)?;
                let mut resolved = Vec::new();
                for (remote, reference) in candidates {
                    if let Some(oid) = self.inspect_commit_target(&reference).await? {
                        resolved.push((remote, reference, oid));
                    }
                }
                if resolved.len() == 1 {
                    let (remote, reference, oid) = resolved.remove(0);
                    tracking_source = Some((remote, reference));
                    oid
                } else if let Some(index) = default.as_ref().and_then(|default| {
                    resolved.iter().position(|(remote, _, _)| remote == default)
                }) {
                    let (remote, reference, oid) = resolved.remove(index);
                    tracking_source = Some((remote, reference));
                    oid
                } else {
                    return Err(GitToolError::InvalidInput(
                        "Checkout target is missing or ambiguous".into(),
                    ));
                }
            }
            None => {
                return Err(GitToolError::InvalidInput(
                    "Checkout target is missing".into(),
                ));
            }
        };
        let oid = std::str::from_utf8(&output)
            .map_err(|_| GitToolError::InvalidInput("Invalid checkout target".into()))?
            .trim()
            .to_owned();
        Ok((oid, tracking_source))
    }

    /// Branch creation without a start argument uses HEAD's resolved reference.
    /// This fixed read-only query remains in admitted configuration discovery.
    async fn inspect_checkout_start(&self, create: bool) -> Result<Option<String>, GitToolError> {
        if !create {
            return Ok(self
                .checkout_tracking_source
                .as_ref()
                .map(|(_, reference)| reference.clone()));
        }
        let mut command = tokio::process::Command::new(&self.program);
        command
            .current_dir(&self.repository.workdir)
            .args([
                "rev-parse",
                "--symbolic-full-name",
                "--verify",
                "--end-of-options",
                "HEAD",
            ])
            .env_clear()
            .envs(&self.environment);
        let output = self
            .run_command(command, std::time::Duration::from_secs(10), 4096)
            .await?;
        let reference = output
            .strip_suffix(b"\n")
            .and_then(|value| std::str::from_utf8(value).ok())
            .filter(|value| !value.is_empty() && !value.contains(['\0', '\n', '\r']))
            .ok_or_else(|| GitToolError::InvalidInput("Invalid checkout start reference".into()))?;
        if reference == "HEAD" {
            return Ok(None);
        }
        if !reference.starts_with("refs/heads/") {
            return Err(GitToolError::InvalidInput(
                "Unsupported checkout HEAD reference".into(),
            ));
        }
        Ok(Some(reference.to_owned()))
    }

    /// Native branch interpretation can expand previous-checkout expressions.
    /// This read-only call runs inside the admitted config-discovery boundary.
    async fn inspect_checkout_branch(&self, branch: &str) -> Result<String, GitToolError> {
        let mut command = tokio::process::Command::new(&self.program);
        command
            .current_dir(&self.repository.workdir)
            .args(["check-ref-format", "--branch", branch])
            .env_clear()
            .envs(&self.environment);
        let output = self
            .run_command(command, std::time::Duration::from_secs(10), 4096)
            .await?;
        let name = output
            .strip_suffix(b"\n")
            .and_then(|value| std::str::from_utf8(value).ok())
            .filter(|value| !value.is_empty() && !value.contains(['\0', '\n', '\r']))
            .ok_or_else(|| {
                GitToolError::InvalidInput("Invalid normalized checkout branch".into())
            })?;
        Ok(name.to_owned())
    }

    // Only Git's quiet missing-reference result permits candidate fallback.
    // Infrastructure errors and diagnostic failures must stop discovery.
    async fn inspect_commit_target(&self, target: &str) -> Result<Option<Vec<u8>>, GitToolError> {
        let mut command = tokio::process::Command::new(&self.program);
        command
            .current_dir(&self.repository.workdir)
            .args([
                "rev-parse",
                "--verify",
                "--quiet",
                "--end-of-options",
                &format!("{target}^{{commit}}"),
            ])
            .env_clear()
            .envs(&self.environment);
        let output = self
            .run_command_output(command, std::time::Duration::from_secs(10), 1024)
            .await?;
        if output.status.success() {
            return Ok(Some(output.stdout));
        }
        if output.status.code() == Some(1) && output.stdout.is_empty() && output.stderr.is_empty() {
            return Ok(None);
        }
        Err(GitToolError::Git(
            "Git commit target inspection failed".into(),
        ))
    }

    async fn inspect_config_at(&self, workdir: &Path) -> Result<Vec<u8>, GitToolError> {
        let mut command = tokio::process::Command::new(&self.program);
        command
            .current_dir(workdir)
            .args([
                "--no-pager",
                "config",
                "--show-origin",
                "--null",
                "--list",
                "--includes",
            ])
            .env_clear()
            .envs(&self.environment);
        let output = self
            .run_command_output(command, std::time::Duration::from_secs(10), 256 * 1024)
            .await
            .map_err(|error| match error {
                GitToolError::InvalidInput(_) => error,
                _ => GitToolError::Git(
                    "Git configuration inspection runner failed; configuration values withheld"
                        .into(),
                ),
            })?;
        if !output.status.success() {
            // Report only process metadata: stderr can contain configuration
            // values, credentials and external paths, even on parse failure.
            return Err(GitToolError::Git(format!(
                "Git configuration inspection exited unsuccessfully (code={:?}, stdout_bytes={}, stderr_bytes={}); configuration values withheld",
                output.status.code(),
                output.stdout.len(),
                output.stderr.len(),
            )));
        }
        Ok(output.stdout)
    }

    async fn inspect_config_paths(&self, variable: &str) -> Result<Vec<PathBuf>, GitToolError> {
        let mut command = tokio::process::Command::new(&self.program);
        command
            .current_dir(&self.repository.workdir)
            .args(["--no-pager", "var", variable])
            .env_clear()
            .envs(&self.environment);
        let output = self.run_command(command, std::time::Duration::from_secs(10), 16 * 1024).await
            .map_err(|error| match error {
                GitToolError::InvalidInput(_) => error,
                _ => GitToolError::InvalidInput("Git configuration path discovery unavailable; use a separately authorized invocation".into()),
            })?;
        if output.is_empty() {
            return Ok(Vec::new());
        }
        let output = output.strip_suffix(b"\n").ok_or_else(|| {
            GitToolError::InvalidInput("Invalid configuration path response".into())
        })?;
        let paths = output
            .split(|byte| *byte == b'\n')
            .map(|path| {
                let path = std::str::from_utf8(path).map(PathBuf::from).map_err(|_| {
                    GitToolError::InvalidInput("Configuration path is not UTF-8".into())
                })?;
                if !path.is_absolute() || path.as_os_str().as_encoded_bytes().contains(&0) {
                    return Err(GitToolError::InvalidInput(
                        "Invalid configuration path".into(),
                    ));
                }
                Ok(path)
            })
            .collect::<Result<Vec<_>, GitToolError>>()?;
        if paths.len()
            > if variable == "GIT_CONFIG_SYSTEM" {
                1
            } else {
                2
            }
        {
            return Err(GitToolError::InvalidInput(
                "Ambiguous configuration path response".into(),
            ));
        }
        Ok(paths)
    }

    async fn inspect_attribute_path_at(
        &self,
        workdir: &Path,
        variable: &str,
    ) -> Result<PathBuf, GitToolError> {
        let mut command = tokio::process::Command::new(&self.program);
        command
            .current_dir(workdir)
            .args(["--no-pager", "var", variable])
            .env_clear()
            .envs(&self.environment);
        let output = self.run_command(command, std::time::Duration::from_secs(10), 16 * 1024)
            .await
            .map_err(|error| match error {
                GitToolError::InvalidInput(_) => error,
                _ => GitToolError::InvalidInput(
                    "Git attribute path discovery unavailable; use a Git installation supporting GIT_ATTR_SYSTEM/GLOBAL or a separately authorized invocation".into(),
                ),
            })?;
        let path = output.strip_suffix(b"\n").ok_or_else(|| {
            GitToolError::InvalidInput(
                "Git attribute path discovery returned an invalid path".into(),
            )
        })?;
        let path = std::str::from_utf8(path)
            .map(PathBuf::from)
            .map_err(|_| GitToolError::InvalidInput("Git attribute path is not UTF-8".into()))?;
        if !path.is_absolute() || path.as_os_str().as_encoded_bytes().contains(&0) {
            return Err(GitToolError::InvalidInput(
                "Git attribute path is not absolute".into(),
            ));
        }
        Ok(path)
    }

    async fn execute_admitted(
        &mut self,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<String, GitToolError> {
        if self.phase != Phase::Mutation {
            return Err(GitToolError::InvalidInput(
                "Git mutation requires a completed discovery phase".into(),
            ));
        }
        if !self.receiver_roots.is_empty() {
            for root in self
                .receiver_metadata
                .iter()
                .map(|entry| &entry.canonical)
                .chain(self.receiver_common.iter().map(|entry| &entry.canonical))
            {
                if !authorizations.iter().any(|token| {
                    token.authorizes_path(
                        "git_push",
                        ToolNature::Write,
                        &self.workspace,
                        root.to_str().unwrap_or_default(),
                    )
                }) {
                    return Err(GitToolError::InvalidInput(
                        "Receiver metadata root lacks Write admission".into(),
                    ));
                }
            }
            if !self.facets.iter().any(|facet| {
                facet.nature == ToolNature::Write
                    && facet.resource_kind == Some(ToolResourceKind::Remote)
                    && facet.resource.as_deref() == Some("git:receiver-metadata")
            }) {
                return Err(GitToolError::InvalidInput(
                    "Receiver metadata Write capability missing".into(),
                ));
            }
            if self.inspect_receiver_storage(authorizations)? != self.receiver_storage {
                return Err(GitToolError::InvalidInput(
                    "Receiver metadata changed after approval".into(),
                ));
            }
        }
        self.revalidate()?;
        // Keep the shared evidence in the plan: receiver storage checks also
        // revalidate it after the final sender probes.
        let auxiliary = self.auxiliary_paths.as_mut().ok_or_else(|| {
            GitToolError::InvalidInput("Missing admitted auxiliary path evidence".into())
        })?;
        let hooks = &self
            .config
            .as_ref()
            .ok_or_else(|| GitToolError::InvalidInput("Missing Git configuration evidence".into()))?
            .hooks;
        auxiliary.capture_admitted(authorizations, hooks)?;
        let auxiliary = self.auxiliary_paths.as_ref().ok_or_else(|| {
            GitToolError::InvalidInput("Missing admitted auxiliary path evidence".into())
        })?;
        let push_attributes = if matches!(self.operation, Operation::Push(_)) {
            Some(super::status_state::StatusState::capture_pack_attributes(
                &self.repository.workdir,
            )?)
        } else {
            None
        };
        if matches!(self.operation, Operation::Push(_)) {
            if self.receiver_roots.is_empty() {
                super::remote_state::validate_direct_transport(&self.push_destinations)?;
            } else {
                let local = self.push_destinations.iter().map(|destination| {
                    super::remote_state::local_destination(destination, &self.repository.workdir)?
                        .ok_or_else(|| GitToolError::InvalidInput(
                            "Mixed local and network push destinations require separate transport command planning".into(),
                        ))
                }).collect::<Result<Vec<_>, _>>()?;
                if local != self.receiver_roots
                    || self.receiver_metadata.len() != local.len()
                    || self.receiver_common.len() != local.len()
                {
                    return Err(GitToolError::InvalidInput(
                        "Push receiver destination evidence is incomplete".into(),
                    ));
                }
                self.validate_receiver_shell_baseline()?;
            }
        }
        self.revalidate_maintenance_program()?;
        let status = if matches!(self.operation, Operation::Commit(_)) {
            Some(super::status_state::StatusState::capture(
                &self.repository.workdir,
                &self
                    .gitlinks
                    .iter()
                    .map(|link| self.repository.workdir.join(&link.path))
                    .collect::<Vec<_>>(),
            )?)
        } else {
            None
        };
        let child_evidence = if matches!(
            self.operation,
            Operation::Commit(_) | Operation::Checkout(_)
        ) {
            let children = self.capture_admitted_children()?;
            let child_inspection = self
                .inspect_child_filters(auxiliary, authorizations)
                .await?;
            Some((children, child_inspection))
        } else {
            None
        };
        self.config
            .as_ref()
            .ok_or_else(|| {
                GitToolError::InvalidInput(
                    "Git configuration discovery requires separate admission".into(),
                )
            })?
            .verify_host(&self.inspect_config().await?)?;
        let config = self
            .config
            .as_ref()
            .ok_or_else(|| GitToolError::Git("missing configuration evidence".into()))?;
        if let Some(target) = &self.checkout_target {
            let mut target_sources = self
                .target_config_sources
                .as_ref()
                .map_or_else(Vec::new, |sources| {
                    sources.captured_paths().cloned().collect()
                });
            target_sources.extend(
                self.target_hook_evidence
                    .iter()
                    .map(|(path, _)| path.clone()),
            );
            config.validate_checkout_target(
                &self.repository,
                target,
                &target_sources,
                auxiliary,
            )?;
        }
        if let Some(inventory) = &self.checkout_deletion {
            if !inventory.complete()
                || inventory.paths().iter().any(|path| {
                    !authorizations.iter().any(|token| {
                        token.normalized_path() == path
                            && token.authorizes_path(
                                "git_checkout",
                                ToolNature::Write,
                                &self.workspace,
                                path.to_str().unwrap_or_default(),
                            )
                    })
                })
            {
                return Err(GitToolError::InvalidInput(
                    "Checkout deletion requires exact Write admission for every child entry".into(),
                ));
            }
            let mut protected: Vec<_> = self
                .child_configs
                .iter()
                .flat_map(|(_, config)| config.files.iter().map(|(path, _)| path.clone()))
                .collect();
            if let Some(sources) = &self.target_config_sources {
                protected.extend(sources.captured_paths().cloned());
            }
            protected.extend(
                self.target_hook_evidence
                    .iter()
                    .map(|(path, _)| path.clone()),
            );
            if let Some((_, inspection)) = &child_evidence {
                protected.extend(inspection.attributes.iter().cloned());
            }
            config.validate_checkout_deletions(&inventory.roots(), &protected, auxiliary)?;
        }
        let mut entries = Vec::new();
        for (path, source) in self.paths.iter().zip(&self.sources) {
            selected_leaves(Path::new(path), source, &mut entries)?;
        }
        if !entries.is_empty() {
            let mut command = tokio::process::Command::new(&self.program);
            command
                .current_dir(&self.repository.workdir)
                .args(["--no-pager", "check-attr", "-z", "filter", "--"])
                .args(&entries)
                .env_clear()
                .envs(&self.environment);
            let output = self
                .run_command(command, std::time::Duration::from_secs(10), 256 * 1024)
                .await?;
            let fields = output.split(|byte| *byte == 0).collect::<Vec<_>>();
            if fields.last() != Some(&b"".as_slice()) || (fields.len() - 1) != entries.len() * 3 {
                return Err(GitToolError::InvalidInput(
                    "Git filter inspection returned an invalid result".into(),
                ));
            }
            for (fields, expected) in fields[..fields.len() - 1]
                .as_chunks::<3>()
                .0
                .iter()
                .zip(&entries)
            {
                if fields[0] != expected.as_bytes()
                    || fields[1] != b"filter"
                    || !config.filter_attribute_is_inert(fields[2])
                {
                    return Err(GitToolError::InvalidInput("Selected Git sources use a configured filter; use a separately authorized Git invocation".into()));
                }
            }
        }
        if let Some(target) = &self.checkout_target
            && !self.paths.is_empty()
        {
            for source in [format!("--source={}", target.commit), "--cached".to_owned()] {
                let mut command = tokio::process::Command::new(&self.program);
                command
                    .current_dir(&self.repository.workdir)
                    .args(["check-attr", "-z", &source, "filter", "--"])
                    .args(&self.paths)
                    .env_clear()
                    .envs(&self.environment);
                let output = self
                    .run_command(command, std::time::Duration::from_secs(10), 256 * 1024)
                    .await?;
                let fields: Vec<_> = output.split(|byte| *byte == 0).collect();
                if fields.last() != Some(&b"".as_slice())
                    || fields.len() != self.paths.len() * 3 + 1
                {
                    return Err(GitToolError::InvalidInput(
                        "Invalid checkout target attribute evidence".into(),
                    ));
                }
                for (fields, path) in fields[..fields.len() - 1]
                    .as_chunks::<3>()
                    .0
                    .iter()
                    .zip(&self.paths)
                {
                    if fields[0] != path.as_bytes()
                        || fields[1] != b"filter"
                        || !config.filter_attribute_is_inert(fields[2])
                    {
                        return Err(GitToolError::InvalidInput("Checkout target uses a configured filter; use a separately authorized invocation".into()));
                    }
                }
            }
        }
        self.revalidate()?;
        auxiliary.revalidate(authorizations)?;
        if let Some((children, child_inspection)) = &child_evidence {
            if children != &self.capture_admitted_children()? {
                return Err(GitToolError::InvalidInput(
                    "Submodule HEAD or index changed during admitted inspection".into(),
                ));
            }
            child_inspection.revalidate()?;
        }
        if let Some(expected) = status
            && expected
                != super::status_state::StatusState::capture(
                    &self.repository.workdir,
                    &self
                        .gitlinks
                        .iter()
                        .map(|link| self.repository.workdir.join(&link.path))
                        .collect::<Vec<_>>(),
                )?
        {
            return Err(GitToolError::InvalidInput(
                "Git status inputs changed during inspection".into(),
            ));
        }
        self.revalidate_maintenance_program()?;
        if let Operation::Checkout(checkout) = &self.operation {
            let (current, tracking_source) = self.resolve_checkout_target(checkout, config).await?;
            if self
                .checkout_target
                .as_ref()
                .is_none_or(|target| target.commit != current)
                || tracking_source != self.checkout_tracking_source
            {
                return Err(GitToolError::InvalidInput(
                    "Checkout target changed after approval".into(),
                ));
            }
            if let Some(expected) = &self.checkout_new_branch
                && self.inspect_checkout_branch(&checkout.branch).await? != *expected
            {
                return Err(GitToolError::InvalidInput(
                    "Checkout branch name changed after approval".into(),
                ));
            }
            if let Some(name) = &self.checkout_new_branch {
                let start = self.inspect_checkout_start(checkout.create).await?;
                if config.checkout_tracking_writes(name, start.as_deref())?
                    != self.checkout_tracking_writes
                {
                    return Err(GitToolError::InvalidInput(
                        "Checkout tracking writes changed after approval".into(),
                    ));
                }
            }
            self.revalidate()?;
        }
        if let Some(expected) = push_attributes
            && expected
                != super::status_state::StatusState::capture_pack_attributes(
                    &self.repository.workdir,
                )?
        {
            return Err(GitToolError::InvalidInput(
                "Push attribute sources changed during inspection".into(),
            ));
        }
        let receiver = if self.receiver_roots.is_empty() {
            None
        } else {
            Some(receiver_command(&self.program)?)
        };
        let mut args = vec!["--no-pager", "--literal-pathspecs"];
        match &self.operation {
            Operation::Pull(_) => {
                return Err(GitToolError::InvalidInput(
                    "Pull cannot use the generic Git mutation dispatcher".into(),
                ));
            }
            Operation::Push(push) => {
                args.extend([
                    "-c",
                    "maintenance.autoDetach=false",
                    "-c",
                    "gc.autoDetach=false",
                    "push",
                ]);
                if let Some(receiver) = &receiver {
                    args.extend(["--receive-pack", receiver]);
                }
                if push.force {
                    args.push("--force");
                }
                args.push("--");
                args.push(push.remote.as_deref().unwrap_or("origin"));
                if let Some(branch) = &push.branch {
                    args.push(branch);
                }
            }
            Operation::Add => {
                args.extend(["add", "--"]);
                args.extend(self.paths.iter().map(String::as_str));
            }
            Operation::Checkout(checkout) => {
                args.push("checkout");
                if checkout.create {
                    args.push("-b");
                }
                args.push(if checkout.branch == "-" && !checkout.create {
                    "@{-1}"
                } else {
                    &checkout.branch
                });
                args.push("--");
            }
            Operation::Commit(commit) => {
                args.extend([
                    "-c",
                    "maintenance.autoDetach=false",
                    "-c",
                    "gc.autoDetach=false",
                    "commit",
                    "-m",
                    &commit.message,
                ]);
                if commit.all {
                    args.push("--all");
                }
            }
        }
        let expected_command = command::FrozenCommand::new(
            &self.program,
            &self.repository.workdir,
            &args,
            &self.environment,
        )?;
        let command = if matches!(self.operation, Operation::Add) {
            let retained = self.native_add_command.as_ref().ok_or_else(|| {
                GitToolError::InvalidInput("Native Add command missing; no command started".into())
            })?;
            if retained != &expected_command {
                return Err(GitToolError::InvalidInput(
                    "Native Add command changed after preparation; request a fresh authorization"
                        .into(),
                ));
            }
            retained.command()
        } else if matches!(self.operation, Operation::Commit(_)) {
            let retained = self.native_commit_command.as_ref().ok_or_else(|| {
                GitToolError::InvalidInput(
                    "Native Commit command missing; no command started".into(),
                )
            })?;
            if retained != &expected_command {
                return Err(GitToolError::InvalidInput(
                    "Native Commit command changed after preparation; request a fresh authorization".into(),
                ));
            }
            retained.command()
        } else {
            expected_command.command()
        };
        if !self.receiver_roots.is_empty()
            && self.inspect_receiver_storage(authorizations)? != self.receiver_storage
        {
            return Err(GitToolError::InvalidInput(
                "Receiver metadata changed during final inspection".into(),
            ));
        }
        if !self.receiver_roots.is_empty() {
            self.revalidate_receiver_shell_selection().await?;
            self.revalidate()?;
            self.revalidate_receiver_roots(authorizations)?;
            self.validate_receiver_shell_baseline()?;
        }
        // Receiver shell selection and Checkout target probes above can run
        // after attribute inspection. Keep the original auxiliary evidence
        // bound until all probes finish, immediately before mutation starts.
        auxiliary.revalidate(authorizations)?;
        self.run_command(command, std::time::Duration::from_secs(300), 256 * 1024)
            .await.map_err(|error| {
                if matches!(self.operation, Operation::Push(_)) {
                    GitToolError::Git(format!("{error}; push may have partially updated remote references; inspect every destination before retrying"))
                } else if self.checkout_deletion.is_some() {
                    GitToolError::Git(format!("{error}; checkout may have partially removed child entries or updated repository state; inspect the worktree before retrying"))
                } else { error }
            })?;
        match &self.operation {
            Operation::Pull(_) => Err(GitToolError::InvalidInput(
                "Pull cannot use the generic Git mutation result".into(),
            )),
            Operation::Push(push) => Ok(format!(
                "pushed to {}",
                push.remote.as_deref().unwrap_or("origin")
            )),
            Operation::Checkout(checkout) => Ok(format!("switched to branch {}", checkout.branch)),
            Operation::Add => Ok(format!("staged {} file(s)", self.paths.len())),
            Operation::Commit(commit) => Ok(format!(
                "committed: {}",
                commit.message.lines().next().unwrap_or_default()
            )),
        }
    }

    fn sender_program_target(&self) -> &Path {
        self.sender_program_binding
            .as_ref()
            .map_or(&self.program, |binding| &binding.canonical)
    }

    fn revalidate_maintenance_program(&self) -> Result<(), GitToolError> {
        if let Some(program) = &self.maintenance_program
            && (program.canonicalize().ok().as_deref() != Some(self.sender_program_target())
                || capture_program(self.sender_program_target())? != self.program_entry)
        {
            #[cfg(test)]
            eprintln!(
                "Git maintenance binding rejected: child={program:?}, canonical={:?}, bound={:?}",
                program.canonicalize(),
                self.program,
            );
            return Err(GitToolError::InvalidInput("Git maintenance executable differs from the bound Git binary; use a separately authorized invocation".into()));
        }
        Ok(())
    }

    fn checkout_child_effective_worktree<'a>(&'a self, discovery: &'a Path) -> &'a Path {
        self.checkout_child_layouts
            .iter()
            .zip(&self.checkout_child_bootstrap)
            .find(|(layout, _)| layout.canonical == discovery)
            .map(|(_, (worktree, _))| worktree.as_path())
            .unwrap_or(discovery)
    }

    /// Called only after the continuation proof is consumed. Shared child
    /// metadata discovered in stage one is not read before this admission.
    fn capture_admitted_children(&self) -> Result<Vec<RepositoryState>, GitToolError> {
        if self.config.is_none() {
            return Err(GitToolError::InvalidInput(
                "Submodule metadata requires continuation admission".into(),
            ));
        }
        if self.child_configs.len() != self.child_locations.len() {
            return Err(GitToolError::Git(
                "Incomplete child discovery evidence".into(),
            ));
        }
        self.child_configs
            .iter()
            .zip(&self.child_locations)
            .map(|((workdir, config), (path, location))| {
                config.revalidate()?;
                if &super::repository_state::ChildLocation::capture(path)? != location {
                    return Err(GitToolError::InvalidInput(
                        "Submodule location changed before metadata capture".into(),
                    ));
                }
                let state = RepositoryState::capture(workdir)?;
                let canonical_workdir = self
                    .checkout_child_effective_worktree(workdir)
                    .canonicalize()
                    .map_err(|_| {
                        GitToolError::InvalidInput("Submodule checkout unavailable".into())
                    })?;
                if state.workdir != canonical_workdir
                    || state.git_dir != location.git_dir
                    || state.common_dir != location.common_dir
                {
                    return Err(GitToolError::InvalidInput(
                        "Submodule metadata does not match admitted locations".into(),
                    ));
                }
                if &super::repository_state::ChildLocation::capture(path)? != location {
                    return Err(GitToolError::InvalidInput(
                        "Submodule location changed during metadata capture".into(),
                    ));
                }
                config.revalidate()?;
                Ok(state)
            })
            .collect()
    }

    fn discover_child_attribute_targets(
        mut self: Box<Self>,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<Box<Self>, GitToolError> {
        // Index contents select worktree rule paths. Admit every child index
        // and worktree before reading any of them, then publish fresh resolved
        // target Reads before capturing attribute bodies or running check-attr.
        for ((workdir, _), (_, location)) in self.child_configs.iter().zip(&self.child_locations) {
            for path in [
                self.checkout_child_effective_worktree(workdir),
                &location.git_dir,
                &location.common_dir,
            ] {
                if !authorizations.iter().any(|token| {
                    token.authorizes_path(
                        self.operation.name(),
                        ToolNature::Read,
                        &self.workspace,
                        path.to_str().unwrap_or_default(),
                    )
                }) {
                    return Err(GitToolError::InvalidInput(
                        "Child attribute discovery lacks metadata/index or worktree Read admission"
                            .into(),
                    ));
                }
            }
        }
        self.revalidate()?;
        self.capture_admitted_children()?;
        let mut sources = std::collections::BTreeSet::new();
        for (workdir, _) in &self.child_configs {
            let effective = self.checkout_child_effective_worktree(workdir);
            let (paths, nested) = tracked_commit_sources_at(workdir, effective)?;
            if !nested.is_empty() {
                return Err(GitToolError::InvalidInput("Nested submodule status requires additional configuration discovery; use a separately authorized invocation".into()));
            }
            for path in paths {
                let absolute = effective.join(path);
                let mut ancestor = absolute.parent();
                while let Some(directory) = ancestor {
                    if !directory.starts_with(effective) {
                        break;
                    }
                    sources.insert(directory.join(".gitattributes"));
                    ancestor = directory.parent();
                }
            }
        }
        let auxiliary = self
            .auxiliary_paths
            .as_mut()
            .ok_or_else(|| GitToolError::InvalidInput("Missing child auxiliary resolver".into()))?;
        let new_sources = sources
            .iter()
            .any(|path| auxiliary.related_paths_for(path).is_none());
        auxiliary.extend(self.operation.name(), sources, authorizations)?;
        auxiliary.add_read_frontier(&mut self.facets)?;
        self.child_attributes_discovered = true;
        if new_sources {
            self.identity = PreparedPlanIdentity::new();
        }
        Ok(self)
    }

    async fn inspect_child_filters(
        &self,
        auxiliary: &super::auxiliary_paths::AuxiliaryPaths,
        authorizations: &[talos_core::tool::ToolExecutionAuthorization],
    ) -> Result<ChildInspectionEvidence, GitToolError> {
        // External/configured sources already belong to the invocation-owned
        // resolver. Never reopen their raw spellings through the worktree snapshot.
        auxiliary.revalidate(authorizations)?;
        for path in &self.child_attributes {
            if auxiliary.leaf_evidence(path).is_none() {
                return Err(GitToolError::InvalidInput(
                    "Missing child auxiliary evidence".into(),
                ));
            }
        }
        let mut attribute_paths = std::collections::BTreeSet::new();
        let mut child_paths = Vec::new();
        let mut child_sources = Vec::new();
        let mut status = Vec::new();
        for (workdir, _) in &self.child_configs {
            let effective = self.checkout_child_effective_worktree(workdir);
            let (paths, nested) = tracked_commit_sources_at(workdir, effective)?;
            if !nested.is_empty() {
                return Err(GitToolError::InvalidInput("Nested submodule status requires additional configuration discovery; use a separately authorized invocation".into()));
            }
            status.push((
                effective.to_owned(),
                super::status_state::StatusState::capture(effective, &[])?,
            ));
            for path in &paths {
                let absolute = effective.join(path);
                let mut ancestor = absolute.parent();
                while let Some(directory) = ancestor {
                    if !directory.starts_with(effective) {
                        break;
                    }
                    let path = directory.join(".gitattributes");
                    if auxiliary.leaf_evidence(&path).is_none() {
                        return Err(GitToolError::InvalidInput(
                            "Child attribute source requires fresh target Read discovery".into(),
                        ));
                    }
                    attribute_paths.insert(path);
                    ancestor = directory.parent();
                }
            }
            child_sources.push(capture_tracked_sources(effective, &paths)?);
            child_paths.push(paths);
        }
        // Bodies and identities are owned by the invocation resolver. Keep
        // only its admitted entries here for Checkout deletion protection.
        let attributes = attribute_paths
            .into_iter()
            .filter(|path| {
                auxiliary
                    .leaf_evidence(path)
                    .is_some_and(|evidence| !matches!(evidence.entry, snapshot::Entry::Missing))
            })
            .collect();
        for (((workdir, config), (_, location)), paths) in self
            .child_configs
            .iter()
            .zip(&self.child_locations)
            .zip(&child_paths)
        {
            config.verify_host(&self.inspect_config_at(workdir).await?)?;
            if paths.is_empty() {
                continue;
            }
            let command = self.child_filter_command(
                &location.git_dir,
                self.checkout_child_effective_worktree(workdir),
                paths,
            )?;
            let output = self
                .run_command(command, std::time::Duration::from_secs(10), 256 * 1024)
                .await?;
            let fields = output.split(|byte| *byte == 0).collect::<Vec<_>>();
            if fields.last() != Some(&b"".as_slice()) || fields.len() - 1 != paths.len() * 3 {
                return Err(GitToolError::InvalidInput(
                    "Submodule filter inspection returned invalid output".into(),
                ));
            }
            for (fields, path) in fields[..fields.len() - 1]
                .as_chunks::<3>()
                .0
                .iter()
                .zip(paths)
            {
                if fields[0] != path.as_bytes()
                    || fields[1] != b"filter"
                    || !config.filter_attribute_is_inert(fields[2])
                {
                    return Err(GitToolError::InvalidInput("Submodule status uses a configured filter; use a separately authorized invocation".into()));
                }
            }
        }
        auxiliary.revalidate(authorizations)?;
        let evidence = ChildInspectionEvidence {
            status,
            attributes,
            sources: self
                .child_configs
                .iter()
                .map(|(workdir, _)| {
                    (
                        workdir.clone(),
                        self.checkout_child_effective_worktree(workdir).to_owned(),
                    )
                })
                .zip(child_paths)
                .zip(child_sources)
                .map(|(((discovery, workdir), paths), sources)| {
                    (discovery, workdir, paths, sources)
                })
                .collect(),
        };
        evidence.revalidate()?;
        Ok(evidence)
    }

    fn child_filter_command(
        &self,
        gitdir: &Path,
        worktree: &Path,
        paths: &[String],
    ) -> Result<tokio::process::Command, GitToolError> {
        let mut command = tokio::process::Command::new(&self.program);
        command
            // Index paths are root-relative, even when discovery was inside
            // core.worktree. Explicit roots prevent cwd-prefix reinterpretation.
            .current_dir(worktree)
            .arg("--git-dir")
            .arg(host_path::directory_argument(gitdir)?.as_os_str())
            .arg("--work-tree")
            .arg(host_path::directory_argument(worktree)?.as_os_str())
            .args(["--no-pager", "check-attr", "-z", "filter", "--"])
            .args(paths)
            .env_clear()
            .envs(&self.environment);
        Ok(command)
    }

    fn revalidate(&self) -> Result<(), GitToolError> {
        if let Operation::Commit(input) = &self.operation {
            let expected = command::FrozenCommand::commit(
                &self.program,
                &self.repository.workdir,
                input,
                &self.environment,
            )?;
            if self.native_commit_command.as_ref() != Some(&expected) {
                return Err(GitToolError::InvalidInput(
                    "Native Commit command changed after preparation; request a fresh authorization".into(),
                ));
            }
        }
        if matches!(self.operation, Operation::Add) {
            let expected = command::FrozenCommand::add(
                &self.program,
                &self.repository.workdir,
                &self.paths,
                &self.environment,
            )?;
            if self.native_add_command.as_ref() != Some(&expected) {
                return Err(GitToolError::InvalidInput(
                    "Native Add command changed after preparation; request a fresh authorization"
                        .into(),
                ));
            }
        }
        if let Operation::Pull(pull) = &self.operation {
            let expected = command::FrozenCommand::pull(
                &self.program,
                &self.repository.workdir,
                pull,
                &self.environment,
            )?;
            if self.native_pull_command.as_ref() != Some(&expected) {
                return Err(GitToolError::InvalidInput(
                    "Native Pull command changed after preparation; request a fresh authorization"
                        .into(),
                ));
            }
        }

        if let Some(binding) = &self.sender_program_binding
            && (self.program.canonicalize().ok().as_ref() != Some(&binding.canonical)
                || binding.spelling.canonicalize().ok().as_ref() != Some(&binding.canonical)
                || snapshot::PathIdentity::capture(&binding.spelling)
                    .ok()
                    .as_ref()
                    != Some(&binding.identity))
        {
            return Err(GitToolError::InvalidInput(
                "Internal Git executable path identity changed after admission".into(),
            ));
        }
        if let Some((path, entry)) = &self.sender_entry_program
            && capture_program(path)? != *entry
        {
            return Err(GitToolError::InvalidInput(
                "Entry Git executable changed after internal program binding".into(),
            ));
        }
        // Refuse a moved layout pointer before rediscovery can read a new root
        // using the retained plan's old-root admission.
        self.repository.revalidate_location()?;
        if let Some(sources) = &self.target_config_sources {
            sources.revalidate().map_err(|_| {
                GitToolError::InvalidInput(
                    "Target Git configuration sources changed after discovery".into(),
                )
            })?;
        }
        if !self.target_hook_evidence.is_empty() {
            if self
                .target_config_sources
                .as_ref()
                .is_none_or(|sources| !sources.complete())
            {
                return Err(GitToolError::InvalidInput(
                    "Target hook ancestor evidence is incomplete".into(),
                ));
            }
            let paths = self
                .target_hook_evidence
                .iter()
                .map(|(path, _)| path.clone())
                .collect::<Vec<_>>();
            if super::config_state::ConfigState::capture_checkout_reload_hooks(&paths)?
                != self.target_hook_evidence
            {
                return Err(GitToolError::InvalidInput(
                    "Target checkout hook evidence changed after discovery".into(),
                ));
            }
        }
        if let Some(inventory) = &self.checkout_deletion {
            inventory.revalidate()?;
        }
        if let Some(expected) = &self.checkout_parents
            && &super::checkout_state::TargetParents::capture(
                &self.repository.workdir,
                &self.paths,
            )? != expected
        {
            return Err(GitToolError::InvalidInput(
                "Checkout parent paths changed after discovery".into(),
            ));
        }
        for (path, expected) in &self.checkout_empty_gitlinks {
            // Refuse a replacement before enumerating its directory names.
            if &snapshot::PathIdentity::capture(path)
                .map_err(|error| GitToolError::InvalidInput(error.to_string()))?
                != expected
                || &capture_empty_gitlink(path)? != expected
            {
                return Err(GitToolError::InvalidInput(
                    "Empty submodule directory changed after discovery".into(),
                ));
            }
        }
        for (path, expected) in &self.checkout_new_gitlinks {
            if snapshot::PathIdentity::capture(path)
                .map_err(|error| GitToolError::InvalidInput(error.to_string()))?
                != expected.identity
                || capture_new_gitlink_directory(path, true)? != *expected
            {
                return Err(GitToolError::InvalidInput(
                    "New submodule directory changed after discovery".into(),
                ));
            }
        }
        if matches!(self.operation, Operation::Push(_)) {
            super::remote_state::reject_legacy_remotes(&self.repository.common_dir)?;
        }
        for layout in &self.checkout_child_layouts {
            layout.revalidate(&layout.canonical)?;
        }
        for metadata in &self.checkout_child_metadata {
            metadata.revalidate(&metadata.canonical)?;
        }
        for (_, config) in &self.checkout_child_bootstrap {
            config.revalidate()?;
        }
        for startup in &self.checkout_child_startup {
            startup.revalidate()?;
        }
        for (spelling, canonical, identity) in &self.checkout_child_worktrees {
            if spelling.canonicalize().ok().as_ref() != Some(canonical)
                || snapshot::PathIdentity::capture(spelling).ok().as_ref() != Some(identity)
            {
                return Err(GitToolError::InvalidInput(
                    "Child worktree changed after discovery".into(),
                ));
            }
        }
        for (path, expected) in &self.child_locations {
            if &super::repository_state::ChildLocation::capture(path)? != expected {
                return Err(GitToolError::InvalidInput(
                    "Submodule location changed after discovery".into(),
                ));
            }
        }
        for (_, config) in &self.child_configs {
            config.revalidate()?;
        }
        let current_environment = execution_environment();
        validate_environment(&current_environment)?;
        if trace_suppressed_environment(current_environment) != self.environment {
            return Err(GitToolError::InvalidInput(
                "Git environment changed after preparation; request a fresh authorization".into(),
            ));
        }
        if capture_program(self.sender_program_target())? != self.program_entry {
            return Err(GitToolError::InvalidInput(
                "Git executable changed after preparation; request a fresh authorization".into(),
            ));
        }
        let current_repository = RepositoryState::capture(&self.workspace)?;
        if current_repository != self.repository {
            return Err(GitToolError::InvalidInput(format!(
                "Git repository metadata changed after preparation; request a fresh authorization (changed categories: {})",
                self.repository
                    .changed_fields(&current_repository)
                    .join(", ")
            )));
        }
        if match &self.operation {
            Operation::Push(_) | Operation::Pull(_) => false,
            Operation::Add => {
                literal_add_paths(
                    &self.workspace,
                    &self.repository.workdir,
                    &serde_json::from_value::<GitAddInput>(self.input.clone())
                        .map_err(|error| GitToolError::InvalidInput(error.to_string()))?
                        .paths,
                )? != self.paths
            }
            Operation::Checkout(_) => {
                if tracked_commit_sources(&self.repository.workdir)?.1 != self.gitlinks {
                    true
                } else if let Some(expected) = &self.checkout_parents {
                    &super::checkout_state::TargetParents::capture(
                        &self.repository.workdir,
                        &self.paths,
                    )? != expected
                } else {
                    false
                }
            }
            Operation::Commit(_) => {
                let (paths, gitlinks) = tracked_commit_sources(&self.repository.workdir)?;
                paths != self.paths || gitlinks != self.gitlinks
            }
        } || match &self.operation {
            Operation::Push(_) | Operation::Pull(_) => Vec::new(),
            Operation::Add => capture_add_sources(&self.repository.workdir, &self.paths)?,
            Operation::Commit(_) | Operation::Checkout(_) => {
                capture_tracked_sources(&self.repository.workdir, &self.paths)?
            }
        } != self.sources
        {
            return Err(GitToolError::InvalidInput(
                "Git repository or sources changed after preparation; request a fresh authorization".into(),
            ));
        }
        if let Some(expected) = &self.commit_state
            && &super::repository_state::CommitState::capture(&self.repository)? != expected
        {
            return Err(GitToolError::InvalidInput(
                "Git commit state changed after preparation".into(),
            ));
        }
        if let Some(config) = &self.config {
            config.revalidate()?;
        }
        Ok(())
    }
}

// Both commit modes may refresh tracked entries across the repository, including
// paths outside a subworkspace. --all additionally stages them. Untracked files
// are not sources for either mode.
#[derive(Debug, PartialEq, Eq)]
struct GitlinkEvidence {
    path: String,
    index_oid: String,
    entry: snapshot::Entry,
    metadata_path: Option<PathBuf>,
    marker: snapshot::Entry,
}

#[cfg(unix)]
fn tracked_symlink(workdir: &Path, path: &str) -> Result<bool, GitToolError> {
    std::panic::catch_unwind(|| {
        let repo = super::discover_repo(workdir)?;
        let index = repo
            .index_or_empty()
            .map_err(|error| GitToolError::Git(error.to_string()))?;
        Ok(index.entries().iter().any(|entry| {
            entry.stage_raw() == 0
                && entry.mode.bits() == 0o120000
                && entry.path(&index) == path.as_bytes()
        }))
    })
    .unwrap_or_else(|_| Err(GitToolError::Git("Git symlink inspection failed".into())))
}

fn tracked_commit_sources(
    workdir: &Path,
) -> Result<(Vec<String>, Vec<GitlinkEvidence>), GitToolError> {
    tracked_commit_sources_at(workdir, workdir)
}

fn tracked_commit_sources_at(
    discovery: &Path,
    workdir: &Path,
) -> Result<(Vec<String>, Vec<GitlinkEvidence>), GitToolError> {
    std::panic::catch_unwind(|| {
        let repo = super::discover_repo(discovery)?;
        let index = repo
            .index_or_empty()
            .map_err(|error| GitToolError::Git(error.to_string()))?;
        let mut paths = std::collections::BTreeSet::new();
        let mut gitlinks = Vec::new();
        for entry in index.entries() {
            if entry.stage_raw() != 0 {
                return Err(GitToolError::InvalidInput(
                    "Git commit has unmerged index entries; resolve conflicts before preparing a commit".into(),
                ));
            }
            let path = std::str::from_utf8(entry.path(&index))
                .map_err(|_| GitToolError::InvalidInput("Tracked Git path is not UTF-8".into()))?;
            if entry.mode.bits() == 0o160000 {
                let normalized = literal_add_paths(workdir, workdir, &[path.to_owned()])?;
                let path = normalized.into_iter().next().ok_or_else(|| GitToolError::InvalidInput("missing gitlink path".into()))?;
                gitlinks.push(capture_gitlink(workdir, path, entry.id.to_string())?);
            } else {
                paths.insert(path.to_owned());
            }
        }
        let paths = if paths.is_empty() { Vec::new() } else {
            literal_add_paths(workdir, workdir, &paths.into_iter().collect::<Vec<_>>())?
        };
        Ok((paths, gitlinks))
    })
    .unwrap_or_else(|_| {
        Err(GitToolError::Git(
            "Git tracked source inspection failed".into(),
        ))
    })
}

fn capture_tracked_sources(
    workdir: &Path,
    paths: &[String],
) -> Result<Vec<snapshot::Entry>, GitToolError> {
    let mut budget = snapshot::Budget {
        entries: 4096,
        bytes: 16 * 1024 * 1024,
    };
    paths
        .iter()
        .map(|path| {
            let absolute = workdir.join(path);
            if std::fs::symlink_metadata(&absolute).is_ok_and(|metadata| metadata.is_dir()) {
                // Replacing a tracked file with a directory does not make its
                // untracked descendants sources for status or commit --all.
                budget.entries = budget.entries.checked_sub(1).ok_or_else(|| {
                    GitToolError::InvalidInput("Submodule source snapshot limit exceeded".into())
                })?;
                return Ok(snapshot::Entry::Directory(Vec::new()));
            }
            snapshot::Entry::capture(&absolute, &mut budget)
                .map_err(|error| GitToolError::InvalidInput(error.to_string()))
        })
        .collect()
}

// Separate from removed-gitlink evidence: deletion still requires an empty
// directory, whereas nonrecursive checkout preserves these untracked names.
#[derive(Debug, PartialEq, Eq)]
struct NewGitlinkDirectory {
    identity: snapshot::PathIdentity,
    names: Vec<std::ffi::OsString>,
}

fn capture_new_gitlink_directory(
    path: &Path,
    allow_marker: bool,
) -> Result<NewGitlinkDirectory, GitToolError> {
    let invalid = || {
        GitToolError::InvalidInput(
            "New gitlink requires a stable uninitialized directory within discovery limits".into(),
        )
    };
    let metadata = std::fs::symlink_metadata(path).map_err(|_| invalid())?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes()
            & windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT
            != 0
        {
            return Err(invalid());
        }
    }
    if !metadata.is_dir() {
        return Err(invalid());
    }
    let identity = snapshot::PathIdentity::capture(path).map_err(|_| invalid())?;
    let mut names = Vec::new();
    let mut bytes = 0usize;
    for entry in std::fs::read_dir(path).map_err(|_| invalid())? {
        let name = entry.map_err(|_| invalid())?.file_name();
        bytes = bytes
            .checked_add(name.as_encoded_bytes().len())
            .ok_or_else(invalid)?;
        if names.len() >= 4096 || bytes > 1024 * 1024 {
            return Err(invalid());
        }
        // No child stat/open: reject metadata names and filesystem aliases
        // from names alone. On Windows, enable Git's device/illegal-name
        // protections before allowing the bounded name inventory to continue;
        // the mutation stage still revalidates the directory identity and
        // repeats this complete inventory.
        let valid = (allow_marker && name == ".git")
            || std::panic::catch_unwind(|| {
                gix::validate::path::component(
                    name.as_encoded_bytes().into(),
                    None,
                    gix::validate::path::component::Options {
                        protect_windows: cfg!(windows),
                        protect_hfs: true,
                        protect_ntfs: true,
                    },
                )
                .is_ok()
            })
            .map_err(|_| invalid())?;
        if !valid {
            return Err(invalid());
        }
        names.push(name);
    }
    names.sort();
    if snapshot::PathIdentity::capture(path).map_err(|_| invalid())? != identity {
        return Err(invalid());
    }
    Ok(NewGitlinkDirectory { identity, names })
}

fn capture_empty_gitlink(path: &Path) -> Result<snapshot::PathIdentity, GitToolError> {
    let invalid = || {
        GitToolError::InvalidInput(
            "Checkout submodule path kind requires a stable empty directory".into(),
        )
    };
    let metadata = std::fs::symlink_metadata(path).map_err(|_| invalid())?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes()
            & windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT
            != 0
        {
            return Err(invalid());
        }
    }
    if !metadata.is_dir() {
        return Err(invalid());
    }
    let before = snapshot::PathIdentity::capture(path).map_err(|_| invalid())?;
    if std::fs::read_dir(path)
        .map_err(|_| invalid())?
        .next()
        .is_some()
    {
        return Err(invalid());
    }
    if snapshot::PathIdentity::capture(path).map_err(|_| invalid())? != before {
        return Err(invalid());
    }
    Ok(before)
}

fn capture_gitlink(
    workdir: &Path,
    path: String,
    index_oid: String,
) -> Result<GitlinkEvidence, GitToolError> {
    let absolute = workdir.join(&path);
    let metadata = match std::fs::symlink_metadata(&absolute) {
        Ok(metadata) => Some(metadata),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(GitToolError::InvalidInput(error.to_string())),
    };
    let mut marker_entry = snapshot::Entry::Missing;
    let (entry, metadata_path) = if metadata.as_ref().is_some_and(|metadata| metadata.is_dir()) {
        let marker = absolute.join(".git");
        let marker_metadata = match std::fs::symlink_metadata(&marker) {
            Ok(metadata) => Some(metadata),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(GitToolError::InvalidInput(error.to_string())),
        };
        let metadata_path = match marker_metadata {
            Some(metadata) if metadata.is_dir() => {
                marker_entry = snapshot::Entry::Directory(Vec::new());
                Some(marker)
            }
            Some(metadata) if metadata.is_file() => {
                marker_entry = snapshot::Entry::capture(&marker, &mut snapshot::Budget { entries: 1, bytes: 16 * 1024 })
                    .map_err(|error| GitToolError::InvalidInput(error.to_string()))?;
                let snapshot::Entry::File(_, bytes) = &marker_entry else {
                    return Err(GitToolError::InvalidInput("Submodule marker changed during inspection".into()));
                };
                let raw = bytes.strip_prefix(b"gitdir: ").ok_or_else(|| GitToolError::InvalidInput("Invalid submodule gitdir marker".into()))?;
                let raw = raw.strip_suffix(b"\n").unwrap_or(raw);
                let raw = raw.strip_suffix(b"\r").unwrap_or(raw);
                let path = PathBuf::from(std::str::from_utf8(raw).map_err(|_| GitToolError::InvalidInput("Submodule gitdir path is not UTF-8".into()))?);
                if path.as_os_str().is_empty() || raw.contains(&0) {
                    return Err(GitToolError::InvalidInput("Invalid submodule gitdir path".into()));
                }
                // Resolve only the pathname. No config/index/HEAD/object reads
                // from its destination occur before this Path facet is admitted.
                Some(if path.is_absolute() { path } else { absolute.join(path) })
            }
            Some(_) => return Err(GitToolError::InvalidInput("Submodule metadata link or special entry requires a separately authorized invocation".into())),
            None => None,
        };
        (snapshot::Entry::Directory(Vec::new()), metadata_path)
    } else {
        let entry = snapshot::Entry::capture(
            &absolute,
            &mut snapshot::Budget {
                entries: 1,
                bytes: 16 * 1024 * 1024,
            },
        )
        .map_err(|error| GitToolError::InvalidInput(error.to_string()))?;
        (entry, None)
    };
    Ok(GitlinkEvidence {
        path,
        index_oid,
        entry,
        metadata_path,
        marker: marker_entry,
    })
}

fn selected_leaves(
    path: &Path,
    entry: &snapshot::Entry,
    output: &mut Vec<String>,
) -> Result<(), GitToolError> {
    match entry {
        snapshot::Entry::Directory(children) => {
            for (name, child) in children {
                selected_leaves(&path.join(name), child, output)?;
            }
        }
        snapshot::Entry::File(..) => output.push(
            path.to_str()
                .ok_or_else(|| GitToolError::InvalidInput("Git source name is not UTF-8".into()))?
                .to_owned(),
        ),
        snapshot::Entry::Missing | snapshot::Entry::Link(_) => {}
    }
    Ok(())
}

fn selected_directory_rules(
    path: &Path,
    entry: &snapshot::Entry,
    output: &mut std::collections::BTreeSet<PathBuf>,
) {
    if let snapshot::Entry::Directory(children) = entry {
        output.insert(path.join(".gitattributes"));
        output.insert(path.join(".gitignore"));
        for (name, child) in children {
            selected_directory_rules(&path.join(name), child, output);
        }
    }
}

fn execution_environment() -> BTreeMap<OsString, OsString> {
    let mut environment = std::env::vars_os().collect::<BTreeMap<_, _>>();
    // Reuse the existing process-hardening loader policy without mutating the
    // parent process. This does not remove any Git configuration override.
    let dangerous = talos_sandbox::hardening::ProcessHardening::dangerous_env_var_names();
    environment.retain(|key, _| {
        !dangerous.iter().any(|name| {
            if cfg!(windows) {
                key.to_str()
                    .is_some_and(|key| key.eq_ignore_ascii_case(name))
            } else {
                key == std::ffi::OsStr::new(name)
            }
        })
    });
    environment
}

/// Trace2 targets are loaded before ordinary configuration inspection. Disable
/// diagnostic output in structured children so read-only discovery cannot write
/// an unapproved file or socket from system/global configuration.
fn trace_suppressed_environment(
    mut environment: BTreeMap<OsString, OsString>,
) -> BTreeMap<OsString, OsString> {
    for name in ["GIT_TRACE2", "GIT_TRACE2_EVENT", "GIT_TRACE2_PERF"] {
        environment.insert(OsString::from(name), OsString::from("0"));
    }
    environment
}

fn validate_environment(environment: &BTreeMap<OsString, OsString>) -> Result<(), GitToolError> {
    let mut bytes = 0usize;
    for (name, value) in environment {
        bytes = bytes.saturating_add(name.len()).saturating_add(value.len());
        let upper = name.to_string_lossy().to_ascii_uppercase();
        if matches!(upper.as_str(), "HOME" | "XDG_CONFIG_HOME")
            && value.as_encoded_bytes().contains(&b'\n')
        {
            return Err(GitToolError::InvalidInput("Git configuration home paths containing newlines cannot be discovered unambiguously; use a separately authorized invocation".into()));
        }
        // The structured tool always captures output with --no-pager. Retain
        // GIT_PAGER in the frozen environment, but it cannot start a pager.
        if upper.starts_with("GIT_") && upper != "GIT_PAGER" {
            // Git environment switches include repository/object redirection,
            // config injection and trace files. Never silently strip them and
            // then run a different command than the user configured.
            return Err(GitToolError::InvalidInput(
                "Git environment overrides require a separately authorized Git invocation".into(),
            ));
        }
    }
    if bytes > 1024 * 1024 {
        return Err(GitToolError::InvalidInput(
            "Git environment snapshot exceeds its limit".into(),
        ));
    }
    Ok(())
}

/// Do not silently discard shell startup configuration. Local receive-pack
/// transport uses a shell, so inherited startup scripts/functions and tracing
/// expressions must use a separately authorized invocation.
fn validate_receiver_shell_environment(
    environment: &BTreeMap<OsString, OsString>,
) -> Result<(), GitToolError> {
    for name in environment.keys() {
        let name = name.to_string_lossy();
        let name = if cfg!(windows) {
            name.to_ascii_uppercase()
        } else {
            name.into_owned()
        };
        // MSYS/Cygwin runtime options can start an external error debugger or
        // change pathname/argument interpretation before the fixed shell
        // command runs. Preserve the snapshot, but require separate authority
        // rather than silently stripping these settings.
        if cfg!(windows)
            && matches!(
                name.as_str(),
                "MSYS" | "CYGWIN" | "CYGWIN_DEBUG" | "MSYS2_ARG_CONV_EXCL" | "MSYS2_ENV_CONV_EXCL"
            )
        {
            return Err(GitToolError::InvalidInput(
                "Local Git transport inherits unbound MSYS/Cygwin runtime configuration; use a separately authorized invocation".into(),
            ));
        }
        if matches!(
            name.as_str(),
            "BASH_ENV"
                | "ENV"
                | "SHELLOPTS"
                | "BASHOPTS"
                | "ZDOTDIR"
                | "PS4"
                | "SSH_CLIENT"
                | "SSH2_CLIENT"
        ) || name.starts_with("BASH_FUNC_")
        {
            return Err(GitToolError::InvalidInput("Local Git transport inherits shell startup or function configuration whose effects are not bound; use a separately authorized invocation".into()));
        }
    }
    Ok(())
}

// Git invokes this shell as noninteractive `sh -c`, never as a login shell.
// Other compiled shells may load unconditional startup files (for example
// zshenv); their path/effect inventory requires separate support.
#[cfg(not(windows))]
fn validate_receiver_shell_path(path: &Path, canonical: &Path) -> Result<(), GitToolError> {
    let supported = (cfg!(target_os = "macos") && canonical == Path::new("/bin/sh"))
        || (cfg!(target_os = "linux")
            && ["/bin/dash", "/usr/bin/dash", "/bin/bash", "/usr/bin/bash"]
                .iter()
                .any(|candidate| canonical == Path::new(candidate)));
    if supported && path == Path::new("/bin/sh") {
        Ok(())
    } else {
        Err(GitToolError::InvalidInput(
            "Receiver transport requires the bound Unix /bin/sh baseline; other shells require separately authorized execution".into(),
        ))
    }
}

fn resolve_git(environment: &BTreeMap<OsString, OsString>) -> Result<PathBuf, GitToolError> {
    let path = environment
        .iter()
        .find_map(|(key, value)| {
            key.to_str()
                .is_some_and(|key| {
                    if cfg!(windows) {
                        key.eq_ignore_ascii_case("PATH")
                    } else {
                        key == "PATH"
                    }
                })
                .then_some(value)
        })
        .ok_or_else(|| GitToolError::Git("PATH is unavailable; cannot resolve host Git".into()))?;
    for directory in std::env::split_paths(path) {
        let candidate = directory.join(if cfg!(windows) { "git.exe" } else { "git" });
        if candidate.is_file() {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if !candidate
                    .metadata()
                    .is_ok_and(|m| m.permissions().mode() & 0o111 != 0)
                {
                    continue;
                }
            }
            return candidate
                .canonicalize()
                .map_err(|_| GitToolError::Git("host Git cannot be resolved".into()));
        }
    }
    Err(GitToolError::Git(
        "git not installed. Install git or use read-only tools.".into(),
    ))
}

struct ProgramEvidence {
    entry: snapshot::Entry,
    #[cfg(unix)]
    identity: (u64, u64, i64, i64),
    #[cfg(windows)]
    _guard: std::fs::File,
    #[cfg(not(any(unix, windows)))]
    modified: Option<std::time::SystemTime>,
}

impl PartialEq for ProgramEvidence {
    fn eq(&self, other: &Self) -> bool {
        if self.entry != other.entry {
            return false;
        }
        #[cfg(unix)]
        {
            self.identity == other.identity
        }
        #[cfg(windows)]
        {
            // Each evidence object holds its own non-write/non-delete shared
            // handle; equality compares the bytes captured through that handle.
            true
        }
        #[cfg(not(any(unix, windows)))]
        {
            self.modified == other.modified
        }
    }
}

impl Eq for ProgramEvidence {}

#[cfg(windows)]
fn capture_program(path: &Path) -> Result<ProgramEvidence, GitToolError> {
    use std::io::Read;
    use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_READ,
    };

    if path.canonicalize().ok().as_deref() != Some(path) {
        return Err(GitToolError::Git(
            "host Git program location changed after preparation".into(),
        ));
    }
    let invalid = || GitToolError::Git("host Git program is unavailable or changed".into());
    let before = std::fs::symlink_metadata(path).map_err(|_| invalid())?;
    if !before.is_file() || before.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(invalid());
    }
    let mut options = std::fs::OpenOptions::new();
    options
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
    let mut guard = options.open(path).map_err(|_| invalid())?;
    let opened = guard.metadata().map_err(|_| invalid())?;
    if !opened.is_file()
        || opened.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
        || opened.len() > 64 * 1024 * 1024
    {
        return Err(invalid());
    }
    let mut bytes = Vec::new();
    (&mut guard)
        .take(64 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| invalid())?;
    if bytes.len() > 64 * 1024 * 1024 {
        return Err(invalid());
    }
    let after = std::fs::symlink_metadata(path).map_err(|_| invalid())?;
    if !after.is_file()
        || after.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
        || before.len() != opened.len()
        || after.len() != opened.len()
        || before.creation_time() != opened.creation_time()
        || after.creation_time() != opened.creation_time()
        || before.last_write_time() != opened.last_write_time()
        || after.last_write_time() != opened.last_write_time()
    {
        return Err(invalid());
    }
    Ok(ProgramEvidence {
        entry: snapshot::Entry::File(opened.permissions(), bytes),
        _guard: guard,
    })
}

#[cfg(not(windows))]
fn capture_program(path: &Path) -> Result<ProgramEvidence, GitToolError> {
    if path.canonicalize().ok().as_deref() != Some(path) {
        return Err(GitToolError::Git(
            "host Git program location changed after preparation".into(),
        ));
    }
    let before = std::fs::symlink_metadata(path)
        .map_err(|_| GitToolError::Git("host Git program metadata is unavailable".into()))?;
    let entry = snapshot::Entry::capture_regular(
        path,
        &mut snapshot::Budget {
            entries: 1,
            bytes: 64 * 1024 * 1024,
        },
    )
    .map_err(|_| {
        GitToolError::Git("host Git program snapshot is unavailable or exceeds its limit".into())
    })?;
    if !matches!(entry, snapshot::Entry::File(..)) {
        return Err(GitToolError::Git(
            "host Git program is no longer a regular file".into(),
        ));
    }
    let after = std::fs::symlink_metadata(path)
        .map_err(|_| GitToolError::Git("host Git program metadata is unavailable".into()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let identity = |metadata: &std::fs::Metadata| {
            (
                metadata.dev(),
                metadata.ino(),
                metadata.ctime(),
                metadata.ctime_nsec(),
            )
        };
        if identity(&before) != identity(&after) {
            return Err(GitToolError::Git(
                "host Git changed during inspection".into(),
            ));
        }
        Ok(ProgramEvidence {
            entry,
            identity: identity(&after),
        })
    }
    #[cfg(not(any(unix, windows)))]
    {
        if before.modified().ok() != after.modified().ok() || before.len() != after.len() {
            return Err(GitToolError::Git(
                "host Git changed during inspection".into(),
            ));
        }
        Ok(ProgramEvidence {
            entry,
            modified: after.modified().ok(),
        })
    }
}

pub(super) fn path_facet(
    nature: ToolNature,
    path: &Path,
    description: &str,
) -> Result<ToolPermissionFacet, GitToolError> {
    let resource = path.to_str().ok_or_else(|| {
        GitToolError::InvalidInput("Git resource path cannot be represented losslessly".into())
    })?;
    Ok(
        ToolPermissionFacet::with_resource(nature, resource, ToolResourceKind::Path)
            .with_description(description),
    )
}

/// Local transport strips GIT_CONFIG_PARAMETERS, so sender `-c` options do not
/// reach receive-pack. Carry the accepted scheduling overrides in its fixed
/// command explicitly, quoting the independently bound Git executable.
fn receiver_command(program: &Path) -> Result<String, GitToolError> {
    #[cfg(windows)]
    let represented = host_path::receiver_program(program)?;
    #[cfg(windows)]
    let program = Path::new(&represented);
    let text = program
        .to_str()
        .filter(|text| program.is_absolute() && !text.contains(['\0', '\n', '\r']))
        .ok_or_else(|| {
            GitToolError::InvalidInput(
                "Receiver Git executable cannot be represented in a fixed command".into(),
            )
        })?;
    let quoted = format!("'{}'", text.replace('\'', "'\\''"));
    Ok(format!(
        "{quoted} -c maintenance.autoDetach=false -c gc.autoDetach=false receive-pack"
    ))
}

#[async_trait]
impl PreparedNormalToolInvocation for AddPlan {
    #[cfg(any(unix, windows))]
    fn cleanup_receipt(&self) -> Option<std::sync::Arc<dyn talos_core::tool::ToolCleanupReceipt>> {
        Some(std::sync::Arc::new(self.execution.cleanup_receipt()))
    }

    fn plan_identity(&self) -> Option<&PreparedPlanIdentity> {
        Some(&self.identity)
    }

    fn prepared_command_evidence(&self) -> Option<talos_core::tool::PreparedCommandEvidence> {
        use talos_core::tool::{
            PreparedCommandDescription, PreparedCommandEffect as Effect, PreparedCommandEvidence,
            PreparedCommandTargets,
        };
        let Operation::Pull(input) = &self.operation else {
            return None;
        };
        if self.phase != Phase::PullCommandMutation || self.native_pull_command.is_none() {
            return None;
        }
        let mut arguments = [
            "-c",
            "maintenance.autoDetach=false",
            "-c",
            "gc.autoDetach=false",
            "--no-pager",
            "pull",
        ]
        .map(str::to_owned)
        .to_vec();
        if input.remote.is_some() {
            arguments.extend(["--".into(), "<retained-remote>".into()]);
        }
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
                operation: "git:pull-command/v1".into(),
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
                    remote_selection: if input.remote.is_some() {
                        "explicit retained remote; name/location withheld"
                    } else {
                        "configured upstream"
                    }
                    .into(),
                    configured_remote_hosts: config.pull_command_remote_hosts(),
                    invocation_remote_host: input.remote.as_deref().and_then(|remote| {
                        super::config_state::command_remote_host(remote.as_bytes())
                    }),
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

    fn permission_profile(&self) -> &[ToolPermissionFacet] {
        &self.facets
    }

    async fn execute_stage(
        mut self: Box<Self>,
        admission: Box<dyn std::any::Any + Send>,
    ) -> PreparedNormalStage {
        if self.phase == Phase::Mutation
            && (self.child_configs.is_empty() || self.child_attributes_discovered)
        {
            return PreparedNormalStage::Complete(self.execute(admission).await);
        }
        let Ok(proof) = admission.downcast::<AdmittedInvocation>() else {
            return PreparedNormalStage::Complete(ToolExecutionOutput::error(
                "Git discovery requires its admitted proof",
            ));
        };
        let Ok(authorizations) = proof.consume_for_prepared(&self.request(), &self.identity) else {
            return PreparedNormalStage::Complete(ToolExecutionOutput::error(
                "Git discovery proof does not authorize this plan",
            ));
        };
        let result = match self.phase {
            Phase::PullCommandMutation => {
                return PreparedNormalStage::Complete(match self.execute_pull_command().await {
                    Ok(content) => ToolExecutionOutput::success(content),
                    Err(error) => ToolExecutionOutput::error(error.to_string()),
                });
            }
            Phase::CheckoutNewGitlinks => self.discover_new_gitlinks(&authorizations),
            Phase::CheckoutChildMarkers => self.discover_checkout_child_markers(&authorizations),
            Phase::CheckoutChildMetadata => self.discover_checkout_child_metadata(&authorizations),
            Phase::CheckoutChildConfiguration => {
                self.discover_checkout_child_configuration(&authorizations)
                    .await
            }
            Phase::CheckoutChildWorktree => {
                self.discover_checkout_child_worktree(&authorizations).await
            }
            Phase::CheckoutDeletion => self.discover_checkout_deletion(&authorizations),
            Phase::TargetConfiguration => self
                .discover_target_configuration(&authorizations)
                .map_err(|error| {
                    GitToolError::InvalidInput(format!("target-branch discovery: {error}"))
                }),
            Phase::SenderConfiguration if matches!(self.operation, Operation::Pull(_)) => {
                self.discover_pull_command().await
            }
            Phase::SenderConfiguration => self.discover_sender(&authorizations).await,
            Phase::ReceiverRoots => {
                let plan = if cfg!(windows) {
                    self.rebind_push_program(&authorizations).await
                } else {
                    Ok(self)
                };
                plan.and_then(|plan| plan.discover_receiver_roots(&authorizations))
            }
            Phase::PushProgram => match self.rebind_push_program(&authorizations).await {
                Ok(mut plan) => {
                    plan.facets.append(&mut plan.sender_mutation_facets);
                    plan.phase = Phase::Mutation;
                    plan.identity = PreparedPlanIdentity::new();
                    Ok(plan)
                }
                Err(error) => Err(error),
            },
            Phase::ReceiverMetadata => self.discover_receiver_metadata(&authorizations),
            Phase::ReceiverCommon => self.discover_receiver_common(&authorizations),
            Phase::ReceiverConfiguration => {
                self.discover_receiver_configuration(&authorizations).await
            }
            Phase::ReceiverAuxiliary => self.discover_receiver_auxiliary(&authorizations),
            Phase::ReceiverMutation => self.prepare_receiver_mutation(&authorizations),
            Phase::Mutation => {
                let previous = self.identity.clone();
                match self.discover_child_attribute_targets(&authorizations) {
                    Ok(mut plan) if plan.identity.same_plan(&previous) => {
                        // No new source frontier: reuse this admission rather
                        // than spending another stage on already-bound rules.
                        return PreparedNormalStage::Complete(
                            match plan.execute_admitted(&authorizations).await {
                                Ok(content) => ToolExecutionOutput::success(content),
                                Err(error) => ToolExecutionOutput::error(error.to_string()),
                            },
                        );
                    }
                    result => result,
                }
            }
        };
        match result {
            Ok(plan) => PreparedNormalStage::Continue(plan),
            Err(error) => {
                PreparedNormalStage::Complete(ToolExecutionOutput::error(error.to_string()))
            }
        }
    }

    async fn execute(
        mut self: Box<Self>,
        admission: Box<dyn std::any::Any + Send>,
    ) -> ToolExecutionOutput {
        let Ok(proof) = admission.downcast::<AdmittedInvocation>() else {
            return ToolExecutionOutput::error(
                "Git mutation requires an admitted invocation proof",
            );
        };
        let Ok(authorizations) = proof.consume_for_prepared(&self.request(), &self.identity) else {
            return ToolExecutionOutput::error("Git invocation proof does not authorize this plan");
        };
        match self.execute_admitted(&authorizations).await {
            Ok(content) => ToolExecutionOutput::success(content),
            Err(error) => ToolExecutionOutput::error(error.to_string()),
        }
    }
}

#[cfg(test)]
#[path = "prepared_add_tests.rs"]
pub(super) mod tests;
