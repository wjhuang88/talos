//! First-class, Session-scoped permission grant values.

use std::fmt;
use std::path::{Path, PathBuf};

use schemars::JsonSchema;
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use talos_core::tool::{
    ToolNature, ToolNetworkScope, ToolProvenance, ToolResourceKind, normalize_authorized_path,
};
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
    network_scope: Option<Box<ToolNetworkScope>>,
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

    /// Returns exact destination/configuration scope for the local approval renderer.
    /// It contains no query values, credentials or invocation fingerprints.
    pub fn network_scope(&self) -> Option<&ToolNetworkScope> {
        self.network_scope.as_deref()
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
    pub(crate) network_scope: Option<ToolNetworkScope>,
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
    pub(crate) network_scope: Option<ToolNetworkScope>,
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
        // The existing Ask carrier includes both proposal slots. Even its
        // Session slot advertises only Once for native command authority;
        // committing it is downgraded to an invocation without storing a grant.
        scope: if crate::native_pull_command::requested(request.facets()) {
            GrantScope::Once
        } else {
            scope
        },
        network_scope: request
            .network_manifest()
            .map(|m| Box::new(m.scope().clone())),
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
    validate_network_profile(request)?;
    let mut compiled = Vec::with_capacity(request.facets().len());
    for (index, facet) in request.facets().iter().enumerate() {
        if facet.resource.as_deref().map(str::trim) == Some(crate::native_pull_command::CAPABILITY)
            && (facet.nature != ToolNature::Execute
                || facet.resource_kind != Some(ToolResourceKind::Remote))
        {
            return Err(GrantError::MissingResource(index));
        }
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
        // These closed capabilities are consumed only by the prepared Git
        // stages. They do not admit generic Remote or Path subtree grants.
        let bound_git_command = request.facets().iter().any(|facet| {
            facet.nature == ToolNature::Execute
                && facet.resource_kind == Some(ToolResourceKind::Command)
                && facet
                    .resource
                    .as_deref()
                    .is_some_and(|path| Path::new(path).is_absolute())
        });
        let pull_command = facet.nature == ToolNature::Execute
            && resource_kind == ToolResourceKind::Remote
            && facet.resource.as_deref() == Some(crate::native_pull_command::CAPABILITY)
            && request.tool_name() == "git_pull"
            && bound_git_command
            && request.facets().iter().any(|root| {
                root.nature == ToolNature::Read
                    && root.resource_kind == Some(ToolResourceKind::Path)
                    && root
                        .resource
                        .as_deref()
                        .is_some_and(|path| Path::new(path).is_absolute())
            });
        if facet.resource.as_deref().map(str::trim) == Some(crate::native_pull_command::CAPABILITY)
            && !pull_command
        {
            return Err(GrantError::MissingResource(index));
        }
        let bound_git_read_path = request.facets().iter().any(|facet| {
            facet.nature == ToolNature::Read
                && facet.resource_kind == Some(ToolResourceKind::Path)
                && facet.resource.as_deref().is_some_and(|path| {
                    let path = Path::new(path);
                    path.is_absolute()
                        && matches!(
                            path.file_name().and_then(std::ffi::OsStr::to_str),
                            Some("git" | "git.exe")
                        )
                })
        });
        let git_config_discovery = facet.nature == ToolNature::Read
            && resource_kind == ToolResourceKind::Remote
            && facet.resource.as_deref() == Some("git:configuration-chain")
            && matches!(
                request.tool_name(),
                "git_add" | "git_commit" | "git_checkout" | "git_push" | "git_pull"
            )
            && bound_git_command;
        let auxiliary_resolution = facet.nature == ToolNature::Read
            && resource_kind == ToolResourceKind::Remote
            && facet.resource.as_deref() == Some("git:auxiliary-path-resolution")
            && matches!(
                request.tool_name(),
                "git_add" | "git_commit" | "git_checkout" | "git_push" | "git_pull"
            )
            && (bound_git_command || bound_git_read_path)
            && request.facets().iter().any(|source| {
                source.nature == ToolNature::Read
                    && source.resource_kind == Some(ToolResourceKind::Path)
                    && source
                        .resource
                        .as_deref()
                        .is_some_and(|path| Path::new(path).is_absolute())
            });
        // Reserve this closed capability name even where generic Remote pairs
        // (notably Network) would otherwise compile. It is metadata Read only.
        if facet.resource.as_deref().map(str::trim) == Some("git:auxiliary-path-resolution")
            && !auxiliary_resolution
        {
            return Err(GrantError::MissingResource(index));
        }
        let receiver_reference_tree = facet.nature == ToolNature::Read
            && resource_kind == ToolResourceKind::Remote
            && facet.resource.as_deref() == Some("git:receiver-reference-tree")
            && request.tool_name() == "git_push"
            && request.facets().iter().any(|facet| {
                facet.nature == ToolNature::Read
                    && facet.resource_kind == Some(ToolResourceKind::Path)
                    && facet.resource.as_deref().is_some_and(|path| {
                        Path::new(path).is_absolute()
                            && Path::new(path).file_name() == Some(std::ffi::OsStr::new("refs"))
                    })
            });
        let receiver_metadata = matches!(facet.nature, ToolNature::Read | ToolNature::Write)
            && resource_kind == ToolResourceKind::Remote
            && facet.resource.as_deref() == Some("git:receiver-metadata")
            && request.tool_name() == "git_push"
            && bound_git_command
            && request.facets().iter().any(|root| {
                root.nature == facet.nature
                    && root.resource_kind == Some(ToolResourceKind::Path)
                    && root
                        .resource
                        .as_deref()
                        .is_some_and(|path| Path::new(path).is_absolute())
            });
        // The prepared consumer enforces root identity, fixed inspection argv
        // and cumulative discovery budgets. This is never generic subtree Read.
        let pull_discovery = facet.nature == ToolNature::Read
            && resource_kind == ToolResourceKind::Remote
            && facet.resource.as_deref() == Some("git:pull-bounded-discovery")
            && request.tool_name() == "git_pull"
            && bound_git_command
            && request.facets().iter().any(|root| {
                root.nature == ToolNature::Read
                    && root.resource_kind == Some(ToolResourceKind::Path)
                    && root
                        .resource
                        .as_deref()
                        .is_some_and(|path| Path::new(path).is_absolute())
            });
        // Reserve the name for every nature/kind, including otherwise valid
        // Network/Remote pairs. A malformed capability must not fall through.
        if facet.resource.as_deref().map(str::trim) == Some("git:pull-bounded-discovery")
            && !pull_discovery
        {
            return Err(GrantError::MissingResource(index));
        }
        let pull_metadata = matches!(facet.nature, ToolNature::Read | ToolNature::Write)
            && resource_kind == ToolResourceKind::Remote
            && facet.resource.as_deref() == Some("git:pull-repository-metadata")
            && request.tool_name() == "git_pull"
            && bound_git_command
            && request.facets().iter().any(|root| {
                root.nature == facet.nature
                    && root.resource_kind == Some(ToolResourceKind::Path)
                    && root
                        .resource
                        .as_deref()
                        .is_some_and(|path| Path::new(path).is_absolute())
            });
        // Dedicated integration authority; generic Path grants remain exact.
        // Runtime consumption additionally binds the actual worktree and stage.
        let pull_integration_worktree =
            matches!(facet.nature, ToolNature::Read | ToolNature::Write)
                && resource_kind == ToolResourceKind::Remote
                && facet.resource.as_deref() == Some("git:pull-integration-worktree")
                && request.tool_name() == "git_pull"
                && bound_git_command
                && request.facets().iter().any(|root| {
                    root.nature == facet.nature
                        && root.resource_kind == Some(ToolResourceKind::Path)
                        && root
                            .resource
                            .as_deref()
                            .is_some_and(|path| Path::new(path).is_absolute())
                });
        let pull_resolution = facet.nature == ToolNature::Read
            && resource_kind == ToolResourceKind::Remote
            && facet.resource.as_deref() == Some("git:pull-worktree-resolution")
            && request.tool_name() == "git_pull"
            && bound_git_command
            && request.facets().iter().any(|scope| {
                scope.nature == ToolNature::Read
                    && scope.resource_kind == Some(ToolResourceKind::Remote)
                    && scope.resource.as_deref() == Some("git:pull-repository-metadata")
            })
            && request.facets().iter().any(|root| {
                root.nature == ToolNature::Read
                    && root.resource_kind == Some(ToolResourceKind::Path)
                    && root
                        .resource
                        .as_deref()
                        .is_some_and(|path| Path::new(path).is_absolute())
            });
        if !supported_resource_pair(facet.nature, resource_kind)
            && !pull_command
            && !git_config_discovery
            && !auxiliary_resolution
            && !receiver_reference_tree
            && !receiver_metadata
            && !pull_metadata
            && !pull_discovery
            && !pull_resolution
            && !pull_integration_worktree
        {
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
    if let Some(manifest) = request.network_manifest() {
        hasher.update([1]);
        hasher.update(manifest.invocation_id().as_bytes());
        let scope = manifest.scope();
        hasher.update(scope.configuration_id().as_bytes());
        hasher.update(scope.configuration_revision().to_le_bytes());
        hash_len(&mut hasher, scope.endpoints().len());
        for endpoint in scope.endpoints().iter().chain(scope.proxy()) {
            hash_text(&mut hasher, endpoint.scheme());
            hash_text(&mut hasher, endpoint.host());
            hasher.update(endpoint.port().to_le_bytes());
            hash_text(&mut hasher, endpoint.method());
            hash_text(&mut hasher, endpoint.path());
        }
        hasher.update([u8::from(scope.proxy().is_some()), scope.redirect_limit()]);
        hash_len(&mut hasher, scope.private_addresses().len());
        for address in scope.private_addresses() {
            hash_text(&mut hasher, &address.to_string());
        }
    }
    RequestFingerprint(hasher.finalize().into())
}

fn validate_network_profile(request: &PermissionRequest<'_>) -> Result<(), GrantError> {
    let Some(manifest) = request.network_manifest() else {
        return Ok(());
    };
    if *request.provenance() != ToolProvenance::Native {
        return Err(GrantError::AdmissionInvalidated);
    }
    let mut hosts: Vec<_> = manifest
        .scope()
        .endpoints()
        .iter()
        .chain(manifest.scope().proxy())
        .map(|e| e.host())
        .collect();
    hosts.sort_unstable();
    hosts.dedup();
    let mut declared = Vec::new();
    for facet in request.facets() {
        if facet.nature != ToolNature::Network
            || facet.resource_kind != Some(ToolResourceKind::Domain)
        {
            return Err(GrantError::AdmissionInvalidated);
        }
        let host = facet
            .resource
            .as_deref()
            .ok_or(GrantError::AdmissionInvalidated)?;
        if !hosts.contains(&host) {
            return Err(GrantError::AdmissionInvalidated);
        }
        declared.push(host);
    }
    declared.sort_unstable();
    declared.dedup();
    if declared != hosts {
        return Err(GrantError::AdmissionInvalidated);
    }
    Ok(())
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

#[cfg(test)]
#[path = "grant_discovery_tests.rs"]
mod discovery_tests;

#[cfg(test)]
#[path = "grant_pull_integration_tests.rs"]
mod pull_integration_tests;

#[cfg(test)]
#[path = "grant_pull_discovery_tests.rs"]
mod pull_discovery_tests;
