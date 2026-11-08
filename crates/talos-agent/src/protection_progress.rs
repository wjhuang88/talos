//! Turn-local confirmed file progress; never reset total or non-command limits.
use std::cell::Cell;

tokio::task_local! {
    pub(crate) static FILE_CHANGED: Cell<bool>;
}

pub(crate) fn record() {
    let _ = FILE_CHANGED.try_with(|changed| changed.set(true));
}

pub(crate) fn take() -> bool {
    FILE_CHANGED
        .try_with(|changed| changed.replace(false))
        .unwrap_or(false)
}

pub(crate) fn is_command(name: &str) -> bool {
    matches!(name, "exec" | "bash" | "powershell" | "validate_project")
}
