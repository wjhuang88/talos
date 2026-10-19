//! Native configuration evidence, checked against the admitted host program.

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

use super::{GitToolError, repository_state::RepositoryState, snapshot};

type CheckoutCandidates = (Vec<(String, String)>, Option<String>);

type Values = BTreeMap<Vec<u8>, Vec<Option<Vec<u8>>>>;

/// Receiver dependencies named by configuration. Construction never opens the
/// paths; their contents belong to the next independently admitted Read stage.
pub(super) struct ReceiverEffects {
    pub paths: BTreeSet<PathBuf>,
    pub hooks: BTreeSet<PathBuf>,
    alternates: BTreeSet<PathBuf>,
}

impl ReceiverEffects {
    /// Called only after every path has fresh Read admission. Unsupported hooks
    /// are refused before any program is started; no hook is silently disabled.
    pub(super) fn capture(
        &self,
    ) -> Result<Vec<(PathBuf, snapshot::RegularEvidence)>, GitToolError> {
        let mut budget = snapshot::Budget {
            entries: 4096,
            bytes: 4 * 1024 * 1024,
        };
        self.paths
            .iter()
            .map(|path| {
                let evidence =
                    snapshot::RegularEvidence::capture(path, &mut budget).map_err(|_| {
                        unsupported("Receiver auxiliary evidence must be a bounded regular file")
                    })?;
                if self.hooks.contains(path)
                    && let snapshot::Entry::File(permissions, _) = &evidence.entry
                {
                    #[cfg(unix)]
                    let executable = {
                        use std::os::unix::fs::PermissionsExt;
                        permissions.mode() & 0o111 != 0
                    };
                    #[cfg(not(unix))]
                    let executable = {
                        let _ = permissions;
                        true
                    };
                    if executable {
                        return Err(unsupported(
                            "Executable receiver hook effects are not supported by this plan",
                        ));
                    }
                }
                if self.alternates.contains(path)
                    && let snapshot::Entry::File(_, bytes) = &evidence.entry
                    && !bytes.is_empty()
                {
                    return Err(unsupported(
                        "Receiver object alternates require independent object-store discovery",
                    ));
                }
                Ok((path.clone(), evidence))
            })
            .collect()
    }
}

/// Child configuration evidence captured only by admitted configuration-chain
/// discovery. Auxiliary reads and child status remain separately gated.
#[derive(PartialEq, Eq)]
pub(super) struct ChildConfig {
    values: Values,
    ordered_output: Vec<u8>,
    origins: BTreeSet<PathBuf>,
    pub files: Vec<(PathBuf, snapshot::Entry)>,
    identities: Vec<snapshot::PathIdentity>,
}

/// Startup layout sources are distinct from the include-expanded config chain.
/// Capture requires admission of both fixed source paths; targets stay unopened.
#[derive(PartialEq, Eq)]
pub(super) struct ChildStartupLayout {
    pub files: Vec<(PathBuf, snapshot::RegularEvidence)>,
    pub targets: BTreeSet<PathBuf>,
}

impl ChildStartupLayout {
    pub(super) fn capture(
        gitdir: &std::path::Path,
        common: &std::path::Path,
        has_common: bool,
    ) -> Result<Self, GitToolError> {
        let mut budget = snapshot::Budget {
            entries: 2,
            bytes: 256 * 1024,
        };
        let mut files = Vec::new();
        let mut targets = BTreeSet::new();
        let common_config = common.join("config");
        let evidence =
            snapshot::RegularEvidence::capture(&common_config, &mut budget).map_err(|_| {
                unsupported("Child startup configuration must be bounded regular files")
            })?;
        let mut layout = startup_values(&evidence.entry)?;
        files.push((common_config, evidence));
        // Git startup ignores all layout settings when repositoryFormatVersion
        // is absent; includes do not participate in this decision.
        if layout.version {
            if layout.worktree_config {
                let path = gitdir.join("config.worktree");
                let evidence =
                    snapshot::RegularEvidence::capture(&path, &mut budget).map_err(|_| {
                        unsupported("Child worktree configuration must be a bounded regular file")
                    })?;
                let worktree = startup_values(&evidence.entry)?;
                if worktree.bare.is_some() {
                    layout.bare = worktree.bare;
                }
                if worktree.worktree.is_some() {
                    layout.worktree = worktree.worktree;
                }
                files.push((path, evidence));
            }
            // get_common_dir suppresses common-file layout hints for linked
            // worktrees, unless extensions.worktreeConfig enables private ones.
            if (!has_common || layout.worktree_config) && layout.bare != Some(true) {
                targets.extend(layout.worktree.map(|path| gitdir.join(path)));
            }
        }
        Ok(Self { files, targets })
    }

    pub(super) fn revalidate(&self) -> Result<(), GitToolError> {
        let mut budget = snapshot::Budget {
            entries: 2,
            bytes: 256 * 1024,
        };
        for (path, expected) in &self.files {
            if &snapshot::RegularEvidence::capture(path, &mut budget)
                .map_err(|_| unsupported("Child startup configuration unavailable"))?
                != expected
            {
                return Err(unsupported("Child startup configuration changed"));
            }
        }
        Ok(())
    }
}

#[derive(Default)]
struct StartupValues {
    version: bool,
    worktree_config: bool,
    bare: Option<bool>,
    worktree: Option<PathBuf>,
}

fn startup_values(entry: &snapshot::Entry) -> Result<StartupValues, GitToolError> {
    let snapshot::Entry::File(_, bytes) = entry else {
        return Ok(StartupValues::default());
    };
    std::panic::catch_unwind(|| {
        let file = gix::config::File::from_bytes_no_includes(
            bytes,
            gix::config::file::Metadata::default(),
            Default::default(),
        )
        .map_err(|_| unsupported("Invalid child startup configuration"))?;
        let worktree = match file.raw_value("core.worktree") {
            Ok(value) => {
                let bytes: &[u8] = value.as_ref();
                let text = std::str::from_utf8(bytes)
                    .ok()
                    .filter(|text| !text.is_empty() && !text.contains(['\0', '\n', '\r']))
                    .ok_or_else(|| unsupported("Invalid child startup worktree"))?;
                Some(PathBuf::from(text))
            }
            Err(_) => None,
        };
        Ok(StartupValues {
            version: file
                .integer("core.repositoryFormatVersion")
                .map_err(|_| unsupported("Invalid child repository format version"))?
                .is_some_and(|version| version >= 0),
            worktree_config: file
                .boolean("extensions.worktreeConfig")
                .map_err(|_| unsupported("Invalid child worktreeConfig"))?
                .unwrap_or(false),
            bare: file
                .boolean("core.bare")
                .map_err(|_| unsupported("Invalid child core.bare"))?,
            worktree,
        })
    })
    .unwrap_or_else(|_| Err(unsupported("Child startup configuration parser failed")))
}

/// Strip origin fields while retaining ordered, possibly valueless records.
fn ordered_records(output: &[u8]) -> Result<Vec<u8>, GitToolError> {
    if output.is_empty() {
        return Ok(Vec::new());
    }
    let fields: Vec<_> = output
        .strip_suffix(&[0])
        .ok_or_else(|| unsupported("Incomplete Pull configuration"))?
        .split(|byte| *byte == 0)
        .collect();
    let (pairs, tail) = fields.as_chunks::<2>();
    if !tail.is_empty() {
        return Err(unsupported("Incomplete Pull configuration"));
    }
    let mut records = Vec::new();
    for [_, record] in pairs {
        records.extend_from_slice(record);
        records.push(0);
    }
    Ok(records)
}

impl ChildConfig {
    pub(super) fn pull_hook_paths(
        &self,
        common: &std::path::Path,
        execution_dir: &std::path::Path,
        environment: &BTreeMap<OsString, OsString>,
    ) -> Result<Vec<PathBuf>, GitToolError> {
        fetch_hook_paths(&self.values, common, execution_dir, environment)
    }

    /// The first recursive fetch uses ordinary fetch default-remote semantics.
    pub(super) fn fetch_remote(&self, branch: Option<&str>) -> Result<String, GitToolError> {
        super::pull_state::fetch_remote(&ordered_records(&self.ordered_output)?, branch, None)
    }

    pub(super) fn prepare_pull_transport(&self, remote: &str) -> Result<(), GitToolError> {
        validate_fetch_external_effects(&self.values)?;
        for key in [
            format!("remote.{remote}.uploadpack"),
            format!("remote.{remote}.vcs"),
        ] {
            if self.values.contains_key(key.as_bytes()) {
                return Err(unsupported(
                    "Configured Pull child transport helper requires separate execution authorization",
                ));
            }
        }
        Ok(())
    }

    /// Resolve fetch destinations using Git's ordered URL and rewrite rules.
    pub(super) fn fetch_destinations(&self, remote: &str) -> Result<Vec<String>, GitToolError> {
        super::remote_state::destinations(
            &super::remote_state::records(&self.ordered_output)?,
            remote,
            false,
        )
    }

    pub(super) fn receiver_effects(
        &self,
        gitdir: &std::path::Path,
        common: &std::path::Path,
        environment: &BTreeMap<OsString, OsString>,
    ) -> Result<ReceiverEffects, GitToolError> {
        if self
            .values
            .get(b"receive.denycurrentbranch".as_slice())
            .and_then(|values| values.last())
            .and_then(|value| value.as_deref())
            .is_some_and(|value| value.eq_ignore_ascii_case(b"updateInstead"))
        {
            return Err(unsupported(
                "Receiver updateInstead can mutate a checked-out sibling worktree and run filters; its effects are not bound",
            ));
        }
        if self
            .values
            .keys()
            .any(|key| key.starts_with(b"hook.") && key.ends_with(b".command"))
        {
            return Err(unsupported(
                "Configured receiver hook commands require separate execution authorization",
            ));
        }
        let enabled = |key: &[u8]| -> Result<bool, GitToolError> {
            self.values
                .get(key)
                .and_then(|values| values.last())
                .map(|value| git_boolean(value.as_deref()))
                .transpose()
                .map(|value| value.unwrap_or(true))
        };
        if self
            .values
            .contains_key(b"core.alternaterefscommand".as_slice())
        {
            return Err(unsupported(
                "Receiver alternateRefsCommand can execute an unbound shell command",
            ));
        }
        if enabled(b"receive.autogc")?
            && enabled(b"maintenance.auto")?
            && self.values.contains_key(b"gc.recentobjectshook".as_slice())
        {
            return Err(unsupported(
                "Receiver maintenance recentObjectsHook requires separate execution authorization",
            ));
        }
        if enabled(b"receive.autogc")?
            && enabled(b"maintenance.auto")?
            && self
                .values
                .get(b"gc.repackfilterto".as_slice())
                .and_then(|values| values.last())
                .and_then(|value| value.as_deref())
                .is_some_and(|value| !value.is_empty())
        {
            return Err(unsupported(
                "Receiver maintenance can write gc.repackFilterTo outside its object store",
            ));
        }
        let directory = configured_path(&self.values, b"core.hookspath", environment)?
            .unwrap_or_else(|| common.join("hooks"));
        // Push-triggered hooks run from GIT_DIR even for non-bare receivers.
        let directory = gitdir.join(directory);
        let hooks: BTreeSet<_> = [
            "pre-receive",
            "update",
            "post-receive",
            "post-update",
            "proc-receive",
            "push-to-checkout",
            "reference-transaction",
            "pre-auto-gc",
        ]
        .into_iter()
        .map(|name| directory.join(name))
        .collect();
        let mut paths = hooks.clone();
        if let Some(skiplist) =
            configured_path(&self.values, b"receive.fsck.skiplist", environment)?
        {
            paths.insert(gitdir.join(skiplist));
        }
        let alternates = BTreeSet::from([
            common.join("objects/info/alternates"),
            common.join("objects/info/http-alternates"),
        ]);
        paths.extend(alternates.iter().cloned());
        paths.insert(common.join("shallow"));
        Ok(ReceiverEffects {
            paths,
            hooks,
            alternates,
        })
    }

    /// Receiver setup can change cwd. Refuse ambiguous relative origins before
    /// interpreting or reading source paths using the caller's initial cwd.
    pub(super) fn capture_receiver(
        output: &[u8],
        gitdir: &std::path::Path,
    ) -> Result<Self, GitToolError> {
        let (_, origins) = parse_origins(output)?;
        if origins.iter().any(|path| !path.is_absolute()) {
            return Err(unsupported(
                "Receiver configuration origin must be absolute",
            ));
        }
        Self::capture(output, gitdir)
    }

