// Keep configuration diagnostics separate from executable identity tests.
#[tokio::test]
async fn sender_configuration_sources_remain_readable_after_rebinding() {
    let root = fixture();
    let plan = Box::new(prepare(root.path()))
        .request_sender_program()
        .await
        .expect("program discovery");
    let admission = proof(&plan);
    let tokens = admission
        .consume_for_prepared(&plan.request(), &plan.identity)
        .expect("fresh program and configuration discovery admission");
    let candidate = plan.sender_program_candidate.as_ref().expect("candidate");
    let admitted = plan.receiver_read_path(candidate, &tokens).expect("program Read");
    let bound = ReceiverChildExecutable::capture(candidate, admitted).expect("bound program");
    let mut diagnostics = Vec::new();
    let mut defaults_readable = true;
    // Probe both already-admitted spellings of the same retained executable.
    // In particular, Windows canonicalization adds a verbatim path prefix;
    // diagnose its effect without changing production execution or identity.
    for (program_label, program) in [
        ("entry", &plan.program),
        ("internal", &bound.canonical),
        ("internal-admitted", &bound.spelling),
    ] {
        bound.revalidate(admitted).expect("retained executable");
        for scope in ["default", "system", "global", "local"] {
            let mut command = tokio::process::Command::new(program);
            command
                .current_dir(&plan.repository.workdir)
                .args(["--no-pager", "config", "--show-origin", "--null", "--list", "--includes"])
                .env_clear()
                .envs(&plan.environment);
            if scope != "default" {
                command.arg(format!("--{scope}"));
            }
            let output = plan.run_command_output(
                command, std::time::Duration::from_secs(10), 256 * 1024,
            ).await;
            match output {
                Ok(output) => {
                    if scope == "default" {
                        defaults_readable &= output.status.success();
                    }
                    // Neither stream nor program/configuration paths may appear
                    // in a CI failure. Missing explicit global/system files can
                    // legitimately fail even when default discovery succeeds.
                    diagnostics.push(format!(
                        "{program_label}/{scope}: code={:?}, stdout_bytes={}, stderr_bytes={}",
                        output.status.code(), output.stdout.len(), output.stderr.len(),
                    ));
                }
                Err(_) => {
                    defaults_readable = false;
                    diagnostics.push(format!("{program_label}/{scope}: runner-error"));
                }
            }
        }
    }
    assert!(defaults_readable, "{}", diagnostics.join("; "));
}
