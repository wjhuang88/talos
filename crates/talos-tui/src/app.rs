use std::pin::Pin;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::{
    event::{self, Event, EventStream, KeyCode, KeyEventKind},
    terminal::enable_raw_mode,
};
use futures::{Stream, StreamExt};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Padding, Paragraph},
};
use talos_conversation::{StatusSnapshot, TipKind, UiOutput, UserInput};
use talos_core::ApprovalChoice;
use talos_core::TuiApprovalRequest;
use tokio::sync::mpsc;

use crate::evolution::{self, EvolutionPanel};
use crate::inline_terminal::{ComponentStack, InlineFrame, InlineTerminal, ViewportComponent};
use crate::sidebar::{SkillInfo, SkillSidebar};
use crate::state::{ApprovalState, CtrlCState, Tip, TuiState};
use crate::widgets::ApprovalOverlay;

struct PreviewComponent<'a> {
    text: &'a str,
}

impl ViewportComponent for PreviewComponent<'_> {
    fn height_hint(&self, _w: u16) -> u16 { 1 }

    fn render(&self, frame: &mut InlineFrame, area: Rect) {
        let line = self.text.split('\n').last().unwrap_or("");
        let display = truncate_end_to_width(line, area.width);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                display,
                Style::default().fg(Color::Rgb(0xE5, 0xE9, 0xF0)),
            ))),
            area,
        );
    }
}

struct QueuePreviewComponent {
    count: usize,
    steering: usize,
    followup: usize,
}

impl ViewportComponent for QueuePreviewComponent {
    fn height_hint(&self, _w: u16) -> u16 {
        if self.count == 0 { 0 } else { 1 + (self.count as u16).min(2) }
    }

    fn render(&self, frame: &mut InlineFrame, area: Rect) {
        let dim = Style::default().fg(Color::Rgb(0x4C, 0x56, 0x6A));
        let mut lines = Vec::new();
        lines.push(Line::from(vec![
            Span::styled(" ", dim),
            Span::styled(
                format!("{} queued input{}", self.count, if self.count == 1 { "" } else { "s" }),
                dim,
            ),
            Span::styled(" (will send after current turn)", dim),
        ]));
        let max_width = (area.width as usize).saturating_sub(4);
        let show_steering = self.steering.min(2);
        for i in 0..show_steering {
            let label = if i == 0 { "steering" } else { "…" };
            let text = if label.len() > max_width {
                format!("{}…", &label[..max_width - 1])
            } else {
                label.to_string()
            };
            lines.push(Line::from(vec![
                Span::styled("  ", dim),
                Span::styled("↳ ", dim.add_modifier(Modifier::DIM)),
                Span::styled(text, dim),
            ]));
        }
        frame.render_widget(Paragraph::new(lines), area);
    }
}

struct GapComponent;

impl ViewportComponent for GapComponent {
    fn height_hint(&self, _w: u16) -> u16 { 1 }
    fn render(&self, _frame: &mut InlineFrame, _area: Rect) {}
}

struct TipsComponent<'a> {
    tip: Option<&'a Tip>,
    is_processing: bool,
}

impl ViewportComponent for TipsComponent<'_> {
    fn height_hint(&self, _w: u16) -> u16 { 1 }

    fn render(&self, frame: &mut InlineFrame, area: Rect) {
        let text = if let Some(tip) = self.tip {
            let color = match tip.kind {
                TipKind::ExitHint | TipKind::QueueHint => Color::Rgb(0xA3, 0xBE, 0x8C),
                TipKind::ApprovalResult => Color::Rgb(0xB4, 0x8E, 0xAD),
                TipKind::LagWarning => Color::Rgb(0xBF, 0x61, 0x6C),
                TipKind::Info => Color::Rgb(0x88, 0xC0, 0xD0),
            };
            Text::from(Line::from(Span::styled(format!(" {}", tip.text), Style::default().fg(color))))
        } else if self.is_processing {
            Text::from(Line::from(Span::styled(
                " Processing… Esc to clear, Ctrl+C to cancel",
                Style::default().fg(Color::Rgb(0xD0, 0x87, 0x70)),
            )))
        } else {
            Text::from(Line::from(Span::styled(
                " Enter to send, Esc to clear, Ctrl+K skills, Ctrl+E evolution",
                Style::default().fg(Color::Rgb(0x4C, 0x56, 0x6A)),
            )))
        };
        frame.render_widget(Paragraph::new(text), area);
    }
}

