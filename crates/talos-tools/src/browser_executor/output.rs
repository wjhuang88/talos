//! Closed output vocabulary and structural projection checks.
//! Host executors remain responsible for frame isolation and sensitive-field redaction before
//! producing these values. Structural validation alone cannot establish content confidentiality.

use std::{
    collections::HashMap,
    hash::Hash,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{
    BrowserArtifactRef, BrowserElementRef, BrowserFrameRef, BrowserOperation, BrowserSnapshotRef,
    BrowserTabRef,
};

macro_rules! vocabulary {
    ($name:ident, $description:literal, $($variant:ident),+ $(,)?) => {
        #[doc = $description]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
        pub enum $name {
            $(#[doc = stringify!($variant)] $variant),+
        }
    };
}

vocabulary!(
    BrowserFailureCode,
    "Closed browser failure codes without driver diagnostics.",
    InvalidReference,
    StaleFrameReference,
    DetachedFrame,
    StaleElementReference,
    ContextUnavailable,
    OriginNotAuthorized,
    UnsupportedVersion,
    UnsupportedOperation,
    ResourceLimit,
    AdmissionExpired,
    InvalidRequest,
    PermissionDenied,
    IndeterminateExecution
);
vocabulary!(
    BrowserReadiness,
    "Bounded host-reported frame readiness.",
    Pending,
    Ready
);

/// Whether execution is known not to have run, ambiguous, or known to have partial effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum BrowserOutcome {
    /// Requires proof that no action ran.
    NotExecuted,
    /// Completion is ambiguous; must never be replayed automatically.
    Unknown,
    /// Some effects are known to have occurred.
    Partial,
}

/// The only success status in an acknowledgement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum BrowserSuccess {
    /// Successful completion.
    Ok,
}

/// PNG is the only permitted screenshot encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum BrowserScreenshotMime {
    /// Portable Network Graphics.
    #[serde(rename = "image/png")]
    Png,
}

/// Closed role vocabulary; hosts map unsupported roles to Generic before projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum BrowserRole {
    /// Unclassified node.
    Generic,
    /// Button.
    Button,
    /// Checkbox.
    Checkbox,
    /// Radio button.
    Radio,
    /// Text input.
    Textbox,
    /// Combo box.
    Combobox,
    /// Option.
    Option,
    /// Link.
    Link,
    /// Heading.
    Heading,
    /// List.
    List,
    /// List item.
    Listitem,
    /// Table.
    Table,
    /// Row.
    Row,
    /// Cell.
    Cell,
    /// Image.
    Image,
    /// Static text.
    Statictext,
}

/// Closed state vocabulary, excluding arbitrary attributes and form values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum BrowserNodeState {
    /// Checked.
    Checked,
    /// Unchecked.
    Unchecked,
    /// Mixed.
    Mixed,
    /// Selected.
    Selected,
    /// Expanded.
    Expanded,
    /// Collapsed.
    Collapsed,
    /// Disabled.
    Disabled,
    /// Read only.
    Readonly,
    /// Required.
    Required,
}

/// A tab inventory entry. Origin is a canonical HTTP origin or the literal opaque.
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BrowserTabEntry {
    /// Opaque tab generation.
    pub tab_ref: BrowserTabRef,
    /// Sanitized origin metadata only.
    pub origin: String,
}

/// A frame inventory entry, without frame names, URLs or driver identifiers.
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BrowserFrameEntry {
    /// Opaque frame generation.
    pub frame_ref: BrowserFrameRef,
    /// Parent in the same output, or null for the root.
    #[serde(deserialize_with = "Deserialize::deserialize")]
    #[schemars(required)]
    pub parent_frame_ref: Option<BrowserFrameRef>,
    /// Canonical origin or opaque; never an opaque document identity.
    pub origin: String,
    /// Bounded readiness.
    pub readiness: BrowserReadiness,
}

