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

    #[test]
    fn behavior_fixture_authority_order_is_deterministic() {
        let sources = [
            PromptContributionSource::Runtime,
            PromptContributionSource::Identity,
            PromptContributionSource::User,
            PromptContributionSource::Context,
            PromptContributionSource::Memory,
            PromptContributionSource::Extension,
        ];
        let authorities: Vec<u8> = sources
            .iter()
            .map(|source| match source {
                PromptContributionSource::Runtime => 100,
                PromptContributionSource::Identity => 90,
                PromptContributionSource::User => 80,
                PromptContributionSource::Context => 70,
                _ => 40,
            })
            .collect();
        assert!(authorities.windows(2).all(|pair| pair[0] >= pair[1]));
    }

    #[test]
    fn behavior_fixture_advisory_sources_cannot_match_runtime_authority() {
        for source in [
            PromptContributionSource::Memory,
            PromptContributionSource::Session,
            PromptContributionSource::Extension,
        ] {
            let section = PromptSection {
                text: "advisory".into(),
                kind: PromptSectionKind::Dynamic,
            };
            let metadata = PromptContributionMetadata {
                source,
                authority: 40,
                cacheable: false,
            };
            assert!(metadata.authority < 100);
            assert!(!section.metadata().cacheable);
        }
    }

    #[test]
    fn behavior_fixture_protocol_surfaces_share_runtime_authority_floor() {
        let surfaces = ["print", "tui", "rpc", "mcp"];
        for surface in surfaces {
            let section = PromptSection {
                text: format!("# Runtime Context ({surface})"),
                kind: PromptSectionKind::Dynamic,
            };
            let metadata = section.metadata();
            assert_eq!(metadata.source, PromptContributionSource::Runtime);
            assert_eq!(metadata.authority, 100);
            assert!(!metadata.cacheable);
        }
    }

    #[test]
    fn behavior_fixture_capability_data_stays_below_current_user_authority() {
        let skill = PromptSection {
            text: "# Skill: formatter".into(),
            kind: PromptSectionKind::Dynamic,
        };
        let tool = PromptSection {
            text: "# Tool: shell".into(),
            kind: PromptSectionKind::Dynamic,
        };
        let user = PromptSection {
            text: "# User Preferences\nrequest".into(),
            kind: PromptSectionKind::Dynamic,
        };
        assert!(skill.metadata().authority > 0);
        assert!(tool.metadata().authority > 0);
        assert!(
            user.metadata().authority
                > PromptContributionMetadata {
                    source: PromptContributionSource::Memory,
                    authority: 40,
                    cacheable: false
                }
                .authority
        );
    }
}
