use std::io;
use std::path::{Component, Path, PathBuf};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::ToolNature;

/// Identifies how a permission resource string should be interpreted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ToolResourceKind {
    /// File or directory path resource.
    Path,
    /// URL host or domain resource.
    Domain,
    /// External command or executable resource.
    Command,
    /// Named remote resource, such as a Git remote.
    Remote,
}

/// Lifetime of a concrete tool-execution authorization.
///
/// This describes the approval scope that produced an authorization. It does
/// not itself persist permission policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolAuthorizationScope {
    /// Authorization applies only to the current invocation.
    Once,
    /// Authorization was produced from a reusable permission rule.
    Persisted,
}

/// A concrete, path-bound authorization for one tool operation.
///
/// Permission-aware composition roots create this value only after resolving
/// `Allow`/`Ask`/`Deny`. File tools compare the normalized path and operation
/// before allowing an invocation to leave the workspace boundary. Calling a
/// file tool through [`AgentTool::execute`] does not provide this capability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolExecutionAuthorization {
    tool_name: String,
    nature: ToolNature,
    resource_kind: ToolResourceKind,
    normalized_resource: PathBuf,
    scope: ToolAuthorizationScope,
}

impl ToolExecutionAuthorization {
    /// Creates a path-bound authorization using the workspace as the base for
    /// relative resources.
    ///
    /// Existing paths and their nearest existing ancestor are canonicalized,
    /// so a later symlink change cannot silently reuse an authorization for a
    /// different target.
    pub fn for_path(
        tool_name: impl Into<String>,
        nature: ToolNature,
        workspace_root: &Path,
        resource: &str,
        scope: ToolAuthorizationScope,
    ) -> io::Result<Self> {
        Ok(Self {
            tool_name: tool_name.into(),
            nature,
            resource_kind: ToolResourceKind::Path,
            normalized_resource: normalize_authorized_path(workspace_root, resource)?,
            scope,
        })
    }

    /// Returns whether this authorization exactly covers a requested path and
    /// operation.
    pub fn authorizes_path(
        &self,
        tool_name: &str,
        nature: ToolNature,
        workspace_root: &Path,
        resource: &str,
    ) -> bool {
        self.tool_name == tool_name
            && self.nature == nature
            && self.resource_kind == ToolResourceKind::Path
            && normalize_authorized_path(workspace_root, resource)
                .is_ok_and(|path| path == self.normalized_resource)
    }

    /// Returns the normalized path carried by this authorization.
    #[must_use]
    pub fn normalized_path(&self) -> &Path {
        &self.normalized_resource
    }

    /// Returns the approval scope that produced this authorization.
    #[must_use]
    pub fn scope(&self) -> ToolAuthorizationScope {
        self.scope
    }
}

fn normalize_authorized_path(workspace_root: &Path, resource: &str) -> io::Result<PathBuf> {
    let requested = Path::new(resource);
    let candidate = if requested.is_absolute() {
        requested.to_path_buf()
    } else {
        workspace_root.join(requested)
    };

    let mut lexical = PathBuf::new();
    for component in candidate.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !lexical.pop() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "path traversal escapes filesystem root",
                    ));
                }
            }
            other => lexical.push(other.as_os_str()),
        }
    }

    let mut existing = lexical.as_path();
    let mut suffix = Vec::new();
    while !existing.exists() {
        let Some(name) = existing.file_name() else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "path has no existing ancestor",
            ));
        };
        suffix.push(name.to_os_string());
        existing = existing.parent().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "path has no existing ancestor")
        })?;
    }

    let mut normalized = existing.canonicalize()?;
    for component in suffix.into_iter().rev() {
        normalized.push(component);
    }
    Ok(normalized)
}

/// One permission facet touched by a tool invocation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ToolPermissionFacet {
    /// Risk nature for this facet.
    pub nature: ToolNature,
    /// Optional concrete resource touched by this facet.
    #[serde(default)]
    pub resource: Option<String>,
    /// Optional interpretation hint for [`resource`](Self::resource).
    #[serde(default)]
    pub resource_kind: Option<ToolResourceKind>,
    /// Optional human-readable detail used in approval or diagnostics.
    #[serde(default)]
    pub description: Option<String>,
}

impl ToolPermissionFacet {
    /// Creates a facet with no concrete resource.
    pub fn new(nature: ToolNature) -> Self {
        Self {
            nature,
            resource: None,
            resource_kind: None,
            description: None,
        }
    }

    /// Creates a facet with a concrete resource.
    pub fn with_resource(
        nature: ToolNature,
        resource: impl Into<String>,
        resource_kind: ToolResourceKind,
    ) -> Self {
        Self {
            nature,
            resource: Some(resource.into()),
            resource_kind: Some(resource_kind),
            description: None,
        }
    }

    /// Adds display-oriented detail to this facet.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}
