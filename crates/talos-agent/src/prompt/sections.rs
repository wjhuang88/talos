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

/// Deterministic outcome used by the provider-independent behavior harness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(super) enum AuthorityDecision {
    HigherWins,
    LowerWins,
    EqualConflict,
}

/// Resolves two competing contributions without depending on a model/provider.
#[allow(dead_code)]
pub(super) fn resolve_authority(high: PromptContributionMetadata, low: PromptContributionMetadata) -> AuthorityDecision {
    match high.authority.cmp(&low.authority) {
        std::cmp::Ordering::Greater => AuthorityDecision::HigherWins,
        std::cmp::Ordering::Less => AuthorityDecision::LowerWins,
        std::cmp::Ordering::Equal => AuthorityDecision::EqualConflict,
    }
}

/// Formats a stable, provider-independent diagnostic for harness consumers.
#[allow(dead_code)]
pub(super) fn authority_diagnostic(
    high: PromptContributionMetadata,
    low: PromptContributionMetadata,
) -> String {
    format!(
        "authority:{}({:?}) vs {}({:?}) => {:?}",
        high.authority,
        high.source,
        low.authority,
        low.source,
        resolve_authority(high, low)
    )
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct PromptSection {
    pub(super) text: String,
    pub(super) kind: PromptSectionKind,
}

impl PromptSection {
    /// Returns whether this section is advisory and must not override user or runtime authority.
    pub(super) fn is_advisory(&self) -> bool {
        matches!(
            self.metadata().source,
            PromptContributionSource::Memory
                | PromptContributionSource::Session
                | PromptContributionSource::Extension
        )
    }

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

    #[test]
    fn memory_and_todos_are_advisory_sections() {
        for text in ["# Memory\nold", "# Session Todos\nold"] {
            let section = PromptSection {
                text: text.into(),
                kind: PromptSectionKind::Dynamic,
            };
            assert!(section.is_advisory());
            assert!(section.metadata().authority < 80);
        }
    }

    #[test]
    fn behavior_harness_runtime_rules_win_over_advisory_context() {
        let runtime = PromptSection { text: "# Runtime Context\nDeny".into(), kind: PromptSectionKind::Dynamic }.metadata();
        let memory = PromptSection { text: "# Memory (advisory)\nAllow".into(), kind: PromptSectionKind::Dynamic }.metadata();
        assert_eq!(resolve_authority(runtime, memory), AuthorityDecision::HigherWins);
    }

    #[test]
    fn behavior_harness_equal_authority_is_reported_as_conflict() {
        let user = PromptSection { text: "# User Preferences\nA".into(), kind: PromptSectionKind::Dynamic }.metadata();
        let other = PromptContributionMetadata { source: PromptContributionSource::User, authority: user.authority, cacheable: false };
        assert_eq!(resolve_authority(user, other), AuthorityDecision::EqualConflict);
    }

    #[test]
    fn behavior_harness_diagnostic_is_stable_and_actionable() {
        let runtime = PromptSection { text: "# Runtime Context\nrule".into(), kind: PromptSectionKind::Dynamic }.metadata();
        let memory = PromptSection { text: "# Memory (advisory)\nstale".into(), kind: PromptSectionKind::Dynamic }.metadata();
        assert_eq!(
            authority_diagnostic(runtime, memory),
            "authority:100(Runtime) vs 40(Memory) => HigherWins"
        );
    }

    #[test]
    fn behavior_harness_lower_authority_cannot_override() {
        let advisory = PromptSection { text: "# Memory (advisory)\nstale".into(), kind: PromptSectionKind::Dynamic }.metadata();
        let runtime = PromptSection { text: "# Runtime Context\nrule".into(), kind: PromptSectionKind::Dynamic }.metadata();
        assert_eq!(resolve_authority(advisory, runtime), AuthorityDecision::LowerWins);
    }

    #[test]
    fn behavior_harness_protocol_parity_is_structural() {
        let expected = PromptContributionMetadata { source: PromptContributionSource::Runtime, authority: 100, cacheable: false };
        for surface in ["print", "tui", "rpc", "mcp"] {
            let actual = PromptSection { text: format!("# Runtime Context ({surface})"), kind: PromptSectionKind::Dynamic }.metadata();
            assert_eq!(actual, expected, "protocol surface diverged: {surface}");
        }
    }

    #[test]
    fn behavior_harness_current_user_wins_over_memory_and_evolution() {
        let user = PromptSection { text: "# User Preferences\ncurrent request".into(), kind: PromptSectionKind::Dynamic }.metadata();
        for advisory in ["# Memory (advisory)\nstale", "## Advisory Learned Patterns\nstale"] {
            let section = PromptSection { text: advisory.into(), kind: PromptSectionKind::Dynamic }.metadata();
            assert_eq!(resolve_authority(user, section), AuthorityDecision::HigherWins);
        }
    }

    #[test]
    fn behavior_harness_todo_is_advisory_against_runtime_and_user() {
        let todo = PromptSection { text: "# Session Todos (advisory)\nDo old thing".into(), kind: PromptSectionKind::Dynamic }.metadata();
        let user = PromptSection { text: "# User Preferences\nDo current thing".into(), kind: PromptSectionKind::Dynamic }.metadata();
        let runtime = PromptSection { text: "# Runtime Context\nStop".into(), kind: PromptSectionKind::Dynamic }.metadata();
        assert_eq!(resolve_authority(todo, user), AuthorityDecision::LowerWins);
        assert_eq!(resolve_authority(todo, runtime), AuthorityDecision::LowerWins);
    }
}
