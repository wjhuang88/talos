//! Command access evidence types (ADR-040).
//!
//! Defines typed evidence for what a command reads, writes, deletes, spawns,
//! or accesses over the network. Evidence is observation, not authority — it
//! never grants permission by itself. The permission engine uses evidence to
//! narrow the approval path, but Deny rules and out-of-repo checks remain
//! authoritative.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// The kind of filesystem/process/network access a command performs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessKind {
    Read,
    Write,
    Delete,
    Spawn,
    Network,
    Unknown,
}

/// How the access was determined.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceState {
    /// Structural analysis of the command proved bounded access.
    Declared,
    /// Runtime observation confirmed the access (not yet implemented).
    Observed,
    /// Access could not be determined.
    Unknown,
}

/// Typed evidence of what a command accesses.
///
/// This type carries no authority. It describes access for the permission
/// engine to use as input; it never overrides Deny rules or out-of-repo
/// enforcement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessEvidence {
    pub kind: AccessKind,
    pub state: EvidenceState,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub paths: Vec<PathBuf>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub detail: String,
}

impl AccessEvidence {
    pub fn declared_read(paths: Vec<PathBuf>) -> Self {
        Self {
            kind: AccessKind::Read,
            state: EvidenceState::Declared,
            paths,
            detail: String::new(),
        }
    }

    pub fn unknown() -> Self {
        Self {
            kind: AccessKind::Unknown,
            state: EvidenceState::Unknown,
            paths: Vec::new(),
            detail: String::new(),
        }
    }

    pub fn network() -> Self {
        Self {
            kind: AccessKind::Network,
            state: EvidenceState::Declared,
            paths: Vec::new(),
            detail: String::new(),
        }
    }

    pub fn spawn() -> Self {
        Self {
            kind: AccessKind::Spawn,
            state: EvidenceState::Declared,
            paths: Vec::new(),
            detail: String::new(),
        }
    }

    pub fn is_unknown(&self) -> bool {
        self.kind == AccessKind::Unknown || self.state == EvidenceState::Unknown
    }

    pub fn is_repo_local(&self, repo_root: &std::path::Path) -> bool {
        if self.paths.is_empty() {
            return false;
        }
        let repo_canonical =
            std::fs::canonicalize(repo_root).unwrap_or_else(|_| repo_root.to_path_buf());
        self.paths.iter().all(|p| {
            let canonical = match std::fs::canonicalize(p) {
                Ok(c) => c,
                Err(_) => {
                    let base = if p.is_absolute() {
                        p.clone()
                    } else {
                        repo_canonical.join(p)
                    };
                    normalize_path(&base)
                }
            };
            canonical.starts_with(&repo_canonical)
        })
    }
}

fn normalize_path(path: &std::path::Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        use std::path::Component;
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if normalized.components().next().is_none() {
                    normalized.push("..");
                } else {
                    let last = normalized.components().next_back();
                    match last {
                        Some(Component::Normal(_)) => {
                            normalized.pop();
                        }
                        Some(Component::RootDir) => {}
                        _ => {
                            normalized.push("..");
                        }
                    }
                }
            }
            other => {
                normalized.push(other.as_os_str());
            }
        }
    }
    normalized
}

/// Classify a simple bash command string into access evidence.
///
/// This is a structural classifier, not a shell parser. Commands with shell
/// metacharacters, variable expansion, or unclassifiable structure produce
/// `Unknown` evidence. This matches ADR-012's boundary: complex shell
/// features fall back to strict behavior.
pub fn classify_command_access(command: &str) -> AccessEvidence {
    let trimmed = command.trim();
    if trimmed.is_empty() || has_shell_control_syntax(trimmed) {
        return AccessEvidence::unknown();
    }

    let mut parts = trimmed.split_whitespace();
    let Some(program) = parts.next() else {
        return AccessEvidence::unknown();
    };
    let args: Vec<&str> = parts.collect();

    if is_env_assignment(program) {
        return AccessEvidence::unknown();
    }

    if args.iter().any(|a| !is_simple_token(a)) {
        return AccessEvidence::unknown();
    }

    match program {
        "ls" | "pwd" | "cat" | "head" | "tail" | "wc" | "grep" | "rg" | "find" | "stat"
        | "file" | "diff" | "sed" | "awk" | "tree" | "which" | "type" => {
            let paths: Vec<PathBuf> = args
                .iter()
                .filter(|a| !a.starts_with('-'))
                .map(PathBuf::from)
                .collect();
            AccessEvidence {
                kind: AccessKind::Read,
                state: EvidenceState::Declared,
                paths,
                detail: program.to_string(),
            }
        }
        "cargo" => classify_cargo_access(&args),
        "rm" | "rmdir" => AccessEvidence {
            kind: AccessKind::Delete,
            state: EvidenceState::Declared,
            paths: args
                .iter()
                .filter(|a| !a.starts_with('-'))
                .map(PathBuf::from)
                .collect(),
            detail: program.to_string(),
        },
        "mv" | "cp" | "mkdir" | "touch" | "tee" | "chmod" | "chown" | "ln" => AccessEvidence {
            kind: AccessKind::Write,
            state: EvidenceState::Declared,
            paths: args
                .iter()
                .filter(|a| !a.starts_with('-'))
                .map(PathBuf::from)
                .collect(),
            detail: program.to_string(),
        },
        "curl" | "wget" | "ssh" | "scp" | "rsync" => AccessEvidence::network(),
        "git" => classify_git_access(&args),
        _ => AccessEvidence::unknown(),
    }
}

