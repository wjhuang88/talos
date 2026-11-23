//! Recognize the single-path Scoop Git shim mapping used by prepared Windows writes.

use super::{GitToolError, ProgramEvidence, capture_program, snapshot};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Component, Path, PathBuf, Prefix};

pub(super) struct Installation {
    pub(super) variable: (OsString, OsString),
    pub(super) shim: PathBuf,
    pub(super) mapping: PathBuf,
    pub(super) target: PathBuf,
    pub(super) evidence: Option<Binding>,
}

pub(super) struct Binding {
    mapping: snapshot::PathIdentity,
    _mapping_guard: ProgramEvidence,
    shim_identity: snapshot::PathIdentity,
    _shim_guard: ProgramEvidence,
    target: snapshot::PathIdentity,
}

type Environment = BTreeMap<OsString, OsString>;

pub(super) fn prepare(
    mut env: Environment,
    operation: &super::Operation,
) -> Result<(Environment, Option<Installation>), GitToolError> {
    let key = env
        .keys()
        .find(|k| k.eq_ignore_ascii_case("GIT_INSTALL_ROOT"))
        .cloned();
    let Some(key) = key else {
        return Ok((env, None));
    };
    if !matches!(
        operation,
        super::Operation::Commit(_) | super::Operation::Checkout(_)
    ) {
        return Ok((env, None));
    }
    let invalid = || {
        GitToolError::InvalidInput(
            "Git installation override requires a separately authorized invocation".into(),
        )
    };
    let value = env.remove(&key).ok_or_else(invalid)?;
    if env
        .iter()
        .fold(key.len().saturating_add(value.len()), |n, (k, v)| {
            n.saturating_add(k.len()).saturating_add(v.len())
        })
        > 1024 * 1024
    {
        return Err(invalid());
    }
    super::validate_environment(&env)?;
    let shim = super::resolve_git(&env)?;
    let target = target_for(&shim, Path::new(&value)).ok_or_else(invalid)?;
    let mapping = shim.with_extension("shim");
    Ok((
        env,
        Some(Installation {
            variable: (key, value),
            shim,
            mapping,
            target,
            evidence: None,
        }),
    ))
}

impl Installation {
    pub(super) fn revalidate_environment(
        &self,
        mut env: Environment,
    ) -> Result<Environment, GitToolError> {
        if env.remove(&self.variable.0).as_ref() != Some(&self.variable.1)
            || super::resolve_git(&env)? != self.shim
        {
            return Err(GitToolError::InvalidInput(
                "Scoop Git environment changed after preparation".into(),
            ));
        }
        self.revalidate()?;
        Ok(env)
    }
    pub(super) fn bind(
        &mut self,
        mapping: &Path,
        target: &Path,
        initial_shim: &ProgramEvidence,
        target_program: &ProgramEvidence,
    ) -> Result<(), GitToolError> {
        let invalid = || {
            GitToolError::InvalidInput(
                "Scoop Git mapping or installation changed; request fresh authorization".into(),
            )
        };
        if self.mapping.canonicalize().ok().as_deref() != Some(mapping)
            || self.target.canonicalize().ok().as_deref() != Some(target)
            || std::fs::metadata(mapping).map_or(true, |m| m.len() > 4096)
        {
            return Err(invalid());
        }
        let identity = snapshot::PathIdentity::capture(&self.mapping).map_err(|_| invalid())?;
        let guard = super::capture_bounded_file(mapping, 4096)?;
        if !matches!(&guard.entry, snapshot::Entry::File(_, bytes) if matches_mapping(bytes, &self.target))
        {
            return Err(invalid());
        }
        let target_identity = snapshot::PathIdentity::capture(target).map_err(|_| invalid())?;
        identity.revalidate(mapping).map_err(|_| invalid())?;
        target_identity
            .revalidate(&self.target)
            .map_err(|_| invalid())?;
        let shim = capture_program(&self.shim)?;
        let shim_id = snapshot::PathIdentity::capture(&self.shim).map_err(|_| invalid())?;
        if shim != *initial_shim
            || !shim_id
                .matches_open_file(&shim._guard)
                .map_err(|_| invalid())?
            || !shim_id
                .matches_open_file(&initial_shim._guard)
                .map_err(|_| invalid())?
            || !identity
                .matches_open_file(&guard._guard)
                .map_err(|_| invalid())?
            || !target_identity
                .matches_open_file(&target_program._guard)
                .map_err(|_| invalid())?
        {
            return Err(invalid());
        }
        self.evidence = Some(Binding {
            mapping: identity,
            _mapping_guard: guard,
            shim_identity: shim_id,
            _shim_guard: shim,
            target: target_identity,
        });
        self.revalidate()
    }

