//! Retained command values shared by preparation and supervised execution.
//! Deliberately has no Debug implementation: environment values may be secrets.

use super::*;

#[derive(PartialEq, Eq)]
pub(super) struct FrozenCommand {
    program: PathBuf,
    workdir: PathBuf,
    arguments: Vec<OsString>,
    environment: BTreeMap<OsString, OsString>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_values_survive_source_changes() {
        let root = tempfile::tempdir().expect("fixture");
        let program = root.path().join("git");
        let mut arguments = vec![OsString::from("pull"), OsString::from("upstream")];
        let mut environment = BTreeMap::from([(OsString::from("BOUND"), OsString::from("before"))]);
        let frozen = FrozenCommand::new(&program, root.path(), &arguments, &environment)
            .expect("frozen command");
        arguments[1] = "other".into();
        environment.insert("BOUND".into(), "after".into());

        let command = frozen.command();
        let command = command.as_std();
        assert_eq!(command.get_program(), program);
        assert_eq!(command.get_current_dir(), Some(root.path()));
        assert_eq!(command.get_args().collect::<Vec<_>>(), ["pull", "upstream"]);
        assert_eq!(
            command.get_envs().collect::<Vec<_>>(),
            [(
                std::ffi::OsStr::new("BOUND"),
                Some(std::ffi::OsStr::new("before"))
            )],
        );
    }
}

impl FrozenCommand {
    /// Keep Git's native upstream selection, integration and recursion policy.
    /// This describes the future operation; it confers no execution authority.
    pub(super) fn pull(
        program: &Path,
        workdir: &Path,
        input: &GitPullInput,
        environment: &BTreeMap<OsString, OsString>,
    ) -> Result<Self, GitToolError> {
        let mut arguments: Vec<OsString> = [
            "-c",
            "maintenance.autoDetach=false",
            "-c",
            "gc.autoDetach=false",
            "--no-pager",
            "pull",
        ]
        .into_iter()
        .map(OsString::from)
        .collect();
        if let Some(remote) = &input.remote {
            super::super::validate_git_positional(remote)?;
            arguments.extend([OsString::from("--"), OsString::from(remote)]);
        }
        Self::new(program, workdir, arguments, environment)
    }

    pub(super) fn new(
        program: &Path,
        workdir: &Path,
        arguments: impl IntoIterator<Item = impl AsRef<std::ffi::OsStr>>,
        environment: &BTreeMap<OsString, OsString>,
    ) -> Result<Self, GitToolError> {
        if !program.is_absolute() || !workdir.is_absolute() {
            return Err(GitToolError::InvalidInput(
                "Git command requires bound absolute executable and working directory".into(),
            ));
        }
        Ok(Self {
            program: program.to_owned(),
            workdir: workdir.to_owned(),
            arguments: arguments
                .into_iter()
                .map(|arg| arg.as_ref().to_owned())
                .collect(),
            environment: environment.clone(),
        })
    }

    // Construct exclusively from retained values, never the current process
    // environment. Identity and permission proof checks belong to the caller;
    // retaining command values alone does not authorize execution.
    pub(super) fn command(&self) -> tokio::process::Command {
        let mut command = tokio::process::Command::new(&self.program);
        command
            .current_dir(&self.workdir)
            .args(&self.arguments)
            .env_clear()
            .envs(&self.environment);
        command
    }
}
