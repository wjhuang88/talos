#[path = "support/lifecycle_clock.rs"]
mod lifecycle_clock;

use std::time::Duration;

#[tokio::test(start_paused = true)]
async fn real_readiness_slower_than_execution_deadline_does_not_expire_it() {
    let clock = lifecycle_clock::ReadinessClock::hold().await;
    let start = tokio::time::Instant::now();
    let deadline = tokio::time::sleep(Duration::from_millis(5));
    tokio::pin!(deadline);
    let (ready, readiness) = tokio::sync::oneshot::channel();
    // An ordinary OS thread models process readiness without independently inhibiting Tokio.
    let worker = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(30));
        let _ = ready.send(());
    });
    tokio::select! {
        _ = &mut deadline => panic!("execution deadline expired before real readiness"),
        result = readiness => {
            result.expect("slow readiness worker signals");
        }
    }
    worker.join().expect("slow readiness worker joins");
    assert_eq!(tokio::time::Instant::now(), start);
    assert!(!deadline.is_elapsed());
    clock
        .advance_past_execution_deadline(Duration::from_millis(5))
        .await;
    deadline.await;
    clock.release().await;
}

#[tokio::test(start_paused = true)]
async fn delayed_std_deadline_is_crossed_after_real_readiness() {
    let clock = lifecycle_clock::ReadinessClock::hold().await;
    let start = tokio::time::Instant::now();
    let execution_timeout = Duration::from_millis(5);
    let (tx, rx) = tokio::sync::oneshot::channel();
    let worker = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(30));
        let deadline = std::time::Instant::now() + execution_timeout;
        tx.send(deadline).expect("deadline receiver");
    });
    let deadline = tokio::time::Instant::from_std(rx.await.expect("admission deadline"));
    worker.join().expect("admission worker");
    assert_eq!(tokio::time::Instant::now(), start);
    assert!(deadline > start + execution_timeout);
    clock
        .advance_past_execution_deadline(execution_timeout)
        .await;
    assert!(tokio::time::Instant::now() >= deadline);
    clock.release().await;
    tokio::time::sleep_until(deadline).await;
}
