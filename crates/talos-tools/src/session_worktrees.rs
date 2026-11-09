//! Session-scoped, verified Git worktree membership. The host owns the receipt store.
use cap_fs_ext::{FollowSymlinks, MetadataExt, OpenOptionsFollowExt};
use cap_std::fs::Dir;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use talos_core::tool::{AgentTool, ToolResult, WorkspaceRootCapability};
use talos_permission::PermissionSessionState;

type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
struct Identity {
    device: u64,
    inode: u64,
}
fn directory(path: &Path) -> Result<(Dir, Identity)> {
    std::panic::catch_unwind(|| {
        let dir =
            Dir::open_ambient_dir(path, cap_std::ambient_authority()).map_err(|e| e.to_string())?;
        let meta = dir.dir_metadata().map_err(|e| e.to_string())?;
        Ok((
            dir,
            Identity {
                device: meta.dev(),
                inode: meta.ino(),
            },
        ))
    })
    .map_err(|_| "directory identity unavailable".to_owned())?
}
fn read_options() -> cap_std::fs::OpenOptions {
    let mut options = cap_std::fs::OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    #[cfg(unix)]
    {
        use cap_std::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NONBLOCK);
    }
    options
}
fn file_identity(path: &Path) -> Result<(cap_std::fs::File, Identity)> {
    std::panic::catch_unwind(|| {
        let (parent, _) = directory(path.parent().ok_or("missing parent")?)?;
        let file = parent
            .open_with(path.file_name().ok_or("missing filename")?, &read_options())
            .map_err(|e| e.to_string())?;
        let meta = file.metadata().map_err(|e| e.to_string())?;
        Ok((
            file,
            Identity {
                device: meta.dev(),
                inode: meta.ino(),
            },
        ))
    })
    .map_err(|_| "file identity unavailable".to_owned())?
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
struct MetadataFile {
    identity: Identity,
    value: String,
}
fn metadata_file(path: &Path) -> Result<MetadataFile> {
    let meta = std::fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > 8192 {
        return Err("invalid worktree metadata".into());
    }
    let (file, identity) = file_identity(path)?;
    if !file.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("worktree metadata is not a regular file".into());
    }
    let mut value = String::new();
    file.take(8193)
        .read_to_string(&mut value)
        .map_err(|e| e.to_string())?;
    if value.len() > 8192 || file_identity(path)?.1 != identity {
        return Err("worktree metadata changed or exceeded limit".into());
    }
    Ok(MetadataFile { identity, value })
}
fn read_link_file(path: &Path) -> Result<String> {
    Ok(metadata_file(path)?.value)
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct Receipt {
    root: PathBuf,
    identity: Identity,
    gitdir: PathBuf,
    git_identity: Identity,
    common: PathBuf,
    common_identity: Identity,
    dotgit: MetadataFile,
    commondir: MetadataFile,
    backlink: MetadataFile,
}
impl Receipt {
    fn capture(root: &Path, common: &Path) -> Result<Self> {
        let root = root.canonicalize().map_err(|e| e.to_string())?;
        let (_, identity) = directory(&root)?;
        let dotgit = metadata_file(&root.join(".git"))?;
        let gitdir = root
            .join(
                dotgit
                    .value
                    .trim()
                    .strip_prefix("gitdir: ")
                    .ok_or("invalid .git pointer")?,
            )
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let (_, git_identity) = directory(&gitdir)?;
        let commondir = metadata_file(&gitdir.join("commondir"))?;
        let actual_common = gitdir
            .join(commondir.value.trim())
            .canonicalize()
            .map_err(|e| e.to_string())?;
        if actual_common != common {
            return Err("worktree belongs to another repository".into());
        }
        let (_, common_identity) = directory(common)?;
        let backlink = metadata_file(&gitdir.join("gitdir"))?;
        if gitdir
            .join(backlink.value.trim())
            .canonicalize()
            .map_err(|e| e.to_string())?
            != root.join(".git")
        {
            return Err("invalid worktree backlink".into());
        }
        if directory(&root)?.1 != identity
            || directory(&gitdir)?.1 != git_identity
            || directory(common)?.1 != common_identity
            || metadata_file(&root.join(".git"))? != dotgit
            || metadata_file(&gitdir.join("commondir"))? != commondir
            || metadata_file(&gitdir.join("gitdir"))? != backlink
        {
            return Err("worktree changed during identity capture".into());
        }
        Ok(Self {
            root,
            identity,
            gitdir,
            git_identity,
            common: common.to_owned(),
            common_identity,
            dotgit,
            commondir,
            backlink,
        })
    }
    fn valid(&self) -> bool {
        Self::capture(&self.root, &self.common).is_ok_and(|now| {
            now.identity == self.identity
                && now.gitdir == self.gitdir
                && now.git_identity == self.git_identity
                && now.common_identity == self.common_identity
                && now.dotgit == self.dotgit
                && now.commondir == self.commondir
                && now.backlink == self.backlink
        })
    }
}
#[derive(Debug)]
struct Root {
    receipt: Receipt,
    _held: Dir,
}
impl WorkspaceRootCapability for Root {
    fn root(&self) -> &Path {
        &self.receipt.root
    }
    fn validate(&self) -> bool {
        self.receipt.valid()
    }
    fn open_read(&self, relative: &Path) -> std::io::Result<std::fs::File> {
        self.checked(|| Ok(self._held.open_with(relative, &read_options())?.into_std()))
    }
    fn files(
        &self,
        relative: &Path,
        max_entries: usize,
        glob_mode: bool,
        max_depth: Option<usize>,
    ) -> std::io::Result<Vec<PathBuf>> {
        self.checked(|| {
            let relative = if relative.as_os_str().is_empty() {
                Path::new(".")
            } else {
                relative
            };
            if self._held.metadata(relative)?.is_file() {
                return Ok(vec![relative.to_owned()]);
            }
            let started = std::time::Instant::now();
            let mut pending = vec![(relative.to_owned(), 0usize)];
            let mut files = Vec::new();
            let mut seen = 0;
            while let Some((path, depth)) = pending.pop() {
                for entry in self._held.read_dir(&path)? {
                    seen += 1;
                    if seen > max_entries || started.elapsed() > std::time::Duration::from_secs(2) {
                        return Err(std::io::Error::other("workspace traversal budget exceeded"));
                    }
                    let entry = entry?;
                    let name = entry.file_name();
                    let text = name.to_string_lossy();
                    if !glob_mode
                        && (text.starts_with('.')
                            || matches!(text.as_ref(), "target" | "node_modules"))
                    {
                        continue;
                    }
                    let kind = entry.file_type()?;
                    let child = path.join(name);
                    if kind.is_dir() {
                        if glob_mode {
                            files.push(child.clone());
                        }
                        if max_depth.is_none_or(|limit| depth + 1 < limit) {
                            pending.push((child, depth + 1));
                        }
                    } else if kind.is_file() {
                        files.push(child);
                    }
                }
            }
            if !self.validate() {
                return Err(std::io::Error::other("workspace identity changed"));
            }
            Ok(files)
        })
    }
    fn create_file(&self, relative: &Path, contents: &[u8]) -> std::io::Result<()> {
        self.checked(|| {
            use std::io::Write;
            if let Some(parent) = relative
                .parent()
                .filter(|path| !path.as_os_str().is_empty())
            {
                self._held.create_dir_all(parent)?;
            }
            let mut options = cap_std::fs::OpenOptions::new();
            options.write(true).create_new(true);
            self._held
                .open_with(relative, &options)?
                .write_all(contents)
        })
    }
    fn replace_file(
        &self,
        relative: &Path,
        expected: &[u8],
        contents: &[u8],
    ) -> std::io::Result<()> {
        self.checked(|| {
            use std::io::Write;
            let parent = self._held.open_dir(
                relative
                    .parent()
                    .filter(|path| !path.as_os_str().is_empty())
                    .unwrap_or(Path::new(".")),
            )?;
            let name = relative
                .file_name()
                .ok_or_else(|| std::io::Error::other("missing filename"))?;
            let file = parent.open_with(name, &read_options())?;
            let meta = file.metadata()?;
            if !meta.is_file() {
                return Err(std::io::Error::other("not a regular file"));
            }
            let mut current = Vec::new();
            file.take(expected.len() as u64 + 1)
                .read_to_end(&mut current)?;
            if current != expected {
                return Err(std::io::Error::other("file revision changed"));
            }
            let temporary = format!(".talos-{}.tmp", uuid::Uuid::new_v4());
            let result = (|| {
                let mut options = cap_std::fs::OpenOptions::new();
                options.write(true).create_new(true);
                let mut output = parent.open_with(&temporary, &options)?;
                output.write_all(contents)?;
                output.set_permissions(meta.permissions())?;
                output.sync_all()?;
                drop(output);
                if !self.validate() {
                    return Err(std::io::Error::other("workspace identity changed"));
                }
                parent.rename(&temporary, &parent, name)
            })();
            if result.is_err() {
                let _ = parent.remove_file(&temporary);
            }
            result
        })
    }
}
impl Root {
    fn checked<T>(&self, action: impl FnOnce() -> std::io::Result<T>) -> std::io::Result<T> {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            if !self.validate() {
                return Err(std::io::Error::other("workspace identity changed"));
            }
            action()
        }))
        .map_err(|_| std::io::Error::other("workspace capability unavailable"))?
    }
}
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Store {
    version: u32,
    session: String,
    original: PathBuf,
    roots: Vec<Receipt>,
}

