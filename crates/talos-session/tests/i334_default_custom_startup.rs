//! I334 E3a: default CLI startup reconciliation is not the custom SDK constructor.
//!
//! A child test process isolates HOME/USERPROFILE precedence from parallel tests,
//! while exercising the actual public SessionManager::new() constructor.

use std::process::Command;

use talos_core::message::Message;
use talos_session::SessionManager;
use uuid::Uuid;

const CHILD_ENV: &str = "TALOS_I334_STARTUP_RECONCILIATION_CHILD";
const FIRST_ID_ENV: &str = "TALOS_I334_STARTUP_FIRST_ID";
const SECOND_ID_ENV: &str = "TALOS_I334_STARTUP_SECOND_ID";

#[test]
fn i334_default_vs_custom_startup_reconciliation() {
    if std::env::var_os(CHILD_ENV).is_some() {
        let manager = SessionManager::new().expect("default manager starts in isolated home");
        let first_id = Uuid::parse_str(&std::env::var(FIRST_ID_ENV).expect("first session ID"))
            .expect("valid first session UUID");
        let second_id = Uuid::parse_str(&std::env::var(SECOND_ID_ENV).expect("second session ID"))
            .expect("valid second session UUID");

        let indexed = manager.list_recent(10).expect("default startup index");
        assert_eq!(
            indexed.len(),
            2,
            "default startup recovers both transcripts"
        );
        for (id, expected_count) in [(first_id, 2), (second_id, 1)] {
            let row = indexed.iter().find(|row| row.id == id).expect("indexed ID");
            assert_eq!(
                row.message_count, expected_count,
                "startup repairs stale or missing message counts"
            );
            let session = manager.get_session(&id).expect("recovered TLOG session");
            assert_eq!(
                session.read_messages().expect("reopened transcript").len(),
                expected_count,
                "index recovery must not replace the durable transcript"
            );
        }
        return;
    }

    let home = tempfile::tempdir().expect("isolated home");
    let sessions_dir = home.path().join(".talos").join("sessions");
    let custom = SessionManager::with_dir(sessions_dir.clone());

    let first = custom
        .create_session("i334", "workspace-i334")
        .expect("first durable session");
    first
        .append(&Message::User {
            content: "first indexed turn".into(),
        })
        .expect("first message");
    custom.update_index(&first).expect("seed an earlier index");
    first
        .append(&Message::Assistant {
            content: "unindexed later turn".into(),
            tool_calls: vec![],
            reasoning: None,
        })
        .expect("second message");

    let second = custom
        .create_session("i334", "workspace-i334")
        .expect("second durable session");
    second
        .append(&Message::User {
            content: "missing from index".into(),
        })
        .expect("unindexed message");
    drop(custom);

    // Constructing with_dir must preserve the existing stale/partial index.
    let custom_again = SessionManager::with_dir(sessions_dir.clone());
    let before = custom_again.list_recent(10).expect("read stale index");
    assert_eq!(before.len(), 1, "custom construction never scans TLOGs");
    assert_eq!(before[0].id, first.id);
    assert_eq!(before[0].message_count, 1, "stale count remains stale");
    drop(custom_again);

    // Running the actual default constructor in a child prevents global HOME
    // mutation and cross-test races, including on Windows runners.
    let output = Command::new(std::env::current_exe().expect("test binary"))
        .arg("--exact")
        .arg("i334_default_vs_custom_startup_reconciliation")
        .arg("--nocapture")
        .env(CHILD_ENV, "1")
        .env(FIRST_ID_ENV, first.id.to_string())
        .env(SECOND_ID_ENV, second.id.to_string())
        .env("HOME", home.path())
        .output()
        .expect("isolated default startup child");
    assert!(
        output.status.success(),
        "default constructor failed; stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // The default manager repaired the colocated index without changing the
    // explicit custom path or inventing a second Session store.
    let after = SessionManager::with_dir(sessions_dir)
        .list_recent(10)
        .expect("reopen reconciled index");
    assert_eq!(after.len(), 2);
    assert!(
        after
            .iter()
            .any(|row| row.id == first.id && row.message_count == 2)
    );
    assert!(
        after
            .iter()
            .any(|row| row.id == second.id && row.message_count == 1)
    );
}
