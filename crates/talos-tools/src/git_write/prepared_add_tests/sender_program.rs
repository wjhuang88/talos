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

#[tokio::test]
async fn push_program_rebind_preserves_sender_effects_without_receiver_reads() {
    let root = fixture();
    let receiver = root.path().join("unopened-receiver.git");
    let plan = AddPlan::prepare_push(
        root.path(),
        &serde_json::json!({"remote": receiver}),
        ToolProvenance::Native,
    )
    .expect("plan");
    let mut plan = Box::new(plan).discover().await.expect("sender discovery");
    if !cfg!(windows) {
        plan.add_push_program_admission().expect("rebind frontier");
    }
    assert_eq!(
        plan.facets
            .iter()
            .filter(|facet| facet.nature == ToolNature::Execute)
            .map(|facet| facet.resource.as_deref())
            .collect::<Vec<_>>(),
        vec![
            plan.maintenance_program
                .as_ref()
                .and_then(|path| path.to_str())
        ],
    );
    plan.command_budget.lock().expect("budget").commands = 20;
    let admission = proof(&plan);
    let authorizations = admission
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("fresh admission");
    let plan = plan
        .rebind_push_program(&authorizations)
        .await
        .expect("Push re-discovery");
    assert!(plan.phase == Phase::ReceiverRoots);
    assert!(plan.receiver_layouts.is_empty());
    assert!(plan.sender_entry_program.is_some());
    assert!(!receiver.exists());
    assert!(plan.command_budget.lock().expect("budget").commands < 20);
    assert!(
        !plan
            .facets
            .iter()
            .any(|facet| facet.resource.as_deref() == Some("git:configuration-chain"))
    );
    assert!(
        plan.sender_mutation_facets
            .iter()
            .any(|facet| facet.nature == ToolNature::Execute
                && facet.resource.as_deref() == plan.program.to_str())
    );
}

#[tokio::test]
async fn push_program_rebind_rejects_changed_destination_plan() {
    let root = fixture();
    let receiver = root.path().join("unopened-receiver.git");
    let plan = AddPlan::prepare_push(
        root.path(),
        &serde_json::json!({"remote": receiver}),
        ToolProvenance::Native,
    )
    .expect("plan");
    let mut plan = Box::new(plan).discover().await.expect("sender discovery");
    if !cfg!(windows) {
        plan.add_push_program_admission().expect("rebind frontier");
    }
    plan.push_destinations
        .push("git://changed.invalid/repository".into());
    let admission = proof(&plan);
    let authorizations = admission
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("fresh admission");
    let error = match plan.rebind_push_program(&authorizations).await {
        Ok(_) => panic!("changed destination accepted"),
        Err(error) => error,
    };
    assert!(
        error.to_string().contains("destinations changed"),
        "{error}"
    );
    assert!(!receiver.exists());
}
