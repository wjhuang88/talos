//! First-class, Session-scoped permission grant values.

use std::fmt;
use std::path::{Path, PathBuf};

use schemars::JsonSchema;
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use talos_core::tool::{ToolNature, ToolProvenance, ToolResourceKind, normalize_authorized_path};
use thiserror::Error;
use uuid::Uuid;

use crate::PermissionRequest;

const FINGERPRINT_DOMAIN: &[u8] = b"talos.permission.request-fingerprint\0v1\0";

/// Supported lifetime of an explicit permission approval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GrantScope {
    /// Authority is consumed by one official adapter invocation.
    Once,
    /// Authority may be reused only by the owning in-memory Session.
    Session,
}

/// Closed class of explicit approver that may create grant authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GrantSource {
    /// A human approved through an interactive Talos surface.
    InteractiveHuman,
    /// A configured embedded-runtime host approval handler approved.
    SdkHostApproval,
}

/// Opaque, Session-local grant identifier used for safe diagnostics.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct GrantId(Uuid);

impl GrantId {
    pub(crate) fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl fmt::Debug for GrantId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("GrantId").field(&self.0).finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
enum CompiledResource {
    Path(PathBuf),
    Text(String),
}

/// One normalized facet in a proposal or Session grant.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct CompiledFacetScope {
    nature: ToolNature,
    resource_kind: ToolResourceKind,
    normalized_resource: Option<CompiledResource>,
}

impl fmt::Debug for CompiledFacetScope {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CompiledFacetScope")
            .field("nature", &self.nature)
            .field("resource_kind", &self.resource_kind)
            .field("normalized_resource", &"<redacted>")
            .finish()
    }
}

/// Bounded approval-surface projection produced by the grant compiler.
#[derive(Clone, PartialEq, Eq)]
pub struct GrantPreview {
    scope: GrantScope,
    facets: Vec<GrantPreviewFacet>,
}

impl fmt::Debug for GrantPreview {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GrantPreview")
            .field("scope", &self.scope)
            .field("facet_count", &self.facets.len())
            .finish()
    }
}

impl GrantPreview {
    /// Returns the requested approval lifetime.
    #[must_use]
    pub const fn scope(&self) -> GrantScope {
        self.scope
    }

    /// Returns the normalized facet descriptions that will be approved.
    #[must_use]
    pub fn facets(&self) -> &[GrantPreviewFacet] {
        &self.facets
    }
}

/// One bounded facet shown on the local approval surface.
#[derive(Clone, PartialEq, Eq)]
pub struct GrantPreviewFacet {
    /// Risk nature of the facet.
    pub nature: ToolNature,
    /// Typed interpretation of the resource.
    pub resource_kind: ToolResourceKind,
    /// Exact normalized scope, or `<none>` for a non-consequential facet.
    pub normalized_scope: String,
}

impl fmt::Debug for GrantPreviewFacet {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GrantPreviewFacet")
            .field("nature", &self.nature)
            .field("resource_kind", &self.resource_kind)
            .field("normalized_scope", &"<redacted>")
            .finish()
    }
}

pub(crate) struct RequestFingerprint([u8; 32]);

impl PartialEq for RequestFingerprint {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for RequestFingerprint {}

impl fmt::Debug for RequestFingerprint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("<redacted-request-fingerprint>")
    }
}

/// Non-authoritative result of compiling a structured permission request.
pub struct ProposedGrant {
    pub(crate) request_id: Uuid,
    pub(crate) snapshot: ProposalSnapshot,
    pub(crate) scope: GrantScope,
    pub(crate) tool_name: String,
    pub(crate) provenance: ToolProvenance,
    pub(crate) facets: Vec<CompiledFacetScope>,
    pub(crate) fingerprint: RequestFingerprint,
    pub(crate) preview: GrantPreview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProposalSnapshot {
    pub(crate) session_id: Uuid,
    pub(crate) revisions: [u64; 6],
    pub(crate) mode: crate::PermissionMode,
    pub(crate) interaction: crate::InteractionCapability,
}

impl ProposedGrant {
    /// Returns the bounded preview derived from the exact compiled proposal.
    #[must_use]
    pub const fn preview(&self) -> &GrantPreview {
        &self.preview
    }

    /// Returns the proposed approval lifetime.
    #[must_use]
    pub const fn scope(&self) -> GrantScope {
        self.scope
    }
}

impl fmt::Debug for ProposedGrant {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProposedGrant")
            .field("request_id", &self.request_id)
            .field("scope", &self.scope)
            .field("tool", &"<redacted>")
            .field("provenance", &"<redacted>")
            .field("facet_count", &self.facets.len())
            .field("fingerprint", &self.fingerprint)
            .finish()
    }
}

/// Approved invocation-local authority. This value is intentionally not `Clone`.
pub(crate) struct ApprovedOnce {
    pub(crate) tool_name: String,
    pub(crate) provenance: ToolProvenance,
    pub(crate) facets: Vec<CompiledFacetScope>,
    pub(crate) fingerprint: RequestFingerprint,
}