/// One already-redacted frame-local accessibility node.
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BrowserSnapshotNode {
    /// Element bound to this snapshot.
    pub element_ref: BrowserElementRef,
    /// Parent in the same output, or null for a root.
    #[serde(deserialize_with = "Deserialize::deserialize")]
    #[schemars(required)]
    pub parent_element_ref: Option<BrowserElementRef>,
    /// Closed role.
    pub role: BrowserRole,
    /// Redacted accessible name, at most 256 UTF-8 bytes.
    pub name: String,
    /// Unique closed states, at most nine.
    pub states: Vec<BrowserNodeState>,
}

/// Unvalidated host output. No model/display/persistence API accepts this directly.
/// This type deliberately has no content-bearing Debug implementation.
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all_fields = "camelCase", deny_unknown_fields)]
pub enum BrowserOutput {
    /// Action/lifecycle/wait acknowledgement.
    Ack {
        /// Exact operation.
        operation: BrowserOperation,
        /// Literal ok.
        status: BrowserSuccess,
    },
    /// Newly created tab.
    TabCreated {
        /// Opaque tab identity.
        tab_ref: BrowserTabRef,
    },
    /// Bounded tab inventory.
    Tabs {
        /// At most 64 entries.
        entries: Vec<BrowserTabEntry>,
        /// Whether the host omitted entries.
        truncated: bool,
    },
    /// Bounded frame tree.
    Frames {
        /// At most 128 nodes, depth 16.
        entries: Vec<BrowserFrameEntry>,
        /// Whether the host omitted subtrees.
        truncated: bool,
    },
    /// Sanitized selected-frame URL components.
    PageUrl {
        /// Canonical HTTP origin.
        origin: String,
        /// Path only, without query or fragment.
        path: String,
    },
    /// Redacted frame-local plain text, excluding descendant documents.
    Read {
        /// At most 32 KiB UTF-8.
        text: String,
        /// Whether the host truncated safely.
        truncated: bool,
    },
    /// Frame-local snapshot without descendant-document nodes.
    Snapshot {
        /// Current snapshot identity.
        snapshot_ref: BrowserSnapshotRef,
        /// At most 512 nodes, depth 32.
        nodes: Vec<BrowserSnapshotNode>,
        /// Whether the host omitted subtrees.
        truncated: bool,
    },
    /// Metadata only; bytes are carried by a separate invocation-bound transient channel.
    Screenshot {
        /// Opaque artifact capability, never a path.
        artifact_ref: BrowserArtifactRef,
        /// PNG only.
        mime: BrowserScreenshotMime,
        /// Width, at most 2048.
        width: u32,
        /// Height, at most 2048.
        height: u32,
        /// Encoded size, at most 2 MiB.
        byte_length: u64,
    },
    /// Bounded failure with no free-form error message.
    Failure {
        /// Closed error code.
        code: BrowserFailureCode,
        /// Null only if no discriminator was admitted.
        #[serde(deserialize_with = "Deserialize::deserialize")]
        #[schemars(required)]
        operation: Option<BrowserOperation>,
        /// Honest effect certainty.
        outcome: BrowserOutcome,
    },
}

/// A structural output contract violation. No untrusted text is included.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("invalid browser output")]
pub struct BrowserOutputError;

/// Bounded, invocation-owned screenshot bytes. The store is transient and never a filesystem
/// handle; callers must explicitly consume a capability before its TTL expires.
#[derive(Clone, Default)]
pub(crate) struct BrowserArtifactStore {
    inner: Arc<Mutex<ArtifactStoreInner>>,
}

#[derive(Default)]
struct ArtifactStoreInner {
    entries: HashMap<BrowserArtifactRef, ArtifactEntry>,
    bytes: usize,
}

struct ArtifactEntry {
    owner: super::BrowserPermissionResource,
    generation: u64,
    bytes: Vec<u8>,
    expires: Instant,
    dimensions: (u32, u32),
}

const ARTIFACT_TTL: Duration = Duration::from_secs(120);
const ARTIFACT_MAX_BYTES: usize = 2 * 1024 * 1024;
const ARTIFACT_MAX_TOTAL: usize = 16 * 1024 * 1024;
const ARTIFACT_MAX_ENTRIES: usize = 32;

