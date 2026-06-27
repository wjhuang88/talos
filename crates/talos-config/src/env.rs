use std::env;
use std::path::PathBuf;

pub(crate) fn home_dir() -> PathBuf {
    if let Some(home) = env::var("HOME").ok().filter(|h| !h.is_empty()) {
        return PathBuf::from(home);
    }
    if let Some(profile) = env::var("USERPROFILE").ok().filter(|p| !p.is_empty()) {
        return PathBuf::from(profile);
    }
    let drive = env::var("HOMEDRIVE").unwrap_or_default();
    let path = env::var("HOMEPATH").unwrap_or_default();
    if !drive.is_empty() && !path.is_empty() {
        return PathBuf::from(format!("{drive}{path}"));
    }
    PathBuf::from(".")
}

/// Performs `${ENV_VAR}` substitution in a string.
///
/// Replaces all occurrences of `${VAR_NAME}` with the value of the
/// corresponding environment variable. If the variable is not set,
/// the placeholder is left unchanged.
pub(crate) fn substitute_env_vars(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' && chars.peek() == Some(&'{') {
            chars.next(); // consume '{'
            let mut var_name = String::new();
            let mut found_close = false;
            while let Some(&c) = chars.peek() {
                chars.next();
                if c == '}' {
                    found_close = true;
                    break;
                }
                var_name.push(c);
            }
            if found_close {
                if let Ok(value) = env::var(&var_name) {
                    result.push_str(&value);
                } else {
                    // Variable not set, keep the placeholder
                    result.push_str("${");
                    result.push_str(&var_name);
                    result.push('}');
                }
            } else {
                result.push_str("${");
                result.push_str(&var_name);
            }
        } else {
            result.push(c);
        }
    }

    result
}