/// A logical Session's worktree creation and membership owner.
/// The store must be host-owned and excluded from ordinary tool writes/imports.
pub struct SessionWorktrees {
    original: PathBuf,
    common: PathBuf,
    session: String,
    store: PathBuf,
    state: Arc<PermissionSessionState>,
    roots: Mutex<Vec<Arc<Root>>>,
    activated: Mutex<Option<talos_permission::PermissionSessionId>>,
    executor: crate::ExecTool,
    git: PathBuf,
    git_identity: Identity,
    _git_handle: cap_std::fs::File,
}
impl SessionWorktrees {
    /// Opens a bounded receipt store and revalidates every persisted worktree.
    pub fn open(
        original: &Path,
        session: &str,
        store_dir: &Path,
        state: Arc<PermissionSessionState>,
    ) -> Result<Arc<Self>> {
        if session.is_empty()
            || !session
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-')
        {
            return Err("invalid Session identity".into());
        }
        let original = original.canonicalize().map_err(|e| e.to_string())?;
        let git = original.join(".git");
        let common = if git.is_dir() {
            git.canonicalize()
        } else {
            let value = read_link_file(&git)?;
            let dir = original.join(
                value
                    .trim()
                    .strip_prefix("gitdir: ")
                    .ok_or("invalid repository")?,
            );
            let value = read_link_file(&dir.join("commondir"))?;
            dir.join(value.trim()).canonicalize()
        }
        .map_err(|e| e.to_string())?;
        state
            .protect_workspace_receipts(store_dir)
            .map_err(|e| e.to_string())?;
        let store = store_dir.join(format!("{session}.worktrees.json"));
        let mut roots = Vec::new();
        if store.exists() {
            let (file, _) = file_identity(&store)?;
            if !file.metadata().map_err(|e| e.to_string())?.is_file() {
                return Err("worktree store is not a regular file".into());
            }
            let mut bytes = Vec::new();
            file.take(256 * 1024 + 1)
                .read_to_end(&mut bytes)
                .map_err(|e| e.to_string())?;
            if bytes.len() > 256 * 1024 {
                return Err("oversized worktree store".into());
            }
            let saved: Store = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            if saved.version != 1
                || saved.session != session
                || saved.original != original
                || saved.roots.len() > 32
            {
                return Err("worktree store binding mismatch".into());
            }
            for receipt in saved.roots {
                if receipt.common == common && receipt.valid() {
                    let (held, identity) = directory(&receipt.root)?;
                    if identity == receipt.identity {
                        roots.push(Arc::new(Root {
                            receipt,
                            _held: held,
                        }));
                    }
                }
            }
        }
        let git = executable()?;
        let (git_handle, git_identity) = file_identity(&git)?;
        let host = Arc::new(Self {
            git,
            git_identity,
            _git_handle: git_handle,
            executor: crate::ExecTool::new(original.clone()),
            original,
            common,
            session: session.into(),
            store,
            state,
            roots: Mutex::new(roots),
            activated: Mutex::new(None),
        });
        Ok(host)
    }
    /// Publishes this runtime's roots only when it begins execution, after handoff.
    pub fn activate(&self) -> Result<()> {
        let mut active = self
            .activated
            .lock()
            .map_err(|_| "workspace activation unavailable")?;
        let current = self.state.session_id().map_err(|e| e.to_string())?;
        if *active != Some(current) {
            self.publish(None)?;
            *active = Some(current);
        }
        Ok(())
    }
    fn publish(&self, extra: Option<Arc<Root>>) -> Result<()> {
        let mut roots = self
            .roots
            .lock()
            .map_err(|_| "workspace state unavailable")?;
        let mut next: Vec<_> = roots.iter().filter(|r| r.validate()).cloned().collect();
        if let Some(root) = extra {
            next.push(root);
        }
        if next.len() > 32 {
            return Err("Session worktree limit reached".into());
        }
        let saved = Store {
            version: 1,
            session: self.session.clone(),
            original: self.original.clone(),
            roots: next.iter().map(|r| r.receipt.clone()).collect(),
        };
        let bytes = serde_json::to_vec(&saved).map_err(|e| e.to_string())?;
        let capabilities = next
            .iter()
            .cloned()
            .map(|r| r as Arc<dyn WorkspaceRootCapability>)
            .collect();
        self.state
            .install_workspace_roots(capabilities, || {
                let parent = self.store.parent().ok_or("missing store parent")?;
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                let temporary =
                    parent.join(format!(".{}.{}.tmp", self.session, uuid::Uuid::new_v4()));
                use std::io::Write;
                let mut file = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&temporary)
                    .map_err(|e| e.to_string())?;
                file.write_all(&bytes)
                    .and_then(|_| file.sync_all())
                    .map_err(|e| e.to_string())?;
                std::fs::rename(&temporary, &self.store).map_err(|e| e.to_string())
            })
            .map_err(|e| e.to_string())?;
        *roots = next;
        Ok(())
    }
    pub(crate) fn review_configuration(&self) -> serde_json::Value {
        serde_json::json!({
            "backend": "direct-argv",
            "executable": self.git,
            "executable_identity": self.git_identity,
            "working_directory": self.original,
            "routing": "literal-worktree-add-v1; approved input determines exact argv and destination; all other commands use shell",
        })
    }

    pub(crate) fn cleanup_receipt(&self) -> Option<Arc<dyn talos_core::tool::ToolCleanupOwner>> {
        self.executor.cleanup_receipt()
    }

    /// Returns the exact destination for supported literal foreground commands.
    pub fn destination(&self, input: &serde_json::Value) -> Option<PathBuf> {
        let args = literal_add(input)?;
        let path = args.get(args.len().checked_sub(2)?)?;
        // parser always appends a start point, so destination is penultimate.
        talos_core::tool::normalize_authorized_path(&self.original, path).ok()
    }
    /// Executes a recognized creation through the owned direct-argv runner.
    /// Must only be called after all shell and exact destination facets are admitted.
    pub async fn execute(&self, input: &serde_json::Value) -> Option<ToolResult> {
        let mut args = literal_add(input)?;
        if let Err(error) = self.activate() {
            return Some(ToolResult::error(error));
        }
        let target = self.destination(input)?;
        let result = self.create(&mut args, &target, input).await;
        Some(match result {
            Ok(output) => output,
            Err(error) => ToolResult::error(error),
        })
    }
    async fn create(
        &self,
        args: &mut [String],
        target: &Path,
        input: &serde_json::Value,
    ) -> Result<ToolResult> {
        let (_, current) = file_identity(&self.git)?;
        if current != self.git_identity {
            return Err("Git executable identity changed".into());
        }
        let git = &self.git;
        if std::env::vars_os().any(|(key, _)| {
            let key = key.to_string_lossy();
            key.starts_with("GIT_") && !matches!(key.as_ref(), "GIT_PAGER" | "GIT_TERMINAL_PROMPT")
        }) {
            return Err("worktree registration requires an unredirected Git environment".into());
        }
        let parent = target.parent().ok_or("missing destination parent")?;
        if !parent.is_dir() {
            return Err("worktree destination parent must exist".into());
        }
        let staging = parent.join(format!(".talos-worktree-{}", uuid::Uuid::new_v4()));
        self.state
            .protect_workspace_receipts(&staging)
            .map_err(|e| e.to_string())?;
        let (held, identity) = publish_empty_directory(&staging, target)?;
        let index = args.len() - 2;
        args[index] = git_directory_argument(target)?;
        let mut command = serde_json::json!({"command":git,"args":args,"timeout_secs":input.get("timeout_secs").and_then(|v|v.as_u64()).unwrap_or(120)});
        if let Some(memory) = input.get("memory_limit_mib") {
            command["memory_limit_mib"] = memory.clone();
        }
        let output = self.executor.execute(command).await;
        if output.is_error {
            if directory(target).is_ok_and(|(_, now)| now == identity) {
                let _ = std::fs::remove_dir(target);
            }
            return Ok(output);
        }
        let receipt = Receipt::capture(target, &self.common)?;
        if receipt.identity != identity {
            return Err("created worktree identity changed".into());
        }
        self.publish(Some(Arc::new(Root {
            receipt,
            _held: held,
        })))?;
        Ok(ToolResult::success(format!(
            "{}\nSession workspace registered: {}",
            output.content,
            target.display()
        )))
    }
}
fn git_directory_argument(path: &Path) -> Result<String> {
    let original = path.to_str().ok_or("Git worktree path is not Unicode")?;
    #[cfg(not(windows))]
    return Ok(original.to_owned());
    #[cfg(windows)]
    {
        // Git for Windows rewrites backslashes but does not preserve Rust's
        // verbatim namespace. Convert only an unchanged Win32 spelling that
        // resolves to the held directory, never merely strip the prefix.
        let spelling = if let Some(unc) = original.strip_prefix(r"\\?\UNC\") {
            format!(r"\\{unc}")
        } else if let Some(disk) = original.strip_prefix(r"\\?\") {
            disk.to_owned()
        } else {
            original.to_owned()
        };
        let candidate = Path::new(&spelling);
        let absolute = std::path::absolute(candidate).map_err(|e| e.to_string())?;
        if !candidate.is_absolute() || absolute.as_os_str() != candidate.as_os_str() {
            return Err("Git worktree path changes under Win32 normalization".into());
        }
        for part in candidate.components() {
            if let std::path::Component::Normal(name) = part {
                let name = name.to_str().ok_or("invalid Git path component")?;
                let stem = name
                    .split('.')
                    .next()
                    .unwrap_or_default()
                    .to_ascii_uppercase();
                let reserved = matches!(
                    stem.as_str(),
                    "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
                ) || ["COM", "LPT"].iter().any(|prefix| {
                    stem.strip_prefix(prefix).is_some_and(|suffix| {
                        matches!(
                            suffix,
                            "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
                        )
                    })
                });
                if reserved || name.ends_with(['.', ' ']) || name.contains(':') {
                    return Err("Git worktree path requires verbatim semantics".into());
                }
            }
        }
        if directory(candidate)?.1 != directory(path)?.1 {
            return Err("Git worktree path identity changed".into());
        }
        Ok(spelling)
    }
}

