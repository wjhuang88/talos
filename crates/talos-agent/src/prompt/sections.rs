#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PromptSectionKind {
    Cacheable,
    Dynamic,
}

/// Classification metadata for a prompt contribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PromptContributionSource {
    Runtime,
    Identity,
    Tools,
    Skills,
    Context,
    Memory,
    Session,
    User,
    Extension,
}

/// Typed provenance attached to a rendered section without changing its text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct PromptContributionMetadata {
    pub(super) source: PromptContributionSource,
    pub(super) authority: u8,
    pub(super) cacheable: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct PromptSection {
    pub(super) text: String,
    pub(super) kind: PromptSectionKind,
}

impl PromptSection {
    pub(super) fn metadata(&self) -> PromptContributionMetadata {
        let source = if self.text.starts_with("# Identity") {
            PromptContributionSource::Identity
        } else if self.text.starts_with("# Tool") {
            PromptContributionSource::Tools
        } else if self.text.starts_with("# Skill") {
            PromptContributionSource::Skills
        } else if self.text.starts_with("# Context") {
            PromptContributionSource::Context
        } else if self.text.starts_with("# Memory") {
            PromptContributionSource::Memory
        } else if self.text.starts_with("# Session Todos") {
            PromptContributionSource::Session
        } else if self.text.starts_with("# User Preferences") {
            PromptContributionSource::User
        } else if self.text.starts_with("# Runtime Context") {
            PromptContributionSource::Runtime
        } else {
            PromptContributionSource::Extension
        };
        PromptContributionMetadata {
            source,
            authority: match source {
                PromptContributionSource::Runtime => 100,
                PromptContributionSource::Identity
                | PromptContributionSource::Tools
                | PromptContributionSource::Skills => 90,
                PromptContributionSource::User => 80,
                PromptContributionSource::Context => 70,
                _ => 40,
            },
            cacheable: self.kind == PromptSectionKind::Cacheable,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_preserves_cache_class_and_source() {
        let section = PromptSection {
            text: "# Identity\nrole\n".into(),
            kind: PromptSectionKind::Cacheable,
        };
        let metadata = section.metadata();
        assert_eq!(metadata.source, PromptContributionSource::Identity);
        assert_eq!(metadata.authority, 90);
        assert!(metadata.cacheable);
    }
}
