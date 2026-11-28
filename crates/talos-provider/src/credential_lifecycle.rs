//! Provider-neutral dynamic credential lifecycle. No transport, vendor login or config schema.
use std::{
    collections::{HashMap, HashSet},
    fmt,
    sync::{Arc, Mutex, MutexGuard},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{sync::watch, time::Instant};
use uuid::Uuid;

const SLOTS: usize = 64;
// Enrollment metadata is separate from the 64 live lease slots.
const ENROLLED_BINDINGS: usize = 256;
const PRODUCERS: usize = 16;
const PER_KEY_WAITERS: usize = 64;
const TOTAL_WAITERS: usize = 256;
const MATERIAL_BYTES: usize = 16 * 1024;
const MAX_TTL: Duration = Duration::from_secs(86_400);
const PRODUCER_TIMEOUT: Duration = Duration::from_secs(30);

/// Redacted lifecycle failures. No source error payload is retained.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum CredentialError {
    /// Ambiguous or inconsistent explicit source selection.
    #[error("invalid credential source configuration")]
    Configuration,
    /// Resolver capacity is exhausted.
    #[error("credential resolver capacity exhausted")]
    Capacity,
    /// Binding is no longer authorized in this context.
    #[error("credential binding revoked")]
    Revoked,
    /// Lease no longer matches the current usable revision.
    #[error("credential lease is stale")]
    Stale,
    /// Caller or acquisition deadline elapsed.
    #[error("credential deadline elapsed")]
    Deadline,
    /// Waiting was cancelled.
    #[error("credential acquisition cancelled")]
    Cancelled,
    /// Source failed, without retaining its diagnostic payload.
    #[error("credential acquisition failed")]
    Acquisition,
    /// Invalid or unusable expiry/clock sample.
    #[error("invalid credential expiry or clock")]
    Expiry,
    /// Invalid credential size.
    #[error("invalid credential material size")]
    Material,
    /// Explicit persistent enrollment requires an available approved store.
    #[error("credential store unavailable; unlock or configure an approved store")]
    StoreUnavailable,
    /// Another store operation for this profile is still in progress.
    #[error("credential store operation busy; retry after it completes")]
    StoreBusy,
    /// A shutdown deadline elapsed before all producers exited.
    #[error("credential resolver shutdown incomplete")]
    ShutdownIncomplete,
    /// Resolver cannot safely proceed.
    #[error("credential resolver unavailable")]
    Unavailable,
}
/// Result of a credential lifecycle operation.
pub type CredentialResult<T> = std::result::Result<T, CredentialError>;
type Result<T> = CredentialResult<T>;

/// Explicit source identity, not a secret or an endpoint supplied by a project.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Source {
    /// Legacy static key path.
    Static,
    /// Opaque enrolled dynamic source identity.
    Dynamic(Uuid),
}

/// The absence of auth preserves the existing config resolution path.
#[derive(Clone, Copy)]
pub enum SourceSelection<'a> {
    /// Auth omitted; legacy static/dispatch resolution is unchanged.
    Omitted,
    /// Explicit selectors; exactly one matching source must be present.
    Explicit(&'a [Source]),
}

/// Check explicit source selection before accessing any credential source.
///
/// Omitted auth is deliberately not inferred from ambient key availability.
/// The returned static selection uses the unchanged legacy config path.
pub fn admit_source(selection: SourceSelection<'_>, expected: Source) -> Result<Source> {
    match selection {
        SourceSelection::Omitted if expected == Source::Static => Ok(Source::Static),
        SourceSelection::Explicit([source]) if *source == expected => Ok(*source),
        _ => Err(CredentialError::Configuration),
    }
}

/// Trusted enrollment identities. The audience is an opaque identity for an exact approved
/// destination, not a URL or model alias. The caller owns destination policy (C).
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct BindingSpec {
    /// Stable enrolled profile.
    pub profile: Uuid,
    /// Opaque account/tenant identity.
    pub account: Uuid,
    /// Approved audience identity.
    pub audience: Uuid,
    /// Enrolled dynamic credential source.
    pub source: Uuid,
}

