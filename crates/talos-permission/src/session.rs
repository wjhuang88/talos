//! Explicit in-memory permission Session state and approval fencing.

use std::fmt;
use std::sync::{Mutex, MutexGuard};

use talos_core::tool::{ToolAuthorizationScope, ToolExecutionAuthorization};
use uuid::Uuid;

use crate::grant::{
    ApprovedOnce, CompiledFacetScope, GrantError, GrantId, GrantScope, GrantSource,
    PermissionGrant, ProposalSnapshot, ProposedGrant, compile_proposal, compiled_identity,
};
use crate::{
    PermissionContext, PermissionDecision, PermissionDecisionReport, PermissionEngine,
    PermissionRequest,
};

/// Opaque identity of one in-memory permission Session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PermissionSessionId(Uuid);

impl PermissionSessionId {
    /// Creates a new unrelated Session identity.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Returns a stable opaque identifier suitable for an in-memory binding.
    #[must_use]
    pub fn stable_id(self) -> String {
        self.0.to_string()
    }
}

impl Default for PermissionSessionId {
    fn default() -> Self {
        Self::new()
    }
}

/// Revision snapshot bound to proposals and pending invocations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PermissionStateRevisions {
    policy: u64,
    mode: u64,
    workspace: u64,
    registration: u64,
    restriction: u64,
    store: u64,
}

/// Redaction-safe snapshot used to bind an approval assessment to one
/// permission Session state. The values carry no policy or resource data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PermissionStateSnapshot {
    /// Opaque in-memory Session identity.
    pub session_id: PermissionSessionId,
    /// Monotonic generations for policy, mode and related state.
    pub revisions: PermissionStateRevisions,
}

impl PermissionStateRevisions {
    /// Returns the six monotonic generations in stable order.
    #[must_use]
    pub const fn as_array(self) -> [u64; 6] {
        [
            self.policy,
            self.mode,
            self.workspace,
            self.registration,
            self.restriction,
            self.store,
        ]
    }
}

impl PermissionStateRevisions {
    const ZERO: Self = Self {
        policy: 0,
        mode: 0,
        workspace: 0,
        registration: 0,
        restriction: 0,
        store: 0,
    };
}

/// Structured evaluation plus any matching Session grant.
pub struct PermissionEvaluation {
    report: PermissionDecisionReport,
    matched_grant: Option<(GrantId, GrantSource)>,
}

/// Result of the single authoritative evaluation for one invocation.
pub enum PermissionInvocation {
    /// Current policy or an existing Session grant authorizes the request.
    Allow(Box<PendingInvocation>),
    /// The exact request requires a bounded approval decision.
    Ask {
        /// Invocation-local approval proposal.
        once: Box<ProposedGrant>,
        /// Capability-relative Session approval proposal.
        session: Box<ProposedGrant>,
    },
    /// Current policy denies the request.
    Deny(PermissionDecision),
}

impl PermissionEvaluation {
    /// Returns the authoritative structured report.
    #[must_use]
    pub const fn report(&self) -> &PermissionDecisionReport {
        &self.report
    }

    /// Returns the compatibility decision.
    #[must_use]
    pub fn decision(&self) -> PermissionDecision {
        self.report.decision()
    }

    /// Returns the matching Session grant, if reuse resolved the request.
    #[must_use]
    pub const fn matched_grant(&self) -> Option<(GrantId, GrantSource)> {
        self.matched_grant
    }
}

struct State {
    session_id: PermissionSessionId,
    revisions: PermissionStateRevisions,
    engine: PermissionEngine,
    grants: Vec<PermissionGrant>,
}

/// Composition-root-owned policy, proposal, grant, and admission state.
pub struct PermissionSessionState {
    state: Mutex<State>,
}

impl fmt::Debug for PermissionSessionState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("PermissionSessionState(<redacted>)")
    }
}

impl PermissionSessionState {
    /// Creates an empty grant store around an existing policy engine.
    #[must_use]
    pub fn new(engine: PermissionEngine) -> Self {
        Self {
            state: Mutex::new(State {
                session_id: PermissionSessionId::new(),
                revisions: PermissionStateRevisions::ZERO,
                engine,
                grants: Vec::new(),
            }),
        }
    }

    /// Evaluates current policy and then resolves only remaining `Ask` facets
    /// with an exact matching Session grant.
    pub fn evaluate(
        &self,
        request: &PermissionRequest<'_>,
        context: &PermissionContext,
    ) -> Result<PermissionEvaluation, GrantError> {
        let state = self
            .state
            .lock()
            .map_err(|_| GrantError::StateUnavailable)?;
        evaluate_locked(&state, request, context)
    }

    /// Begins one invocation with exactly one policy evaluation.
    pub fn begin_invocation(
        &self,
        request: &PermissionRequest<'_>,
        context: &PermissionContext,
    ) -> Result<PermissionInvocation, GrantError> {
        let state = self.lock_state()?;
        begin_invocation_locked(&state, request, context)
    }

    /// Begins one invocation without waiting for a contended Session fence.
    ///
    /// Deadline-bound orchestration uses this entry point so lock contention fails closed instead
    /// of extending the caller's total permission budget.
    pub fn try_begin_invocation(
        &self,
        request: &PermissionRequest<'_>,
        context: &PermissionContext,
    ) -> Result<PermissionInvocation, GrantError> {
        let state = self.try_lock_state()?;
        begin_invocation_locked(&state, request, context)
    }

