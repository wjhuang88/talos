//! Confined, bounded Level 1/2 skill disclosure shared by hosts (ADR-092).

use crate::{Result, Skill, SkillError};
use std::io::{Error as IoError, ErrorKind, Read};
use std::path::{Component, Path, PathBuf};

/// Default byte budget for an activated Skill body.
pub const DEFAULT_ACTIVATION_BODY_BYTES: usize = 24 * 1024;
/// Default byte budget for one activated Skill reference.
pub const DEFAULT_ACTIVATION_REFERENCE_BYTES: usize = 16 * 1024;

/// Limits applied to confined skill file reads, listings and chunks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillReadLimits {
    /// Largest file, in bytes, that may be read.
    pub max_file_bytes: u64,
    /// Largest text chunk, in bytes, returned by [`chunk_text`].
    pub max_chunk_bytes: usize,
    /// Maximum number of entries returned by [`list_skill_files`].
    pub max_listed_files: usize,
    /// Maximum directory depth visited by [`list_skill_files`].
    pub max_list_depth: usize,
}

impl Default for SkillReadLimits {
    fn default() -> Self {
        Self {
            max_file_bytes: 2 * 1024 * 1024,
            max_chunk_bytes: 48 * 1024,
            max_listed_files: 256,
            max_list_depth: 8,
        }
    }
}

/// Bounded listing of files inside a skill directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillFileListing {
    /// Skill-relative file paths using `/` separators, sorted.
    pub files: Vec<String>,
    /// Whether entries were omitted because a limit was reached.
    pub truncated: bool,
}

/// One UTF-8 chunk of a larger text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillTextChunk {
    /// Chunk text.
    pub text: String,
    /// Byte offset where this chunk starts.
    pub offset: usize,
    /// Byte offset to request next, or `None` when the text is complete.
    pub next_offset: Option<usize>,
    /// Total byte length of the source text.
    pub total_bytes: usize,
}

fn rejected(message: impl Into<String>) -> SkillError {
    SkillError::IoError(IoError::new(ErrorKind::InvalidInput, message.into()))
}

fn skill_dir(skill: &Skill) -> Result<&Path> {
    skill
        .source_path
        .parent()
        .ok_or_else(|| SkillError::InvalidFrontmatter("skill has no parent directory".into()))
}

/// Resolves a skill-relative path to a canonical regular file inside the skill directory.
///
/// Rejects empty, absolute, root/prefix and `..` paths, and any target that resolves
/// (including through symbolic links) outside the canonical skill directory.
pub fn resolve_skill_file(skill: &Skill, relative_path: &str) -> Result<PathBuf> {
    let rel = Path::new(relative_path.trim());
    if rel.as_os_str().is_empty() {
        return Err(rejected("reference path is required"));
    }
    if rel.is_absolute()
        || rel.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::Prefix(_) | Component::RootDir
            )
        })
    {
        return Err(rejected(
            "reference path must stay inside the skill directory",
        ));
    }
    let canonical_dir = skill_dir(skill)?.canonicalize()?;
    let canonical = canonical_dir.join(rel).canonicalize()?;
    if !canonical.starts_with(&canonical_dir) {
        return Err(SkillError::IoError(IoError::new(
            ErrorKind::PermissionDenied,
            "reference path escapes the skill directory",
        )));
    }
    if !std::fs::metadata(&canonical)?.is_file() {
        return Err(rejected("reference path is not a regular file"));
    }
    Ok(canonical)
}

/// Reads a UTF-8 file confined to the skill directory and bounded by `limits`.
pub fn read_skill_file(
    skill: &Skill,
    relative_path: &str,
    limits: &SkillReadLimits,
) -> Result<String> {
    let path = resolve_skill_file(skill, relative_path)?;
    let file = std::fs::File::open(&path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err(rejected("reference path is not a regular file"));
    }
    if metadata.len() > limits.max_file_bytes {
        return Err(rejected(format!(
            "reference file is {} bytes, exceeding the {} byte limit",
            metadata.len(),
            limits.max_file_bytes
        )));
    }
    let mut bytes = Vec::new();
    file.take(limits.max_file_bytes.saturating_add(1))
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limits.max_file_bytes {
        return Err(rejected("reference file grew beyond the byte limit"));
    }
    String::from_utf8(bytes).map_err(|_| rejected("reference file is not valid UTF-8 text"))
}