/// Resolver-issued binding for one configuration generation.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Binding {
    context: Uuid,
    spec: BindingSpec,
    generation: u64,
}

impl Binding {
    /// Non-secret enrollment metadata for the acquisition adapter.
    pub fn spec(&self) -> BindingSpec {
        self.spec
    }
}

/// Clock readings in seconds/subseconds from stable monotonic origin and Unix epoch.
#[derive(Clone, Copy)]
pub struct ClockSample {
    /// Monotonic elapsed time.
    pub monotonic: Duration,
    /// Wall time since Unix epoch.
    pub wall: Duration,
}

/// Trusted injectable clock; invalid readings must return a redacted error.
/// Sampling runs under the registry gate and must be fast, nonblocking and non-reentrant.
pub trait CredentialClock: Send + Sync {
    /// Sample both clocks.
    fn sample(&self) -> Result<ClockSample>;
}

struct SystemClock(std::time::Instant);

impl CredentialClock for SystemClock {
    fn sample(&self) -> Result<ClockSample> {
        Ok(ClockSample {
            monotonic: self.0.elapsed(),
            wall: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| CredentialError::Expiry)?,
        })
    }
}

/// Bounded secret material. Not serializable or cloneable.
pub struct CredentialMaterial(Vec<u8>);

impl CredentialMaterial {
    /// Copy bounded material from a source-owned buffer. Sources must bound their own input.
    pub fn new(bytes: &[u8]) -> Result<Self> {
        if bytes.is_empty() || bytes.len() > MATERIAL_BYTES {
            return Err(CredentialError::Material);
        }
        Ok(Self(bytes.to_vec()))
    }
}

impl fmt::Debug for CredentialMaterial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CredentialMaterial(***)")
    }
}

impl Drop for CredentialMaterial {
    fn drop(&mut self) {
        // Best effort only; no claim of physical erasure or allocator guarantees.
        self.0.fill(0);
    }
}

/// Dynamic acquisition output; expiry is mandatory.
pub struct AcquiredCredential {
    /// Bounded material.
    pub material: CredentialMaterial,
    /// Issuer expiry measured from acquisition start, if supplied.
    pub ttl: Option<Duration>,
    /// Absolute issuer expiry since Unix epoch, if supplied.
    pub absolute_expiry: Option<Duration>,
}

/// Cancellation signal passed to a producer. Dropped/closed signals are cancelled.
#[derive(Clone)]
pub struct Cancellation(watch::Receiver<bool>);

impl Cancellation {
    /// Observe whether cancellation has been requested.
    pub fn is_cancelled(&self) -> bool {
        *self.0.borrow() || self.0.has_changed().is_err()
    }

    /// Wait until cancellation.
    pub async fn cancelled(&mut self) {
        let _ = self.0.wait_for(|value| *value).await;
    }
}

/// Provider-specific acquisition seam. Implementations must honor cancellation and deadline,
/// bound temporary buffers, clean up child work, and never panic/log with secret material.
#[async_trait::async_trait]
pub trait CredentialSource: Send + Sync {
    /// Acquire material for an already admitted binding.
    async fn acquire(
        &self,
        binding: Binding,
        cancellation: Cancellation,
        deadline: Instant,
    ) -> Result<AcquiredCredential>;
}

/// Separate refresh material for explicit persistence; access leases cannot be written here.
/// Like access material, it is bounded, non-serializable and redacted in Debug.
#[derive(Debug)]
pub struct RefreshMaterial(CredentialMaterial);

impl RefreshMaterial {
    /// Construct from explicitly enrolled refresh material, not an access lease.
    pub fn new(bytes: &[u8]) -> Result<Self> {
        CredentialMaterial::new(bytes).map(Self)
    }

