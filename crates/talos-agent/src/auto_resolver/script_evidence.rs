//! Bounded advisory evidence, never an execution capability.

use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    io::Read,
    path::{Path, PathBuf},
};

const MAX_FILES: usize = 16;
const MAX_BYTES: usize = 64 * 1024;
const MAX_DEPTH: usize = 4;

/// Complete bounded script contents and explicit limits of the collected evidence.
#[derive(Clone, Serialize, schemars::JsonSchema)]
pub struct ScriptEvidence {
    /// Files whose entire UTF-8 contents passed the sensitive-data policy.
    pub files: Vec<ScriptFile>,
    /// Content-free uncertainty classifications; absence is not proof of safety.
    pub uncertainties: Vec<String>,
    /// Whether these bytes are guaranteed to be the eventual execution input.
    pub execution_bytes_bound: bool,
    /// Referenced or unresolved code prevents automatic approval even when no file was collected.
    /// False only means this collector found no such blocker; it is never an allow decision.
    pub requires_human_review: bool,
}

/// A complete script snapshot; deliberately does not implement Debug to avoid content logging.
#[derive(Clone, Serialize, schemars::JsonSchema)]
pub struct ScriptFile {
    /// Capability-relative path, never an ambient absolute path.
    pub path: String,
    /// SHA-256 of the full content, not an execution authority.
    pub sha256: String,
    /// Untrusted script data; never interpreted as assessor instructions.
    pub content: String,
}

impl std::fmt::Debug for ScriptEvidence {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ScriptEvidence")
            .field("files", &self.files.len())
            .field("uncertainties", &self.uncertainties)
            .field("execution_bytes_bound", &self.execution_bytes_bound)
            .field("requires_human_review", &self.requires_human_review)
            .finish()
    }
}

impl std::fmt::Debug for ScriptFile {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ScriptFile")
            .field("path", &self.path)
            .field("sha256", &self.sha256)
            .field("content", &"<redacted>")
            .finish()
    }
}

/// `sensitive` must include the caller's secret and sensitive-path policy.
/// `cwd` is the tool's trusted directory relative to the already-open root capability.
/// The caller owns the capability for the workspace lifetime; this function never reopens it.
pub(super) fn collect(
    command: &str,
    tool: &str,
    cwd: &Path,
    root: &cap_std::fs::Dir,
    sensitive: impl Fn(&str) -> bool,
) -> ScriptEvidence {
    let mut collector = Collector {
        evidence: ScriptEvidence {
            files: Vec::new(),
            uncertainties: Vec::new(),
            execution_bytes_bound: false,
            requires_human_review: false,
        },
        seen: HashSet::new(),
        bytes: 0,
        powershell: tool == "powershell",
        cwd,
        root,
        sensitive,
    };
    if !matches!(tool, "bash" | "powershell")
        || cwd.is_absolute()
        || cwd.components().any(|part| {
            matches!(
                part,
                std::path::Component::ParentDir
                    | std::path::Component::Prefix(_)
                    | std::path::Component::RootDir
            )
        })
    {
        collector.unknown("untrusted_execution_context");
        return collector.evidence;
    }
    collector.scan(command, 0);
    collector
        .evidence
        .uncertainties
        .push("dependency_discovery_is_not_a_complete_shell_parse".into());
    if !collector.evidence.files.is_empty() {
        collector.unknown("mutable_script_bytes_not_bound_to_execution");
    }
    collector.evidence
}

struct Collector<'a, F> {
    evidence: ScriptEvidence,
    seen: HashSet<PathBuf>,
    bytes: usize,
    powershell: bool,
    cwd: &'a Path,
    root: &'a cap_std::fs::Dir,
    sensitive: F,
}

