use std::io::{self, Stdout};

use crossterm::{
    cursor::{Hide, MoveTo, SetCursorStyle, Show},
    event::{
        DisableBracketedPaste, EnableBracketedPaste, KeyboardEnhancementFlags,
        PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute, queue,
    style::Color as CColor,
    terminal::{
        self, Clear, ClearType, EnableLineWrap, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    buffer::Buffer,
    layout::{Position, Rect, Size},
    widgets::{StatefulWidget, Widget},
};

pub struct InlineFrame<'a> {
    #[allow(dead_code)]
    area: Rect,
    buffer: &'a mut Buffer,
}

impl<'a> InlineFrame<'a> {
    #[allow(dead_code)]
    pub const fn area(&self) -> Rect {
        self.area
    }

    #[cfg(test)]
    pub(crate) fn new(area: Rect, buffer: &'a mut Buffer) -> Self {
        Self { area, buffer }
    }

    pub fn render_widget<W: Widget>(&mut self, widget: W, area: Rect) {
        widget.render(area, self.buffer);
    }

    #[allow(dead_code)]
    pub fn render_stateful_widget<W: StatefulWidget>(
        &mut self,
        widget: W,
        area: Rect,
        state: &mut W::State,
    ) {
        widget.render(area, self.buffer, state);
    }
}

pub trait ViewportComponent {
    fn height_hint(&self, available_width: u16) -> u16;
    fn render(&self, frame: &mut InlineFrame, area: Rect);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct HistoryAttrs {
    pub(crate) bold: bool,
    pub(crate) italic: bool,
    pub(crate) underlined: bool,
    pub(crate) dim: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HistorySegment {
    pub(crate) text: String,
    pub(crate) fg: Option<CColor>,
    pub(crate) attrs: HistoryAttrs,
}

impl HistorySegment {
    pub(crate) fn raw(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            fg: None,
            attrs: HistoryAttrs::default(),
        }
    }

    pub(crate) fn styled(text: impl Into<String>, fg: Option<CColor>, attrs: HistoryAttrs) -> Self {
        Self {
            text: text.into(),
            fg,
            attrs,
        }
    }
}

/// Legacy layout helper retained solely for isolated component tests. The
/// interactive renderer uses `AppLayout` directly.
#[cfg(test)]
pub struct ComponentStack<'a> {
    components: Vec<&'a dyn ViewportComponent>,
}

#[cfg(test)]
impl<'a> ComponentStack<'a> {
    pub fn new(components: Vec<&'a dyn ViewportComponent>) -> Self {
        Self { components }
    }
    pub fn total_height(&self, width: u16) -> u16 {
        self.components.iter().map(|c| c.height_hint(width)).sum()
    }
    pub fn layout(&self, area: Rect, width: u16) -> Vec<(&'a dyn ViewportComponent, Rect)> {
        let mut y = area.y;
        self.components
            .iter()
            .filter_map(|component| {
                let height = component
                    .height_hint(width)
                    .min(area.bottom().saturating_sub(y));
                (height > 0).then(|| {
                    let rect = Rect::new(area.x, y, area.width, height);
                    y = y.saturating_add(height);
                    (*component, rect)
                })
            })
            .collect()
    }
}

pub struct TerminalSession {
    backend: CrosstermBackend<Stdout>,
    buffers: [Buffer; 2],
    current: usize,
    frame_area: Rect,
    screen_size: Size,
    last_known_cursor_pos: Position,
    lifecycle: TerminalLifecycleState,
    #[cfg(test)]
    test_mode: bool,
    #[cfg(test)]
    cursor_visible: bool,
    #[cfg(test)]
    test_cursor_position: Option<Position>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct TerminalLifecycleState {
    raw_mode: bool,
    alternate_screen: bool,
    cursor_hidden: bool,
    bracketed_paste: bool,
    keyboard_enhancement: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TerminalAction {
    EnableRawMode,
    PushKeyboardEnhancement,
    EnterAlternateScreen,
    HideCursor,
    EnableBracketedPaste,
    ClearFrame,
    DisableBracketedPaste,
    ShowCursor,
    LeaveAlternateScreen,
    PopKeyboardEnhancement,
    DisableRawMode,
}

fn keyboard_enhancement_flags() -> KeyboardEnhancementFlags {
    KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
        | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
        | KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS
}

fn keyboard_enhancement_supported(result: io::Result<bool>) -> bool {
    result.unwrap_or(false)
}

impl TerminalSession {
    pub fn new() -> io::Result<Self> {
        let stdout = io::stdout();
        let mut backend = CrosstermBackend::new(stdout);

        let screen_size = backend.size()?;
        let keyboard_enabled =
            keyboard_enhancement_supported(terminal::supports_keyboard_enhancement());
        let lifecycle = initialize_lifecycle(keyboard_enabled, |action| {
            execute_backend_action(&mut backend, action)
        })?;
        let frame_area = Rect::new(0, 0, screen_size.width, screen_size.height);

        let buffers = [Buffer::empty(frame_area), Buffer::empty(frame_area)];

        Ok(Self {
            backend,
            buffers,
            current: 0,
            frame_area,
            screen_size,
            last_known_cursor_pos: Position::new(0, 0),
            lifecycle,
            #[cfg(test)]
            test_mode: false,
            #[cfg(test)]
            cursor_visible: false,
            #[cfg(test)]
            test_cursor_position: None,
        })
    }

    /// Creates a minimal instance for unit testing without terminal access.
    #[cfg(test)]
    pub(crate) fn test_instance() -> Self {
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let frame_area = Rect::new(0, 0, 80, 24);
        let buffers = [Buffer::empty(frame_area), Buffer::empty(frame_area)];
        Self {
            backend,
            buffers,
            current: 0,
            frame_area,
            screen_size: Size::new(80, 24),
            last_known_cursor_pos: Position::new(0, 0),
            lifecycle: TerminalLifecycleState::default(),
            test_mode: true,
            cursor_visible: false,
            test_cursor_position: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn set_test_size(&mut self, size: Size) {
        self.screen_size = size;
    }

    #[cfg(test)]
    pub(crate) const fn test_cursor_visible(&self) -> bool {
        self.cursor_visible
    }

    #[cfg(test)]
    pub(crate) const fn test_cursor_position(&self) -> Option<Position> {
        self.test_cursor_position
    }

    #[cfg(test)]
    pub(crate) fn test_rendered_text(&self) -> String {
        let buffer = &self.buffers[1 - self.current];
        let area = buffer.area;
        (area.y..area.bottom())
            .map(|y| {
                (area.x..area.right())
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
                    .trim_end()
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[allow(dead_code)]
    pub const fn backend(&self) -> &CrosstermBackend<Stdout> {
        &self.backend
    }

    pub fn backend_mut(&mut self) -> &mut CrosstermBackend<Stdout> {
        &mut self.backend
    }

    #[allow(dead_code)]
    pub fn size(&mut self) -> io::Result<Size> {
        #[cfg(test)]
        if self.test_mode {
            return Ok(self.screen_size);
        }
        let size = self.backend.size()?;
        self.screen_size = size;
        Ok(size)
    }

    pub fn draw(&mut self, size: Size, draw_fn: impl FnOnce(&mut InlineFrame)) -> io::Result<()> {
        self.screen_size = size;
        let area = Rect::new(0, 0, size.width, size.height);
        let changed = self.frame_area != area;
        self.frame_area = area;
        self.buffers[0].resize(area);
        self.buffers[1].resize(area);
        self.draw_inner(draw_fn, changed, size.height)
    }

    fn draw_inner(
        &mut self,
        draw_fn: impl FnOnce(&mut InlineFrame),
        force_clear: bool,
        render_height: u16,
    ) -> io::Result<()> {
        let area = self.frame_area;
        let prev_idx = 1 - self.current;

        let render_area = Rect {
            height: render_height.min(area.height),
            ..area
        };

        {
            let buffer = &mut self.buffers[self.current];
            buffer.reset();
            buffer.resize(area);

            let mut frame = InlineFrame {
                area: render_area,
                buffer,
            };
            draw_fn(&mut frame);
        }

        #[cfg(test)]
        if self.test_mode {
            self.current = prev_idx;
            return Ok(());
        }

        if force_clear {
            let writer = self.backend_mut();
            for y in area.y..area.bottom() {
                queue!(writer, MoveTo(0, y))?;
                queue!(writer, Clear(ClearType::UntilNewLine))?;
            }
            io::Write::flush(writer)?;
        }

        let prev_buf = &self.buffers[prev_idx];
        let current = &self.buffers[self.current];

        if force_clear {
            let blank = Buffer::empty(area);
            let cells: Vec<_> = blank.diff_iter(current).collect();
            self.backend.draw(cells.into_iter())?;
        } else {
            let cells: Vec<_> = prev_buf.diff_iter(current).collect();
            self.backend.draw(cells.into_iter())?;
        }

        Backend::flush(&mut self.backend)?;

        self.current = prev_idx;

        Ok(())
    }

    pub fn set_cursor(&mut self, col: u16, row: u16) -> io::Result<()> {
        let Some(position) = clamp_cursor(Position::new(col, row), self.screen_size) else {
            return Ok(());
        };
        self.last_known_cursor_pos = position;
        #[cfg(test)]
        if self.test_mode {
            self.cursor_visible = true;
            self.test_cursor_position = Some(position);
            return Ok(());
        }
        let writer = self.backend_mut();
        queue!(writer, MoveTo(position.x, position.y))?;
        queue!(writer, Show)?;
        io::Write::flush(writer)?;
        Ok(())
    }

    pub fn hide_cursor(&mut self) -> io::Result<()> {
        #[cfg(test)]
        if self.test_mode {
            self.cursor_visible = false;
            self.test_cursor_position = None;
            return Ok(());
        }
        let writer = self.backend_mut();
        queue!(writer, Hide)?;
        io::Write::flush(writer)
    }

    pub fn set_cursor_in_rect(
        &mut self,
        rect: Rect,
        local_col: u16,
        local_row: u16,
    ) -> io::Result<()> {
        if rect.width == 0 || rect.height == 0 {
            return self.hide_cursor();
        }
        let position = cursor_position_in_rect(rect, local_col, local_row)
            .expect("non-empty rectangle has a cursor position");
        self.set_cursor(position.x, position.y)
    }

    /// Sets a modal cursor only when its semantic row exists in the rendered
    /// component rectangle. Unlike the composer helper, this never clamps a
    /// vertical coordinate onto a different panel row.
    pub fn set_cursor_if_visible_in_rect(
        &mut self,
        rect: Rect,
        local_col: u16,
        local_row: u16,
    ) -> io::Result<()> {
        if rect.width == 0 || rect.height == 0 || local_row >= rect.height {
            return self.hide_cursor();
        }
        self.set_cursor(
            rect.x
                .saturating_add(local_col.min(rect.width.saturating_sub(1))),
            rect.y.saturating_add(local_row),
        )
    }

    pub fn restore(&mut self) -> io::Result<()> {
        restore_lifecycle(&mut self.lifecycle, execute_stdout_action)
    }

    #[allow(dead_code)]
    pub fn get_frame_area(&self) -> Rect {
        self.frame_area
    }
}

fn cursor_position_in_rect(rect: Rect, local_col: u16, local_row: u16) -> Option<Position> {
    (rect.width > 0 && rect.height > 0).then(|| {
        let max_col = rect.width.saturating_sub(1);
        let max_row = rect.height.saturating_sub(1);
        Position::new(
            rect.x.saturating_add(local_col.min(max_col)),
            rect.y.saturating_add(local_row.min(max_row)),
        )
    })
}

fn initialize_lifecycle(
    keyboard_enabled: bool,
    mut run: impl FnMut(TerminalAction) -> io::Result<()>,
) -> io::Result<TerminalLifecycleState> {
    let mut lifecycle = TerminalLifecycleState::default();
    run(TerminalAction::EnableRawMode)?;
    lifecycle.raw_mode = true;
    if keyboard_enabled {
        if let Err(error) = run(TerminalAction::PushKeyboardEnhancement) {
            return Err(setup_error(error, &mut lifecycle, &mut run));
        }
        lifecycle.keyboard_enhancement = true;
    }
    for action in [
        TerminalAction::EnterAlternateScreen,
        TerminalAction::HideCursor,
        TerminalAction::EnableBracketedPaste,
    ] {
        if let Err(error) = run(action) {
            return Err(setup_error(error, &mut lifecycle, &mut run));
        }
        match action {
            TerminalAction::EnterAlternateScreen => lifecycle.alternate_screen = true,
            TerminalAction::HideCursor => lifecycle.cursor_hidden = true,
            TerminalAction::EnableBracketedPaste => lifecycle.bracketed_paste = true,
            _ => unreachable!("only setup actions are listed above"),
        }
    }
    if let Err(error) = run(TerminalAction::ClearFrame) {
        return Err(setup_error(error, &mut lifecycle, &mut run));
    }
    Ok(lifecycle)
}

fn setup_error(
    setup: io::Error,
    lifecycle: &mut TerminalLifecycleState,
    run: &mut impl FnMut(TerminalAction) -> io::Result<()>,
) -> io::Error {
    match rollback_lifecycle(lifecycle, run) {
        Ok(()) => setup,
        Err(cleanup) => {
            io::Error::new(setup.kind(), format!("{setup}; rollback failed: {cleanup}"))
        }
    }
}

fn rollback_lifecycle(
    lifecycle: &mut TerminalLifecycleState,
    run: &mut impl FnMut(TerminalAction) -> io::Result<()>,
) -> io::Result<()> {
    // Preserve the setup error: rollback attempts every completed transition.
    restore_lifecycle(lifecycle, run)
}

fn restore_lifecycle(
    lifecycle: &mut TerminalLifecycleState,
    mut run: impl FnMut(TerminalAction) -> io::Result<()>,
) -> io::Result<()> {
    let mut first_error = None;
    attempt_restore(
        &mut lifecycle.bracketed_paste,
        TerminalAction::DisableBracketedPaste,
        &mut run,
        &mut first_error,
    );
    attempt_restore(
        &mut lifecycle.cursor_hidden,
        TerminalAction::ShowCursor,
        &mut run,
        &mut first_error,
    );
    attempt_restore(
        &mut lifecycle.alternate_screen,
        TerminalAction::LeaveAlternateScreen,
        &mut run,
        &mut first_error,
    );
    attempt_restore(
        &mut lifecycle.keyboard_enhancement,
        TerminalAction::PopKeyboardEnhancement,
        &mut run,
        &mut first_error,
    );
    attempt_restore(
        &mut lifecycle.raw_mode,
        TerminalAction::DisableRawMode,
        &mut run,
        &mut first_error,
    );
    first_error.map_or(Ok(()), Err)
}

fn attempt_restore(
    enabled: &mut bool,
    action: TerminalAction,
    run: &mut impl FnMut(TerminalAction) -> io::Result<()>,
    first_error: &mut Option<io::Error>,
) {
    if !*enabled {
        return;
    }
    match run(action) {
        Ok(()) => *enabled = false,
        Err(error) if first_error.is_none() => *first_error = Some(error),
        Err(_) => {}
    }
}

fn execute_backend_action(
    backend: &mut CrosstermBackend<Stdout>,
    action: TerminalAction,
) -> io::Result<()> {
    match action {
        TerminalAction::EnableRawMode => terminal::enable_raw_mode(),
        TerminalAction::PushKeyboardEnhancement => {
            execute!(
                backend,
                PushKeyboardEnhancementFlags(keyboard_enhancement_flags())
            )
        }
        TerminalAction::EnterAlternateScreen => execute!(backend, EnterAlternateScreen),
        TerminalAction::HideCursor => execute!(backend, Hide),
        TerminalAction::EnableBracketedPaste => execute!(backend, EnableBracketedPaste),
        TerminalAction::ClearFrame => execute!(backend, Clear(ClearType::All), MoveTo(0, 0)),
        action => execute_stdout_action(action),
    }
}

fn execute_stdout_action(action: TerminalAction) -> io::Result<()> {
    let mut stdout = io::stdout();
    match action {
        TerminalAction::DisableBracketedPaste => execute!(stdout, DisableBracketedPaste),
        TerminalAction::ShowCursor => execute!(stdout, SetCursorStyle::DefaultUserShape, Show),
        TerminalAction::LeaveAlternateScreen => execute!(
            stdout,
            crossterm::style::ResetColor,
            EnableLineWrap,
            LeaveAlternateScreen
        ),
        TerminalAction::PopKeyboardEnhancement => execute!(stdout, PopKeyboardEnhancementFlags),
        TerminalAction::DisableRawMode => terminal::disable_raw_mode(),
        _ => Err(io::Error::other("invalid terminal restore action")),
    }
}

pub(crate) fn clamp_cursor(position: Position, size: Size) -> Option<Position> {
    (size.width > 0 && size.height > 0).then(|| {
        Position::new(
            position.x.min(size.width.saturating_sub(1)),
            position.y.min(size.height.saturating_sub(1)),
        )
    })
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn initialize_with_failure(
        fail_at: TerminalAction,
    ) -> (io::Result<TerminalLifecycleState>, Vec<TerminalAction>) {
        let mut actions = Vec::new();
        let result = initialize_lifecycle(false, |action| {
            actions.push(action);
            (action != fail_at)
                .then_some(())
                .ok_or_else(|| io::Error::other("injected terminal failure"))
        });
        (result, actions)
    }

    #[test]
    fn keyboard_flags_disambiguate_modified_enter() {
        assert!(
            keyboard_enhancement_flags()
                .contains(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
        );
        assert!(
            keyboard_enhancement_flags()
                .contains(KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES)
        );
        assert!(
            keyboard_enhancement_flags().contains(KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS)
        );
    }

    #[test]
    fn keyboard_support_probe_degrades_on_false_or_error() {
        assert!(keyboard_enhancement_supported(Ok(true)));
        assert!(!keyboard_enhancement_supported(Ok(false)));
        assert!(!keyboard_enhancement_supported(Err(io::Error::other(
            "probe failed"
        ))));
    }

    #[test]
    fn cursor_is_clamped_to_terminal_bounds() {
        assert_eq!(
            clamp_cursor(Position::new(9, 9), Size::new(2, 3)),
            Some(Position::new(1, 2))
        );
        assert_eq!(clamp_cursor(Position::new(0, 0), Size::new(0, 1)), None);
        assert_eq!(clamp_cursor(Position::new(0, 0), Size::new(1, 0)), None);
    }

    #[test]
    fn cursor_in_rect_clamps_to_final_component_bounds() {
        let rect = Rect::new(5, 7, 3, 2);
        assert_eq!(
            cursor_position_in_rect(rect, 99, 99),
            Some(Position::new(7, 8))
        );
        assert_eq!(cursor_position_in_rect(Rect::new(0, 0, 0, 2), 0, 0), None);
    }

    #[test]
    fn modal_cursor_hides_instead_of_clamping_a_clipped_row() {
        let mut session = TerminalSession::test_instance();
        session.set_test_size(Size::new(20, 10));
        let panel = Rect::new(4, 5, 3, 2);

        session
            .set_cursor_if_visible_in_rect(panel, 99, 2)
            .expect("hiding a clipped modal cursor succeeds");
        assert!(!session.test_cursor_visible());
        assert_eq!(session.test_cursor_position(), None);

        session
            .set_cursor_if_visible_in_rect(panel, 99, 1)
            .expect("visible modal cursor succeeds");
        assert!(session.test_cursor_visible());
        assert_eq!(session.test_cursor_position(), Some(Position::new(6, 6)));
    }

    #[test]
    fn alternate_screen_entry_failure_aborts_terminal_session() {
        let (result, actions) = initialize_with_failure(TerminalAction::EnterAlternateScreen);
        assert!(result.is_err());
        assert_eq!(
            actions,
            vec![
                TerminalAction::EnableRawMode,
                TerminalAction::EnterAlternateScreen,
                TerminalAction::DisableRawMode,
            ]
        );
    }

    #[test]
    fn bracketed_paste_enable_failure_rolls_back_entered_states() {
        let (result, actions) = initialize_with_failure(TerminalAction::EnableBracketedPaste);
        assert!(result.is_err());
        assert_eq!(
            actions,
            vec![
                TerminalAction::EnableRawMode,
                TerminalAction::EnterAlternateScreen,
                TerminalAction::HideCursor,
                TerminalAction::EnableBracketedPaste,
                TerminalAction::ShowCursor,
                TerminalAction::LeaveAlternateScreen,
                TerminalAction::DisableRawMode,
            ]
        );
    }

    #[test]
    fn cursor_hide_failure_rolls_back_alternate_screen() {
        let (result, actions) = initialize_with_failure(TerminalAction::HideCursor);
        assert!(result.is_err());
        assert_eq!(
            actions,
            vec![
                TerminalAction::EnableRawMode,
                TerminalAction::EnterAlternateScreen,
                TerminalAction::HideCursor,
                TerminalAction::LeaveAlternateScreen,
                TerminalAction::DisableRawMode,
            ]
        );
    }

    #[test]
    fn restore_is_idempotent_after_partial_initialization() {
        let mut lifecycle = TerminalLifecycleState {
            raw_mode: true,
            alternate_screen: true,
            ..TerminalLifecycleState::default()
        };
        let mut actions = Vec::new();
        restore_lifecycle(&mut lifecycle, |action| {
            actions.push(action);
            Ok(())
        })
        .expect("restore succeeds");
        restore_lifecycle(&mut lifecycle, |action| {
            actions.push(action);
            Ok(())
        })
        .expect("second restore succeeds");
        assert_eq!(
            actions,
            vec![
                TerminalAction::LeaveAlternateScreen,
                TerminalAction::DisableRawMode
            ]
        );
    }

    #[test]
    fn leave_alternate_screen_runs_only_if_enter_succeeded() {
        let mut lifecycle = TerminalLifecycleState {
            raw_mode: true,
            ..TerminalLifecycleState::default()
        };
        let mut actions = Vec::new();
        restore_lifecycle(&mut lifecycle, |action| {
            actions.push(action);
            Ok(())
        })
        .expect("restore succeeds");
        assert_eq!(actions, vec![TerminalAction::DisableRawMode]);
    }

    #[test]
    fn drop_restore_only_targets_enabled_terminal_states() {
        let mut lifecycle = TerminalLifecycleState {
            bracketed_paste: true,
            cursor_hidden: true,
            ..TerminalLifecycleState::default()
        };
        let mut actions = Vec::new();
        restore_lifecycle(&mut lifecycle, |action| {
            actions.push(action);
            Ok(())
        })
        .expect("restore succeeds");
        assert_eq!(
            actions,
            vec![
                TerminalAction::DisableBracketedPaste,
                TerminalAction::ShowCursor
            ]
        );
    }

    #[test]
    fn restore_attempts_all_enabled_states_and_retries_only_failures() {
        let mut lifecycle = TerminalLifecycleState {
            raw_mode: true,
            alternate_screen: true,
            cursor_hidden: true,
            bracketed_paste: true,
            keyboard_enhancement: true,
        };
        let mut first = Vec::new();
        assert!(
            restore_lifecycle(&mut lifecycle, |action| {
                first.push(action);
                (action != TerminalAction::DisableBracketedPaste)
                    .then_some(())
                    .ok_or_else(|| io::Error::other("paste failure"))
            })
            .is_err()
        );
        assert_eq!(first.len(), 5);
        assert!(lifecycle.bracketed_paste);
        assert!(!lifecycle.cursor_hidden && !lifecycle.alternate_screen);
        let mut retry = Vec::new();
        restore_lifecycle(&mut lifecycle, |action| {
            retry.push(action);
            Ok(())
        })
        .expect("retry succeeds");
        assert_eq!(retry, vec![TerminalAction::DisableBracketedPaste]);
    }
}