fn publish_empty_directory(staging: &Path, target: &Path) -> Result<(Dir, Identity)> {
    std::panic::catch_unwind(|| {
        let builder = std::fs::DirBuilder::new();
        #[cfg(unix)]
        let mut builder = builder;
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(staging).map_err(|e| e.to_string())?;
        let (parent, _) = directory(staging)?;
        parent.create_dir("root").map_err(|e| e.to_string())?;
        #[cfg(not(windows))]
        let held = parent.open_dir("root").map_err(|e| e.to_string())?;
        #[cfg(not(windows))]
        let metadata = held.dir_metadata().map_err(|e| e.to_string())?;
        #[cfg(windows)]
        let custody = {
            use std::os::windows::fs::OpenOptionsExt;
            use windows_sys::Win32::Storage::FileSystem::{
                FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_DELETE,
                FILE_SHARE_READ, FILE_SHARE_WRITE,
            };
            // cap-std directory handles intentionally prohibit rename. This
            // identity-only handle permits atomic publication; capability IO
            // starts only after opening the published directory below.
            let file = std::fs::OpenOptions::new()
                .read(true)
                .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
                .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
                .open(staging.join("root"))
                .map_err(|e| e.to_string())?;
            cap_std::fs::File::from_std(file)
        };
        #[cfg(windows)]
        let metadata = custody.metadata().map_err(|e| e.to_string())?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err("staged worktree is not an ordinary directory".into());
        }
        let identity = Identity {
            device: metadata.dev(),
            inode: metadata.ino(),
        };
        let result = publish_no_replace(staging, target);
        if let Err(error) = result {
            let _ = parent.remove_dir("root");
            drop(parent);
            let _ = std::fs::remove_dir(staging);
            return Err(error.to_string());
        }
        drop(parent);
        let _ = std::fs::remove_dir(staging);
        let (published, published_identity) = directory(target)?;
        if published_identity != identity {
            return Err("published worktree identity changed".into());
        }
        #[cfg(windows)]
        let held = published;
        #[cfg(not(windows))]
        drop(published);
        Ok((held, identity))
    })
    .map_err(|_| "worktree publication unavailable".to_owned())?
}