    fn lock_state(&self) -> Result<MutexGuard<'_, State>, GrantError> {
        self.state.lock().map_err(|_| GrantError::StateUnavailable)
    }

    fn try_lock_state(&self) -> Result<MutexGuard<'_, State>, GrantError> {
        self.state
            .try_lock()
            .map_err(|_| GrantError::StateUnavailable)
    }

    /// Commits an invocation-local proposal without waiting for a contended Session fence.
    pub fn try_approve_once(
        &self,
        proposal: ProposedGrant,
        request: &PermissionRequest<'_>,
        context: &PermissionContext,
    ) -> Result<PendingInvocation, GrantError> {
        if proposal.scope != GrantScope::Once {
            return Err(GrantError::ScopeMismatch);
        }
        let state = self.try_lock_state()?;
        approve_once_locked(&state, proposal, request, context)
    }

    /// Commits a Session proposal without waiting for a contended Session fence.
    pub fn try_approve_session(
        &self,
        proposal: ProposedGrant,
        request: &PermissionRequest<'_>,
        context: &PermissionContext,
        source: GrantSource,
    ) -> Result<PendingInvocation, GrantError> {
        if proposal.scope != GrantScope::Session {
            return Err(GrantError::ScopeMismatch);
        }
        let mut state = self.try_lock_state()?;
        approve_session_locked(&mut state, proposal, request, context, source)
    }

    /// Consumes an invocation without waiting for a contended Session fence.
    pub fn try_admit(
        &self,
        pending: PendingInvocation,
        request: &PermissionRequest<'_>,
        context: &PermissionContext,
    ) -> Result<Vec<ToolExecutionAuthorization>, GrantError> {
        let state = self.try_lock_state()?;
        admit_locked(&state, pending, request, context)
    }

    #[cfg(test)]
    fn with_state_fence_held(&self, check: impl FnOnce()) {
        let _state = self.state.lock().expect("state fence");
        check();
    }
}

fn begin_invocation_locked(
    state: &State,
    request: &PermissionRequest<'_>,
    context: &PermissionContext,
) -> Result<PermissionInvocation, GrantError> {
    let evaluation = evaluate_locked(state, request, context)?;
    match evaluation.decision() {
        PermissionDecision::Allow => {
            let (facets, fingerprint) = compiled_identity(request, state.engine.workspace_root())?;
            let authority = match evaluation.matched_grant {
                Some((id, source)) => PendingAuthority::Session { id, source },
                None => PendingAuthority::Policy,
            };
            Ok(PermissionInvocation::Allow(Box::new(PendingInvocation {
                session_id: state.session_id,
                revisions: state.revisions,
                context: *context,
                tool_name: request.tool_name().to_string(),
                provenance: request.provenance().clone(),
                facets,
                fingerprint,
                authority,
            })))
        }
        PermissionDecision::Ask => {
            let snapshot = ProposalSnapshot {
                session_id: state.session_id.0,
                revisions: state.revisions.as_array(),
                mode: context.mode(),
                interaction: context.interaction(),
            };
            Ok(PermissionInvocation::Ask {
                once: Box::new(compile_proposal(
                    request,
                    state.engine.workspace_root(),
                    GrantScope::Once,
                    snapshot,
                )?),
                session: Box::new(compile_proposal(
                    request,
                    state.engine.workspace_root(),
                    GrantScope::Session,
                    snapshot,
                )?),
            })
        }
        decision @ PermissionDecision::Deny(_) => Ok(PermissionInvocation::Deny(decision)),
    }
}

impl PermissionSessionState {
    /// Compiles one non-authoritative approval proposal from the current state.
    pub fn propose(
        &self,
        request: &PermissionRequest<'_>,
        context: &PermissionContext,
        scope: GrantScope,
    ) -> Result<ProposedGrant, GrantError> {
        let state = self
            .state
            .lock()
            .map_err(|_| GrantError::StateUnavailable)?;
        if evaluate_locked(&state, request, context)?.decision() != PermissionDecision::Ask {
            return Err(GrantError::NotAwaitingApproval);
        }
        compile_proposal(
            request,
            state.engine.workspace_root(),
            scope,
            ProposalSnapshot {
                session_id: state.session_id.0,
                revisions: state.revisions.as_array(),
                mode: context.mode(),
                interaction: context.interaction(),
            },
        )
    }

    /// Commits a proposal as invocation-local authority after exact revalidation.
    pub fn approve_once(
        &self,
        proposal: ProposedGrant,
        request: &PermissionRequest<'_>,
        context: &PermissionContext,
    ) -> Result<PendingInvocation, GrantError> {
        if proposal.scope != GrantScope::Once {
            return Err(GrantError::ScopeMismatch);
        }
        let state = self.lock_state()?;
        approve_once_locked(&state, proposal, request, context)
    }