    pub(super) fn attribute_paths(
        &self,
        workdir: &std::path::Path,
        common_dir: &std::path::Path,
        environment: &BTreeMap<OsString, OsString>,
    ) -> Result<BTreeSet<PathBuf>, GitToolError> {
        let mut paths = BTreeSet::from([
            common_dir.join("info/attributes"),
            common_dir.join("info/exclude"),
        ]);
        if let Some(path) = configured_path(&self.values, b"core.attributesfile", environment)? {
            paths.insert(if path.is_absolute() {
                path
            } else {
                workdir.join(path)
            });
        }
        let excludes =
            configured_path(&self.values, b"core.excludesfile", environment)?.or_else(|| {
                environment
                    .get(OsStr::new("XDG_CONFIG_HOME"))
                    .filter(|value| !value.is_empty())
                    .map(PathBuf::from)
                    .or_else(|| {
                        environment
                            .get(OsStr::new("HOME"))
                            .map(|home| PathBuf::from(home).join(".config"))
                    })
                    .map(|xdg| xdg.join("git/ignore"))
            });
        if let Some(path) = excludes {
            paths.insert(if path.is_absolute() {
                path
            } else {
                workdir.join(path)
            });
        }
        Ok(paths)
    }

    pub(super) fn capture(output: &[u8], workdir: &std::path::Path) -> Result<Self, GitToolError> {
        let (values, origins) = parse_origins(output)?;
        refuse_promisor_objects(&values)?;
        if let Some(value) = values
            .get(b"core.fsmonitor".as_slice())
            .and_then(|values| values.last())
            && git_boolean(value.as_deref()).unwrap_or(true)
        {
            return Err(unsupported(
                "Configured submodule fsmonitor execution is not supported by this plan",
            ));
        }
        let mut budget = snapshot::Budget {
            entries: 4096,
            bytes: 4 * 1024 * 1024,
        };
        let mut identities = Vec::new();
        let files = origins
            .iter()
            .map(|path| {
                let path = if path.is_absolute() {
                    path.clone()
                } else {
                    workdir.join(path)
                };
                let identity = snapshot::PathIdentity::capture(&path)
                    .map_err(|_| unsupported("Submodule configuration identity unavailable"))?;
                let entry = snapshot::Entry::capture_regular(&path, &mut budget).map_err(|_| {
                    unsupported("Submodule configuration source cannot be captured")
                })?;
                if !matches!(entry, snapshot::Entry::File(..) | snapshot::Entry::Missing) {
                    return Err(unsupported(
                        "Submodule configuration sources must be regular files",
                    ));
                }
                if snapshot::PathIdentity::capture(&path).ok().as_ref() != Some(&identity) {
                    return Err(unsupported(
                        "Submodule configuration identity changed during capture",
                    ));
                }
                identities.push(identity);
                Ok((path, entry))
            })
            .collect::<Result<Vec<_>, GitToolError>>()?;
        Ok(Self {
            values,
            ordered_output: output.to_vec(),
            origins,
            files,
            identities,
        })
    }

    pub(super) fn verify_host(&self, output: &[u8]) -> Result<(), GitToolError> {
        let (values, origins) = parse_origins(output)?;
        if values != self.values || origins != self.origins || output != self.ordered_output {
            return Err(unsupported(
                "Submodule configuration changed during discovery",
            ));
        }
        Ok(())
    }

    pub(super) fn revalidate(&self) -> Result<(), GitToolError> {
        let mut budget = snapshot::Budget {
            entries: 4096,
            bytes: 4 * 1024 * 1024,
        };
        for ((path, expected), identity) in self.files.iter().zip(&self.identities) {
            if snapshot::PathIdentity::capture(path).ok().as_ref() != Some(identity) {
                return Err(unsupported("Submodule configuration path identity changed"));
            }
            let actual = snapshot::Entry::capture_regular(path, &mut budget)
                .map_err(|_| unsupported("Submodule configuration evidence unavailable"))?;
            if &actual != expected {
                return Err(unsupported(
                    "Submodule configuration changed after discovery",
                ));
            }
            if snapshot::PathIdentity::capture(path).ok().as_ref() != Some(identity) {
                return Err(unsupported("Submodule configuration path identity changed"));
            }
        }
        Ok(())
    }
}

#[derive(PartialEq, Eq)]
pub(super) struct ConfigState {
    values: Values,
    ordered_output: Vec<u8>,
    origins: BTreeSet<PathBuf>,
    pub files: Vec<(PathBuf, snapshot::Entry)>,
    identities: Vec<snapshot::PathIdentity>,
    pub auxiliary: BTreeSet<PathBuf>,
    /// Effective sources outside the target tree's ordinary attribute rules.
    pub stable_auxiliary: BTreeSet<PathBuf>,
    hooks: Vec<PathBuf>,
}

fn unsupported(reason: &str) -> GitToolError {
    GitToolError::InvalidInput(format!(
        "{reason}; use a separately authorized Git invocation"
    ))
}

// Git applies both recursion keys to one variable in declaration order.
fn push_recursion_enabled(output: &[u8]) -> Result<bool, GitToolError> {
    if output.is_empty() {
        return Ok(false);
    }
    let fields: Vec<_> = output
        .strip_suffix(&[0])
        .ok_or_else(|| unsupported("Incomplete push configuration"))?
        .split(|byte| *byte == 0)
        .collect();
    let (pairs, tail) = fields.as_chunks::<2>();
    if !tail.is_empty() {
        return Err(unsupported("Incomplete push configuration"));
    }
    let mut enabled = false;
    for [_, record] in pairs {
        let (key, value) = match record.iter().position(|byte| *byte == b'\n') {
            Some(split) => (&record[..split], Some(&record[split + 1..])),
            None => (*record, None),
        };
        if key.eq_ignore_ascii_case(b"submodule.recurse") {
            enabled = git_boolean(value)?;
        } else if key.eq_ignore_ascii_case(b"push.recursesubmodules") {
            enabled = match value {
                Some(b"no") => false,
                Some(b"check" | b"on-demand" | b"only") => true,
                _ => {
                    if git_boolean(value)? {
                        return Err(unsupported(
                            "Git push recursion requires a named mode, not boolean true",
                        ));
                    }
                    false
                }
            };
        }
    }
    Ok(enabled)
}

// Canonicalize the existing prefix so a hooksPath symlink or `..` spelling
// cannot hide its overlap with a target worktree entry. Missing suffixes are
// retained as paths only; no content is read through them.
fn resolve_optional_path(path: &std::path::Path) -> Result<PathBuf, GitToolError> {
    let mut prefix = path.to_path_buf();
    let mut suffix = Vec::new();
    loop {
        match prefix.canonicalize() {
            Ok(mut resolved) => {
                for part in suffix.iter().rev() {
                    resolved.push(part);
                }
                return Ok(resolved);
            }
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
                ) =>
            {
                let name = prefix
                    .file_name()
                    .ok_or_else(|| unsupported("Checkout auxiliary path cannot be resolved"))?
                    .to_os_string();
                suffix.push(name);
                if !prefix.pop() {
                    return Err(unsupported("Checkout auxiliary path cannot be resolved"));
                }
            }
            Err(_) => return Err(unsupported("Checkout auxiliary path cannot be resolved")),
        }
    }
}

// Target entries may not exist yet, so canonicalize alone cannot establish
// equality on case-insensitive or normalization-insensitive volumes. Treat
// ambiguous non-ASCII components conservatively rather than assuming byte
// equality proves disjoint effect scope.
fn potential_path_prefix(prefix: &std::path::Path, path: &std::path::Path) -> bool {
    let mut parts = path.components();
    prefix.components().all(|left| {
        let Some(right) = parts.next() else {
            return false;
        };
        let left = left.as_os_str().as_encoded_bytes();
        let right = right.as_os_str().as_encoded_bytes();
        left.eq_ignore_ascii_case(right) || !left.is_ascii() || !right.is_ascii()
    })
}

fn lexical_path(path: &std::path::Path) -> PathBuf {
    let mut result = PathBuf::new();
    for part in path.components() {
        match part {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                result.pop();
            }
            _ => result.push(part.as_os_str()),
        }
    }
    result
}

fn auxiliary_spellings(path: &std::path::Path) -> Result<BTreeSet<PathBuf>, GitToolError> {
    let mut current = path.to_path_buf();
    let mut spellings = BTreeSet::new();
    // Preserve every symlink hop: an external alias may lead through a tracked
    // worktree alias and back outside. Final canonicalization loses that hop.
    for _ in 0..64 {
        spellings.insert(lexical_path(&current));
        spellings.insert(resolve_optional_path(&current)?);
        let components: Vec<_> = current.components().collect();
        let mut prefix = PathBuf::new();
        let mut next = None;
        for (position, component) in components.iter().enumerate() {
            prefix.push(component.as_os_str());
            // Even an ancestor later followed by `..` is a real traversal
            // dependency; replacing it can turn a valid lookup into ENOTDIR.
            spellings.insert(lexical_path(&prefix));
            match std::fs::symlink_metadata(&prefix) {
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    let target = std::fs::read_link(&prefix)
                        .map_err(|_| unsupported("Checkout auxiliary link unavailable"))?;
                    let mut resolved = if target.is_absolute() {
                        target
                    } else {
                        prefix
                            .parent()
                            .ok_or_else(|| unsupported("Invalid auxiliary link"))?
                            .join(target)
                    };
                    for remaining in &components[position + 1..] {
                        resolved.push(remaining.as_os_str());
                    }
                    next = Some(resolved);
                    break;
                }
                Ok(_) => {}
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
                    ) =>
                {
                    break;
                }
                Err(_) => return Err(unsupported("Checkout auxiliary path unavailable")),
            }
        }
        match next {
            Some(path) => current = path,
            None => return Ok(spellings),
        }
    }
    Err(unsupported(
        "Checkout auxiliary symlink chain exceeds limit",
    ))
}

impl ConfigState {
    /// Predict tracking values from the invocation's retained configuration.
    /// The caller supplies a resolved branch reference, never a raw revision.
    /// These records describe writes to local config, not effective overrides.
    pub(super) fn checkout_tracking_writes(
        &self,
        name: &str,
        start: Option<&str>,
    ) -> Result<Vec<super::target_config::Record>, GitToolError> {
        let last = |key: &[u8]| self.values.get(key).and_then(|values| values.last());
        let mode = match last(b"branch.autosetupmerge") {
            None => b"true".as_slice(),
            Some(Some(value)) if matches!(value.as_slice(), b"always" | b"inherit" | b"simple") => {
                value
            }
            Some(value) if git_boolean(value.as_deref())? => b"true",
            Some(_) => b"false",
        };
        let rebase = match last(b"branch.autosetuprebase") {
            None => b"never".as_slice(),
            Some(Some(value))
                if matches!(
                    value.as_slice(),
                    b"never" | b"local" | b"remote" | b"always"
                ) =>
            {
                value
            }
            _ => return Err(unsupported("Invalid checkout autoSetupRebase")),
        };
        let Some(start) = start.filter(|_| mode != b"false") else {
            return Ok(Vec::new());
        };
        let (origin, merges) = if mode == b"inherit" {
            let branch = start.strip_prefix("refs/heads/").unwrap_or(start);
            let remote_key = format!("branch.{branch}.remote");
            let merge_key = format!("branch.{branch}.merge");
            let Some(Some(remote)) = last(remote_key.as_bytes()) else {
                return Ok(Vec::new());
            };
            let merges = self
                .values
                .get(merge_key.as_bytes())
                .cloned()
                .unwrap_or_default();
            if merges.is_empty() || merges.iter().any(Option::is_none) {
                return Ok(Vec::new());
            }
            (
                Some(remote.clone()),
                merges.into_iter().flatten().collect::<Vec<_>>(),
            )
        } else {
            let mut matches = Vec::new();
            for (key, values) in &self.values {
                let Some(remote) = key
                    .strip_prefix(b"remote.")
                    .and_then(|key| key.strip_suffix(b".fetch"))
                else {
                    continue;
                };
                for value in values {
                    let bytes = value
                        .as_deref()
                        .ok_or_else(|| unsupported("Invalid checkout fetch mapping"))?;
                    let spec = std::panic::catch_unwind(|| {
                        gix::refspec::parse(
                            gix::bstr::BStr::new(bytes),
                            gix::refspec::parse::Operation::Fetch,
                        )
                    })
                    .map_err(|_| unsupported("Invalid checkout fetch mapping"))?
                    .map_err(|_| unsupported("Invalid checkout fetch mapping"))?;
                    // Native remote_find_tracking ignores negative fetch rules
                    // and uses the first positive mapping for each remote.
                    if bytes.starts_with(b"^") {
                        continue;
                    }
                    let (Some(from), Some(to)) = (spec.source(), spec.destination()) else {
                        continue;
                    };
                    let from = std::str::from_utf8(from.as_ref())
                        .map_err(|_| unsupported("Non-UTF8 checkout fetch mapping"))?;
                    let to = std::str::from_utf8(to.as_ref())
                        .map_err(|_| unsupported("Non-UTF8 checkout fetch mapping"))?;
                    let source = if let Some((prefix, suffix)) = to.split_once('*') {
                        start
                            .strip_prefix(prefix)
                            .and_then(|middle| middle.strip_suffix(suffix))
                            .map(|middle| from.replace('*', middle))
                    } else if to == start {
                        Some(from.to_owned())
                    } else {
                        None
                    };
                    if let Some(source) = source {
                        matches.push((remote.to_vec(), source.into_bytes()));
                        break;
                    }
                }
            }
            if matches.len() > 1 {
                return Err(unsupported("Ambiguous checkout tracking mappings"));
            }
            if let Some((remote, source)) = matches.pop() {
                if mode == b"simple" && source.strip_prefix(b"refs/heads/") != Some(name.as_bytes())
                {
                    return Ok(Vec::new());
                }
                (Some(remote), vec![source])
            } else if mode == b"always" {
                (None, vec![start.as_bytes().to_vec()])
            } else {
                return Ok(Vec::new());
            }
        };
        if origin.is_none()
            && merges
                .iter()
                .any(|reference| reference.strip_prefix(b"refs/heads/") == Some(name.as_bytes()))
        {
            return Ok(Vec::new());
        }
        let rebasing = rebase == b"always"
            || (rebase == b"local" && origin.is_none())
            || (rebase == b"remote" && origin.is_some());
        if rebasing && merges.len() > 1 {
            return Err(unsupported(
                "Checkout cannot inherit multiple upstreams with auto rebase",
            ));
        }
        let record = |suffix: &str, value| super::target_config::Record {
            key: format!("branch.{name}.{suffix}").into_bytes(),
            value: Some(value),
        };
        let mut writes = vec![record("remote", origin.unwrap_or_else(|| b".".to_vec()))];
        writes.extend(merges.into_iter().map(|value| record("merge", value)));
        if rebasing {
            writes.push(record("rebase", b"true".to_vec()));
        }
        Ok(writes)
    }

