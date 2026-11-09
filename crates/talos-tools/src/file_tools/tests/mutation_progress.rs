use super::*;
use serde_json::json;
use talos_core::tool::observe_file_mutation;

#[tokio::test]
async fn real_file_changes_and_noops_have_distinct_progress() {
    let root = tempfile::tempdir().expect("root");
    let (read, write, edit, delete) = snapshot_aware_file_tools(root.path().to_owned());
    let (result, changed) =
        observe_file_mutation(write.execute(json!({"path":"file", "content":"before\n"}))).await;
    assert!(!result.is_error && changed);
    let (result, changed) =
        observe_file_mutation(write.execute(json!({"path":"file", "content":"before\n"}))).await;
    assert!(result.is_error && !changed);
    for anchored in [false, true] {
        for noop in [true, false] {
            let (result, changed) =
                observe_file_mutation(read.execute(json!({"path":"file"}))).await;
            assert!(!result.is_error && !changed);
            let mut lines = result.content.lines();
            let snapshot = lines
                .next()
                .expect("snapshot")
                .strip_prefix("[snapshot:")
                .and_then(|s| s.strip_suffix(']'))
                .expect("id");
            let anchor = lines
                .next()
                .expect("line")
                .split_once('|')
                .expect("anchor")
                .0;
            let new = if noop { "before" } else { "after" };
            let input = if anchored {
                json!({"path":"file","snapshot_id":snapshot,"operations":[{"op":"replace","target":anchor,"content":new}],"retain_backup":true})
            } else {
                json!({"path":"file","old_string":"before","new_string":new,"retain_backup":true})
            };
            let (result, changed) = observe_file_mutation(edit.execute(input)).await;
            assert!(!result.is_error, "{}", result.content);
            assert_eq!(changed, !noop);
            let id = result
                .content
                .lines()
                .find_map(|l| l.strip_prefix("recovery_id: "))
                .expect("receipt");
            let (restored, changed) =
                observe_file_mutation(edit.execute(json!({"path":"file","recovery_id":id}))).await;
            assert!(!restored.is_error, "{}", restored.content);
            assert_eq!(changed, !noop, "restoring identical bytes is not progress");
        }
    }
    let (result, changed) =
        observe_file_mutation(delete.execute(json!({"path":"file","retain_backup":true}))).await;
    assert!(!result.is_error && changed);
    let id = result
        .content
        .split("recovery_id: ")
        .nth(1)
        .expect("receipt")
        .split(';')
        .next()
        .expect("id");
    let (result, changed) =
        observe_file_mutation(delete.execute(json!({"path":"file","recovery_id":id}))).await;
    assert!(!result.is_error && changed);
    let (result, changed) = observe_file_mutation(delete.execute(json!({"path":"file"}))).await;
    assert!(!result.is_error && changed);
    let (result, changed) = observe_file_mutation(delete.execute(json!({"path":"file"}))).await;
    assert!(result.is_error && !changed);
}