    /// Scoped access for a trusted acquisition adapter; do not retain or log copies.
    pub fn with_bytes<T>(&self, consume: impl FnOnce(&[u8]) -> T) -> T {
        consume(&self.0.0)
    }
}

/// Approved OS-store boundary; no filesystem fallback is supplied by this crate.
/// Implementations must be explicitly enrolled and return only redacted errors.
/// This synchronous seam may block: callers must use a blocking-capable execution context.
/// Writes already admitted before revocation may finish in the external store; the resolver
/// cannot roll back that effect. Durable refresh deletion and re-enrollment are separate
/// vendor/enrollment responsibilities, not access-lease revocation. A replacement enrollment
/// must use a new source identity or explicitly clean up the previous durable entry; this seam
/// does not automatically reload or erase persisted refresh material.
pub trait RefreshStore: Send + Sync {
    /// Read enrolled refresh material.
    fn read(&self, binding: BindingSpec) -> Result<RefreshMaterial>;
    /// Persist explicitly enrolled refresh material.
    fn write(&self, binding: BindingSpec, material: &[u8]) -> Result<()>;
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct Expiry {
    monotonic: Duration,
    wall: Duration,
}

impl Expiry {
    fn calculate(start: ClockSample, value: &AcquiredCredential) -> Result<Self> {
        let absolute = value
            .absolute_expiry
            .map(|expiry| {
                expiry
                    .checked_sub(start.wall)
                    .ok_or(CredentialError::Expiry)
            })
            .transpose()?;
        let span = match (value.ttl, absolute) {
            (Some(ttl), Some(absolute)) => ttl.min(absolute),
            (Some(ttl), None) => ttl,
            (None, Some(absolute)) => absolute,
            (None, None) => return Err(CredentialError::Expiry),
        }
        .min(MAX_TTL);
        let margin = (span / 10).clamp(Duration::from_secs(1), Duration::from_secs(60));
        let usable = span
            .checked_sub(margin)
            .filter(|duration| !duration.is_zero())
            .ok_or(CredentialError::Expiry)?;
        Ok(Self {
            monotonic: start
                .monotonic
                .checked_add(usable)
                .ok_or(CredentialError::Expiry)?,
            wall: start
                .wall
                .checked_add(usable)
                .ok_or(CredentialError::Expiry)?,
        })
    }

    fn usable(self, now: ClockSample) -> bool {
        now.monotonic < self.monotonic && now.wall < self.wall
    }
}

/// Opaque non-serializable lease identity. Contains no credential material.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct LeaseRef {
    binding: Binding,
    revision: u64,
    expiry: Expiry,
}

impl fmt::Debug for LeaseRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("LeaseRef(<opaque>)")
    }
}

/// Result of a generation/revision checked invalidation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Invalidation {
    /// Current revision was removed.
    Invalidated,
    /// Another revision is current, or the old revision is absent.
    Stale,
    /// Binding is revoked.
    Revoked,
}

type FlightResult = Option<Result<LeaseRef>>;

struct Flight {
    id: u64,
    waiters: usize,
    result: watch::Sender<FlightResult>,
}

enum Entry {
    Flight(Flight),
    Cached(LeaseRef, CredentialMaterial),
}

struct Enrollment {
    generation: u64,
    bindings: HashSet<BindingSpec>,
}

struct State {
    closed: bool,
    sequence: u64,
    last_monotonic: Option<Duration>,
    profiles: HashMap<Uuid, Enrollment>,
    // One admitted synchronous store operation per profile, including revoked profiles
    // until the actual operation returns. Never held as a mutex across store code.
    store_operations: HashSet<Uuid>,
    entries: HashMap<Binding, Entry>,
    // Includes retired flights until their actual producer task has joined.
    producers: HashMap<u64, watch::Sender<bool>>,
    waiters: usize,
}

