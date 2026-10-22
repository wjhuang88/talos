//! Native configuration evidence, checked against the admitted host program.

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

use super::{GitToolError, repository_state::RepositoryState, snapshot};

type CheckoutCandidates = (Vec<(String, String)>, Option<String>);

type Values = BTreeMap<Vec<u8>, Vec<Option<Vec<u8>>>>;

#[path = "config_state/merge_options.rs"]
mod merge_options;

/// Receiver dependencies named by configuration. Construction never opens the
/// paths; their contents belong to the next independently admitted Read stage.
pub(super) struct ReceiverEffects {
    pub paths: BTreeSet<PathBuf>,
    pub hooks: BTreeSet<PathBuf>,
    alternates: BTreeSet<PathBuf>,
}

impl ReceiverEffects {
    /// Validate operation effects against the identity-bound, separately
    /// admitted snapshots retained by the invocation resolver.
    pub(super) fn validate_admitted(
        &self,
        auxiliary: &super::auxiliary_paths::AuxiliaryPaths,
    ) -> Result<(), GitToolError> {
        for path in &self.paths {
            let evidence = auxiliary
                .leaf_evidence(path)
                .ok_or_else(|| unsupported("Receiver auxiliary resolution evidence is missing"))?;
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
            if self.hooks.contains(path) && auxiliary.original_leaf_is_link(path) {
                return Err(unsupported(
                    "Receiver hook link execution is not supported by this plan",
                ));
            }
            if self.alternates.contains(path)
                && let snapshot::Entry::File(_, bytes) = &evidence.entry
                && !bytes.is_empty()
            {
                return Err(unsupported(
                    "Receiver object alternates require independent object-store discovery",
                ));
            }
        }
        Ok(())
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
        Self::capture_with_budget(gitdir, common, has_common, None)
    }

    pub(super) fn capture_native_pull(
        gitdir: &std::path::Path,
        common: &std::path::Path,
        has_common: bool,
        shared: &super::pull_inventory::ReferenceBudget,
    ) -> Result<Self, GitToolError> {
        Self::capture_with_budget(gitdir, common, has_common, Some(shared))
    }