/// Lists regular files inside the skill directory without following symbolic links.
///
/// At most `max_listed_files` entries are returned and at most sixteen times that many
/// directory entries are visited; `truncated` reports either limit.
pub fn list_skill_files(skill: &Skill, limits: &SkillReadLimits) -> Result<SkillFileListing> {
    let root = skill_dir(skill)?;
    let mut files = Vec::new();
    let mut truncated = false;
    let max_visited = limits.max_listed_files.saturating_mul(16).max(1);
    let walker = walkdir::WalkDir::new(root)
        .follow_links(false)
        .max_depth(limits.max_list_depth)
        .sort_by_file_name();
    for (visited, entry) in walker.into_iter().enumerate() {
        if visited >= max_visited {
            truncated = true;
            break;
        }
        let Ok(entry) = entry else {
            continue;
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let Ok(rel) = entry.path().strip_prefix(root) else {
            continue;
        };
        let parts: Option<Vec<&str>> = rel.components().map(|c| c.as_os_str().to_str()).collect();
        let Some(parts) = parts else {
            continue;
        };
        if files.len() >= limits.max_listed_files {
            truncated = true;
            break;
        }
        files.push(parts.join("/"));
    }
    files.sort();
    Ok(SkillFileListing { files, truncated })
}

/// Returns at most `max_bytes` of `text` starting at `offset`, on UTF-8 boundaries.
///
/// An `offset` inside a multi-byte character is moved back to that character's start.
pub fn chunk_text(text: &str, offset: usize, max_bytes: usize) -> SkillTextChunk {
    let total_bytes = text.len();
    let mut start = offset.min(total_bytes);
    while !text.is_char_boundary(start) {
        start -= 1;
    }
    let mut end = start.saturating_add(max_bytes.max(1)).min(total_bytes);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    if end == start && start < total_bytes {
        end = start + text[start..].chars().next().map_or(1, char::len_utf8);
    }
    SkillTextChunk {
        text: text[start..end].to_string(),
        offset: start,
        next_offset: (end < total_bytes).then_some(end),
        total_bytes,
    }
}

/// Truncates text to `max_bytes` on a UTF-8 boundary, appending a truncation marker.
pub fn bounded_text(text: &str, max_bytes: usize) -> String {
    if text.len() <= max_bytes {
        return text.to_string();
    }
    let mut end = max_bytes;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!(
        "{}\n\n[truncated: original_bytes={}, kept_bytes={}]",
        &text[..end],
        text.len(),
        end
    )
}

/// Builds the bounded activated-skill context text used by hosts.
///
/// The format matches the CLI `/skills activate` and `/skills reference` output:
/// a `## Skill Body` block followed by `## Reference: <path>` blocks.
#[derive(Debug, Clone)]
pub struct SkillActivation {
    skill: Skill,
    content: String,
    references: Vec<String>,
}

impl SkillActivation {
    /// Starts an activation with the skill body bounded to `max_body_bytes`.
    pub fn new(skill: &Skill, max_body_bytes: usize) -> Self {
        let body = bounded_text(&skill.body, max_body_bytes);
        Self {
            skill: skill.clone(),
            content: format!("## Skill Body\n{body}\n"),
            references: Vec::new(),
        }
    }

    /// Appends a confined reference file bounded to `max_reference_bytes`.
    pub fn add_reference(
        &mut self,
        relative_path: &str,
        max_reference_bytes: usize,
        limits: &SkillReadLimits,
    ) -> Result<&str> {
        let reference = read_skill_file(&self.skill, relative_path, limits)?;
        let reference = bounded_text(&reference, max_reference_bytes);
        let display = relative_path.trim();
        self.content
            .push_str(&format!("\n## Reference: {display}\n{reference}\n"));
        self.references.push(display.to_string());
        Ok(&self.content)
    }

    /// Returns the skill name.
    pub fn name(&self) -> &str {
        &self.skill.name
    }

    /// Returns the rendered context text.
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Returns references loaded so far, in order.
    pub fn references(&self) -> &[String] {
        &self.references
    }
}
