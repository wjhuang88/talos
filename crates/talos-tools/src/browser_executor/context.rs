//! Trusted host reference state. Model input never installs or changes this registry.

use std::collections::{HashMap, HashSet};

use super::{
    BrowserArtifactStore, BrowserCommand, BrowserElementRef, BrowserFrameRef,
    BrowserInvocationTicket, BrowserLifecycle, BrowserRequest, BrowserSnapshotRef, BrowserTabRef,
    BrowserTicketError,
};

/// Canonical exact HTTP origin without path, userinfo, query or fragment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserOrigin {
    scheme: String,
    host: String,
    port: u16,
}

impl BrowserOrigin {
    /// Converts host-verified effective URL evidence into an exact origin.
    /// Inherited/opaque document origins must be resolved by the trusted host first.
    pub fn from_effective_url(value: &str) -> Result<Self, BrowserContextError> {
        let url = reqwest::Url::parse(value).map_err(|_| BrowserContextError::OriginUnavailable)?;
        if !matches!(url.scheme(), "http" | "https")
            || !url.username().is_empty()
            || url.password().is_some()
        {
            return Err(BrowserContextError::OriginUnavailable);
        }
        Ok(Self {
            scheme: url.scheme().to_owned(),
            host: url
                .host_str()
                .ok_or(BrowserContextError::OriginUnavailable)?
                .to_owned(),
            port: url
                .port_or_known_default()
                .ok_or(BrowserContextError::OriginUnavailable)?,
        })
    }
}

/// Stable host-state admission errors without supplied input or driver diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum BrowserContextError {
    /// The local invocation reservation is no longer usable.
    #[error(transparent)]
    Ticket(#[from] BrowserTicketError),
    /// The reference does not belong to the current tab/frame generation.
    #[error("invalid browser reference")]
    InvalidReference,
    /// Discovery has not confirmed that the frame is ready.
    #[error("browser frame is pending")]
    Pending,
    /// Observation and interaction require a trusted effective HTTP origin.
    #[error("browser frame origin unavailable")]
    OriginUnavailable,
    /// Host lifecycle synchronization was lost.
    #[error("browser context unavailable")]
    Unavailable,
    /// The bounded registry capacity or depth was exceeded.
    #[error("browser context resource limit")]
    ResourceLimit,
}

struct Frame {
    parent: Option<BrowserFrameRef>,
    origin: Option<BrowserOrigin>,
    opaque: bool,
    ready: bool,
    depth: usize,
    snapshot: Option<Snapshot>,
}

struct Snapshot {
    reference: BrowserSnapshotRef,
    elements: HashSet<BrowserElementRef>,
}

/// Newly minted snapshot identities for a trusted host's bounded, frame-local capture.
/// This carries no page content and grants no permission to inspect another frame.
#[derive(Debug)]
pub struct BrowserSnapshotIdentity {
    /// Snapshot generation identifier.
    pub snapshot: BrowserSnapshotRef,
    /// Element identifiers in the supplied capture order.
    pub elements: Vec<BrowserElementRef>,
}

#[derive(Default)]
struct Tab {
    frames: HashMap<BrowserFrameRef, Frame>,
    top: Option<BrowserFrameRef>,
}

/// Trusted host's bounded tab/frame identity registry.
///
/// References are random and scoped to this instance. Only host lifecycle/discovery code should
/// call mutation methods. This registry does not grant permission or execute browser operations.
pub struct BrowserContext {
    tabs: HashMap<BrowserTabRef, Tab>,
    available: bool,
    next_reference: u64,
    tickets: BrowserLifecycle,
    artifacts: BrowserArtifactStore,
    artifact_owner: Option<(super::BrowserPermissionResource, u64)>,
}

/// A privately admitted observation target and its one-shot reservation, not authorization.
#[derive(Debug)]
pub struct PreparedBrowserObservation {
    ticket: BrowserInvocationTicket,
    scope: BrowserFrameScope,
}

impl PreparedBrowserObservation {
    pub(crate) fn binding(&self) -> super::ticket::InvocationBinding {
        self.ticket.binding()
    }
    /// Exact resolved scope for the eventual browser-specific permission evaluator.
    pub fn scope(&self) -> &BrowserFrameScope {
        &self.scope
    }
    /// Validated immutable request bound to this reservation.
    pub fn request(&self) -> &BrowserRequest {
        self.ticket.request()
    }
}

fn mint_reference(counter: &mut u64) -> Result<String, BrowserContextError> {
    let sequence = counter
        .checked_add(1)
        .ok_or(BrowserContextError::ResourceLimit)?;
    let random = std::panic::catch_unwind(uuid::Uuid::new_v4)
        .map_err(|_| BrowserContextError::Unavailable)?;
    *counter = sequence;
    Ok(format!("{}_{sequence}", random.simple()))
}

/// Exact observation target resolved from trusted state, not a permission grant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserFrameScope {
    /// The exact tab identity.
    pub tab: BrowserTabRef,
    /// The exact frame identity.
    pub frame: BrowserFrameRef,
    /// The current top-level origin; does not confer child authority.
    pub top_origin: BrowserOrigin,
    /// The selected frame's independently authorized origin.
    pub frame_origin: BrowserOrigin,
}

impl Default for BrowserContext {
    fn default() -> Self {
        Self::new()
    }
}

impl BrowserContext {
    /// Creates an empty domain after trusted host synchronization.
    pub fn new() -> Self {
        Self {
            tabs: HashMap::new(),
            available: true,
            next_reference: 0,
            tickets: BrowserLifecycle::new(),
            artifacts: BrowserArtifactStore::default(),
            artifact_owner: None,
        }
    }

    /// Returns the transient screenshot capability store owned by this host context.
    pub(crate) fn artifact_store(&self) -> &BrowserArtifactStore {
        &self.artifacts
    }

    /// Binds the artifact publisher to the currently dispatched invocation.
    pub(crate) fn begin_artifact_scope(
        &mut self,
        resource: super::BrowserPermissionResource,
        generation: u64,
    ) {
        self.artifact_owner = Some((resource, generation));
    }

