use super::encode;

fn encoded(arguments: &[&str]) -> String {
    let arguments = arguments
        .iter()
        .map(|arg| arg.encode_utf16().collect())
        .collect::<Vec<_>>();
    let command = encode(&arguments).expect("valid fixture");
    String::from_utf16(&command[..command.len() - 1]).expect("Unicode fixture")
}

#[test]
fn quoting_handles_empty_quotes_and_trailing_backslashes() {
    assert_eq!(encoded(&["git", "", "two words"]), "git \"\" \"two words\"");
    assert_eq!(encoded(&["git", "a\"b"]), "git \"a\\\"b\"");
    assert_eq!(encoded(&["git", "a b\\"]), "git \"a b\\\\\"");
    assert_eq!(encoded(&["git", "a\\b"]), "git a\\b");
}

#[test]
fn preserves_unpaired_utf16_without_replacement() {
    assert_eq!(
        encode(&[vec![0xd800, 32, 0xdc00]]).expect("raw Windows text"),
        vec![34, 0xd800, 32, 0xdc00, 34, 0]
    );
}

#[test]
fn rejects_nul_and_enforces_limit_including_terminator() {
    assert!(encode(&[]).is_err());
    assert!(encode(&[vec![65, 0, 66]]).is_err());
    assert!(encode(&[vec![65; 32_766]]).is_ok());
    assert!(encode(&[vec![65; 32_767]]).is_err());
    // Escaping, inter-argument spaces, and quote delimiters also consume space.
    assert!(encode(&[vec![34; 16_383]]).is_err());
    assert!(encode(&[vec![65; 32_765], vec![66]]).is_err());
}