    /// Atomically installs a Session grant and returns authority for the current invocation.
    pub fn approve_session(
        &self,
        proposal: ProposedGrant,
        request: &PermissionRequest<'_>,
        context: &PermissionContext,
        source: GrantSource,
    ) -> Result<PendingInvocation, GrantError> {
        if proposal.scope != GrantScope::Session {
            return Err(GrantError::ScopeMismatch);
        }
        let mut state = self.lock_state()?;
        approve_session_locked(&mut state, proposal, request, context, source)
    }

    /// Prepares an already-authorized policy or matching Session request.
    pub fn prepare_authorized(
        &self,
        request: &PermissionRequest<'_>,
        context: &PermissionContext,
    ) -> Result<Option<PendingInvocation>, GrantError> {
        let state = self
            .state
            .lock()
            .map_err(|_| GrantError::StateUnavailable)?;
        let evaluation = evaluate_locked(&state, request, context)?;
        if evaluation.decision() != PermissionDecision::Allow {
            return Ok(None);
        }
        let (facets, fingerprint) = compiled_identity(request, state.engine.workspace_root())?;
        let authority = match evaluation.matched_grant {
            Some((id, source)) => PendingAuthority::Session { id, source },
            None => PendingAuthority::Policy,
        };
        Ok(Some(PendingInvocation {
            session_id: state.session_id,
            revisions: state.revisions,
            context: *context,
            tool_name: request.tool_name().to_string(),
            provenance: request.provenance().clone(),
            facets,
            fingerprint,
            authority,
        }))
    }

    /// Clears all grants and invalidates every issued-but-unstarted invocation.
    pub fn clear(&self) -> Result<(), GrantError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| GrantError::StateUnavailable)?;
        let next_store = next_generation(state.revisions.store)?;
        state.grants.clear();
        state.revisions.store = next_store;
        Ok(())
    }

    /// Returns the number of currently installed Session grants.
    pub fn grant_count(&self) -> Result<usize, GrantError> {
        let state = self
            .state
            .lock()
            .map_err(|_| GrantError::StateUnavailable)?;
        Ok(state.grants.len())
    }

    /// Returns the opaque identity of this in-memory permission Session.
    pub fn session_id(&self) -> Result<PermissionSessionId, GrantError> {
        let state = self
            .state
            .lock()
            .map_err(|_| GrantError::StateUnavailable)?;
        Ok(state.session_id)
    }

    /// Returns the current redaction-safe Session identity and revision
    /// generations for binding an external assessment.
    pub fn state_snapshot(&self) -> Result<PermissionStateSnapshot, GrantError> {
        let state = self
            .state
            .lock()
            .map_err(|_| GrantError::StateUnavailable)?;
        Ok(PermissionStateSnapshot {
            session_id: state.session_id,
            revisions: state.revisions,
        })
    }

    /// Starts a new unrelated permission Session at a successful runtime
    /// publication boundary.
    pub fn rebind_session(&self) -> Result<(), GrantError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| GrantError::StateUnavailable)?;
        let next_store = next_generation(state.revisions.store)?;
        state.session_id = PermissionSessionId::new();
        state.grants.clear();
        state.revisions.store = next_store;
        Ok(())
    }

    /// Publishes an external Session transition while holding the permission
    /// state fence, then rebinds the permission Session only if publication
    /// succeeds. Permission evaluation and admission cannot observe the new
    /// routes with the old Session grants.
    pub fn publish_and_rebind<E>(
        &self,
        publish: impl FnOnce() -> Result<(), E>,
    ) -> Result<Result<(), E>, GrantError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| GrantError::StateUnavailable)?;
        let result = publish();
        if result.is_ok() {
            let next_store = next_generation(state.revisions.store)?;
            state.session_id = PermissionSessionId::new();
            state.grants.clear();
            state.revisions.store = next_store;
        }
        Ok(result)
    }

    /// Replaces authoritative policy while preserving grants only when the
    /// workspace identity is unchanged.
    pub fn replace_policy(&self, engine: PermissionEngine) -> Result<(), GrantError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| GrantError::StateUnavailable)?;
        let next_policy = next_generation(state.revisions.policy)?;
        let workspace_changed = state.engine.workspace_root() != engine.workspace_root();
        let next_workspace = workspace_changed
            .then(|| next_generation(state.revisions.workspace))
            .transpose()?;
        let next_store = workspace_changed
            .then(|| next_generation(state.revisions.store))
            .transpose()?;
        state.engine = engine;
        state.revisions.policy = next_policy;
        if let (Some(next_workspace), Some(next_store)) = (next_workspace, next_store) {
            state.revisions.workspace = next_workspace;
            state.revisions.store = next_store;
            state.grants.clear();
        }
        Ok(())
    }

    /// Invalidates pending work after a mode change.
    pub fn bump_mode_generation(&self) -> Result<(), GrantError> {
        self.bump(|revisions| &mut revisions.mode)
    }

    /// Invalidates pending work after a workspace change.
    pub fn bump_workspace_generation(&self) -> Result<(), GrantError> {
        self.bump_and_clear(|revisions| &mut revisions.workspace)
    }

    /// Invalidates pending work after tool registration changes.
    pub fn bump_registration_generation(&self) -> Result<(), GrantError> {
        self.bump_and_clear(|revisions| &mut revisions.registration)
    }

    /// Invalidates pending work after sandbox/fallback restriction changes.
    pub fn bump_restriction_generation(&self) -> Result<(), GrantError> {
        self.bump(|revisions| &mut revisions.restriction)
    }

    fn bump(
        &self,
        field: impl Fn(&mut PermissionStateRevisions) -> &mut u64,
    ) -> Result<(), GrantError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| GrantError::StateUnavailable)?;
        let value = field(&mut state.revisions);
        *value = next_generation(*value)?;
        Ok(())
    }

    fn bump_and_clear(
        &self,
        field: impl Fn(&mut PermissionStateRevisions) -> &mut u64,
    ) -> Result<(), GrantError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| GrantError::StateUnavailable)?;
        let next_field = next_generation(*field(&mut state.revisions))?;
        let next_store = next_generation(state.revisions.store)?;
        *field(&mut state.revisions) = next_field;
        state.revisions.store = next_store;
        state.grants.clear();
        Ok(())
    }

    /// Consumes a pending invocation at the execution admission fence.
    pub fn admit(
        &self,
        pending: PendingInvocation,
        request: &PermissionRequest<'_>,
        context: &PermissionContext,
    ) -> Result<Vec<ToolExecutionAuthorization>, GrantError> {
        let state = self.lock_state()?;
        admit_locked(&state, pending, request, context)
    }
}

