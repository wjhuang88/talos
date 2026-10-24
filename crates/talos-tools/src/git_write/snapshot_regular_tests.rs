use super::*;

#[test]
fn streaming_revalidation_checks_chunks_length_identity_and_absence() {
    let root = tempfile::tempdir().expect("fixture");
    let path = root.path().join("source");
    let bytes = vec![b'a'; 3 * 8192 + 17];
    std::fs::write(&path, &bytes).expect("source");
    let evidence = RegularEvidence::capture(
        &path,
        &mut Budget {
            entries: 1,
            bytes: bytes.len(),
        },
    )
    .expect("capture");
    evidence.revalidate(&path).expect("unchanged");
    for offset in [0, 8192, bytes.len() - 1] {
        let mut changed = bytes.clone();
        changed[offset] = b'b';
        std::fs::write(&path, changed).expect("same-length change");
        assert!(evidence.revalidate(&path).is_err());
        std::fs::write(&path, &bytes).expect("restore bytes");
    }
    std::fs::write(&path, &bytes[..bytes.len() - 1]).expect("shorten");
    assert!(evidence.revalidate(&path).is_err());
    std::fs::write(&path, [&bytes[..], b"extra"].concat()).expect("extend");
    assert!(evidence.revalidate(&path).is_err());
    std::fs::write(&path, &bytes).expect("restore");
    evidence.revalidate(&path).expect("unchanged again");
    std::fs::rename(&path, root.path().join("saved")).expect("move original");
    std::fs::write(&path, &bytes).expect("same bytes, different identity");
    assert!(evidence.revalidate(&path).is_err());
    std::fs::remove_file(&path).expect("remove replacement");
    assert!(evidence.revalidate(&path).is_err());
    let missing = RegularEvidence::capture(
        &path,
        &mut Budget {
            entries: 1,
            bytes: 0,
        },
    )
    .expect("missing evidence");
    missing.revalidate(&path).expect("still missing");
    std::fs::write(&path, b"").expect("created empty file");
    assert!(missing.revalidate(&path).is_err());
}

#[cfg(unix)]
#[test]
fn streaming_revalidation_rejects_permissions_links_and_special_leaves() {
    use std::os::unix::fs::PermissionsExt;
    let root = tempfile::tempdir().expect("fixture");
    let path = root.path().join("source");
    std::fs::write(&path, b"bytes").expect("source");
    let evidence = RegularEvidence::capture(
        &path,
        &mut Budget {
            entries: 1,
            bytes: 5,
        },
    )
    .expect("capture");
    let original = std::fs::metadata(&path).expect("metadata").permissions();
    std::fs::set_permissions(
        &path,
        std::fs::Permissions::from_mode(original.mode() ^ 0o100),
    )
    .expect("change mode");
    assert!(evidence.revalidate(&path).is_err());
    std::fs::set_permissions(&path, original).expect("restore mode");
    evidence.revalidate(&path).expect("restored");
    std::fs::rename(&path, root.path().join("saved")).expect("move source");
    std::os::unix::fs::symlink("saved", &path).expect("link");
    assert!(evidence.revalidate(&path).is_err());
    std::fs::remove_file(&path).expect("remove link");
    std::fs::create_dir(&path).expect("directory");
    assert!(evidence.revalidate(&path).is_err());
}
