use super::super::Values;
use super::*;

#[test]
fn branch_option_split_matches_native_git_alias_tokenization() {
    // Aliases and branch.mergeOptions call the same native split_cmdline.
    // Compare with a direct argv invocation, so quoting the expected output
    // does not require a second hand-written shell parser.
    let root = tempfile::tempdir().expect("fixture");
    for input in [
        "",
        "--ff",
        " --ff",
        "--ff ",
        "--ff  --no-edit",
        "'--no-edit' \"--ff\"",
        "--no-''edit",
        "#literal",
        "NAME=value",
        "one\\ two",
        "one\\\ntwo",
        "'one\\two'",
        "\"one\\two\"",
        "one\t\r\ntwo",
        "''",
        "\"\"",
        "a' b'c",
        "'a\"b'",
        "\"a'b\"",
        "a\u{b}b\u{c}c",
    ] {
        let alias = format!("rev-parse --sq-quote {input}");
        let parsed = split(alias.as_bytes()).expect("valid quoting");
        let expected = std::process::Command::new("git")
            .current_dir(root.path())
            .args(["rev-parse", "--sq-quote"])
            .args(
                parsed
                    .iter()
                    .skip(2)
                    .map(|arg| std::str::from_utf8(arg).expect("ASCII")),
            )
            .output()
            .expect("direct argv");
        let actual = std::process::Command::new("git")
            .current_dir(root.path())
            .args(["-c", &format!("alias.i300probe={alias}"), "i300probe"])
            .output()
            .expect("native split_cmdline");
        assert!(
            expected.status.success() && actual.status.success(),
            "{input:?}"
        );
        assert_eq!(actual.stdout, expected.stdout, "{input:?}");
    }
    for input in ["one\\", "'one", "\"one"] {
        assert!(split(input.as_bytes()).is_err());
        let actual = std::process::Command::new("git")
            .current_dir(root.path())
            .args([
                "-c",
                &format!("alias.i300probe=rev-parse --sq-quote {input}"),
                "i300probe",
            ])
            .output()
            .expect("native malformed alias");
        assert!(!actual.status.success());
    }
    assert!(split(b"one\0two").is_err());
}

#[test]
fn only_selected_branch_options_enter_integration_effect_validation() {
    let mut config = ConfigState {
        values: Values::new(),
        ordered_output: Vec::new(),
        origins: Default::default(),
        files: Vec::new(),
        identities: Vec::new(),
        auxiliary: Default::default(),
        stable_auxiliary: Default::default(),
        hooks: Vec::new(),
    };
    for key in [b"pull.twohead".as_slice(), b"pull.octopus".as_slice()] {
        for value in ["ort", "recursive", "subtree", "ort recursive"] {
            config
                .values
                .insert(key.to_vec(), vec![Some(value.as_bytes().to_vec())]);
            config
                .validate_pull_merge_strategy()
                .expect("Git strategies");
        }
        for value in [
            "octopus",
            "resolve",
            "ours",
            "secret",
            "ort secret",
            " ort",
            "ort ",
            "ort  recursive",
            "'ort'",
            "ort\trecursive",
            "",
        ] {
            config
                .values
                .insert(key.to_vec(), vec![Some(value.as_bytes().to_vec())]);
            let error = config
                .validate_pull_merge_strategy()
                .expect_err("unmodeled strategy");
            assert!(!error.to_string().contains("secret"));
        }
        config.values.insert(key.to_vec(), vec![None]);
        assert!(config.validate_pull_merge_strategy().is_err());
        config.values.insert(
            key.to_vec(),
            vec![Some(b"secret".to_vec()), Some(b"ort".to_vec())],
        );
        config
            .validate_pull_merge_strategy()
            .expect("last strategy wins");
        config.values.remove(key);
    }
    config.values.insert(
        b"branch.other.mergeoptions".to_vec(),
        vec![Some(b"-Ssecret".to_vec())],
    );
    config
        .validate_pull_merge_options(Some("Main"))
        .expect("other branch");
    for options in [
        "--ff --no-edit",
        "'--no-edit' --no-gpg-sign",
        "--no-verify-signatures",
    ] {
        config.values.insert(
            b"branch.Main.mergeoptions".to_vec(),
            vec![Some(options.as_bytes().to_vec())],
        );
        config
            .validate_pull_merge_options(Some("Main"))
            .expect("modeled flags");
    }
    for options in [
        "--gpg-sign=secret",
        "-Ssecret",
        "--edit",
        "-e",
        "--file=secret",
        "-Fsecret",
        "--strategy=secret",
        "-s secret",
        "--ver",
        "--no-edit --edit",
        "--quit",
        "--continue",
        "--",
        "",
        "--ff ",
        "'unterminated-secret",
    ] {
        config.values.insert(
            b"branch.Main.mergeoptions".to_vec(),
            vec![Some(options.as_bytes().to_vec())],
        );
        let error = config
            .validate_pull_merge_options(Some("Main"))
            .expect_err("unmodeled effect");
        assert!(!error.to_string().contains("secret"));
        config
            .validate_pull_merge_options(Some("main"))
            .expect("case-sensitive branch");
        config
            .validate_pull_merge_options(None)
            .expect("detached HEAD");
    }
    config.values.insert(
        b"branch.Main.mergeoptions".to_vec(),
        vec![Some(b"--edit".to_vec()), Some(b"--no-edit".to_vec())],
    );
    config
        .validate_pull_merge_options(Some("Main"))
        .expect("last declaration wins");
    config
        .values
        .insert(b"branch.Main.mergeoptions".to_vec(), vec![None]);
    assert!(config.validate_pull_merge_options(Some("Main")).is_err());
}