    /// Prevent read-only Pull graph probes from triggering lazy network fetches.
    pub(super) fn prepare_pull_inspection(&self) -> Result<(), GitToolError> {
        refuse_promisor_objects(&self.values)
    }

    pub(super) fn pull_diff_order_file(
        &self,
        workdir: &std::path::Path,
        environment: &BTreeMap<OsString, OsString>,
    ) -> Result<Option<PathBuf>, GitToolError> {
        Ok(
            configured_path(&self.values, b"diff.orderfile", environment)?.map(|path| {
                if path.is_absolute() {
                    path
                } else {
                    workdir.join(path)
                }
            }),
        )
    }
    /// Preserve declaration order and valueless records for Pull policy parsing.
    pub(super) fn pull_records(&self) -> Result<Vec<u8>, GitToolError> {
        ordered_records(&self.ordered_output)
    }

    pub(super) fn pull_remote_records(&self) -> Result<Vec<(String, String)>, GitToolError> {
        super::remote_state::records(&self.ordered_output)
    }
    pub fn capture_host(
        output: &[u8],
        repository: &RepositoryState,
        environment: &BTreeMap<OsString, OsString>,
    ) -> Result<Self, GitToolError> {
        let (values, paths) = parse_origins(output)?;
        if values
            .keys()
            .any(|key| key.starts_with(b"hook.") && key.ends_with(b".command"))
        {
            return Err(unsupported(
                "Configured Git hook commands require separate execution authorization",
            ));
        }
        let mut auxiliary = BTreeSet::new();
        auxiliary.extend(declared_include_paths(
            output,
            &repository.workdir,
            environment,
        )?);
        auxiliary.insert(repository.common_dir.join("config"));
        auxiliary.insert(repository.git_dir.join("config.worktree"));
        let origins = paths.clone();
        // External filters and fsmonitor introduce execution not represented by
        // source/index facets. Do not disable them or stage unfiltered bytes.
        for (key, entries) in &values {
            let key = key.as_slice();
            if key == b"core.fsmonitor" {
                let value = entries
                    .last()
                    .ok_or_else(|| unsupported("Git fsmonitor configuration is empty"))?;
                if git_boolean(value.as_deref()).unwrap_or(true) {
                    return Err(unsupported(
                        "Configured Git fsmonitor execution is not supported by this plan",
                    ));
                }
            }
        }
        for key in [b"core.attributesfile".as_slice(), b"core.excludesfile"] {
            if let Some(path) = configured_path(&values, key, environment)? {
                auxiliary.insert(if path.is_absolute() {
                    path
                } else {
                    repository.workdir.join(path)
                });
            }
        }
        // Git uses these defaults even without an explicit core.*File value.
        let xdg = environment
            .get(OsStr::new("XDG_CONFIG_HOME"))
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .or_else(|| {
                environment
                    .get(OsStr::new("HOME"))
                    .map(|home| PathBuf::from(home).join(".config"))
            });
        if let Some(xdg) = xdg {
            if !values.contains_key(b"core.attributesfile".as_slice()) {
                auxiliary.insert(xdg.join("git/attributes"));
            }
            if !values.contains_key(b"core.excludesfile".as_slice()) {
                auxiliary.insert(xdg.join("git/ignore"));
            }
        }
        let hooks = configured_path(&values, b"core.hookspath", environment)?
            .unwrap_or_else(|| repository.common_dir.join("hooks"));
        let hooks = if hooks.is_absolute() {
            hooks
        } else {
            repository.workdir.join(hooks)
        };
        let hook = hooks.join("post-index-change");
        auxiliary.insert(hook.clone());
        auxiliary.insert(repository.common_dir.join("info/attributes"));
        auxiliary.insert(repository.common_dir.join("info/exclude"));
        let mut budget = snapshot::Budget {
            entries: 4096,
            bytes: 4 * 1024 * 1024,
        };
        let mut identities = Vec::new();
        let files = paths
            .into_iter()
            .map(|path| {
                let path = if path.is_absolute() {
                    path
                } else {
                    repository.workdir.join(path)
                };
                let identity = snapshot::PathIdentity::capture(&path)
                    .map_err(|_| unsupported("Git configuration identity unavailable"))?;
                let entry = snapshot::Entry::capture_regular(&path, &mut budget)
                    .map_err(|_| unsupported("Git configuration source cannot be captured"))?;
                if !matches!(entry, snapshot::Entry::File(..) | snapshot::Entry::Missing) {
                    return Err(unsupported(
                        "Git configuration sources must be regular files",
                    ));
                }
                if snapshot::PathIdentity::capture(&path).ok().as_ref() != Some(&identity) {
                    return Err(unsupported("Git configuration path identity changed"));
                }
                if let snapshot::Entry::File(_, bytes) = &entry {
                    // Inspect declarations in their source bytes, preserving
                    // every occurrence. This is pure parsing of an already
                    // admitted source, never automatic include traversal.
                    auxiliary.extend(super::target_config::declared_paths(
                        bytes,
                        &path,
                        environment,
                        None,
                    )?);
                }
                identities.push(identity);
                Ok((path, entry))
            })
            .collect::<Result<_, _>>()?;
        Ok(Self {
            values,
            files,
            identities,
            origins,
            stable_auxiliary: auxiliary.clone(),
            ordered_output: output.to_vec(),
            auxiliary,
            hooks: vec![hook],
        })
    }

    pub(super) fn revalidate(&self) -> Result<(), GitToolError> {
        let mut budget = snapshot::Budget {
            entries: 4096,
            bytes: 4 * 1024 * 1024,
        };
        for ((path, expected), identity) in self.files.iter().zip(&self.identities) {
            if snapshot::PathIdentity::capture(path).ok().as_ref() != Some(identity) {
                return Err(unsupported("Git configuration path identity changed"));
            }
            let actual = snapshot::Entry::capture_regular(path, &mut budget)
                .map_err(|_| unsupported("Git configuration evidence unavailable"))?;
            if &actual != expected {
                return Err(unsupported("Git configuration changed after preparation"));
            }
            if snapshot::PathIdentity::capture(path).ok().as_ref() != Some(identity) {
                return Err(unsupported("Git configuration path identity changed"));
            }
        }
        Ok(())
    }

    /// Extend the fixed configuration discovery result without reading hook
    /// contents. Those paths require the continuation's separate admission.
    pub fn prepare_commit(
        &mut self,
        repository: &RepositoryState,
        environment: &BTreeMap<OsString, OsString>,
        commit_state: &super::repository_state::CommitState,
    ) -> Result<(), GitToolError> {
        let maintenance_enabled = self
            .values
            .get(b"maintenance.auto".as_slice())
            .and_then(|values| values.last())
            .map(|value| git_boolean(value.as_deref()))
            .transpose()?
            .unwrap_or(true);
        if maintenance_enabled && self.values.contains_key(b"gc.recentobjectshook".as_slice()) {
            return Err(unsupported(
                "Git automatic maintenance gc.recentObjectsHook requires separate execution authorization",
            ));
        }
        let repack_redirect = self
            .values
            .get(b"gc.repackfilterto".as_slice())
            .and_then(|values| values.last())
            .and_then(|value| value.as_deref())
            .is_some_and(|value| !value.is_empty());
        if maintenance_enabled && repack_redirect {
            return Err(unsupported(
                "Git automatic maintenance can write filtered packs to gc.repackFilterTo outside the declared object store",
            ));
        }
        refuse_promisor_objects(&self.values)?;
        let rerere_enabled = self
            .values
            .get(b"rerere.enabled".as_slice())
            .and_then(|values| values.last())
            .map(|value| git_boolean(value.as_deref()))
            .transpose()?
            .unwrap_or_else(|| commit_state.rerere_cache_exists(repository));
        if rerere_enabled && commit_state.has_rerere_records(repository) {
            return Err(unsupported(
                "Active Git rerere replay may modify worktree files and execute merge drivers",
            ));
        }
        if let Some(value) = self
            .values
            .get(b"commit.gpgsign".as_slice())
            .and_then(|values| values.last())
            && git_boolean(value.as_deref())?
        {
            return Err(unsupported(
                "Configured Git commit signing execution is not supported by this plan",
            ));
        }
        let directory = configured_path(&self.values, b"core.hookspath", environment)?
            .unwrap_or_else(|| repository.common_dir.join("hooks"));
        let directory = if directory.is_absolute() {
            directory
        } else {
            repository.workdir.join(directory)
        };
        for name in [
            "pre-commit",
            "prepare-commit-msg",
            "commit-msg",
            "post-commit",
            "reference-transaction",
            "pre-auto-gc",
        ] {
            let hook = directory.join(name);
            self.auxiliary.insert(hook.clone());
            self.hooks.push(hook);
        }
        // -m makes commit.template inactive. Do not read or grant an external
        // template which the actual command will not consume.
        Ok(())
    }

    pub(super) fn checkout_candidates(
        &self,
        branch: &str,
    ) -> Result<CheckoutCandidates, GitToolError> {
        if let Some(value) = self
            .values
            .get(b"checkout.guess".as_slice())
            .and_then(|values| values.last())
            && !git_boolean(value.as_deref())?
        {
            return Ok((Vec::new(), None));
        }
        let default = self
            .values
            .get(b"checkout.defaultremote".as_slice())
            .and_then(|values| values.last())
            .and_then(|value| value.as_ref())
            .map(|value| {
                String::from_utf8(value.clone())
                    .map_err(|_| unsupported("Checkout default remote is not UTF-8"))
            })
            .transpose()?;
        let source = format!("refs/heads/{branch}");
        let mut candidates = Vec::new();
        for (key, values) in &self.values {
            let Some(remote) = key
                .strip_prefix(b"remote.")
                .and_then(|key| key.strip_suffix(b".fetch"))
            else {
                continue;
            };
            let remote =
                std::str::from_utf8(remote).map_err(|_| unsupported("Remote name is not UTF-8"))?;
            for value in values.iter().flatten() {
                let spec = std::str::from_utf8(value)
                    .map_err(|_| unsupported("Fetch mapping is not UTF-8"))?
                    .trim_start_matches('+');
                if spec.starts_with('^') {
                    // Checkout's existing remote-tracking lookup uses positive
                    // mappings even when fetching that source is excluded. Git
                    // still decides whether to install tracking configuration.
                    continue;
                }
                let Some((from, to)) = spec.split_once(':') else {
                    continue;
                };
                let mapped = if let Some((prefix, suffix)) = from.split_once('*') {
                    source
                        .strip_prefix(prefix)
                        .and_then(|middle| middle.strip_suffix(suffix))
                        .map(|middle| to.replace('*', middle))
                } else if from == source {
                    Some(to.to_owned())
                } else {
                    None
                };
                if let Some(mapped) = mapped {
                    candidates.push((remote.to_owned(), mapped));
                }
            }
        }
        candidates.sort();
        candidates.dedup();
        Ok((candidates, default))
    }