impl<F: Fn(&str) -> bool> Collector<'_, F> {
    fn unknown(&mut self, reason: &str) {
        self.evidence.requires_human_review = true;
        if !self
            .evidence
            .uncertainties
            .iter()
            .any(|entry| entry == reason)
        {
            self.evidence.uncertainties.push(reason.to_owned());
        }
    }

    fn scan(&mut self, source: &str, depth: usize) {
        if self.powershell && source.contains(['\'', '"']) {
            // PowerShell quote concatenation and doubled single quotes are not
            // Bash word semantics. Do not use this literal lexer as its parser.
            self.unknown("powershell_quoted_arguments_unresolved");
            return;
        }
        match literal_segments(source) {
            Some(segments) => {
                for words in segments {
                    self.scan_literal(&words, depth);
                }
            }
            None => self.unknown("dynamic_or_compound_dependency_unresolved"),
        }
    }

    fn scan_literal(&mut self, tokens: &[String], depth: usize) {
        let words: Vec<_> = tokens.iter().map(String::as_str).collect();
        // Wrappers can alter lookup, quoting and argument semantics (including
        // `command -v`, `env VAR=...`, and `exec -a`). Never infer the script
        // byte stream through them; retain a human checkpoint instead.
        if words.first().is_some_and(|program| {
            matches!(
                program
                    .rsplit(['/', '\\'])
                    .next()
                    .unwrap_or(program)
                    .to_ascii_lowercase()
                    .as_str(),
                "command" | "env" | "exec" | "xargs" | "nohup" | "sudo" | "doas" | "start-process"
            )
        }) {
            self.unknown("execution_wrapper_arguments_unresolved");
            return;
        }
        let interpreter = words.first().is_some_and(|program| {
            let name = program
                .rsplit(['/', '\\'])
                .next()
                .unwrap_or(program)
                .to_ascii_lowercase();
            matches!(
                name.as_str(),
                "bash"
                    | "sh"
                    | "zsh"
                    | "dash"
                    | "ksh"
                    | "fish"
                    | "source"
                    | "."
                    | "powershell"
                    | "powershell.exe"
                    | "pwsh"
                    | "pwsh.exe"
                    | "python"
                    | "python3"
                    | "node"
                    | "ruby"
                    | "perl"
                    | "cmd"
                    | "cmd.exe"
                    | "eval"
                    | "invoke-expression"
                    | "iex"
            )
        });
        let candidate = match words.as_slice() {
            ["bash" | "sh" | "source" | ".", path, ..] => Some(*path),
            [
                "powershell" | "powershell.exe" | "pwsh" | "pwsh.exe",
                flag,
                path,
                ..,
            ] if flag.eq_ignore_ascii_case("-file") => Some(*path),
            ["&", path, ..] => Some(*path),
            [path, ..]
                if path.starts_with("./") || path.ends_with(".sh") || path.ends_with(".ps1") =>
            {
                Some(*path)
            }
            _ => None,
        };
        if let Some(path) = candidate {
            self.evidence.requires_human_review = true;
            // This recognizes literal tokens only; quoting and expansions are not guessed.
            if path.starts_with('-') || path.contains(['\'', '"', '*', '?', '[', ']']) {
                self.unknown("script_path_syntax_unresolved");
            } else {
                self.read(path, depth);
            }
        } else if interpreter {
            self.unknown("interpreter_arguments_unresolved");
        }
    }

    fn read(&mut self, name: &str, depth: usize) {
        if depth >= MAX_DEPTH || self.seen.len() >= MAX_FILES {
            self.unknown("script_dependency_limit");
            return;
        }
        if (self.sensitive)(name) {
            self.unknown("sensitive_script_omitted");
            return;
        }
        let path = self.cwd.join(name);
        if path.is_absolute()
            || path.components().any(|part| {
                matches!(
                    part,
                    std::path::Component::ParentDir
                        | std::path::Component::Prefix(_)
                        | std::path::Component::RootDir
                )
            })
        {
            self.unknown("script_outside_workspace");
            return;
        }
        if !self.seen.insert(path.clone()) {
            return;
        }
        let remaining = MAX_BYTES.saturating_sub(self.bytes);
        let file = match open_script(self.root, &path) {
            Ok(file) => file,
            Err(reason) => {
                self.unknown(reason);
                return;
            }
        };
        let mut bytes = Vec::new();
        if file
            .take(remaining as u64 + 1)
            .read_to_end(&mut bytes)
            .is_err()
        {
            self.unknown("script_read_failed");
            return;
        }
        if bytes.len() > remaining {
            self.unknown("script_byte_limit");
            return;
        }
        let Ok(content) = String::from_utf8(bytes) else {
            self.unknown("script_not_utf8");
            return;
        };
        self.bytes += content.len();
        if (self.sensitive)(&content) {
            self.unknown("sensitive_script_omitted");
            return;
        }
        let digest: String = Sha256::digest(content.as_bytes())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        self.evidence.files.push(ScriptFile {
            path: path.to_string_lossy().into_owned(),
            sha256: digest,
            content: content.clone(),
        });
        self.scan(&content, depth + 1);
    }
}