fn classify_cargo_access(args: &[&str]) -> AccessEvidence {
    match args.first().copied() {
        Some("test" | "check" | "build" | "clippy" | "metadata" | "tree") => AccessEvidence {
            kind: AccessKind::Read,
            state: EvidenceState::Declared,
            paths: Vec::new(),
            detail: "cargo".to_string(),
        },
        Some("fmt" | "fix" | "run" | "install" | "publish") => AccessEvidence {
            kind: AccessKind::Write,
            state: EvidenceState::Declared,
            paths: Vec::new(),
            detail: "cargo".to_string(),
        },
        _ => AccessEvidence::unknown(),
    }
}

fn classify_git_access(args: &[&str]) -> AccessEvidence {
    match args.first().copied() {
        Some("status" | "diff" | "log" | "show" | "branch" | "stash" | "blame") => AccessEvidence {
            kind: AccessKind::Read,
            state: EvidenceState::Declared,
            paths: Vec::new(),
            detail: "git".to_string(),
        },
        Some("add" | "commit" | "checkout" | "merge" | "rebase" | "reset" | "clean") => {
            AccessEvidence {
                kind: AccessKind::Write,
                state: EvidenceState::Declared,
                paths: Vec::new(),
                detail: "git".to_string(),
            }
        }
        Some("push" | "pull" | "fetch" | "clone") => AccessEvidence::network(),
        _ => AccessEvidence::unknown(),
    }
}

fn has_shell_control_syntax(command: &str) -> bool {
    command.contains('|')
        || command.contains(';')
        || command.contains('\n')
        || command.contains("&&")
        || command.contains("||")
        || command.contains("$(")
        || command.contains('`')
        || command.contains('>')
        || command.contains('<')
        || command.ends_with('&')
}

fn is_env_assignment(token: &str) -> bool {
    let Some((name, _)) = token.split_once('=') else {
        return false;
    };
    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
        && !name.as_bytes()[0].is_ascii_digit()
}