fn approve_once_locked(
    state: &State,
    proposal: ProposedGrant,
    request: &PermissionRequest<'_>,
    context: &PermissionContext,
) -> Result<PendingInvocation, GrantError> {
    validate_proposal(state, &proposal, request, context)?;
    let approved = ApprovedOnce {
        tool_name: proposal.tool_name,
        provenance: proposal.provenance,
        facets: proposal.facets,
        fingerprint: proposal.fingerprint,
    };
    Ok(PendingInvocation::once(
        state.session_id,
        state.revisions,
        *context,
        approved,
    ))
}

fn approve_session_locked(
    state: &mut State,
    proposal: ProposedGrant,
    request: &PermissionRequest<'_>,
    context: &PermissionContext,
    source: GrantSource,
) -> Result<PendingInvocation, GrantError> {
    validate_proposal(state, &proposal, request, context)?;
    let id = GrantId::new();
    let next_store = next_generation(state.revisions.store)?;
    let next_revisions = PermissionStateRevisions {
        store: next_store,
        ..state.revisions
    };
    let pending = PendingInvocation {
        session_id: state.session_id,
        revisions: next_revisions,
        context: *context,
        tool_name: proposal.tool_name.clone(),
        provenance: proposal.provenance.clone(),
        facets: proposal.facets.clone(),
        fingerprint: proposal.fingerprint,
        authority: PendingAuthority::Session { id, source },
    };
    state.grants.push(PermissionGrant {
        id,
        source,
        tool_name: proposal.tool_name,
        provenance: proposal.provenance,
        facets: proposal.facets,
    });
    state.revisions = next_revisions;
    Ok(pending)
}

fn admit_locked(
    state: &State,
    pending: PendingInvocation,
    request: &PermissionRequest<'_>,
    context: &PermissionContext,
) -> Result<Vec<ToolExecutionAuthorization>, GrantError> {
    if pending.session_id != state.session_id || pending.revisions != state.revisions {
        return Err(GrantError::AdmissionInvalidated);
    }
    if pending.context != *context {
        return Err(GrantError::AdmissionInvalidated);
    }
    let (facets, fingerprint) = compiled_identity(request, state.engine.workspace_root())?;
    if pending.tool_name != request.tool_name()
        || pending.provenance != *request.provenance()
        || pending.facets != facets
        || pending.fingerprint != fingerprint
    {
        return Err(GrantError::AdmissionInvalidated);
    }
    let scope = match pending.authority {
        PendingAuthority::Policy => ToolAuthorizationScope::Policy,
        PendingAuthority::Once => ToolAuthorizationScope::Once,
        PendingAuthority::Session { id, source }
            if state.grants.iter().any(|grant| {
                grant.id == id
                    && grant.source == source
                    && grant_matches(grant, request.tool_name(), request.provenance(), &facets)
            }) =>
        {
            ToolAuthorizationScope::Session
        }
        _ => return Err(GrantError::AdmissionInvalidated),
    };
    state
        .engine
        .execution_authorizations(
            request.tool_name(),
            request.facets(),
            request.input(),
            scope,
        )
        .map_err(|_| GrantError::AdmissionInvalidated)
}

/// Non-clone authority waiting at the official tool admission fence.
pub struct PendingInvocation {
    session_id: PermissionSessionId,
    revisions: PermissionStateRevisions,
    context: PermissionContext,
    tool_name: String,
    provenance: talos_core::tool::ToolProvenance,
    facets: Vec<CompiledFacetScope>,
    fingerprint: crate::grant::RequestFingerprint,
    authority: PendingAuthority,
}

enum PendingAuthority {
    Policy,
    Once,
    Session { id: GrantId, source: GrantSource },
}

