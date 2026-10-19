#[test]
fn windows_internal_program_report_matches_only_retained_canonical_spelling() {
    let bound = Path::new(r"\\?\C:\Program Files\Git\ucrt64\libexec\git-core\git.exe");
    assert!(windows_bound_program_spelling(
        Path::new(r"//?/C:/Program Files/Git/ucrt64/libexec/git-core\git.exe"),
        bound,
    ));
    for reported in [
        "C:/Program Files/Git/ucrt64/libexec/git-core/git.exe",
        "//?/C:/Program Files/Git/ucrt64/libexec/git-core/../git-core/git.exe",
        "//?/C:/Program Files/Git/ucrt64/libexec/git-core/other.exe",
        "//?/c:/Program Files/Git/ucrt64/libexec/git-core/git.exe",
        "//./C:/Program Files/Git/ucrt64/libexec/git-core/git.exe",
    ] {
        assert!(!windows_bound_program_spelling(Path::new(reported), bound));
    }
}

#[test]
fn internal_program_validation_distinguishes_unadmitted_path_from_target_drift() {
    let root = tempfile::tempdir().expect("fixture");
    let admitted = root.path().join("admitted");
    let unadmitted = root.path().join("unadmitted-missing");
    std::fs::write(&admitted, b"fixture").expect("program fixture");
    let bound = admitted.canonicalize().expect("bound target");
    validate_internal_program(&admitted, &admitted, &bound).expect("unchanged binding");
    #[cfg(windows)]
    {
        let reported = bound.to_str().expect("fixture path").replace('\\', "/");
        validate_internal_program(Path::new(&reported), &admitted, &bound)
            .expect("Git for Windows canonical report");
        let missing = root.path().join("missing-admitted");
        let error = validate_internal_program(Path::new(&reported), &missing, &bound)
            .expect_err("canonical report must not bypass admitted path revalidation");
        assert!(error.to_string().contains("cannot be resolved"), "{error}");
    }
    let error = validate_internal_program(&unadmitted, &admitted, &bound)
        .expect_err("unadmitted spelling");
    assert!(error.to_string().contains("spelling changed"), "{error}");
    let error = validate_internal_program(&admitted, &admitted, &unadmitted)
        .expect_err("different canonical target");
    assert!(error.to_string().contains("canonical target changed"), "{error}");
    std::fs::remove_file(&admitted).expect("remove admitted program");
    let error = validate_internal_program(&admitted, &admitted, &bound)
        .expect_err("missing admitted executable");
    assert!(error.to_string().contains("cannot be resolved"), "{error}");
}

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

#[tokio::test]
async fn configuration_failure_reports_status_without_configuration_contents() {
    let root = fixture();
    let mut plan = prepare(root.path());
    let invalid = root.path().join("invalid-config");
    std::fs::write(&invalid, "[private]\ncredential=DO_NOT_DISCLOSE\n[broken\n")
        .expect("invalid configuration");
    plan.environment.insert("GIT_CONFIG_GLOBAL".into(), invalid.into_os_string());
    let error = plan.inspect_config().await.expect_err("invalid configuration").to_string();
    assert!(error.contains("exited unsuccessfully"), "{error}");
    assert!(error.contains("code=Some("), "{error}");
    assert!(error.contains("stderr_bytes="), "{error}");
    assert!(!error.contains("DO_NOT_DISCLOSE"));
    assert!(!error.contains("invalid-config"));
    assert!(!error.contains("private"));
}