fn is_simple_token(token: &str) -> bool {
    !token.contains('"')
        && !token.contains('\'')
        && !token.contains('\\')
        && !token.contains('*')
        && !token.contains('?')
        && !token.contains('{')
        && !token.contains('}')
        && !token.contains('~')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_only_command_classified_as_read() {
        let ev = classify_command_access("cat Cargo.toml");
        assert_eq!(ev.kind, AccessKind::Read);
        assert_eq!(ev.state, EvidenceState::Declared);
        assert!(ev.paths.iter().any(|p| p.to_string_lossy() == "Cargo.toml"));
    }

    #[test]
    fn test_complex_shell_is_unknown() {
        let ev = classify_command_access("cat Cargo.toml | grep foo");
        assert_eq!(ev.kind, AccessKind::Unknown);
        assert!(ev.is_unknown());
    }

    #[test]
    fn test_pipe_command_is_unknown() {
        let ev = classify_command_access("echo hello | cat");
        assert!(ev.is_unknown());
    }

    #[test]
    fn test_redirect_is_unknown() {
        let ev = classify_command_access("echo hello > file.txt");
        assert!(ev.is_unknown());
    }

    #[test]
    fn test_command_substitution_is_unknown() {
        let ev = classify_command_access("cat $(find . -name foo)");
        assert!(ev.is_unknown());
    }

    #[test]
    fn test_env_assignment_is_unknown() {
        let ev = classify_command_access("FOO=bar cargo test");
        assert!(ev.is_unknown());
    }

    #[test]
    fn test_delete_command() {
        let ev = classify_command_access("rm -rf target/");
        assert_eq!(ev.kind, AccessKind::Delete);
        assert!(ev.paths.iter().any(|p| p.to_string_lossy() == "target/"));
    }

    #[test]
    fn test_write_command() {
        let ev = classify_command_access("mkdir new_dir");
        assert_eq!(ev.kind, AccessKind::Write);
    }

    #[test]
    fn test_network_command() {
        let ev = classify_command_access("curl https://example.com");
        assert_eq!(ev.kind, AccessKind::Network);
    }

    #[test]
    fn test_git_push_is_network() {
        let ev = classify_command_access("git push origin main");
        assert_eq!(ev.kind, AccessKind::Network);
    }

    #[test]
    fn test_git_status_is_read() {
        let ev = classify_command_access("git status");
        assert_eq!(ev.kind, AccessKind::Read);
    }

    #[test]
    fn test_cargo_test_is_read() {
        let ev = classify_command_access("cargo test");
        assert_eq!(ev.kind, AccessKind::Read);
    }

    #[test]
    fn test_cargo_publish_is_write() {
        let ev = classify_command_access("cargo publish");
        assert_eq!(ev.kind, AccessKind::Write);
    }

    #[test]
    fn test_unknown_program_is_unknown() {
        let ev = classify_command_access("some-custom-binary --flag");
        assert!(ev.is_unknown());
    }

    #[test]
    fn test_empty_command_is_unknown() {
        let ev = classify_command_access("");
        assert!(ev.is_unknown());
    }

    #[test]
    fn test_glob_is_unknown() {
        let ev = classify_command_access("cat *.rs");
        assert!(ev.is_unknown());
    }

    #[test]
    fn test_evidence_serialization() {
        let ev = classify_command_access("cat Cargo.toml");
        let json = serde_json::to_string(&ev).expect("serialize");
        assert!(json.contains("read"));
        assert!(json.contains("declared"));
        let back: AccessEvidence = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.kind, AccessKind::Read);
    }

    #[test]
    fn test_is_repo_local() {
        let ev = AccessEvidence {
            kind: AccessKind::Read,
            state: EvidenceState::Declared,
            paths: vec![PathBuf::from("Cargo.toml")],
            detail: String::new(),
        };
        let root = std::env::current_dir().expect("cwd");
        assert!(ev.is_repo_local(&root));
    }

    #[test]
    fn test_out_of_repo_path_not_repo_local() {
        let ev = AccessEvidence {
            kind: AccessKind::Read,
            state: EvidenceState::Declared,
            paths: vec![PathBuf::from("/etc/passwd")],
            detail: String::new(),
        };
        let root = std::env::current_dir().expect("cwd");
        assert!(!ev.is_repo_local(&root));
    }

    #[test]
    fn test_empty_paths_not_repo_local() {
        let ev = AccessEvidence::network();
        let root = std::env::current_dir().expect("cwd");
        assert!(!ev.is_repo_local(&root));
    }

    #[test]
    fn test_traversal_nonexistent_path_not_repo_local() {
        let ev = AccessEvidence {
            kind: AccessKind::Read,
            state: EvidenceState::Declared,
            paths: vec![PathBuf::from("../../etc/passwd")],
            detail: String::new(),
        };
        let root = std::env::current_dir().expect("cwd");
        assert!(
            !ev.is_repo_local(&root),
            "traversal path ../../etc/passwd must NOT be repo-local"
        );
    }

    #[test]
    fn test_traversal_absolute_nonexistent_not_repo_local() {
        let ev = AccessEvidence {
            kind: AccessKind::Read,
            state: EvidenceState::Declared,
            paths: vec![PathBuf::from("/etc/nonexistent/../../../passwd")],
            detail: String::new(),
        };
        let root = std::env::current_dir().expect("cwd");
        assert!(!ev.is_repo_local(&root));
    }

    #[test]
    fn test_normalize_path_resolves_parent_dir() {
        let p = normalize_path(std::path::Path::new("/a/b/../c"));
        assert_eq!(p, PathBuf::from("/a/c"));
    }

    #[test]
    fn test_normalize_path_resolves_multiple_parent_dir() {
        let p = normalize_path(std::path::Path::new("/a/b/../../c"));
        assert_eq!(p, PathBuf::from("/c"));
    }

    #[test]
    fn test_normalize_path_parent_at_root_stays_at_root() {
        let p = normalize_path(std::path::Path::new("/.."));
        assert_eq!(p, PathBuf::from("/"));
    }
}