impl fmt::Debug for ApprovedOnce {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ApprovedOnce")
            .field("tool", &"<redacted>")
            .field("provenance", &"<redacted>")
            .field("facet_count", &self.facets.len())
            .finish()
    }
}

/// Reusable authority owned by one in-memory permission Session.
pub(crate) struct PermissionGrant {
    pub(crate) id: GrantId,
    pub(crate) source: GrantSource,
    pub(crate) tool_name: String,
    pub(crate) provenance: ToolProvenance,
    pub(crate) facets: Vec<CompiledFacetScope>,
}

impl fmt::Debug for PermissionGrant {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PermissionGrant")
            .field("id", &self.id)
            .field("source", &self.source)
            .field("tool", &"<redacted>")
            .field("provenance", &"<redacted>")
            .field("facet_count", &self.facets.len())
            .finish()
    }
}

/// Failure to compile or validate scoped grant authority.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum GrantError {
    /// The permission Session lock was poisoned.
    #[error("permission Session state is unavailable")]
    StateUnavailable,
    /// The request is not awaiting approval.
    #[error("permission request is not awaiting approval")]
    NotAwaitingApproval,
    /// The request or state changed after the proposal was created.
    #[error("permission approval is stale")]
    StaleApproval,
    /// A consequential facet has no safe typed resource.
    #[error("permission facet {0} has no safe typed resource")]
    MissingResource(usize),
    /// A path could not be normalized for exact matching.
    #[error("permission path could not be normalized")]
    InvalidPath,
    /// The proposal scope is incompatible with the requested transition.
    #[error("permission proposal scope is incompatible")]
    ScopeMismatch,
    /// Admission was invalidated before the tool started.
    #[error("permission admission was invalidated")]
    AdmissionInvalidated,
}

pub(crate) fn compile_proposal(
    request: &PermissionRequest<'_>,
    workspace_root: Option<&Path>,
    scope: GrantScope,
    snapshot: ProposalSnapshot,
) -> Result<ProposedGrant, GrantError> {
    let facets = compile_facets(request, workspace_root, true)?;
    let fingerprint = fingerprint_request(request, &facets);
    let preview = GrantPreview {
        scope,
        facets: facets
            .iter()
            .map(|facet| GrantPreviewFacet {
                nature: facet.nature,
                resource_kind: facet.resource_kind,
                normalized_scope: facet
                    .normalized_resource
                    .as_ref()
                    .map(preview_resource)
                    .unwrap_or_else(|| "<none>".to_string()),
            })
            .collect(),
    };
    Ok(ProposedGrant {
        request_id: Uuid::new_v4(),
        snapshot,
        scope,
        tool_name: request.tool_name().to_string(),
        provenance: request.provenance().clone(),
        facets,
        fingerprint,
        preview,
    })
}

pub(crate) fn compiled_identity(
    request: &PermissionRequest<'_>,
    workspace_root: Option<&Path>,
) -> Result<(Vec<CompiledFacetScope>, RequestFingerprint), GrantError> {
    let facets = compile_facets(request, workspace_root, false)?;
    let fingerprint = fingerprint_request(request, &facets);
    Ok((facets, fingerprint))
}

fn compile_facets(
    request: &PermissionRequest<'_>,
    workspace_root: Option<&Path>,
    require_grant_scope: bool,
) -> Result<Vec<CompiledFacetScope>, GrantError> {
    let mut compiled = Vec::with_capacity(request.facets().len());
    for (index, facet) in request.facets().iter().enumerate() {
        let consequential = matches!(
            facet.nature,
            ToolNature::Write | ToolNature::Execute | ToolNature::Network
        ) || facet.resource_kind.is_some();
        let Some(resource_kind) = facet.resource_kind else {
            if consequential && require_grant_scope {
                return Err(GrantError::MissingResource(index));
            }
            compiled.push(CompiledFacetScope {
                nature: facet.nature,
                resource_kind: ToolResourceKind::Remote,
                normalized_resource: None,
            });
            continue;
        };
        if !supported_resource_pair(facet.nature, resource_kind) {
            return Err(GrantError::MissingResource(index));
        }
        let raw_resource = facet.resource.clone();
        let normalized_resource = match (resource_kind, raw_resource) {
            (ToolResourceKind::Path, Some(resource)) => {
                let root = workspace_root.ok_or(GrantError::InvalidPath)?;
                let normalized = normalize_authorized_path(root, &resource)
                    .map_err(|_| GrantError::InvalidPath)?;
                Some(CompiledResource::Path(normalized))
            }
            (ToolResourceKind::Domain, Some(resource)) => {
                let host = url::Host::parse(resource.trim())
                    .map_err(|_| GrantError::MissingResource(index))?;
                Some(CompiledResource::Text(
                    host.to_string().to_ascii_lowercase(),
                ))
            }
            (ToolResourceKind::Command | ToolResourceKind::Remote, Some(resource)) => {
                let resource = resource.trim();
                if resource.is_empty() {
                    return Err(GrantError::MissingResource(index));
                }
                Some(CompiledResource::Text(resource.to_string()))
            }
            (_, None) if consequential && require_grant_scope => {
                return Err(GrantError::MissingResource(index));
            }
            (_, None) => None,
        };
        compiled.push(CompiledFacetScope {
            nature: facet.nature,
            resource_kind,
            normalized_resource,
        });
    }
    Ok(compiled)
}

