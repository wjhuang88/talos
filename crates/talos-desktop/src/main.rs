//! Fixture-backed, renderer-neutral Desktop visual prototype (I277).
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Fixture {
    title: String,
    status: String,
    goal: String,
}

fn catalog(locale: &str) -> (&'static str, &'static str, &'static str) {
    match locale {
        "zh-CN" => ("Talos 桌面预览", "模拟状态", "目标"),
        _ => ("Talos Desktop Preview", "Mock status", "Goal"),
    }
}

fn main() {
    let locale = std::env::args().nth(1).unwrap_or_else(|| "en-US".into());
    let fixture: Fixture =
        serde_json::from_str(r#"{"title":"Execution","status":"ready","goal":"Review fixture"}"#)
            .expect("embedded fixture is valid");
    let (title, status, goal) = catalog(&locale);
    println!(
        "{title}\n{status}: {}\n{goal}: {}\n{}",
        fixture.status, fixture.goal, fixture.title
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn locales_only_change_labels() {
        let fixture: Fixture = serde_json::from_str(
            r#"{"title":"Execution","status":"ready","goal":"Review fixture"}"#,
        )
        .unwrap();
        assert_eq!(fixture.status, "ready");
        assert_ne!(catalog("en-US").0, catalog("zh-CN").0);
    }
    #[test]
    fn unsupported_locale_falls_back_to_english() {
        assert_eq!(catalog("fr-FR"), catalog("en-US"));
    }
}