impl PendingInvocation {
    fn once(
        session_id: PermissionSessionId,
        revisions: PermissionStateRevisions,
        context: PermissionContext,
        approved: ApprovedOnce,
    ) -> Self {
        let ApprovedOnce {
            tool_name,
            provenance,
            facets,
            fingerprint,
        } = approved;
        Self {
            session_id,
            revisions,
            context,
            tool_name,
            provenance,
            facets,
            fingerprint,
            authority: PendingAuthority::Once,
        }
    }
}

impl fmt::Debug for PendingInvocation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("PendingInvocation(<redacted>)")
    }
}

fn evaluate_locked(
    state: &State,
    request: &PermissionRequest<'_>,
    context: &PermissionContext,
) -> Result<PermissionEvaluation, GrantError> {
    let report = state.engine.evaluate_request(request, context);
    if report.decision() != PermissionDecision::Ask {
        return Ok(PermissionEvaluation {
            report,
            matched_grant: None,
        });
    }
    let (facets, _) = compiled_identity(request, state.engine.workspace_root())?;
    if let Some(grant) = state
        .grants
        .iter()
        .find(|grant| grant_matches(grant, request.tool_name(), request.provenance(), &facets))
    {
        let id = grant.id;
        let source = grant.source;
        return Ok(PermissionEvaluation {
            report: report.resolve_asks_with_grant(id, source),
            matched_grant: Some((id, source)),
        });
    }
    Ok(PermissionEvaluation {
        report,
        matched_grant: None,
    })
}

fn grant_matches(
    grant: &PermissionGrant,
    tool_name: &str,
    provenance: &talos_core::tool::ToolProvenance,
    facets: &[CompiledFacetScope],
) -> bool {
    grant.tool_name == tool_name && grant.provenance == *provenance && grant.facets == facets
}

fn validate_proposal(
    state: &State,
    proposal: &ProposedGrant,
    request: &PermissionRequest<'_>,
    context: &PermissionContext,
) -> Result<(), GrantError> {
    if proposal.snapshot
        != (ProposalSnapshot {
            session_id: state.session_id.0,
            revisions: state.revisions.as_array(),
            mode: context.mode(),
            interaction: context.interaction(),
        })
    {
        return Err(GrantError::StaleApproval);
    }
    let (facets, fingerprint) = compiled_identity(request, state.engine.workspace_root())?;
    if proposal.tool_name != request.tool_name()
        || proposal.provenance != *request.provenance()
        || proposal.facets != facets
        || proposal.fingerprint != fingerprint
    {
        return Err(GrantError::StaleApproval);
    }
    Ok(())
}