struct InputPadComponent;

impl ViewportComponent for InputPadComponent {
    fn height_hint(&self, _w: u16) -> u16 { 1 }

    fn render(&self, frame: &mut InlineFrame, area: Rect) {
        frame.render_widget(
            Paragraph::new("").style(Style::default().bg(Color::Rgb(0x3B, 0x42, 0x52))),
            area,
        );
    }
}

struct InputComponent<'a> {
    state: &'a TuiState,
    approval: &'a ApprovalState,
}

impl ViewportComponent for InputComponent<'_> {
    fn height_hint(&self, _w: u16) -> u16 { 1 }

    fn render(&self, frame: &mut InlineFrame, area: Rect) {
        let input_bg = Color::Rgb(0x3B, 0x42, 0x52);
        let input_text = build_input_text(self.state);
        let input_block = Block::default()
            .style(Style::default().bg(input_bg))
            .padding(Padding::new(1, 1, 0, 0));
        frame.render_widget(Paragraph::new(input_text).block(input_block), area);

        if let ApprovalState::Visible { tool_name, arguments, selected } = self.approval {
            let overlay = ApprovalOverlay::new(tool_name, arguments, selected);
            frame.render_widget(overlay, area);
        }
    }
}

struct StatusComponent<'a> {
    status: &'a StatusSnapshot,
}

impl ViewportComponent for StatusComponent<'_> {
    fn height_hint(&self, _w: u16) -> u16 { 1 }

    fn render(&self, frame: &mut InlineFrame, area: Rect) {
        let text = build_status_text(self.status);
        frame.render_widget(Paragraph::new(text), area);
    }
}

pub struct Tui {
    state: TuiState,
    terminal: InlineTerminal,
    skill_sidebar: SkillSidebar,
    evolution_panel: EvolutionPanel,
    ui_output_rx: Option<mpsc::UnboundedReceiver<UiOutput>>,
    user_input_tx: Option<mpsc::UnboundedSender<UserInput>>,
    pending_scrollback: Vec<String>,
    active_stream: Option<Pin<Box<dyn Stream<Item = String> + Send>>>,
    stream_buffer: String,
    streaming_preview: String,
}

impl Tui {
    pub fn new() -> Result<Self> {
        let _splash_lines = print_splash_scrollback();

        let (_, cursor_y) = crossterm::cursor::position().map_err(|e| anyhow::anyhow!("{e}"))?;
        let (_, screen_h) = crossterm::terminal::size().map_err(|e| anyhow::anyhow!("{e}"))?;
        let viewport_height: u16 = 7;
        if cursor_y.saturating_add(viewport_height) > screen_h {
            for _ in 0..viewport_height.saturating_sub(1) {
                println!();
            }
        }

        enable_raw_mode()?;
        let terminal = InlineTerminal::new()?;

        Ok(Self {
            state: TuiState::new(),
            terminal,
            skill_sidebar: SkillSidebar::new(),
            evolution_panel: evolution::EvolutionPanel::new(),
            ui_output_rx: None,
            user_input_tx: None,
            pending_scrollback: Vec::new(),
            active_stream: None,
            stream_buffer: String::new(),
            streaming_preview: String::new(),
        })
    }

    pub fn set_ui_output_rx(&mut self, rx: mpsc::UnboundedReceiver<UiOutput>) {
        self.ui_output_rx = Some(rx);
    }

    pub fn set_user_input_tx(&mut self, tx: mpsc::UnboundedSender<UserInput>) {
        self.user_input_tx = Some(tx);
    }

    pub fn set_model_name(&mut self, name: String) {
        self.state.status.model_name = name;
    }

    pub fn toggle_skill_sidebar(&mut self) {
        self.skill_sidebar.toggle();
    }

