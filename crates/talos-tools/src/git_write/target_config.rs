//! Pure parsing for a separately admitted target-branch configuration source.
//! Parsing is pure; source capture requires exact admission for each frontier.

use super::GitToolError;

#[derive(Clone, Eq, PartialEq)]
pub(super) struct Record {
    pub key: Vec<u8>,
    pub value: Option<Vec<u8>>,
}

impl std::fmt::Debug for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Record")
            .field("key", &"<redacted>")
            .field("value", &self.value.as_ref().map(|_| "<redacted>"))
            .finish()
    }
}

pub(super) fn parse_records(bytes: &[u8]) -> Result<Vec<Record>, GitToolError> {
    if bytes.len() > 4 * 1024 * 1024 {
        return Err(invalid_config());
    }
    std::panic::catch_unwind(|| parse_records_inner(bytes)).map_err(|_| invalid_config())?
}

/// Predict local tracking setters in their physical section order. This does
/// not expand includes, write bytes, or grant authority to the resulting values.
fn tracking_overlay(bytes: &[u8], writes: &[Record]) -> Result<Vec<Record>, GitToolError> {
    std::panic::catch_unwind(|| tracking_overlay_inner(bytes, writes))
        .map_err(|_| invalid_config())?
}

enum ConfigItem {
    Section(Vec<u8>),
    Value(Record),
    Comment,
}

fn tracking_overlay_inner(bytes: &[u8], writes: &[Record]) -> Result<Vec<Record>, GitToolError> {
    use gix::bstr::ByteSlice;
    use gix::config::parse::{EventRef, Events};
    let records = parse_records(bytes)?;
    if writes.is_empty() {
        return Ok(records);
    }
    let mut write_bytes = 4 * 1024 * 1024usize;
    if writes.len() > 4096 {
        return Err(invalid_config());
    }
    for record in writes {
        write_bytes = write_bytes
            .checked_sub(record.key.len())
            .and_then(|budget| budget.checked_sub(record.value.as_ref().map_or(0, Vec::len)))
            .ok_or_else(invalid_config)?;
    }
    let mut records = records.into_iter();
    let events = Events::from_bytes(bytes, None).map_err(|_| invalid_config())?;
    let mut items = Vec::new();
    let mut entry_line = false;
    for event in events.frontmatter() {
        match event {
            EventRef::Value(_) | EventRef::ValueDone(_) => {
                items.push(ConfigItem::Value(
                    records.next().ok_or_else(invalid_config)?,
                ));
                entry_line = true;
            }
            EventRef::Comment { .. } if !entry_line => items.push(ConfigItem::Comment),
            EventRef::Newline(_) => entry_line = false,
            _ => {}
        }
        if items.len() > 16384 {
            return Err(invalid_config());
        }
    }
    for section in events.sections() {
        let EventRef::SectionHeader {
            name,
            separator,
            subsection_name,
        } = section.header()
        else {
            return Err(invalid_config());
        };
        let mut prefix = name.to_vec();
        prefix.make_ascii_lowercase();
        if let Some(subsection) = subsection_name {
            prefix.push(b'.');
            let mut subsection = subsection.to_vec();
            if separator.is_some_and(|separator| separator.as_bytes() == b".") {
                subsection.make_ascii_lowercase();
            }
            prefix.extend(subsection);
        }
        items.push(ConfigItem::Section(prefix));
        if items.len() > 16384 {
            return Err(invalid_config());
        }
        let mut entry_line = false;
        for event in section.body() {
            match event {
                EventRef::Value(_) | EventRef::ValueDone(_) => {
                    items.push(ConfigItem::Value(
                        records.next().ok_or_else(invalid_config)?,
                    ));
                    entry_line = true;
                }
                // Git's ENTRY offset includes its inline comment; removing or
                // replacing that entry removes the comment as well.
                EventRef::Comment { .. } if !entry_line => items.push(ConfigItem::Comment),
                EventRef::Newline(_) => entry_line = false,
                _ => {}
            }
            if items.len() > 16384 {
                return Err(invalid_config());
            }
        }
    }
    if records.next().is_some() {
        return Err(invalid_config());
    }
    let remote = writes.first().ok_or_else(invalid_config)?;
    let prefix = remote
        .key
        .strip_suffix(b".remote")
        .ok_or_else(invalid_config)?;
    if !prefix.starts_with(b"branch.") || remote.value.is_none() {
        return Err(invalid_config());
    }
    let merge_key = [prefix, b".merge"].concat();
    let rebase_key = [prefix, b".rebase"].concat();
    if writes.iter().skip(1).any(|record| {
        record.value.is_none() || (record.key != merge_key && record.key != rebase_key)
    }) {
        return Err(invalid_config());
    }
    // Git's branch installer checks setters for negative results. Positive
    // CONFIG_NOTHING_SET on duplicate remote/rebase keeps those values intact.
    set_tracking_value(&mut items, prefix, remote, false);
    let matches = items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            matches!(item, ConfigItem::Value(record) if record.key == merge_key).then_some(index)
        })
        .collect::<Vec<_>>();
    if let [index] = matches.as_slice() {
        // Native unset removes now-empty adjacent matching sections only when
        // no comments or other keys surround the removed entry.
        let mut begin = *index;
        let mut section_seen = false;
        let mut removable = true;
        while begin > 0 {
            match &items[begin - 1] {
                ConfigItem::Comment => {
                    removable = false;
                    break;
                }
                ConfigItem::Value(_) => {
                    removable = section_seen;
                    break;
                }
                ConfigItem::Section(section) if section == prefix => section_seen = true,
                ConfigItem::Section(_) => break,
            }
            begin -= 1;
        }
        let mut end = *index + 1;
        while removable && end < items.len() {
            match &items[end] {
                ConfigItem::Section(section) if section == prefix => end += 1,
                ConfigItem::Section(_) => break,
                ConfigItem::Comment | ConfigItem::Value(_) => removable = false,
            }
        }
        if removable {
            items.drain(begin..end);
        } else {
            items.remove(*index);
        }
    }
    // Duplicate merge unset returns positive 5, so old entries remain and
    // each new merge is appended using CONFIG_REGEX_NONE.
    for record in writes.iter().skip(1) {
        set_tracking_value(&mut items, prefix, record, record.key == merge_key);
    }
    let mut result = Vec::new();
    let mut budget = 4 * 1024 * 1024usize;
    for item in items {
        if let ConfigItem::Value(record) = item {
            budget = budget
                .checked_sub(record.key.len() + record.value.as_ref().map_or(0, Vec::len))
                .ok_or_else(invalid_config)?;
            if result.len() >= 4096 {
                return Err(invalid_config());
            }
            result.push(record);
        }
    }
    Ok(result)
}

fn set_tracking_value(items: &mut Vec<ConfigItem>, prefix: &[u8], record: &Record, append: bool) {
    if !append {
        let matches = items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| {
                matches!(item, ConfigItem::Value(existing) if existing.key == record.key)
                    .then_some(index)
            })
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [index] => {
                items[*index] = ConfigItem::Value(record.clone());
                return;
            }
            [] => {}
            _ => return,
        }
    }
    let section = items
        .iter()
        .rposition(|item| matches!(item, ConfigItem::Section(name) if name == prefix));
    let position = if let Some(section) = section {
        let end = items
            .iter()
            .enumerate()
            .skip(section + 1)
            .find_map(|(index, item)| matches!(item, ConfigItem::Section(_)).then_some(index))
            .unwrap_or(items.len());
        items[section + 1..end]
            .iter()
            .rposition(|item| matches!(item, ConfigItem::Value(_)))
            .map_or(section + 1, |offset| section + 2 + offset)
    } else {
        items.push(ConfigItem::Section(prefix.to_vec()));
        items.len()
    };
    items.insert(position, ConfigItem::Value(record.clone()));
}

/// Compute a lexical Read frontier without existence checks or include traversal.
/// With no target branch, retain every declared dependency. Other condition kinds
/// remain potential dependencies: deciding them requires separately admitted context.
pub(super) fn declared_paths(
    bytes: &[u8],
    origin: &std::path::Path,
    environment: &std::collections::BTreeMap<std::ffi::OsString, std::ffi::OsString>,
    target_branch: Option<&[u8]>,
) -> Result<Vec<std::path::PathBuf>, GitToolError> {
    let mut paths = Vec::new();
    for record in parse_records(bytes)? {
        if let Some(path) = include_path(&record, origin, environment, target_branch)? {
            paths.push(path);
        }
    }
    Ok(paths)
}

fn include_path(
    record: &Record,
    origin: &std::path::Path,
    environment: &std::collections::BTreeMap<std::ffi::OsString, std::ffi::OsString>,
    target_branch: Option<&[u8]>,
) -> Result<Option<std::path::PathBuf>, GitToolError> {
    use gix::bstr::ByteSlice;
    let condition = if record.key == b"include.path" {
        None
    } else if let Some(condition) = record
        .key
        .strip_prefix(b"includeif.")
        .and_then(|key| key.strip_suffix(b".path"))
    {
        Some(condition)
    } else {
        return Ok(None);
    };
    if let (Some(pattern), Some(branch)) = (
        condition.and_then(|condition| condition.strip_prefix(b"onbranch:")),
        target_branch,
    ) {
        // Git matches the short local branch name with pathname wildmatch.
        // A trailing slash denotes every branch below that prefix.
        if pattern.len() > 4096 || branch.len() > 4096 {
            return Err(invalid_config());
        }
        let mut pattern = pattern.to_vec();
        if pattern.ends_with(b"/") {
            pattern.extend_from_slice(b"**");
        }
        // The dependency matcher returns false (and logs the pattern) on
        // its recursion guard. Refuse before calling it: every recursive
        // descent consumes a star; count even escaped/bracketed stars.
        if pattern.iter().filter(|byte| **byte == b'*').count() >= 32 {
            return Err(invalid_config());
        }
        let matches = std::panic::catch_unwind(|| {
            gix::glob::wildmatch(
                pattern.as_bstr(),
                branch.as_bstr(),
                gix::glob::wildmatch::Mode::NO_MATCH_SLASH_LITERAL,
            )
        })
        .map_err(|_| invalid_config())?;
        if !matches {
            return Ok(None);
        }
    }
    if let Some(path) =
        super::config_state::configured_include_path_value(&record.value, environment)?
    {
        Ok(Some(if path.is_absolute() {
            path
        } else {
            origin.parent().ok_or_else(invalid_config)?.join(path)
        }))
    } else {
        Ok(None)
    }
}

fn bounded_match(pattern: &[u8], value: &[u8]) -> Result<bool, GitToolError> {
    bounded_match_mode(pattern, value, false)
}