    /// Bind checkout-specific configured execution before worktree mutation.
    /// Hook contents are inspected only after continuation Path admission.
    /// Resolve push helper effects without opening hooks before continuation
    /// admission. Transport-specific checks remain a separate execution gate.
    pub(super) fn prepare_push(
        &mut self,
        repository: &RepositoryState,
        environment: &BTreeMap<OsString, OsString>,
        remote: &str,
    ) -> Result<(), GitToolError> {
        refuse_promisor_objects(&self.values)?;
        if push_recursion_enabled(&self.ordered_output)? {
            return Err(unsupported(
                "Recursive submodule execution is not represented by this plan",
            ));
        }
        if let Some(value) = self
            .values
            .get(b"push.gpgsign".as_slice())
            .and_then(|values| values.last())
            && !matches!(git_boolean(value.as_deref()), Ok(false))
        {
            return Err(unsupported("Push signing is not represented by this plan"));
        }
        if self
            .values
            .get(b"push.negotiate".as_slice())
            .and_then(|values| values.last())
            .is_some_and(|value| !matches!(git_boolean(value.as_deref()), Ok(false)))
        {
            return Err(unsupported(
                "Push negotiation starts a separate fetch whose destination effects require admission",
            ));
        }
        for key in [
            "core.gitproxy".to_owned(),
            format!("remote.{remote}.vcs"),
            format!("remote.{remote}.receivepack"),
        ] {
            if self.values.contains_key(key.as_bytes()) {
                return Err(unsupported(
                    "Configured push transport program requires separate execution authorization",
                ));
            }
        }
        let directory = configured_path(&self.values, b"core.hookspath", environment)?
            .unwrap_or_else(|| repository.common_dir.join("hooks"));
        let directory = if directory.is_absolute() {
            directory
        } else {
            repository.workdir.join(directory)
        };
        for name in ["pre-push", "reference-transaction"] {
            let hook = directory.join(name);
            self.auxiliary.insert(hook.clone());
            self.hooks.push(hook);
        }
        // pack-objects consults the `delta` attribute while choosing deltas.
        // Global and info sources are already captured by capture_host; push
        // has no selected worktree paths to add this root rule automatically.
        self.auxiliary
            .insert(repository.workdir.join(".gitattributes"));
        Ok(())
    }

    /// Resolve fetch-triggered hooks without inspecting them before Read admission.
    pub(super) fn prepare_pull(
        &mut self,
        repository: &RepositoryState,
        environment: &BTreeMap<OsString, OsString>,
    ) -> Result<Vec<PathBuf>, GitToolError> {
        validate_fetch_external_effects(&self.values)?;
        let paths = fetch_hook_paths(
            &self.values,
            &repository.common_dir,
            &repository.workdir,
            environment,
        )?;
        self.auxiliary.extend(paths.iter().cloned());
        self.hooks.extend(paths.iter().cloned());
        Ok(paths)
    }

    /// Resolve integration hook candidates without reading or executing them.
    /// Fetch admission does not cover this separately admitted frontier.
    pub(super) fn pull_integration_hook_paths(
        &self,
        repository: &RepositoryState,
        environment: &BTreeMap<OsString, OsString>,
        integration: &super::pull_state::Integration,
    ) -> Result<Vec<PathBuf>, GitToolError> {
        use super::pull_state::Integration;
        self.validate_pull_integration_signing(integration)?;
        let directory = configured_path(&self.values, b"core.hookspath", environment)?
            .unwrap_or_else(|| repository.common_dir.join("hooks"));
        let directory = if directory.is_absolute() {
            directory
        } else {
            repository.workdir.join(directory)
        };
        let names: &[&str] = match integration {
            Integration::Merge { .. } => &[
                "pre-merge-commit",
                "prepare-commit-msg",
                "commit-msg",
                "post-merge",
                "post-checkout",
                "post-index-change",
                "reference-transaction",
                "pre-auto-gc",
            ],
            Integration::Rebase { .. } => &[
                "pre-rebase",
                "prepare-commit-msg",
                "commit-msg",
                "post-commit",
                "post-checkout",
                "post-index-change",
                "post-rewrite",
                "reference-transaction",
                "pre-auto-gc",
            ],
            Integration::Unborn(_) | Integration::Reconcile { .. } => {
                &["reference-transaction", "post-index-change"]
            }
        };
        Ok(names.iter().map(|name| directory.join(name)).collect())
    }

    // These booleans can start a signing/verifier program even when no hook
    // exists. Worktree scope approval does not authorize that execution.
    // Branch merge options and other external effects need their own checks;
    // this is deliberately not a complete integration execution validator.
    fn validate_pull_integration_signing(
        &self,
        integration: &super::pull_state::Integration,
    ) -> Result<(), GitToolError> {
        use super::pull_state::Integration;
        let keys: &[&[u8]] = match integration {
            Integration::Merge { .. } => &[b"commit.gpgsign", b"merge.verifysignatures"],
            Integration::Rebase { .. } => &[b"commit.gpgsign"],
            Integration::Unborn(_) | Integration::Reconcile { .. } => &[],
        };
        for key in keys {
            let mut enabled = false;
            for value in self.values.get(*key).into_iter().flatten() {
                // Git parses every declaration, including overridden values.
                enabled = git_boolean(value.as_deref())?;
            }
            if enabled {
                return Err(unsupported(
                    "Pull integration signing or signature verification requires separate execution authorization",
                ));
            }
        }
        Ok(())
    }

    /// Validate the selected fetch remote after policy discovery, before any fetch mutation.
    pub(super) fn prepare_pull_transport(&self, remote: &str) -> Result<(), GitToolError> {
        validate_fetch_external_effects(&self.values)?;
        for key in [
            format!("remote.{remote}.uploadpack"),
            format!("remote.{remote}.vcs"),
        ] {
            if self.values.contains_key(key.as_bytes()) {
                return Err(unsupported(
                    "Configured Pull transport helper requires separate execution authorization",
                ));
            }
        }
        Ok(())
    }

    /// Capture fetch-triggered hooks only after their exact paths were admitted.
    pub(super) fn capture_pull_hooks(
        paths: &[PathBuf],
    ) -> Result<Vec<(PathBuf, snapshot::RegularEvidence)>, GitToolError> {
        let mut budget = snapshot::Budget {
            entries: paths.len(),
            bytes: 4 * 1024 * 1024,
        };
        paths
            .iter()
            .map(|path| {
                let evidence = snapshot::RegularEvidence::capture(path, &mut budget)
                    .map_err(|_| unsupported("Git Pull hook must be a regular file"))?;
                if let snapshot::Entry::File(permissions, _) = &evidence.entry {
                    #[cfg(unix)]
                    let executable = {
                        use std::os::unix::fs::PermissionsExt;
                        permissions.mode() & 0o111 != 0
                    };
                    #[cfg(not(unix))]
                    let executable = {
                        let _ = permissions;
                        true
                    };
                    if executable {
                        return Err(unsupported(
                            "Executable Git Pull hooks require separate execution authorization",
                        ));
                    }
                }
                Ok((path.clone(), evidence))
            })
            .collect()
    }

    pub(super) fn has_branch_conditionals(&self) -> bool {
        self.values
            .keys()
            .any(|key| key.starts_with(b"includeif.onbranch:") && key.ends_with(b".path"))
    }

    /// Check the configuration potentially reloaded by post-switch object
    /// comparison. This does not replace invocation-cached hooks or filters.
    pub(super) fn validate_checkout_reload(
        records: &[(PathBuf, super::target_config::Record)],
    ) -> Result<(), GitToolError> {
        let mut values = Values::new();
        for (_, record) in records {
            values
                .entry(record.key.clone())
                .or_default()
                .push(record.value.clone());
        }
        refuse_promisor_objects(&values)?;
        if values
            .get(b"status.comparebranches".as_slice())
            .and_then(|entries| entries.last())
            .is_some_and(Option::is_none)
        {
            return Err(unsupported(
                "Target status.compareBranches requires a string value",
            ));
        }
        Ok(())
    }

    /// Compute late traditional-hook candidates without inspecting their paths.
    /// Git retains a non-null source hooksPath independently of its configset.
    /// With no such cached value, admit the target candidate conservatively.
    pub(super) fn checkout_reload_hook_paths(
        &self,
        records: &[(PathBuf, super::target_config::Record)],
        repository: &RepositoryState,
        environment: &BTreeMap<OsString, OsString>,
    ) -> Result<Vec<PathBuf>, GitToolError> {
        let source = configured_path(&self.values, b"core.hookspath", environment)?;
        let directory = if let Some(source) = source {
            source
        } else {
            let mut values = Values::new();
            for (_, record) in records {
                values
                    .entry(record.key.clone())
                    .or_default()
                    .push(record.value.clone());
            }
            configured_path(&values, b"core.hookspath", environment)?
                .unwrap_or_else(|| repository.common_dir.join("hooks"))
        };
        let directory = if directory.is_absolute() {
            directory
        } else {
            repository.workdir.join(directory)
        };
        Ok(["post-checkout", "reference-transaction"]
            .into_iter()
            .map(|name| directory.join(name))
            .collect())
    }

    /// Capture only after every candidate has separate exact Read admission.
    pub(super) fn capture_checkout_reload_hooks(
        paths: &[PathBuf],
    ) -> Result<Vec<(PathBuf, snapshot::RegularEvidence)>, GitToolError> {
        let mut budget = snapshot::Budget {
            entries: 4096,
            bytes: 4 * 1024 * 1024,
        };
        paths
            .iter()
            .map(|path| {
                let evidence =
                    snapshot::RegularEvidence::capture(path, &mut budget).map_err(|_| {
                        unsupported("Target checkout hook must be a bounded regular file")
                    })?;
                if let snapshot::Entry::File(permissions, _) = &evidence.entry {
                    #[cfg(unix)]
                    let executable = {
                        use std::os::unix::fs::PermissionsExt;
                        permissions.mode() & 0o111 != 0
                    };
                    #[cfg(not(unix))]
                    let executable = {
                        let _ = permissions;
                        true
                    };
                    if executable {
                        return Err(unsupported("Target checkout hook execution is not bound"));
                    }
                }
                Ok((path.clone(), evidence))
            })
            .collect()
    }

    pub(super) fn prepare_checkout(
        &mut self,
        repository: &RepositoryState,
        environment: &BTreeMap<OsString, OsString>,
        existing_local_branch: bool,
    ) -> Result<(), GitToolError> {
        refuse_promisor_objects(&self.values)?;
        // Existing local branch switches retain the initial process config.
        // Creating/tracking branches can write config and invalidate that cache;
        // their post-switch readers require separate target-chain discovery.
        if !existing_local_branch
            && self
                .values
                .keys()
                .any(|key| key.starts_with(b"includeif.onbranch:") && key.ends_with(b".path"))
        {
            return Err(unsupported(
                "Branch-conditional Git configuration requires target-branch discovery before checkout",
            ));
        }
        if let Some(value) = self
            .values
            .get(b"submodule.recurse".as_slice())
            .and_then(|values| values.last())
            && git_boolean(value.as_deref())?
        {
            return Err(unsupported(
                "Recursive checkout requires child worktree mutation authorization",
            ));
        }
        let directory = configured_path(&self.values, b"core.hookspath", environment)?
            .unwrap_or_else(|| repository.common_dir.join("hooks"));
        let directory = if directory.is_absolute() {
            directory
        } else {
            repository.workdir.join(directory)
        };
        for name in ["post-checkout", "reference-transaction"] {
            let hook = directory.join(name);
            self.auxiliary.insert(hook.clone());
            self.hooks.push(hook);
        }
        Ok(())
    }