fn publish_no_replace(staging: &Path, target: &Path) -> std::io::Result<()> {
    #[cfg(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "ios",
        target_os = "android"
    ))]
    {
        let source = Dir::open_ambient_dir(staging, cap_std::ambient_authority())?;
        let destination = Dir::open_ambient_dir(
            target
                .parent()
                .ok_or_else(|| std::io::Error::other("missing parent"))?,
            cap_std::ambient_authority(),
        )?;
        rustix::fs::renameat_with(
            &source,
            "root",
            &destination,
            target
                .file_name()
                .ok_or_else(|| std::io::Error::other("missing filename"))?,
            rustix::fs::RenameFlags::NOREPLACE,
        )?;
        Ok(())
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        fn wide(path: &Path) -> std::io::Result<Vec<u16>> {
            let mut value: Vec<_> = path.as_os_str().encode_wide().collect();
            if value.contains(&0) {
                return Err(std::io::Error::other("embedded NUL in path"));
            }
            value.push(0);
            Ok(value)
        }
        let source = wide(&staging.join("root"))?;
        let destination = wide(target)?;
        // SAFETY (ADR-099): both NUL-terminated UTF-16 buffers remain alive for
        // this synchronous call. Flags 0 forbid replacement and copy fallback.
        let result = unsafe {
            windows_sys::Win32::Storage::FileSystem::MoveFileExW(
                source.as_ptr(),
                destination.as_ptr(),
                0,
            )
        };
        if result == 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(())
    }
    #[cfg(not(any(
        windows,
        target_os = "linux",
        target_os = "macos",
        target_os = "ios",
        target_os = "android"
    )))]
    {
        let _ = (staging, target);
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "atomic worktree publication unsupported",
        ))
    }
}