    pub fn toggle_evolution_panel(&mut self) {
        self.evolution_panel.toggle();
    }

    pub fn update_evolution_patterns(&mut self, patterns: Vec<evolution::PatternInfo>) {
        self.evolution_panel.update_patterns(patterns);
    }

    pub fn update_skills(&mut self, skills: Vec<SkillInfo>) {
        self.skill_sidebar.update_skills(skills);
    }

    pub fn approval_choice(&self) -> Option<&ApprovalChoice> {
        match &self.state.approval_state {
            ApprovalState::Visible { selected, .. } => Some(selected),
            ApprovalState::Hidden => None,
        }
    }

    pub fn handle_approval_key(&mut self, key: char) -> Option<ApprovalChoice> {
        let ApprovalState::Visible { selected, .. } = &mut self.state.approval_state else {
            return None;
        };

        match key {
            'y' => {
                *selected = ApprovalChoice::ApproveOnce;
                Some(ApprovalChoice::ApproveOnce)
            }
            'a' => {
                *selected = ApprovalChoice::AlwaysApprove;
                Some(ApprovalChoice::AlwaysApprove)
            }
            'n' => {
                *selected = ApprovalChoice::Deny;
                Some(ApprovalChoice::Deny)
            }
            _ => None,
        }
    }

    pub fn show_approval(&mut self, tool_name: &str, arguments: &str) {
        self.state.approval_state = ApprovalState::Visible {
            tool_name: tool_name.to_string(),
            arguments: arguments.to_string(),
            selected: ApprovalChoice::ApproveOnce,
        };
    }

    pub fn hide_approval(&mut self) {
        self.state.approval_state = ApprovalState::Hidden;
    }

    pub async fn run_with_approval(
        &mut self,
        mut approval_rx: mpsc::UnboundedReceiver<TuiApprovalRequest>,
    ) -> Result<()> {
        let mut event_stream = EventStream::new();
        let mut render_interval = tokio::time::interval(Duration::from_millis(50));
        let mut ui_output_rx = self.ui_output_rx.take().expect("ui_output_rx not set");

        loop {
            self.state.expire_tip();
            self.flush_pending_scrollback()?;
            self.draw_frame()?;

            tokio::select! {
                _ = render_interval.tick() => {}
                Some(Ok(event)) = event_stream.next() => {
                    if self.handle_input_event(&event) {
                        break;
                    }
                }
                Some(output) = ui_output_rx.recv() => {
                    if self.handle_ui_output(output) {
                        break;
                    }
                }
                Some(request) = approval_rx.recv() => {
                    self.state.pending_approval_response = Some(request.response);
                    self.show_approval(&request.tool_name, &request.arguments);
                }
                Some(chunk) = self.next_stream_chunk() => {
                    self.consume_stream_chunk(&chunk);
                }
            }

            if self.state.should_exit {
                break;
            }
        }

        self.restore();
        Ok(())
    }

    async fn next_stream_chunk(&mut self) -> Option<String> {
        match self.active_stream.as_mut() {
            Some(stream) => {
                let chunk = stream.next().await;
                if chunk.is_none() {
                    self.finalize_active_stream();
                }
                chunk
            }
            None => std::future::pending().await,
        }
    }

    fn finalize_active_stream(&mut self) {
        self.stream_buffer.clear();
        self.active_stream = None;
    }

    fn consume_stream_chunk(&mut self, chunk: &str) {
        self.stream_buffer.push_str(chunk);
        while let Some(pos) = self.stream_buffer.find('\n') {
            let line = self.stream_buffer[..pos].to_string();
            self.stream_buffer = self.stream_buffer[pos + 1..].to_string();
            self.pending_scrollback.push(line);
        }
        self.streaming_preview = self.stream_buffer.clone();
    }