impl BrowserArtifactStore {
    /// Stores verified PNG bytes for one exact invocation and session generation.
    pub(crate) fn insert(
        &self,
        owner: super::BrowserPermissionResource,
        generation: u64,
        bytes: Vec<u8>,
        width: u32,
        height: u32,
    ) -> Result<BrowserArtifactRef, BrowserOutputError> {
        validate_png(&bytes, width, height)?;
        let mut inner = self.inner.lock().map_err(|_| BrowserOutputError)?;
        purge_expired(&mut inner);
        if bytes.len() > ARTIFACT_MAX_BYTES
            || bytes.len().saturating_add(inner.bytes) > ARTIFACT_MAX_TOTAL
            || inner.entries.len() >= ARTIFACT_MAX_ENTRIES
        {
            return Err(BrowserOutputError);
        }
        let random =
            std::panic::catch_unwind(uuid::Uuid::new_v4).map_err(|_| BrowserOutputError)?;
        let reference = BrowserArtifactRef::try_new(format!("artifact_{}", random.simple()))
            .map_err(|_| BrowserOutputError)?;
        if inner.entries.contains_key(&reference) {
            return Err(BrowserOutputError);
        }
        inner.bytes += bytes.len();
        inner.entries.insert(
            reference.clone(),
            ArtifactEntry {
                owner,
                generation,
                bytes,
                expires: Instant::now() + ARTIFACT_TTL,
                dimensions: (width, height),
            },
        );
        Ok(reference)
    }

    /// Consumes one capability. It cannot be replayed, used by another invocation, or used after
    /// lifecycle invalidation. Returned bytes are removed from the transient store.
    pub(crate) fn consume(
        &self,
        reference: &BrowserArtifactRef,
        owner: &super::BrowserPermissionResource,
        generation: u64,
    ) -> Result<(Vec<u8>, Instant), BrowserOutputError> {
        let mut inner = self.inner.lock().map_err(|_| BrowserOutputError)?;
        purge_expired(&mut inner);
        let entry = inner.entries.get(reference).ok_or(BrowserOutputError)?;
        if &entry.owner != owner || entry.generation != generation {
            return Err(BrowserOutputError);
        }
        let entry = inner.entries.remove(reference).ok_or(BrowserOutputError)?;
        inner.bytes = inner.bytes.saturating_sub(entry.bytes.len());
        Ok((entry.bytes, entry.expires))
    }

    pub(crate) fn validate_release(
        &self,
        output: &BrowserOutput,
        owner: &super::BrowserPermissionResource,
        generation: u64,
    ) -> Result<(), BrowserOutputError> {
        let BrowserOutput::Screenshot {
            artifact_ref,
            width,
            height,
            byte_length,
            ..
        } = output
        else {
            self.discard_owner(owner);
            return Ok(());
        };
        let mut inner = self.inner.lock().map_err(|_| BrowserOutputError)?;
        purge_expired(&mut inner);
        let entry = inner.entries.get(artifact_ref).ok_or(BrowserOutputError)?;
        if &entry.owner != owner
            || entry.generation != generation
            || entry.dimensions != (*width, *height)
            || entry.bytes.len() as u64 != *byte_length
        {
            return Err(BrowserOutputError);
        }
        // A call may publish only its selected result; intermediate captures cannot linger.
        inner
            .entries
            .retain(|reference, entry| &entry.owner != owner || reference == artifact_ref);
        inner.bytes = inner.entries.values().map(|entry| entry.bytes.len()).sum();
        Ok(())
    }

    /// Discards all transient capabilities for one invocation.
    pub(crate) fn discard_owner(&self, owner: &super::BrowserPermissionResource) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.entries.retain(|_, entry| &entry.owner != owner);
            inner.bytes = inner.entries.values().map(|entry| entry.bytes.len()).sum();
        }
    }
}

