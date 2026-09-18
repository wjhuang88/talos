//! Fixture-backed Desktop window; no live Runtime or Session binding.
mod input;
mod presentation;
mod window;

fn main() -> std::process::ExitCode {
    #[cfg(feature = "visual-test")]
    if std::env::args().nth(1).as_deref() == Some("--capture") {
        let Some(directory) = std::env::args_os().nth(2) else {
            eprintln!("Usage: talos-desktop-mock --capture <new-output-directory>");
            return std::process::ExitCode::FAILURE;
        };
        return window::capture(directory.into());
    }
    window::run(std::env::args().nth(1).unwrap_or_else(|| "en-US".into()))
}
