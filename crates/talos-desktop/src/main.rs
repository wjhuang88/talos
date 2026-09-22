//! Desktop window with a GPUI presentation boundary and embedded Talos Runtime host.
mod input;
mod presentation;
mod provider;
mod runtime_host;
mod window;

fn main() -> std::process::ExitCode {
    #[cfg(feature = "visual-test")]
    if matches!(
        std::env::args().nth(1).as_deref(),
        Some("--capture" | "--capture-live")
    ) {
        let Some(directory) = std::env::args_os().nth(2) else {
            eprintln!("Usage: talos-desktop-mock --capture <new-output-directory>");
            return std::process::ExitCode::FAILURE;
        };
        return window::capture(
            directory.into(),
            std::env::args().nth(1).as_deref() == Some("--capture-live"),
        );
    }
    let mut args = std::env::args().skip(1);
    let first = args.next();
    let live = first.as_deref() == Some("--live");
    let locale = if live { args.next() } else { first }.unwrap_or_else(|| "en-US".into());
    window::run(locale, live)
}