    /// Publishes screenshot bytes only for the currently dispatched invocation.
    pub fn publish_screenshot(
        &self,
        bytes: Vec<u8>,
        width: u32,
        height: u32,
    ) -> Result<super::BrowserArtifactRef, super::BrowserOutputError> {
        let (owner, generation) = self
            .artifact_owner
            .as_ref()
            .ok_or(super::BrowserOutputError)?;
        if owner.operation != super::BrowserOperation::Screenshot {
            return Err(super::BrowserOutputError);
        }
        self.artifacts
            .insert(owner.clone(), *generation, bytes, width, height)
    }

    pub(crate) fn end_artifact_scope(&mut self, completed: bool) {
        if let Some((owner, _)) = self.artifact_owner.take()
            && !completed
        {
            self.artifacts.discard_owner(&owner);
        }
    }

    fn ensure_available(&self) -> Result<(), BrowserContextError> {
        if self.available {
            Ok(())
        } else {
            Err(BrowserContextError::Unavailable)
        }
    }

    /// Records one host-observed tab, up to 64 per session.
    pub fn insert_tab(&mut self) -> Result<BrowserTabRef, BrowserContextError> {
        self.ensure_available()?;
        if self.tabs.len() >= 64 {
            return Err(BrowserContextError::ResourceLimit);
        }
        let reference = BrowserTabRef::try_new(mint_reference(&mut self.next_reference)?)
            .map_err(|_| BrowserContextError::Unavailable)?;
        if self.tabs.contains_key(&reference) {
            return Err(BrowserContextError::Unavailable);
        }
        self.tabs.insert(reference.clone(), Tab::default());
        Ok(reference)
    }

    /// Records a fresh host-observed frame. A tab has one root, at most 128 frames and depth 16.
    /// Pending frames must be invalidated and freshly discovered to become ready.
    pub fn insert_frame(
        &mut self,
        tab: &BrowserTabRef,
        parent: Option<&BrowserFrameRef>,
        origin: Option<BrowserOrigin>,
        ready: bool,
    ) -> Result<BrowserFrameRef, BrowserContextError> {
        self.ensure_available()?;
        let state = self
            .tabs
            .get_mut(tab)
            .ok_or(BrowserContextError::InvalidReference)?;
        if state.frames.len() >= 128 {
            return Err(BrowserContextError::ResourceLimit);
        }
        let depth = match parent {
            Some(parent) => {
                state
                    .frames
                    .get(parent)
                    .ok_or(BrowserContextError::InvalidReference)?
                    .depth
                    + 1
            }
            None if state.top.is_none() => 1,
            None => return Err(BrowserContextError::InvalidReference),
        };
        if depth > 16 {
            return Err(BrowserContextError::ResourceLimit);
        }
        let reference = BrowserFrameRef::try_new(mint_reference(&mut self.next_reference)?)
            .map_err(|_| BrowserContextError::Unavailable)?;
        if state.frames.contains_key(&reference) {
            return Err(BrowserContextError::Unavailable);
        }
        state.frames.insert(
            reference.clone(),
            Frame {
                parent: parent.cloned(),
                origin,
                opaque: false,
                ready,
                depth,
                snapshot: None,
            },
        );
        if parent.is_none() {
            state.top = Some(reference.clone());
        }
        Ok(reference)
    }

    /// Resolves an explicit tab/frame tuple for observation or interaction.
    /// No parent-origin inheritance or matching by name is performed.
    pub fn resolve(
        &self,
        tab: &BrowserTabRef,
        frame: &BrowserFrameRef,
    ) -> Result<BrowserFrameScope, BrowserContextError> {
        self.ensure_available()?;
        let state = self
            .tabs
            .get(tab)
            .ok_or(BrowserContextError::InvalidReference)?;
        let selected = state
            .frames
            .get(frame)
            .ok_or(BrowserContextError::InvalidReference)?;
        if !selected.ready {
            return Err(BrowserContextError::Pending);
        }
        let mut parent = selected.parent.as_ref();
        while let Some(reference) = parent {
            let ancestor = state
                .frames
                .get(reference)
                .ok_or(BrowserContextError::InvalidReference)?;
            if !ancestor.ready {
                return Err(BrowserContextError::Pending);
            }
            parent = ancestor.parent.as_ref();
        }
        let top = state
            .top
            .as_ref()
            .and_then(|top| state.frames.get(top))
            .ok_or(BrowserContextError::InvalidReference)?;
        let top_origin = top
            .origin
            .clone()
            .ok_or(BrowserContextError::OriginUnavailable)?;
        let frame_origin = selected
            .origin
            .clone()
            .ok_or(BrowserContextError::OriginUnavailable)?;
        Ok(BrowserFrameScope {
            tab: tab.clone(),
            frame: frame.clone(),
            top_origin,
            frame_origin,
        })
    }

    /// Records a host-verified opaque document, distinguished from unknown origin evidence.
    /// Opaque frames permit inventory/lifecycle only, never observation or interaction.
    pub fn insert_opaque_frame(
        &mut self,
        tab: &BrowserTabRef,
        parent: Option<&BrowserFrameRef>,
        ready: bool,
    ) -> Result<BrowserFrameRef, BrowserContextError> {
        let reference = self.insert_frame(tab, parent, None, ready)?;
        self.tabs
            .get_mut(tab)
            .and_then(|tab| tab.frames.get_mut(&reference))
            .ok_or(BrowserContextError::InvalidReference)?
            .opaque = true;
        Ok(reference)
    }

    fn top_origin(
        &self,
        tab: &BrowserTabRef,
    ) -> Result<super::BrowserDocumentOrigin, BrowserContextError> {
        let state = self
            .tabs
            .get(tab)
            .ok_or(BrowserContextError::InvalidReference)?;
        let reference = state
            .top
            .as_ref()
            .ok_or(BrowserContextError::OriginUnavailable)?;
        let frame = state
            .frames
            .get(reference)
            .ok_or(BrowserContextError::InvalidReference)?;
        match &frame.origin {
            Some(origin) => Ok(super::BrowserDocumentOrigin::Http(origin.clone())),
            None if frame.opaque => Ok(super::BrowserDocumentOrigin::OpaqueDocument(
                reference.clone(),
            )),
            None => Err(BrowserContextError::OriginUnavailable),
        }
    }

