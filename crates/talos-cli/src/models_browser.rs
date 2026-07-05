use std::collections::BTreeMap;
use std::io::{self, IsTerminal, Write};

use anyhow::{Context, Result};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use talos_config::Config;

pub(crate) fn run_available_models_browser(initial_filter: Option<&str>) -> Result<()> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        anyhow::bail!(
            "--available-models-browser requires an interactive terminal; use --available-models for script output"
        );
    }

    let mut config = Config::load().context("failed to load configuration")?;
    let rows = build_browser_rows(&config);
    let mut state = CatalogBrowserState::new(rows);
    if let Some(filter) = initial_filter {
        state.set_query(filter);
    }

    let mut stdout = io::stdout();
    terminal::enable_raw_mode().context("failed to enable raw mode")?;
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

    let result = run_browser_loop(&mut stdout, &mut config, &mut state);

    let _ = execute!(stdout, cursor::Show, LeaveAlternateScreen);
    let _ = terminal::disable_raw_mode();
    result
}

fn run_browser_loop<W: Write>(
    stdout: &mut W,
    config: &mut Config,
    state: &mut CatalogBrowserState,
) -> Result<()> {
    let mut search_mode = false;
    loop {
        draw(stdout, state, search_mode)?;
        match event::read().context("failed to read terminal event")? {
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Esc, ..
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) if !search_mode => break,
            Event::Key(KeyEvent {
                code: KeyCode::Char('/'),
                ..
            }) if !search_mode => search_mode = true,
            Event::Key(KeyEvent {
                code: KeyCode::Esc, ..
            }) if search_mode => search_mode = false,
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            }) if search_mode => search_mode = false,
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                ..
            }) if search_mode => state.pop_query_char(),
            Event::Key(KeyEvent {
                code: KeyCode::Char(ch),
                ..
            }) if search_mode => state.push_query_char(ch),
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                ..
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Char('j'),
                ..
            }) => state.move_down(1),
            Event::Key(KeyEvent {
                code: KeyCode::Up, ..
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Char('k'),
                ..
            }) => state.move_up(1),
            Event::Key(KeyEvent {
                code: KeyCode::PageDown,
                ..
            }) => state.move_down(state.page_size()),
            Event::Key(KeyEvent {
                code: KeyCode::PageUp,
                ..
            }) => state.move_up(state.page_size()),
            Event::Key(KeyEvent {
                code: KeyCode::Char('g'),
                ..
            }) => state.first(),
            Event::Key(KeyEvent {
                code: KeyCode::Char('G'),
                ..
            }) => state.last(),
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                ..
            }) => {
                if let Some(row) = state.selected_row().cloned() {
                    select_row(stdout, config, state, &row)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn draw<W: Write>(
    stdout: &mut W,
    state: &mut CatalogBrowserState,
    search_mode: bool,
) -> Result<()> {
    let (width, height) = terminal::size().unwrap_or((100, 30));
    let visible_height = height.saturating_sub(1).max(1) as usize;
    state.set_view_height(visible_height.saturating_sub(4));
    queue!(stdout, cursor::MoveTo(0, 0), Clear(ClearType::All))?;
    for line in state.render_lines(visible_height, width as usize, search_mode) {
        queue!(
            stdout,
            SetForegroundColor(line.color),
            Print(line.text),
            ResetColor,
            Print("\r\n")
        )?;
    }
    stdout.flush()?;
    Ok(())
}

fn select_row<W: Write>(
    stdout: &mut W,
    config: &mut Config,
    state: &mut CatalogBrowserState,
    row: &CatalogBrowserRow,
) -> Result<()> {
    if row.authenticated {
        config.set_active_model(&row.qualified)?;
        config.save().context("failed to save configuration")?;
        state.message = format!("Active model set to {}", row.qualified);
        return Ok(());
    }

    terminal::disable_raw_mode().ok();
    execute!(stdout, LeaveAlternateScreen, cursor::Show).ok();
    let prompt_result = prompt_provider_setup(config, row);
    execute!(stdout, EnterAlternateScreen, cursor::Hide).ok();
    terminal::enable_raw_mode().ok();

    match prompt_result {
        Ok(()) => {
            *state = CatalogBrowserState::new(build_browser_rows(config));
            state.set_query(&row.provider);
            state.message = format!("Connected {} and selected {}", row.provider, row.qualified);
            Ok(())
        }
        Err(err) => {
            state.message = format!("Setup cancelled: {err}");
            Ok(())
        }
    }
}

fn prompt_provider_setup(config: &mut Config, row: &CatalogBrowserRow) -> Result<()> {
    println!();
    println!(
        "Set up provider '{}'. Existing keys are never displayed.",
        row.provider
    );
    print!("API key: ");
    io::stdout().flush().ok();
    let api_key = read_trimmed_line()?;
    if api_key.is_empty() {
        anyhow::bail!("empty API key");
    }

    let default_base_url = default_base_url(config, row);
    match &default_base_url {
        Some(url) => print!("Base URL [{url}]: "),
        None => print!("Base URL [leave empty]: "),
    }
    io::stdout().flush().ok();
    let typed_base_url = read_trimmed_line()?;

    apply_provider_setup(config, row, &api_key, typed_base_url)?;
    config.save().context("failed to save configuration")?;
    Ok(())
}

fn apply_provider_setup(
    config: &mut Config,
    row: &CatalogBrowserRow,
    api_key: &str,
    typed_base_url: String,
) -> Result<()> {
    if api_key.trim().is_empty() {
        anyhow::bail!("empty API key");
    }

    let default_base_url = default_base_url(config, row);
    config.set_provider_credential(&row.provider, api_key.trim());
    let provider_entry = config.providers.entry(row.provider.clone()).or_default();
    if provider_entry.api_key_env.is_none() {
        provider_entry.api_key_env = row.env_var.clone();
    }
    if typed_base_url.trim().is_empty() {
        if provider_entry.base_url.is_none() {
            provider_entry.base_url = default_base_url;
        }
    } else {
        provider_entry.base_url = Some(typed_base_url.trim().to_string());
    }
    config.set_active_model(&row.qualified)?;
    Ok(())
}

fn read_trimmed_line() -> Result<String> {
    let mut value = String::new();
    io::stdin()
        .read_line(&mut value)
        .context("failed to read terminal input")?;
    Ok(value.trim().to_string())
}

fn default_base_url(config: &Config, row: &CatalogBrowserRow) -> Option<String> {
    config
        .providers
        .get(&row.provider)
        .and_then(|p| p.base_url.clone())
        .or_else(|| row.api_base_url.clone())
        .or_else(|| talos_config::builtin_provider_config(&row.provider).and_then(|p| p.base_url))
}

fn build_browser_rows(config: &Config) -> Vec<CatalogBrowserRow> {
    let provider_meta = talos_config::model::builtin_providers()
        .into_iter()
        .map(|p| (p.id.clone(), p))
        .collect::<BTreeMap<_, _>>();

    let mut rows = talos_config::model::builtin_models()
        .into_iter()
        .map(|model| {
            let provider = model.provider.clone();
            let meta = provider_meta.get(&provider);
            let ctx = model
                .context_limit
                .map(|c| format!("{}K", c / 1000))
                .unwrap_or_else(|| "?".to_string());
            let pricing = model
                .pricing
                .as_ref()
                .map(|p| {
                    let input = p.input_per_1m.unwrap_or(0.0);
                    let output = p.output_per_1m.unwrap_or(0.0);
                    format!("${input:.2}/${output:.2}/1M")
                })
                .unwrap_or_default();
            CatalogBrowserRow {
                provider: provider.clone(),
                provider_name: meta
                    .map(|p| p.name.clone())
                    .filter(|name| !name.is_empty())
                    .unwrap_or_else(|| provider.clone()),
                model_id: model.id.clone(),
                qualified: format!("{provider}/{}", model.id),
                authenticated: config.provider_authenticated(&provider),
                current: config.provider == provider && config.model == model.id,
                context: ctx,
                pricing,
                api_base_url: meta.and_then(|p| p.api_base_url.clone()),
                env_var: meta.and_then(|p| p.env_var.clone()),
            }
        })
        .collect::<Vec<_>>();

    rows.sort_by(|a, b| {
        a.provider
            .cmp(&b.provider)
            .then_with(|| a.model_id.cmp(&b.model_id))
    });
    rows
}

#[derive(Debug, Clone)]
struct CatalogBrowserRow {
    provider: String,
    provider_name: String,
    model_id: String,
    qualified: String,
    authenticated: bool,
    current: bool,
    context: String,
    pricing: String,
    api_base_url: Option<String>,
    env_var: Option<String>,
}

#[derive(Debug, Clone)]
struct RenderLine {
    text: String,
    color: Color,
}

#[derive(Debug)]
struct CatalogBrowserState {
    rows: Vec<CatalogBrowserRow>,
    filtered: Vec<usize>,
    selected: usize,
    offset: usize,
    query: String,
    view_height: usize,
    message: String,
}

impl CatalogBrowserState {
    fn new(rows: Vec<CatalogBrowserRow>) -> Self {
        let mut state = Self {
            rows,
            filtered: Vec::new(),
            selected: 0,
            offset: 0,
            query: String::new(),
            view_height: 20,
            message: String::new(),
        };
        state.apply_filter();
        state
    }

    fn set_view_height(&mut self, height: usize) {
        self.view_height = height.max(1);
        self.ensure_selected_visible();
    }

    fn page_size(&self) -> usize {
        self.view_height.saturating_sub(1).max(1)
    }

    fn set_query(&mut self, query: &str) {
        self.query = query.trim().to_lowercase();
        self.apply_filter();
    }

    fn push_query_char(&mut self, ch: char) {
        self.query.push(ch.to_ascii_lowercase());
        self.apply_filter();
    }

    fn pop_query_char(&mut self) {
        self.query.pop();
        self.apply_filter();
    }

    fn apply_filter(&mut self) {
        self.filtered = self
            .rows
            .iter()
            .enumerate()
            .filter_map(|(idx, row)| row.matches(self.query.as_str()).then_some(idx))
            .collect();
        self.selected = self.selected.min(self.filtered.len().saturating_sub(1));
        self.offset = self.offset.min(self.selected);
        self.ensure_selected_visible();
    }

    fn move_down(&mut self, count: usize) {
        if self.filtered.is_empty() {
            return;
        }
        self.selected = (self.selected + count).min(self.filtered.len() - 1);
        self.ensure_selected_visible();
    }

    fn move_up(&mut self, count: usize) {
        self.selected = self.selected.saturating_sub(count);
        self.ensure_selected_visible();
    }

    fn first(&mut self) {
        self.selected = 0;
        self.ensure_selected_visible();
    }

    fn last(&mut self) {
        self.selected = self.filtered.len().saturating_sub(1);
        self.ensure_selected_visible();
    }

    fn selected_row(&self) -> Option<&CatalogBrowserRow> {
        self.filtered
            .get(self.selected)
            .and_then(|idx| self.rows.get(*idx))
    }

    fn ensure_selected_visible(&mut self) {
        if self.selected < self.offset {
            self.offset = self.selected;
        }
        let bottom = self.offset + self.view_height;
        if self.selected >= bottom {
            self.offset = self.selected.saturating_sub(self.view_height - 1);
        }
    }

    fn render_lines(&mut self, height: usize, width: usize, search_mode: bool) -> Vec<RenderLine> {
        let mut lines = Vec::new();
        let selected_position = if self.filtered.is_empty() {
            0
        } else {
            self.selected + 1
        };
        lines.push(RenderLine {
            text: fit(
                &format!(
                    "Talos model catalog  {selected_position}/{}  query: {}",
                    self.filtered.len(),
                    if self.query.is_empty() {
                        "<none>"
                    } else {
                        &self.query
                    }
                ),
                width,
            ),
            color: Color::Cyan,
        });
        lines.push(RenderLine {
            text: fit(
                if search_mode {
                    "search: type to filter, Enter/Esc to close"
                } else {
                    "j/k arrows scroll  g/G jump  / search  Enter select/setup  c connect  q quit"
                },
                width,
            ),
            color: Color::DarkGrey,
        });
        if !self.message.is_empty() {
            lines.push(RenderLine {
                text: fit(&self.message, width),
                color: Color::Green,
            });
        }

        let body_height = height.saturating_sub(lines.len()).max(1);
        self.set_view_height(body_height);
        if self.filtered.is_empty() {
            lines.push(RenderLine {
                text: fit("No matching models.", width),
                color: Color::Yellow,
            });
            return lines;
        }

        let mut last_provider = None::<String>;
        for visible_idx in self.offset..self.filtered.len() {
            if lines.len() >= height {
                break;
            }
            let row_idx = self.filtered[visible_idx];
            let row = &self.rows[row_idx];
            if last_provider.as_deref() != Some(row.provider.as_str()) {
                if lines.len() >= height {
                    break;
                }
                let status = if row.authenticated {
                    "Ready"
                } else {
                    "Setup required"
                };
                lines.push(RenderLine {
                    text: fit(&format!("  {}  -  {status}", row.provider_name), width),
                    color: Color::Yellow,
                });
                last_provider = Some(row.provider.clone());
            }
            if lines.len() >= height {
                break;
            }
            let selected = visible_idx == self.selected;
            let marker = if selected { ">" } else { " " };
            let current = if row.current { "*" } else { " " };
            let auth = if row.authenticated { "ready" } else { "setup" };
            let pricing = if row.pricing.is_empty() {
                String::new()
            } else {
                format!("  {}", row.pricing)
            };
            lines.push(RenderLine {
                text: fit(
                    &format!(
                        "{marker} {current} {:<44} {:<6} ctx:{}{}",
                        row.qualified, auth, row.context, pricing
                    ),
                    width,
                ),
                color: if selected {
                    Color::White
                } else if row.authenticated {
                    Color::Grey
                } else {
                    Color::DarkGrey
                },
            });
        }
        lines
    }
}

impl CatalogBrowserRow {
    fn matches(&self, query: &str) -> bool {
        query.is_empty()
            || self.provider.to_lowercase().contains(query)
            || self.provider_name.to_lowercase().contains(query)
            || self.model_id.to_lowercase().contains(query)
            || self.qualified.to_lowercase().contains(query)
    }
}

fn fit(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let chars = text.chars().collect::<Vec<_>>();
    if chars.len() <= width {
        return text.to_string();
    }
    if width == 1 {
        return "…".to_string();
    }
    chars
        .into_iter()
        .take(width - 1)
        .chain(std::iter::once('…'))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_rows() -> Vec<CatalogBrowserRow> {
        vec![
            CatalogBrowserRow {
                provider: "anthropic".to_string(),
                provider_name: "Anthropic".to_string(),
                model_id: "claude-sonnet".to_string(),
                qualified: "anthropic/claude-sonnet".to_string(),
                authenticated: true,
                current: true,
                context: "200K".to_string(),
                pricing: "$3.00/$15.00/1M".to_string(),
                api_base_url: None,
                env_var: Some("ANTHROPIC_API_KEY".to_string()),
            },
            CatalogBrowserRow {
                provider: "openai".to_string(),
                provider_name: "OpenAI".to_string(),
                model_id: "gpt-4.1".to_string(),
                qualified: "openai/gpt-4.1".to_string(),
                authenticated: false,
                current: false,
                context: "128K".to_string(),
                pricing: String::new(),
                api_base_url: Some("https://api.openai.com/v1".to_string()),
                env_var: Some("OPENAI_API_KEY".to_string()),
            },
        ]
    }

    #[test]
    fn filters_by_provider_model_and_qualified_name() {
        let mut state = CatalogBrowserState::new(sample_rows());
        state.set_query("openai/gpt");
        assert_eq!(state.filtered.len(), 1);
        assert_eq!(state.selected_row().unwrap().qualified, "openai/gpt-4.1");
    }

    #[test]
    fn navigation_stays_on_model_rows() {
        let mut state = CatalogBrowserState::new(sample_rows());
        state.move_down(1);
        assert_eq!(state.selected_row().unwrap().qualified, "openai/gpt-4.1");
        state.move_down(100);
        assert_eq!(state.selected_row().unwrap().qualified, "openai/gpt-4.1");
        state.move_up(1);
        assert_eq!(
            state.selected_row().unwrap().qualified,
            "anthropic/claude-sonnet"
        );
    }

    #[test]
    fn render_marks_current_and_setup_without_secrets() {
        let mut state = CatalogBrowserState::new(sample_rows());
        let lines = state
            .render_lines(12, 100, false)
            .into_iter()
            .map(|line| line.text)
            .collect::<Vec<_>>()
            .join("\n");
        assert!(lines.contains("anthropic/claude-sonnet"));
        assert!(lines.contains("openai/gpt-4.1"));
        assert!(lines.contains("setup"));
        assert!(!lines.contains("sk-"));
        assert!(!lines.contains("API key:"));
    }

    #[test]
    fn provider_setup_updates_target_and_preserves_unrelated_config() {
        let row = sample_rows().pop().unwrap();
        let mut config = Config::default();
        config.provider = "anthropic".to_string();
        config.model = "claude-sonnet".to_string();
        config
            .providers
            .entry("anthropic".to_string())
            .or_default()
            .api_key = Some("sk-anthropic-existing".to_string());

        apply_provider_setup(
            &mut config,
            &row,
            "sk-openai-new",
            "https://custom.openai.test/v1".to_string(),
        )
        .unwrap();

        assert_eq!(config.provider, "openai");
        assert_eq!(config.model, "gpt-4.1");
        let openai = config.providers.get("openai").unwrap();
        assert_eq!(openai.api_key.as_deref(), Some("sk-openai-new"));
        assert_eq!(openai.api_key_env.as_deref(), Some("OPENAI_API_KEY"));
        assert_eq!(
            openai.base_url.as_deref(),
            Some("https://custom.openai.test/v1")
        );
        assert_eq!(
            config
                .providers
                .get("anthropic")
                .and_then(|p| p.api_key.as_deref()),
            Some("sk-anthropic-existing")
        );
    }

    #[test]
    fn fit_truncates_to_width() {
        assert_eq!(fit("abcdef", 4), "abc…");
        assert_eq!(fit("abcdef", 1), "…");
    }
}