    fn handle_ui_output(&mut self, output: UiOutput) -> bool {
        match output {
            UiOutput::Stream(msg) => {
                if self.active_stream.is_some() {
                    self.finalize_active_stream();
                }
                if !self.streaming_preview.is_empty() {
                    self.pending_scrollback.push(std::mem::take(&mut self.streaming_preview));
                }
                self.active_stream = Some(msg.stream);
                self.stream_buffer.clear();
            }
            UiOutput::Status(snapshot) => {
                self.state.status = snapshot;
            }
            UiOutput::Tip { text, kind } => {
                self.state.tip = Some(Tip {
                    kind,
                    text,
                    ttl: Duration::from_secs(2),
                    created_at: Instant::now(),
                });
            }
            UiOutput::Exit => {
                self.state.should_exit = true;
                return true;
            }
        }
        false
    }

    fn flush_pending_scrollback(&mut self) -> Result<()> {
        if self.pending_scrollback.is_empty() {
            return Ok(());
        }
        let lines = std::mem::take(&mut self.pending_scrollback);
        for line in &lines {
            self.terminal.insert_history(line)?;
        }
        Ok(())
    }

    fn draw_frame(&mut self) -> Result<()> {
        let state = &self.state;
        let status = &state.status;

        let preview = PreviewComponent { text: &self.streaming_preview };
        let queue = QueuePreviewComponent {
            count: status.steering_count + status.followup_count,
            steering: status.steering_count,
            followup: status.followup_count,
        };
        let gap = GapComponent;
        let tips = TipsComponent { tip: state.tip.as_ref(), is_processing: status.is_processing };
        let input_pad_top = InputPadComponent;
        let input = InputComponent { state, approval: &state.approval_state };
        let input_pad_bot = InputPadComponent;
        let status_comp = StatusComponent { status };

        let stack = ComponentStack::new(vec![
            &preview, &queue, &gap, &tips, &input_pad_top, &input, &input_pad_bot, &status_comp,
        ]);

        let total_height = stack.total_height(self.terminal.screen_size().width);

        self.terminal.draw(total_height, |frame| {
            let layout = stack.layout(frame.area(), frame.area().width);
            for (component, area) in layout {
                component.render(frame, area);
            }
        })?;

        Ok(())
    }

    fn handle_input_event(&mut self, event: &Event) -> bool {
        match event {
            Event::Key(key) => {
                if key.kind != KeyEventKind::Press {
                    return false;
                }
                match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        return self.state.handle_ctrl_c();
                    }
                    KeyCode::Char('k') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        self.toggle_skill_sidebar();
                    }
                    KeyCode::Char('e') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        self.toggle_evolution_panel();
                    }
                    KeyCode::Char(c)
                        if !matches!(self.state.approval_state, ApprovalState::Hidden) =>
                    {
                        if let Some(choice) = self.handle_approval_key(c) {
                            if let Some(response_tx) = self.state.pending_approval_response.take() {
                                let _ = response_tx.send(choice.clone());
                            }
                            self.hide_approval();
                            self.state.tip = Some(Tip {
                                kind: TipKind::ApprovalResult,
                                text: format!(
                                    "Tool call {}",
                                    match choice {
                                        ApprovalChoice::ApproveOnce => "approved once",
                                        ApprovalChoice::AlwaysApprove => "always approved",
                                        ApprovalChoice::Deny => "denied",
                                    }
                                ),
                                ttl: Duration::from_secs(2),
                                created_at: Instant::now(),
                            });
                        }
                    }
                    KeyCode::Char(c) => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        self.state.input_append_char(c);
                    }
                    KeyCode::Backspace => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        self.state.input_backspace();
                    }
                    KeyCode::Left => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        self.state.input_cursor_left();
                    }
                    KeyCode::Right => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        self.state.input_cursor_right();
                    }
                    KeyCode::Enter => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        let input = self.state.input_submit();
                        if !input.is_empty() {
                            if let Some(ref tx) = self.user_input_tx {
                                let _ = tx.send(UserInput::Message(input));
                            }
                        }
                    }
                    KeyCode::Esc => {
                        self.state.ctrl_c_state = CtrlCState::Idle;
                        self.state.input_clear();
                    }
                    _ => {}
                }
            }
            Event::Resize(_, _) => {}
            _ => {}
        }
        false
    }

    fn restore(&self) {
        self.terminal.restore();
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        self.restore();
    }
}

