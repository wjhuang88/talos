#[test]
fn pull_integration_signing_requires_its_own_execution_admission() {
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
                config.validate_pull_integration_signing_mode(true, false).is_err(),
                enabled
            );
            assert_eq!(
                config.validate_pull_integration_signing_mode(false, true).is_err(),
                enabled && key == b"commit.gpgsign",
            );
            // read-tree/update-ref do not create or verify signed commits.
            config
                .validate_pull_integration_signing_mode(false, false)
                .expect("no commit creation or verification");
        }
        config.values.insert(
            key.to_vec(),
            vec![
                Some(b"invalid-secret-value".to_vec()),
                Some(b"false".to_vec()),
            ],
        );
        let error = config
            .validate_pull_integration_signing_mode(true, false)
            .expect_err("invalid early value");
        assert!(!error.to_string().contains("invalid-secret-value"));
    }
}
