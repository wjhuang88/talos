//! Disposable scheduling experiment, never called by production search.
//! Synthetic outcomes are already classified; this does not test adapter parsing,
//! caller-context propagation, health/eligibility, or transport/task cancellation.
//! Durations and a two-future cap are fixture parameters, not shipping policy.
#![cfg(feature = "network")]

use std::future::{Future, poll_fn};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::Poll;
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::time::{Instant, sleep, sleep_until};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Outcome {
    Failed,
    Empty,
    Invalid,
    Valid,
}

#[derive(Debug, PartialEq, Eq)]
enum Terminal {
    Winner(usize),
    NoEligible,
    Exhausted(Vec<(usize, Outcome)>),
    Deadline,
    Cancelled,
}

type Work = Pin<Box<dyn Future<Output = Outcome> + Send>>;
type Candidate = Box<dyn FnOnce() -> Work + Send>;

// Own futures directly. No spawned work, adapter trait or reusable runtime API.
async fn experiment(
    candidates: Vec<Candidate>,
    deadline: Instant,
    hedge_delay: Duration,
    mut cancellation: oneshot::Receiver<()>,
) -> Terminal {
    let mut remaining = candidates.into_iter().enumerate();
    let mut active: Vec<(usize, Work)> = Vec::new();
    let mut failures = Vec::new();
    let mut launch_next = true;
    let hedge = sleep(hedge_delay);
    tokio::pin!(hedge);
    loop {
        // Control signals take precedence at the boundary, including before launch.
        if let Some(terminal) = control(deadline, &mut cancellation) {
            return terminal;
        }
        if launch_next || active.is_empty() {
            if let Some((id, start)) = remaining.next() {
                active.push((id, start()));
                hedge.as_mut().reset(Instant::now() + hedge_delay);
            } else if active.is_empty() {
                return if failures.is_empty() {
                    Terminal::NoEligible
                } else {
                    Terminal::Exhausted(failures)
                };
            }
        }
        tokio::select! {
            biased;
            _ = &mut cancellation => return Terminal::Cancelled,
            _ = sleep_until(deadline) => return Terminal::Deadline,
            (index, outcome) = poll_fn(|cx| {
                for (index, (_, work)) in active.iter_mut().enumerate() {
                    if let Poll::Ready(outcome) = work.as_mut().poll(cx) {
                        return Poll::Ready((index, outcome));
                    }
                }
                Poll::Pending
            }) => {
                let (id, completed) = active.remove(index);
                drop(completed);
                if outcome == Outcome::Valid {
                    // A candidate poll can itself make a control signal ready.
                    return control(deadline, &mut cancellation)
                        .unwrap_or(Terminal::Winner(id));
                }
                failures.push((id, outcome));
                // Reenter the checked launch boundary before filling the slot.
                launch_next = remaining.len() > 0;
            }
            _ = &mut hedge, if active.len() < 2 && remaining.len() > 0 => {
                launch_next = true;
            }
        }
    }
}

fn control(deadline: Instant, cancellation: &mut oneshot::Receiver<()>) -> Option<Terminal> {
    if Instant::now() >= deadline {
        return Some(Terminal::Deadline);
    }
    match cancellation.try_recv() {
        Ok(()) | Err(oneshot::error::TryRecvError::Closed) => Some(Terminal::Cancelled),
        Err(oneshot::error::TryRecvError::Empty) => None,
    }
}

#[derive(Default, Debug)]
struct Counts {
    starts: Vec<usize>,
    drops: Vec<usize>,
    active: usize,
    peak: usize,
}

type Observations = Arc<Mutex<Counts>>;

struct Owned {
    id: usize,
    counts: Observations,
}

impl Drop for Owned {
    fn drop(&mut self) {
        let mut counts = self.counts.lock().expect("fixture observation mutex");
        counts.active -= 1;
        counts.drops.push(self.id);
    }
}

fn begin(id: usize, counts: Observations) -> Owned {
    {
        let mut counts = counts.lock().expect("fixture observation mutex");
        counts.starts.push(id);
        counts.active += 1;
        counts.peak = counts.peak.max(counts.active);
    }
    Owned { id, counts }
}

fn candidate(id: usize, delay_ms: u64, outcome: Outcome, counts: &Observations) -> Candidate {
    let counts = Arc::clone(counts);
    Box::new(move || {
        let owned = begin(id, counts);
        Box::pin(async move {
            let _owned = owned;
            sleep(Duration::from_millis(delay_ms)).await;
            outcome
        })
    })
}