impl State {
    fn next_id(&mut self) -> Result<u64> {
        self.sequence = self
            .sequence
            .checked_add(1)
            .ok_or(CredentialError::Unavailable)?;
        Ok(self.sequence)
    }

    fn retire(&mut self, binding: Binding, error: CredentialError) {
        if let Some(Entry::Flight(flight)) = self.entries.remove(&binding) {
            flight.result.send_replace(Some(Err(error)));
            if let Some(cancel) = self.producers.get(&flight.id) {
                cancel.send_replace(true);
            }
        }
    }

    fn retire_flight(&mut self, binding: Binding, id: u64, error: CredentialError) {
        if matches!(self.entries.get(&binding), Some(Entry::Flight(f)) if f.id == id) {
            self.retire(binding, error);
        }
    }

    fn observe(&mut self, now: ClockSample) -> Result<ClockSample> {
        if self.last_monotonic.is_some_and(|last| now.monotonic < last) {
            self.retire_all(CredentialError::Expiry);
            return Err(CredentialError::Expiry);
        }
        self.last_monotonic = Some(now.monotonic);
        self.prune(now);
        Ok(now)
    }

    fn retire_all(&mut self, error: CredentialError) {
        let keys: Vec<_> = self.entries.keys().copied().collect();
        for key in keys {
            self.retire(key, error);
        }
    }

    fn prune(&mut self, now: ClockSample) {
        self.entries.retain(|_, entry| match entry {
            Entry::Flight(_) => true,
            Entry::Cached(lease, _) => lease.expiry.usable(now),
        });
    }
}

/// Explicit owner of one provider credential lifecycle for a host or SDK composition.
/// Share resolver handles among the providers of this context, not across independent
/// hosts. No default RuntimeBuilder, config or vendor route is changed.
pub struct CredentialContext {
    resolver: Arc<CredentialResolver>,
}

impl CredentialContext {
    /// Create a lifecycle owner with a trusted acquisition adapter and system clock.
    pub fn new(source: Arc<dyn CredentialSource>) -> Self {
        Self {
            resolver: CredentialResolver::new(source),
        }
    }

    /// Create a lifecycle owner with an injected clock.
    pub fn with_clock(source: Arc<dyn CredentialSource>, clock: Arc<dyn CredentialClock>) -> Self {
        Self {
            resolver: CredentialResolver::with_clock(source, clock),
        }
    }

    /// Share the same resolver with a provider in this host context.
    pub fn resolver(&self) -> Arc<CredentialResolver> {
        Arc::clone(&self.resolver)
    }

    /// Close admission and await producer termination; call before shutting down the runtime.
    pub async fn shutdown(&self, deadline: Instant) -> Result<()> {
        self.resolver.shutdown(deadline).await
    }
}

impl Drop for CredentialContext {
    fn drop(&mut self) {
        // Drop can revoke and request cancellation, but cannot join an async producer.
        // The explicit shutdown method is the normal bounded lifecycle gate.
        let _ = self.resolver.close();
    }
}

/// One resolver per explicit host/factory lifecycle context. No process-global singleton.
/// Hosts must shut down the context before discarding it; sources must cooperate with shutdown.
pub struct CredentialResolver {
    identity: Uuid,
    state: Mutex<State>,
    source: Arc<dyn CredentialSource>,
    clock: Arc<dyn CredentialClock>,
    producer_count: watch::Sender<usize>,
}

impl CredentialResolver {
    fn new(source: Arc<dyn CredentialSource>) -> Arc<Self> {
        Self::with_clock(source, Arc::new(SystemClock(std::time::Instant::now())))
    }

    fn with_clock(source: Arc<dyn CredentialSource>, clock: Arc<dyn CredentialClock>) -> Arc<Self> {
        Arc::new(Self {
            identity: Uuid::new_v4(),
            state: Mutex::new(State {
                closed: false,
                sequence: 0,
                last_monotonic: None,
                profiles: HashMap::new(),
                store_operations: HashSet::new(),
                entries: HashMap::new(),
                producers: HashMap::new(),
                waiters: 0,
            }),
            source,
            clock,
            producer_count: watch::channel(0).0,
        })
    }

