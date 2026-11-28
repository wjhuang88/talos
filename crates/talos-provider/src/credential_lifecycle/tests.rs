use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::{mpsc, oneshot};

const CANARY: &[u8] = b"synthetic-lifecycle-canary";

fn spec(profile: Uuid) -> BindingSpec {
    BindingSpec {
        profile,
        account: Uuid::new_v4(),
        audience: Uuid::new_v4(),
        source: Uuid::new_v4(),
    }
}

fn bind(resolver: &CredentialResolver, spec: BindingSpec) -> Binding {
    resolver
        .bind(
            spec,
            SourceSelection::Explicit(&[Source::Dynamic(spec.source)]),
        )
        .expect("admitted binding")
}

fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(20)
}

fn material() -> AcquiredCredential {
    AcquiredCredential {
        material: CredentialMaterial::new(CANARY).expect("bounded synthetic material"),
        ttl: Some(Duration::from_secs(100)),
        absolute_expiry: None,
    }
}

#[derive(Default)]
struct Counters {
    running: AtomicUsize,
    peak: AtomicUsize,
    calls: AtomicUsize,
}

struct Running(Arc<Counters>);

impl Drop for Running {
    fn drop(&mut self) {
        self.0.running.fetch_sub(1, Ordering::SeqCst);
    }
}

struct Started {
    release: oneshot::Sender<()>,
    cancellation: Cancellation,
}

struct DelayedSource {
    counters: Arc<Counters>,
    started: mpsc::UnboundedSender<Started>,
}

#[async_trait::async_trait]
impl CredentialSource for DelayedSource {
    async fn acquire(
        &self,
        _binding: Binding,
        cancellation: Cancellation,
        _deadline: Instant,
    ) -> Result<AcquiredCredential> {
        self.counters.calls.fetch_add(1, Ordering::SeqCst);
        let running = self.counters.running.fetch_add(1, Ordering::SeqCst) + 1;
        self.counters.peak.fetch_max(running, Ordering::SeqCst);
        let _running = Running(Arc::clone(&self.counters));
        let (release, wait) = oneshot::channel();
        self.started
            .send(Started {
                release,
                cancellation,
            })
            .map_err(|_| CredentialError::Acquisition)?;
        // Deliberately delay cancellation until the explicit exit barrier.
        wait.await.map_err(|_| CredentialError::Acquisition)?;
        Ok(material())
    }
}

struct SampleGate {
    skip: usize,
    entered: oneshot::Sender<()>,
    release: std::sync::mpsc::Receiver<()>,
}

struct FakeClock {
    sample: Mutex<ClockSample>,
    gate: Mutex<Option<SampleGate>>,
}

impl FakeClock {
    fn set(&self, monotonic: u64, wall: u64) {
        *self.sample.lock().expect("clock lock") = ClockSample {
            monotonic: Duration::from_secs(monotonic),
            wall: Duration::from_secs(wall),
        };
    }

    fn pause_sample(&self, skip: usize) -> (oneshot::Receiver<()>, std::sync::mpsc::Sender<()>) {
        let (entered, entry) = oneshot::channel();
        let (release, exit) = std::sync::mpsc::channel();
        let previous = self.gate.lock().expect("gate lock").replace(SampleGate {
            skip,
            entered,
            release: exit,
        });
        assert!(previous.is_none());
        (entry, release)
    }
}

impl CredentialClock for FakeClock {
    fn sample(&self) -> Result<ClockSample> {
        let gate = {
            let mut slot = self.gate.lock().expect("gate lock");
            match slot.as_mut() {
                Some(gate) if gate.skip > 0 => {
                    gate.skip -= 1;
                    None
                }
                _ => slot.take(),
            }
        };
        if let Some(gate) = gate {
            gate.entered.send(()).expect("sample entered");
            gate.release
                .recv_timeout(Duration::from_secs(10))
                .expect("sample released");
        }
        Ok(*self.sample.lock().expect("clock lock"))
    }
}

type Fixture = (
    Arc<CredentialResolver>,
    Arc<Counters>,
    mpsc::UnboundedReceiver<Started>,
    Arc<FakeClock>,
    Binding,
);

fn fixture() -> Fixture {
    let counters = Arc::new(Counters::default());
    let (started, receiver) = mpsc::unbounded_channel();
    let clock = Arc::new(FakeClock {
        sample: Mutex::new(ClockSample {
            monotonic: Duration::from_secs(100),
            wall: Duration::from_secs(1000),
        }),
        gate: Mutex::new(None),
    });
    let resolver = CredentialResolver::with_clock(
        Arc::new(DelayedSource {
            counters: Arc::clone(&counters),
            started,
        }),
        clock.clone(),
    );
    let spec = spec(Uuid::new_v4());
    resolver
        .replace_profile(spec.profile, &[spec])
        .expect("enroll");
    let binding = bind(&resolver, spec);
    (resolver, counters, receiver, clock, binding)
}

async fn start(
    resolver: &Arc<CredentialResolver>,
    binding: Binding,
    started: &mut mpsc::UnboundedReceiver<Started>,
) -> (
    watch::Sender<bool>,
    tokio::task::JoinHandle<Result<LeaseRef>>,
    Started,
) {
    let (cancel, signal) = cancellation_pair();
    let resolver = Arc::clone(resolver);
    let waiter = tokio::spawn(async move { resolver.acquire(binding, deadline(), signal).await });
    let producer = tokio::time::timeout_at(deadline(), started.recv())
        .await
        .expect("producer enters")
        .expect("producer event");
    (cancel, waiter, producer)
}