    fn capture_with_budget(
        gitdir: &std::path::Path,
        common: &std::path::Path,
        has_common: bool,
        shared: Option<&super::pull_inventory::ReferenceBudget>,
    ) -> Result<Self, GitToolError> {
        let mut budget = snapshot::Budget {
            entries: 2,
            bytes: 256 * 1024,
        };
        let mut capture = |path: &std::path::Path| {
            if let Some(shared) = shared {
                shared.capture_with_budget(path, &mut budget)
            } else {
                snapshot::RegularEvidence::capture(path, &mut budget).map_err(|_| {
                    unsupported("Child startup configuration must be bounded regular files")
                })
            }
        };
        let mut files = Vec::new();
        let mut targets = BTreeSet::new();
        let common_config = common.join("config");
        let evidence = capture(&common_config)?;
        let mut layout = startup_values(&evidence.entry)?;
        files.push((common_config, evidence));
        // Git startup ignores all layout settings when repositoryFormatVersion
        // is absent; includes do not participate in this decision.
        if layout.version {
            if layout.worktree_config {
                let path = gitdir.join("config.worktree");
                let evidence = capture(&path)?;
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
    pub(super) fn filter_attribute_is_inert(&self, attribute: &[u8]) -> bool {
        filter_attribute_is_inert(&self.values, attribute)
    }

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

    pub(super) fn capture_native_pull(
        output: &[u8],
        gitdir: &std::path::Path,
        budget: &super::pull_inventory::ReferenceBudget,
    ) -> Result<Self, GitToolError> {
        budget.retain_configuration_output(output)?;
        let (_, origins) = parse_origins(output)?;
        if origins.iter().any(|path| !path.is_absolute()) {
            return Err(unsupported(
                "Native Pull configuration origin must be absolute",
            ));
        }
        Self::capture_with_budget(output, gitdir, Some(budget))
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
        Self::capture_with_budget(output, workdir, None)
    }

    fn capture_with_budget(
        output: &[u8],
        workdir: &std::path::Path,
        shared: Option<&super::pull_inventory::ReferenceBudget>,
    ) -> Result<Self, GitToolError> {
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
                let entry = if let Some(shared) = shared {
                    shared.capture(&path)?.entry
                } else {
                    snapshot::Entry::capture_regular(&path, &mut budget).map_err(|_| {
                        unsupported("Submodule configuration source cannot be captured")
                    })?
                };
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

    /// Compare admitted resolved snapshots with the configuration discovery
    /// bytes without reopening the original traversal spellings.
    pub(super) fn validate_auxiliary_snapshot(
        &self,
        auxiliary: &super::auxiliary_paths::AuxiliaryPaths,
    ) -> Result<(), GitToolError> {
        for (path, expected) in &self.files {
            if auxiliary
                .leaf_evidence(path)
                .map(|evidence| &evidence.entry)
                != Some(expected)
            {
                return Err(unsupported(
                    "Receiver configuration changed after discovery",
                ));
            }
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
    pub(super) hooks: Vec<PathBuf>,
}

// check-attr uses these same bytes for literal driver names and sentinel states.
// Refuse ambiguous configured programs until their execution is separately admitted.
fn filter_attribute_is_inert(values: &Values, attribute: &[u8]) -> bool {
    matches!(attribute, b"unset" | b"unspecified")
        && ![b"clean".as_slice(), b"smudge", b"process"]
            .iter()
            .any(|suffix| {
                let key = [b"filter.".as_slice(), attribute, b".", suffix].concat();
                values
                    .get(&key)
                    .and_then(|values| values.last())
                    .is_some_and(|value| value.as_deref().is_none_or(|value| !value.is_empty()))
            })
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
    #[cfg(windows)]
    let resolved_parents = windows_auxiliary_parents(path)?;
    #[cfg(windows)]
    let path = resolved_parents.as_path();
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

// Verbatim Windows paths do not interpret `..` during filesystem access.
// Resolve each cancelled directory through the filesystem before taking its
// parent; lexical folding would incorrectly accept missing/file ancestors.
// auxiliary_spellings separately inventories the original traversal spelling.
#[cfg(windows)]
fn windows_auxiliary_parents(path: &std::path::Path) -> Result<PathBuf, GitToolError> {
    let mut prefix = PathBuf::new();
    for part in path.components() {
        if part == std::path::Component::ParentDir {
            if !std::fs::metadata(&prefix)
                .map_err(|_| unsupported("Checkout auxiliary ancestor unavailable"))?
                .is_dir()
            {
                return Err(unsupported(
                    "Checkout auxiliary ancestor is not a directory",
                ));
            }
            prefix = prefix
                .canonicalize()
                .map_err(|_| unsupported("Checkout auxiliary ancestor cannot be resolved"))?;
            prefix.pop();
        } else {
            prefix.push(part.as_os_str());
        }
    }
    Ok(prefix)
}

// Target entries may not exist yet, so canonicalize alone cannot establish
// equality on case-insensitive or normalization-insensitive volumes. Treat
// ambiguous non-ASCII components conservatively rather than assuming byte
// equality proves disjoint effect scope.
fn potential_path_prefix(prefix: &std::path::Path, path: &std::path::Path) -> bool {
    let potentially_equal = |left: &OsStr, right: &OsStr| {
        let left = left.as_encoded_bytes();
        let right = right.as_encoded_bytes();
        left.eq_ignore_ascii_case(right)
            || !left.is_ascii()
            || !right.is_ascii()
            // The admitted resolver retains traversal spelling, including
            // Windows 8.3 aliases, whereas repository roots may use long
            // names. A tilde component is ambiguous without another metadata
            // read. Conservatively protect possible overlap; never use this
            // comparison to grant Read/Write authority.
            || (cfg!(windows) && (left.contains(&b'~') || right.contains(&b'~')))
    };
    let mut parts = path.components();
    prefix.components().all(|left| {
        let Some(right) = parts.next() else {
            return false;
        };
        // canonicalize returns verbatim Windows prefixes, while Git reports
        // ordinary drive/UNC paths. These spellings must overlap for effect
        // protection; this comparison does not grant filesystem authority.
        if let (std::path::Component::Prefix(left), std::path::Component::Prefix(right)) =
            (left, right)
        {
            use std::path::Prefix;
            match (left.kind(), right.kind()) {
                (
                    Prefix::Disk(left) | Prefix::VerbatimDisk(left),
                    Prefix::Disk(right) | Prefix::VerbatimDisk(right),
                ) => {
                    return left.eq_ignore_ascii_case(&right);
                }
                (
                    Prefix::UNC(ls, lh) | Prefix::VerbatimUNC(ls, lh),
                    Prefix::UNC(rs, rh) | Prefix::VerbatimUNC(rs, rh),
                ) => {
                    return potentially_equal(ls, rs) && potentially_equal(lh, rh);
                }
                _ => {}
            }
        }
        potentially_equal(left.as_os_str(), right.as_os_str())
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
            #[cfg(windows)]
            if *component == std::path::Component::ParentDir {
                // Keep the cancelled ancestor in spellings (inserted on the
                // previous iteration), but use a resolved path for OS lookup.
                let mut traversal = prefix.as_os_str().to_os_string();
                traversal.push(r"\..");
                prefix = windows_auxiliary_parents(std::path::Path::new(&traversal))?;
                spellings.insert(prefix.clone());
                continue;
            }
            prefix.push(component.as_os_str());
            // A Windows prefix alone (notably \\?\C:) is not the absolute
            // root. Inspect it only after appending the following RootDir.
            if matches!(component, std::path::Component::Prefix(_))
                && matches!(
                    components.get(position + 1),
                    Some(std::path::Component::RootDir)
                )
            {
                continue;
            }
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
    pub(super) fn filter_attribute_is_inert(&self, attribute: &[u8]) -> bool {
        filter_attribute_is_inert(&self.values, attribute)
    }

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

    /// Resolve integration hook candidates without reading or executing them.
    /// Fetch admission does not cover this separately admitted frontier.
    pub(super) fn pull_integration_hook_paths(
        &self,
        repository: &RepositoryState,
        environment: &BTreeMap<OsString, OsString>,
        integration: &super::pull_state::Integration,
        branch: Option<&str>,
    ) -> Result<Vec<PathBuf>, GitToolError> {
        use super::pull_state::Integration;
        let merge = matches!(integration, Integration::Merge { .. });
        let rebase = match integration {
            Integration::Rebase { mode, .. } => Some(mode),
            _ => None,
        };
        self.pull_operation_hook_paths(repository, environment, merge, rebase, branch)
    }

    /// Discover the native operation's hook frontier from pre-fetch policy only.
    /// No hook contents, fetched identities or integration outcome are needed.
    pub(super) fn prepare_native_pull(
        &mut self,
        repository: &RepositoryState,
        environment: &BTreeMap<OsString, OsString>,
        branch: Option<&str>,
    ) -> Result<Vec<PathBuf>, GitToolError> {
        let paths = self.native_pull_hook_paths(repository, environment, branch)?;
        self.auxiliary.extend(paths.iter().cloned());
        self.hooks.extend(paths.iter().cloned());
        self.hooks.sort();
        self.hooks.dedup();
        Ok(paths)
    }

    /// Recompute the complete native frontier without reading hook contents.
    pub(super) fn native_pull_hook_paths(
        &self,
        repository: &RepositoryState,
        environment: &BTreeMap<OsString, OsString>,
        branch: Option<&str>,
    ) -> Result<Vec<PathBuf>, GitToolError> {
        let mode = super::pull_state::native_rebase_mode(&self.pull_records()?, branch)?;
        let merge = mode == super::pull_state::Rebase::Off;
        let mut paths = self.pull_operation_hook_paths(
            repository,
            environment,
            merge,
            (!merge).then_some(&mode),
            branch,
        )?;
        validate_fetch_external_effects(&self.values)?;
        paths.extend(fetch_hook_paths(
            &self.values,
            &repository.common_dir,
            &repository.workdir,
            environment,
        )?);
        paths.sort();
        paths.dedup();
        Ok(paths)
    }

    fn pull_operation_hook_paths(
        &self,
        repository: &RepositoryState,
        environment: &BTreeMap<OsString, OsString>,
        merge: bool,
        rebase: Option<&super::pull_state::Rebase>,
        branch: Option<&str>,
    ) -> Result<Vec<PathBuf>, GitToolError> {
        // Current-worktree attributes cannot predict attributes introduced by
        // the fetched tree or by replayed commits. A worktree write grant does
        // not authorize configured programs selected by those new attributes.
        for (key, values) in &self.values {
            let filter = key.starts_with(b"filter.")
                && [b".clean".as_slice(), b".smudge", b".process"]
                    .iter()
                    .any(|suffix| key.ends_with(suffix));
            let driver = (merge || rebase.is_some())
                && key.starts_with(b"merge.")
                && key.ends_with(b".driver");
            if (filter || driver)
                && values
                    .last()
                    .is_some_and(|value| value.as_deref().is_none_or(|value| !value.is_empty()))
            {
                return Err(unsupported(if filter {
                    "Pull integration configured filter requires separate execution authorization"
                } else {
                    "Pull integration custom merge driver requires separate execution authorization"
                }));
            }
        }
        if rebase == Some(&super::pull_state::Rebase::Interactive) {
            return Err(unsupported(
                "Interactive Pull rebase requires separate sequence editor execution authorization",
            ));
        }
        self.validate_pull_integration_signing_mode(merge, rebase.is_some())?;
        // The sequencer expands this format into its executable todo list.
        // In particular, %nexec can launch a command even for noninteractive
        // rebase. Worktree scope does not authorize those commands. Empty
        // formats retain Git's default; custom formats need effect discovery.
        if rebase.is_some()
            && self
                .values
                .get(b"rebase.instructionformat".as_slice())
                .and_then(|values| values.last())
                .is_some_and(|value| value.as_deref().is_none_or(|value| !value.is_empty()))
        {
            return Err(unsupported(
                "Custom rebase instruction format requires separate executable-todo effect authorization",
            ));
        }
        if merge {
            self.validate_pull_merge_strategy()?;
            self.validate_pull_merge_options(branch)?;
        }
        let directory = configured_path(&self.values, b"core.hookspath", environment)?
            .unwrap_or_else(|| repository.common_dir.join("hooks"));
        let directory = if directory.is_absolute() {
            directory
        } else {
            repository.workdir.join(directory)
        };
        let names: &[&str] = match (merge, rebase.is_some()) {
            (true, _) => &[
                "pre-merge-commit",
                "prepare-commit-msg",
                "commit-msg",
                "post-merge",
                "post-checkout",
                "post-index-change",
                "reference-transaction",
                "pre-auto-gc",
            ],
            (_, true) => &[
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
            _ => &["reference-transaction", "post-index-change"],
        };
        let mut paths: Vec<_> = names.iter().map(|name| directory.join(name)).collect();
        // Plain rebase honors rebase.backend=apply and invokes git-am.
        // --rebase-merges selects the merge backend regardless of this setting.
        if rebase == Some(&super::pull_state::Rebase::On)
            && self
                .values
                .get(b"rebase.backend".as_slice())
                .and_then(|values| values.last())
                .and_then(|value| value.as_deref())
                == Some(b"apply")
        {
            paths.extend(
                ["applypatch-msg", "pre-applypatch", "post-applypatch"]
                    .iter()
                    .map(|name| directory.join(name)),
            );
        }
        Ok(paths)
    }

    // These booleans can start a signing/verifier program even when no hook
    // exists. Worktree scope approval does not authorize that execution.
    // Branch merge options and other external effects need their own checks;
    // this is deliberately not a complete integration execution validator.
    fn validate_pull_integration_signing_mode(
        &self,
        merge: bool,
        rebase: bool,
    ) -> Result<(), GitToolError> {
        let keys: &[&[u8]] = match (merge, rebase) {
            (true, _) => &[b"commit.gpgsign", b"merge.verifysignatures"],
            (_, true) => &[b"commit.gpgsign"],
            _ => &[],
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

    /// Integration can rewrite dynamic paths: retained executable/configuration
    /// sources must therefore live outside its worktree mutation range.
    pub(super) fn validate_pull_integration_sources(
        &self,
        repository: &RepositoryState,
        hooks: &[PathBuf],
    ) -> Result<(), GitToolError> {
        for path in self
            .files
            .iter()
            .map(|(path, _)| path)
            .chain(self.stable_auxiliary.iter())
            .chain(hooks.iter())
        {
            for resolved in auxiliary_spellings(path)? {
                if resolved != repository.workdir
                    && potential_path_prefix(&repository.workdir, &resolved)
                    && ![&repository.git_dir, &repository.common_dir]
                        .iter()
                        .any(|root| resolved.starts_with(root))
                {
                    return Err(unsupported(
                        "Pull integration may rewrite an effective configuration or hook source",
                    ));
                }
            }
        }
        Ok(())
    }

    /// Capture Pull hooks only after their exact paths were admitted.
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
        auxiliary: &super::auxiliary_paths::AuxiliaryPaths,
    ) -> Result<(), GitToolError> {
        for path in self
            .files
            .iter()
            .map(|(path, _)| path)
            .chain(self.stable_auxiliary.iter())
            .chain(self.hooks.iter())
            .chain(child_sources.iter())
        {
            for resolved in auxiliary
                .related_paths_for(path)
                .ok_or_else(|| unsupported("Checkout auxiliary resolution evidence missing"))?
            {
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
        auxiliary: &super::auxiliary_paths::AuxiliaryPaths,
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
            for resolved in auxiliary
                .related_paths_for(hook)
                .ok_or_else(|| unsupported("Checkout auxiliary resolution evidence missing"))?
            {
                for (path, (mode, _)) in &target.entries {
                    let target_path = repository.workdir.join(path);
                    let same = potential_path_prefix(&target_path, &resolved)
                        && potential_path_prefix(&resolved, &target_path);
                    // Git for Windows does not use the tree executable bit
                    // to decide whether a checked-out hook can run. Match the
                    // conservative existing-file hook admission rule above.
                    let executable = *mode & 0o111 != 0
                        || (cfg!(windows) && matches!(*mode, 0o100644 | 0o100755));
                    if (same && (executable || *mode == 0o120000))
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
            for resolved in auxiliary
                .related_paths_for(&path)
                .ok_or_else(|| unsupported("Checkout auxiliary resolution evidence missing"))?
            {
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

    /// Legacy leaf-capture characterization. Runtime callers use AuxiliaryPaths
    /// so component identity and exact target admission remain bound together.
    #[cfg(test)]
    pub fn capture_auxiliary(
        &self,
    ) -> Result<Vec<(PathBuf, snapshot::RegularEvidence)>, GitToolError> {
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
                // Retain the admitted traversal spelling as the evidence key,
                // but do not pass verbatim `..` to Win32 file APIs. Resolve
                // cancelled ancestors through the filesystem, as in overlap
                // inspection, rather than lexically dropping them.
                #[cfg(windows)]
                let resolved = windows_auxiliary_parents(path)?;
                #[cfg(windows)]
                let capture_path = resolved.as_path();
                #[cfg(not(windows))]
                let capture_path = path.as_path();
                let evidence = snapshot::RegularEvidence::capture(capture_path, &mut budget)
                    .map_err(|_| unsupported("Git auxiliary file cannot be captured"))?;
                Ok((path.clone(), evidence))
            })
            .collect()
    }

    /// Reuse retained leaf identities and bytes rather than accepting a new
    /// snapshot with identical contents after an auxiliary file was replaced.
    /// Traversal-chain admission and identity are a separate caller obligation.
    #[cfg(test)]
    pub fn revalidate_auxiliary(
        &self,
        evidence: &[(PathBuf, snapshot::RegularEvidence)],
    ) -> Result<(), GitToolError> {
        if !self
            .auxiliary
            .iter()
            .eq(evidence.iter().map(|(path, _)| path))
        {
            return Err(unsupported("Git auxiliary source set changed"));
        }
        for (path, expected) in evidence {
            #[cfg(windows)]
            let resolved = windows_auxiliary_parents(path)?;
            #[cfg(windows)]
            let path = &resolved;
            expected
                .revalidate(path)
                .map_err(|_| unsupported("Git auxiliary sources changed during inspection"))?;
        }
        Ok(())
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
                if path.has_root()
                    || path
                        .components()
                        .any(|part| matches!(part, std::path::Component::Prefix(_)))
                {
                    return Err(unsupported("Include path has an ambiguous root or prefix"));
                }
                let origin = std::str::from_utf8(&origin[b"file:".len()..])
                    .map_err(|_| unsupported("Invalid include origin"))?;
                let origin = PathBuf::from(origin);
                let origin = if origin.is_absolute() {
                    origin
                } else {
                    workdir.join(origin)
                };
                let parent = origin
                    .parent()
                    .ok_or_else(|| unsupported("Include origin has no parent"))?;
                // PathBuf::push normalizes `..` on Windows verbatim roots.
                // Keep Git's spelling so Sources can bind cancelled ancestors.
                let mut spelling = parent.as_os_str().to_os_string();
                let parent_text = parent.as_os_str().as_encoded_bytes();
                if !(parent_text.ends_with(b"/") || cfg!(windows) && parent_text.ends_with(b"\\")) {
                    spelling.push(std::path::MAIN_SEPARATOR_STR);
                }
                // Verbatim Windows paths only recognize backslashes during
                // component traversal; Git accepts either separator.
                #[cfg(windows)]
                spelling.push(path.as_os_str().to_string_lossy().replace('/', "\\"));
                #[cfg(not(windows))]
                spelling.push(path.as_os_str());
                result.insert(PathBuf::from(spelling));
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
    use super::*;

    include!("config_state_tests/pull_signing.rs");
    include!("config_state_tests/pull_sources.rs");
    include!("config_state_tests/child_layout.rs");
    include!("config_state_tests/child_configuration.rs");
    include!("config_state_tests/checkout.rs");
    include!("config_state_tests/receiver_push.rs");
    include!("config_state_tests/parsing.rs");
    include!("config_state_tests/auxiliary.rs");
    include!("config_state_tests/pull.rs");
}