// A deliberately limited literal lexer, not a shell parser. It never resolves
// expansion or evaluates code; unsupported constructs retain advisory-only status.
fn literal_segments(source: &str) -> Option<Vec<Vec<String>>> {
    let mut segments = Vec::new();
    let mut words = Vec::new();
    let mut word = String::new();
    let mut started = false;
    let mut quote = None;
    let mut chars = source.chars().peekable();
    while let Some(ch) = chars.next() {
        if let Some(delimiter) = quote {
            if ch == delimiter {
                quote = None;
            } else if delimiter == '"' && matches!(ch, '$' | '`' | '\\') {
                return None;
            } else {
                word.push(ch);
            }
            continue;
        }
        match ch {
            '\'' | '"' => {
                quote = Some(ch);
                started = true;
            }
            '$' | '`' | '<' | '>' | '(' | ')' | '{' | '}' | '\\' | '*' | '?' | '[' | ']' => {
                return None;
            }
            '#' if !started => {
                while chars.peek().is_some_and(|c| *c != '\n') {
                    chars.next();
                }
            }
            ';' | '|' | '&' | '\n' => {
                if ch == '&' && chars.next() != Some('&') {
                    return None;
                }
                if ch == '|' && chars.peek() == Some(&'|') {
                    chars.next();
                }
                if started {
                    words.push(std::mem::take(&mut word));
                    started = false;
                }
                if !words.is_empty() {
                    segments.push(std::mem::take(&mut words));
                }
            }
            c if c.is_whitespace() => {
                if started {
                    words.push(std::mem::take(&mut word));
                    started = false;
                }
            }
            _ => {
                word.push(ch);
                started = true;
            }
        }
    }
    if quote.is_some() {
        return None;
    }
    if started {
        words.push(word);
    }
    if !words.is_empty() {
        segments.push(words);
    }
    Some(segments)
}

