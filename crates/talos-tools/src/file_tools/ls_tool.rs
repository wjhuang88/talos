use std::path::{Path, PathBuf};

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use talos_core::tool::{AgentTool, ToolResult};
use talos_core::tool_parameters;

use super::{FileToolError, is_skip_dir, resolve_workspace_path};

/// Input parameters for the [`LsTool`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LsInput {
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub all: bool,
    #[serde(default)]
    pub recursive: bool,
    #[serde(default)]
    pub long: bool,
}

/// A tool that lists directory contents with file metadata.
pub struct LsTool {
    workspace_root: PathBuf,
}

impl LsTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    async fn execute_inner(&self, input: Value) -> Result<String, FileToolError> {
        let ls_input: LsInput = serde_json::from_value(input)
            .map_err(|e| FileToolError::InvalidInput(e.to_string()))?;

        let canonical_root = self
            .workspace_root
            .canonicalize()
            .unwrap_or_else(|_| self.workspace_root.clone());

        let target = match &ls_input.path {
            Some(p) => resolve_workspace_path(&self.workspace_root, p)?,
            None => canonical_root.clone(),
        };

        if !target.exists() {
            return Err(FileToolError::FileNotFound(
                ls_input.path.unwrap_or_default(),
            ));
        }

        if target.is_file() {
            return Ok(format_entry(&target, &canonical_root, ls_input.long));
        }

        if ls_input.recursive {
            let mut output = String::new();
            for entry in walkdir::WalkDir::new(&target)
                .into_iter()
                .filter_entry(|e| {
                    if e.depth() == 0 {
                        return true;
                    }
                    if !ls_input.all && is_hidden_entry(e.file_name()) {
                        return false;
                    }
                    !(e.file_type().is_dir() && is_skip_dir(&e.file_name().to_string_lossy()))
                })
                .filter_map(Result::ok)
                .filter(|e| e.depth() > 0)
            {
                output.push_str(&format_entry(entry.path(), &canonical_root, ls_input.long));
                output.push('\n');
            }
            return Ok(output.trim_end().to_string());
        }

        let mut entries: Vec<_> = std::fs::read_dir(&target)?
            .filter_map(Result::ok)
            .filter(|e| ls_input.all || !is_hidden_entry(&e.file_name()))
            .collect();

        entries.sort_by_key(|e| {
            let is_dir = e.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
            (!is_dir, e.file_name())
        });

        let mut output = String::new();
        for entry in &entries {
            output.push_str(&format_entry(&entry.path(), &canonical_root, ls_input.long));
            output.push('\n');
        }

        Ok(output.trim_end().to_string())
    }
}

#[async_trait]
impl AgentTool for LsTool {
    fn name(&self) -> &str {
        "ls"
    }

    fn description(&self) -> &str {
        "List directory contents (supports long format, hidden files, recursive)"
    }

    fn parameters(&self) -> Value {
        tool_parameters!(LsInput)
    }

    async fn execute(&self, input: Value) -> ToolResult {
        match self.execute_inner(input).await {
            Ok(content) => ToolResult::success(content),
            Err(e) => ToolResult::error(e.to_string()),
        }
    }

    fn is_read_only(&self) -> bool {
        true
    }
    fn summary_fields(&self) -> &'static [&'static str] {
        &["path"]
    }
}

fn is_hidden_entry(name: &std::ffi::OsStr) -> bool {
    name.to_string_lossy().starts_with('.')
}

fn format_entry(path: &Path, root: &Path, long: bool) -> String {
    let display = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string();

    if long {
        return format_entry_long(path, &display);
    }

    let meta = std::fs::symlink_metadata(path).ok();
    match &meta {
        Some(m) => {
            let ft = m.file_type();
            if ft.is_dir() {
                format!("{display}/")
            } else if ft.is_symlink() {
                format!("{display}@")
            } else if is_executable(m) {
                format!("{display}* {}", m.len())
            } else {
                format!("{display} {}", m.len())
            }
        }
        None => display,
    }
}

#[cfg(unix)]
fn is_executable(meta: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    meta.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn is_executable(_meta: &std::fs::Metadata) -> bool {
    false
}

#[cfg(unix)]
fn format_entry_long(path: &Path, display: &str) -> String {
    use std::os::unix::fs::MetadataExt;

    let meta = match std::fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(_) => {
            return format!("?---------    -      -      -        -  ????-??-?? ??:??  {display}");
        }
    };

    let ft = meta.file_type();
    let type_char = if ft.is_dir() {
        'd'
    } else if ft.is_symlink() {
        'l'
    } else {
        '-'
    };

    let perms = format_permissions(meta.mode());
    let nlink = meta.nlink();
    let uid = meta.uid();
    let gid = meta.gid();
    let size = meta.len();
    let mtime = format_mtime(meta.mtime());

    format!("{type_char}{perms}  {nlink:>3}  {uid:>5}  {gid:>5}  {size:>8}  {mtime}  {display}")
}

#[cfg(not(unix))]
fn format_entry_long(path: &Path, display: &str) -> String {
    let meta = std::fs::symlink_metadata(path).ok();
    let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
    format!("-         {size:>8}  {display}")
}

#[cfg(unix)]
fn format_permissions(mode: u32) -> String {
    let bits: [(u32, char); 9] = [
        (0o400, 'r'),
        (0o200, 'w'),
        (0o100, 'x'),
        (0o040, 'r'),
        (0o020, 'w'),
        (0o010, 'x'),
        (0o004, 'r'),
        (0o002, 'w'),
        (0o001, 'x'),
    ];
    bits.iter()
        .map(|(bit, c)| if mode & bit != 0 { *c } else { '-' })
        .collect()
}

#[cfg(unix)]
fn format_mtime(mtime: i64) -> String {
    let secs = mtime.max(0) as u64;
    let days = secs / 86400;
    let rem = secs % 86400;
    let hour = rem / 3600;
    let min = (rem % 3600) / 60;

    let z = days as i64 + 719468;
    let era = if z >= 0 {
        z / 146097
    } else {
        (z - 146096) / 146097
    };
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };

    format!("{year:04}-{m:02}-{d:02} {hour:02}:{min:02}")
}
