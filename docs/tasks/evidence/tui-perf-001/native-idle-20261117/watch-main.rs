#[path = "git_watch.rs"]
mod git_watch;
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let root = std::env::args().nth(1).expect("workspace");
    let start = std::time::Instant::now();
    let mut watch = git_watch::Watch::start(root.into())
        .expect("spawn")
        .expect("slot");
    let initial = tokio::time::timeout(std::time::Duration::from_secs(5), watch.changed())
        .await
        .expect("setup timeout")
        .expect("setup result");
    println!(
        "setup_seconds={},healthy={}",
        start.elapsed().as_secs_f64(),
        initial.healthy
    );
    assert!(initial.healthy, "native watcher setup failed");
    let until = tokio::time::Instant::now() + std::time::Duration::from_secs(3);
    let mut updates = 0;
    loop {
        match tokio::time::timeout_at(until, watch.changed()).await {
            Err(_) => break,
            Ok(Some(state)) => {
                assert!(state.healthy);
                updates += 1;
            }
            Ok(None) => panic!("watch stopped"),
        }
    }
    println!("observed_seconds=3,notifications={updates}");
}
