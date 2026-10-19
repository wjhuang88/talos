#[tokio::test]
async fn sender_program_binding_requires_new_exact_read_admission() {
    let root = fixture();
    let plan = prepare(root.path());
    let old = proof(&plan);
    let plan = Box::new(plan)
        .request_sender_program()
        .await
        .expect("path discovery");
    assert!(
        old.consume_for_prepared(&plan.request(), &plan.identity)
            .is_err()
    );
    assert!(plan.config.is_none());
    assert_eq!(plan.command_budget.lock().expect("budget").commands, 127);
    assert!(plan.discover_sender(&[]).await.is_err());
}

#[tokio::test]
async fn sender_program_binding_preserves_invocation_budget() {
    let root = fixture();
    let plan = prepare(root.path());
    plan.command_budget.lock().expect("budget").commands = 20;
    let plan = Box::new(plan)
        .request_sender_program()
        .await
        .expect("path discovery");
    let proof = proof(&plan);
    let authorizations = proof
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("fresh admission");
    let plan = plan
        .discover_sender(&authorizations)
        .await
        .expect("first discovery");
    assert!(plan.sender_program_candidate.is_none());
    assert!(plan.sender_entry_program.is_some());
    assert!(plan.config.is_some());
    assert!(plan.command_budget.lock().expect("budget").commands < 18);
    plan.revalidate().expect("bound state");
}