fn purge_expired(inner: &mut ArtifactStoreInner) {
    let now = Instant::now();
    inner.entries.retain(|_, entry| entry.expires > now);
    inner.bytes = inner.entries.values().map(|entry| entry.bytes.len()).sum();
}

fn validate_png(bytes: &[u8], width: u32, height: u32) -> Result<(), BrowserOutputError> {
    const SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    if bytes.len() < 33 || bytes.len() > ARTIFACT_MAX_BYTES || &bytes[..8] != SIGNATURE {
        return Err(BrowserOutputError);
    }
    let ihdr_len = u32::from_be_bytes(bytes[8..12].try_into().map_err(|_| BrowserOutputError)?);
    if ihdr_len != 13 || &bytes[12..16] != b"IHDR" {
        return Err(BrowserOutputError);
    }
    let actual_width =
        u32::from_be_bytes(bytes[16..20].try_into().map_err(|_| BrowserOutputError)?);
    let actual_height =
        u32::from_be_bytes(bytes[20..24].try_into().map_err(|_| BrowserOutputError)?);
    if actual_width != width
        || actual_height != height
        || !(1..=2048).contains(&width)
        || !(1..=2048).contains(&height)
        || u64::from(width) * u64::from(height) > 4_000_000
    {
        return Err(BrowserOutputError);
    }
    // Header checks bound the decode before allocation; full decoding rejects truncated or
    // corrupt compressed data. Contain dependency panics at this untrusted byte boundary.
    std::panic::catch_unwind(|| {
        let mut reader =
            image::ImageReader::with_format(std::io::Cursor::new(bytes), image::ImageFormat::Png);
        let mut limits = image::Limits::default();
        limits.max_image_width = Some(2048);
        limits.max_image_height = Some(2048);
        limits.max_alloc = Some(64 * 1024 * 1024);
        reader.limits(limits);
        reader.decode().map_err(|_| BrowserOutputError)
    })
    .map_err(|_| BrowserOutputError)??;
    Ok(())
}

/// Output that passed structural validation; host isolation/redaction remains a separate gate.
pub struct ValidatedBrowserOutput {
    model_json: String,
    summary: String,
    is_failure: bool,
}

impl ValidatedBrowserOutput {
    /// Whether the validated payload is a typed failure, including known partial effects.
    pub fn is_failure(&self) -> bool {
        self.is_failure
    }
    /// Returns the bounded model projection. Page data must be treated as untrusted content.
    pub fn model_json(&self) -> &str {
        &self.model_json
    }
    /// Returns the content-free summary shared by display and persistence.
    pub fn summary(&self) -> &str {
        &self.summary
    }
}

fn clean_origin(value: &str, opaque_allowed: bool) -> bool {
    if opaque_allowed && value == "opaque" {
        return true;
    }
    if value.len() > 2048 {
        return false;
    }
    let Ok(url) = reqwest::Url::parse(value) else {
        return false;
    };
    matches!(url.scheme(), "http" | "https")
        && url.host_str().is_some()
        && url.origin().ascii_serialization() == value
}

fn tree_valid<T: Eq + Hash>(nodes: &[(&T, Option<&T>)], max_depth: usize) -> bool {
    let index: HashMap<&T, Option<&T>> = nodes.iter().copied().collect();
    if index.len() != nodes.len() {
        return false;
    }
    for (node, _) in nodes {
        let mut current = Some(*node);
        let mut depth = 0;
        while let Some(reference) = current {
            depth += 1;
            if depth > max_depth {
                return false;
            }
            let Some(parent) = index.get(reference) else {
                return false;
            };
            current = *parent;
        }
    }
    true
}

