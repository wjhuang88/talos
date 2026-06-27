#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PromptSectionKind {
    Cacheable,
    Dynamic,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct PromptSection {
    pub(super) text: String,
    pub(super) kind: PromptSectionKind,
}