fn next_generation(value: u64) -> Result<u64, GrantError> {
    value.checked_add(1).ok_or(GrantError::StateUnavailable)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use talos_core::tool::{
        ToolAuthorizationScope, ToolNature, ToolPermissionFacet, ToolProvenance, ToolResourceKind,
    };

    fn context() -> PermissionContext {
        PermissionContext::new(
            crate::PermissionMode::Interactive,
            crate::InteractionCapability::Available,
        )
    }

    fn path_facet(nature: ToolNature, path: &std::path::Path) -> ToolPermissionFacet {
        ToolPermissionFacet::with_resource(
            nature,
            path.display().to_string(),
            ToolResourceKind::Path,
        )
    }

    #[test]
    fn deadline_bound_begin_fails_closed_when_state_fence_is_contended() {
        let state = PermissionSessionState::new(PermissionEngine::new());
        let input = json!({});
        let facets = [ToolPermissionFacet::new(ToolNature::Read)];
        let request = PermissionRequest::new("read", ToolProvenance::Native, &facets, &input);

        state.with_state_fence_held(|| {
            assert!(matches!(
                state.try_begin_invocation(&request, &context()),
                Err(GrantError::StateUnavailable)
            ));
        });
    }

    fn stale_after_mutation(root: &std::path::Path, mutate: impl FnOnce(&PermissionSessionState)) {
        let target = root.join("stale-target.txt");
        std::fs::write(&target, b"test").expect("write fixture");
        let state =
            PermissionSessionState::new(PermissionEngine::with_workspace_root(root.to_path_buf()));
        let input = json!({"path": target});
        let facets = [path_facet(ToolNature::Write, &target)];
        let request = PermissionRequest::new("write", ToolProvenance::Native, &facets, &input);
        let context = context();
        let proposal = state
            .propose(&request, &context, GrantScope::Session)
            .expect("proposal");
        mutate(&state);
        assert!(matches!(
            state.approve_session(proposal, &request, &context, GrantSource::InteractiveHuman,),
            Err(GrantError::StaleApproval)
        ));
        assert_eq!(state.grant_count().expect("grant count"), 0);
    }

    #[test]
    fn every_revision_invalidates_a_pending_proposal() {
        let root = tempfile::tempdir().expect("tempdir");
        stale_after_mutation(root.path(), |state| {
            state
                .replace_policy(PermissionEngine::with_workspace_root(
                    root.path().to_path_buf(),
                ))
                .expect("replace policy");
        });
        stale_after_mutation(root.path(), |state| {
            state.bump_mode_generation().expect("mode generation");
        });
        stale_after_mutation(root.path(), |state| {
            state
                .bump_workspace_generation()
                .expect("workspace generation");
        });
        stale_after_mutation(root.path(), |state| {
            state
                .bump_registration_generation()
                .expect("registration generation");
        });
        stale_after_mutation(root.path(), |state| {
            state
                .bump_restriction_generation()
                .expect("restriction generation");
        });
        stale_after_mutation(root.path(), |state| {
            state.clear().expect("store generation");
        });
    }

    #[test]
    fn another_session_rejects_a_proposal_snapshot() {
        let root = tempfile::tempdir().expect("tempdir");
        let target = root.path().join("session-target.txt");
        std::fs::write(&target, b"test").expect("write fixture");
        let first = PermissionSessionState::new(PermissionEngine::with_workspace_root(
            root.path().to_path_buf(),
        ));
        let second = PermissionSessionState::new(PermissionEngine::with_workspace_root(
            root.path().to_path_buf(),
        ));
        let input = json!({"path": target});
        let facets = [path_facet(ToolNature::Write, &target)];
        let request = PermissionRequest::new("write", ToolProvenance::Native, &facets, &input);
        let context = context();
        let proposal = first
            .propose(&request, &context, GrantScope::Session)
            .expect("proposal");

        assert!(matches!(
            second.approve_session(proposal, &request, &context, GrantSource::InteractiveHuman,),
            Err(GrantError::StaleApproval)
        ));
    }

    #[test]
    fn once_authority_is_consumed_without_entering_the_store() {
        let root = tempfile::tempdir().expect("tempdir");
        let target = root.path().join("once-target.txt");
        std::fs::write(&target, b"test").expect("write fixture");
        let state = PermissionSessionState::new(PermissionEngine::with_workspace_root(
            root.path().to_path_buf(),
        ));
        let input = json!({"path": target});
        let facets = [path_facet(ToolNature::Write, &target)];
        let request = PermissionRequest::new("write", ToolProvenance::Native, &facets, &input);
        let context = context();
        let proposal = state
            .propose(&request, &context, GrantScope::Once)
            .expect("proposal");
        let pending = state
            .approve_once(proposal, &request, &context)
            .expect("Once approval");
        let authorizations = state.admit(pending, &request, &context).expect("admission");

        assert!(
            authorizations
                .iter()
                .all(|authorization| authorization.scope() == ToolAuthorizationScope::Once)
        );
        assert_eq!(state.grant_count().expect("grant count"), 0);
        assert_eq!(
            state
                .evaluate(&request, &context)
                .expect("evaluation")
                .decision(),
            PermissionDecision::Ask
        );
    }

    #[test]
    fn clear_before_admission_invalidates_once_authority() {
        let root = tempfile::tempdir().expect("tempdir");
        let target = root.path().join("clear-target.txt");
        std::fs::write(&target, b"test").expect("write fixture");
        let state = PermissionSessionState::new(PermissionEngine::with_workspace_root(
            root.path().to_path_buf(),
        ));
        let input = json!({"path": target});
        let facets = [path_facet(ToolNature::Write, &target)];
        let request = PermissionRequest::new("write", ToolProvenance::Native, &facets, &input);
        let context = context();
        let proposal = state
            .propose(&request, &context, GrantScope::Once)
            .expect("proposal");
        let pending = state
            .approve_once(proposal, &request, &context)
            .expect("Once approval");
        state.clear().expect("clear");

        assert!(matches!(
            state.admit(pending, &request, &context),
            Err(GrantError::AdmissionInvalidated)
        ));
    }

    #[test]
    fn context_changes_invalidate_proposal_and_pending_authority() {
        let root = tempfile::tempdir().expect("tempdir");
        let target = root.path().join("context-target.txt");
        std::fs::write(&target, b"test").expect("write fixture");
        let state = PermissionSessionState::new(PermissionEngine::with_workspace_root(
            root.path().to_path_buf(),
        ));
        let input = json!({"path": target});
        let facets = [path_facet(ToolNature::Write, &target)];
        let request = PermissionRequest::new("write", ToolProvenance::Native, &facets, &input);
        let interactive = context();
        let headless = PermissionContext::new(
            crate::PermissionMode::Headless,
            crate::InteractionCapability::Unavailable,
        );
        let stale_proposal = state
            .propose(&request, &interactive, GrantScope::Once)
            .expect("proposal");
        assert!(matches!(
            state.approve_once(stale_proposal, &request, &headless),
            Err(GrantError::StaleApproval)
        ));

        let proposal = state
            .propose(&request, &interactive, GrantScope::Once)
            .expect("proposal");
        let pending = state
            .approve_once(proposal, &request, &interactive)
            .expect("Once approval");
        assert!(matches!(
            state.admit(pending, &request, &headless),
            Err(GrantError::AdmissionInvalidated)
        ));
    }

    #[test]
    fn rebind_changes_session_identity_and_invalidates_old_authority() {
        let root = tempfile::tempdir().expect("tempdir");
        let approved_target = root.path().join("approved-target.txt");
        let pending_target = root.path().join("pending-target.txt");
        std::fs::write(&approved_target, b"approved").expect("write fixture");
        std::fs::write(&pending_target, b"pending").expect("write fixture");
        let state = PermissionSessionState::new(PermissionEngine::with_workspace_root(
            root.path().to_path_buf(),
        ));
        let context = context();

        let approved_input = json!({"path": approved_target});
        let approved_facets = [path_facet(ToolNature::Write, &approved_target)];
        let approved_request = PermissionRequest::new(
            "write",
            ToolProvenance::Native,
            &approved_facets,
            &approved_input,
        );
        let approved_proposal = state
            .propose(&approved_request, &context, GrantScope::Session)
            .expect("proposal");
        state
            .approve_session(
                approved_proposal,
                &approved_request,
                &context,
                GrantSource::InteractiveHuman,
            )
            .expect("Session approval");

        let pending_input = json!({"path": pending_target});
        let pending_facets = [path_facet(ToolNature::Write, &pending_target)];
        let pending_request = PermissionRequest::new(
            "write",
            ToolProvenance::Native,
            &pending_facets,
            &pending_input,
        );
        let pending_proposal = state
            .propose(&pending_request, &context, GrantScope::Session)
            .expect("proposal");
        let old_id = state.session_id().expect("session identity");

        state.rebind_session().expect("rebind");

        assert_ne!(state.session_id().expect("session identity"), old_id);
        assert_eq!(state.grant_count().expect("grant count"), 0);
        assert_eq!(
            state
                .evaluate(&approved_request, &context)
                .expect("evaluation")
                .decision(),
            PermissionDecision::Ask
        );
        assert!(matches!(
            state.approve_session(
                pending_proposal,
                &pending_request,
                &context,
                GrantSource::InteractiveHuman,
            ),
            Err(GrantError::StaleApproval)
        ));
    }

    #[test]
    fn session_grants_do_not_cross_provider_or_session_identity() {
        let root = tempfile::tempdir().expect("tempdir");
        let target = root.path().join("provider-target.txt");
        std::fs::write(&target, b"test").expect("write fixture");
        let state = PermissionSessionState::new(PermissionEngine::with_workspace_root(
            root.path().to_path_buf(),
        ));
        let fresh = PermissionSessionState::new(PermissionEngine::with_workspace_root(
            root.path().to_path_buf(),
        ));
        let input = json!({"path": target});
        let facets = [path_facet(ToolNature::Write, &target)];
        let approved_provenance = ToolProvenance::McpRemote {
            server: "server-a".to_string(),
        };
        let approved = PermissionRequest::new("write", approved_provenance, &facets, &input);
        let context = context();
        let proposal = state
            .propose(&approved, &context, GrantScope::Session)
            .expect("proposal");
        state
            .approve_session(proposal, &approved, &context, GrantSource::InteractiveHuman)
            .expect("Session approval");
        let other_provider = ToolProvenance::McpRemote {
            server: "server-b".to_string(),
        };
        let collision = PermissionRequest::new("write", other_provider, &facets, &input);

        assert_eq!(
            state
                .evaluate(&collision, &context)
                .expect("evaluation")
                .decision(),
            PermissionDecision::Ask
        );
        assert_eq!(
            fresh
                .evaluate(&approved, &context)
                .expect("evaluation")
                .decision(),
            PermissionDecision::Ask
        );
    }

    #[test]
    fn multi_facet_compilation_is_atomic() {
        let root = tempfile::tempdir().expect("tempdir");
        let target = root.path().join("atomic-target.txt");
        std::fs::write(&target, b"test").expect("write fixture");
        let state = PermissionSessionState::new(PermissionEngine::with_workspace_root(
            root.path().to_path_buf(),
        ));
        let input = json!({"path": target, "command": "sentinel"});
        let facets = [
            path_facet(ToolNature::Write, &target),
            ToolPermissionFacet::new(ToolNature::Execute),
        ];
        let request = PermissionRequest::new("hybrid", ToolProvenance::Native, &facets, &input);

        assert!(matches!(
            state.propose(&request, &context(), GrantScope::Session),
            Err(GrantError::MissingResource(1))
        ));
        assert_eq!(state.grant_count().expect("grant count"), 0);
    }

    #[test]
    fn canonical_json_fingerprint_and_debug_views_are_redacted() {
        let root = tempfile::tempdir().expect("tempdir");
        let target = root.path().join("secret-sentinel.txt");
        std::fs::write(&target, b"test").expect("write fixture");
        let state = PermissionSessionState::new(PermissionEngine::with_workspace_root(
            root.path().to_path_buf(),
        ));
        let facets = [path_facet(ToolNature::Write, &target)];
        let first_input = serde_json::from_str(r#"{"b":2,"a":{"y":1,"x":0}}"#).expect("first JSON");
        let second_input =
            serde_json::from_str(r#"{"a":{"x":0,"y":1},"b":2}"#).expect("second JSON");
        let first = PermissionRequest::new(
            "secret-sentinel-tool",
            ToolProvenance::McpRemote {
                server: "secret-sentinel-provider".to_string(),
            },
            &facets,
            &first_input,
        );
        let second = PermissionRequest::new(
            "secret-sentinel-tool",
            ToolProvenance::McpRemote {
                server: "secret-sentinel-provider".to_string(),
            },
            &facets,
            &second_input,
        );
        let first_proposal = state
            .propose(&first, &context(), GrantScope::Session)
            .expect("first proposal");
        let second_proposal = state
            .propose(&second, &context(), GrantScope::Session)
            .expect("second proposal");

        assert_eq!(first_proposal.fingerprint, second_proposal.fingerprint);
        for debug in [
            format!("{first_proposal:?}"),
            format!("{:?}", first_proposal.preview()),
            format!("{:?}", first_proposal.preview().facets()),
        ] {
            assert!(!debug.contains("secret-sentinel"), "leaked Debug: {debug}");
        }
    }

    #[test]
    fn installed_grant_report_and_schema_are_redaction_safe() {
        let root = tempfile::tempdir().expect("tempdir");
        let target = root.path().join("report-secret-sentinel.txt");
        std::fs::write(&target, b"test").expect("write fixture");
        let state = PermissionSessionState::new(PermissionEngine::with_workspace_root(
            root.path().to_path_buf(),
        ));
        let input = json!({"path": target, "credential": "report-secret-sentinel"});
        let facets = [path_facet(ToolNature::Write, &target)];
        let request = PermissionRequest::new(
            "report-secret-sentinel-tool",
            ToolProvenance::McpRemote {
                server: "report-secret-sentinel-provider".to_string(),
            },
            &facets,
            &input,
        );
        let context = context();
        let proposal = state
            .propose(&request, &context, GrantScope::Session)
            .expect("proposal");
        state
            .approve_session(proposal, &request, &context, GrantSource::InteractiveHuman)
            .expect("Session approval");
        let evaluation = state.evaluate(&request, &context).expect("evaluation");
        let report_json = serde_json::to_value(evaluation.report()).expect("serialize report");
        let encoded = report_json.to_string();

        assert!(!encoded.contains("report-secret-sentinel"));
        assert_eq!(report_json["facets"][0]["source"]["kind"], "grant");
        assert_eq!(
            report_json["facets"][0]["source"]["grant_source"],
            "interactive_human"
        );
        assert_eq!(report_json["facets"][0]["reason"], "session_grant_allow");

        let schema = schemars::schema_for!(crate::PermissionDecisionReport);
        let schema_json = serde_json::to_string(&schema).expect("serialize schema");
        assert!(schema_json.contains("session_grant_allow"));
        assert!(schema_json.contains("interactive_human"));
        assert!(!schema_json.contains("report-secret-sentinel"));
    }

    #[test]
    fn publication_fence_blocks_observers_and_rebinds_only_on_success() {
        use std::sync::{Arc, mpsc};

        let state = Arc::new(PermissionSessionState::new(PermissionEngine::new()));
        let old_session = state.session_id().expect("old Session ID");
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let publishing = state.clone();
        let publisher = std::thread::spawn(move || {
            publishing
                .publish_and_rebind(|| {
                    entered_tx.send(()).expect("publish entered");
                    release_rx.recv().expect("publish release");
                    Ok::<(), ()>(())
                })
                .expect("permission fence")
                .expect("publication")
        });
        entered_rx.recv().expect("publication started");

        let (observed_tx, observed_rx) = mpsc::channel();
        let observing = state.clone();
        let observer = std::thread::spawn(move || {
            observed_tx
                .send(observing.session_id().expect("observed Session ID"))
                .expect("send observed ID");
        });
        assert!(
            observed_rx
                .recv_timeout(std::time::Duration::from_millis(25))
                .is_err(),
            "permission observers must not cross the publication fence"
        );
        release_tx.send(()).expect("release publication");
        publisher.join().expect("publisher thread");
        let new_session = observed_rx.recv().expect("new Session ID");
        observer.join().expect("observer thread");
        assert_ne!(new_session, old_session);

        let current = state.session_id().expect("current Session ID");
        let failed = state
            .publish_and_rebind(|| Err::<(), _>("route unavailable"))
            .expect("permission fence");
        assert_eq!(failed, Err("route unavailable"));
        assert_eq!(state.session_id().expect("retained Session ID"), current);
    }

    #[cfg(unix)]
    #[test]
    fn retargeted_symlink_does_not_reuse_exact_path_grant() {
        let root = tempfile::tempdir().expect("tempdir");
        let external = tempfile::tempdir().expect("tempdir");
        let first_target = external.path().join("first.txt");
        let second_target = external.path().join("second.txt");
        std::fs::write(&first_target, b"first").expect("write fixture");
        std::fs::write(&second_target, b"second").expect("write fixture");
        let link = root.path().join("selected.txt");
        std::os::unix::fs::symlink(&first_target, &link).expect("create symlink");
        let state = PermissionSessionState::new(PermissionEngine::with_workspace_root(
            root.path().to_path_buf(),
        ));
        let input = json!({"path": link});
        let facets = [path_facet(ToolNature::Write, &link)];
        let request = PermissionRequest::new("write", ToolProvenance::Native, &facets, &input);
        let context = context();
        let proposal = state
            .propose(&request, &context, GrantScope::Session)
            .expect("proposal");
        state
            .approve_session(proposal, &request, &context, GrantSource::InteractiveHuman)
            .expect("Session approval");

        std::fs::remove_file(&link).expect("remove symlink");
        std::os::unix::fs::symlink(&second_target, &link).expect("retarget symlink");

        assert_eq!(
            state
                .evaluate(&request, &context)
                .expect("evaluation")
                .decision(),
            PermissionDecision::Ask
        );
    }
}
