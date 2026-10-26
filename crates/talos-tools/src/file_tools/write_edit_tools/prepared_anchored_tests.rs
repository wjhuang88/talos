use super::tests::admit;
use super::*;

#[tokio::test]
async fn retained_anchored_plan_reviews_exact_result_and_checks_freshness() {
    for scenario in [
        "allow",
        "changed_before",
        "changed_after",
        "invalidated",
        "oversized",
        "bad_anchor",
        "overlap",
    ] {
        let root = tempfile::tempdir().expect("root");
        let path = root.path().join("file.txt");
        let original = "before\nafter\n";
        std::fs::write(&path, original).expect("write");
        let (read, _, edit, _) =
            crate::file_tools::snapshot_aware_file_tools(root.path().to_owned());
        let result = read.execute(serde_json::json!({"path":"file.txt"})).await;
        assert!(!result.is_error, "{}", result.content);
        let mut lines = result.content.lines();
        let snapshot = lines
            .next()
            .expect("snapshot")
            .strip_prefix("[snapshot:")
            .and_then(|s| s.strip_suffix(']'))
            .expect("id");
        let target = lines
            .next()
            .expect("line")
            .split_once('|')
            .expect("anchor")
            .0;
        let content = if scenario == "oversized" {
            "x".repeat(talos_core::tool::MAX_FILE_MUTATION_REVIEW_BYTES + 1)
        } else {
            "changed".into()
        };
        let operation = serde_json::json!({"op":"replace","target":if scenario == "bad_anchor" { "1:zz" } else { target },"content":content});
        let operations = if scenario == "overlap" {
            vec![operation.clone(), operation]
        } else {
            vec![operation]
        };
        let input = serde_json::json!({"path":"file.txt","snapshot_id":snapshot,"retain_backup":true,"operations":operations});
        let prepared = prepare(&edit, &input, ToolProvenance::Native)
            .expect("prepare")
            .expect("plan");
        assert!(prepared.prepared_file_mutation_evidence().is_none());
        if scenario == "changed_before" {
            std::fs::write(&path, "concurrent update").expect("change");
        }
        let proof = admit(root.path(), prepared.as_ref(), &input);
        let stage = prepared.execute_stage(proof).await;
        if matches!(
            scenario,
            "changed_before" | "oversized" | "bad_anchor" | "overlap"
        ) {
            let PreparedNormalStage::Complete(output) = stage else {
                panic!("capture must refuse")
            };
            assert!(output.result.is_error, "{scenario}");
            assert_eq!(
                std::fs::read_to_string(&path).expect("preserved"),
                if scenario == "changed_before" {
                    "concurrent update"
                } else {
                    original
                }
            );
            continue;
        }
        let PreparedNormalStage::Continue(prepared) = stage else {
            panic!("capture")
        };
        let evidence = prepared
            .prepared_file_mutation_evidence()
            .expect("evidence");
        assert_eq!(evidence.description().before, original);
        assert_eq!(
            evidence.description().after.as_deref(),
            Some("changed\nafter\n")
        );
        if scenario == "changed_after" {
            std::fs::write(&path, "concurrent update").expect("change");
        }
        if scenario == "invalidated" {
            edit.snapshots
                .as_ref()
                .expect("registry")
                .invalidate_path(&path.canonicalize().expect("path"))
                .expect("invalidate");
        }
        let proof = admit(root.path(), prepared.as_ref(), &input);
        let PreparedNormalStage::Complete(output) = prepared.execute_stage(proof).await else {
            panic!("execution")
        };
        if scenario != "allow" {
            assert!(output.result.is_error, "{scenario}");
            assert_eq!(
                std::fs::read_to_string(&path).expect("preserved"),
                if scenario == "changed_after" {
                    "concurrent update"
                } else {
                    original
                }
            );
            continue;
        }
        assert!(!output.result.is_error, "{}", output.result.content);
        assert_eq!(
            std::fs::read_to_string(&path).expect("edited"),
            "changed\nafter\n"
        );
        let id = output
            .result
            .content
            .lines()
            .find_map(|line| line.strip_prefix("recovery_id: "))
            .expect("receipt");
        let result = edit
            .execute(serde_json::json!({"path":"file.txt","recovery_id":id}))
            .await;
        assert!(!result.is_error, "{}", result.content);
        assert_eq!(std::fs::read_to_string(&path).expect("restored"), original);
    }
}