    pub(super) fn validate_checkout_deletions(
        &self,
        roots: &[PathBuf],
        child_sources: &[PathBuf],
    ) -> Result<(), GitToolError> {
        for path in self
            .files
            .iter()
            .map(|(path, _)| path)
            .chain(self.stable_auxiliary.iter())
            .chain(self.hooks.iter())
            .chain(child_sources.iter())
        {
            for resolved in auxiliary_spellings(path)? {
                if roots
                    .iter()
                    .any(|root| potential_path_prefix(root, &resolved))
                {
                    return Err(unsupported(
                        "Checkout deletion overlaps an effective configuration or auxiliary source",
                    ));
                }
            }
        }
        Ok(())
    }

    /// Checkout must not create its own helper or rewrite configuration that a
    /// later child process can reload. Only invoked after auxiliary admission.
    pub(super) fn validate_checkout_target(
        &self,
        repository: &RepositoryState,
        target: &super::checkout_state::TargetTree,
        target_sources: &[PathBuf],
    ) -> Result<(), GitToolError> {
        let affected = std::panic::catch_unwind(|| {
            let repo = super::discover_repo(&repository.workdir)?;
            let index = repo
                .index_or_empty()
                .map_err(|e| GitToolError::Git(e.to_string()))?;
            let mut current = BTreeMap::new();
            for entry in index.entries() {
                let path = std::str::from_utf8(entry.path(&index))
                    .map_err(|_| unsupported("Non-UTF8 checkout source"))?;
                current.insert(
                    path.to_owned(),
                    (entry.mode.bits() as u16, entry.id.to_string()),
                );
            }
            Ok::<_, GitToolError>(
                current
                    .keys()
                    .chain(target.entries.keys())
                    .filter(|path| {
                        // Nonrecursive gitlink updates/removals preserve an
                        // initialized child's files. A tracked file becoming
                        // a gitlink does replace that file and must participate
                        // in configuration/hook target-effect checks.
                        !(current
                            .get(*path)
                            .is_some_and(|(mode, _)| *mode == 0o160000)
                            && target
                                .entries
                                .get(*path)
                                .is_none_or(|(mode, _)| *mode == 0o160000))
                            && !(!current.contains_key(*path)
                                && target
                                    .entries
                                    .get(*path)
                                    .is_some_and(|(mode, _)| *mode == 0o160000))
                            && !(!current.contains_key(*path)
                                && target
                                    .entries
                                    .get(*path)
                                    .is_some_and(|(mode, _)| *mode == 0o040000))
                            && current.get(*path) != target.entries.get(*path)
                    })
                    .map(|path| repository.workdir.join(path))
                    .collect::<BTreeSet<_>>(),
            )
        })
        .unwrap_or_else(|_| {
            Err(unsupported(
                "Checkout configuration overlap inspection failed",
            ))
        })?;
        for hook in &self.hooks {
            for resolved in auxiliary_spellings(hook)? {
                for (path, (mode, _)) in &target.entries {
                    let target_path = repository.workdir.join(path);
                    let same = potential_path_prefix(&target_path, &resolved)
                        && potential_path_prefix(&resolved, &target_path);
                    if (same && (*mode & 0o111 != 0 || *mode == 0o120000))
                        || (*mode != 0o040000
                            && !same
                            && potential_path_prefix(&target_path, &resolved))
                    {
                        return Err(unsupported(
                            "Checkout target introduces or changes an executable hook",
                        ));
                    }
                }
            }
        }
        let mut protected: Vec<PathBuf> = self.files.iter().map(|(path, _)| path.clone()).collect();
        protected.extend(self.stable_auxiliary.iter().cloned());
        protected.extend(target_sources.iter().cloned());
        for path in protected {
            for resolved in auxiliary_spellings(&path)? {
                if affected
                    .iter()
                    .any(|changed| potential_path_prefix(changed, &resolved))
                {
                    return Err(unsupported(
                        "Checkout rewrites an effective configuration or auxiliary source",
                    ));
                }
            }
        }
        Ok(())
    }

