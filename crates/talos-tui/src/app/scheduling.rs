use futures::{Stream, StreamExt};
use std::pin::Pin;
use std::time::{Duration, Instant};

/// Tokio's monotonic clock also supports deterministic deadline tests.
pub(super) fn now() -> Instant {
    tokio::time::Instant::now().into_std()
}

pub(super) async fn stream_event(
    stream: &mut Option<Pin<Box<dyn Stream<Item = String> + Send>>>,
) -> Option<String> {
    match stream.as_mut() {
        Some(stream) => stream.next().await,
        None => std::future::pending().await,
    }
}

pub(super) const STREAM_FRAME_INTERVAL: Duration = Duration::from_millis(33);

pub(super) fn stream_deadline(last_submission: Instant) -> Instant {
    last_submission + STREAM_FRAME_INTERVAL
}

/// An optional periodic deadline which input traffic cannot postpone.
#[derive(Default)]
pub(super) struct Cadence {
    next: Option<Instant>,
}

impl Cadence {
    pub(super) fn update(&mut self, enabled: bool, now: Instant, period: Duration) {
        if enabled {
            self.next.get_or_insert(now + period);
        } else {
            self.next = None;
        }
    }

    pub(super) fn deadline(&self) -> Option<Instant> {
        self.next
    }

    pub(super) fn consume(&mut self, now: Instant, period: Duration) -> usize {
        let Some(deadline) = self.next.filter(|deadline| *deadline <= now) else {
            return 0;
        };
        let elapsed = now.duration_since(deadline);
        let steps = elapsed.as_nanos() / period.as_nanos() + 1;
        let remainder = Duration::from_nanos((elapsed.as_nanos() % period.as_nanos()) as u64);
        self.next = Some(now + period - remainder);
        steps.min(usize::MAX as u128) as usize
    }
}

pub(super) async fn wait_until(deadline: Option<Instant>) {
    match deadline {
        Some(deadline) => tokio::time::sleep_until(deadline.into()).await,
        None => std::future::pending().await,
    }
}

pub(super) async fn approval_closed(
    sender: &mut Option<tokio::sync::oneshot::Sender<talos_core::ApprovalChoice>>,
) {
    match sender.as_mut() {
        Some(sender) => sender.closed().await,
        None => std::future::pending().await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::FutureExt;

    #[tokio::test]
    async fn legacy_eof_is_an_event_but_absent_stream_is_not() {
        let mut stream: Option<Pin<Box<dyn Stream<Item = String> + Send>>> =
            Some(Box::pin(futures::stream::iter(["final token".to_owned()])));
        assert_eq!(
            stream_event(&mut stream).await.as_deref(),
            Some("final token")
        );
        assert_eq!(stream_event(&mut stream).now_or_never(), Some(None));
        stream = None;
        assert!(stream_event(&mut stream).now_or_never().is_none());
    }

    #[tokio::test]
    async fn absent_deadline_and_approval_do_not_wake() {
        assert!(wait_until(None).now_or_never().is_none());
        assert!(approval_closed(&mut None).now_or_never().is_none());
    }

    #[tokio::test]
    async fn dropped_approval_receiver_wakes_without_timer() {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let mut sender = Some(sender);
        assert!(approval_closed(&mut sender).now_or_never().is_none());
        drop(receiver);
        assert_eq!(approval_closed(&mut sender).now_or_never(), Some(()));
    }

    #[test]
    fn traffic_does_not_postpone_and_missed_frames_do_not_burst() {
        let start = Instant::now();
        let period = Duration::from_millis(150);
        let mut cadence = Cadence::default();
        cadence.update(true, start, period);
        cadence.update(true, start + Duration::from_millis(100), period);
        assert_eq!(cadence.deadline(), Some(start + period));
        assert_eq!(
            cadence.consume(start + Duration::from_millis(620), period),
            4
        );
        assert_eq!(cadence.deadline(), Some(start + Duration::from_millis(750)));
        assert_eq!(
            cadence.consume(start + Duration::from_millis(620), period),
            0
        );
        cadence.update(false, start, period);
        assert_eq!(cadence.deadline(), None);
    }
}