fn assert_released(counts: &Observations, starts: &[usize], peak: usize) {
    let counts = counts.lock().expect("fixture observation mutex");
    assert_eq!(counts.starts, starts);
    assert_eq!(counts.active, 0);
    assert_eq!(counts.peak, peak);
    let mut drops = counts.drops.clone();
    drops.sort_unstable();
    assert_eq!(drops, starts);
}

#[tokio::test(start_paused = true)]
async fn unusable_primary_does_not_win_and_failure_launches_without_hedge_wait() {
    for outcome in [Outcome::Failed, Outcome::Empty, Outcome::Invalid] {
        let counts = Observations::default();
        let (_sender, receiver) = oneshot::channel();
        let start = Instant::now();
        let result = experiment(
            vec![
                candidate(0, 5, outcome, &counts),
                candidate(1, 15, Outcome::Valid, &counts),
            ],
            start + Duration::from_millis(100),
            Duration::from_millis(50),
            receiver,
        )
        .await;
        assert_eq!(result, Terminal::Winner(1));
        assert_eq!(start.elapsed(), Duration::from_millis(20));
        assert_released(&counts, &[0, 1], 1);
    }
}

#[tokio::test(start_paused = true)]
async fn hedge_waits_then_winner_drops_loser_and_never_starts_third() {
    let counts = Observations::default();
    let (_sender, receiver) = oneshot::channel();
    let start = Instant::now();
    let result = experiment(
        vec![
            candidate(0, 90, Outcome::Valid, &counts),
            candidate(1, 5, Outcome::Valid, &counts),
            candidate(2, 0, Outcome::Valid, &counts),
        ],
        start + Duration::from_millis(100),
        Duration::from_millis(10),
        receiver,
    )
    .await;
    assert_eq!(result, Terminal::Winner(1));
    assert_eq!(start.elapsed(), Duration::from_millis(15));
    assert_released(&counts, &[0, 1], 2);
}

#[tokio::test(start_paused = true)]
async fn all_unusable_attempts_are_finite_with_two_owned_futures_at_most() {
    let counts = Observations::default();
    let (_sender, receiver) = oneshot::channel();
    let result = experiment(
        vec![
            candidate(0, 30, Outcome::Failed, &counts),
            candidate(1, 5, Outcome::Empty, &counts),
            candidate(2, 50, Outcome::Invalid, &counts),
        ],
        Instant::now() + Duration::from_millis(100),
        Duration::from_millis(10),
        receiver,
    )
    .await;
    assert_eq!(
        result,
        Terminal::Exhausted(vec![
            (1, Outcome::Empty),
            (0, Outcome::Failed),
            (2, Outcome::Invalid)
        ])
    );
    assert_released(&counts, &[0, 1, 2], 2);
}

#[tokio::test(start_paused = true)]
async fn enclosing_deadline_does_not_reset_for_hedges_or_replacements() {
    let counts = Observations::default();
    let (_sender, receiver) = oneshot::channel();
    let start = Instant::now();
    let result = experiment(
        vec![
            candidate(0, 100, Outcome::Valid, &counts),
            candidate(1, 5, Outcome::Failed, &counts),
            candidate(2, 100, Outcome::Valid, &counts),
            candidate(3, 0, Outcome::Valid, &counts),
        ],
        start + Duration::from_millis(25),
        Duration::from_millis(10),
        receiver,
    )
    .await;
    assert_eq!(result, Terminal::Deadline);
    assert_eq!(start.elapsed(), Duration::from_millis(25));
    assert_released(&counts, &[0, 1, 2], 2);
}

#[tokio::test(start_paused = true)]
async fn expired_budget_or_pre_cancelled_signal_prevents_any_launch() {
    for cancelled in [false, true] {
        let counts = Observations::default();
        let (sender, receiver) = oneshot::channel();
        if cancelled {
            sender.send(()).expect("fixture receiver remains owned");
        }
        let deadline = Instant::now() + Duration::from_millis(if cancelled { 100 } else { 0 });
        let result = experiment(
            vec![candidate(0, 0, Outcome::Valid, &counts)],
            deadline,
            Duration::from_millis(10),
            receiver,
        )
        .await;
        assert_eq!(
            result,
            if cancelled {
                Terminal::Cancelled
            } else {
                Terminal::Deadline
            }
        );
        assert_released(&counts, &[], 0);
    }
}