    /// Called only after the continuation's explicit auxiliary Path Read admission.
    pub fn capture_auxiliary(&self) -> Result<Vec<(PathBuf, snapshot::Entry)>, GitToolError> {
        for hook in &self.hooks {
            match std::fs::symlink_metadata(hook) {
                Ok(metadata) => {
                    #[cfg(unix)]
                    let executable = {
                        use std::os::unix::fs::PermissionsExt;
                        metadata.permissions().mode() & 0o111 != 0
                    };
                    #[cfg(not(unix))]
                    let executable = true;
                    if executable || metadata.file_type().is_symlink() {
                        return Err(unsupported(&format!(
                            "Configured Git {} hook execution is not supported by this plan",
                            hook.file_name().unwrap_or_default().to_string_lossy()
                        )));
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(_) => return Err(unsupported("Git hook cannot be inspected")),
            }
        }
        let mut budget = snapshot::Budget {
            entries: 4096,
            bytes: 4 * 1024 * 1024,
        };
        self.auxiliary
            .iter()
            .map(|path| {
                let entry = snapshot::Entry::capture(path, &mut budget)
                    .map_err(|_| unsupported("Git auxiliary file cannot be captured"))?;
                if !matches!(entry, snapshot::Entry::File(..) | snapshot::Entry::Missing) {
                    return Err(unsupported("Git auxiliary sources must be regular files"));
                }
                Ok((path.clone(), entry))
            })
            .collect()
    }

    pub fn verify_host(&self, output: &[u8]) -> Result<(), GitToolError> {
        let (host, origins) = parse_origins(output)?;
        if host != self.values || origins != self.origins || output != self.ordered_output {
            return Err(unsupported(
                "Host Git configuration differs from the prepared native configuration",
            ));
        }
        Ok(())
    }
}

fn fetch_hook_paths(
    values: &Values,
    common: &std::path::Path,
    worktree: &std::path::Path,
    environment: &BTreeMap<OsString, OsString>,
) -> Result<Vec<PathBuf>, GitToolError> {
    let directory = configured_path(values, b"core.hookspath", environment)?
        .unwrap_or_else(|| common.join("hooks"));
    let directory = if directory.is_absolute() {
        directory
    } else {
        worktree.join(directory)
    };
    Ok(["reference-transaction", "pre-auto-gc"]
        .into_iter()
        .map(|name| directory.join(name))
        .collect())
}

fn validate_fetch_external_effects(values: &Values) -> Result<(), GitToolError> {
    // Git fetch consumes this extra source before opening the selected
    // remote. Its bundles can read local paths or contact other endpoints;
    // approving the configured remote does not authorize either effect.
    // Git itself uses an empty final value to disable bundle retrieval.
    if values
        .get(b"fetch.bundleuri".as_slice())
        .and_then(|values| values.last())
        .is_some_and(|value| value.as_deref() != Some(b""))
    {
        return Err(unsupported(
            "Pull bundle URI sources require separate read or network authorization",
        ));
    }
    let maintenance_enabled = values
        .get(b"maintenance.auto".as_slice())
        .and_then(|values| values.last())
        .map(|value| git_boolean(value.as_deref()))
        .transpose()?
        .unwrap_or(true);
    if values.contains_key(b"core.gitproxy".as_slice())
        || maintenance_enabled
            && (values.contains_key(b"gc.recentobjectshook".as_slice())
                || values
                    .get(b"gc.repackfilterto".as_slice())
                    .and_then(|values| values.last())
                    .and_then(|value| value.as_deref())
                    .is_some_and(|value| !value.is_empty()))
    {
        return Err(unsupported(
            "Configured Git fetch helper or external maintenance effects require separate authorization",
        ));
    }
    Ok(())
}

fn refuse_promisor_objects(values: &Values) -> Result<(), GitToolError> {
    for (key, entries) in values {
        if key == b"extensions.partialclone"
            || (key.starts_with(b"remote.")
                && (key.ends_with(b".partialclonefilter")
                    || (key.ends_with(b".promisor")
                        && entries
                            .iter()
                            .any(|value| git_boolean(value.as_deref()).unwrap_or(true)))))
        {
            return Err(unsupported(
                "Git object inspection may fetch missing objects from a promisor remote; network effects require a separately authorized invocation",
            ));
        }
    }
    Ok(())
}

/// Parse the explicit boolean spellings supported by gix.
/// An implicit value (no `=`) is true, matching Git's config semantics. Invalid
/// values are refused rather than being guessed as disabled.
pub(super) fn git_boolean(value: Option<&[u8]>) -> Result<bool, GitToolError> {
    let Some(value) = value else {
        return Ok(true);
    };
    let value = gix::bstr::BStr::new(value);
    if let Ok(boolean) = gix::config::Boolean::try_from(value) {
        return Ok(boolean.into());
    }
    if let Ok(integer) = gix::config::Integer::try_from(value) {
        return integer
            .to_decimal()
            .map(|value| value != 0)
            .ok_or_else(|| unsupported("Unsupported Git boolean configuration"));
    }
    Err(unsupported("Unsupported Git boolean configuration"))
}

/// Include declarations remain relevant even when their destination is empty,
/// missing, or inactive and therefore contributes no --show-origin values.
/// Only return paths here; contents require continuation admission.
pub(super) fn declared_include_paths(
    output: &[u8],
    workdir: &std::path::Path,
    environment: &BTreeMap<OsString, OsString>,
) -> Result<BTreeSet<PathBuf>, GitToolError> {
    parse_origins(output)?;
    let mut result = BTreeSet::new();
    if output.is_empty() {
        return Ok(result);
    }
    let fields: Vec<_> = output[..output.len() - 1]
        .split(|byte| *byte == 0)
        .collect();
    for [origin, record] in fields.as_chunks::<2>().0 {
        let mut encoded = record.to_vec();
        encoded.push(0);
        let values = parse_host(&encoded)?;
        for key in values.keys().filter(|key| {
            key.as_slice() == b"include.path"
                || (key.starts_with(b"includeif.") && key.ends_with(b".path"))
        }) {
            let Some(value) = values.get(key).and_then(|values| values.last()) else {
                continue;
            };
            let Some(path) = configured_include_path_value(value, environment)? else {
                continue;
            };
            if path.is_absolute() {
                result.insert(path);
            } else {
                let origin = std::str::from_utf8(&origin[b"file:".len()..])
                    .map_err(|_| unsupported("Invalid include origin"))?;
                let origin = PathBuf::from(origin);
                let origin = if origin.is_absolute() {
                    origin
                } else {
                    workdir.join(origin)
                };
                result.insert(
                    origin
                        .parent()
                        .ok_or_else(|| unsupported("Include origin has no parent"))?
                        .join(path),
                );
            }
        }
    }
    Ok(result)
}

pub(super) fn configured_path(
    values: &Values,
    key: &[u8],
    environment: &BTreeMap<OsString, OsString>,
) -> Result<Option<PathBuf>, GitToolError> {
    let Some(value) = values.get(key).and_then(|values| values.last()) else {
        return Ok(None);
    };
    configured_path_value(value, environment)
}

pub(super) fn configured_path_value(
    value: &Option<Vec<u8>>,
    environment: &BTreeMap<OsString, OsString>,
) -> Result<Option<PathBuf>, GitToolError> {
    configured_path_value_inner(value, environment, false)
}

// Include paths retain their spelling: Sources separately admits and binds every
// traversal ancestor before reading the leaf. Other configured paths retain the
// receiver syntax restriction until their own referent discovery is implemented.
pub(super) fn configured_include_path_value(
    value: &Option<Vec<u8>>,
    environment: &BTreeMap<OsString, OsString>,
) -> Result<Option<PathBuf>, GitToolError> {
    configured_path_value_inner(value, environment, true)
}

fn configured_path_value_inner(
    value: &Option<Vec<u8>>,
    environment: &BTreeMap<OsString, OsString>,
    include: bool,
) -> Result<Option<PathBuf>, GitToolError> {
    let value = value
        .as_ref()
        .ok_or_else(|| unsupported("Implicit Git path configuration"))?;
    let value = std::str::from_utf8(value)
        .map_err(|_| unsupported("Git path configuration is not UTF-8"))?;
    if value.as_bytes().contains(&0) {
        return Err(unsupported("Git path configuration contains NUL"));
    }
    if let Some(relative) = value.strip_prefix("~/") {
        let home = environment
            .get(OsStr::new("HOME"))
            .ok_or_else(|| unsupported("Git home path unavailable"))?;
        let home_path = std::path::Path::new(home);
        let relative_path = std::path::Path::new(relative);
        if !home_path.is_absolute()
            || relative_path.has_root()
            || relative_path
                .components()
                .any(|part| part == std::path::Component::ParentDir)
        {
            return Err(unsupported("Git home path interpolation is ambiguous"));
        }
        super::remote_state::validate_local_path_syntax(home_path)?;
        return Ok(Some(PathBuf::from(home).join(relative)));
    }
    if value.starts_with('~') || value.starts_with("%(") {
        return Err(unsupported("Git path interpolation is unsupported"));
    }
    if !include {
        super::remote_state::validate_local_path_syntax(std::path::Path::new(value))?;
    }
    Ok(Some(PathBuf::from(value)))
}

fn parse_origins(bytes: &[u8]) -> Result<(Values, BTreeSet<PathBuf>), GitToolError> {
    if bytes.is_empty() {
        return Ok((BTreeMap::new(), BTreeSet::new()));
    }
    if bytes.last() != Some(&0) {
        return Err(unsupported("Incomplete Git configuration origins"));
    }
    let fields = bytes[..bytes.len() - 1]
        .split(|byte| *byte == 0)
        .collect::<Vec<_>>();
    let (pairs, remainder) = fields.as_chunks::<2>();
    if !remainder.is_empty() {
        return Err(unsupported("Invalid Git configuration origins"));
    }
    let mut paths = BTreeSet::new();
    let mut records = Vec::new();
    for [origin, record] in pairs {
        let path = origin
            .strip_prefix(b"file:")
            .ok_or_else(|| unsupported("Non-file Git configuration origin"))?;
        let path = std::str::from_utf8(path)
            .map_err(|_| unsupported("Git configuration origin is not UTF-8"))?;
        paths.insert(PathBuf::from(path));
        records.extend_from_slice(record);
        records.push(0);
    }
    Ok((parse_host(&records)?, paths))
}

pub(super) fn parse_host(bytes: &[u8]) -> Result<Values, GitToolError> {
    if !bytes.is_empty() && bytes.last() != Some(&0) {
        return Err(unsupported("Incomplete host Git configuration output"));
    }
    let mut values: Values = BTreeMap::new();
    for record in bytes.split_inclusive(|byte| *byte == 0) {
        let record = &record[..record.len() - 1];
        let (key, value) = match record.iter().position(|byte| *byte == b'\n') {
            Some(split) => (&record[..split], Some(record[split + 1..].to_vec())),
            None => (record, None),
        };
        let mut key = key.to_vec();
        let first = key
            .iter()
            .position(|byte| *byte == b'.')
            .ok_or_else(|| unsupported("Invalid host Git configuration key"))?;
        let last = key
            .iter()
            .rposition(|byte| *byte == b'.')
            .ok_or_else(|| unsupported("Invalid host Git configuration key"))?;
        key[..first].make_ascii_lowercase();
        key[last + 1..].make_ascii_lowercase();
        values.entry(key).or_default().push(value);
    }
    Ok(values)
}

#[cfg(test)]
mod tests {
    include!("config_state_tests/pull_signing.rs");

    #[test]
    fn child_startup_layout_matches_native_selection_rules() {
        for case in [
            "stale-private",
            "linked-suppressed",
            "private-override",
            "bare",
            "no-version",
        ] {
            let root = tempfile::tempdir().expect("fixture");
            let worktree = root.path().canonicalize().expect("canonical");
            assert!(
                std::process::Command::new("git")
                    .current_dir(&worktree)
                    .args(["init"])
                    .output()
                    .expect("init")
                    .status
                    .success()
            );
            let gitdir = worktree.join(".git");
            let selected = worktree.join("selected");
            std::fs::create_dir(&selected).expect("selected worktree");
            let has_common = case == "linked-suppressed";
            if has_common {
                // Presence matters even if commondir resolves back to gitdir.
                std::fs::write(gitdir.join("commondir"), ".\n").expect("commondir");
            }
            let version = if case == "no-version" {
                ""
            } else {
                "repositoryFormatVersion=0\n"
            };
            let bare = case == "bare";
            let raw = if matches!(
                case,
                "linked-suppressed" | "private-override" | "bare" | "no-version"
            ) {
                worktree.join("unopened-missing")
            } else {
                selected.clone()
            };
            let extension = if case == "private-override" {
                "[extensions]\nworktreeConfig=true\n"
            } else {
                ""
            };
            std::fs::write(
                gitdir.join("config"),
                format!(
                    "[core]\n{version}bare={bare}\nworktree={}\n{extension}",
                    raw.display()
                ),
            )
            .expect("config");
            let private_target = if case == "private-override" {
                selected.clone()
            } else {
                worktree.join("ignored-missing")
            };
            std::fs::write(
                gitdir.join("config.worktree"),
                format!("[core]\nworktree={}\n", private_target.display()),
            )
            .expect("private config");
            let layout =
                super::ChildStartupLayout::capture(&gitdir, &gitdir, has_common).expect("layout");
            let expected = if matches!(case, "stale-private" | "private-override") {
                BTreeSet::from([selected.clone()])
            } else {
                BTreeSet::new()
            };
            assert_eq!(layout.targets, expected, "{case}");
            let output = std::process::Command::new("git")
                .current_dir(&worktree)
                .arg("--git-dir")
                .arg(&gitdir)
                .args(["rev-parse", "--show-toplevel"])
                .output()
                .expect("native startup");
            if bare {
                assert!(!output.status.success(), "bare has no worktree");
            } else {
                assert!(
                    output.status.success(),
                    "{case}: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                assert_eq!(
                    PathBuf::from(String::from_utf8(output.stdout).expect("UTF8").trim()),
                    expected.iter().next().unwrap_or(&worktree).clone(),
                    "{case}"
                );
            }
        }
    }

    #[test]
    fn child_startup_layout_retains_raw_target_hidden_by_include() {
        let root = tempfile::tempdir().expect("fixture");
        let gitdir = root.path().join("metadata");
        std::fs::create_dir(&gitdir).expect("metadata");
        std::fs::write(gitdir.join("config"),
            "[core]\nrepositoryFormatVersion=0\nworktree=../unapproved\n[include]\npath=override.conf\n").expect("config");
        std::fs::write(
            gitdir.join("override.conf"),
            "[core]\nworktree=../approved\n",
        )
        .expect("include");
        let layout = super::ChildStartupLayout::capture(&gitdir, &gitdir, false).expect("layout");
        assert_eq!(
            layout.targets,
            BTreeSet::from([gitdir.join("../unapproved")])
        );
        assert!(!root.path().join("unapproved").exists());
        layout.revalidate().expect("unchanged");
        std::fs::write(
            gitdir.join("config.worktree"),
            "[core]\nworktree=../another\n",
        )
        .expect("late config");
        layout
            .revalidate()
            .expect("disabled private config is ignored");
        let layout =
            super::ChildStartupLayout::capture(&gitdir, &gitdir, false).expect("new layout");
        assert!(!layout.targets.contains(&gitdir.join("../another")));
        std::fs::write(gitdir.join("config"),
            "[core]\nrepositoryFormatVersion=0\nworktree=../unapproved\n[extensions]\nworktreeConfig=true\n").expect("enabled config");
        assert!(layout.revalidate().is_err());
        let enabled =
            super::ChildStartupLayout::capture(&gitdir, &gitdir, false).expect("enabled layout");
        assert_eq!(enabled.targets, BTreeSet::from([gitdir.join("../another")]));
    }

    #[test]
    fn checkout_reload_hook_inventory_preserves_source_cache_and_defers_reads() {
        let root = tempfile::tempdir().expect("fixture");
        assert!(
            std::process::Command::new("git")
                .arg("init")
                .current_dir(root.path())
                .output()
                .expect("git")
                .status
                .success()
        );
        let repository = RepositoryState::capture(root.path()).expect("repository");
        let environment = BTreeMap::new();
        let mut config = ConfigState::capture_host(b"", &repository, &environment).expect("config");
        let records = vec![(
            PathBuf::from("target.conf"),
            super::super::target_config::Record {
                key: b"core.hookspath".to_vec(),
                value: Some(b"target-hooks".to_vec()),
            },
        )];
        let paths = config
            .checkout_reload_hook_paths(&records, &repository, &environment)
            .expect("inventory");
        assert_eq!(
            paths,
            vec![
                repository.workdir.join("target-hooks/post-checkout"),
                repository
                    .workdir
                    .join("target-hooks/reference-transaction")
            ]
        );
        assert!(!repository.workdir.join("target-hooks").exists());
        let absent =
            ConfigState::capture_checkout_reload_hooks(&paths).expect("admitted missing paths");
        assert!(
            absent
                .iter()
                .all(|(_, evidence)| matches!(evidence.entry, snapshot::Entry::Missing))
        );
        std::fs::create_dir(repository.workdir.join("target-hooks")).expect("directory");
        std::fs::write(&paths[0], b"fixture").expect("hook");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&paths[0], std::fs::Permissions::from_mode(0o700))
                .expect("mode");
        }
        assert!(ConfigState::capture_checkout_reload_hooks(&paths).is_err());
        config.values.insert(
            b"core.hookspath".to_vec(),
            vec![Some(b"source-hooks".to_vec())],
        );
        let cached = config
            .checkout_reload_hook_paths(&records, &repository, &environment)
            .expect("cached source");
        assert_eq!(
            cached[0],
            repository.workdir.join("source-hooks/post-checkout")
        );
        // Inventory generation never probes even an executable target candidate.
        assert_eq!(
            paths,
            ConfigState::capture_host(b"", &repository, &environment)
                .expect("config")
                .checkout_reload_hook_paths(&records, &repository, &environment)
                .expect("pure inventory")
        );
    }

    #[test]
    fn checkout_reload_refuses_target_promisor_and_uses_final_comparison_value() {
        let records = |entries: &[(&str, Option<&str>)]| {
            entries
                .iter()
                .map(|(key, value)| {
                    (
                        PathBuf::from("target.conf"),
                        super::super::target_config::Record {
                            key: key.as_bytes().to_vec(),
                            value: value.map(|value| value.as_bytes().to_vec()),
                        },
                    )
                })
                .collect::<Vec<_>>()
        };
        for entry in [
            ("remote.target.promisor", Some("true")),
            ("remote.target.promisor", None),
            ("remote.target.promisor", Some("invalid")),
            ("remote.target.partialclonefilter", Some("blob:none")),
            ("extensions.partialclone", Some("target")),
        ] {
            assert!(ConfigState::validate_checkout_reload(&records(&[entry])).is_err());
        }
        assert!(
            ConfigState::validate_checkout_reload(&records(&[
                ("remote.target.promisor", Some("false")),
                ("status.comparebranches", None),
                ("status.comparebranches", Some("@{upstream}")),
            ]))
            .is_ok()
        );
        assert!(
            ConfigState::validate_checkout_reload(&records(&[
                ("status.comparebranches", Some("@{upstream}")),
                ("status.comparebranches", None),
            ]))
            .is_err()
        );
    }

    #[test]
    fn receiver_effect_inventory_defers_reads_and_rejects_unbound_commands() {
        let root = tempfile::tempdir().expect("fixture");
        let base = root.path().canonicalize().expect("canonical");
        let mut config = ChildConfig::capture(b"", &base).expect("empty configuration");
        config.values.insert(
            b"core.hookspath".to_vec(),
            vec![Some(b"relative-hooks".to_vec())],
        );
        config.values.insert(
            b"receive.fsck.skiplist".to_vec(),
            vec![Some(b"relative-skiplist".to_vec())],
        );
        let effects = config
            .receiver_effects(&base, &base, &BTreeMap::new())
            .expect("inventory");
        assert!(
            effects
                .hooks
                .contains(&base.join("relative-hooks/pre-receive"))
        );
        assert!(effects.paths.contains(&base.join("relative-skiplist")));
        assert!(!base.join("relative-hooks").exists());
        assert!(!base.join("relative-skiplist").exists());
        effects.capture().expect("missing auxiliaries");
        for (key, value) in [
            ("hook.check.command", "command"),
            ("core.alternateRefsCommand", "command"),
            ("gc.recentObjectsHook", "command"),
            ("gc.repackFilterTo", "/outside"),
            ("receive.denyCurrentBranch", "updateInstead"),
        ] {
            let key = key.to_ascii_lowercase().into_bytes();
            config
                .values
                .insert(key.clone(), vec![Some(value.as_bytes().to_vec())]);
            assert!(
                config
                    .receiver_effects(&base, &base, &BTreeMap::new())
                    .is_err(),
                "{key:?}"
            );
            config.values.remove(&key);
        }
        config
            .values
            .insert(b"receive.autogc".to_vec(), vec![Some(b"false".to_vec())]);
        config.values.insert(
            b"gc.recentobjectshook".to_vec(),
            vec![Some(b"inactive".to_vec())],
        );
        config.values.insert(
            b"gc.repackfilterto".to_vec(),
            vec![Some(b"/inactive".to_vec())],
        );
        config
            .receiver_effects(&base, &base, &BTreeMap::new())
            .expect("disabled maintenance");
    }