fn bounded_match_mode(
    pattern: &[u8],
    value: &[u8],
    ignore_case: bool,
) -> Result<bool, GitToolError> {
    use gix::bstr::ByteSlice;
    if pattern.len() > 4096
        || value.len() > 4096
        || pattern.iter().filter(|byte| **byte == b'*').count() >= 32
    {
        return Err(invalid_config());
    }
    std::panic::catch_unwind(|| {
        gix::glob::wildmatch(
            pattern.as_bstr(),
            value.as_bstr(),
            gix::glob::wildmatch::Mode::NO_MATCH_SLASH_LITERAL
                | if ignore_case {
                    gix::glob::wildmatch::Mode::IGNORE_CASE
                } else {
                    gix::glob::wildmatch::Mode::empty()
                },
        )
    })
    .map_err(|_| invalid_config())
}

/// Match only identity-bound, non-symlink paths. This function performs no IO.
fn gitdir_matches(
    pattern: &[u8],
    origin: &std::path::Path,
    gitdir: &std::path::Path,
    environment: &std::collections::BTreeMap<std::ffi::OsString, std::ffi::OsString>,
    ignore_case: bool,
) -> Result<bool, GitToolError> {
    if pattern.len() > 4096 || pattern.contains(&0) {
        return Err(invalid_config());
    }
    let mut pattern = pattern.to_vec();
    if pattern.starts_with(b"~/") {
        let home = environment
            .get(std::ffi::OsStr::new("HOME"))
            .ok_or_else(invalid_config)?;
        let home = std::path::Path::new(home);
        if !home.is_absolute() || home.as_os_str().len() > 4096 {
            return Err(invalid_config());
        }
        let mut expanded = home
            .to_str()
            .ok_or_else(invalid_config)?
            .as_bytes()
            .to_vec();
        expanded.extend_from_slice(&pattern[1..]);
        pattern = expanded;
    } else if pattern.starts_with(b"~") || pattern.starts_with(b"%(") {
        return Err(invalid_config());
    }
    let mut prefix = 0;
    if pattern.starts_with(b"./") {
        // Ancestor capture excludes links and parent traversal. Remove purely
        // lexical `.`/duplicate separators as native realpath does here.
        let origin: std::path::PathBuf = origin.components().collect();
        let parent = origin.parent().ok_or_else(invalid_config)?;
        let mut expanded = parent
            .to_str()
            .ok_or_else(invalid_config)?
            .as_bytes()
            .to_vec();
        if !expanded.ends_with(b"/") {
            expanded.push(b'/');
        }
        prefix = expanded.len();
        expanded.extend_from_slice(&pattern[2..]);
        pattern = expanded;
    } else if !std::path::Path::new(std::str::from_utf8(&pattern).map_err(|_| invalid_config())?)
        .is_absolute()
    {
        pattern.splice(..0, b"**/".iter().copied());
    }
    if pattern.ends_with(b"/") {
        pattern.extend_from_slice(b"**");
    }
    let gitdir: std::path::PathBuf = gitdir.components().collect();
    let text = gitdir.to_str().ok_or_else(invalid_config)?.as_bytes();
    if pattern.len() > 4096 || text.len() > 4096 {
        return Err(invalid_config());
    }
    if prefix > 0
        && (text.len() < prefix
            || if ignore_case {
                !pattern[..prefix].eq_ignore_ascii_case(&text[..prefix])
            } else {
                pattern[..prefix] != text[..prefix]
            })
    {
        return Ok(false);
    }
    bounded_match_mode(&pattern[prefix..], &text[prefix..], ignore_case)
}

/// Bounded potential target sources. This discovers Read dependencies, not the
/// effective ordered config: conditional/hasconfig semantics are a separate gate.
pub(super) struct Sources {
    branch: Option<Vec<u8>>,
    pending: std::collections::BTreeSet<std::path::PathBuf>,
    pending_directories: std::collections::BTreeMap<std::path::PathBuf, bool>,
    files: std::collections::BTreeMap<std::path::PathBuf, super::snapshot::RegularEvidence>,
    ancestors:
        std::collections::BTreeMap<std::path::PathBuf, Option<super::snapshot::PathIdentity>>,
    budget: super::snapshot::Budget,
    path_bytes: usize,
    requests: std::collections::BTreeSet<std::path::PathBuf>,
    tracking: Option<(std::path::PathBuf, Vec<Record>)>,
}

type ExpandedRecord = (std::path::PathBuf, Record);

struct Expansion {
    gitdir: Option<std::path::PathBuf>,
    records: Vec<ExpandedRecord>,
    payload_bytes: usize,
    parsed_bytes: usize,
    visits: usize,
    remaining_records: usize,
    match_bytes: usize,
    hasconfig: bool,
    mode: ExpansionMode,
    urls: Vec<Vec<u8>>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ExpansionMode {
    Discover,
    ScanUrls,
    Evaluate,
}

impl Sources {
    pub(super) fn new(
        branch: Option<&[u8]>,
        roots: impl IntoIterator<Item = std::path::PathBuf>,
    ) -> Result<Self, GitToolError> {
        if branch.is_some_and(|branch| branch.len() > 4096) {
            return Err(invalid_config());
        }
        let mut result = Self {
            branch: branch.map(<[u8]>::to_vec),
            pending: Default::default(),
            pending_directories: Default::default(),
            files: Default::default(),
            ancestors: Default::default(),
            budget: super::snapshot::Budget {
                entries: 4096,
                bytes: 4 * 1024 * 1024,
            },
            path_bytes: 4 * 1024 * 1024,
            requests: Default::default(),
            tracking: None,
        };
        result.add(roots)?;
        Ok(result)
    }

    pub(super) fn add(
        &mut self,
        paths: impl IntoIterator<Item = std::path::PathBuf>,
    ) -> Result<(), GitToolError> {
        for path in paths {
            self.request_path(&path)?;
            if !self.files.contains_key(&path) && !self.pending.contains(&path) {
                if self.files.len() + self.pending.len() >= 4096 {
                    return Err(invalid_config());
                }
                self.pending.insert(path);
            }
        }
        Ok(())
    }

    fn request_path(&mut self, path: &std::path::Path) -> Result<(), GitToolError> {
        // Retain even directories cancelled by `..`: Git traverses them before
        // opening the source. Every such ancestor needs its own admission and
        // identity; lexical normalization alone cannot prove it is safe.
        if path.as_os_str().len() > 4096 || !path.is_absolute() {
            return Err(invalid_config());
        }
        for dependency in path.ancestors() {
            if !self.requests.contains(dependency) {
                if self.requests.len() >= 4096 {
                    return Err(invalid_config());
                }
                self.path_bytes = self
                    .path_bytes
                    .checked_sub(dependency.as_os_str().len())
                    .ok_or_else(invalid_config)?;
                self.requests.insert(dependency.to_path_buf());
            }
        }
        Ok(())
    }

    /// Request only a directory identity and its ancestors, never its contents.
    pub(super) fn add_directory_context(
        &mut self,
        path: &std::path::Path,
    ) -> Result<(), GitToolError> {
        self.request_path(path)?;
        if self.ancestors.get(path).is_some_and(Option::is_none) {
            return Err(invalid_config());
        }
        if !self.ancestors.contains_key(path) {
            self.pending_directories.insert(path.to_path_buf(), true);
        }
        Ok(())
    }

    /// Admit an auxiliary parent identity, retaining absence when it is missing.
    /// Unlike gitdir context, a hook directory need not exist.
    pub(super) fn add_optional_directory_context(
        &mut self,
        path: &std::path::Path,
    ) -> Result<(), GitToolError> {
        self.request_path(path)?;
        if !self.ancestors.contains_key(path) {
            self.pending_directories
                .entry(path.to_path_buf())
                .or_insert(false);
        }
        Ok(())
    }

    pub(super) fn paths(&self) -> std::collections::BTreeSet<std::path::PathBuf> {
        self.requests.clone()
    }

    pub(super) fn complete(&self) -> bool {
        self.pending.is_empty() && self.pending_directories.is_empty()
    }

    /// Config sources whose original bytes or absence have exact admission.
    /// Ancestor directories are evidence dependencies, not writable inputs.
    pub(super) fn captured_paths(&self) -> impl Iterator<Item = &std::path::PathBuf> {
        self.files.keys()
    }

    /// Overlay only an already admitted local config source. Source evidence
    /// remains the original bytes and is still revalidated before execution.
    pub(super) fn predict_tracking(
        &mut self,
        path: &std::path::Path,
        writes: &[Record],
    ) -> Result<(), GitToolError> {
        let evidence = self.files.get(path).ok_or_else(invalid_config)?;
        let bytes = match &evidence.entry {
            super::snapshot::Entry::Missing => b"".as_slice(),
            super::snapshot::Entry::File(_, bytes) => bytes,
            _ => return Err(invalid_config()),
        };
        self.tracking = Some((path.to_path_buf(), tracking_overlay(bytes, writes)?));
        Ok(())
    }

    pub(super) fn advance(
        &mut self,
        admitted: impl Fn(&std::path::Path) -> bool,
        environment: &std::collections::BTreeMap<std::ffi::OsString, std::ffi::OsString>,
    ) -> Result<(), GitToolError> {
        let paths = self.paths();
        // Check the whole frontier before any metadata lookup or file read.
        if paths.iter().any(|path| !admitted(path)) {
            return Err(invalid_config());
        }
        self.revalidate()?;
        let directories = self
            .pending_directories
            .iter()
            .map(|(path, required)| (path.clone(), *required))
            .collect::<Vec<_>>();
        for (path, required) in directories {
            for directory in path.ancestors().collect::<Vec<_>>().into_iter().rev() {
                let identity = directory_identity(directory)?;
                if required && identity.is_none() {
                    return Err(invalid_config());
                }
                if let Some(expected) = self.ancestors.get(directory) {
                    if expected != &identity {
                        return Err(invalid_config());
                    }
                } else {
                    self.ancestors.insert(directory.to_path_buf(), identity);
                }
            }
            self.pending_directories.remove(&path);
        }
        let pending = self.pending.iter().cloned().collect::<Vec<_>>();
        for path in &pending {
            let parents = path.ancestors().skip(1).collect::<Vec<_>>();
            for parent in parents.into_iter().rev() {
                let identity = directory_identity(parent)?;
                if let Some(expected) = self.ancestors.get(parent) {
                    if expected != &identity {
                        return Err(invalid_config());
                    }
                } else {
                    self.ancestors.insert(parent.to_path_buf(), identity);
                }
            }
            let evidence = super::snapshot::RegularEvidence::capture(path, &mut self.budget)
                .map_err(|_| invalid_config())?;
            let includes = match &evidence.entry {
                super::snapshot::Entry::File(_, bytes) => {
                    declared_paths(bytes, path, environment, self.branch.as_deref())?
                }
                super::snapshot::Entry::Missing => Vec::new(),
                _ => return Err(invalid_config()),
            };
            self.pending.remove(path);
            self.files.insert(path.clone(), evidence);
            self.add(includes)?;
        }
        self.revalidate()
    }

