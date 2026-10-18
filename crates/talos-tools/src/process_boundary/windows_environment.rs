//! UTF-16 environment blocks without reading or mutating the parent environment.

/// Entries must already have Windows case-insensitive ordering and uniqueness.
fn encode(entries: impl IntoIterator<Item = (Vec<u16>, Vec<u16>)>) -> Result<Vec<u16>, String> {
    let mut block = Vec::new();
    for (name, value) in entries {
        // Drive-current-directory pseudo variables (=C:) are inherited by
        // Windows. Accept that exact shape, but no arbitrary embedded '='.
        let drive = name.len() == 3
            && name[0] == b'=' as u16
            && ((b'A' as u16..=b'Z' as u16).contains(&name[1])
                || (b'a' as u16..=b'z' as u16).contains(&name[1]))
            && name[2] == b':' as u16;
        if name.is_empty()
            || name.contains(&0)
            || (!drive && name.contains(&(b'=' as u16)))
            || value.contains(&0)
        {
            return Err("Windows environment contains an invalid name or NUL".into());
        }
        block.extend(name);
        block.push(b'=' as u16);
        block.extend(value);
        block.push(0);
    }
    if block.is_empty() {
        block.push(0);
    }
    block.push(0);
    Ok(block)
}

/// Encode an explicit snapshot. `Command` supplies the standard library's
/// Windows case-insensitive key handling; it is never spawned and inherits no
/// environment. Later entries replace earlier case-equivalent entries.
#[cfg(windows)]
pub(super) fn snapshot(
    entries: impl IntoIterator<Item = (std::ffi::OsString, std::ffi::OsString)>,
) -> Result<Vec<u16>, String> {
    use std::os::windows::ffi::OsStrExt;
    let entries = entries.into_iter().collect::<Vec<_>>();
    // Validate before Command's native case comparison: even inserting an
    // invalid empty key can panic inside Windows CompareStringOrdinal.
    for (name, value) in &entries {
        encode([(name.encode_wide().collect(), value.encode_wide().collect())])?;
    }
    std::panic::catch_unwind(move || {
        let mut ordering = std::process::Command::new("unused-environment-ordering");
        ordering.env_clear().envs(entries);
        encode(ordering.get_envs().filter_map(|(name, value)| {
            value.map(|value| (name.encode_wide().collect(), value.encode_wide().collect()))
        }))
    })
    .map_err(|_| "Windows environment key ordering failed".to_owned())?
}

#[cfg(test)]
#[path = "windows_environment_tests.rs"]
mod tests;
