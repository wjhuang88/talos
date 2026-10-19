#[test]
fn pull_integration_signing_requires_its_own_execution_admission() {
    use super::super::pull_state::{FastForward, Integration, Rebase};

    let merge = Integration::Merge {
        heads: vec!["1".repeat(40)],
        fast_forward: FastForward::Unspecified,
        autostash: false,
    };
    let rebase = Integration::Rebase {
        onto: "1".repeat(40),
        upstream: "2".repeat(40),
        mode: Rebase::On,
        autostash: false,
    };
    let unborn = Integration::Unborn("1".repeat(40));
    let reconcile = Integration::Reconcile {
        original: "1".repeat(40),
        current: "2".repeat(40),
    };
    let mut config = ConfigState {
        values: Values::new(),
        ordered_output: Vec::new(),
        origins: BTreeSet::new(),
        files: Vec::new(),
        identities: Vec::new(),
        auxiliary: BTreeSet::new(),
        stable_auxiliary: BTreeSet::new(),
        hooks: Vec::new(),
    };
    for key in [b"commit.gpgsign".as_slice(), b"merge.verifysignatures"] {
        for (entries, enabled) in [
            (vec![None], true),
            (vec![Some(b"true".to_vec())], true),
            (vec![Some(b"2".to_vec())], true),
            (vec![Some(b"false".to_vec())], false),
            (vec![Some(b"0".to_vec())], false),
            (vec![Some(b"true".to_vec()), Some(b"false".to_vec())], false),
        ] {
            config.values.clear();
            config.values.insert(key.to_vec(), entries);
            assert_eq!(
                config.validate_pull_integration_signing(&merge).is_err(),
                enabled
            );
            assert_eq!(
                config.validate_pull_integration_signing(&rebase).is_err(),
                enabled && key == b"commit.gpgsign",
            );
            // read-tree/update-ref do not create or verify signed commits.
            config
                .validate_pull_integration_signing(&unborn)
                .expect("unborn");
            config
                .validate_pull_integration_signing(&reconcile)
                .expect("reconcile");
        }
        config.values.insert(
            key.to_vec(),
            vec![
                Some(b"invalid-secret-value".to_vec()),
                Some(b"false".to_vec()),
            ],
        );
        let error = config
            .validate_pull_integration_signing(&merge)
            .expect_err("invalid early value");
        assert!(!error.to_string().contains("invalid-secret-value"));
    }
}