    /// Repository format discovery does not traverse includes or global config.
    pub(super) fn worktree_config_enabled(
        &self,
        repository_config: &std::path::Path,
    ) -> Result<bool, GitToolError> {
        let evidence = self
            .files
            .get(repository_config)
            .ok_or_else(invalid_config)?;
        let bytes = match &evidence.entry {
            super::snapshot::Entry::Missing => return Ok(false),
            super::snapshot::Entry::File(_, bytes) => bytes,
            _ => return Err(invalid_config()),
        };
        let mut enabled = false;
        for record in parse_records(bytes)? {
            if record.key == b"extensions.worktreeconfig" {
                enabled = super::config_state::git_boolean(record.value.as_deref())?;
            }
        }
        Ok(enabled)
    }

    /// Expand already-captured sources only. No filesystem calls belong here.
    /// Root ordering is supplied by the caller, not inferred from the file set.
    pub(super) fn ordered_records(
        &self,
        roots: &[std::path::PathBuf],
        environment: &std::collections::BTreeMap<std::ffi::OsString, std::ffi::OsString>,
        gitdir: Option<&std::path::Path>,
    ) -> Result<Vec<ExpandedRecord>, GitToolError> {
        if !self.complete() {
            return Err(invalid_config());
        }
        // Capture has rejected symlinks in every ancestor. Thus this admitted
        // absolute spelling also supplies the canonical matching context.
        if let Some(gitdir) = gitdir
            && !self.ancestors.get(gitdir).is_some_and(Option::is_some)
        {
            return Err(invalid_config());
        }
        let mut expansion = Expansion {
            gitdir: gitdir.map(std::path::Path::to_path_buf),
            records: Vec::new(),
            payload_bytes: 4 * 1024 * 1024,
            parsed_bytes: 4 * 1024 * 1024,
            visits: 4096,
            remaining_records: 4096,
            match_bytes: 4 * 1024 * 1024,
            hasconfig: false,
            mode: ExpansionMode::Discover,
            urls: Vec::new(),
        };
        for root in roots {
            self.expand(root, 0, false, environment, &mut expansion)?;
        }
        if expansion.hasconfig {
            expansion.records.clear();
            expansion.mode = ExpansionMode::ScanUrls;
            for root in roots {
                self.expand(root, 0, false, environment, &mut expansion)?;
            }
            expansion.mode = ExpansionMode::Evaluate;
            for root in roots {
                self.expand(root, 0, false, environment, &mut expansion)?;
            }
        }
        Ok(expansion.records)
    }

    fn expand(
        &self,
        path: &std::path::Path,
        depth: usize,
        forbid_url: bool,
        environment: &std::collections::BTreeMap<std::ffi::OsString, std::ffi::OsString>,
        expansion: &mut Expansion,
    ) -> Result<(), GitToolError> {
        expansion.visits = expansion.visits.checked_sub(1).ok_or_else(invalid_config)?;
        let evidence = self.files.get(path).ok_or_else(invalid_config)?;
        let overlay = self.tracking.as_ref().filter(|(origin, _)| origin == path);
        let bytes = match &evidence.entry {
            super::snapshot::Entry::Missing if overlay.is_some() => b"".as_slice(),
            super::snapshot::Entry::Missing => return Ok(()),
            super::snapshot::Entry::File(_, bytes) => bytes,
            _ => return Err(invalid_config()),
        };
        // Native Git counts only includes it can actually open. A missing file
        // at this depth was already handled above using bound absence evidence.
        if depth > 10 {
            return Err(invalid_config());
        }
        expansion.parsed_bytes = expansion
            .parsed_bytes
            .checked_sub(bytes.len())
            .ok_or_else(invalid_config)?;
        let records = if let Some((_, records)) = overlay {
            let payload = records
                .iter()
                .try_fold(0usize, |total, record| {
                    total
                        .checked_add(record.key.len())?
                        .checked_add(record.value.as_ref().map_or(0, Vec::len))
                })
                .ok_or_else(invalid_config)?;
            expansion.parsed_bytes = expansion
                .parsed_bytes
                .checked_sub(payload)
                .ok_or_else(invalid_config)?;
            records.clone()
        } else {
            parse_records(bytes)?
        };
        for record in records {
            expansion.remaining_records = expansion
                .remaining_records
                .checked_sub(1)
                .ok_or_else(invalid_config)?;
            let remote_url = record
                .key
                .strip_prefix(b"remote.")
                .and_then(|key| key.strip_suffix(b".url"))
                .is_some();
            if expansion.mode == ExpansionMode::ScanUrls && remote_url {
                if forbid_url {
                    return Err(GitToolError::InvalidInput("Remote URL is forbidden in a conditional include during target configuration lookahead".into()));
                }
                let url = record.value.as_ref().ok_or_else(invalid_config)?;
                expansion.payload_bytes = expansion
                    .payload_bytes
                    .checked_sub(url.len())
                    .ok_or_else(invalid_config)?;
                if expansion.urls.len() >= 4096 {
                    return Err(invalid_config());
                }
                expansion.urls.push(url.clone());
            }
            // Git evaluates a conditional section before checking whether its
            // key is `path`. Even another key can trigger remote-URL lookahead
            // and its conditional-source URL prohibition.
            if record.key.strip_prefix(b"includeif.").is_some_and(|key| {
                key.iter()
                    .rposition(|byte| *byte == b'.')
                    .is_some_and(|end| key[..end].starts_with(b"hasconfig:remote.*.url:"))
            }) {
                expansion.hasconfig = true;
            }
            let mut conditional_include = false;
            let include = if let Some(condition) = record
                .key
                .strip_prefix(b"includeif.")
                .and_then(|key| key.strip_suffix(b".path"))
            {
                conditional_include = true;
                if let Some(pattern) = condition
                    .strip_prefix(b"gitdir:")
                    .or_else(|| condition.strip_prefix(b"gitdir/i:"))
                {
                    let gitdir = expansion.gitdir.as_ref().ok_or_else(invalid_config)?;
                    if gitdir_matches(
                        pattern,
                        path,
                        gitdir,
                        environment,
                        condition.starts_with(b"gitdir/i:"),
                    )? {
                        include_path(&record, path, environment, None)?
                    } else {
                        None
                    }
                } else if let Some(pattern) = condition.strip_prefix(b"hasconfig:remote.*.url:") {
                    expansion.hasconfig = true;
                    let enabled = match expansion.mode {
                        ExpansionMode::Discover => false,
                        ExpansionMode::ScanUrls => true,
                        ExpansionMode::Evaluate => {
                            let mut matched = false;
                            for url in &expansion.urls {
                                // Bound the aggregate lookahead comparisons,
                                // not only each pattern and URL independently.
                                expansion.match_bytes = expansion
                                    .match_bytes
                                    .checked_sub(pattern.len().saturating_add(url.len()))
                                    .ok_or_else(invalid_config)?;
                                if bounded_match(pattern, url)? {
                                    matched = true;
                                    break;
                                }
                            }
                            matched
                        }
                    };
                    if enabled {
                        include_path(&record, path, environment, None)?
                    } else {
                        None
                    }
                } else if condition.starts_with(b"onbranch:") {
                    if let Some(branch) = self.branch.as_deref() {
                        include_path(&record, path, environment, Some(branch))?
                    } else {
                        None // Detached HEAD never matches onbranch, even `*`.
                    }
                } else {
                    None // Git ignores unknown condition kinds.
                }
            } else {
                include_path(&record, path, environment, self.branch.as_deref())?
            };
            if expansion.mode != ExpansionMode::ScanUrls {
                if expansion.records.len() >= 4096 {
                    return Err(invalid_config());
                }
                let payload = path
                    .as_os_str()
                    .len()
                    .checked_add(record.key.len())
                    .and_then(|length| {
                        length.checked_add(record.value.as_ref().map_or(0, Vec::len))
                    })
                    .ok_or_else(invalid_config)?;
                expansion.payload_bytes = expansion
                    .payload_bytes
                    .checked_sub(payload)
                    .ok_or_else(invalid_config)?;
                expansion.records.push((path.to_path_buf(), record));
            }
            if let Some(include) = include {
                self.expand(
                    &include,
                    depth + 1,
                    forbid_url
                        || (expansion.mode == ExpansionMode::ScanUrls && conditional_include),
                    environment,
                    expansion,
                )?;
            }
        }
        Ok(())
    }

    pub(super) fn revalidate(&self) -> Result<(), GitToolError> {
        for (path, expected) in &self.ancestors {
            if &directory_identity(path)? != expected {
                return Err(invalid_config());
            }
        }
        let mut budget = super::snapshot::Budget {
            entries: 4096,
            bytes: 4 * 1024 * 1024,
        };
        for (path, expected) in &self.files {
            if &super::snapshot::RegularEvidence::capture(path, &mut budget)
                .map_err(|_| invalid_config())?
                != expected
            {
                return Err(invalid_config());
            }
        }
        Ok(())
    }
}

fn directory_identity(
    path: &std::path::Path,
) -> Result<Option<super::snapshot::PathIdentity>, GitToolError> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
            super::snapshot::PathIdentity::capture(path)
                .map(Some)
                .map_err(|_| invalid_config())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        _ => Err(invalid_config()),
    }
}

fn parse_records_inner(bytes: &[u8]) -> Result<Vec<Record>, GitToolError> {
    use gix::bstr::ByteSlice;
    use gix::config::parse::{EventRef, Events};

    let events = Events::from_bytes(bytes, None).map_err(|_| invalid_config())?;
    let mut records = Vec::new();
    let mut remaining_bytes: usize = 4 * 1024 * 1024;
    parse_body(
        events.frontmatter(),
        &[],
        &mut records,
        &mut remaining_bytes,
    )?;
    for section in events.sections() {
        let EventRef::SectionHeader {
            name,
            separator,
            subsection_name,
        } = section.header()
        else {
            return Err(invalid_config());
        };
        let mut prefix = name.to_vec();
        prefix.make_ascii_lowercase();
        if let Some(subsection) = subsection_name {
            prefix.push(b'.');
            let mut subsection = subsection.to_vec();
            if separator.is_some_and(|separator| separator.as_bytes() == b".") {
                // Git's legacy `[section.subsection]` form folds the subsection.
                subsection.make_ascii_lowercase();
            }
            prefix.extend_from_slice(&subsection);
        }
        prefix.push(b'.');

        parse_body(section.body(), &prefix, &mut records, &mut remaining_bytes)?;
    }
    Ok(records)
}