    #[cfg(unix)]
    #[test]
    fn receiver_auxiliary_capture_refuses_executable_hooks_and_object_alternates() {
        use std::os::unix::fs::PermissionsExt;
        let root = tempfile::tempdir().expect("fixture");
        let base = root.path().canonicalize().expect("canonical");
        let config = ChildConfig::capture(b"", &base).expect("config");
        let effects = config
            .receiver_effects(&base, &base, &BTreeMap::new())
            .expect("inventory");
        std::fs::create_dir(base.join("hooks")).expect("hooks");
        let hook = base.join("hooks/pre-receive");
        std::fs::write(&hook, "#!/bin/sh\nexit 0\n").expect("hook");
        std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755))
            .expect("executable");
        assert!(
            effects
                .capture()
                .expect_err("refuse hook")
                .to_string()
                .contains("Executable receiver hook")
        );
        std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o644))
            .expect("inactive hook");
        effects.capture().expect("inactive hook allowed");
        std::fs::create_dir_all(base.join("objects/info")).expect("objects");
        std::fs::write(
            base.join("objects/info/alternates"),
            "/unopened-object-store\n",
        )
        .expect("alternates");
        assert!(
            effects
                .capture()
                .expect_err("refuse alternates")
                .to_string()
                .contains("object alternates")
        );
    }

    #[test]
    fn receiver_capture_rejects_relative_origin_before_snapshot() {
        let root = tempfile::tempdir().expect("fixture");
        let error = ChildConfig::capture_receiver(
            b"file:missing-relative\0core.bare\nfalse\0",
            root.path(),
        )
        .err()
        .expect("relative origin refusal");
        assert!(error.to_string().contains("origin must be absolute"));
    }

    #[test]
    fn include_traversal_preserves_spelling_without_relaxing_other_paths() {
        let environment = BTreeMap::new();
        for spelling in ["directory/../config", "/directory/../config"] {
            let value = Some(spelling.as_bytes().to_vec());
            assert_eq!(
                configured_include_path_value(&value, &environment)
                    .expect("include spelling")
                    .expect("path"),
                PathBuf::from(spelling)
            );
            assert!(configured_path_value(&value, &environment).is_err());
            let output = format!("file:/repository/config\0include.path\n{spelling}\0");
            let paths = declared_include_paths(
                output.as_bytes(),
                std::path::Path::new("/repository"),
                &environment,
            )
            .expect("declaration only");
            let expected = if spelling.starts_with('/') {
                PathBuf::from(spelling)
            } else {
                PathBuf::from("/repository").join(spelling)
            };
            assert_eq!(paths, BTreeSet::from([expected]));
        }
    }

    #[test]
    fn declared_home_include_paths_reject_ambiguous_interpolation() {
        for (home, value) in [
            ("/alias/../home", "~/config"),
            ("/home", "~//absolute"),
            ("/home", "~/../outside"),
            ("relative", "~/config"),
        ] {
            let environment = BTreeMap::from([(OsString::from("HOME"), OsString::from(home))]);
            let output = format!("file:/config\0include.path\n{value}\0");
            assert!(
                declared_include_paths(
                    output.as_bytes(),
                    std::path::Path::new("/repository"),
                    &environment
                )
                .is_err()
            );
        }
    }

    #[test]
    fn configuration_verification_rejects_cross_key_reordering() {
        let root = tempfile::tempdir().expect("fixture");
        assert!(
            std::process::Command::new("git")
                .arg("init")
                .current_dir(root.path())
                .output()
                .expect("git")
                .status
                .success()
        );
        let repository = RepositoryState::capture(root.path()).expect("repository");
        let path = root.path().join(".git/config");
        let origin = format!("file:{}", path.display());
        let first =
            format!("{origin}\0submodule.recurse\ntrue\0{origin}\0push.recursesubmodules\nfalse\0");
        let reordered =
            format!("{origin}\0push.recursesubmodules\nfalse\0{origin}\0submodule.recurse\ntrue\0");
        assert_eq!(
            parse_origins(first.as_bytes()).expect("first"),
            parse_origins(reordered.as_bytes()).expect("second")
        );
        let config = ConfigState::capture_host(first.as_bytes(), &repository, &BTreeMap::new())
            .expect("capture");
        config.verify_host(first.as_bytes()).expect("same ordering");
        assert!(config.verify_host(reordered.as_bytes()).is_err());
    }

    #[test]
    fn child_fetch_remote_binds_branch_and_single_remote_defaults() {
        let root = tempfile::Builder::new()
            .prefix("talos-config-")
            .tempdir()
            .expect("fixture");
        let path = root.path().join("config-without-dot");
        std::fs::write(&path, "").expect("source");
        let output = [
            "remote.backup.url\ngit://backup.invalid/repo",
            "remote.upstream.url\ngit://upstream.invalid/repo",
            "branch.Topic.remote\nupstream",
        ]
        .iter()
        .map(|record| format!("file:{}\0{record}\0", path.display()))
        .collect::<String>();
        let config = ChildConfig::capture(output.as_bytes(), root.path()).expect("config");
        assert_eq!(
            config.fetch_remote(Some("Topic")).expect("branch"),
            "upstream"
        );
        assert_eq!(config.fetch_remote(Some("topic")).expect("case"), "origin");
        assert_eq!(config.fetch_remote(None).expect("detached"), "origin");
        let output = format!(
            "file:{}\0remote.backup.url\ngit://backup.invalid/repo\0",
            path.display()
        );
        let config = ChildConfig::capture(output.as_bytes(), root.path()).expect("config");
        assert_eq!(config.fetch_remote(None).expect("single"), "backup");
    }

    #[test]
    fn push_recursion_uses_cross_key_declaration_order() {
        for (records, expected) in [
            (
                "file:a\0submodule.recurse\ntrue\0file:b\0push.recursesubmodules\nfalse\0",
                false,
            ),
            (
                "file:a\0push.recursesubmodules\nfalse\0file:b\0submodule.recurse\ntrue\0",
                true,
            ),
            (
                "file:a\0push.recursesubmodules\non-demand\0file:b\0submodule.recurse\nfalse\0",
                false,
            ),
            (
                "file:a\0submodule.recurse\nfalse\0file:b\0push.recursesubmodules\ncheck\0",
                true,
            ),
        ] {
            assert_eq!(
                push_recursion_enabled(records.as_bytes()).expect("config"),
                expected
            );
        }
        assert!(push_recursion_enabled(b"file:a\0submodule.recurse\ninvalid\0").is_err());
        // Git aborts on an invalid early value even when a later key disables
        // recursion. Do not approve a plan by considering only the last value.
        for value in ["true", "yes", "1", "ON", "invalid"] {
            let output = format!(
                "file:a\0push.recursesubmodules\n{value}\0file:b\0submodule.recurse\nfalse\0"
            );
            assert!(
                push_recursion_enabled(output.as_bytes()).is_err(),
                "{value}"
            );
        }
    }

    #[test]
    fn push_helper_checks_preserve_disabled_settings_and_defer_hook_reads() {
        let root = tempfile::tempdir().expect("fixture");
        assert!(
            std::process::Command::new("git")
                .args(["init"])
                .current_dir(root.path())
                .output()
                .expect("git")
                .status
                .success()
        );
        let repository = RepositoryState::capture(root.path()).expect("repository");
        let environment = BTreeMap::new();
        for (key, value, refused) in [
            ("push.gpgsign", "false", false),
            ("push.gpgsign", "if-asked", true),
            ("push.recursesubmodules", "false", false),
            ("push.recursesubmodules", "on-demand", true),
            ("submodule.recurse", "true", true),
            ("submodule.recurse", "false", false),
            ("push.negotiate", "true", true),
            ("push.negotiate", "false", false),
            ("remote.origin.receivepack", "custom-receive", true),
            ("remote.origin.vcs", "custom", true),
            ("core.gitproxy", "custom-proxy", true),
        ] {
            let output = format!(
                "file:{}\0{key}\n{value}\0",
                repository.common_dir.join("config").display()
            );
            let mut config =
                ConfigState::capture_host(output.as_bytes(), &repository, &environment)
                    .expect("config");
            assert_eq!(
                config
                    .prepare_push(&repository, &environment, "origin",)
                    .is_err(),
                refused,
                "{key}"
            );
            if !refused {
                assert!(
                    config
                        .auxiliary
                        .contains(&repository.common_dir.join("hooks/pre-push"))
                );
            }
        }
    }

    #[test]
    fn pull_helper_checks_follow_maintenance_and_selected_remote() {
        let root = tempfile::tempdir().expect("fixture");
        assert!(
            std::process::Command::new("git")
                .args(["init"])
                .current_dir(root.path())
                .output()
                .expect("git")
                .status
                .success()
        );
        let repository = RepositoryState::capture(root.path()).expect("repository");
        let capture = |records: &str| {
            let source = repository.common_dir.join("config");
            let output = records
                .split_terminator('\0')
                .map(|record| format!("file:{}\0{record}\0", source.display()))
                .collect::<String>();
            ConfigState::capture_host(output.as_bytes(), &repository, &BTreeMap::new())
                .expect("config")
        };

        for records in [
            "maintenance.auto\nfalse\0gc.recentObjectsHook\nhook\0gc.repackFilterTo\n/out\0",
            "maintenance.auto\n0\0gc.recentObjectsHook\nhook\0gc.repackFilterTo\n\0",
        ] {
            let mut config = capture(records);
            config
                .prepare_pull(&repository, &BTreeMap::new())
                .expect("disabled maintenance keeps external settings inactive");
        }
        for records in [
            "maintenance.auto\ntrue\0gc.recentObjectsHook\nhook\0",
            "maintenance.auto\nyes\0gc.repackFilterTo\n/out\0",
            "gc.recentObjectsHook\nhook\0",
            "gc.repackFilterTo\n/out\0",
        ] {
            let mut config = capture(records);
            assert!(
                config.prepare_pull(&repository, &BTreeMap::new()).is_err(),
                "{records}"
            );
        }

        let config = capture("remote.origin.uploadpack\ncustom-upload\0");
        assert!(config.prepare_pull_transport("origin").is_err());
        assert!(config.prepare_pull_transport("other").is_ok());
        let config = capture("remote.origin.vcs\ncustom-vcs\0");
        assert!(config.prepare_pull_transport("origin").is_err());
        assert!(config.prepare_pull_transport("other").is_ok());

        for records in [
            "fetch.bundleURI\n/outside.bundle\0",
            "fetch.bundleURI\nhttps://example.invalid/bundles\0",
            "fetch.bundleURI\0",
            "fetch.bundleURI\n\0fetch.bundleURI\n/outside.bundle\0",
        ] {
            assert!(
                capture(records)
                    .prepare_pull(&repository, &BTreeMap::new())
                    .is_err(),
                "{records}"
            );
        }
        for records in [
            "fetch.bundleURI\n\0",
            "fetch.bundleURI\n/outside.bundle\0fetch.bundleURI\n\0",
        ] {
            capture(records)
                .prepare_pull(&repository, &BTreeMap::new())
                .expect("empty final bundle URI disables retrieval");
        }
    }

    #[test]
    fn checkout_hooks_are_deferred_and_recursive_effects_are_explicit() {
        let root = tempfile::tempdir().expect("fixture");
        assert!(
            std::process::Command::new("git")
                .arg("init")
                .arg(root.path())
                .output()
                .expect("git")
                .status
                .success()
        );
        let repository = RepositoryState::capture(root.path()).expect("repository");
        let environment = BTreeMap::new();
        let mut config = ConfigState::capture_host(b"", &repository, &environment).expect("config");
        let hook = repository.common_dir.join("hooks/post-checkout");
        std::fs::write(&hook, "#!/bin/sh\nexit 1\n").expect("hook");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755))
                .expect("executable");
        }
        config
            .prepare_checkout(&repository, &environment, false)
            .expect("discovery does not inspect hook");
        assert!(config.auxiliary.contains(&hook));
        assert!(
            config
                .capture_auxiliary()
                .expect_err("active hook")
                .to_string()
                .contains("post-checkout")
        );
        let mut config = ConfigState::capture_host(b"", &repository, &environment).expect("config");
        config
            .values
            .insert(b"submodule.recurse".to_vec(), vec![Some(b"true".to_vec())]);
        assert!(
            config
                .prepare_checkout(&repository, &environment, false)
                .expect_err("child write scope")
                .to_string()
                .contains("child worktree")
        );
        config
            .values
            .insert(b"submodule.recurse".to_vec(), vec![Some(b"false".to_vec())]);
        config
            .prepare_checkout(&repository, &environment, false)
            .expect("nonrecursive gitlinks allowed");
        config.values.insert(
            b"includeif.onbranch:target.path".to_vec(),
            vec![Some(b"target-config".to_vec())],
        );
        assert!(
            config
                .prepare_checkout(&repository, &environment, false)
                .expect_err("conditional reload")
                .to_string()
                .contains("target-branch discovery")
        );
    }

    #[test]
    fn child_excludes_follow_child_config_and_frozen_environment() {
        let root = tempfile::tempdir().expect("fixture");
        let config = ChildConfig::capture(b"", root.path()).expect("empty config");
        let common = root.path().join("metadata");
        let environment = BTreeMap::from([
            (OsString::from("HOME"), OsString::from("/home-fixture")),
            (
                OsString::from("XDG_CONFIG_HOME"),
                OsString::from("/xdg-fixture"),
            ),
        ]);
        let paths = config
            .attribute_paths(root.path(), &common, &environment)
            .expect("defaults");
        assert!(paths.contains(&PathBuf::from("/xdg-fixture/git/ignore")));
        assert!(paths.contains(&common.join("info/exclude")));
        assert!(!paths.contains(&PathBuf::from("/home-fixture/.config/git/ignore")));
        let source = root.path().join("config");
        std::fs::write(&source, "[core]\n excludesFile = rules\n").expect("source");
        let output = format!("file:{}\0core.excludesfile\nrules\0", source.display());
        let configured = ChildConfig::capture(output.as_bytes(), root.path()).expect("config");
        let paths = configured
            .attribute_paths(root.path(), &common, &environment)
            .expect("override");
        assert!(paths.contains(&root.path().join("rules")));
        assert!(!paths.contains(&PathBuf::from("/xdg-fixture/git/ignore")));
    }

    #[test]
    fn child_promisor_configuration_refuses_object_inspection() {
        let root = tempfile::tempdir().expect("fixture");
        for value in [
            b"remote.origin.promisor\ntrue\0".as_slice(),
            b"remote.origin.partialclonefilter\nblob:none\0",
            b"extensions.partialclone\norigin\0",
        ] {
            let mut output =
                format!("file:{}\0", root.path().join("config").display()).into_bytes();
            output.extend_from_slice(value);
            let error = ChildConfig::capture(&output, root.path())
                .err()
                .expect("refusal");
            assert!(error.to_string().contains("promisor remote"), "{error}");
        }
    }

    #[test]
    fn child_pull_hook_paths_preserve_common_default_and_effective_worktree_override() {
        let root = tempfile::tempdir().expect("fixture");
        let source = root.path().join("config");
        std::fs::write(&source, "").expect("source");
        let common = root.path().join("common");
        let worktree = root.path().join("effective-worktree");
        let environment = BTreeMap::new();
        let config = ChildConfig::capture(b"", root.path()).expect("empty config");
        assert_eq!(
            config
                .pull_hook_paths(&common, &worktree, &environment)
                .expect("defaults"),
            [
                common.join("hooks/reference-transaction"),
                common.join("hooks/pre-auto-gc")
            ]
        );
        let output = format!(
            "file:{}\0core.hookspath\nrelative-hooks\0",
            source.display()
        );
        let config = ChildConfig::capture(output.as_bytes(), root.path()).expect("override");
        assert_eq!(
            config
                .pull_hook_paths(&common, &worktree, &environment)
                .expect("relative"),
            [
                worktree.join("relative-hooks/reference-transaction"),
                worktree.join("relative-hooks/pre-auto-gc")
            ]
        );
        assert!(!common.exists());
        assert!(
            !worktree.exists(),
            "path planning must not open hooks or their roots"
        );
    }

    #[test]
    fn child_fetch_refuses_external_effects_and_honors_disabling_values() {
        let root = tempfile::tempdir().expect("fixture");
        std::fs::write(root.path().join("config"), "").expect("source");
        let capture = |records: &[(&str, &str)]| {
            let mut output = String::new();
            for (key, value) in records {
                output.push_str(&format!(
                    "file:{}\0{key}\n{value}\0",
                    root.path().join("config").display()
                ));
            }
            ChildConfig::capture(output.as_bytes(), root.path()).expect("capture")
        };
        for records in [
            vec![("fetch.bundleuri", "https://example.com/bundle")],
            vec![("core.gitproxy", "proxy-program")],
            vec![("gc.recentobjectshook", "hook-program")],
            vec![("gc.repackfilterto", "/external/objects")],
            vec![("maintenance.auto", "invalid")],
        ] {
            assert!(capture(&records).prepare_pull_transport("origin").is_err());
        }
        for records in [
            vec![
                ("fetch.bundleuri", "https://example.com/bundle"),
                ("fetch.bundleuri", ""),
            ],
            vec![("gc.repackfilterto", "")],
            vec![
                ("maintenance.auto", "false"),
                ("gc.recentobjectshook", "hook-program"),
                ("gc.repackfilterto", "/external/objects"),
            ],
        ] {
            capture(&records)
                .prepare_pull_transport("origin")
                .expect("disabled effect");
        }
    }

    #[test]
    fn child_configuration_rejects_new_origins_before_source_capture() {
        let root = tempfile::tempdir().expect("fixture");
        let source = root.path().join("config");
        std::fs::write(&source, "[custom]\n value = same\n").expect("source");
        let output = format!("file:{}\0custom.value\nsame\0", source.display());
        let config = ChildConfig::capture(output.as_bytes(), root.path()).expect("capture");
        config.verify_host(output.as_bytes()).expect("unchanged");
        // The newly reported origin is deliberately not a readable regular
        // configuration file. Output comparison must reject it without capture.
        let changed = format!("file:{}\0custom.value\nsame\0", root.path().display());
        assert!(
            config
                .verify_host(changed.as_bytes())
                .expect_err("new origin")
                .to_string()
                .contains("changed during discovery")
        );
        config.revalidate().expect("original evidence retained");
    }

    #[test]
    fn child_fetch_destination_preserves_first_url_and_rewrite_order() {
        let root = tempfile::tempdir().expect("fixture");
        std::fs::write(root.path().join("config"), "").expect("source");
        let output = |records: &[&str]| {
            records
                .iter()
                .map(|record| format!("file:{}\0{record}\0", root.path().join("config").display()))
                .collect::<String>()
        };
        let first = output(&[
            "remote.origin.url\nalias:repo",
            "remote.origin.url\ngit://ignored.invalid/repo",
            "url.git://first.invalid/.insteadof\nalias:",
            "url.git://second.invalid/.insteadof\nalias:",
        ]);
        let config = ChildConfig::capture(first.as_bytes(), root.path()).expect("config");
        assert_eq!(
            config.fetch_destinations("origin").expect("fetch"),
            ["git://first.invalid/repo"]
        );
        let reordered = output(&[
            "remote.origin.url\nalias:repo",
            "remote.origin.url\ngit://ignored.invalid/repo",
            "url.git://second.invalid/.insteadof\nalias:",
            "url.git://first.invalid/.insteadof\nalias:",
        ]);
        assert!(config.verify_host(reordered.as_bytes()).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn child_configuration_rejects_equal_content_ancestor_retarget() {
        let root = tempfile::tempdir().expect("fixture");
        for directory in ["first", "second"] {
            std::fs::create_dir(root.path().join(directory)).expect("directory");
            std::fs::write(
                root.path().join(directory).join("config"),
                "[core]\n fsmonitor = false\n",
            )
            .expect("config");
        }
        let link = root.path().join("current");
        std::os::unix::fs::symlink(root.path().join("first"), &link).expect("link");
        let path = link.join("config");
        let output = format!("file:{}\0core.fsmonitor\nfalse\0", path.display());
        let config = super::ChildConfig::capture(output.as_bytes(), root.path()).expect("capture");
        config.revalidate().expect("unchanged");
        std::fs::remove_file(&link).expect("remove fixture link");
        std::os::unix::fs::symlink(root.path().join("second"), &link).expect("retarget");
        assert!(
            config
                .revalidate()
                .expect_err("identity changed")
                .to_string()
                .contains("path identity")
        );
    }

    #[cfg(unix)]
    #[test]
    fn child_configuration_rejects_identical_file_replacement() {
        let root = tempfile::tempdir().expect("fixture");
        let path = root.path().join("config");
        std::fs::write(&path, "[core]\n fsmonitor = false\n").expect("config");
        let output = format!("file:{}\0core.fsmonitor\nfalse\0", path.display());
        let config = super::ChildConfig::capture(output.as_bytes(), root.path()).expect("capture");
        let replacement = root.path().join("replacement");
        std::fs::copy(&path, &replacement).expect("identical bytes");
        std::fs::rename(replacement, &path).expect("replace");
        assert!(
            config
                .revalidate()
                .expect_err("identity changed")
                .to_string()
                .contains("path identity")
        );
    }

    use super::*;

    #[test]
    fn path_interpolation_uses_only_the_frozen_environment() {
        let values = parse_host(b"core.attributesfile\n~/rules\0").expect("config");
        let environment =
            BTreeMap::from([(OsString::from("HOME"), OsString::from("/captured-home"))]);
        assert_eq!(
            configured_path(&values, b"core.attributesfile", &environment).expect("captured home"),
            Some(PathBuf::from("/captured-home/rules")),
        );
        assert!(configured_path(&values, b"core.attributesfile", &BTreeMap::new()).is_err());
    }

    #[test]
    fn pull_diff_order_file_resolves_relative_paths_under_workdir() {
        let root = tempfile::tempdir().expect("fixture");
        let workdir = root.path().canonicalize().expect("workdir");
        let mut state = ConfigState {
            values: Values::new(),
            ordered_output: Vec::new(),
            origins: BTreeSet::new(),
            files: Vec::new(),
            identities: Vec::new(),
            auxiliary: BTreeSet::new(),
            stable_auxiliary: BTreeSet::new(),
            hooks: Vec::new(),
        };
        for (value, expected) in [
            (b"rules/order".to_vec(), workdir.join("rules/order")),
            (
                workdir
                    .join("absolute-order")
                    .to_string_lossy()
                    .as_bytes()
                    .to_vec(),
                workdir.join("absolute-order"),
            ),
        ] {
            state
                .values
                .insert(b"diff.orderfile".to_vec(), vec![Some(value)]);
            assert_eq!(
                state
                    .pull_diff_order_file(&workdir, &BTreeMap::new())
                    .expect("path"),
                Some(expected)
            );
        }
        state.values.insert(
            b"diff.orderfile".to_vec(),
            vec![Some(b"rules\0order".to_vec())],
        );
        assert!(
            state
                .pull_diff_order_file(&workdir, &BTreeMap::new())
                .is_err()
        );
    }

    #[test]
    fn host_parser_preserves_duplicate_order_implicit_empty_and_subsection_case() {
        let values = parse_host(
            b"Core.Bare\0core.bare\n\0core.bare\nfalse\0remote.Case.URL\nline1\nline2\0",
        )
        .expect("config");
        assert_eq!(
            values[b"core.bare".as_slice()],
            vec![None, Some(vec![]), Some(b"false".to_vec())]
        );
        assert_eq!(
            values[b"remote.Case.url".as_slice()],
            vec![Some(b"line1\nline2".to_vec())]
        );
        assert!(parse_host(b"core.bare\ntrue").is_err());
    }

    #[test]
    fn host_parser_preserves_non_utf8_configuration_bytes() {
        let values = parse_host(b"custom.value\n\xff\xfe\0").expect("raw config");
        assert_eq!(
            values[b"custom.value".as_slice()],
            vec![Some(vec![0xff, 0xfe])]
        );
    }

    #[test]
    fn git_boolean_matches_git_disabled_spellings() {
        for value in [b"false".as_slice(), b"0", b"off", b"no", b"", b"00", b"0k"] {
            assert!(!git_boolean(Some(value)).expect("valid boolean"));
        }
        for value in [b"true".as_slice(), b"1", b"on", b"yes", b"-2", b"1k"] {
            assert!(git_boolean(Some(value)).expect("valid boolean"));
        }
        assert!(git_boolean(None).expect("implicit true"));
        assert!(git_boolean(Some(b"maybe")).is_err());
    }
}
