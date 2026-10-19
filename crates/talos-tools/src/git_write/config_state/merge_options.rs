//! Branch merge options use Git's split_cmdline, not shell tokenization.

use super::{ConfigState, GitToolError, unsupported};

impl ConfigState {
    pub(super) fn validate_pull_merge_strategy(&self) -> Result<(), GitToolError> {
        // builtin/merge.c:add_strategies splits on literal spaces, without
        // shell quoting. The selected config can discover git-merge-* on PATH.
        // Native reduce_parents can change which key applies. Until that
        // reduction is retained as evidence, conservatively inspect both.
        // Default octopus helper effects still require execution validation.
        for key in [b"pull.octopus".as_slice(), b"pull.twohead".as_slice()] {
            let Some(value) = self.values.get(key).and_then(|values| values.last()) else {
                continue;
            };
            let value = value
                .as_deref()
                .ok_or_else(|| unsupported("Merge strategy needs a value"))?;
            if value
                .split(|byte| *byte == b' ')
                .any(|strategy| !matches!(strategy, b"recursive" | b"ort" | b"subtree"))
            {
                return Err(unsupported(
                    "External Pull merge strategy requires separate program execution authorization",
                ));
            }
        }
        Ok(())
    }

    pub(super) fn validate_pull_merge_options(
        &self,
        branch: Option<&str>,
    ) -> Result<(), GitToolError> {
        let Some(branch) = branch else { return Ok(()) };
        let key = format!("branch.{branch}.mergeoptions");
        let Some(value) = self
            .values
            .get(key.as_bytes())
            .and_then(|values| values.last())
        else {
            return Ok(());
        };
        let value = value
            .as_deref()
            .ok_or_else(|| unsupported("Branch merge options need a value"))?;
        let options = split(value)?;
        // These flags add no separately modeled helper/path effects; the merge
        // still needs complete hook/editor/driver validation. This is not a
        // replacement for Git's option parser. Abbreviations, clustered short
        // flags and value-taking options
        // need explicit effect modeling before they can enter the execution plan.
        for option in options {
            if !matches!(
                option.as_slice(),
                b"--ff"
                    | b"--no-ff"
                    | b"--ff-only"
                    | b"--stat"
                    | b"--no-stat"
                    | b"--summary"
                    | b"--no-summary"
                    | b"--compact-summary"
                    | b"--no-compact-summary"
                    | b"--no-log"
                    | b"--squash"
                    | b"--no-squash"
                    | b"--commit"
                    | b"--no-commit"
                    | b"--no-edit"
                    | b"--no-gpg-sign"
                    | b"--no-verify-signatures"
                    | b"--signoff"
                    | b"--no-signoff"
                    | b"--quiet"
                    | b"--verbose"
                    | b"--progress"
                    | b"--no-progress"
                    | b"--allow-unrelated-histories"
                    | b"--no-allow-unrelated-histories"
                    | b"--overwrite-ignore"
                    | b"--no-overwrite-ignore"
                    | b"-n"
                    | b"-q"
                    | b"-v"
            ) {
                return Err(unsupported(
                    "Branch merge options require additional effect discovery before integration execution",
                ));
            }
        }
        Ok(())
    }
}

/// Port of Git 2.54 alias.c:split_cmdline. In particular, leading/trailing
/// whitespace creates empty arguments, '#' is literal, and backslash-newline
/// preserves the newline. No shell expansion or environment assignment occurs.
fn split(input: &[u8]) -> Result<Vec<Vec<u8>>, GitToolError> {
    let invalid = || unsupported("Invalid branch merge option quoting");
    if input.contains(&0) {
        return Err(invalid());
    }
    let mut args = vec![Vec::new()];
    let mut quote = None;
    let mut cursor = 0;
    while let Some(&byte) = input.get(cursor) {
        // Git's sane_ctype isspace covers these four bytes, not C's VT/FF.
        if quote.is_none() && matches!(byte, b' ' | b'\t' | b'\n' | b'\r') {
            cursor += 1;
            while input
                .get(cursor)
                .is_some_and(|b| matches!(b, b' ' | b'\t' | b'\n' | b'\r'))
            {
                cursor += 1;
            }
            args.push(Vec::new());
            continue;
        }
        if quote.is_none() && matches!(byte, b'\'' | b'"') {
            quote = Some(byte);
        } else if quote == Some(byte) {
            quote = None;
        } else {
            let byte = if byte == b'\\' && quote != Some(b'\'') {
                cursor += 1;
                *input.get(cursor).ok_or_else(invalid)?
            } else {
                byte
            };
            // There is always an initial argument, including for empty input.
            if let Some(argument) = args.last_mut() {
                argument.push(byte);
            }
        }
        cursor += 1;
    }
    if quote.is_some() {
        return Err(invalid());
    }
    Ok(args)
}

#[cfg(test)]
#[path = "merge_options/tests.rs"]
mod tests;