fn parse_body<'a>(
    events: impl Iterator<Item = gix::config::parse::EventRef<'a>>,
    prefix: &[u8],
    records: &mut Vec<Record>,
    remaining_bytes: &mut usize,
) -> Result<(), GitToolError> {
    use gix::config::parse::EventRef;
    let mut key = None;
    let mut raw = Vec::new();
    let mut explicit = false;
    let mut continued = false;
    for event in events {
        match event {
            EventRef::SectionValueName(name) => {
                if key.is_some() || records.len() >= 4096 {
                    return Err(invalid_config());
                }
                // A long subsection repeated by many tiny keys can expand
                // far beyond the input size. Reserve before cloning it.
                let key_bytes = prefix
                    .len()
                    .checked_add(name.len())
                    .ok_or_else(invalid_config)?;
                *remaining_bytes = remaining_bytes
                    .checked_sub(key_bytes)
                    .ok_or_else(invalid_config)?;
                let mut full_key = prefix.to_vec();
                let mut name = name.to_vec();
                name.make_ascii_lowercase();
                full_key.extend_from_slice(&name);
                key = Some(full_key);
            }
            EventRef::KeyValueSeparator => explicit = true,
            EventRef::Value(value) => {
                raw.extend_from_slice(value);
                finish(records, &mut key, &mut raw, explicit, remaining_bytes)?;
                explicit = false;
            }
            EventRef::ValueNotDone(value) => {
                raw.extend_from_slice(value);
                continued = true;
            }
            EventRef::ValueDone(value) => {
                if !continued {
                    return Err(invalid_config());
                }
                raw.extend_from_slice(value);
                finish(records, &mut key, &mut raw, explicit, remaining_bytes)?;
                explicit = false;
                continued = false;
            }
            EventRef::Comment { .. }
            | EventRef::Newline(_)
            | EventRef::Whitespace(_)
            | EventRef::SectionHeader { .. } => {}
        }
    }
    if key.is_some() || continued {
        return Err(invalid_config());
    }
    Ok(())
}

fn finish(
    records: &mut Vec<Record>,
    key: &mut Option<Vec<u8>>,
    raw: &mut Vec<u8>,
    explicit: bool,
    remaining_bytes: &mut usize,
) -> Result<(), GitToolError> {
    if records.len() >= 4096 {
        return Err(invalid_config());
    }
    let key = key.take().ok_or_else(invalid_config)?;
    if explicit {
        // Normalization only shrinks raw values. Account its upper bound before
        // allocating normalized bytes, including empty/duplicate records.
        *remaining_bytes = remaining_bytes
            .checked_sub(raw.len())
            .ok_or_else(invalid_config)?;
    }
    let value = explicit.then(|| {
        gix::config::value::normalize(gix::bstr::BStr::new(raw))
            .as_ref()
            .to_vec()
    });
    records.push(Record { key, value });
    raw.clear();
    Ok(())
}

fn invalid_config() -> GitToolError {
    GitToolError::InvalidInput("Target Git configuration cannot be parsed safely".into())
}

#[cfg(test)]
mod tests {
    // Canonicalization introduces a verbatim prefix on Windows. Git's config
    // CLI does not reliably accept that spelling; these native parity fixtures
    // must use the ordinary absolute spelling supplied by tempfile instead.
    fn native_fixture_root(temporary: &tempfile::TempDir) -> std::path::PathBuf {
        #[cfg(windows)]
        {
            let root = temporary.path().to_path_buf();
            assert!(root.is_absolute());
            assert!(!root.as_os_str().to_string_lossy().starts_with("\\\\?\\"));
            root
        }
        #[cfg(not(windows))]
        temporary.path().canonicalize().expect("canonical root")
    }

    #[cfg(unix)]
    #[test]
    fn native_symlink_include_uses_spelled_origin_for_relative_children() {
        let temporary = tempfile::tempdir().expect("fixture");
        let root = temporary.path().canonicalize().expect("root");
        let alias = root.join("alias");
        let actual = root.join("actual");
        std::fs::create_dir(&alias).expect("alias parent");
        std::fs::create_dir(&actual).expect("referent parent");
        std::fs::write(actual.join("config"), b"[include]\npath=child\n").expect("referent");
        std::fs::write(alias.join("child"), b"[fixture]\nmarker=spelled\n").expect("spelled child");
        std::fs::write(actual.join("child"), b"[fixture]\nmarker=canonical\n")
            .expect("canonical child");
        std::os::unix::fs::symlink(actual.join("config"), alias.join("config"))
            .expect("config link");
        let output = std::process::Command::new("git")
            .args(["config", "--includes", "--file"])
            .arg(alias.join("config"))
            .args(["--get", "fixture.marker"])
            .output()
            .expect("native Git");
        assert!(output.status.success(), "{output:?}");
        assert_eq!(output.stdout, b"spelled\n");
    }

    #[test]
    fn optional_directory_frontier_retains_absence_and_rejects_later_creation() {
        let root = tempfile::tempdir().expect("fixture");
        let path = root
            .path()
            .canonicalize()
            .expect("root")
            .join("missing/nested-hooks");
        let mut sources = Sources::new(None, []).expect("sources");
        sources
            .add_optional_directory_context(&path)
            .expect("frontier");
        assert!(
            sources
                .advance(|candidate| candidate != path, &Default::default())
                .is_err()
        );
        assert!(sources.ancestors.is_empty());
        sources
            .advance(|_| true, &Default::default())
            .expect("exact-admitted absence");
        assert!(sources.complete());
        assert_eq!(sources.ancestors.get(&path), Some(&None));
        sources.revalidate().expect("unchanged absence");
        assert!(sources.add_directory_context(&path).is_err());
        std::fs::create_dir_all(&path).expect("later creation");
        assert!(sources.revalidate().is_err());
        let absent = root.path().join("required-missing");
        let mut required = Sources::new(None, []).expect("required sources");
        required
            .add_directory_context(&absent)
            .expect("required frontier");
        assert!(required.advance(|_| true, &Default::default()).is_err());
    }

    use super::*;

    #[cfg(unix)]
    #[test]
    fn parent_traversal_keeps_cancelled_directory_admission_and_identity() {
        let temporary = tempfile::tempdir().expect("fixture");
        let root = temporary.path().canonicalize().expect("root");
        std::fs::create_dir_all(root.join("a/b")).expect("directories");
        std::fs::write(root.join("source"), b"[include]\npath=child\n").expect("source");
        std::fs::write(root.join("child"), b"[fixture]\nmarker=retained\n").expect("child");
        let source = root.join("a/b/../../source");
        let mut sources = Sources::new(None, [source.clone()]).expect("frontier");
        assert!(sources.paths().contains(&root.join("a/b")));
        assert!(
            sources
                .advance(|path| path != root.join("a/b"), &Default::default())
                .is_err()
        );
        assert!(sources.files.is_empty());
        assert!(sources.ancestors.is_empty());
        while !sources.complete() {
            sources
                .advance(|_| true, &Default::default())
                .expect("admitted");
        }
        let records = sources
            .ordered_records(&[source], &Default::default(), None)
            .expect("records");
        assert!(
            records
                .iter()
                .any(|(origin, record)| origin == &root.join("a/b/../../child")
                    && record.key == b"fixture.marker")
        );
        std::fs::rename(root.join("a/b"), root.join("saved-b")).expect("retain inode");
        std::fs::create_dir(root.join("a/b")).expect("substitute directory");
        assert!(sources.revalidate().is_err());
    }

