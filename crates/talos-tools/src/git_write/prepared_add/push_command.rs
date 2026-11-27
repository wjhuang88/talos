//! Native Push uses whole-command authority, including configured transports.
use super::*;

// Preserve Git's diagnostic lines while withholding URL credentials and common
// credential fields. Arbitrary helper prose cannot be proven secret-free.
pub(super) fn push_diagnostic(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let mut result = String::new();
    for line in text.lines() {
        let lower = line.to_ascii_lowercase();
        if [
            "authorization:",
            "password=",
            "password:",
            "token=",
            "token:",
            "secret=",
            "api_key",
            "private key-----",
        ]
        .iter()
        .any(|key| lower.contains(key))
        {
            result.push_str("[credential-bearing diagnostic line redacted]\n");
            continue;
        }
        for word in line.split_inclusive(char::is_whitespace) {
            let mut safe = word.to_owned();
            if let Some(scheme) = safe.find("://") {
                let start = scheme + 3;
                let end = safe[start..]
                    .find(['/', '?', '#'])
                    .map_or(safe.len(), |n| start + n);
                if let Some(at) = safe[start..end].rfind('@') {
                    safe.replace_range(start..start + at, "***");
                }
                if let Some(query) = safe.find(['?', '#']) {
                    safe.truncate(query);
                    safe.push_str("[query redacted]");
                }
            }
            result.extend(safe.chars().filter(|c| !c.is_control() || *c == '\t'));
        }
        result.push('\n');
        if result.len() > 16 * 1024 {
            let mut end = 16 * 1024;
            while !result.is_char_boundary(end) {
                end -= 1;
            }
            result.truncate(end);
            result.push_str("\n[diagnostic truncated]\n");
            break;
        }
    }
    if result.is_empty() {
        "(empty)".into()
    } else {
        result
    }
}

#[cfg(test)]
mod diagnostic_tests {
    use super::push_diagnostic;
    #[test]
    fn preserves_rejection_and_masks_credentials() {
        let output = push_diagnostic(b"To https://user:password@example.invalid/repo\n ! [rejected] main -> main (fetch first)\nerror: failed to push some refs\nAuthorization: Bearer secret\n");
        assert!(output.contains("(fetch first)"));
        assert!(output.contains("error: failed to push some refs"));
        assert!(output.contains("https://***@example.invalid/repo"));
        assert!(!output.contains("password"));
        assert!(!output.contains("Bearer secret"));
    }
}

impl AddPlan {
    pub(super) async fn discover_push_command(
        mut self: Box<Self>,
    ) -> Result<Box<Self>, GitToolError> {
        self.revalidate()?;
        let output = self.inspect_config().await?;
        let config = super::super::config_state::ConfigState::capture_pull_command(
            &output,
            &self.repository,
            &self.environment,
        )?;
        config.verify_host(&self.inspect_config().await?)?;
        // Preserve foreground maintenance for explicit all-local destinations.
        // This classification does not read receiver paths or grant authority.
        if let Operation::Push(input) = &self.operation
            && let Some(remote) = &input.remote
        {
            let destinations = super::super::remote_state::destinations(
                &super::super::remote_state::records(&output)?,
                remote,
                true,
            )?;
            let mut local = Vec::new();
            for destination in &destinations {
                let url = gix::url::parse(destination)
                    .map_err(|_| GitToolError::InvalidInput("Invalid Push destination".into()))?;
                if url.scheme != gix::url::Scheme::File {
                    local.clear();
                    break;
                }
                if let Some(path) = super::super::remote_state::local_destination(
                    destination,
                    &self.repository.workdir,
                )? {
                    local.push(path);
                }
            }
            self.receiver_roots = local;
        }
        self.config = Some(config);
        self.native_push_command = Some(self.expected_push_command()?);
        self.facets.clear();
        for root in [
            &self.repository.workdir,
            &self.repository.git_dir,
            &self.repository.common_dir,
        ] {
            self.facets.push(path_facet(
                ToolNature::Read,
                root,
                "known native Push repository root; command effects are not confined to this path",
            )?);
        }
        self.facets.push(ToolPermissionFacet::with_resource(
            ToolNature::Execute,
            self.program.to_str().ok_or_else(|| GitToolError::InvalidInput("Git program path is not UTF-8".into()))?,
            ToolResourceKind::Command,
        ).with_description("frozen native Git push argv and retained environment; configured remote/refspec defaults preserved; Trace2 disabled"));
        self.facets.push(ToolPermissionFacet::with_resource(
            ToolNature::Execute, "git:push-command", ToolResourceKind::Remote,
        ).with_description("one native Push command and its transitive network, local/remote metadata, recursion, hooks, credential/SSH/HTTP/transport helpers and configured programs; known roots are review evidence, not confinement; no automatic retry or force escalation"));
        self.phase = Phase::PushCommandMutation;
        self.identity = PreparedPlanIdentity::new();
        self.revalidate()?;
        Ok(self)
    }

    pub(super) async fn execute_push_command(&self) -> Result<String, GitToolError> {
        self.revalidate()?;
        if self.phase != Phase::PushCommandMutation
            || !matches!(self.operation, Operation::Push(_))
            || !self.facets.iter().any(|facet| {
                facet.nature == ToolNature::Execute
                    && facet.resource_kind == Some(ToolResourceKind::Remote)
                    && facet.resource.as_deref() == Some("git:push-command")
            })
        {
            return Err(GitToolError::InvalidInput(
                "Native Push command authority missing; no command started".into(),
            ));
        }
        self.config
            .as_ref()
            .ok_or_else(|| {
                GitToolError::InvalidInput("Native Push configuration evidence missing".into())
            })?
            .verify_host(&self.inspect_config().await?)?;
        self.revalidate()?;
        let command = self
            .native_push_command
            .as_ref()
            .ok_or_else(|| GitToolError::InvalidInput("Native Push command missing".into()))?;
        let output = self.run_command_output(command.command(), std::time::Duration::from_secs(120), 1024 * 1024)
            .await.map_err(|_| GitToolError::Git("Native Push execution failed; partial remote effects may remain; inspect before retrying".into()))?;
        if !output.status.success() {
            return Err(GitToolError::Git(format!(
                "Native Push failed ({}); partial remote effects may remain; inspect before retrying\nstderr:\n{}\nstdout:\n{}",
                output.status,
                push_diagnostic(&output.stderr),
                push_diagnostic(&output.stdout)
            )));
        }
        self.repository.revalidate_location()?;
        Ok("Native Push completed using Git's configured transport and refspec behavior".into())
    }
}