    pub(super) fn revalidate(&self) -> Result<(), GitToolError> {
        if let Some(binding) = &self.evidence
            && (binding.mapping.revalidate(&self.mapping).is_err()
                || binding.target.revalidate(&self.target).is_err()
                || binding.shim_identity.revalidate(&self.shim).is_err())
        {
            return Err(GitToolError::InvalidInput(
                "Scoop Git installation changed after admission".into(),
            ));
        }
        Ok(())
    }
}

/// Derive the only permitted target spelling from the independently selected shim.
/// Neither the environment nor the mapping can select another installation.
pub(super) fn target_for(shim: &Path, install_root: &Path) -> Option<PathBuf> {
    let raw = install_root.to_str()?;
    let mut components = install_root.components();
    if !raw
        .as_bytes()
        .get(1..3)
        .is_some_and(|prefix| prefix == b":\\")
        || raw.contains('/')
        || !matches!(components.next(), Some(Component::Prefix(prefix)) if matches!(prefix.kind(), Prefix::Disk(_)))
        || components.next() != Some(Component::RootDir)
        || raw.split('\\').skip(1).any(|part| {
            part.is_empty()
                || part.ends_with(['.', ' '])
                || part.contains([':', '/', '"'])
                || part.chars().any(char::is_control)
        })
    {
        return None;
    }
    let base = install_root.ancestors().nth(3)?;
    let expected = base.join("apps").join("git").join("current");
    if expected.as_os_str() != install_root.as_os_str() {
        return None;
    }
    let expected_shim = base.join("shims").join("git.exe");
    if shim.to_str()?.strip_prefix(r"\\?\")? != expected_shim.to_str()? {
        return None;
    }
    Some(expected.join("cmd").join("git.exe"))
}

/// Only the literal target, in Scoop's one-directive format, is admitted.
pub(super) fn matches_mapping(bytes: &[u8], target: &Path) -> bool {
    let Some(target) = target.to_str() else {
        return false;
    };
    let line = bytes
        .strip_suffix(b"\r\n")
        .or_else(|| bytes.strip_suffix(b"\n"))
        .unwrap_or(bytes);
    line.strip_prefix(b"path = \"")
        .and_then(|s| s.strip_suffix(b"\""))
        == Some(target.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binding_requires_admitted_target_and_original_mapping() {
        use std::fs;
        let dir = tempfile::tempdir().expect("test directory");
        let root = dir.path().join("apps/git/current");
        let shims = dir.path().join("shims");
        fs::create_dir_all(root.join("cmd")).expect("cmd");
        fs::create_dir(&shims).expect("shims");
        let shim = shims.join("git.exe");
        let mapping = shims.join("git.shim");
        let program = root.join("cmd/git.exe");
        fs::write(&shim, b"shim fixture").expect("shim");
        fs::write(&program, b"program fixture").expect("program");
        let shim = shim.canonicalize().expect("shim path");
        let target = program.canonicalize().expect("program path");
        let initial = capture_program(&shim).expect("shim guard");
        let selected = capture_program(&target).expect("target guard");
        let mut installation = Installation {
            variable: ("GIT_INSTALL_ROOT".into(), root.into_os_string()),
            shim: shim.clone(),
            mapping: mapping.clone(),
            target: program.clone(),
            evidence: None,
        };
        fs::write(&mapping, b"path = \"another installation\"").expect("changed mapping");
        let admitted = mapping.canonicalize().expect("mapping path");
        assert!(
            installation
                .bind(&admitted, &target, &initial, &selected)
                .is_err()
        );
        fs::write(&mapping, format!("path = \"{}\"", program.display())).expect("literal mapping");
        assert!(
            installation
                .bind(&admitted, &shim, &initial, &selected)
                .is_err()
        );
        installation
            .bind(&admitted, &target, &initial, &selected)
            .expect("bind");
        assert!(fs::write(&mapping, b"changed").is_err());
        assert!(
            installation
                .revalidate_environment(Environment::new())
                .is_err()
        );
        installation.revalidate().expect("unchanged installation");
    }

    #[test]
    fn mapping_binds_one_literal_target() {
        let target = Path::new(r"C:\scoop\apps\git\current\cmd\git.exe");
        let line = format!("path = \"{}\"", target.display());
        assert!(matches_mapping(format!("{line}\r\n").as_bytes(), target));
        assert!(!matches_mapping(
            format!("{line}\r\nargs = --version").as_bytes(),
            target
        ));
        assert!(!matches_mapping(
            format!("{line}\n{line}").as_bytes(),
            target
        ));
        assert!(!matches_mapping(
            line.as_bytes(),
            Path::new(r"C:\other\git.exe")
        ));
    }
}
