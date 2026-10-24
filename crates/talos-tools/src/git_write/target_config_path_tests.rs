//! Pure platform path matching regressions; these grant no filesystem authority.

use super::{gitdir_match_path, gitdir_matches};
#[cfg(windows)]
use std::collections::BTreeMap;
use std::path::Path;

#[cfg(windows)]
#[test]
fn admitted_windows_paths_use_git_matching_spelling() {
    for (path, expected) in [
        (r"C:\work\repo\.git", "C:/work/repo/.git"),
        (r"\\?\C:\work\repo\.git", "C:/work/repo/.git"),
        (r"\\server\share\repo\.git", "//server/share/repo/.git"),
        (
            r"\\?\UNC\server\share\repo\.git",
            "//server/share/repo/.git",
        ),
    ] {
        assert_eq!(gitdir_match_path(Path::new(path)).expect("path"), expected);
    }
    assert!(gitdir_match_path(Path::new(r"\\.\pipe\probe")).is_err());
    assert!(gitdir_match_path(Path::new(r"\\?\GLOBALROOT\Device\probe")).is_err());
}

#[cfg(windows)]
#[test]
fn condition_matching_normalizes_paths_without_changing_glob_escapes() {
    let environment = BTreeMap::from([("HOME".into(), r"\\?\C:\work".into())]);
    let origin = Path::new(r"\\?\C:\work\source");
    let gitdir = Path::new(r"\\?\C:\work\[repo]\.git");
    for pattern in [
        br"C:/work/\[repo\]/.git".as_slice(),
        br"./\[repo\]/.git",
        br"~/\[repo\]/.git",
        br"\[repo\]/.git",
    ] {
        assert!(gitdir_matches(pattern, origin, gitdir, &environment, false).expect("match"));
    }
    assert!(
        !gitdir_matches(br"C:/work/[repo]/.git", origin, gitdir, &environment, false)
            .expect("unescaped bracket pattern")
    );
}

#[cfg(unix)]
#[test]
fn unix_backslashes_and_nt_looking_names_remain_literal() {
    let gitdir = Path::new(r"/work/\\?\C:\repo/.git");
    assert_eq!(
        gitdir_match_path(gitdir).expect("path"),
        gitdir.to_str().expect("UTF-8")
    );
    assert!(
        gitdir_matches(
            br"**/\\\\\?\\C:\\repo/.git",
            Path::new("/work/source"),
            gitdir,
            &Default::default(),
            false,
        )
        .expect("literal backslashes")
    );
}

#[cfg(unix)]
#[test]
fn home_expansion_keeps_literal_dot_components() {
    let environment = std::collections::BTreeMap::from([("HOME".into(), "/work/./home".into())]);
    assert!(
        !gitdir_matches(
            b"~/.git",
            Path::new("/work/source"),
            Path::new("/work/home/.git"),
            &environment,
            false,
        )
        .expect("literal home expansion")
    );
}