fn executable() -> Result<PathBuf> {
    let name = if cfg!(windows) { "git.exe" } else { "git" };
    std::env::var_os("PATH")
        .into_iter()
        .flat_map(|p| std::env::split_paths(&p).collect::<Vec<_>>())
        .map(|p| p.join(name))
        .find(|p| p.is_file())
        .ok_or_else(|| "Git executable unavailable".to_owned())?
        .canonicalize()
        .map_err(|e| e.to_string())
}
fn literal_add(input: &serde_json::Value) -> Option<Vec<String>> {
    if input.get("background").and_then(|v| v.as_bool()) == Some(true) {
        return None;
    }
    let command = input.get("command")?.as_str()?;
    if command.len() > 8192
        || command
            .chars()
            .any(|c| c.is_control() || "$`;|&<>*?!{}()\\~#%[]".contains(c))
    {
        return None;
    }
    let mut words = Vec::new();
    let mut word = String::new();
    let mut quote = None;
    let mut started = false;
    for c in command.chars() {
        if let Some(q) = quote {
            if c == q {
                quote = None;
            } else {
                word.push(c);
            }
        } else if c == '\'' || c == '"' {
            started = true;
            quote = Some(c);
        } else if c.is_whitespace() {
            if started {
                words.push(std::mem::take(&mut word));
                started = false;
            }
        } else {
            started = true;
            word.push(c);
        }
    }
    if quote.is_some() {
        return None;
    }
    if started {
        words.push(word);
    }
    if words.get(0..3)? != ["git", "worktree", "add"] {
        return None;
    }
    let mut i = 3;
    if words.get(i).map(String::as_str) == Some("-b") {
        if words
            .get(i + 1)
            .is_none_or(|branch| branch.is_empty() || branch.starts_with('-'))
        {
            return None;
        }
        i += 2;
    } else if words.get(i).map(String::as_str) == Some("--detach") {
        i += 1;
    }
    let path = words.get(i)?;
    if path.starts_with('-') || path.is_empty() || words.len() > i + 2 {
        return None;
    }
    if words.get(i + 1).is_some_and(|word| word.starts_with('-')) {
        return None;
    }
    let mut args = words[1..].to_vec();
    if words.len() == i + 1 {
        if i == 3 {
            return None;
        }
        args.push("HEAD".into());
    }
    Some(args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_core::tool::{ToolNature, ToolPermissionFacet, ToolResourceKind};
    fn git(root: &Path, args: &[&str]) {
        let output = std::process::Command::new("git")
            .current_dir(root)
            .args(args)
            .output()
            .expect("git");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    #[tokio::test]
    async fn owned_creation_registers_resumes_and_rejects_replacement() {
        let temp = tempfile::tempdir().expect("temp");
        let root = temp.path().join("repo");
        std::fs::create_dir(&root).expect("repo");
        git(&root, &["init"]);
        git(
            &root,
            &[
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.invalid",
                "commit",
                "--allow-empty",
                "-m",
                "initial",
            ],
        );
        let make_state = || {
            let mut engine = talos_permission::PermissionEngine::with_workspace_root(root.clone());
            engine.set_ordinary_workspace_files(true);
            Arc::new(PermissionSessionState::new(engine))
        };
        let state = make_state();
        let store = temp.path().join("store");
        let host =
            SessionWorktrees::open(&root, "test-session", &store, state.clone()).expect("host");
        let input = serde_json::json!({"command":"git worktree add -b linked ../linked"});
        let output = host.execute(&input).await.expect("recognized");
        assert!(!output.is_error, "{}", output.content);
        let target = temp.path().join("linked/new.txt");
        let value = serde_json::json!({"path":target,"content":"hello"});
        let profile = [ToolPermissionFacet::with_resource(
            ToolNature::Write,
            target.to_string_lossy(),
            ToolResourceKind::Path,
        )];
        let request = talos_permission::PermissionRequest::native("write", &profile, &value)
            .with_ordinary_workspace_file();
        let context = talos_permission::PermissionContext::compatibility();
        assert_eq!(
            state
                .evaluate(&request, &context)
                .expect("evaluate")
                .decision(),
            talos_permission::PermissionDecision::Allow
        );
        let pending = match state.begin_invocation(&request, &context).expect("begin") {
            talos_permission::PermissionInvocation::Allow(p) => p,
            _ => panic!("not allowed"),
        };
        let tokens = state.admit(*pending, &request, &context).expect("admit");
        let write = crate::WriteTool::new(root.clone()).with_atomic_create_capability(Arc::new(
            crate::file_tools::CapStdAtomicCreateCapability::open(&root).expect("cap"),
        ));
        assert!(
            !write
                .execute_authorized(value.clone(), &tokens)
                .await
                .is_error
        );
        for (name, input) in [
            ("read", serde_json::json!({"path":target})),
            (
                "edit",
                serde_json::json!({"path":target,"old_string":"hello","new_string":"updated"}),
            ),
            (
                "write",
                serde_json::json!({"path":temp.path().join("linked/without-cap.txt"),"content":"second"}),
            ),
            (
                "grep",
                serde_json::json!({"path":temp.path().join("linked"),"pattern":"updated"}),
            ),
            (
                "glob",
                serde_json::json!({"path":temp.path().join("linked"),"pattern":"*.txt"}),
            ),
        ] {
            let tool: Box<dyn AgentTool> = match name {
                "read" => Box::new(crate::ReadTool::new(root.clone())),
                "edit" => Box::new(crate::EditTool::new(root.clone())),
                "write" => Box::new(crate::WriteTool::new(root.clone())),
                "grep" => Box::new(crate::GrepTool::new(root.clone())),
                "glob" => Box::new(crate::GlobTool::new(root.clone())),
                _ => unreachable!(),
            };
            let facets = tool.permission_profile(&input);
            let operation = talos_permission::PermissionRequest::native(name, &facets, &input)
                .with_ordinary_workspace_file();
            let pending = match state
                .begin_invocation(&operation, &context)
                .expect("begin operation")
            {
                talos_permission::PermissionInvocation::Allow(pending) => *pending,
                _ => panic!("registered workspace operation must be allowed: {name}"),
            };
            let authorization = state
                .admit(pending, &operation, &context)
                .expect("operation admission");
            let result = tool.execute_authorized(input.clone(), &authorization).await;
            assert!(!result.is_error, "{name}: {}", result.content);
            if name == "grep" {
                assert!(result.content.contains("updated"));
            }
            if name == "glob" {
                assert!(result.content.contains("without-cap.txt"));
            }
        }
        let large = temp.path().join("linked/large");
        std::fs::create_dir(&large).expect("large directory");
        for index in 0..32 {
            std::fs::write(large.join(format!("{index}.txt")), "content").expect("child");
        }
        {
            let roots = host.roots.lock().expect("roots");
            let shallow = roots[0]
                .files(Path::new(""), 10, true, Some(1))
                .expect("shallow glob must not scan child directories");
            assert!(shallow.iter().any(|path| path.ends_with("large")));
            assert!(shallow.iter().any(|path| path.ends_with(".git")));
            assert!(
                roots[0].files(Path::new(""), 10, true, None).is_err(),
                "recursive query retains traversal bound"
            );
        }
        let mut reloaded = talos_permission::PermissionEngine::with_workspace_root(root.clone());
        reloaded.set_ordinary_workspace_files(true);
        state.replace_policy(reloaded).expect("reload policy");
        assert_eq!(
            state
                .evaluate(&request, &context)
                .expect("membership after policy reload")
                .decision(),
            talos_permission::PermissionDecision::Allow
        );
        let receipt_input = serde_json::json!({"path":store.join("test-session.worktrees.json")});
        let receipt_facets = [ToolPermissionFacet::with_resource(
            ToolNature::Write,
            store.join("test-session.worktrees.json").to_string_lossy(),
            ToolResourceKind::Path,
        )];
        let receipt_request =
            talos_permission::PermissionRequest::native("write", &receipt_facets, &receipt_input);
        assert!(matches!(
            state
                .evaluate(&receipt_request, &context)
                .expect("receipt policy")
                .decision(),
            talos_permission::PermissionDecision::Deny(_)
        ));
        // Trust of the original root must not expand non-capability tool authority.
        let mut trusted_engine =
            talos_permission::PermissionEngine::with_workspace_root(root.clone());
        trusted_engine.set_trusted_workspace(true);
        trusted_engine
            .load_from_config(&serde_json::json!({"rules":[{"tool":"delete","decision":"Ask"}]}))
            .expect("explicit ask");
        let trusted = Arc::new(PermissionSessionState::new(trusted_engine));
        let trusted_host = SessionWorktrees::open(&root, "test-session", &store, trusted.clone())
            .expect("trusted host");
        trusted_host.activate().expect("trusted activation");
        for (name, nature) in [
            ("delete", ToolNature::Write),
            ("ls", ToolNature::Read),
            ("read_image", ToolNature::Read),
        ] {
            let facets = [ToolPermissionFacet::with_resource(
                nature,
                target.to_string_lossy(),
                ToolResourceKind::Path,
            )];
            let operation = talos_permission::PermissionRequest::native(name, &facets, &value);
            assert_eq!(
                trusted
                    .evaluate(&operation, &context)
                    .expect("evaluate")
                    .decision(),
                talos_permission::PermissionDecision::Ask
            );
        }
        let restored = make_state();
        let host2 = SessionWorktrees::open(&root, "test-session", &store, restored.clone())
            .expect("restore");
        host2.activate().expect("activate");
        assert_eq!(
            restored
                .evaluate(&request, &context)
                .expect("evaluate")
                .decision(),
            talos_permission::PermissionDecision::Allow
        );
        let unrelated = make_state();
        let other = SessionWorktrees::open(&root, "different-session", &store, unrelated.clone())
            .expect("other");
        other.activate().expect("activate");
        assert_eq!(
            unrelated
                .evaluate(&request, &context)
                .expect("evaluate")
                .decision(),
            talos_permission::PermissionDecision::Ask
        );
        // Ask-only requests deliberately bypass the ordinary-file convenience default.
        let asked = talos_permission::PermissionRequest::native("write", &profile, &value);
        let once = state
            .propose(&asked, &context, talos_permission::GrantScope::Once)
            .expect("once proposal");
        let once = state
            .approve_once(once, &asked, &context)
            .expect("once approval");
        let session = state
            .propose(&asked, &context, talos_permission::GrantScope::Session)
            .expect("session proposal");
        let session = state
            .approve_session(
                session,
                &asked,
                &context,
                talos_permission::GrantSource::InteractiveHuman,
            )
            .expect("session approval");
        assert_eq!(state.grant_count().expect("count"), 1);
        #[cfg(not(windows))]
        {
            std::fs::rename(temp.path().join("linked"), temp.path().join("old-linked"))
                .expect("move");
            std::fs::create_dir(temp.path().join("linked")).expect("replacement");
        }
        #[cfg(windows)]
        {
            // Retained cap-std directory handles prevent root replacement on
            // Windows. Verify that defense, then revoke via metadata identity.
            assert!(
                std::fs::rename(temp.path().join("linked"), temp.path().join("old-linked"))
                    .is_err()
            );
            let metadata = temp.path().join("linked/.git");
            let contents = std::fs::read(&metadata).expect("metadata");
            std::fs::rename(&metadata, temp.path().join("old-dotgit")).expect("move metadata");
            std::fs::write(metadata, contents).expect("replacement metadata");
        }
        assert_eq!(
            state
                .evaluate(&request, &context)
                .expect("evaluate")
                .decision(),
            talos_permission::PermissionDecision::Ask
        );
        assert!(
            state.admit(once, &asked, &context).is_err(),
            "old once revoked"
        );
        assert!(
            state.admit(session, &asked, &context).is_err(),
            "old session revoked"
        );
        assert_eq!(state.grant_count().expect("count"), 0);
        assert!(
            write
                .execute_authorized(value.clone(), &tokens)
                .await
                .is_error
        );
        // Windows revokes through metadata replacement, so the old output
        // remains in this directory. Fresh create authorization still requires
        // a missing target on every platform.
        #[cfg(windows)]
        std::fs::remove_file(&target).expect("remove old worktree output");
        for scope in [
            talos_permission::GrantScope::Once,
            talos_permission::GrantScope::Session,
        ] {
            let proposal = state
                .propose(&asked, &context, scope)
                .expect("fresh external approval");
            let pending = match scope {
                talos_permission::GrantScope::Once => {
                    state.approve_once(proposal, &asked, &context)
                }
                talos_permission::GrantScope::Session => state.approve_session(
                    proposal,
                    &asked,
                    &context,
                    talos_permission::GrantSource::InteractiveHuman,
                ),
            }
            .expect("fresh approval");
            let fresh = state
                .admit(pending, &asked, &context)
                .expect("fresh admission");
            assert!(
                fresh
                    .iter()
                    .all(|token| token.workspace_capability().is_none())
            );
            let result = write.execute_authorized(value.clone(), &fresh).await;
            assert!(!result.is_error, "{}", result.content);
            std::fs::remove_file(&target).expect("reset target");
        }
    }
    #[cfg(windows)]
    #[test]
    fn git_directory_spelling_preserves_identity_and_rejects_verbatim_only_names() {
        let temp = tempfile::tempdir().expect("temp");
        let canonical = temp.path().canonicalize().expect("canonical");
        let spelling = git_directory_argument(&canonical).expect("Git path");
        assert!(!spelling.starts_with(r"\\?\"));
        assert_eq!(
            directory(Path::new(&spelling)).expect("spelling").1,
            directory(&canonical).expect("canonical").1
        );
        for name in ["trailing.", "trailing ", "NUL", "COM1"] {
            assert!(git_directory_argument(&canonical.join(name)).is_err());
        }
    }
    #[test]
    fn publication_never_replaces_existing_objects() {
        for kind in ["empty", "nonempty", "file"] {
            let temp = tempfile::tempdir().expect("temp");
            let target = temp.path().join("target");
            if kind == "file" {
                std::fs::write(&target, "original").expect("file");
            } else {
                std::fs::create_dir(&target).expect("dir");
                if kind == "nonempty" {
                    std::fs::write(target.join("keep"), "original").expect("keep");
                }
            }
            let staging = temp.path().join("staging");
            assert!(publish_empty_directory(&staging, &target).is_err());
            if kind == "file" {
                assert_eq!(std::fs::read_to_string(&target).expect("read"), "original");
            } else {
                assert!(target.is_dir());
                if kind == "nonempty" {
                    assert_eq!(
                        std::fs::read_to_string(target.join("keep")).expect("read"),
                        "original"
                    );
                }
            }
        }
    }

    #[test]
    fn publication_rejects_competing_destination_after_staging() {
        let temp = tempfile::tempdir().expect("temp");
        let staging = temp.path().join("staging");
        let target = temp.path().join("target");
        std::fs::create_dir_all(staging.join("root")).expect("stage");
        let competitor = target.clone();
        std::thread::spawn(move || std::fs::create_dir(competitor).expect("competitor"))
            .join()
            .expect("join");
        let (_, before) = directory(&target).expect("identity");
        assert!(publish_no_replace(&staging, &target).is_err());
        assert_eq!(directory(&target).expect("identity").1, before);
        assert!(staging.join("root").is_dir());
    }

    #[cfg(windows)]
    #[test]
    fn publication_does_not_replace_destination_junction() {
        let temp = tempfile::tempdir().expect("temp");
        let elsewhere = temp.path().join("elsewhere");
        let target = temp.path().join("target");
        std::fs::create_dir(&elsewhere).expect("elsewhere");
        let status = std::process::Command::new("cmd.exe")
            .args(["/D", "/C", "mklink", "/J"])
            .arg(&target)
            .arg(&elsewhere)
            .status()
            .expect("junction command");
        assert!(status.success());
        assert!(publish_empty_directory(&temp.path().join("staging"), &target).is_err());
        assert_eq!(
            target.canonicalize().expect("target"),
            elsewhere.canonicalize().expect("elsewhere")
        );
        std::fs::remove_dir(&target).expect("remove junction");
    }

    #[cfg(unix)]
    #[test]
    fn publication_does_not_follow_destination_symlink() {
        let temp = tempfile::tempdir().expect("temp");
        let elsewhere = temp.path().join("elsewhere");
        std::fs::create_dir(&elsewhere).expect("elsewhere");
        let target = temp.path().join("target");
        std::os::unix::fs::symlink(&elsewhere, &target).expect("symlink");
        assert!(publish_empty_directory(&temp.path().join("staging"), &target).is_err());
        assert!(
            std::fs::symlink_metadata(&target)
                .expect("metadata")
                .file_type()
                .is_symlink()
        );
    }

    #[test]
    fn ambiguous_or_dynamic_commands_never_register() {
        for command in [
            "git worktree add $TARGET",
            "git worktree add ../implicit-branch",
            "git worktree add -b x ~/expanded",
            "git worktree add -b '' ../empty-branch",
            "git worktree add ../x; echo yes",
            "echo git worktree add ../x",
            "git worktree add ../x && true",
            "git -C elsewhere worktree add ../x",
        ] {
            assert!(
                literal_add(&serde_json::json!({"command":command})).is_none(),
                "{command}"
            );
        }
    }
}