    fn resolve_target(
        &self,
        request: &BrowserRequest,
    ) -> Result<super::BrowserPermissionTarget, BrowserContextError> {
        use super::BrowserPermissionTarget as Target;
        use BrowserCommand::*;
        self.ensure_available()?;
        let operation = request.operation();
        Ok(match request.command() {
            TabNew {} => Target::SessionCreate,
            TabList {} => Target::SessionInventory,
            WindowClose {} => Target::SessionClose,
            WaitMilliseconds { milliseconds } => Target::SessionWait {
                milliseconds: *milliseconds,
            },
            TabClose { tab_ref } | TabSwitch { tab_ref } => {
                if !self.tabs.contains_key(tab_ref) {
                    return Err(BrowserContextError::InvalidReference);
                }
                Target::TabLifecycle {
                    tab: tab_ref.clone(),
                    operation,
                }
            }
            FrameTree { tab_ref } => Target::FrameInventory {
                tab: tab_ref.clone(),
                top_origin: self.top_origin(tab_ref)?,
            },
            Open { tab_ref, url, .. } => Target::Navigate {
                tab: tab_ref.clone(),
                current_origin: self.top_origin(tab_ref)?,
                destination: BrowserOrigin::from_effective_url(url)?,
            },
            Read { tab_ref, frame_ref }
            | Snapshot { tab_ref, frame_ref }
            | CurrentUrl { tab_ref, frame_ref }
            | Screenshot { tab_ref, frame_ref } => Target::Observe {
                scope: self.resolve(tab_ref, frame_ref)?,
                operation,
                element: None,
            },
            Press {
                tab_ref, frame_ref, ..
            }
            | Scroll {
                tab_ref, frame_ref, ..
            } => Target::Interact {
                scope: self.resolve(tab_ref, frame_ref)?,
                operation,
                element: None,
            },
            Click {
                tab_ref,
                frame_ref,
                snapshot_ref,
                element_ref,
            }
            | Hover {
                tab_ref,
                frame_ref,
                snapshot_ref,
                element_ref,
            }
            | Check {
                tab_ref,
                frame_ref,
                snapshot_ref,
                element_ref,
            }
            | Uncheck {
                tab_ref,
                frame_ref,
                snapshot_ref,
                element_ref,
            }
            | Fill {
                tab_ref,
                frame_ref,
                snapshot_ref,
                element_ref,
                ..
            }
            | Select {
                tab_ref,
                frame_ref,
                snapshot_ref,
                element_ref,
                ..
            }
            | WaitForElement {
                tab_ref,
                frame_ref,
                snapshot_ref,
                element_ref,
            } => {
                let scope = self.resolve_element(tab_ref, frame_ref, snapshot_ref, element_ref)?;
                let element = Some((snapshot_ref.clone(), element_ref.clone()));
                if operation == super::BrowserOperation::WaitForElement {
                    Target::Observe {
                        scope,
                        operation,
                        element,
                    }
                } else {
                    Target::Interact {
                        scope,
                        operation,
                        element,
                    }
                }
            }
        })
    }

    /// Rechecks trusted document identity before releasing an observation.
    pub(crate) fn validate_observation_release(
        &self,
        request: &BrowserRequest,
        target: &super::BrowserPermissionTarget,
    ) -> Result<(), BrowserContextError> {
        self.ensure_available()?;
        if self.resolve_target(request)? != *target {
            return Err(BrowserContextError::InvalidReference);
        }
        Ok(())
    }

    /// Resolves every v2 operation into its exact resource before permission evaluation.
    pub fn prepare_invocation(
        &mut self,
        request: BrowserRequest,
    ) -> Result<super::PreparedBrowserInvocation, BrowserContextError> {
        let target = self.resolve_target(&request)?;
        let ticket = self.tickets.prepare(request)?;
        Ok(super::PreparedBrowserInvocation { ticket, target })
    }