#[tokio::test(start_paused = true)]
async fn explicit_inflight_cancellation_releases_owned_work() {
    let counts = Observations::default();
    let (sender, receiver) = oneshot::channel();
    let router = experiment(
        vec![
            candidate(0, 100, Outcome::Valid, &counts),
            candidate(1, 100, Outcome::Valid, &counts),
        ],
        Instant::now() + Duration::from_millis(200),
        Duration::from_millis(10),
        receiver,
    );
    let cancel = async {
        sleep(Duration::from_millis(20)).await;
        sender.send(()).expect("fixture receiver remains owned");
    };
    let (result, ()) = tokio::join!(router, cancel);
    assert_eq!(result, Terminal::Cancelled);
    assert_released(&counts, &[0, 1], 2);
}

#[tokio::test(start_paused = true)]
async fn external_drop_is_cleanup_without_a_returned_cancelled_outcome() {
    let counts = Observations::default();
    let (_sender, receiver) = oneshot::channel();
    let mut router = Box::pin(experiment(
        vec![candidate(0, 100, Outcome::Valid, &counts)],
        Instant::now() + Duration::from_millis(200),
        Duration::from_millis(10),
        receiver,
    ));
    poll_fn(|cx| {
        assert!(router.as_mut().poll(cx).is_pending());
        Poll::Ready(())
    })
    .await;
    drop(router);
    assert_released(&counts, &[0], 1);
}

#[tokio::test(start_paused = true)]
async fn no_eligible_candidate_is_distinct_from_all_failed() {
    let (_sender, receiver) = oneshot::channel();
    assert_eq!(
        experiment(
            vec![],
            Instant::now() + Duration::from_millis(100),
            Duration::from_millis(10),
            receiver
        )
        .await,
        Terminal::NoEligible
    );
}

#[tokio::test(start_paused = true)]
async fn fast_valid_primary_never_launches_hedge() {
    let counts = Observations::default();
    let (_sender, receiver) = oneshot::channel();
    let result = experiment(
        vec![
            candidate(0, 5, Outcome::Valid, &counts),
            candidate(1, 0, Outcome::Valid, &counts),
        ],
        Instant::now() + Duration::from_millis(100),
        Duration::from_millis(10),
        receiver,
    )
    .await;
    assert_eq!(result, Terminal::Winner(0));
    assert_released(&counts, &[0], 1);
}

#[tokio::test(start_paused = true)]
async fn failed_hedge_does_not_end_pending_primary() {
    let counts = Observations::default();
    let (_sender, receiver) = oneshot::channel();
    let start = Instant::now();
    let result = experiment(
        vec![
            candidate(0, 30, Outcome::Valid, &counts),
            candidate(1, 5, Outcome::Failed, &counts),
        ],
        start + Duration::from_millis(100),
        Duration::from_millis(10),
        receiver,
    )
    .await;
    assert_eq!(result, Terminal::Winner(0));
    assert_eq!(start.elapsed(), Duration::from_millis(30));
    assert_released(&counts, &[0, 1], 2);
}

#[tokio::test(start_paused = true)]
async fn deadline_wins_over_success_ready_at_the_same_instant() {
    let counts = Observations::default();
    let (_sender, receiver) = oneshot::channel();
    let result = experiment(
        vec![candidate(0, 20, Outcome::Valid, &counts)],
        Instant::now() + Duration::from_millis(20),
        Duration::from_millis(10),
        receiver,
    )
    .await;
    assert_eq!(result, Terminal::Deadline);
    assert_released(&counts, &[0], 1);
}

#[tokio::test(start_paused = true)]
async fn cancellation_during_candidate_poll_prevents_further_launches_or_success() {
    for ready_outcome in [Some(Outcome::Failed), None, Some(Outcome::Valid)] {
        let counts = Observations::default();
        let (sender, receiver) = oneshot::channel();
        let observed = Arc::clone(&counts);
        let interrupt: Candidate = Box::new(move || {
            let owned = begin(0, observed);
            let mut sender = Some(sender);
            let mut delay = Box::pin(sleep(Duration::from_millis(if ready_outcome.is_some() {
                5
            } else {
                10
            })));
            Box::pin(poll_fn(move |cx| {
                let _keep_owned = &owned;
                if delay.as_mut().poll(cx).is_pending() {
                    return Poll::Pending;
                }
                if let Some(sender) = sender.take() {
                    sender.send(()).expect("fixture receiver remains owned");
                }
                match ready_outcome {
                    Some(outcome) => Poll::Ready(outcome),
                    None => Poll::Pending,
                }
            }))
        });
        let result = experiment(
            vec![interrupt, candidate(1, 0, Outcome::Valid, &counts)],
            Instant::now() + Duration::from_millis(100),
            Duration::from_millis(10),
            receiver,
        )
        .await;
        assert_eq!(result, Terminal::Cancelled);
        assert_released(&counts, &[0], 1);
    }
}