pub(crate) fn build_input_text(state: &TuiState) -> Text<'static> {
    let buffer = &state.input_buffer;
    let char_count = buffer.chars().count();
    let cursor_pos = state.cursor_pos.min(char_count);

    let before: String = buffer.chars().take(cursor_pos).collect();
    let after: String = buffer.chars().skip(cursor_pos).collect();

    let cursor_style = Style::default()
        .fg(Color::Rgb(0x2E, 0x34, 0x40))
        .bg(Color::Rgb(0x88, 0xC0, 0xD0));

    let mut spans = Vec::new();
    spans.push(Span::styled(
        " ❯ ",
        Style::default().fg(Color::Rgb(0xA3, 0xBE, 0x8C)),
    ));
    spans.push(Span::raw(before));
    if after.is_empty() {
        spans.push(Span::styled(" ", cursor_style));
    } else {
        let mut chars = after.chars();
        if let Some(first) = chars.next() {
            let rest: String = chars.collect();
            spans.push(Span::styled(first.to_string(), cursor_style));
            spans.push(Span::raw(rest));
        } else {
            spans.push(Span::styled(" ", cursor_style));
        }
    }

    Text::from(Line::from(spans))
}

pub(crate) fn build_status_text(status: &StatusSnapshot) -> Text<'static> {
    let model_name = status.model_name.clone();
    let total_tokens = status.usage.input_tokens + status.usage.output_tokens;
    let cost = calculate_cost(&status.usage);

    let branch_info = status
        .branch_id
        .as_ref()
        .map(|b| {
            let short: String = b.chars().take(8).collect();
            format!(" │ {short}")
        })
        .unwrap_or_default();

    let queue_info = if status.steering_count > 0 || status.followup_count > 0 {
        let mut parts = Vec::new();
        if status.steering_count > 0 {
            parts.push(format!("S:{}", status.steering_count));
        }
        if status.followup_count > 0 {
            parts.push(format!("F:{}", status.followup_count));
        }
        format!(" │ {}", parts.join(", "))
    } else {
        String::new()
    };

    let dim = Style::default().fg(Color::Rgb(0x4C, 0x56, 0x6A));
    let sep = Span::styled(" │ ", dim);
    let val = Style::default().fg(Color::Rgb(0x81, 0xA1, 0xC1));

    let spans = vec![
        Span::styled(" ", dim),
        Span::styled(model_name, val),
        sep.clone(),
        Span::styled(format!("{} tokens", total_tokens), val),
        Span::styled(branch_info, val),
        sep.clone(),
        Span::styled(cost, val),
        Span::styled(queue_info, val),
    ];

    Text::from(Line::from(spans))
}

pub(crate) fn calculate_cost(usage: &talos_core::message::Usage) -> String {
    let total = usage.input_tokens + usage.output_tokens;
    let cost = (total as f64) * 0.003 / 1000.0;
    format!("${cost:.4}")
}

fn truncate_end_to_width(s: &str, max_width: u16) -> String {
    let max = max_width as usize;
    if unicode_width::UnicodeWidthStr::width(s) <= max {
        return s.to_string();
    }
    let chars: Vec<char> = s.chars().collect();
    let mut width = 0usize;
    let mut start = chars.len();
    for (i, ch) in chars.iter().enumerate().rev() {
        let cw = unicode_width::UnicodeWidthChar::width(*ch).unwrap_or(0);
        if width + cw > max {
            break;
        }
        width += cw;
        start = i;
    }
    chars[start..].iter().collect()
}

fn print_splash_scrollback() -> u16 {
    let version = env!("CARGO_PKG_VERSION");
    println!();
    println!("  \u{1f6e0} Talos v{version}");
    println!("  Safety-first agent runtime");
    println!();
    4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_to_width_ascii() {
        assert_eq!(truncate_end_to_width("hello world", 5), "world");
    }

    #[test]
    fn truncate_to_width_cjk() {
        assert_eq!(truncate_end_to_width("你好世界", 4), "世界");
    }

    #[test]
    fn truncate_to_width_short_enough() {
        assert_eq!(truncate_end_to_width("hi", 10), "hi");
    }
}