    #[cfg(unix)]
    #[test]
    fn missing_parent_traversal_never_reads_lexically_normalized_source() {
        let temporary = tempfile::tempdir().expect("fixture");
        let root = temporary.path().canonicalize().expect("root");
        std::fs::write(
            root.join("source"),
            b"invalid config that must not be parsed",
        )
        .expect("sentinel");
        let source = root.join("missing/../source");
        let mut sources = Sources::new(None, [source.clone()]).expect("frontier");
        sources
            .advance(|_| true, &Default::default())
            .expect("missing original path");
        assert!(matches!(
            sources.files[&source].entry,
            super::super::snapshot::Entry::Missing
        ));
        std::fs::create_dir(root.join("missing")).expect("later creation");
        assert!(sources.revalidate().is_err());
        std::fs::write(root.join("not-directory"), b"file").expect("file");
        let mut file_parent =
            Sources::new(None, [root.join("not-directory/../source")]).expect("frontier");
        assert!(file_parent.advance(|_| true, &Default::default()).is_err());
        assert!(file_parent.files.is_empty());
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(root.join("missing"), root.join("link")).expect("link");
            let mut link_parent =
                Sources::new(None, [root.join("link/../source")]).expect("frontier");
            assert!(link_parent.advance(|_| true, &Default::default()).is_err());
            assert!(link_parent.files.is_empty());
        }
    }

    #[cfg(unix)]
    #[test]
    fn native_checkout_missing_reporting_objects_do_not_spawn_fetch() {
        use std::os::unix::fs::PermissionsExt;
        for missing_ancestor in [false, true] {
            for comparison in ["@{upstream}", "@{push}"] {
                let temporary = tempfile::tempdir().expect("fixture");
                let root = temporary.path();
                let helpers = root.join("helpers");
                std::fs::create_dir(&helpers).expect("helpers");
                let helper = helpers.join("git-remote-fixture");
                std::fs::write(
                    &helper,
                    b"#!/bin/sh\necho attempted > helper-marker\nexit 88\n",
                )
                .expect("helper");
                std::fs::set_permissions(&helper, std::fs::Permissions::from_mode(0o755))
                    .expect("mode");
                let mut paths = vec![helpers];
                paths.extend(std::env::split_paths(
                    &std::env::var_os("PATH").unwrap_or_default(),
                ));
                let path = std::env::join_paths(paths).expect("PATH");
                let trace = root.join("trace.json");
                let git = |args: &[&str], tracing: bool| {
                    let output = std::process::Command::new("git")
                        .current_dir(root)
                        .env_clear()
                        .envs(
                            std::env::vars_os()
                                .filter(|(key, _)| !key.to_string_lossy().starts_with("GIT_")),
                        )
                        .env("PATH", &path)
                        .env("GIT_CONFIG_NOSYSTEM", "1")
                        .env("GIT_CONFIG_GLOBAL", "/dev/null")
                        .env("GIT_TRACE2", "0")
                        .env("GIT_TRACE2_PERF", "0")
                        // Isolated native experiment only; production disables Trace2.
                        .env(
                            "GIT_TRACE2_EVENT",
                            if tracing {
                                trace.as_os_str()
                            } else {
                                std::ffi::OsStr::new("0")
                            },
                        )
                        .args(args)
                        .output()
                        .expect("Git");
                    if !tracing {
                        assert!(output.status.success(), "{args:?}: {output:?}");
                    }
                    output
                };
                git(&["init", "-qb", "base"], false);
                git(
                    &[
                        "-c",
                        "user.name=Fixture",
                        "-c",
                        "user.email=fixture@example.invalid",
                        "-c",
                        "commit.gpgSign=false",
                        "commit",
                        "--allow-empty",
                        "-qm",
                        "initial",
                    ],
                    false,
                );
                let initial =
                    String::from_utf8(git(&["rev-parse", "HEAD"], false).stdout).expect("OID");
                let absent = "1234567890123456789012345678901234567890";
                if missing_ancestor {
                    let tree = String::from_utf8(git(&["rev-parse", "HEAD^{tree}"], false).stdout)
                        .expect("tree");
                    std::fs::write(root.join(".git/fixture-commit"), format!("tree {}\nparent {absent}\nauthor Fixture <fixture@example.invalid> 1700000000 +0000\ncommitter Fixture <fixture@example.invalid> 1700000000 +0000\n\nmissing ancestor\n", tree.trim())).expect("commit");
                    let child = String::from_utf8(
                        git(
                            &["hash-object", "-t", "commit", "-w", ".git/fixture-commit"],
                            false,
                        )
                        .stdout,
                    )
                    .expect("child");
                    git(&["update-ref", "refs/heads/base", child.trim()], false);
                }
                std::fs::create_dir_all(root.join(".git/refs/remotes/origin")).expect("refs");
                std::fs::write(
                    root.join(".git/refs/remotes/origin/main"),
                    format!(
                        "{}\n",
                        if missing_ancestor {
                            initial.trim()
                        } else {
                            absent
                        }
                    ),
                )
                .expect("upstream");
                for (key, value) in [
                    ("branch.autoSetupMerge", "always"),
                    ("core.hooksPath", "source-hooks"),
                    ("remote.origin.url", "fixture://must-not-run"),
                    ("remote.origin.fetch", "+refs/heads/*:refs/remotes/origin/*"),
                    ("remote.origin.promisor", "false"),
                    ("branch.topic.remote", "origin"),
                    ("branch.topic.merge", "refs/heads/main"),
                    ("push.default", "upstream"),
                    ("status.compareBranches", comparison),
                    ("includeIf.onbranch:topic.path", "target.conf"),
                ] {
                    git(&["config", key, value], false);
                }
                std::fs::write(root.join(".git/target.conf"), format!("[status]\ncompareBranches={comparison}\n[remote \"origin\"]\npromisor=false\n")).expect("target config");
                let output = git(&["checkout", "-b", "topic"], true);
                assert_eq!(
                    output.status.success(),
                    !missing_ancestor,
                    "{comparison}: {output:?}"
                );
                assert_eq!(
                    std::fs::read_to_string(root.join(".git/HEAD")).expect("HEAD"),
                    "ref: refs/heads/topic\n"
                );
                assert!(!root.join("helper-marker").exists());
                let events = std::fs::read_to_string(&trace).expect("trace");
                assert!(!events.is_empty());
                for line in events.lines() {
                    let event: serde_json::Value = serde_json::from_str(line).expect("event");
                    assert_ne!(event["event"], "child_start", "{comparison}: {line}");
                }
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn native_checkout_retains_hook_cache_but_may_reload_target_configuration() {
        use std::os::unix::fs::PermissionsExt;
        for tracked in [false, true] {
            for source_hooks_path in [false, true] {
                for malformed in [false, true] {
                    let temporary = tempfile::tempdir().expect("fixture");
                    let root = temporary.path();
                    let git = |args: &[&str]| {
                        std::process::Command::new("git")
                            .current_dir(root)
                            .env_clear()
                            .envs(
                                std::env::vars_os()
                                    .filter(|(key, _)| !key.to_string_lossy().starts_with("GIT_")),
                            )
                            .env("GIT_CONFIG_NOSYSTEM", "1")
                            .env("GIT_CONFIG_GLOBAL", "/dev/null")
                            .env("GIT_TRACE2", "0")
                            .env("GIT_TRACE2_EVENT", "0")
                            .env("GIT_TRACE2_PERF", "0")
                            .args(args)
                            .output()
                            .expect("Git")
                    };
                    assert!(git(&["init", "-qb", "base"]).status.success());
                    if tracked {
                        std::fs::write(root.join("file"), b"initial\n").expect("file");
                        assert!(git(&["add", "file"]).status.success());
                    }
                    assert!(
                        git(&[
                            "-c",
                            "user.name=Fixture",
                            "-c",
                            "user.email=fixture@example.invalid",
                            "-c",
                            "commit.gpgSign=false",
                            "commit",
                            "--allow-empty",
                            "-qm",
                            "initial",
                        ])
                        .status
                        .success()
                    );
                    for (key, value) in [
                        ("branch.autoSetupMerge", "always"),
                        ("branch.autoSetupRebase", "always"),
                        ("includeIf.onbranch:topic.path", "target.conf"),
                    ] {
                        assert!(git(&["config", key, value]).status.success());
                    }
                    if source_hooks_path {
                        assert!(
                            git(&["config", "core.hooksPath", "source-hooks"])
                                .status
                                .success()
                        );
                    }
                    std::fs::create_dir(root.join("target-hooks")).expect("hooks");
                    let hook = root.join("target-hooks/post-checkout");
                    std::fs::write(&hook, b"#!/bin/sh\necho target > file-hook-marker\n")
                        .expect("hook");
                    std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755))
                        .expect("mode");
                    std::fs::write(root.join(".git/target.conf"), if malformed {
                        b"[malformed\n".as_slice()
                    } else {
                        b"[core]\nhooksPath=target-hooks\n[hook \"target\"]\nevent=post-checkout\ncommand=echo configured > configured-hook-marker\n"
                    }).expect("target config");
                    let output = git(&["checkout", "-b", "topic"]);
                    // A non-null source hooksPath remains cached. Without it,
                    // a lookup can repopulate ordinary config before HEAD moves.
                    // Configured hook lists have a separate invocation cache.
                    assert_eq!(
                        output.status.success(),
                        !(malformed && source_hooks_path),
                        "tracked={tracked}, source_hooks_path={source_hooks_path}, malformed={malformed}"
                    );
                    assert_eq!(
                        std::fs::read(root.join(".git/HEAD")).expect("HEAD"),
                        b"ref: refs/heads/topic\n"
                    );
                    assert!(!root.join("file-hook-marker").exists());
                    assert!(!root.join("configured-hook-marker").exists());
                }
            }
        }
    }

    #[test]
    fn tracking_overlay_bounds_empty_section_and_write_payloads() {
        let write = Record {
            key: b"branch.topic.remote".to_vec(),
            value: Some(b".".to_vec()),
        };
        // Empty sections consume no record budget but still require bounded
        // physical-order bookkeeping for an admitted configuration file.
        let sections = b"[branch \"topic\"]\n".repeat(16385);
        assert!(tracking_overlay(&sections, std::slice::from_ref(&write)).is_err());
        let oversized = Record {
            value: Some(vec![b'a'; 4 * 1024 * 1024]),
            ..write
        };
        assert!(tracking_overlay(b"", &[oversized]).is_err());
    }

    #[test]
    fn tracking_overlay_matches_native_checkout_section_order_and_duplicates() {
        for suffix in [
            "",
            "[branch \"topic\"]\nremote=old\nmerge=refs/heads/old\n[include]\npath=absent\n",
            "[branch \"topic\"]\nremote=old\n[include]\npath=absent\n[branch \"topic\"]\n",
            "[branch \"topic\"]\nremote=first\nremote=second\nmerge=refs/heads/old\n",
            "[branch \"topic\"]\nmerge=refs/heads/one\nmerge=refs/heads/two\nrebase=false\nrebase=true\n",
            "[branch \"topic\"]\nremote=old\n[include]\npath=absent\n[branch \"topic\"]\nmerge=refs/heads/old\n",
            "[branch \"topic\"]\nremote=old\n[include]\npath=absent\n# preserve section\n[branch \"topic\"]\nmerge=refs/heads/old\n",
            "[branch \"topic\"]\nremote=old\n[include]\npath=absent\n[branch \"topic\"]\nmerge=refs/heads/old # inline\n",
            "[branch \"topic\"]\nremote=old\n[include]\npath=absent\n[branch \"topic\"]\nmerge=refs/heads/old\n[branch \"topic\"]\n",
            "[branch \"topic\"]\nremote=old\n[include]\npath=absent\n[branch \"topic\"] # header\nmerge=refs/heads/old\n",
            "[branch \"topic\"]\nremote=old # remote\n[include]\npath=absent\n[branch \"topic\"]\nmerge=refs/heads/old\n",
            "[branch \"topic\"]\nremote=old\n[include]\npath=absent\n[branch \"topic\"]\nmerge=refs/heads/old\n# after\n",
            "[BRANCH.topic]\nREMOTE=old\nMERGE=refs/heads/old\n[include]\npath=absent\n[branch \"topic\"]\n",
        ] {
            let temporary = tempfile::tempdir().expect("fixture");
            let root = native_fixture_root(&temporary);
            let git = |args: &[&str]| {
                let output = std::process::Command::new("git")
                    .current_dir(&root)
                    .args(args)
                    .output()
                    .expect("Git");
                assert!(output.status.success(), "{output:?}; suffix={suffix}");
                output.stdout
            };
            git(&["init", "--quiet"]);
            git(&["symbolic-ref", "HEAD", "refs/heads/base"]);
            git(&[
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "-c",
                "commit.gpgSign=false",
                "commit",
                "--allow-empty",
                "-qm",
                "initial",
            ]);
            let source = root.join(".git/config");
            let mut bytes = std::fs::read(&source).expect("config");
            bytes.splice(..0, b"path=before # frontmatter\n".iter().copied());
            bytes.extend_from_slice(b"[branch]\nautoSetupMerge=always\nautoSetupRebase=always\n");
            bytes.extend_from_slice(suffix.as_bytes());
            std::fs::write(&source, &bytes).expect("config");
            std::fs::write(root.join(".git/absent"), b"[branch \"topic\"]\nremote=included\nmerge=refs/heads/included\n[fixture]\nmarker=include\n").expect("included config");
            let writes = [
                Record {
                    key: b"branch.topic.remote".to_vec(),
                    value: Some(b".".to_vec()),
                },
                Record {
                    key: b"branch.topic.merge".to_vec(),
                    value: Some(b"refs/heads/base".to_vec()),
                },
                Record {
                    key: b"branch.topic.rebase".to_vec(),
                    value: Some(b"true".to_vec()),
                },
            ];
            let expected = tracking_overlay(&bytes, &writes).expect("overlay");
            let mut sources = Sources::new(Some(b"topic"), [source.clone()]).expect("sources");
            while !sources.complete() {
                sources
                    .advance(|_| true, &Default::default())
                    .expect("frontier");
            }
            sources
                .predict_tracking(&source, &writes)
                .expect("admitted overlay");
            sources
                .revalidate()
                .expect("overlay does not alter source evidence");
            let expanded = sources
                .ordered_records(std::slice::from_ref(&source), &Default::default(), None)
                .expect("predicted expansion");
            git(&["checkout", "-qb", "topic"]);
            let actual =
                parse_records(&std::fs::read(&source).expect("native config")).expect("parse");
            assert_eq!(actual, expected, "suffix={suffix}");
            let mut encoded = Vec::new();
            for (_, record) in expanded {
                encoded.extend(record.key);
                if let Some(value) = record.value {
                    encoded.push(b'\n');
                    encoded.extend(value);
                }
                encoded.push(0);
            }
            assert_eq!(
                encoded,
                git(&[
                    "config",
                    "--includes",
                    "--file",
                    source.to_str().expect("path"),
                    "--null",
                    "--list"
                ]),
                "expanded suffix={suffix}"
            );
        }
    }

    #[test]
    fn parses_ordered_values_without_resolving_includes() {
        let input = b"[include]\n path = first.conf\n path=\n path\n[includeIf \"onbranch:Created\"]\n path = \"a\\tb.conf\"\n[include]\n path = first.conf\n[fixture]\n text = a\\\n  b\n";
        let actual = parse_records(input).expect("parse");
        let pairs: Vec<_> = actual
            .iter()
            .map(|record| (record.key.as_slice(), record.value.as_deref()))
            .collect();
        assert_eq!(
            pairs,
            [
                (b"include.path".as_slice(), Some(b"first.conf".as_slice())),
                (b"include.path", Some(b"".as_slice())),
                (b"include.path", None),
                (
                    b"includeif.onbranch:Created.path",
                    Some(b"a\tb.conf".as_slice())
                ),
                (b"include.path", Some(b"first.conf".as_slice())),
                (b"fixture.text", Some(b"a  b".as_slice())),
            ]
        );
        let root = tempfile::tempdir().expect("fixture");
        let source = root.path().join("config");
        std::fs::write(&source, input).expect("source");
        let output = std::process::Command::new("git")
            .args(["config", "--no-includes", "--file"])
            .arg(&source)
            .args(["--null", "--list"])
            .output()
            .expect("native Git");
        assert!(output.status.success(), "{output:?}");
        let mut expected = Vec::new();
        for record in &actual {
            expected.extend_from_slice(&record.key);
            if let Some(value) = &record.value {
                expected.push(b'\n');
                expected.extend_from_slice(value);
            }
            expected.push(0);
        }
        assert_eq!(expected, output.stdout);
    }

    #[test]
    fn branch_frontier_matches_native_conditional_includes() {
        for (pattern, branch) in [
            ("Topic", "Topic"),
            ("Topic", "topic"),
            ("feature/", "feature/deep/topic"),
            ("feature/*", "feature/deep/topic"),
            ("feature/**", "feature/deep/topic"),
            ("[Tt]opic", "Topic"),
            ("?opic", "Topic"),
            ("feature/", "feature"),
        ] {
            let root = tempfile::tempdir().expect("fixture");
            let run = |args: &[&str]| {
                std::process::Command::new("git")
                    .current_dir(root.path())
                    .args(args)
                    .output()
                    .expect("native Git")
            };
            assert!(run(&["init", "--quiet"]).status.success());
            assert!(
                run(&["symbolic-ref", "HEAD", &format!("refs/heads/{branch}")])
                    .status
                    .success()
            );
            let source = root.path().join("source");
            let included = root.path().join("included");
            std::fs::write(&included, b"[fixture]\nmarker=yes\n").expect("include");
            let bytes = format!("[includeIf \"onbranch:{pattern}\"]\npath=included\n");
            std::fs::write(&source, &bytes).expect("source");
            let native = run(&[
                "config",
                "--includes",
                "--file",
                source.to_str().expect("source"),
                "--get",
                "fixture.marker",
            ]);
            assert!(
                native.status.success() || native.status.code() == Some(1),
                "{native:?}"
            );
            let frontier = declared_paths(
                bytes.as_bytes(),
                &source,
                &Default::default(),
                Some(branch.as_bytes()),
            )
            .expect("frontier");
            assert_eq!(
                !frontier.is_empty(),
                native.status.success(),
                "{pattern} {branch}"
            );
            if native.status.success() {
                assert_eq!(frontier, [included]);
            }
        }
    }

    #[test]
    fn complex_branch_pattern_refuses_before_dependency_recursion_guard() {
        let pattern = "*a".repeat(65);
        let branch = "a".repeat(65);
        let bytes = format!("[includeIf \"onbranch:{pattern}\"]\npath=secret-target\n");
        let origin = std::path::Path::new("/not-present/config");
        let error = declared_paths(
            bytes.as_bytes(),
            origin,
            &Default::default(),
            Some(branch.as_bytes()),
        )
        .expect_err("bounded matcher");
        assert!(!error.to_string().contains(&pattern));
        // Source discovery does not evaluate any condition or trigger the guard.
        assert_eq!(
            declared_paths(bytes.as_bytes(), origin, &Default::default(), None)
                .expect("source declarations")
                .len(),
            1
        );
    }

    #[test]
    fn lexical_frontier_retains_order_without_opening_named_paths() {
        let source = std::path::Path::new("/not-present/config");
        let bytes = b"[include]\npath=first\npath=first\n[includeIf \"onbranch:other\"]\npath=target\n[includeIf \"hasconfig:remote.*.url:*\"]\npath=later\n";
        let all =
            declared_paths(bytes, source, &Default::default(), None).expect("all dependencies");
        assert_eq!(
            all,
            [
                "/not-present/first",
                "/not-present/first",
                "/not-present/target",
                "/not-present/later"
            ]
            .map(std::path::PathBuf::from)
        );
        let target = declared_paths(bytes, source, &Default::default(), Some(b"main"))
            .expect("target frontier");
        assert_eq!(
            target,
            [
                "/not-present/first",
                "/not-present/first",
                "/not-present/later"
            ]
            .map(std::path::PathBuf::from)
        );
    }

    #[test]
    fn admitted_source_frontiers_defer_nested_reads_and_bind_absence() {
        let temporary = tempfile::tempdir().expect("fixture");
        let root = temporary.path().canonicalize().expect("canonical fixture");
        let source = root.as_path().join("source");
        let child = root.as_path().join("child");
        let missing = root.as_path().join("missing");
        std::fs::write(&source, b"[include]\npath=child\n").expect("source");
        std::fs::write(&child, b"[include]\npath=missing\n").expect("child");
        let mut sources = Sources::new(Some(b"main"), [source.clone()]).expect("roots");
        let first = sources.paths();
        assert!(!first.contains(&child));
        assert!(
            sources
                .advance(|path| path != source, &Default::default())
                .is_err()
        );
        assert!(sources.files.is_empty(), "denial must precede all capture");
        sources
            .advance(|path| first.contains(path), &Default::default())
            .expect("source only");
        assert!(!sources.files.contains_key(&child));
        assert!(sources.paths().contains(&child));
        let second = sources.paths();
        sources
            .advance(|path| second.contains(path), &Default::default())
            .expect("child only");
        assert!(!sources.files.contains_key(&missing));
        let third = sources.paths();
        sources
            .advance(|path| third.contains(path), &Default::default())
            .expect("missing evidence");
        assert!(sources.complete());
        std::fs::write(&missing, b"[fixture]\nkey=changed\n").expect("drift");
        assert!(sources.revalidate().is_err());
    }

    #[test]
    fn source_frontiers_require_ancestor_admission_and_refuse_links() {
        let temporary = tempfile::tempdir().expect("fixture");
        let root = temporary.path().canonicalize().expect("canonical fixture");
        let source = root.as_path().join("source");
        std::fs::write(&source, b"[fixture]\nkey=value\n").expect("source");
        let mut sources = Sources::new(Some(b"main"), [source.clone()]).expect("roots");
        assert!(
            sources
                .advance(|path| path == source, &Default::default())
                .is_err()
        );
        assert!(sources.files.is_empty());
        #[cfg(unix)]
        {
            let link = root.as_path().join("link");
            std::os::unix::fs::symlink(&source, &link).expect("link");
            let mut links = Sources::new(Some(b"main"), [link]).expect("roots");
            assert!(links.advance(|_| true, &Default::default()).is_err());
        }
        #[cfg(unix)]
        {
            let mut traversal = Sources::new(Some(b"main"), [root.join("missing/../source")])
                .expect("traversal frontier");
            traversal
                .advance(|_| true, &Default::default())
                .expect("native missing traversal");
            assert!(matches!(
                traversal.files.values().next().expect("source").entry,
                super::super::snapshot::Entry::Missing
            ));
        }
        assert!(
            Sources::new(
                Some(b"main"),
                [std::path::PathBuf::from(format!("/{}", "a".repeat(4097)))]
            )
            .is_err()
        );
    }

    #[cfg(windows)]
    #[test]
    fn verbatim_join_normalization_precedes_source_admission() {
        let temporary = tempfile::tempdir().expect("fixture");
        let root = temporary.path().canonicalize().expect("verbatim root");
        let source = root.join("missing/../source");
        // Rust PathBuf::push/join removes dot components for verbatim roots.
        // No missing directory is traversed by the resulting source path.
        assert_eq!(source, root.join("source"));
        std::fs::write(&source, b"[fixture]\nmarker=effective\n").expect("source");
        let mut sources = Sources::new(None, [source.clone()]).expect("frontier");
        assert!(sources.paths().contains(&source));
        assert!(!sources.paths().contains(&root.join("missing")));
        assert!(
            sources
                .advance(|path| path != source, &Default::default())
                .is_err()
        );
        assert!(sources.files.is_empty());
        while !sources.complete() {
            sources
                .advance(|_| true, &Default::default())
                .expect("admitted");
        }
        let records = sources
            .ordered_records(std::slice::from_ref(&source), &Default::default(), None)
            .expect("records");
        assert!(
            records
                .iter()
                .any(|(origin, record)| origin == &source && record.key == b"fixture.marker")
        );
        std::fs::write(&source, b"[fixture]\nmarker=changed\n").expect("drift");
        assert!(sources.revalidate().is_err());
    }

    #[test]
    fn source_request_budget_is_cumulative_including_ancestors() {
        let paths = (0..4096).map(|index| std::path::PathBuf::from(format!("/fixture/{index}")));
        assert!(Sources::new(Some(b"main"), paths).is_err());
        let long = "x".repeat(3500);
        let paths =
            (0..1300).map(|index| std::path::PathBuf::from(format!("/fixture/{long}{index}")));
        assert!(Sources::new(Some(b"main"), paths).is_err());
    }

    #[test]
    fn ordered_expansion_matches_native_git_at_include_positions() {
        let temporary = tempfile::tempdir().expect("fixture");
        let root = native_fixture_root(&temporary);
        let git = |args: &[&str]| {
            std::process::Command::new("git")
                .current_dir(&root)
                .args(args)
                .output()
                .expect("native Git")
        };
        assert!(git(&["init", "--quiet"]).status.success());
        assert!(
            git(&["symbolic-ref", "HEAD", "refs/heads/topic/created"])
                .status
                .success()
        );
        let source = root.join("source");
        let child = root.join("child");
        let target = root.join("target");
        std::fs::write(&source, b"[fixture]\nmarker=before\n[include]\npath=child\n[fixture]\nmarker=middle\n[include]\npath=child\n[includeIf \"onbranch:topic/\"]\npath=target\n[fixture]\nmarker=after\n").expect("source");
        std::fs::write(&child, b"[fixture]\nmarker=child\nimplicit\nempty=\n").expect("child");
        std::fs::write(&target, b"[fixture]\nmarker=target\n").expect("target");
        let native = git(&[
            "config",
            "--includes",
            "--file",
            source.to_str().expect("source"),
            "--null",
            "--list",
        ]);
        assert!(native.status.success(), "{native:?}");
        let mut sources = Sources::new(Some(b"topic/created"), [source.clone()]).expect("roots");
        while !sources.complete() {
            sources
                .advance(|_| true, &Default::default())
                .expect("admitted frontier");
        }
        // Expansion must only use captured bytes, even if files disappear.
        std::fs::remove_file(&child).expect("remove child");
        let expanded = sources
            .ordered_records(std::slice::from_ref(&source), &Default::default(), None)
            .expect("ordered expansion");
        let mut actual = Vec::new();
        for (origin, record) in expanded {
            assert!([&source, &child, &target].contains(&&origin));
            actual.extend(record.key);
            if let Some(value) = record.value {
                actual.push(b'\n');
                actual.extend(value);
            }
            actual.push(0);
        }
        assert_eq!(actual, native.stdout);
        assert!(sources.revalidate().is_err());
    }

    #[test]
    fn ordered_expansion_obeys_depth_missing_and_cycle_rules() {
        let temporary = tempfile::tempdir().expect("fixture");
        let root = native_fixture_root(&temporary);
        for index in 0..=10 {
            std::fs::write(
                root.join(format!("config{index}")),
                format!("[include]\npath=config{}\n", index + 1),
            )
            .expect("chain");
        }
        let source = root.join("config0");
        let native = || {
            std::process::Command::new("git")
                .args(["config", "--includes", "--file"])
                .arg(&source)
                .args(["--null", "--list"])
                .output()
                .expect("native Git")
        };
        assert!(
            native().status.success(),
            "missing eleventh include is ignored"
        );
        let capture = || {
            let mut sources = Sources::new(Some(b"main"), [source.clone()]).expect("roots");
            while !sources.complete() {
                sources
                    .advance(|_| true, &Default::default())
                    .expect("frontier");
            }
            sources
        };
        let sources = capture();
        assert!(
            sources
                .ordered_records(std::slice::from_ref(&source), &Default::default(), None)
                .is_ok()
        );
        std::fs::write(root.join("config11"), b"[fixture]\nmarker=too-deep\n").expect("too deep");
        assert!(!native().status.success());
        assert!(
            capture()
                .ordered_records(std::slice::from_ref(&source), &Default::default(), None)
                .is_err()
        );
        std::fs::write(&source, b"[include]\npath=config0\n").expect("cycle");
        assert!(!native().status.success());
        assert!(
            capture()
                .ordered_records(&[source], &Default::default(), None)
                .is_err()
        );
    }

    #[test]
    fn repeated_include_expansion_has_a_cumulative_payload_budget() {
        let temporary = tempfile::tempdir().expect("fixture");
        let root = temporary.path().canonicalize().expect("root");
        let source = root.join("source");
        std::fs::write(&source, format!("[include]\n{}", "path=child\n".repeat(22)))
            .expect("source");
        std::fs::write(
            root.join("child"),
            format!("[fixture]\nlarge={}\n", "x".repeat(200_000)),
        )
        .expect("child");
        let mut sources = Sources::new(Some(b"main"), [source.clone()]).expect("roots");
        while !sources.complete() {
            sources
                .advance(|_| true, &Default::default())
                .expect("frontier");
        }
        assert!(
            sources
                .ordered_records(&[source], &Default::default(), None)
                .is_err()
        );
    }

    #[test]
    fn repeated_comment_only_sources_charge_parse_work_before_parsing() {
        let temporary = tempfile::tempdir().expect("fixture");
        let root = temporary.path().canonicalize().expect("root");
        let source = root.join("source");
        std::fs::write(&source, format!("[include]\n{}", "path=child\n".repeat(6)))
            .expect("source");
        std::fs::write(
            root.join("child"),
            format!("#{}\n", "x".repeat(1024 * 1024)),
        )
        .expect("child");
        let mut sources = Sources::new(Some(b"main"), [source.clone()]).expect("roots");
        while !sources.complete() {
            sources
                .advance(|_| true, &Default::default())
                .expect("frontier");
        }
        assert!(
            sources
                .ordered_records(&[source], &Default::default(), None)
                .is_err()
        );
    }

    #[test]
    fn hasconfig_lookahead_and_url_prohibition_match_native_git() {
        for (pattern, later_url, child_bytes, success) in [
            (
                "https://example.invalid/**",
                "https://example.invalid/a/b",
                "[fixture]\nmarker=selected\n",
                true,
            ),
            (
                "https://example.invalid/*",
                "https://example.invalid/a/b",
                "[fixture]\nmarker=selected\n",
                true,
            ),
            (
                "https://other.invalid/**",
                "https://example.invalid/a",
                "[remote \"illegal\"]\nurl=forbidden\n",
                false,
            ),
            (
                "https://example.invalid/**",
                "https://example.invalid/a",
                "[include]\npath=grandchild\n",
                false,
            ),
        ] {
            let temporary = tempfile::tempdir().expect("fixture");
            let root = native_fixture_root(&temporary);
            let source = root.join("source");
            std::fs::write(&source, format!("[includeIf \"hasconfig:remote.*.url:{pattern}\"]\npath=child\n[include]\npath=later\n")).expect("source");
            std::fs::write(
                root.join("later"),
                format!("[remote \"origin\"]\nurl={later_url}\n"),
            )
            .expect("later URL");
            std::fs::write(root.join("child"), child_bytes).expect("child");
            std::fs::write(
                root.join("grandchild"),
                b"[remote \"illegal\"]\nurl=forbidden\n",
            )
            .expect("grandchild");
            let native = std::process::Command::new("git")
                .args(["config", "--includes", "--file"])
                .arg(&source)
                .args(["--null", "--list"])
                .output()
                .expect("native Git");
            assert_eq!(native.status.success(), success, "{native:?}");
            let mut sources = Sources::new(Some(b"main"), [source.clone()]).expect("roots");
            while !sources.complete() {
                sources
                    .advance(|_| true, &Default::default())
                    .expect("frontier");
            }
            let actual = sources.ordered_records(&[source], &Default::default(), None);
            assert_eq!(actual.is_ok(), success);
            if let Ok(records) = actual {
                let mut encoded = Vec::new();
                for (_, record) in records {
                    encoded.extend(record.key);
                    if let Some(value) = record.value {
                        encoded.push(b'\n');
                        encoded.extend(value);
                    }
                    encoded.push(0);
                }
                assert_eq!(encoded, native.stdout);
            }
        }
    }

    #[test]
    fn detached_head_never_matches_onbranch_wildcards() {
        let temporary = tempfile::tempdir().expect("fixture");
        let root = temporary.path().canonicalize().expect("root");
        let output = std::process::Command::new("git")
            .current_dir(&root)
            .args(["init", "--quiet"])
            .output()
            .expect("init");
        assert!(output.status.success());
        std::fs::write(
            root.join(".git/HEAD"),
            b"1111111111111111111111111111111111111111\n",
        )
        .expect("detached HEAD");
        let source = root.join("source");
        std::fs::write(root.join("child"), b"[fixture]\nmarker=selected\n").expect("child");
        for pattern in ["*", "**", "", "HEAD"] {
            std::fs::write(
                &source,
                format!("[includeIf \"onbranch:{pattern}\"]\npath=child\n"),
            )
            .expect("source");
            let output = std::process::Command::new("git")
                .current_dir(&root)
                .args(["config", "--includes", "--file"])
                .arg(&source)
                .args(["--get", "fixture.marker"])
                .output()
                .expect("native");
            assert_eq!(output.status.code(), Some(1), "{output:?}");
            let mut sources = Sources::new(None, [source.clone()]).expect("roots");
            while !sources.complete() {
                sources
                    .advance(|_| true, &Default::default())
                    .expect("capture");
            }
            let records = sources
                .ordered_records(std::slice::from_ref(&source), &Default::default(), None)
                .expect("evaluate");
            assert!(
                !records
                    .iter()
                    .any(|(_, record)| record.key == b"fixture.marker")
            );
        }
    }

    #[test]
    fn linked_gitdir_context_requires_admission_without_reading_children() {
        let temporary = tempfile::tempdir().expect("fixture");
        let root = temporary.path().canonicalize().expect("root");
        let gitdir = root.join("common/worktrees/linked");
        std::fs::create_dir_all(&gitdir).expect("gitdir");
        // Invalid config bytes must not be read or parsed as context evidence.
        std::fs::write(gitdir.join("config.worktree"), b"[broken\n").expect("child");
        let source = root.join("source");
        std::fs::write(&source, b"[includeIf \"gitdir:linked\"]\npath=selected\n").expect("source");
        std::fs::write(root.join("selected"), b"[fixture]\nmarker=yes\n").expect("selected");
        let mut sources = Sources::new(Some(b"main"), [source.clone()]).expect("roots");
        sources
            .add_directory_context(&gitdir)
            .expect("context request");
        assert!(
            sources
                .advance(|path| path != gitdir, &Default::default())
                .is_err()
        );
        assert!(sources.files.is_empty());
        assert!(sources.ancestors.is_empty());
        while !sources.complete() {
            sources
                .advance(|_| true, &Default::default())
                .expect("capture");
        }
        assert!(!sources.paths().contains(&gitdir.join("config.worktree")));
        let records = sources
            .ordered_records(&[source], &Default::default(), Some(&gitdir))
            .expect("evaluate");
        assert!(
            records
                .iter()
                .any(|(_, record)| record.key == b"fixture.marker")
        );
        std::fs::rename(&gitdir, root.join("old-linked")).expect("move");
        std::fs::create_dir(&gitdir).expect("replacement");
        assert!(sources.revalidate().is_err());
    }

    #[test]
    fn host_root_order_and_worktree_selection_match_native_git() {
        let temporary = tempfile::tempdir().expect("fixture");
        let root = native_fixture_root(&temporary);
        let home = root.join("home");
        let xdg = root.join("xdg");
        let repository = root.join("repo");
        std::fs::create_dir_all(&home).expect("home");
        std::fs::create_dir_all(xdg.join("git")).expect("xdg");
        std::fs::create_dir_all(&repository).expect("repo");
        let system = root.join("system");
        let global = home.join(".gitconfig");
        let xdg_config = xdg.join("git/config");
        let gitdir = repository.join(".git");
        let repository_config = gitdir.join("config");
        let worktree = gitdir.join("config.worktree");
        let native = |args: &[&str]| {
            std::process::Command::new("git")
                .current_dir(&repository)
                .env("HOME", &home)
                .env("XDG_CONFIG_HOME", &xdg)
                .env("GIT_CONFIG_SYSTEM", &system)
                .env_remove("GIT_CONFIG_GLOBAL")
                .env_remove("GIT_CONFIG_NOSYSTEM")
                .args(args)
                .output()
                .expect("native Git")
        };
        assert!(native(&["init", "--quiet"]).status.success());
        std::fs::write(&system, b"[fixture]\nmarker=system\n").expect("system");
        let configured = native(&[
            "config",
            "--file",
            system.to_str().expect("system path"),
            "include.path",
            global.to_str().expect("global path"),
        ]);
        assert!(configured.status.success(), "{configured:?}");
        std::fs::write(&xdg_config, b"[fixture]\nmarker=xdg\n").expect("xdg");
        std::fs::write(
            &global,
            b"[fixture]\nmarker=user\n[extensions]\nworktreeConfig=true\n",
        )
        .expect("global");
        std::fs::write(
            gitdir.join("included"),
            b"[extensions]\nworktreeConfig=true\n[fixture]\nmarker=included\n",
        )
        .expect("include");
        std::fs::write(&worktree, b"[fixture]\nmarker=worktree\n").expect("worktree");
        let original = std::fs::read_to_string(&repository_config).expect("config");
        for (extension, enabled) in [
            ("", false),
            ("[extensions]\nworktreeConfig=true\n", true),
            (
                "[extensions]\nworktreeConfig=true\nworktreeConfig=false\n",
                false,
            ),
        ] {
            std::fs::write(
                &repository_config,
                format!(
                    "{original}\n[include]\npath=included\n[fixture]\nmarker=local\n{extension}"
                ),
            )
            .expect("local");
            let mut roots = Vec::new();
            for variable in ["GIT_CONFIG_SYSTEM", "GIT_CONFIG_GLOBAL"] {
                let output = native(&["var", variable]);
                assert!(output.status.success(), "{output:?}");
                roots.extend(
                    std::str::from_utf8(&output.stdout)
                        .expect("paths")
                        .lines()
                        .map(std::path::PathBuf::from),
                );
            }
            assert_eq!(roots, [system.clone(), xdg_config.clone(), global.clone()]);
            roots.push(repository_config.clone());
            let mut sources = Sources::new(Some(b"main"), roots.clone()).expect("roots");
            while !sources.complete() {
                sources
                    .advance(|_| true, &Default::default())
                    .expect("capture");
            }
            assert_eq!(
                sources
                    .worktree_config_enabled(&repository_config)
                    .expect("format"),
                enabled
            );
            assert!(!sources.files.contains_key(&worktree));
            if enabled {
                sources.add([worktree.clone()]).expect("request");
                assert!(!sources.complete());
                assert!(!sources.files.contains_key(&worktree));
                sources
                    .advance(|_| true, &Default::default())
                    .expect("worktree capture");
                roots.push(worktree.clone());
            }
            let records = sources
                .ordered_records(&roots, &Default::default(), Some(&gitdir))
                .expect("ordered config");
            let mut encoded = Vec::new();
            for (_, record) in records {
                if record.key == b"fixture.marker" {
                    encoded.extend(record.value.expect("marker"));
                    encoded.push(b'\n');
                }
            }
            let output = native(&["config", "--get-all", "fixture.marker"]);
            assert!(output.status.success(), "{output:?}");
            assert_eq!(encoded, output.stdout);
        }
    }

    #[test]
    fn gitdir_conditions_match_native_with_admitted_directory_identity() {
        let temporary = tempfile::tempdir().expect("fixture");
        let root = temporary
            .path()
            .canonicalize()
            .expect("root")
            .join("[literal]");
        let repository = root.join("repo");
        std::fs::create_dir_all(&repository).expect("repository");
        let native = |args: &[&str]| {
            std::process::Command::new("git")
                .current_dir(&repository)
                .env("HOME", &root)
                .args(args)
                .output()
                .expect("native Git")
        };
        assert!(native(&["init", "--quiet"]).status.success());
        let gitdir = repository.join(".git");
        let source = root.join("source");
        std::fs::write(root.join("child"), b"[fixture]\nmarker=selected\n").expect("child");
        let environment = std::collections::BTreeMap::from([(
            std::ffi::OsString::from("HOME"),
            root.as_os_str().to_owned(),
        )]);
        for condition in [
            "gitdir:./repo/.git".to_string(),
            "gitdir:./repo/".to_string(),
            "gitdir:./other/.git".to_string(),
            "gitdir:repo/.git".to_string(),
            "gitdir:repo/".to_string(),
            "gitdir:REPO/.git".to_string(),
            "gitdir/i:REPO/.git".to_string(),
            "gitdir:~/repo/.git".to_string(),
            format!("gitdir:{}", gitdir.display()),
            format!(
                "gitdir:{}",
                gitdir
                    .display()
                    .to_string()
                    .replace('[', "\\[")
                    .replace(']', "\\]")
            ),
        ] {
            // Escape config-string backslashes, retaining the glob escapes.
            std::fs::write(
                &source,
                format!(
                    "[includeIf \"{}\"]\npath=child\n",
                    condition.replace('\\', "\\\\")
                ),
            )
            .expect("source");
            let output = native(&[
                "config",
                "--includes",
                "--file",
                source.to_str().expect("source"),
                "--get",
                "fixture.marker",
            ]);
            assert!(
                output.status.success() || output.status.code() == Some(1),
                "{output:?}"
            );
            let mut sources = Sources::new(Some(b"main"), [source.clone(), gitdir.join("config")])
                .expect("roots");
            while !sources.complete() {
                sources.advance(|_| true, &environment).expect("capture");
            }
            let records = sources
                .ordered_records(std::slice::from_ref(&source), &environment, Some(&gitdir))
                .expect("evaluate");
            assert_eq!(
                records
                    .iter()
                    .any(|(_, record)| record.key == b"fixture.marker"),
                output.status.success(),
                "{condition}"
            );
            assert!(
                sources
                    .ordered_records(
                        std::slice::from_ref(&source),
                        &environment,
                        Some(&root.join("unadmitted"))
                    )
                    .is_err()
            );
        }
    }

    #[test]
    fn hasconfig_comparisons_share_a_cumulative_budget() {
        let temporary = tempfile::tempdir().expect("fixture");
        let root = temporary.path().canonicalize().expect("root");
        let source = root.join("source");
        let mut config = String::new();
        for index in 0..100 {
            config.push_str(&format!(
                "[remote \"r{index}\"]\nurl=https://example.invalid/{}\n",
                "a".repeat(1000)
            ));
        }
        for _ in 0..100 {
            config.push_str(
                "[includeIf \"hasconfig:remote.*.url:https://other.invalid/**\"]\npath=missing\n",
            );
        }
        std::fs::write(&source, config).expect("source");
        let mut sources = Sources::new(Some(b"main"), [source.clone()]).expect("roots");
        while !sources.complete() {
            sources
                .advance(|_| true, &Default::default())
                .expect("capture");
        }
        assert!(
            sources
                .ordered_records(&[source], &Default::default(), None)
                .is_err()
        );
    }

    #[test]
    fn hasconfig_non_path_key_still_triggers_url_prohibition() {
        let temporary = tempfile::tempdir().expect("fixture");
        let root = native_fixture_root(&temporary);
        let source = root.join("source");
        let git = |args: &[&str]| {
            std::process::Command::new("git")
                .current_dir(&root)
                .args(args)
                .output()
                .expect("native Git")
        };
        assert!(git(&["init", "--quiet"]).status.success());
        assert!(
            git(&["symbolic-ref", "HEAD", "refs/heads/main"])
                .status
                .success()
        );
        std::fs::write(
            root.join("child"),
            b"[remote \"origin\"]\nurl=https://example.invalid/repo\n",
        )
        .expect("child");
        for key in ["unknown", "path"] {
            std::fs::write(&source, format!("[includeIf \"onbranch:main\"]\npath=child\n[includeIf \"hasconfig:remote.*.url:**\"]\n{key}=missing\n"))
                .expect("source");
            let native = git(&[
                "config",
                "--includes",
                "--file",
                source.to_str().expect("path"),
                "--list",
            ]);
            assert!(!native.status.success(), "{native:?}");
            assert!(
                String::from_utf8_lossy(&native.stderr)
                    .contains("remote URLs cannot be configured")
            );
            let mut sources = Sources::new(Some(b"main"), [source.clone()]).expect("roots");
            while !sources.complete() {
                sources
                    .advance(|_| true, &Default::default())
                    .expect("capture");
            }
            let error = sources
                .ordered_records(std::slice::from_ref(&source), &Default::default(), None)
                .expect_err("URL prohibition");
            assert!(error.to_string().contains("Remote URL is forbidden"));
        }
    }

    #[test]
    fn malformed_file_fails_without_exposing_its_contents() {
        let error = parse_records(b"[broken\nsecret = value\n").expect_err("malformed");
        assert!(!error.to_string().contains("secret"));
    }

    #[test]
    fn source_records_match_native_git_syntax_and_order() {
        for input in [
            b"path = before\nimplicit\nempty=\n[fixture]\nkey = after\n".as_slice(),
            b"[Core]\n Bare\n bare=\n bare=\"\"\n".as_slice(),
            b"[REMOTE.Origin]\n URL=old\n[remote \"Origin\"]\n URL=new\n",
            b"[fixture \"a\\\"b\\\\c\"]\n key=one # comment\n key=two;comment\n",
            b"[fixture]\n key=\"a\\n\\t\\b\\\\\\\"b\"\n",
            b"[fixture]\n key= a\" b \"c   \n key=one\\\n two\\\n three\n",
            b"[fixture]\r\n key=\r\n key\r\n key=value\r\n",
            b"[fixture]\n key=last\\",
            b"\xef\xbb\xbf[fixture]\n key=value\n",
            b"[include]\n path=missing\n[fixture]\n marker=before\n[include]\n path=missing\n[fixture]\n marker=after\n",
        ] {
            let root = tempfile::tempdir().expect("fixture");
            let path = root.path().join("config");
            std::fs::write(&path, input).expect("config");
            let output = std::process::Command::new("git")
                .args(["config", "--no-includes", "--file"])
                .arg(&path)
                .args(["--null", "--list"])
                .output()
                .expect("native Git");
            assert!(output.status.success(), "{output:?}");
            let records = parse_records(input).expect("pure parser");
            let mut actual = Vec::new();
            for record in records {
                actual.extend(record.key);
                if let Some(value) = record.value {
                    actual.push(b'\n');
                    actual.extend(value);
                }
                actual.push(0);
            }
            assert_eq!(actual, output.stdout, "input {input:?}");
        }
    }

    #[test]
    fn source_parser_enforces_byte_and_record_limits() {
        assert!(parse_records(&vec![b' '; 4 * 1024 * 1024 + 1]).is_err());
        let mut input = b"[fixture]\n".to_vec();
        for _ in 0..4096 {
            input.extend_from_slice(b"key=value\n");
        }
        assert_eq!(parse_records(&input).expect("at limit").len(), 4096);
        input.extend_from_slice(b"key=value\n");
        assert!(parse_records(&input).is_err());
        let mut expanded = b"[fixture \"".to_vec();
        expanded.extend(vec![b'a'; 1024 * 1024]);
        expanded.extend_from_slice(b"\"]\nk=v\nk=v\nk=v\nk=v\n");
        assert!(parse_records(&expanded).is_err());
    }
}