const fn supported_resource_pair(nature: ToolNature, kind: ToolResourceKind) -> bool {
    matches!(
        (nature, kind),
        (ToolNature::Read | ToolNature::Write, ToolResourceKind::Path)
            | (
                ToolNature::Execute,
                ToolResourceKind::Command | ToolResourceKind::Path
            )
            | (
                ToolNature::Network,
                ToolResourceKind::Domain | ToolResourceKind::Remote
            )
            | (ToolNature::Internal, ToolResourceKind::Remote)
    )
}

fn preview_resource(resource: &CompiledResource) -> String {
    match resource {
        CompiledResource::Path(path) => path.to_string_lossy().into_owned(),
        CompiledResource::Text(value) => value.clone(),
    }
}

fn fingerprint_request(
    request: &PermissionRequest<'_>,
    facets: &[CompiledFacetScope],
) -> RequestFingerprint {
    let mut hasher = Sha256::new();
    hasher.update(FINGERPRINT_DOMAIN);
    hash_text(&mut hasher, request.tool_name());
    hash_provenance(&mut hasher, request.provenance());
    hash_len(&mut hasher, facets.len());
    for facet in facets {
        hasher.update([nature_tag(facet.nature), kind_tag(facet.resource_kind)]);
        hash_optional_resource(&mut hasher, facet.normalized_resource.as_ref());
    }
    hash_json(&mut hasher, request.input());
    RequestFingerprint(hasher.finalize().into())
}

fn hash_provenance(hasher: &mut Sha256, provenance: &ToolProvenance) {
    match provenance {
        ToolProvenance::Native => hasher.update([0]),
        ToolProvenance::McpRemote { server } => {
            hasher.update([1]);
            hash_text(hasher, server);
        }
        ToolProvenance::Plugin {
            name,
            version,
            carrier,
        } => {
            hasher.update([2]);
            hash_text(hasher, name);
            hash_text(hasher, version);
            hash_text(hasher, carrier);
        }
    }
}

fn hash_json(hasher: &mut Sha256, value: &Value) {
    match value {
        Value::Null => hasher.update([0]),
        Value::Bool(value) => hasher.update([1, u8::from(*value)]),
        Value::Number(value) => {
            hasher.update([2]);
            hash_text(hasher, &value.to_string());
        }
        Value::String(value) => {
            hasher.update([3]);
            hash_text(hasher, value);
        }
        Value::Array(values) => {
            hasher.update([4]);
            hash_len(hasher, values.len());
            for value in values {
                hash_json(hasher, value);
            }
        }
        Value::Object(values) => {
            hasher.update([5]);
            hash_len(hasher, values.len());
            let mut keys = values.keys().collect::<Vec<_>>();
            keys.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
            for key in keys {
                hash_text(hasher, key);
                hash_json(hasher, &values[key]);
            }
        }
    }
}

fn hash_optional_resource(hasher: &mut Sha256, value: Option<&CompiledResource>) {
    match value {
        Some(CompiledResource::Path(value)) => {
            hasher.update([1]);
            hash_path(hasher, value);
        }
        Some(CompiledResource::Text(value)) => {
            hasher.update([2]);
            hash_text(hasher, value);
        }
        None => hasher.update([0]),
    }
}

#[cfg(unix)]
fn hash_path(hasher: &mut Sha256, value: &Path) {
    use std::os::unix::ffi::OsStrExt;

    let bytes = value.as_os_str().as_bytes();
    hash_len(hasher, bytes.len());
    hasher.update(bytes);
}

#[cfg(windows)]
fn hash_path(hasher: &mut Sha256, value: &Path) {
    use std::os::windows::ffi::OsStrExt;

    let units = value.as_os_str().encode_wide().collect::<Vec<_>>();
    hash_len(hasher, units.len());
    for unit in units {
        hasher.update(unit.to_be_bytes());
    }
}

fn hash_text(hasher: &mut Sha256, value: &str) {
    hash_len(hasher, value.len());
    hasher.update(value.as_bytes());
}

fn hash_len(hasher: &mut Sha256, value: usize) {
    hasher.update(u64::try_from(value).unwrap_or(u64::MAX).to_be_bytes());
}

const fn nature_tag(value: ToolNature) -> u8 {
    match value {
        ToolNature::Read => 0,
        ToolNature::Write => 1,
        ToolNature::Execute => 2,
        ToolNature::Network => 3,
        ToolNature::Internal => 4,
    }
}

const fn kind_tag(value: ToolResourceKind) -> u8 {
    match value {
        ToolResourceKind::Path => 0,
        ToolResourceKind::Domain => 1,
        ToolResourceKind::Command => 2,
        ToolResourceKind::Remote => 3,
    }
}