async fn install(
    resolver: &Arc<CredentialResolver>,
    binding: Binding,
    started: &mut mpsc::UnboundedReceiver<Started>,
) -> LeaseRef {
    let (_cancel, waiter, producer) = start(resolver, binding, started).await;
    producer.release.send(()).expect("release producer");
    waiter.await.expect("join waiter").expect("installed lease")
}

async fn wait_count(resolver: &CredentialResolver, count: usize) {
    let mut receiver = resolver.producer_count.subscribe();
    tokio::time::timeout_at(deadline(), receiver.wait_for(|value| *value == count))
        .await
        .expect("actual producer join")
        .expect("count");
}

#[derive(Default)]
struct FakeStore(AtomicUsize);

impl RefreshStore for FakeStore {
    fn read(&self, _binding: BindingSpec) -> Result<RefreshMaterial> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Err(CredentialError::Acquisition)
    }

    fn write(&self, _binding: BindingSpec, _material: &[u8]) -> Result<()> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Err(CredentialError::Acquisition)
    }
}

#[tokio::test]
async fn explicit_conflicts_rejected_before_store_acquisition_or_dispatch() {
    let (resolver, counters, mut started, _clock, valid) = fixture();
    let store = FakeStore::default();
    let outbound = AtomicUsize::new(0);
    let spec = valid.spec();
    let dynamic = Source::Dynamic(spec.source);
    let wrong = Source::Dynamic(Uuid::new_v4());
    let mismatched = BindingSpec {
        source: Uuid::new_v4(),
        ..spec
    };
    for (requested, sources) in [
        (spec, vec![dynamic, wrong]),
        (spec, vec![dynamic, Source::Static]),
        (spec, vec![wrong]),
        (spec, vec![]),
        (mismatched, vec![Source::Dynamic(mismatched.source)]),
    ] {
        // All effects are downstream of admission; no store or transport is called on failure.
        let result = async {
            let binding = resolver.bind(requested, SourceSelection::Explicit(&sources))?;
            resolver.read_refresh(binding, &store)?;
            let (_cancel, signal) = cancellation_pair();
            let lease = resolver.acquire(binding, deadline(), signal).await?;
            resolver.validate_current(binding, lease, deadline())?;
            outbound.fetch_add(1, Ordering::SeqCst);
            Ok::<(), CredentialError>(())
        }
        .await;
        assert_eq!(result, Err(CredentialError::Configuration));
    }
    assert_eq!(store.0.load(Ordering::SeqCst), 0);
    assert_eq!(counters.calls.load(Ordering::SeqCst), 0);
    assert_eq!(outbound.load(Ordering::SeqCst), 0);
    assert!(matches!(
        admit_source(SourceSelection::Omitted, Source::Static),
        Ok(Source::Static)
    ));
    assert!(matches!(
        resolver.bind(spec, SourceSelection::Omitted),
        Err(CredentialError::Configuration)
    ));
    // Single explicit source: headless supplied fake acquisition is allowed without a store.
    let lease = install(&resolver, valid, &mut started).await;
    assert!(resolver.validate_current(valid, lease, deadline()).is_ok());
    assert_eq!(store.0.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn retired_producers_hold_capacity_until_actual_exit() {
    let (resolver, counters, mut started, _clock, binding) = fixture();
    let other_spec = spec(binding.spec().profile);
    resolver
        .replace_profile(binding.spec().profile, &[binding.spec(), other_spec])
        .expect("enroll distinct keys");
    let binding = bind(&resolver, binding.spec());
    let other = bind(&resolver, other_spec);
    let mut retired = Vec::new();
    for expected in 1..=PRODUCERS {
        let (cancel, waiter, producer) = start(&resolver, binding, &mut started).await;
        cancel.send_replace(true);
        assert_eq!(
            waiter.await.expect("join waiter"),
            Err(CredentialError::Cancelled)
        );
        assert!(producer.cancellation.is_cancelled());
        assert_eq!(counters.running.load(Ordering::SeqCst), expected);
        assert_eq!(resolver.lock().expect("state").producers.len(), expected);
        retired.push(producer);
    }
    for _ in 0..4 {
        let (_cancel, signal) = cancellation_pair();
        assert_eq!(
            resolver.acquire(binding, deadline(), signal).await,
            Err(CredentialError::Capacity)
        );
        let (_cancel, signal) = cancellation_pair();
        assert_eq!(
            resolver.acquire(other, deadline(), signal).await,
            Err(CredentialError::Capacity)
        );
        assert_eq!(counters.running.load(Ordering::SeqCst), PRODUCERS);
    }
    // Replacing a profile also cannot recover a permit from a retired running task.
    resolver
        .replace_profile(binding.spec().profile, &[binding.spec()])
        .expect("replace");
    let binding = bind(&resolver, binding.spec());
    for _ in 0..4 {
        retired
            .pop()
            .expect("retired producer")
            .release
            .send(())
            .expect("release");
        wait_count(&resolver, PRODUCERS - 1).await;
        assert_eq!(counters.running.load(Ordering::SeqCst), PRODUCERS - 1);
        let (cancel, waiter, producer) = start(&resolver, binding, &mut started).await;
        cancel.send_replace(true);
        assert_eq!(waiter.await.expect("join"), Err(CredentialError::Cancelled));
        retired.push(producer);
        assert_eq!(counters.running.load(Ordering::SeqCst), PRODUCERS);
        assert_eq!(counters.peak.load(Ordering::SeqCst), PRODUCERS);
    }
    assert_eq!(
        resolver.shutdown(Instant::now()).await,
        Err(CredentialError::ShutdownIncomplete)
    );
    for producer in retired {
        assert!(producer.cancellation.is_cancelled());
        producer.release.send(()).expect("release retired producer");
    }
    resolver
        .shutdown(deadline())
        .await
        .expect("joined shutdown");
    assert_eq!(counters.running.load(Ordering::SeqCst), 0);
    assert_eq!(counters.peak.load(Ordering::SeqCst), PRODUCERS);
    let state = resolver.lock().expect("state");
    assert!(state.entries.is_empty());
    assert!(state.producers.is_empty());
    assert_eq!(state.waiters, 0);
}

#[tokio::test]
async fn late_401_cannot_invalidate_new_revision_or_admit_old_request() {
    let (resolver, counters, mut started, clock, binding) = fixture();
    let l1 = install(&resolver, binding, &mut started).await;
    clock.set(190, 1090); // exact safety boundary of 100s TTL.
    let l2 = install(&resolver, binding, &mut started).await;
    assert_ne!(l1, l2);
    assert_eq!(
        resolver.invalidate_if_current(binding, l1),
        Ok(Invalidation::Stale)
    );
    assert_eq!(
        resolver.validate_current(binding, l1, deadline()),
        Err(CredentialError::Stale)
    );
    assert_eq!(resolver.validate_current(binding, l2, deadline()), Ok(()));
    let (_cancel, signal) = cancellation_pair();
    assert_eq!(resolver.acquire(binding, deadline(), signal).await, Ok(l2));
    assert_eq!(counters.calls.load(Ordering::SeqCst), 2);
    assert_eq!(
        resolver.invalidate_if_current(binding, l2),
        Ok(Invalidation::Invalidated)
    );
    assert_eq!(
        resolver.invalidate_if_current(binding, l2),
        Ok(Invalidation::Stale)
    );
}

#[tokio::test]
async fn same_key_waiters_share_flight_and_one_waiter_can_cancel() {
    let (resolver, counters, mut started, _clock, binding) = fixture();
    let (first_cancel, first, producer) = start(&resolver, binding, &mut started).await;
    let (_second_cancel, signal) = cancellation_pair();
    let mut second = std::pin::pin!(resolver.acquire(binding, deadline(), signal));
    assert!(futures_util::poll!(&mut second).is_pending());
    first_cancel.send_replace(true);
    assert_eq!(first.await.expect("join"), Err(CredentialError::Cancelled));
    assert!(!producer.cancellation.is_cancelled());
    producer.release.send(()).expect("release");
    let lease = second.await.expect("remaining waiter succeeds");
    assert_eq!(counters.calls.load(Ordering::SeqCst), 1);
    let (_cancel, signal) = cancellation_pair();
    assert_eq!(
        resolver.acquire(binding, deadline(), signal).await,
        Ok(lease)
    );
}

#[tokio::test]
async fn revoked_producer_cannot_publish_into_new_generation() {
    let (resolver, _counters, mut started, _clock, binding) = fixture();
    let (_cancel, waiter, producer) = start(&resolver, binding, &mut started).await;
    resolver
        .revoke_profile(binding.spec().profile)
        .expect("logout");
    assert_eq!(waiter.await.expect("join"), Err(CredentialError::Revoked));
    resolver
        .replace_profile(binding.spec().profile, &[binding.spec()])
        .expect("enroll");
    let replacement = bind(&resolver, binding.spec());
    let lease = install(&resolver, replacement, &mut started).await;
    producer.release.send(()).expect("late success");
    wait_count(&resolver, 0).await;
    assert_eq!(
        resolver.validate_current(replacement, lease, deadline()),
        Ok(())
    );
    assert_eq!(
        resolver.validate_current(binding, lease, deadline()),
        Err(CredentialError::Revoked)
    );
}

#[tokio::test]
async fn forward_wall_jump_retires_lease_and_backward_jump_never_resurrects() {
    let (resolver, _counters, mut started, clock, binding) = fixture();
    let lease = install(&resolver, binding, &mut started).await;
    clock.set(101, 1090);
    assert_eq!(
        resolver.validate_current(binding, lease, deadline()),
        Err(CredentialError::Stale)
    );
    clock.set(102, 999);
    assert_eq!(
        resolver.validate_current(binding, lease, deadline()),
        Err(CredentialError::Stale)
    );
    let lease = install(&resolver, binding, &mut started).await;
    clock.set(192, 10);
    assert_eq!(
        resolver.validate_current(binding, lease, deadline()),
        Err(CredentialError::Stale)
    );
}

#[tokio::test]
async fn storage_unavailable_and_material_redaction() {
    let (resolver, _counters, mut started, _clock, binding) = fixture();
    let store = FakeStore::default();
    let secret = CredentialMaterial::new(CANARY).expect("material");
    assert_eq!(format!("{secret:?}"), "CredentialMaterial(***)");
    assert!(matches!(
        resolver.read_refresh(binding, &store),
        Err(CredentialError::StoreUnavailable)
    ));
    let refresh = RefreshMaterial::new(CANARY).expect("refresh material");
    assert_eq!(
        resolver.write_refresh(binding, &store, &refresh),
        Err(CredentialError::StoreUnavailable)
    );
    assert_eq!(store.0.load(Ordering::SeqCst), 2);
    assert!(CredentialMaterial::new(&vec![0; MATERIAL_BYTES + 1]).is_err());
    let lease = install(&resolver, binding, &mut started).await;
    assert_eq!(format!("{lease:?}"), "LeaseRef(<opaque>)");
    assert_eq!(
        resolver.with_current_credential(binding, lease, deadline(), |bytes| bytes == CANARY),
        Ok(true)
    );
    resolver
        .revoke_profile(binding.spec().profile)
        .expect("revoke");
    assert_eq!(
        resolver.write_refresh(binding, &store, &refresh),
        Err(CredentialError::Revoked)
    );
    assert_eq!(store.0.load(Ordering::SeqCst), 2);
}

#[test]
fn expiry_bounds_and_invalid_expiry() {
    let start = ClockSample {
        monotonic: Duration::from_secs(100),
        wall: Duration::from_secs(1000),
    };
    let mut value = material();
    value.ttl = Some(Duration::from_secs(200_000));
    let expiry = Expiry::calculate(start, &value).expect("cap");
    assert_eq!(expiry.monotonic, Duration::from_secs(86_440));
    value.absolute_expiry = Some(Duration::from_secs(1030));
    let expiry = Expiry::calculate(start, &value).expect("earlier expiry");
    assert_eq!(expiry.monotonic, Duration::from_secs(127));
    value.ttl = Some(Duration::from_secs(1));
    assert!(Expiry::calculate(start, &value).is_err());
    value.ttl = None;
    value.absolute_expiry = None;
    assert!(Expiry::calculate(start, &value).is_err());
    value.ttl = Some(Duration::from_secs(100));
    let overflow = ClockSample {
        monotonic: Duration::MAX,
        wall: start.wall,
    };
    assert!(Expiry::calculate(overflow, &value).is_err());
}

#[tokio::test]
async fn cache_capacity_reclaims_only_expired_idle_entries() {
    let (resolver, counters, mut started, clock, first) = fixture();
    let specs: Vec<_> = (0..SLOTS).map(|_| spec(first.spec().profile)).collect();
    let inventory: Vec<_> = std::iter::once(first.spec())
        .chain(specs.iter().copied())
        .collect();
    resolver
        .replace_profile(first.spec().profile, &inventory)
        .expect("enroll");
    let first = bind(&resolver, first.spec());
    let mut leases = Vec::new();
    for spec in specs {
        let binding = bind(&resolver, spec);
        let lease = install(&resolver, binding, &mut started).await;
        leases.push((binding, lease));
    }
    let (_cancel, signal) = cancellation_pair();
    assert_eq!(
        resolver.acquire(first, deadline(), signal).await,
        Err(CredentialError::Capacity)
    );
    assert_eq!(counters.calls.load(Ordering::SeqCst), SLOTS);
    clock.set(190, 1090);
    let replacement = install(&resolver, leases[0].0, &mut started).await;
    assert_ne!(replacement, leases[0].1);
    assert_eq!(resolver.lock().expect("state").entries.len(), 1);
}

#[test]
fn omitted_auth_keeps_legacy_inline_precedence_and_redaction() {
    let config: talos_config::Config = serde_json::from_value(serde_json::json!({
        "provider": "fixture-static",
        "model": "fixture",
        "providers": {
            "fixture-static": {
                "api_key": "synthetic-lifecycle-canary",
                "api_key_env": "TALOS_I338_UNUSED_ENV"
            }
        }
    }))
    .expect("legacy config");
    assert!(matches!(
        admit_source(SourceSelection::Omitted, Source::Static),
        Ok(Source::Static)
    ));
    assert_eq!(
        config.api_key().expect("legacy inline key").as_bytes(),
        CANARY
    );
    assert!(!format!("{config:?}").contains("synthetic-lifecycle-canary"));
    let missing: talos_config::Config = serde_json::from_value(serde_json::json!({
        "provider": "fixture-static", "model": "fixture"
    }))
    .expect("legacy config");
    assert!(missing.api_key().is_err());
}

// The blocking store uses entry/exit channels instead of sleeps. Both store I/O and
// lock-sensitive probes run on blocking threads so a regression cannot freeze the
// current-thread Tokio test runtime and defeat its timeout.
struct SlowStore {
    entered: Mutex<Option<oneshot::Sender<()>>>,
    release: Mutex<std::sync::mpsc::Receiver<()>>,
    reads: AtomicUsize,
    writes: AtomicUsize,
}

impl SlowStore {
    fn hold(&self) -> Result<()> {
        self.entered
            .lock()
            .expect("entry lock")
            .take()
            .expect("one admitted operation")
            .send(())
            .map_err(|_| CredentialError::Acquisition)?;
        self.release
            .lock()
            .expect("release lock")
            .recv_timeout(Duration::from_secs(10))
            .map_err(|_| CredentialError::Deadline)
    }
}

impl RefreshStore for SlowStore {
    fn read(&self, _binding: BindingSpec) -> Result<RefreshMaterial> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        self.hold()?;
        RefreshMaterial::new(CANARY)
    }

    fn write(&self, _binding: BindingSpec, _material: &[u8]) -> Result<()> {
        self.hold()?;
        self.writes.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[tokio::test]
async fn slow_refresh_store_does_not_hold_registry_and_revocation_rejects_late_result() {
    for write in [false, true] {
        let (resolver, _counters, mut started, _clock, binding) = fixture();
        let old = install(&resolver, binding, &mut started).await;
        let other_spec = spec(Uuid::new_v4());
        resolver
            .replace_profile(other_spec.profile, &[other_spec])
            .expect("enroll other");
        let other = bind(&resolver, other_spec);
        let (entered_tx, entered_rx) = oneshot::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        let store = Arc::new(SlowStore {
            entered: Mutex::new(Some(entered_tx)),
            release: Mutex::new(release_rx),
            reads: AtomicUsize::new(0),
            writes: AtomicUsize::new(0),
        });

        let task = {
            let resolver = Arc::clone(&resolver);
            let store = Arc::clone(&store);
            tokio::task::spawn_blocking(move || {
                if write {
                    let refresh = RefreshMaterial::new(CANARY).expect("material");
                    resolver.write_refresh(binding, store.as_ref(), &refresh)
                } else {
                    resolver.read_refresh(binding, store.as_ref()).map(drop)
                }
            })
        };
        tokio::time::timeout(Duration::from_secs(3), entered_rx)
            .await
            .expect("store entered before timeout")
            .expect("entry signal");

        // This must complete while the first store is still deliberately held.
        let probe = {
            let resolver = Arc::clone(&resolver);
            tokio::task::spawn_blocking(move || {
                let store = FakeStore::default();
                let refresh = RefreshMaterial::new(CANARY).expect("material");
                assert!(matches!(
                    resolver.read_refresh(binding, &store),
                    Err(CredentialError::StoreBusy)
                ));
                assert_eq!(
                    resolver.write_refresh(binding, &store, &refresh),
                    Err(CredentialError::StoreBusy)
                );
                assert_eq!(store.0.load(Ordering::SeqCst), 0);
                resolver
                    .revoke_profile(binding.spec().profile)
                    .expect("revoke");
                assert_eq!(
                    resolver.validate_current(binding, old, deadline()),
                    Err(CredentialError::Revoked)
                );
                assert!(matches!(
                    resolver.read_refresh(binding, &store),
                    Err(CredentialError::Revoked)
                ));
                resolver
                    .replace_profile(binding.spec().profile, &[binding.spec()])
                    .expect("replace");
                let replacement = bind(&resolver, binding.spec());
                assert!(matches!(
                    resolver.read_refresh(replacement, &store),
                    Err(CredentialError::StoreBusy)
                ));
                assert_eq!(store.0.load(Ordering::SeqCst), 0);
                replacement
            })
        };
        let replacement = tokio::time::timeout(Duration::from_secs(3), probe)
            .await
            .expect("revocation and replacement must not wait for store")
            .expect("probe joined");

        // Exercise a real independent acquisition and dispatch-admission check,
        // not merely a try_lock, before releasing the slow store.
        let other_lease = install(&resolver, other, &mut started).await;
        assert_eq!(
            resolver.validate_current(other, other_lease, deadline()),
            Ok(())
        );
        release_tx.send(()).expect("release slow store");
        assert_eq!(
            task.await.expect("store joined"),
            Err(CredentialError::Revoked)
        );
        assert_eq!(
            store.reads.load(Ordering::SeqCst),
            if write { 0 } else { 1 }
        );
        // A write admitted before revoke can physically complete: Revoked is
        // not an external-store rollback. No old lease or successful read is returned.
        assert_eq!(
            store.writes.load(Ordering::SeqCst),
            if write { 1 } else { 0 }
        );
        assert!(resolver.lock().expect("state").store_operations.is_empty());
        let fake = FakeStore::default();
        assert!(matches!(
            resolver.read_refresh(replacement, &fake),
            Err(CredentialError::StoreUnavailable)
        ));
        assert_eq!(fake.0.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn store_callback_can_reenter_registry_without_deadlocking() {
    struct ReentrantStore<'a> {
        resolver: &'a CredentialResolver,
        binding: Binding,
    }

    impl RefreshStore for ReentrantStore<'_> {
        fn read(&self, _binding: BindingSpec) -> Result<RefreshMaterial> {
            // Same-profile reentry must fail before calling the nested store.
            let nested = FakeStore::default();
            assert!(matches!(
                self.resolver.read_refresh(self.binding, &nested),
                Err(CredentialError::StoreBusy)
            ));
            assert_eq!(nested.0.load(Ordering::SeqCst), 0);
            self.resolver.revoke_profile(self.binding.spec().profile)?;
            RefreshMaterial::new(CANARY)
        }

        fn write(&self, _binding: BindingSpec, _material: &[u8]) -> Result<()> {
            Err(CredentialError::Acquisition)
        }
    }

    let (resolver, _counters, _started, _clock, binding) = fixture();
    let (done_tx, done_rx) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        let store = ReentrantStore {
            resolver: &resolver,
            binding,
        };
        assert!(matches!(
            resolver.read_refresh(binding, &store),
            Err(CredentialError::Revoked)
        ));
        assert!(resolver.lock().expect("state").store_operations.is_empty());
        done_tx.send(()).expect("completion");
    });
    done_rx
        .recv_timeout(Duration::from_secs(3))
        .expect("reentry must not deadlock");
    worker.join().expect("worker joined");
}

#[tokio::test]
async fn enrollment_checks_all_binding_dimensions_and_context_isolation() {
    let (resolver, counters, mut started, _clock, first) = fixture();
    let base = first.spec();
    let variants = [
        BindingSpec {
            account: Uuid::new_v4(),
            ..base
        },
        BindingSpec {
            audience: Uuid::new_v4(),
            ..base
        },
        BindingSpec {
            source: Uuid::new_v4(),
            ..base
        },
        BindingSpec {
            profile: Uuid::new_v4(),
            ..base
        },
    ];
    for requested in variants {
        assert!(matches!(
            resolver.bind(
                requested,
                SourceSelection::Explicit(&[Source::Dynamic(requested.source)])
            ),
            Err(CredentialError::Configuration)
        ));
    }
    assert_eq!(counters.calls.load(Ordering::SeqCst), 0);
    let inventory = [base, variants[0], variants[1], variants[2]];
    resolver
        .replace_profile(base.profile, &inventory)
        .expect("trusted enrollment");
    resolver
        .replace_profile(variants[3].profile, &[variants[3]])
        .expect("separate profile");
    let base_binding = bind(&resolver, base);
    let original = install(&resolver, base_binding, &mut started).await;
    for requested in variants {
        let binding = bind(&resolver, requested);
        let lease = install(&resolver, binding, &mut started).await;
        assert_ne!(lease, original);
        assert_eq!(
            resolver.validate_current(binding, original, deadline()),
            Err(CredentialError::Stale)
        );
        assert_eq!(
            resolver.invalidate_if_current(binding, original),
            Ok(Invalidation::Stale)
        );
    }
    assert_eq!(counters.calls.load(Ordering::SeqCst), 5);
    assert_eq!(
        resolver.validate_current(base_binding, original, deadline()),
        Ok(())
    );
}

#[tokio::test]
async fn explicit_sdk_context_shares_resolver_and_closes_outstanding_handles() {
    let counters = Arc::new(Counters::default());
    let (events, mut started) = mpsc::unbounded_channel();
    let source = Arc::new(DelayedSource {
        counters: Arc::clone(&counters),
        started: events,
    });
    let context = CredentialContext::new(source.clone());
    let independent = CredentialContext::new(source);
    let first = context.resolver();
    let second = context.resolver();
    let other = independent.resolver();
    assert!(Arc::ptr_eq(&first, &second));
    assert!(!Arc::ptr_eq(&first, &other));
    let spec = spec(Uuid::new_v4());
    first
        .replace_profile(spec.profile, &[spec])
        .expect("enroll");
    other
        .replace_profile(spec.profile, &[spec])
        .expect("enroll separate host");
    let binding = bind(&first, spec);
    let (_cancel, waiter, producer) = start(&first, binding, &mut started).await;
    let (_cancel, signal) = cancellation_pair();
    let mut joined = std::pin::pin!(second.acquire(binding, deadline(), signal));
    assert!(futures_util::poll!(&mut joined).is_pending());
    producer.release.send(()).expect("release");
    let lease = waiter.await.expect("waiter").expect("lease");
    assert_eq!(joined.await, Ok(lease));
    assert_eq!(counters.calls.load(Ordering::SeqCst), 1);
    let other_binding = bind(&other, spec);
    let (_cancel, signal) = cancellation_pair();
    assert_eq!(
        other.acquire(binding, deadline(), signal).await,
        Err(CredentialError::Revoked)
    );
    let independent_lease = install(&other, other_binding, &mut started).await;
    assert_ne!(lease, independent_lease);
    context.shutdown(deadline()).await.expect("explicit close");
    assert_eq!(
        second.validate_current(binding, lease, deadline()),
        Err(CredentialError::Revoked)
    );
    assert_eq!(
        other.validate_current(other_binding, independent_lease, deadline()),
        Ok(())
    );
    other
        .invalidate_if_current(other_binding, independent_lease)
        .expect("refresh");
    let (_cancel, waiter, producer) = start(&other, other_binding, &mut started).await;
    drop(independent);
    assert_eq!(waiter.await.expect("waiter"), Err(CredentialError::Revoked));
    assert!(producer.cancellation.is_cancelled());
    producer
        .release
        .send(())
        .expect("finish cancelled producer");
    other
        .shutdown(deadline())
        .await
        .expect("join after owner drop");
    assert_eq!(counters.running.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn waiter_limits_hold_across_keys_and_release_on_future_drop() {
    let (resolver, counters, mut started, _clock, first) = fixture();
    let specs: Vec<_> = (0..5).map(|_| spec(first.spec().profile)).collect();
    resolver
        .replace_profile(first.spec().profile, &specs)
        .expect("enroll");
    let bindings: Vec<_> = specs.iter().map(|spec| bind(&resolver, *spec)).collect();
    let mut pending = Vec::new();
    let mut producers = Vec::new();
    for binding in bindings.iter().copied().take(4) {
        for _ in 0..PER_KEY_WAITERS {
            let (cancel, signal) = cancellation_pair();
            let mut future = Box::pin(resolver.acquire(binding, deadline(), signal));
            assert!(futures_util::poll!(&mut future).is_pending());
            pending.push((cancel, future));
        }
        let producer = tokio::time::timeout_at(deadline(), started.recv())
            .await
            .expect("producer starts")
            .expect("event");
        producers.push(producer);
        let (_cancel, signal) = cancellation_pair();
        assert_eq!(
            resolver.acquire(binding, deadline(), signal).await,
            Err(CredentialError::Capacity)
        );
    }
    assert_eq!(resolver.lock().expect("registry").waiters, TOTAL_WAITERS);
    let (_cancel, signal) = cancellation_pair();
    assert_eq!(
        resolver.acquire(bindings[4], deadline(), signal).await,
        Err(CredentialError::Capacity)
    );
    assert_eq!(counters.calls.load(Ordering::SeqCst), 4);
    drop(pending);
    assert_eq!(resolver.lock().expect("registry").waiters, 0);
    for producer in producers {
        assert!(producer.cancellation.is_cancelled());
        producer.release.send(()).expect("finish");
    }
    wait_count(&resolver, 0).await;
    assert!(resolver.lock().expect("registry").entries.is_empty());
}

#[tokio::test(start_paused = true)]
async fn caller_and_producer_deadlines_retire_identity_without_releasing_running_task() {
    let (resolver, counters, mut started, _clock, binding) = fixture();
    for (caller_seconds, advance_seconds) in [(5, 5), (60, 30)] {
        let (_cancel, signal) = cancellation_pair();
        let owner = Arc::clone(&resolver);
        let deadline = Instant::now() + Duration::from_secs(caller_seconds);
        let waiter = tokio::spawn(async move { owner.acquire(binding, deadline, signal).await });
        let producer = started.recv().await.expect("producer enters");
        tokio::time::advance(Duration::from_secs(advance_seconds)).await;
        assert_eq!(
            waiter.await.expect("waiter"),
            Err(CredentialError::Deadline)
        );
        assert!(producer.cancellation.is_cancelled());
        assert_eq!(counters.running.load(Ordering::SeqCst), 1);
        assert_eq!(resolver.lock().expect("registry").producers.len(), 1);
        producer.release.send(()).expect("actual exit");
        wait_count(&resolver, 0).await;
        assert!(resolver.lock().expect("registry").entries.is_empty());
    }
    resolver.shutdown(deadline()).await.expect("closed");
    let (_cancel, signal) = cancellation_pair();
    assert_eq!(
        resolver.acquire(binding, deadline(), signal).await,
        Err(CredentialError::Revoked)
    );
}

#[tokio::test]
async fn monotonic_regression_retires_cached_lease_and_active_flight() {
    let (resolver, _counters, mut started, clock, first) = fixture();
    let other_spec = spec(first.spec().profile);
    resolver
        .replace_profile(first.spec().profile, &[first.spec(), other_spec])
        .expect("enroll");
    let first = bind(&resolver, first.spec());
    let other = bind(&resolver, other_spec);
    let lease = install(&resolver, first, &mut started).await;
    let (_cancel, waiter, producer) = start(&resolver, other, &mut started).await;
    clock.set(99, 1001);
    assert_eq!(
        resolver.validate_current(first, lease, deadline()),
        Err(CredentialError::Expiry)
    );
    assert_eq!(waiter.await.expect("waiter"), Err(CredentialError::Expiry));
    assert!(producer.cancellation.is_cancelled());
    clock.set(101, 1002);
    assert_eq!(
        resolver.validate_current(first, lease, deadline()),
        Err(CredentialError::Stale)
    );
    producer.release.send(()).expect("finish");
    wait_count(&resolver, 0).await;
    assert!(resolver.lock().expect("registry").entries.is_empty());
}

#[test]
fn legacy_shared_factory_preview_does_not_require_credential_context() {
    use crate::openai_responses::factory::build_configured_provider;
    for protocol in ["anthropic-messages", "openai-chat", "openai-responses"] {
        let config: talos_config::Config = serde_json::from_value(serde_json::json!({
            "provider": "fixture-static",
            "model": "fixture",
            "providers": {
                "fixture-static": {
                    "protocol": protocol,
                    "base_url": "https://example.com/v1",
                    "api_key": "synthetic-lifecycle-canary"
                }
            }
        }))
        .expect("legacy config");
        let key = config.api_key().expect("host static admission");
        let provider = build_configured_provider(&config, &key).expect("shared factory");
        let preview = provider.request_preview(&[]).expect("redacted preview");
        assert!(preview.get("error").is_none());
        assert!(!preview.to_string().contains("synthetic-lifecycle-canary"));
        assert!(
            preview["url"]
                .as_str()
                .expect("route")
                .starts_with("https://example.com/")
        );
        if protocol == "anthropic-messages" {
            assert!(preview["headers"].get("x-api-key").is_some());
        } else {
            assert!(preview["headers"].get("Authorization").is_some());
        }
    }
}

#[tokio::test]
async fn acquisition_failure_is_shared_without_retry_or_secret_diagnostic() {
    let (resolver, counters, mut started, _clock, binding) = fixture();
    let (_cancel, first, producer) = start(&resolver, binding, &mut started).await;
    let (_cancel, signal) = cancellation_pair();
    let mut second = std::pin::pin!(resolver.acquire(binding, deadline(), signal));
    assert!(futures_util::poll!(&mut second).is_pending());
    // Closing the fake's exit channel produces a typed acquisition failure.
    drop(producer.release);
    assert_eq!(
        first.await.expect("first waiter"),
        Err(CredentialError::Acquisition)
    );
    let error = second.await.expect_err("shared failure");
    assert_eq!(error, CredentialError::Acquisition);
    assert_eq!(error.to_string(), "credential acquisition failed");
    assert_eq!(counters.calls.load(Ordering::SeqCst), 1);
    wait_count(&resolver, 0).await;
    let lease = install(&resolver, binding, &mut started).await;
    assert_eq!(
        resolver.validate_current(binding, lease, deadline()),
        Ok(())
    );
    assert_eq!(counters.calls.load(Ordering::SeqCst), 2);
}

#[test]
fn invalid_or_oversized_enrollment_preserves_current_binding() {
    let (resolver, _counters, _started, _clock, binding) = fixture();
    let base = binding.spec();
    assert_eq!(
        resolver.replace_profile(base.profile, &[]),
        Err(CredentialError::Configuration)
    );
    assert_eq!(
        resolver.replace_profile(base.profile, &[base, base]),
        Err(CredentialError::Configuration)
    );
    assert_eq!(
        resolver.replace_profile(
            base.profile,
            &[BindingSpec {
                profile: Uuid::new_v4(),
                ..base
            }]
        ),
        Err(CredentialError::Configuration)
    );
    let oversized = vec![base; ENROLLED_BINDINGS + 1];
    assert_eq!(
        resolver.replace_profile(base.profile, &oversized),
        Err(CredentialError::Capacity)
    );
    assert!(bind(&resolver, base) == binding);
}


// Run the synchronous registry/clock section on a blocking worker entered into
// the same paused Tokio runtime. The test thread can advance its clock or cancel
// after sample entry, while the worker still owns the registry gate.
async fn terminate_at_sample(
    entered: oneshot::Receiver<()>,
    release: std::sync::mpsc::Sender<()>,
    cancel: &watch::Sender<bool>,
    terminal: CredentialError,
) {
    tokio::time::timeout(Duration::from_secs(10), entered)
        .await
        .expect("clock sample entered before timeout")
        .expect("sample entry signal");
    match terminal {
        CredentialError::Cancelled => {
            cancel.send_replace(true);
        }
        CredentialError::Deadline => {
            tokio::time::advance(Duration::from_secs(5)).await;
        }
        _ => panic!("unexpected terminal condition"),
    }
    release.send(()).expect("release sample");
}

async fn acquire_admission_race(cached: bool) {
    for terminal in [CredentialError::Cancelled, CredentialError::Deadline] {
        let (resolver, counters, mut started, clock, binding) = fixture();
        let existing = if cached {
            Some(install(&resolver, binding, &mut started).await)
        } else {
            None
        };
        let calls = counters.calls.load(Ordering::SeqCst);
        let sequence = resolver.lock().expect("registry").sequence;
        let (entered, release) = clock.pause_sample(0);
        let (cancel, signal) = cancellation_pair();
        let caller_deadline = Instant::now() + Duration::from_secs(5);
        let worker = {
            let resolver = Arc::clone(&resolver);
            tokio::task::spawn_blocking(move || {
                tokio::runtime::Handle::current().block_on(resolver.acquire(
                    binding,
                    caller_deadline,
                    signal,
                ))
            })
        };
        terminate_at_sample(entered, release, &cancel, terminal).await;
        assert_eq!(worker.await.expect("caller joined"), Err(terminal));
        {
            let state = resolver.lock().expect("registry");
            // Even a spawned-and-immediately-retired flight would advance sequence.
            assert_eq!(state.sequence, sequence);
            assert_eq!(state.waiters, 0);
            assert!(state.producers.is_empty());
            assert_eq!(state.entries.len(), if cached { 1 } else { 0 });
        }
        assert_eq!(counters.calls.load(Ordering::SeqCst), calls);
        assert!(started.try_recv().is_err());
        if let Some(lease) = existing {
            assert_eq!(resolver.validate_current(binding, lease, deadline()), Ok(()));
        }
        resolver.shutdown(deadline()).await.expect("shutdown");
        assert_eq!(counters.calls.load(Ordering::SeqCst), calls);
    }
}

#[tokio::test(start_paused = true)]
async fn cached_acquire_rechecks_terminal_caller_after_clock_sample() {
    acquire_admission_race(true).await;
}

#[tokio::test(start_paused = true)]
async fn uncached_acquire_rechecks_terminal_caller_before_flight_admission() {
    acquire_admission_race(false).await;
}

#[tokio::test(start_paused = true)]
async fn acquire_rechecks_terminal_caller_during_final_lease_validation() {
    for terminal in [CredentialError::Cancelled, CredentialError::Deadline] {
        let (resolver, counters, mut started, clock, binding) = fixture();
        let (cancel, signal) = cancellation_pair();
        let caller_deadline = Instant::now() + Duration::from_secs(5);
        let worker = {
            let resolver = Arc::clone(&resolver);
            tokio::task::spawn_blocking(move || {
                tokio::runtime::Handle::current().block_on(resolver.acquire(
                    binding,
                    caller_deadline,
                    signal,
                ))
            })
        };
        let producer = started.recv().await.expect("producer enters");
        // Skip producer publication's sample; pause the waiting caller's final
        // validation after it has already received the successful flight result.
        let (entered, release) = clock.pause_sample(1);
        producer.release.send(()).expect("release producer");
        terminate_at_sample(entered, release, &cancel, terminal).await;
        assert_eq!(worker.await.expect("caller joined"), Err(terminal));
        assert_eq!(resolver.lock().expect("registry").waiters, 0);
        assert_eq!(counters.calls.load(Ordering::SeqCst), 1);
        let (_cancel, signal) = cancellation_pair();
        let lease = resolver
            .acquire(binding, deadline(), signal)
            .await
            .expect("published lease remains usable for another caller");
        assert_eq!(resolver.validate_current(binding, lease, deadline()), Ok(()));
        assert_eq!(counters.calls.load(Ordering::SeqCst), 1);
        resolver.shutdown(deadline()).await.expect("shutdown");
    }
}

#[tokio::test(start_paused = true)]
async fn current_rechecks_deadline_before_validation_or_secret_access() {
    let (resolver, counters, mut started, clock, binding) = fixture();
    let lease = install(&resolver, binding, &mut started).await;
    let consumed = Arc::new(AtomicUsize::new(0));
    for scoped in [false, true] {
        let (entered, release) = clock.pause_sample(0);
        let caller_deadline = Instant::now() + Duration::from_secs(5);
        let worker = {
            let resolver = Arc::clone(&resolver);
            let consumed = Arc::clone(&consumed);
            tokio::task::spawn_blocking(move || {
                if scoped {
                    resolver.with_current_credential(binding, lease, caller_deadline, |_| {
                        consumed.fetch_add(1, Ordering::SeqCst);
                    })
                } else {
                    resolver.validate_current(binding, lease, caller_deadline)
                }
            })
        };
        let (cancel, _signal) = cancellation_pair();
        terminate_at_sample(entered, release, &cancel, CredentialError::Deadline).await;
        assert_eq!(worker.await.expect("validation joined"), Err(CredentialError::Deadline));
        assert_eq!(consumed.load(Ordering::SeqCst), 0);
        assert_eq!(resolver.validate_current(binding, lease, deadline()), Ok(()));
        assert_eq!(counters.calls.load(Ordering::SeqCst), 1);
    }
    resolver.shutdown(deadline()).await.expect("shutdown");
}