impl BrowserOutput {
    /// Checks operation correspondence, tree integrity, metadata and serialized byte budgets.
    /// Does not prove host document isolation, artifact ownership or sensitive-data redaction.
    pub fn validate(
        self,
        expected: Option<BrowserOperation>,
    ) -> Result<ValidatedBrowserOutput, BrowserOutputError> {
        use BrowserOperation as Op;
        let (kind, limit, valid) = match &self {
            Self::Ack { operation, .. } => (
                "Ack",
                1024,
                Some(*operation) == expected
                    && matches!(
                        operation,
                        Op::Open
                            | Op::Click
                            | Op::Fill
                            | Op::Select
                            | Op::Hover
                            | Op::Check
                            | Op::Uncheck
                            | Op::Press
                            | Op::Scroll
                            | Op::WaitForElement
                            | Op::WaitMilliseconds
                            | Op::TabClose
                            | Op::TabSwitch
                            | Op::WindowClose
                    ),
            ),
            Self::TabCreated { .. } => ("TabCreated", 1024, expected == Some(Op::TabNew)),
            Self::Tabs { entries, .. } => (
                "Tabs",
                16 * 1024,
                expected == Some(Op::TabList)
                    && entries.len() <= 64
                    && entries.iter().all(|e| clean_origin(&e.origin, true))
                    && entries
                        .iter()
                        .map(|e| &e.tab_ref)
                        .collect::<std::collections::HashSet<_>>()
                        .len()
                        == entries.len(),
            ),
            Self::Frames { entries, .. } => (
                "Frames",
                64 * 1024,
                expected == Some(Op::FrameTree)
                    && entries.len() <= 128
                    && entries.iter().all(|e| clean_origin(&e.origin, true))
                    && (entries.is_empty()
                        || entries
                            .iter()
                            .filter(|e| e.parent_frame_ref.is_none())
                            .count()
                            == 1)
                    && tree_valid(
                        &entries
                            .iter()
                            .map(|e| (&e.frame_ref, e.parent_frame_ref.as_ref()))
                            .collect::<Vec<_>>(),
                        16,
                    ),
            ),
            Self::PageUrl { origin, path } => (
                "PageUrl",
                4096,
                expected == Some(Op::CurrentUrl)
                    && clean_origin(origin, false)
                    && path.len() <= 4096
                    && path.starts_with('/')
                    && !path.contains(['?', '#', '\\'])
                    && !path.chars().any(|c| c.is_control() || c.is_whitespace()),
            ),
            Self::Read { text, .. } => (
                "Read",
                40 * 1024,
                expected == Some(Op::Read) && text.len() <= 32 * 1024 && !text.contains('\0'),
            ),
            Self::Snapshot { nodes, .. } => (
                "Snapshot",
                64 * 1024,
                expected == Some(Op::Snapshot)
                    && nodes.len() <= 512
                    && nodes.iter().all(|n| {
                        n.name.len() <= 256
                            && !n.name.contains('\0')
                            && n.states.len() <= 9
                            && n.states
                                .iter()
                                .collect::<std::collections::HashSet<_>>()
                                .len()
                                == n.states.len()
                    })
                    && tree_valid(
                        &nodes
                            .iter()
                            .map(|n| (&n.element_ref, n.parent_element_ref.as_ref()))
                            .collect::<Vec<_>>(),
                        32,
                    ),
            ),
            Self::Screenshot {
                width,
                height,
                byte_length,
                ..
            } => (
                "Screenshot",
                96 * 1024,
                expected == Some(Op::Screenshot)
                    && (1..=2048).contains(width)
                    && (1..=2048).contains(height)
                    && u64::from(*width) * u64::from(*height) <= 4_000_000
                    && (1..=2 * 1024 * 1024).contains(byte_length),
            ),
            Self::Failure {
                operation, outcome, ..
            } => (
                "Failure",
                1024,
                *operation == expected
                    && (expected.is_some() || *outcome == BrowserOutcome::NotExecuted),
            ),
        };
        if !valid {
            return Err(BrowserOutputError);
        }
        let model_json = serde_json::to_string(&self).map_err(|_| BrowserOutputError)?;
        if model_json.len() > limit || model_json.len() > 96 * 1024 {
            return Err(BrowserOutputError);
        }
        // Construct the summary from fixed vocabulary only: no URL, page text, input value,
        // snapshot name, artifact reference or driver diagnostic can enter durable views.
        let summary = match &self {
            Self::Failure {
                code,
                operation,
                outcome,
            } => format!("Browser {operation:?}: {code:?} ({outcome:?})"),
            _ => format!("Browser {expected:?}: {kind}"),
        };
        Ok(ValidatedBrowserOutput {
            model_json,
            summary,
            is_failure: matches!(self, Self::Failure { .. }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn validate(
        value: Value,
        operation: BrowserOperation,
    ) -> Result<ValidatedBrowserOutput, BrowserOutputError> {
        serde_json::from_value::<BrowserOutput>(value)
            .map_err(|_| BrowserOutputError)?
            .validate(Some(operation))
    }

    const PNG_1X1: &[u8] = &[
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0, 0, 0, 0x0d, 0x49, 0x48, 0x44, 0x52, 0,
        0, 0, 0x01, 0, 0, 0, 0x01, 8, 2, 0, 0, 0, 0x90, 0x77, 0x53, 0xde, 0, 0, 0, 0x0c, 0x49,
        0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0xf8, 0xff, 0xff, 0x3f, 0, 5, 0xfe, 2, 0xfe, 0xa3,
        0x35, 0x81, 0x84, 0, 0, 0, 0, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];

    #[test]
    fn artifact_store_validates_binds_and_consumes_png_once() {
        let store = BrowserArtifactStore::default();
        let owner = artifact_owner();
        let mut png = std::io::Cursor::new(Vec::new());
        image::DynamicImage::new_rgb8(1, 1)
            .write_to(&mut png, image::ImageFormat::Png)
            .expect("encode fixture");
        let png = png.into_inner();
        let reference = store
            .insert(owner.clone(), 3, png.clone(), 1, 1)
            .expect("png");
        assert!(store.consume(&reference, &artifact_owner(), 3).is_err());
        let (bytes, _) = store.consume(&reference, &owner, 3).expect("owner");
        assert_eq!(bytes, png);
        assert!(store.consume(&reference, &owner, 3).is_err());
    }

    fn artifact_owner() -> super::super::BrowserPermissionResource {
        let mut context = super::super::BrowserContext::new();
        let request = super::super::BrowserRequest::parse_raw(
            r#"{"protocolVersion":2,"operation":"wait-milliseconds","milliseconds":1}"#,
        )
        .expect("request");
        let invocation = context.prepare_invocation(request).expect("invocation");
        context
            .invocation_permission_resource(&invocation)
            .expect("resource")
    }

    #[test]
    fn artifact_store_rejects_fake_or_mismatched_png_metadata() {
        let store = BrowserArtifactStore::default();
        assert!(
            store
                .insert(artifact_owner(), 1, vec![0; 67], 1, 1)
                .is_err()
        );
        assert!(
            store
                .insert(artifact_owner(), 1, PNG_1X1.to_vec(), 2, 1)
                .is_err()
        );
    }

    fn encoded_png() -> Vec<u8> {
        let mut bytes = std::io::Cursor::new(Vec::new());
        image::DynamicImage::new_rgb8(1, 1)
            .write_to(&mut bytes, image::ImageFormat::Png)
            .expect("encode");
        bytes.into_inner()
    }

    #[test]
    fn artifacts_expire_reclaim_capacity_and_reject_wrong_generation() {
        let store = BrowserArtifactStore::default();
        let owner = artifact_owner();
        let png = encoded_png();
        let reference = store
            .insert(owner.clone(), 7, png.clone(), 1, 1)
            .expect("insert");
        assert!(store.consume(&reference, &owner, 8).is_err());
        {
            let mut inner = store.inner.lock().expect("lock");
            inner.entries.get_mut(&reference).expect("entry").expires = Instant::now();
        }
        assert!(store.consume(&reference, &owner, 7).is_err());
        assert_eq!(store.inner.lock().expect("lock").bytes, 0);
        for _ in 0..ARTIFACT_MAX_ENTRIES {
            store
                .insert(owner.clone(), 7, png.clone(), 1, 1)
                .expect("capacity");
        }
        assert!(store.insert(owner.clone(), 7, png.clone(), 1, 1).is_err());
        store.discard_owner(&artifact_owner());
        assert_eq!(
            store.inner.lock().expect("lock").entries.len(),
            ARTIFACT_MAX_ENTRIES
        );
        store.discard_owner(&owner);
        assert_eq!(store.inner.lock().expect("lock").bytes, 0);
        assert!(store.insert(owner, 7, png, 1, 1).is_ok());
    }

    #[test]
    fn png_payload_corruption_and_encoded_byte_limit_fail_closed() {
        let store = BrowserArtifactStore::default();
        let png = encoded_png();
        assert!(
            store
                .insert(artifact_owner(), 1, png[..33].to_vec(), 1, 1)
                .is_err()
        );
        let mut corrupt = png.clone();
        corrupt[29] ^= 0xff;
        assert!(store.insert(artifact_owner(), 1, corrupt, 1, 1).is_err());
        let mut oversized = png;
        oversized.resize(ARTIFACT_MAX_BYTES + 1, 0);
        assert!(store.insert(artifact_owner(), 1, oversized, 1, 1).is_err());
    }

    #[test]
    fn release_requires_exact_registered_capture_and_reclaims_intermediates() {
        let store = BrowserArtifactStore::default();
        let owner = artifact_owner();
        let bytes = encoded_png();
        let first = store
            .insert(owner.clone(), 1, bytes.clone(), 1, 1)
            .expect("first");
        let selected = store
            .insert(owner.clone(), 1, bytes.clone(), 1, 1)
            .expect("selected");
        let output = |reference, width, byte_length| BrowserOutput::Screenshot {
            artifact_ref: reference,
            mime: BrowserScreenshotMime::Png,
            width,
            height: 1,
            byte_length,
        };
        assert!(
            store
                .validate_release(&output(selected.clone(), 2, bytes.len() as u64), &owner, 1)
                .is_err()
        );
        assert!(
            store
                .validate_release(&output(selected.clone(), 1, 1), &owner, 1)
                .is_err()
        );
        assert!(
            store
                .validate_release(
                    &output(first.clone(), 1, bytes.len() as u64),
                    &artifact_owner(),
                    1
                )
                .is_err()
        );
        store
            .validate_release(&output(selected.clone(), 1, bytes.len() as u64), &owner, 1)
            .expect("release");
        assert!(store.consume(&first, &owner, 1).is_err());
        assert_eq!(
            store.consume(&selected, &owner, 1).expect("selected").0,
            bytes
        );
        assert!(
            store
                .validate_release(&output(selected, 1, 1), &owner, 1)
                .is_err()
        );
    }

    #[test]
    fn closed_outputs_match_only_their_operation_and_have_content_free_summaries() {
        let fixtures = [
            (
                json!({"kind":"Ack","operation":"click","status":"ok"}),
                BrowserOperation::Click,
            ),
            (
                json!({"kind":"TabCreated","tabRef":"t"}),
                BrowserOperation::TabNew,
            ),
            (
                json!({"kind":"Tabs","entries":[{"tabRef":"t","origin":"opaque"}],"truncated":false}),
                BrowserOperation::TabList,
            ),
            (
                json!({"kind":"Frames","entries":[{"frameRef":"f","parentFrameRef":null,"origin":"https://secret.example","readiness":"Ready"}],"truncated":false}),
                BrowserOperation::FrameTree,
            ),
            (
                json!({"kind":"PageUrl","origin":"https://secret.example","path":"/secret"}),
                BrowserOperation::CurrentUrl,
            ),
            (
                json!({"kind":"Read","text":"secret","truncated":false}),
                BrowserOperation::Read,
            ),
            (
                json!({"kind":"Snapshot","snapshotRef":"s","nodes":[{"elementRef":"e","parentElementRef":null,"role":"button","name":"secret","states":["disabled"]}],"truncated":false}),
                BrowserOperation::Snapshot,
            ),
            (
                json!({"kind":"Screenshot","artifactRef":"secret","mime":"image/png","width":100,"height":100,"byteLength":100}),
                BrowserOperation::Screenshot,
            ),
            (
                json!({"kind":"Failure","code":"IndeterminateExecution","operation":"click","outcome":"unknown"}),
                BrowserOperation::Click,
            ),
        ];
        for (value, operation) in fixtures {
            let output = validate(value.clone(), operation).expect("valid output");
            assert!(!output.summary().contains("secret"));
            assert!(output.summary().len() < 4096);
            assert_eq!(
                serde_json::from_str::<Value>(output.model_json()).expect("json"),
                value
            );
            assert!(validate(value.clone(), BrowserOperation::Open).is_err());
            let mut extra = value;
            extra["driverMessage"] = "secret".into();
            assert!(validate(extra, operation).is_err());
        }
    }

    #[test]
    fn trees_reject_missing_parents_duplicates_cycles_and_overdepth() {
        assert!(tree_valid(&[(&2, Some(&1)), (&1, None)], 2));
        assert!(!tree_valid(&[(&1, Some(&2))], 16));
        assert!(!tree_valid(&[(&1, None), (&1, None)], 16));
        assert!(!tree_valid(&[(&1, Some(&2)), (&2, Some(&1))], 16));
        assert!(!tree_valid(&[(&1, None), (&2, Some(&1))], 1));
        let missing = json!({"kind":"Frames","entries":[{"frameRef":"f","origin":"opaque","readiness":"Pending"}],"truncated":false});
        assert!(validate(missing, BrowserOperation::FrameTree).is_err());
    }

    #[test]
    fn output_budgets_apply_to_utf8_and_serialized_envelope() {
        for text in ["界".repeat(10923), "\u{1}".repeat(10000)] {
            assert!(
                validate(
                    json!({"kind":"Read","text":text,"truncated":true}),
                    BrowserOperation::Read
                )
                .is_err()
            );
        }
        assert!(
            validate(
                json!({"kind":"Read","text":"x".repeat(32768),"truncated":false}),
                BrowserOperation::Read
            )
            .is_ok()
        );
        for (width, height, size) in [
            (2049, 1, 1),
            (2048, 2048, 1),
            (100, 100, 2_097_153),
            (0, 1, 1),
        ] {
            assert!(validate(json!({"kind":"Screenshot","artifactRef":"a","mime":"image/png","width":width,"height":height,"byteLength":size}), BrowserOperation::Screenshot).is_err());
        }
        assert!(validate(json!({"kind":"Snapshot","snapshotRef":"s","nodes":[{"elementRef":"e","parentElementRef":null,"role":"button","name":"x","states":["disabled","disabled"]}],"truncated":false}), BrowserOperation::Snapshot).is_err());
        assert!(validate(json!({"kind":"Tabs","entries":vec![json!({"tabRef":"t","origin":"opaque"}); 65],"truncated":true}), BrowserOperation::TabList).is_err());
    }

    #[test]
    fn urls_inventory_and_errors_reject_leaky_metadata() {
        for origin in [
            "https://example.com/?secret",
            "https://user:secret@example.com",
            "https://example.com#secret",
            "https://example.com/path",
            "HTTPS://EXAMPLE.COM",
            "null",
        ] {
            assert!(!clean_origin(origin, true));
        }
        for path in [
            "/x?secret",
            "/x#secret",
            "/x\nsecret",
            "https://secret.example",
        ] {
            assert!(
                validate(
                    json!({"kind":"PageUrl","origin":"https://example.com","path":path}),
                    BrowserOperation::CurrentUrl
                )
                .is_err()
            );
        }
        let missing_operation =
            json!({"kind":"Failure","code":"InvalidRequest","outcome":"notExecuted"});
        assert!(serde_json::from_value::<BrowserOutput>(missing_operation).is_err());
        let unadmitted: BrowserOutput = serde_json::from_value(json!({"kind":"Failure","code":"InvalidRequest","operation":null,"outcome":"notExecuted"})).expect("failure");
        assert!(unadmitted.validate(None).is_ok());
    }
}