    fn lock(&self) -> Result<MutexGuard<'_, State>> {
        self.state.lock().map_err(|_| CredentialError::Unavailable)
    }

    fn observe_clock(&self, state: &mut State) -> Result<ClockSample> {
        match self.clock.sample() {
            Ok(now) => state.observe(now),
            Err(_) => {
                state.retire_all(CredentialError::Expiry);
                Err(CredentialError::Expiry)
            }
        }
    }

    fn check(&self, state: &State, binding: Binding) -> Result<()> {
        if state.closed
            || binding.context != self.identity
            || !state
                .profiles
                .get(&binding.spec.profile)
                .is_some_and(|profile| {
                    profile.generation == binding.generation
                        && profile.bindings.contains(&binding.spec)
                })
        {
            return Err(CredentialError::Revoked);
        }
        Ok(())
    }

    /// Register or replace the trusted binding inventory of an enrolled profile.
    /// Replacement revokes all prior bindings atomically. Enrollment authority is
    /// supplied by the host outside project input; this is not destination policy.
    pub fn replace_profile(&self, profile: Uuid, bindings: &[BindingSpec]) -> Result<()> {
        if bindings.is_empty() || bindings.iter().any(|binding| binding.profile != profile) {
            return Err(CredentialError::Configuration);
        }
        if bindings.len() > ENROLLED_BINDINGS {
            return Err(CredentialError::Capacity);
        }
        let bindings_set: HashSet<_> = bindings.iter().copied().collect();
        if bindings_set.len() != bindings.len() {
            return Err(CredentialError::Configuration);
        }
        let mut state = self.lock()?;
        if state.closed {
            return Err(CredentialError::Revoked);
        }
        if !state.profiles.contains_key(&profile) && state.profiles.len() >= SLOTS {
            return Err(CredentialError::Capacity);
        }
        let other_bindings: usize = state
            .profiles
            .iter()
            .filter(|(id, _)| **id != profile)
            .map(|(_, enrollment)| enrollment.bindings.len())
            .sum();
        if other_bindings + bindings_set.len() > ENROLLED_BINDINGS {
            return Err(CredentialError::Capacity);
        }
        let generation = state.next_id()?;
        Self::retire_profile(&mut state, profile);
        state.profiles.insert(
            profile,
            Enrollment {
                generation,
                bindings: bindings_set,
            },
        );
        Ok(())
    }

    fn retire_profile(state: &mut State, profile: Uuid) {
        let keys: Vec<_> = state
            .entries
            .keys()
            .filter(|key| key.spec.profile == profile)
            .copied()
            .collect();
        for key in keys {
            state.retire(key, CredentialError::Revoked);
        }
        state.profiles.remove(&profile);
    }

    /// Revoke all bindings for an enrolled profile. Later enrollment uses a new generation.
    pub fn revoke_profile(&self, profile: Uuid) -> Result<()> {
        let mut state = self.lock()?;
        Self::retire_profile(&mut state, profile);
        Ok(())
    }

    /// Validate explicit source admission against the trusted enrollment inventory.
    /// No acquisition or secret-store access occurs here.
    pub fn bind(&self, spec: BindingSpec, selection: SourceSelection<'_>) -> Result<Binding> {
        admit_source(selection, Source::Dynamic(spec.source))?;
        let state = self.lock()?;
        if state.closed {
            return Err(CredentialError::Revoked);
        }
        let enrollment = state
            .profiles
            .get(&spec.profile)
            .ok_or(CredentialError::Configuration)?;
        if !enrollment.bindings.contains(&spec) {
            return Err(CredentialError::Configuration);
        }
        Ok(Binding {
            context: self.identity,
            spec,
            generation: enrollment.generation,
        })
    }

    // Admission and result publication are generation-checked under the registry lock.
    // External code runs without that lock. A per-profile reservation prevents reentrant
    // or concurrent operations against the same persistent enrollment, without making
    // revoke/replace or other lease operations wait for synchronous I/O.
    fn store_operation<T>(
        &self,
        binding: Binding,
        operation: impl FnOnce() -> Result<T>,
    ) -> Result<T> {
        {
            let mut state = self.lock()?;
            self.check(&state, binding)?;
            if state.store_operations.contains(&binding.spec.profile) {
                return Err(CredentialError::StoreBusy);
            }
            if state.store_operations.len() >= SLOTS {
                return Err(CredentialError::Capacity);
            }
            state.store_operations.insert(binding.spec.profile);
        }
        let _reservation = StoreReservation {
            resolver: self,
            profile: binding.spec.profile,
        };
        let result = operation().map_err(|_| CredentialError::StoreUnavailable);
        let state = self.lock()?;
        self.check(&state, binding)?;
        result
    }

    /// Read from an explicitly selected approved store, after binding validation.
    /// No fallback, UI prompt or file persistence. A read racing revocation is
    /// discarded and returns Revoked. Another operation for this profile returns StoreBusy.
    pub fn read_refresh(
        &self,
        binding: Binding,
        store: &dyn RefreshStore,
    ) -> Result<RefreshMaterial> {
        self.store_operation(binding, || store.read(binding.spec))
    }

    /// Write explicitly enrolled refresh material to an approved store only.
    /// No fallback or silent downgrade if the store is unavailable.
    /// An already admitted write may physically finish after revocation; its result
    /// is rejected, not rolled back. A new generation cannot enter this profile's
    /// store until the old operation returns. Durable enrollment cleanup is external.
    pub fn write_refresh(
        &self,
        binding: Binding,
        store: &dyn RefreshStore,
        material: &RefreshMaterial,
    ) -> Result<()> {
        self.store_operation(binding, || {
            material.with_bytes(|bytes| store.write(binding.spec, bytes))
        })
    }

    // Check at the actual admission boundary, not just before a potentially
    // contended registry lock or an injected clock sample.
    fn check_admission(deadline: Instant, cancellation: Option<&Cancellation>) -> Result<()> {
        if cancellation.is_some_and(Cancellation::is_cancelled) {
            return Err(CredentialError::Cancelled);
        }
        if Instant::now() >= deadline {
            return Err(CredentialError::Deadline);
        }
        Ok(())
    }

    /// Acquire a cached or single-flight lease. Dropping the future detaches the waiter.
    /// Caller cancellation does not cancel other waiters.
    pub async fn acquire(
        self: &Arc<Self>,
        binding: Binding,
        deadline: Instant,
        mut cancellation: Cancellation,
    ) -> Result<LeaseRef> {
        Self::check_admission(deadline, Some(&cancellation))?;
        let (id, mut receiver) = {
            let mut state = self.lock()?;
            Self::check_admission(deadline, Some(&cancellation))?;
            self.check(&state, binding)?;
            let observed = self.observe_clock(&mut state);
            Self::check_admission(deadline, Some(&cancellation))?;
            let now = observed?;
            if let Some(Entry::Cached(lease, _)) = state.entries.get(&binding) {
                return Ok(*lease);
            }
            if state.waiters >= TOTAL_WAITERS {
                return Err(CredentialError::Capacity);
            }
            let (id, receiver) =
                if let Some(Entry::Flight(flight)) = state.entries.get_mut(&binding) {
                    if flight.waiters >= PER_KEY_WAITERS {
                        return Err(CredentialError::Capacity);
                    }
                    flight.waiters += 1;
                    (flight.id, flight.result.subscribe())
                } else {
                    if state.entries.len() >= SLOTS || state.producers.len() >= PRODUCERS {
                        return Err(CredentialError::Capacity);
                    }
                    let runtime = tokio::runtime::Handle::try_current()
                        .map_err(|_| CredentialError::Unavailable)?;
                    let id = state.next_id()?;
                    let (result, receiver) = watch::channel(None);
                    let (cancel, signal) = watch::channel(false);
                    state.entries.insert(
                        binding,
                        Entry::Flight(Flight {
                            id,
                            waiters: 1,
                            result,
                        }),
                    );
                    state.producers.insert(id, cancel);
                    self.producer_count.send_replace(state.producers.len());
                    let resolver = Arc::clone(self);
                    let producer_deadline = Instant::now() + PRODUCER_TIMEOUT;
                    runtime.spawn(async move {
                        resolver
                            .produce(binding, id, now, Cancellation(signal), producer_deadline)
                            .await;
                    });
                    (id, receiver)
                };
            state.waiters += 1;
            (id, receiver)
        };
        let _waiter = Waiter {
            resolver: Arc::clone(self),
            binding,
            id,
        };
        let result = tokio::select! {
            biased;
            () = cancellation.cancelled() => Err(CredentialError::Cancelled),
            () = tokio::time::sleep_until(deadline) => Err(CredentialError::Deadline),
            result = receiver.wait_for(Option::is_some) => {
                result.map_err(|_| CredentialError::Unavailable)?
                    .as_ref().copied().ok_or(CredentialError::Unavailable)?
            }
        }?;
        {
            let mut state = self.lock()?;
            self.current(&mut state, binding, result, deadline, Some(&cancellation))?;
        }
        Ok(result)
    }

    async fn produce(
        self: Arc<Self>,
        binding: Binding,
        id: u64,
        start: ClockSample,
        cancellation: Cancellation,
        deadline: Instant,
    ) {
        let source = Arc::clone(&self.source);
        // The supervisor retains the registry permit until the actual producer has joined.
        // A retired identity can never publish, even if its source ignores cancellation.
        let mut task =
            tokio::spawn(async move { source.acquire(binding, cancellation, deadline).await });
        let joined = tokio::select! {
            result = &mut task => result,
            () = tokio::time::sleep_until(deadline) => {
                if let Ok(mut state) = self.lock() {
                    state.retire_flight(binding, id, CredentialError::Deadline);
                }
                task.await
            }
        };
        let Ok(mut state) = self.lock() else { return };
        state.producers.remove(&id);
        self.producer_count.send_replace(state.producers.len());
        if !matches!(state.entries.get(&binding), Some(Entry::Flight(f)) if f.id == id) {
            return;
        }
        let result = (|| {
            self.check(&state, binding)?;
            if Instant::now() >= deadline {
                return Err(CredentialError::Deadline);
            }
            let acquired = joined.map_err(|_| CredentialError::Acquisition)??;
            let expiry = Expiry::calculate(start, &acquired)?;
            if !expiry.usable(self.observe_clock(&mut state)?) {
                return Err(CredentialError::Expiry);
            }
            Ok((
                LeaseRef {
                    binding,
                    revision: id,
                    expiry,
                },
                acquired.material,
            ))
        })();
        if let Some(Entry::Flight(flight)) = state.entries.remove(&binding) {
            match result {
                Ok((lease, material)) => {
                    state
                        .entries
                        .insert(binding, Entry::Cached(lease, material));
                    flight.result.send_replace(Some(Ok(lease)));
                }
                Err(error) => {
                    flight.result.send_replace(Some(Err(error)));
                }
            }
        }
    }

    fn detach(&self, binding: Binding, id: u64) {
        let Ok(mut state) = self.lock() else { return };
        state.waiters -= 1;
        if let Some(Entry::Flight(flight)) = state.entries.get_mut(&binding)
            && flight.id == id
        {
            flight.waiters -= 1;
            if flight.waiters == 0 {
                state.retire_flight(binding, id, CredentialError::Cancelled);
            }
        }
    }

    fn current<'a>(
        &self,
        state: &'a mut State,
        binding: Binding,
        lease: LeaseRef,
        deadline: Instant,
        cancellation: Option<&Cancellation>,
    ) -> Result<&'a CredentialMaterial> {
        Self::check_admission(deadline, cancellation)?;
        self.check(state, binding)?;
        let observed = self.observe_clock(state);
        Self::check_admission(deadline, cancellation)?;
        observed?;
        match state.entries.get(&binding) {
            Some(Entry::Cached(current, material)) if *current == lease => Ok(material),
            _ => Err(CredentialError::Stale),
        }
    }

    /// Atomic binding/generation/revision/expiry check at dispatch admission.
    /// C owns final sealed-request authorization and cancellation of admitted transport.
    pub fn validate_current(
        &self,
        binding: Binding,
        lease: LeaseRef,
        deadline: Instant,
    ) -> Result<()> {
        let mut state = self.lock()?;
        self.current(&mut state, binding, lease, deadline, None)
            .map(|_| ())
    }

    /// Scoped synchronous access for a trusted authorizer. No await/reentry in the callback;
    /// the caller must not retain or log copies beyond its bounded authorization operation.
    pub fn with_current_credential<T>(
        &self,
        binding: Binding,
        lease: LeaseRef,
        deadline: Instant,
        consume: impl FnOnce(&[u8]) -> T,
    ) -> Result<T> {
        let mut state = self.lock()?;
        let material = self.current(&mut state, binding, lease, deadline, None)?;
        Ok(consume(&material.0))
    }

    /// Invalidate only the exact current generation/revision, never a replacement lease.
    pub fn invalidate_if_current(&self, binding: Binding, lease: LeaseRef) -> Result<Invalidation> {
        let mut state = self.lock()?;
        if self.check(&state, binding).is_err() {
            return Ok(Invalidation::Revoked);
        }
        self.observe_clock(&mut state)?;
        if matches!(state.entries.get(&binding), Some(Entry::Cached(current, _)) if *current == lease)
        {
            state.entries.remove(&binding);
            Ok(Invalidation::Invalidated)
        } else {
            Ok(Invalidation::Stale)
        }
    }

    fn close(&self) -> Result<()> {
        let mut state = self.lock()?;
        state.closed = true;
        state.profiles.clear();
        let keys: Vec<_> = state.entries.keys().copied().collect();
        for key in keys {
            state.retire(key, CredentialError::Revoked);
        }
        for cancellation in state.producers.values() {
            cancellation.send_replace(true);
        }
        Ok(())
    }

    /// Close admission, revoke all leases, cancel producers and await actual termination.
    /// On deadline failure the resolver stays closed and still tracks unjoined producers.
    pub async fn shutdown(&self, deadline: Instant) -> Result<()> {
        let mut count = self.producer_count.subscribe();
        self.close()?;
        tokio::time::timeout_at(deadline, count.wait_for(|count| *count == 0))
            .await
            .map_err(|_| CredentialError::ShutdownIncomplete)?
            .map_err(|_| CredentialError::Unavailable)?;
        Ok(())
    }
}

/// Create a caller-owned cancellation sender and read-only signal.
pub fn cancellation_pair() -> (watch::Sender<bool>, Cancellation) {
    let (sender, receiver) = watch::channel(false);
    (sender, Cancellation(receiver))
}

struct StoreReservation<'a> {
    resolver: &'a CredentialResolver,
    profile: Uuid,
}

impl Drop for StoreReservation<'_> {
    fn drop(&mut self) {
        if let Ok(mut state) = self.resolver.lock() {
            state.store_operations.remove(&self.profile);
        }
    }
}

struct Waiter {
    resolver: Arc<CredentialResolver>,
    binding: Binding,
    id: u64,
}

impl Drop for Waiter {
    fn drop(&mut self) {
        self.resolver.detach(self.binding, self.id);
    }
}

#[cfg(test)]
mod tests;