    /// Verifies output identities against the selected document, not merely wire shape.
    pub(crate) fn validate_output_identity(
        &self,
        target: &super::BrowserPermissionTarget,
        output: &super::BrowserOutput,
    ) -> Result<(), BrowserContextError> {
        use super::{BrowserOutput, BrowserPermissionTarget};
        match output {
            BrowserOutput::Snapshot {
                snapshot_ref,
                nodes,
                ..
            } => {
                let BrowserPermissionTarget::Observe { scope, .. } = target else {
                    return Err(BrowserContextError::InvalidReference);
                };
                self.resolve(&scope.tab, &scope.frame)?;
                let snapshot = self
                    .tabs
                    .get(&scope.tab)
                    .and_then(|tab| tab.frames.get(&scope.frame))
                    .and_then(|frame| frame.snapshot.as_ref())
                    .ok_or(BrowserContextError::InvalidReference)?;
                if snapshot.reference != *snapshot_ref
                    || nodes
                        .iter()
                        .any(|node| !snapshot.elements.contains(&node.element_ref))
                {
                    return Err(BrowserContextError::InvalidReference);
                }
            }
            BrowserOutput::PageUrl { origin, .. } => {
                let BrowserPermissionTarget::Observe { scope, .. } = target else {
                    return Err(BrowserContextError::InvalidReference);
                };
                if BrowserOrigin::from_effective_url(origin)? != scope.frame_origin {
                    return Err(BrowserContextError::InvalidReference);
                }
            }
            BrowserOutput::TabCreated { tab_ref } => {
                self.ensure_available()?;
                if !self.tabs.contains_key(tab_ref) {
                    return Err(BrowserContextError::InvalidReference);
                }
            }
            BrowserOutput::Tabs { entries, .. } => {
                self.ensure_available()?;
                if !matches!(target, BrowserPermissionTarget::SessionInventory) {
                    return Err(BrowserContextError::InvalidReference);
                }
                for entry in entries {
                    let expected = self.top_origin(&entry.tab_ref)?;
                    let matches = match expected {
                        super::BrowserDocumentOrigin::Http(origin) => {
                            BrowserOrigin::from_effective_url(&entry.origin)? == origin
                        }
                        super::BrowserDocumentOrigin::OpaqueDocument(_) => entry.origin == "opaque",
                    };
                    if !matches {
                        return Err(BrowserContextError::InvalidReference);
                    }
                }
            }
            BrowserOutput::Frames { entries, .. } => {
                self.ensure_available()?;
                let BrowserPermissionTarget::FrameInventory { tab, top_origin } = target else {
                    return Err(BrowserContextError::InvalidReference);
                };
                if self.top_origin(tab)? != *top_origin {
                    return Err(BrowserContextError::InvalidReference);
                }
                let state = self
                    .tabs
                    .get(tab)
                    .ok_or(BrowserContextError::InvalidReference)?;
                for entry in entries {
                    let frame = state
                        .frames
                        .get(&entry.frame_ref)
                        .ok_or(BrowserContextError::InvalidReference)?;
                    let origin_matches = match &frame.origin {
                        Some(origin) => {
                            BrowserOrigin::from_effective_url(&entry.origin)? == *origin
                        }
                        None => frame.opaque && entry.origin == "opaque",
                    };
                    if !origin_matches
                        || frame.parent != entry.parent_frame_ref
                        || frame.ready != (entry.readiness == super::BrowserReadiness::Ready)
                    {
                        return Err(BrowserContextError::InvalidReference);
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Revalidates both the one-shot ticket and every operation-specific reference.
    pub fn validate_invocation(
        &self,
        prepared: &super::PreparedBrowserInvocation,
    ) -> Result<(), BrowserContextError> {
        self.ensure_available()?;
        self.tickets.validate(&prepared.ticket)?;
        if self.resolve_target(prepared.request())? != prepared.target {
            return Err(BrowserContextError::InvalidReference);
        }
        Ok(())
    }

    /// Derives an exact permission challenge only while the invocation is still valid.
    pub fn invocation_permission_resource(
        &self,
        prepared: &super::PreparedBrowserInvocation,
    ) -> Result<super::BrowserPermissionResource, BrowserContextError> {
        self.validate_invocation(prepared)?;
        super::BrowserPermissionResource::from_invocation(prepared)
            .map_err(|_| BrowserContextError::Unavailable)
    }

    /// Retires a canceled or denied invocation without executing anything.
    pub fn discard_invocation(
        &mut self,
        prepared: super::PreparedBrowserInvocation,
    ) -> Result<(), BrowserContextError> {
        self.tickets.discard(prepared.ticket).map_err(Into::into)
    }

    pub(crate) fn consume_invocation(
        &mut self,
        prepared: super::PreparedBrowserInvocation,
    ) -> Result<BrowserRequest, BrowserContextError> {
        self.tickets.consume(prepared.ticket).map_err(Into::into)
    }

    /// Replaces one frame's snapshot and revokes all elements of the previous snapshot.
    /// The host must supply only frame-local, already bounded capture nodes.
    pub fn replace_snapshot(
        &mut self,
        tab: &BrowserTabRef,
        frame: &BrowserFrameRef,
        element_count: usize,
    ) -> Result<BrowserSnapshotIdentity, BrowserContextError> {
        self.resolve(tab, frame)?;
        if element_count > 512 {
            return Err(BrowserContextError::ResourceLimit);
        }
        let snapshot = BrowserSnapshotRef::try_new(mint_reference(&mut self.next_reference)?)
            .map_err(|_| BrowserContextError::Unavailable)?;
        let mut elements = Vec::with_capacity(element_count);
        for _ in 0..element_count {
            elements.push(
                BrowserElementRef::try_new(mint_reference(&mut self.next_reference)?)
                    .map_err(|_| BrowserContextError::Unavailable)?,
            );
        }
        let selected = self
            .tabs
            .get_mut(tab)
            .and_then(|tab| tab.frames.get_mut(frame))
            .ok_or(BrowserContextError::InvalidReference)?;
        selected.snapshot = Some(Snapshot {
            reference: snapshot.clone(),
            elements: elements.iter().cloned().collect(),
        });
        self.tickets.change_document();
        Ok(BrowserSnapshotIdentity { snapshot, elements })
    }

    /// Resolves an element only in its exact current tab/frame/snapshot tuple.
    pub fn resolve_element(
        &self,
        tab: &BrowserTabRef,
        frame: &BrowserFrameRef,
        snapshot: &BrowserSnapshotRef,
        element: &BrowserElementRef,
    ) -> Result<BrowserFrameScope, BrowserContextError> {
        let scope = self.resolve(tab, frame)?;
        let current = self
            .tabs
            .get(tab)
            .and_then(|tab| tab.frames.get(frame))
            .and_then(|frame| frame.snapshot.as_ref())
            .ok_or(BrowserContextError::InvalidReference)?;
        if &current.reference != snapshot || !current.elements.contains(element) {
            return Err(BrowserContextError::InvalidReference);
        }
        Ok(scope)
    }

    /// Conservatively revokes previously disclosed frame/element refs on a tab switch.
    /// Returning to a tab requires fresh discovery; no old interaction identity is revived.
    pub fn switch_tab(
        &mut self,
        previous: &BrowserTabRef,
        selected: &BrowserTabRef,
    ) -> Result<(), BrowserContextError> {
        self.ensure_available()?;
        if !self.tabs.contains_key(selected) {
            return Err(BrowserContextError::InvalidReference);
        }
        let previous = self
            .tabs
            .get_mut(previous)
            .ok_or(BrowserContextError::InvalidReference)?;
        previous.frames.clear();
        previous.top = None;
        self.tickets.change_document();
        Ok(())
    }

    /// Revokes a navigated/detached frame and all descendants without relocation or tombstones.
    pub fn invalidate_frame(
        &mut self,
        tab: &BrowserTabRef,
        frame: &BrowserFrameRef,
    ) -> Result<(), BrowserContextError> {
        self.ensure_available()?;
        let state = self
            .tabs
            .get_mut(tab)
            .ok_or(BrowserContextError::InvalidReference)?;
        if !state.frames.contains_key(frame) {
            return Err(BrowserContextError::InvalidReference);
        }
        let mut removed = vec![frame.clone()];
        let mut index = 0;
        while index < removed.len() {
            let parent = &removed[index];
            let children: Vec<_> = state
                .frames
                .iter()
                .filter(|(_, value)| value.parent.as_ref() == Some(parent))
                .map(|(key, _)| key.clone())
                .collect();
            removed.extend(children);
            index += 1;
        }
        for reference in removed {
            state.frames.remove(&reference);
        }
        if state.top.as_ref() == Some(frame) {
            state.top = None;
        }
        self.tickets.change_document();
        Ok(())
    }

    /// Revokes a closed tab and all its references.
    pub fn close_tab(&mut self, tab: &BrowserTabRef) -> Result<(), BrowserContextError> {
        self.ensure_available()?;
        self.tabs
            .remove(tab)
            .map(|_| ())
            .ok_or(BrowserContextError::InvalidReference)?;
        self.tickets.change_document();
        Ok(())
    }

    /// Fails closed after lifecycle stream loss; trusted resynchronization needs a new domain.
    pub fn lose_synchronization(&mut self) {
        self.artifacts = BrowserArtifactStore::default();
        self.available = false;
        self.tabs.clear();
        self.tickets.lose_synchronization();
    }

    /// Installs a newly synchronized host session and revokes all previous session identities.
    /// Only trusted host lifecycle code may call this after establishing fresh state.
    /// No tab, frame, snapshot, pending approval or old ticket survives replacement.
    pub fn replace_session(&mut self) {
        self.artifacts = BrowserArtifactStore::default();
        self.tabs.clear();
        // A new allocation prevents old outstanding tickets from matching even if their
        // numeric epochs/nonces happen to equal those issued by the new session.
        self.tickets = BrowserLifecycle::new();
        self.available = true;
        // Preserve the reference sequence across replacement: refs must never be revived.
    }

    /// Admits a frame-local read target before any permission evaluation.
    /// Other operation families require their own complete resource admission path.
    pub fn prepare_observation(
        &mut self,
        request: BrowserRequest,
    ) -> Result<PreparedBrowserObservation, BrowserContextError> {
        let (tab, frame) = match request.command() {
            BrowserCommand::Read { tab_ref, frame_ref }
            | BrowserCommand::Snapshot { tab_ref, frame_ref }
            | BrowserCommand::CurrentUrl { tab_ref, frame_ref }
            | BrowserCommand::Screenshot { tab_ref, frame_ref } => (tab_ref, frame_ref),
            _ => return Err(BrowserContextError::InvalidReference),
        };
        let scope = self.resolve(tab, frame)?;
        let ticket = self.tickets.prepare(request)?;
        Ok(PreparedBrowserObservation { scope, ticket })
    }

    /// Re-resolves the exact admitted target and validates the same host reservation.
    /// A valid reservation alone never grants permission to dispatch.
    pub fn validate_observation(
        &self,
        prepared: &PreparedBrowserObservation,
    ) -> Result<(), BrowserContextError> {
        self.ensure_available()?;
        self.tickets.validate(&prepared.ticket)?;
        let current = self.resolve(&prepared.scope.tab, &prepared.scope.frame)?;
        if current != prepared.scope {
            return Err(BrowserContextError::InvalidReference);
        }
        Ok(())
    }

    /// Retires a canceled or denied observation without executing any browser operation.
    pub fn discard_observation(
        &mut self,
        prepared: PreparedBrowserObservation,
    ) -> Result<(), BrowserContextError> {
        self.tickets.discard(prepared.ticket).map_err(Into::into)
    }

    /// Returns the exact permission resource for a still-valid prepared observation.
    pub fn browser_permission_resource(
        &self,
        prepared: &PreparedBrowserObservation,
    ) -> Result<super::BrowserPermissionResource, BrowserContextError> {
        self.validate_observation(prepared)?;
        super::BrowserPermissionResource::from_observation(prepared)
            .map_err(|_| BrowserContextError::Unavailable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn origin(host: &str) -> BrowserOrigin {
        BrowserOrigin::from_effective_url(host).expect("valid host evidence")
    }

    #[test]
    fn all_operations_prepare_exact_typed_targets_and_revalidate() {
        use super::super::BrowserPermissionTarget as Target;
        let mut host = BrowserContext::new();
        let tab = host.insert_tab().expect("tab");
        let frame = host
            .insert_frame(&tab, None, Some(origin("https://example.com")), true)
            .expect("frame");
        let snapshot = host.replace_snapshot(&tab, &frame, 1).expect("snapshot");
        let operations = [
            "open",
            "read",
            "snapshot",
            "current-url",
            "screenshot",
            "click",
            "fill",
            "select",
            "hover",
            "check",
            "uncheck",
            "press",
            "scroll",
            "wait-for-element",
            "wait-milliseconds",
            "tab-new",
            "tab-list",
            "tab-close",
            "tab-switch",
            "window-close",
            "frame-tree",
        ];
        for operation in operations {
            let mut raw = serde_json::json!({"protocolVersion":2,"operation":operation});
            match operation {
                "tab-new" | "tab-list" | "window-close" => (),
                "wait-milliseconds" => raw["milliseconds"] = 1.into(),
                _ => raw["tabRef"] = serde_json::json!(tab),
            }
            match operation {
                "open" => raw["url"] = "https://destination.example/path".into(),
                "read" | "snapshot" | "current-url" | "screenshot" | "click" | "fill"
                | "select" | "hover" | "check" | "uncheck" | "press" | "scroll"
                | "wait-for-element" => raw["frameRef"] = serde_json::json!(frame),
                _ => (),
            }
            if matches!(
                operation,
                "click" | "fill" | "select" | "hover" | "check" | "uncheck" | "wait-for-element"
            ) {
                raw["snapshotRef"] = serde_json::json!(snapshot.snapshot);
                raw["elementRef"] = serde_json::json!(snapshot.elements[0]);
            }
            match operation {
                "fill" => raw["text"] = "input secret".into(),
                "select" => raw["option"] = "choice".into(),
                "press" => raw["key"] = "Enter".into(),
                "scroll" => raw["direction"] = "down".into(),
                _ => (),
            }
            let prepared = host
                .prepare_invocation(BrowserRequest::parse_raw(&raw.to_string()).expect("request"))
                .expect(operation);
            let resource = host
                .invocation_permission_resource(&prepared)
                .expect("resource");
            assert_eq!(&resource.target, prepared.target());
            assert_eq!(resource.operation, prepared.request().operation());
            let expected_class = match prepared.target() {
                Target::SessionInventory => super::super::BrowserResourceClass::SessionInventory,
                Target::SessionCreate => super::super::BrowserResourceClass::SessionCreate,
                Target::SessionWait { .. } => super::super::BrowserResourceClass::SessionWait,
                Target::SessionClose => super::super::BrowserResourceClass::SessionClose,
                Target::TabLifecycle { .. } => super::super::BrowserResourceClass::TabLifecycle,
                Target::FrameInventory { .. } => super::super::BrowserResourceClass::FrameInventory,
                Target::Navigate { .. } => super::super::BrowserResourceClass::Navigate,
                Target::Observe { .. } => super::super::BrowserResourceClass::Observe,
                Target::Interact { .. } => super::super::BrowserResourceClass::Interact,
            };
            assert_eq!(resource.operation.resource_class(), expected_class);
            host.discard_invocation(prepared).expect("retire");
        }
    }

    #[test]
    fn bootstrap_unknown_and_opaque_origins_are_distinct() {
        let mut host = BrowserContext::new();
        for operation in ["tab-list", "tab-new", "window-close", "wait-milliseconds"] {
            let mut raw = serde_json::json!({"protocolVersion":2,"operation":operation});
            if operation == "wait-milliseconds" {
                raw["milliseconds"] = 1.into();
            }
            let prepared = host
                .prepare_invocation(BrowserRequest::parse_raw(&raw.to_string()).expect("request"))
                .expect("no frame needed");
            host.discard_invocation(prepared).expect("discard");
        }
        let tab = host.insert_tab().expect("tab");
        let unknown = host.insert_frame(&tab, None, None, true).expect("unknown");
        let navigate = serde_json::json!({"protocolVersion":2,"operation":"open","tabRef":tab,"url":"https://example.com"}).to_string();
        assert!(
            host.prepare_invocation(BrowserRequest::parse_raw(&navigate).expect("request"))
                .is_err()
        );
        host.invalidate_frame(&tab, &unknown)
            .expect("replace unknown");
        let opaque = host
            .insert_opaque_frame(&tab, None, true)
            .expect("verified empty document");
        let prepared = host
            .prepare_invocation(BrowserRequest::parse_raw(&navigate).expect("request"))
            .expect("navigate opaque tab");
        assert!(
            matches!(prepared.target(), super::super::BrowserPermissionTarget::Navigate { current_origin: super::super::BrowserDocumentOrigin::OpaqueDocument(reference), .. } if reference == &opaque)
        );
        assert!(host.resolve(&tab, &opaque).is_err());
        host.replace_session();
        assert!(host.validate_invocation(&prepared).is_err());
    }

    #[test]
    fn observation_release_rejects_navigation_after_ticket_consumption() {
        let mut host = BrowserContext::new();
        let tab = host.insert_tab().expect("tab");
        let frame = host
            .insert_frame(&tab, None, Some(origin("https://example.com")), true)
            .expect("frame");
        let raw = serde_json::json!({"protocolVersion":2,"operation":"read","tabRef":tab,"frameRef":frame});
        let prepared = host
            .prepare_invocation(BrowserRequest::parse_raw(&raw.to_string()).expect("request"))
            .expect("prepare");
        let target = prepared.target().clone();
        let request = host.consume_invocation(prepared).expect("dispatch");
        assert!(host.validate_observation_release(&request, &target).is_ok());
        host.invalidate_frame(&tab, &frame).expect("navigation");
        assert!(
            host.validate_observation_release(&request, &target)
                .is_err()
        );
    }

    #[test]
    fn output_identity_rejects_foreign_snapshot_origin_and_unregistered_tab() {
        use super::super::{BrowserOutput, BrowserPermissionTarget};
        let mut host = BrowserContext::new();
        let tab = host.insert_tab().expect("tab");
        let parent = host
            .insert_frame(&tab, None, Some(origin("https://parent.example")), true)
            .expect("parent");
        let child = host
            .insert_frame(
                &tab,
                Some(&parent),
                Some(origin("https://child.example")),
                true,
            )
            .expect("child");
        let own = host
            .replace_snapshot(&tab, &parent, 0)
            .expect("own snapshot");
        let foreign = host
            .replace_snapshot(&tab, &child, 0)
            .expect("child snapshot");
        let target = BrowserPermissionTarget::Observe {
            scope: host.resolve(&tab, &parent).expect("scope"),
            operation: super::super::BrowserOperation::Snapshot,
            element: None,
        };
        for (snapshot_ref, accepted) in [(own.snapshot, true), (foreign.snapshot, false)] {
            let output = BrowserOutput::Snapshot {
                snapshot_ref,
                nodes: vec![],
                truncated: false,
            };
            assert_eq!(
                host.validate_output_identity(&target, &output).is_ok(),
                accepted
            );
        }
        for (origin, accepted) in [
            ("https://parent.example", true),
            ("https://child.example", false),
        ] {
            let output = BrowserOutput::PageUrl {
                origin: origin.to_owned(),
                path: "/".to_owned(),
            };
            assert_eq!(
                host.validate_output_identity(&target, &output).is_ok(),
                accepted
            );
        }
        let output = BrowserOutput::TabCreated {
            tab_ref: BrowserTabRef::try_new("unregistered").expect("reference"),
        };
        assert!(
            host.validate_output_identity(&BrowserPermissionTarget::SessionCreate, &output)
                .is_err()
        );
    }

    #[test]
    fn inventory_output_checks_trusted_origin_parent_and_readiness() {
        use super::super::{
            BrowserFrameEntry, BrowserOutput, BrowserPermissionTarget, BrowserReadiness,
            BrowserTabEntry,
        };
        let mut host = BrowserContext::new();
        let tab = host.insert_tab().expect("tab");
        let root = host
            .insert_frame(&tab, None, Some(origin("https://example.com")), true)
            .expect("root");
        let child = host
            .insert_frame(
                &tab,
                Some(&root),
                Some(origin("https://child.example")),
                false,
            )
            .expect("child");
        let target = BrowserPermissionTarget::FrameInventory {
            tab: tab.clone(),
            top_origin: host.top_origin(&tab).expect("origin"),
        };
        for (parent, origin, readiness, accepted) in [
            (
                Some(root.clone()),
                "https://child.example",
                BrowserReadiness::Pending,
                true,
            ),
            (
                None,
                "https://child.example",
                BrowserReadiness::Pending,
                false,
            ),
            (
                Some(root.clone()),
                "https://example.com",
                BrowserReadiness::Pending,
                false,
            ),
            (
                Some(root.clone()),
                "https://child.example",
                BrowserReadiness::Ready,
                false,
            ),
        ] {
            let output = BrowserOutput::Frames {
                entries: vec![BrowserFrameEntry {
                    frame_ref: child.clone(),
                    parent_frame_ref: parent,
                    origin: origin.to_owned(),
                    readiness,
                }],
                truncated: true,
            };
            assert_eq!(
                host.validate_output_identity(&target, &output).is_ok(),
                accepted
            );
        }
        for (origin, accepted) in [
            ("https://example.com", true),
            ("https://child.example", false),
            ("opaque", false),
        ] {
            let output = BrowserOutput::Tabs {
                entries: vec![BrowserTabEntry {
                    tab_ref: tab.clone(),
                    origin: origin.to_owned(),
                }],
                truncated: false,
            };
            assert_eq!(
                host.validate_output_identity(&BrowserPermissionTarget::SessionInventory, &output)
                    .is_ok(),
                accepted
            );
        }
        host.invalidate_frame(&tab, &child).expect("detach");
        let output = BrowserOutput::Frames {
            entries: vec![BrowserFrameEntry {
                frame_ref: child,
                parent_frame_ref: Some(root),
                origin: "https://child.example".to_owned(),
                readiness: BrowserReadiness::Pending,
            }],
            truncated: true,
        };
        assert!(host.validate_output_identity(&target, &output).is_err());
    }

    #[test]
    fn prepared_interaction_rechecks_snapshot_before_permission() {
        let mut host = BrowserContext::new();
        let tab = host.insert_tab().expect("tab");
        let frame = host
            .insert_frame(&tab, None, Some(origin("https://example.com")), true)
            .expect("frame");
        let snapshot = host.replace_snapshot(&tab, &frame, 1).expect("snapshot");
        let raw = serde_json::json!({"protocolVersion":2,"operation":"fill","tabRef":tab,"frameRef":frame,"snapshotRef":snapshot.snapshot,"elementRef":snapshot.elements[0],"text":"secret"});
        let prepared = host
            .prepare_invocation(BrowserRequest::parse_raw(&raw.to_string()).expect("request"))
            .expect("prepare");
        assert!(host.invocation_permission_resource(&prepared).is_ok());
        host.replace_snapshot(&tab, &frame, 1)
            .expect("replace snapshot");
        assert!(host.invocation_permission_resource(&prepared).is_err());
        assert!(
            host.prepare_invocation(BrowserRequest::parse_raw(&raw.to_string()).expect("request"))
                .is_err()
        );
    }

    #[test]
    fn child_scope_never_inherits_parent_and_foreign_tab_is_rejected() {
        let mut host = BrowserContext::new();
        let tab = host.insert_tab().expect("tab");
        let root = host
            .insert_frame(&tab, None, Some(origin("https://parent.example")), true)
            .expect("root");
        let child = host
            .insert_frame(
                &tab,
                Some(&root),
                Some(origin("https://child.example")),
                true,
            )
            .expect("child");
        let scope = host.resolve(&tab, &child).expect("scope");
        assert_ne!(scope.top_origin, scope.frame_origin);
        let other = host.insert_tab().expect("other tab");
        assert_eq!(
            host.resolve(&other, &child),
            Err(BrowserContextError::InvalidReference)
        );
        host.invalidate_frame(&tab, &root).expect("navigation");
        assert_eq!(
            host.resolve(&tab, &child),
            Err(BrowserContextError::InvalidReference)
        );
    }

    #[test]
    fn pending_opaque_and_stream_loss_are_not_observable() {
        let mut host = BrowserContext::new();
        let tab = host.insert_tab().expect("tab");
        let root = host
            .insert_frame(&tab, None, Some(origin("https://parent.example")), true)
            .expect("root");
        let pending = host
            .insert_frame(
                &tab,
                Some(&root),
                Some(origin("https://child.example")),
                false,
            )
            .expect("pending");
        assert_eq!(
            host.resolve(&tab, &pending),
            Err(BrowserContextError::Pending)
        );
        let opaque = host
            .insert_frame(&tab, Some(&root), None, true)
            .expect("opaque");
        assert_eq!(
            host.resolve(&tab, &opaque),
            Err(BrowserContextError::OriginUnavailable)
        );
        host.lose_synchronization();
        assert_eq!(
            host.resolve(&tab, &root),
            Err(BrowserContextError::Unavailable)
        );
        assert_eq!(host.insert_tab(), Err(BrowserContextError::Unavailable));
    }

    #[test]
    fn exact_origin_canonicalization_preserves_scheme_and_port() {
        assert_eq!(
            origin("https://EXAMPLE.com:443/path?secret#fragment"),
            origin("https://example.com")
        );
        assert_ne!(origin("http://example.com"), origin("https://example.com"));
        assert_ne!(
            origin("https://example.com:8443"),
            origin("https://example.com")
        );
        assert_eq!(
            origin("https://bücher.example"),
            origin("https://xn--bcher-kva.example")
        );
    }

    #[test]
    fn registry_budgets_and_subtree_revocation_preserve_siblings() {
        let mut host = BrowserContext::new();
        let tab = host.insert_tab().expect("tab");
        let root = host
            .insert_frame(&tab, None, Some(origin("https://example.com")), true)
            .expect("root");
        let sibling = host
            .insert_frame(&tab, Some(&root), Some(origin("https://example.com")), true)
            .expect("sibling");
        let mut deepest = root.clone();
        let mut branch = None;
        for _ in 1..16 {
            deepest = host
                .insert_frame(
                    &tab,
                    Some(&deepest),
                    Some(origin("https://example.com")),
                    true,
                )
                .expect("nested frame");
            if branch.is_none() {
                branch = Some(deepest.clone());
            }
        }
        assert_eq!(
            host.insert_frame(&tab, Some(&deepest), None, true),
            Err(BrowserContextError::ResourceLimit)
        );
        host.invalidate_frame(&tab, &branch.expect("branch"))
            .expect("rebuild");
        assert!(host.resolve(&tab, &sibling).is_ok());
        assert_eq!(
            host.resolve(&tab, &deepest),
            Err(BrowserContextError::InvalidReference)
        );
        for _ in 2..128 {
            host.insert_frame(&tab, Some(&root), None, false)
                .expect("capacity");
        }
        assert_eq!(
            host.insert_frame(&tab, Some(&root), None, false),
            Err(BrowserContextError::ResourceLimit)
        );
        for _ in 1..64 {
            host.insert_tab().expect("tab capacity");
        }
        assert_eq!(host.insert_tab(), Err(BrowserContextError::ResourceLimit));
        host.close_tab(&tab).expect("close tab");
        assert_eq!(
            host.resolve(&tab, &sibling),
            Err(BrowserContextError::InvalidReference)
        );
        assert!(host.insert_tab().is_ok());
    }

    #[test]
    fn ready_descendant_cannot_bypass_pending_ancestor() {
        let mut host = BrowserContext::new();
        let tab = host.insert_tab().expect("tab");
        let root = host
            .insert_frame(&tab, None, Some(origin("https://example.com")), false)
            .expect("pending root");
        let child = host
            .insert_frame(&tab, Some(&root), Some(origin("https://example.com")), true)
            .expect("ready child");
        assert_eq!(
            host.resolve(&tab, &child),
            Err(BrowserContextError::Pending)
        );
    }

    #[test]
    fn admitted_observation_is_bound_to_host_and_revoked_by_navigation() {
        let mut host = BrowserContext::new();
        let tab = host.insert_tab().expect("tab");
        let root = host
            .insert_frame(&tab, None, Some(origin("https://parent.example")), true)
            .expect("root");
        let child = host
            .insert_frame(
                &tab,
                Some(&root),
                Some(origin("https://child.example")),
                true,
            )
            .expect("child");
        let raw = serde_json::json!({"protocolVersion":2,"operation":"read","tabRef":tab,"frameRef":child}).to_string();
        let prepared = host
            .prepare_observation(BrowserRequest::parse_raw(&raw).expect("request"))
            .expect("admission");
        assert!(host.validate_observation(&prepared).is_ok());
        assert!(
            BrowserContext::new()
                .validate_observation(&prepared)
                .is_err()
        );
        host.invalidate_frame(&tab, &child)
            .expect("navigation during approval");
        assert!(host.validate_observation(&prepared).is_err());
        assert!(
            host.prepare_observation(BrowserRequest::parse_raw(&raw).expect("request"))
                .is_err()
        );
    }

    #[test]
    fn loss_of_sync_revokes_prepared_work_and_closed_refs_never_revive() {
        let mut host = BrowserContext::new();
        let tab = host.insert_tab().expect("tab");
        let root = host
            .insert_frame(&tab, None, Some(origin("https://example.com")), true)
            .expect("root");
        let raw = serde_json::json!({"protocolVersion":2,"operation":"read","tabRef":tab,"frameRef":root}).to_string();
        let prepared = host
            .prepare_observation(BrowserRequest::parse_raw(&raw).expect("request"))
            .expect("admission");
        host.lose_synchronization();
        assert_eq!(
            host.validate_observation(&prepared),
            Err(BrowserContextError::Unavailable)
        );
        let mut fresh = BrowserContext::new();
        assert!(
            fresh
                .prepare_observation(BrowserRequest::parse_raw(&raw).expect("request"))
                .is_err()
        );
    }

    #[test]
    fn session_replacement_revokes_pending_work_and_recovers_only_with_fresh_refs() {
        let mut host = BrowserContext::new();
        let old_tab = host.insert_tab().expect("tab");
        let old_frame = host
            .insert_frame(&old_tab, None, Some(origin("https://example.com")), true)
            .expect("frame");
        let raw = serde_json::json!({"protocolVersion":2,"operation":"read","tabRef":old_tab,"frameRef":old_frame}).to_string();
        let pending = host
            .prepare_observation(BrowserRequest::parse_raw(&raw).expect("request"))
            .expect("pending");
        host.lose_synchronization();
        host.replace_session();
        assert!(host.validate_observation(&pending).is_err());
        assert!(
            host.prepare_observation(BrowserRequest::parse_raw(&raw).expect("request"))
                .is_err()
        );
        let tab = host.insert_tab().expect("new tab");
        let frame = host
            .insert_frame(&tab, None, Some(origin("https://example.com")), true)
            .expect("new frame");
        assert_ne!(tab, old_tab);
        assert_ne!(frame, old_frame);
        let raw = serde_json::json!({"protocolVersion":2,"operation":"read","tabRef":tab,"frameRef":frame}).to_string();
        let fresh = host
            .prepare_observation(BrowserRequest::parse_raw(&raw).expect("request"))
            .expect("fresh pending");
        assert!(host.validate_observation(&fresh).is_ok());
        // Retiring a previous-session ticket cannot consume the new session's nonce 1.
        assert!(host.discard_observation(pending).is_err());
        assert!(host.validate_observation(&fresh).is_ok());
    }

    #[test]
    fn snapshot_replacement_rejects_old_and_foreign_element_tuples() {
        let mut host = BrowserContext::new();
        let tab = host.insert_tab().expect("tab");
        let root = host
            .insert_frame(&tab, None, Some(origin("https://example.com")), true)
            .expect("root");
        let child = host
            .insert_frame(
                &tab,
                Some(&root),
                Some(origin("https://child.example")),
                true,
            )
            .expect("child");
        let first = host.replace_snapshot(&tab, &root, 2).expect("snapshot");
        assert!(
            host.resolve_element(&tab, &root, &first.snapshot, &first.elements[0])
                .is_ok()
        );
        assert!(
            host.resolve_element(&tab, &child, &first.snapshot, &first.elements[0])
                .is_err()
        );
        let second = host
            .replace_snapshot(&tab, &root, 512)
            .expect("replacement");
        assert_ne!(first.snapshot, second.snapshot);
        assert!(
            host.resolve_element(&tab, &root, &first.snapshot, &first.elements[0])
                .is_err()
        );
        assert!(
            host.resolve_element(&tab, &root, &second.snapshot, &first.elements[0])
                .is_err()
        );
        assert_eq!(
            host.replace_snapshot(&tab, &root, 513)
                .expect_err("over budget"),
            BrowserContextError::ResourceLimit
        );
        assert!(
            host.resolve_element(&tab, &root, &second.snapshot, &second.elements[511])
                .is_ok()
        );
        let other = host.insert_tab().expect("other");
        host.switch_tab(&tab, &other).expect("switch");
        host.switch_tab(&other, &tab).expect("return");
        assert!(
            host.resolve_element(&tab, &root, &second.snapshot, &second.elements[0])
                .is_err()
        );
    }
}