fn open_script(root: &cap_std::fs::Dir, path: &Path) -> Result<cap_std::fs::File, &'static str> {
    use cap_fs_ext::{DirExt, FollowSymlinks, MetadataExt, OpenOptionsFollowExt};
    let parts: Vec<_> = path
        .components()
        .filter_map(|part| match part {
            std::path::Component::Normal(name) => Some(name),
            _ => None,
        })
        .collect();
    if parts.is_empty() {
        return Err("script_not_regular_file");
    }
    let mut parent = root.try_clone().map_err(|_| "script_unavailable")?;
    for part in &parts[..parts.len() - 1] {
        parent = parent
            .open_dir_nofollow(Path::new(part))
            .map_err(|_| "script_unavailable_or_link_rejected")?;
    }
    let mut options = cap_std::fs::OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    #[cfg(unix)]
    {
        use cap_fs_ext::OpenOptionsSyncExt;
        options.nonblock(true);
    }
    let file = parent
        .open_with(Path::new(parts[parts.len() - 1]), &options)
        .map_err(|_| "script_unavailable_or_link_rejected")?;
    let metadata = file.metadata().map_err(|_| "script_unavailable")?;
    if !metadata.is_file() {
        return Err("script_not_regular_file");
    }
    if metadata.nlink() != 1 {
        return Err("script_hardlink_rejected");
    }
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn quoted_literals_do_not_hide_compound_script_dependencies() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().canonicalize().expect("root");
        for command in [
            "pwd && printf 'I281_READONLY_OK\\n'",
            "printf 'literal ; | && $HOME `text`'",
            "pwd && printf \"literal value\"",
        ] {
            assert!(
                !collect(command, "bash", &root, &root, |_| false).requires_human_review,
                "{command}"
            );
        }
        for command in [
            "command 'bash' 'missing.sh'",
            "env 'bash' 'missing.sh'",
            "pwd && 'bash' 'missing.sh'",
            "ba\"sh\" missing.sh",
            "printf \"$(touch bad)\"",
            "printf $HOME",
            "printf 'unterminated",
            "printf x > out",
            "pwd & ls",
            "printf x\\;bash missing.sh",
        ] {
            assert!(
                collect(command, "bash", &root, &root, |_| false).requires_human_review,
                "{command}"
            );
        }
        assert!(
            collect("Write-Output 'a''b'", "powershell", &root, &root, |_| false)
                .requires_human_review
        );
    }

    #[test]
    fn execution_wrappers_never_hide_script_dependencies() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().canonicalize().expect("root");
        for command in [
            "command bash missing.sh",
            "env bash missing.sh",
            "env FOO=bar ./missing.sh",
            "exec bash missing.sh",
            "xargs bash missing.sh",
            "nohup bash missing.sh",
            "sudo bash missing.sh",
            "Start-Process pwsh -File missing.ps1",
        ] {
            let evidence = collect(command, "bash", &root, &root, |_| false);
            assert!(evidence.requires_human_review, "{command}");
            assert!(
                evidence
                    .uncertainties
                    .iter()
                    .any(|item| item == "execution_wrapper_arguments_unresolved"),
                "{command}"
            );
        }
    }

    fn collect(
        command: &str,
        tool: &str,
        cwd: &Path,
        root: &Path,
        sensitive: impl Fn(&str) -> bool,
    ) -> ScriptEvidence {
        let capability = cap_std::fs::Dir::open_ambient_dir(root, cap_std::ambient_authority())
            .expect("root capability");
        super::collect(
            command,
            tool,
            cwd.strip_prefix(root).expect("relative cwd"),
            &capability,
            sensitive,
        )
    }

    #[cfg(unix)]
    #[test]
    fn fifo_open_is_nonblocking_and_never_read() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().canonicalize().expect("canonical");
        // Test fixture only: rustix does not expose mkfifo on Darwin.
        let status = std::process::Command::new("mkfifo")
            .arg(root.join("pipe.sh"))
            .status()
            .expect("mkfifo fixture utility must be available on Unix test hosts");
        assert!(status.success());
        let evidence = collect("bash pipe.sh", "bash", &root, &root, |_| false);
        assert!(evidence.files.is_empty());
        assert!(
            evidence
                .uncertainties
                .iter()
                .any(|s| s == "script_not_regular_file")
        );
    }

    #[test]
    fn captures_nested_content_without_claiming_execution_binding() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().canonicalize().expect("canonical");
        fs::write(root.join("one.sh"), "source two.sh\npwd\n").expect("write");
        fs::write(root.join("two.sh"), "echo hello\n").expect("write");
        let evidence = collect("bash one.sh arg", "bash", &root, &root, |_| false);
        assert_eq!(evidence.files.len(), 2);
        assert_eq!(evidence.files[1].content, "echo hello\n");
        assert!(!evidence.execution_bytes_bound);
        assert!(evidence.requires_human_review);
    }

    #[test]
    fn absent_or_unresolved_scripts_are_not_mistaken_for_direct_commands() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().canonicalize().expect("canonical");
        for command in [
            "bash absent.sh",
            "pwsh -NoProfile -File absent.ps1",
            "sh -c 'pwd'",
            "pwd; bash absent.sh",
            "pwd && ./absent.sh",
            "source $SCRIPT",
            "/bin/bash",
            "python3 absent.py",
        ] {
            let evidence = collect(command, "bash", &root, &root, |_| false);
            assert!(evidence.files.is_empty());
            assert!(evidence.requires_human_review, "{command}");
            assert!(!evidence.execution_bytes_bound);
        }
        for command in [
            "pwd",
            "ls -la",
            "cat Cargo.toml | head",
            "pwd && ls",
            "pwd; ls",
            "pwd || ls",
        ] {
            let evidence = collect(command, "bash", &root, &root, |_| false);
            assert!(!evidence.requires_human_review, "{command}");
            assert!(!evidence.execution_bytes_bound);
        }
    }

    #[test]
    fn scans_each_literal_compound_segment_for_script_dependencies() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().canonicalize().expect("canonical");
        for command in [
            "pwd && bash missing.sh",
            "pwd | ./missing.sh",
            "pwd; 'bash' missing.sh",
            "pwd || './missing.sh'",
        ] {
            let evidence = collect(command, "bash", &root, &root, |_| false);
            assert!(evidence.requires_human_review, "{command}");
        }
        fs::write(root.join("one.sh"), "pwd").expect("write");
        let evidence = collect("pwd && bash one.sh", "bash", &root, &root, |_| false);
        assert_eq!(evidence.files.len(), 1);
        assert!(evidence.requires_human_review);
    }

    #[test]
    fn rejects_secret_oversize_and_invalid_utf8_without_partial_content() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().canonicalize().expect("canonical");
        for bytes in [b"secret".to_vec(), vec![b'x'; MAX_BYTES + 1], vec![255]] {
            fs::write(root.join("one.ps1"), bytes).expect("write");
            let evidence = collect("pwsh -File one.ps1", "powershell", &root, &root, |s| {
                s.contains("secret")
            });
            assert!(evidence.files.is_empty());
            assert!(evidence.requires_human_review);
            assert!(!evidence.uncertainties.is_empty());
        }
    }

    #[test]
    fn cycles_are_bounded_and_dynamic_content_is_uncertain() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().canonicalize().expect("canonical");
        fs::write(root.join("one.sh"), "source one.sh\nsource $OTHER\n").expect("write");
        let evidence = collect("sh one.sh", "bash", &root, &root, |_| false);
        assert_eq!(evidence.files.len(), 1);
        assert!(evidence.uncertainties.iter().any(|s| s.contains("dynamic")));
    }

    #[cfg(unix)]
    #[test]
    fn excludes_symlink_escape_and_directories() {
        let dir = tempfile::tempdir().expect("tempdir");
        let outside = tempfile::tempdir().expect("outside");
        let root = dir.path().canonicalize().expect("canonical");
        fs::write(outside.path().join("x"), "pwd").expect("write");
        std::os::unix::fs::symlink(outside.path().join("x"), root.join("one.sh")).expect("symlink");
        assert!(
            collect("bash one.sh", "bash", &root, &root, |_| false)
                .files
                .is_empty()
        );
        assert!(
            collect("bash .", "bash", &root, &root, |_| false)
                .files
                .is_empty()
        );
    }

    #[cfg(unix)]
    #[test]
    fn rejects_internal_symlink_and_hardlink_secret_aliases() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().canonicalize().expect("canonical");
        fs::write(root.join(".env"), "bare-sensitive-value").expect("write");
        std::os::unix::fs::symlink(".env", root.join("safe.sh")).expect("symlink");
        fs::hard_link(root.join(".env"), root.join("hard.sh")).expect("hardlink");
        for command in ["bash safe.sh", "bash hard.sh"] {
            let evidence = collect(command, "bash", &root, &root, |_| false);
            assert!(evidence.files.is_empty());
            assert!(evidence.requires_human_review);
        }
        fs::create_dir(root.join("secrets")).expect("directory");
        fs::write(root.join("secrets/one.sh"), "bare-sensitive-value").expect("write");
        std::os::unix::fs::symlink("secrets", root.join("alias")).expect("symlink");
        assert!(
            collect("bash alias/one.sh", "bash", &root, &root, |_| false)
                .files
                .is_empty()
        );
    }

    #[cfg(unix)]
    #[test]
    fn root_capability_is_not_reopened_after_path_replacement() {
        let dir = tempfile::tempdir().expect("tempdir");
        let outside = tempfile::tempdir().expect("outside");
        let root = dir.path().join("root");
        fs::create_dir(&root).expect("root");
        fs::write(root.join("one.sh"), "echo original").expect("write");
        fs::write(outside.path().join("one.sh"), "echo outside").expect("write");
        let capability = cap_std::fs::Dir::open_ambient_dir(&root, cap_std::ambient_authority())
            .expect("capability");
        fs::rename(&root, dir.path().join("old-root")).expect("rename");
        std::os::unix::fs::symlink(outside.path(), &root).expect("symlink");
        let evidence = super::collect("bash one.sh", "bash", Path::new("."), &capability, |_| {
            false
        });
        assert_eq!(evidence.files.len(), 1);
        assert_eq!(evidence.files[0].content, "echo original");
    }
}
