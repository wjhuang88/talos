use crate::{Result, SkillError, SkillFrontmatter};

/// Splits a SKILL.md file into frontmatter YAML and Markdown body.
///
/// The content must start with `---` (optionally preceded by whitespace),
/// contain YAML, then end with another `---` delimiter.
pub(crate) fn split_frontmatter(content: &str) -> Result<(&str, &str)> {
    let trimmed = content.trim_start();

    if !trimmed.starts_with("---") {
        return Err(SkillError::InvalidFrontmatter(
            "file must start with '---' delimiter".into(),
        ));
    }

    let after_open = &trimmed[3..];
    let close_pos = after_open
        .find("\n---")
        .ok_or_else(|| SkillError::InvalidFrontmatter("missing closing '---' delimiter".into()))?;

    let frontmatter = after_open[..close_pos].trim();
    let body = &after_open[close_pos + 4..];

    Ok((frontmatter, body))
}

/// Validates that all required frontmatter fields are present and non-empty.
pub(crate) fn validate_frontmatter(fm: &SkillFrontmatter) -> Result<()> {
    if fm.name.is_empty() {
        return Err(SkillError::InvalidFrontmatter(
            "'name' field is required and must not be empty".into(),
        ));
    }
    if fm.description.is_empty() {
        return Err(SkillError::InvalidFrontmatter(
            "'description' field is required and must not be empty".into(),
        ));
    }
    Ok(())
}
