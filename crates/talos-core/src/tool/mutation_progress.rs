//! Invocation-local evidence for repeat protection, never permission authority.
use std::cell::Cell;
use std::future::Future;

tokio::task_local! {
    static CHANGED: Cell<bool>;
}

/// Runs one invocation and reports whether a built-in file operation changed state.
///
/// This observes only the supplied future, not spawned tasks. Callers must also
/// verify built-in registration and the original successful result. An observed
/// change is not semantic progress, a file revision, or an authorization grant.
/// Cancellation and panic discard this scope; nested invocations are isolated.
pub async fn observe_file_mutation<F: Future>(invocation: F) -> (F::Output, bool) {
    CHANGED
        .scope(Cell::new(false), async {
            let result = invocation.await;
            (result, CHANGED.with(Cell::get))
        })
        .await
}

// Called on the invocation task after the filesystem operation has completed.
// Safe outside an observer (standalone tools retain their existing behavior).
/// Records a completed filesystem change in the current observation scope.
///
/// Producers must compare actual bytes for edits and call after the operation
/// succeeds. This is advisory evidence only: consumers must verify registration
/// and the original result. Outside an observer this is a no-op.
pub fn record_file_mutation() {
    let _ = CHANGED.try_with(|changed| changed.set(true));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mutation_observation_is_local_and_does_not_leak_after_cancellation() {
        record_file_mutation(); // Safe without an observer.
        let ((_, inner), outer) = observe_file_mutation(async {
            observe_file_mutation(async {
                record_file_mutation();
            })
            .await
        })
        .await;
        assert!(inner);
        assert!(!outer);
        let (changed, unchanged) = tokio::join!(
            observe_file_mutation(async {
                tokio::task::yield_now().await;
                record_file_mutation();
            }),
            observe_file_mutation(async {
                tokio::task::yield_now().await;
            }),
        );
        assert!(changed.1);
        assert!(!unchanged.1);
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(observe_file_mutation(async {
            record_file_mutation();
            let _ = ready_tx.send(());
            std::future::pending::<()>().await;
        }));
        ready_rx.await.expect("observing");
        task.abort();
        assert!(task.await.expect_err("cancelled").is_cancelled());
        let panicked = tokio::spawn(observe_file_mutation(async {
            record_file_mutation();
            panic!("fixture panic");
        }));
        assert!(panicked.await.expect_err("panic isolated").is_panic());
        assert!(!observe_file_mutation(async {}).await.1);
    }
}
