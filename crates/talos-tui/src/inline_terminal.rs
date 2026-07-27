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

pub struct ComponentStack<'a> {
    components: Vec<&'a dyn ViewportComponent>,
}

impl<'a> ComponentStack<'a> {
    pub fn new(components: Vec<&'a dyn ViewportComponent>) -> Self {
        Self { components }
    }

    pub fn total_height(&self, available_width: u16) -> u16 {
        self.components
            .iter()
            .map(|c| c.height_hint(available_width))
            .sum()
    }

    pub fn layout(
        &self,
        area: Rect,
        available_width: u16,
    ) -> Vec<(&'a dyn ViewportComponent, Rect)> {
        let mut result = Vec::new();
        let mut y = area.y;

        for component in &self.components {
            let remaining = area.bottom().saturating_sub(y);
            let h = component.height_hint(available_width).min(remaining);
            if h == 0 {
                continue;
            }
            let rect = Rect {
                x: area.x,
                y,
                width: area.width,
                height: h,
            };
            result.push((*component, rect));
            y = y.saturating_add(h);
        }

        result
    }
}

pub struct InlineTerminal {
    backend: CrosstermBackend<Stdout>,
    buffers: [Buffer; 2],
    current: usize,
    frame_area: Rect,
    screen_size: Size,
    last_known_cursor_pos: Position,
    restored: bool,
    keyboard_enhancement_enabled: bool,
}

fn keyboard_enhancement_flags() -> KeyboardEnhancementFlags {
    KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
        | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
        | KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS
}

fn keyboard_enhancement_supported(result: io::Result<bool>) -> bool {
    result.unwrap_or(false)
}

impl InlineTerminal {
    pub fn new() -> io::Result<Self> {
        let stdout = io::stdout();
        let mut backend = CrosstermBackend::new(stdout);

        let screen_size = backend.size()?;

        let keyboard_enhancement_enabled =
            if keyboard_enhancement_supported(terminal::supports_keyboard_enhancement()) {
                execute!(
                    backend,
                    PushKeyboardEnhancementFlags(keyboard_enhancement_flags())
                )
                .is_ok()
            } else {
                false
            };
        let _ = execute!(
            backend,
            EnterAlternateScreen,
            Hide,
            EnableBracketedPaste,
            Clear(ClearType::All),
            MoveTo(0, 0)
        );
        let frame_area = Rect::new(0, 0, screen_size.width, screen_size.height);

        let buffers = [Buffer::empty(frame_area), Buffer::empty(frame_area)];

        Ok(Self {
            backend,
            buffers,
            current: 0,
            frame_area,
            screen_size,
            last_known_cursor_pos: Position::new(0, 0),
            restored: false,
            keyboard_enhancement_enabled,
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
            restored: false,
            keyboard_enhancement_enabled: false,
        }
    }

    #[allow(dead_code)]
    pub const fn backend(&self) -> &CrosstermBackend<Stdout> {
        &self.backend
    }

    pub fn backend_mut(&mut self) -> &mut CrosstermBackend<Stdout> {
        &mut self.backend
    }

    #[allow(dead_code)]
    pub const fn viewport_area(&self) -> Rect {
        self.frame_area
    }

    pub fn notify_resize(&mut self) {
        // Resize only invalidates the next full-frame projection. No terminal
        // cleanup is needed because alternate screen is an output surface.
    }

    #[allow(dead_code)]
    pub const fn screen_size(&self) -> Size {
        self.screen_size
    }

    pub fn size(&mut self) -> io::Result<Size> {
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
        let writer = self.backend_mut();
        queue!(writer, MoveTo(position.x, position.y))?;
        queue!(writer, Show)?;
        io::Write::flush(writer)?;
        Ok(())
    }

    pub fn restore(&mut self) {
        if self.restored {
            return;
        }
        self.restored = true;
        let _ = execute!(io::stdout(), crossterm::style::ResetColor,);
        if self.keyboard_enhancement_enabled {
            let _ = execute!(io::stdout(), PopKeyboardEnhancementFlags);
            self.keyboard_enhancement_enabled = false;
        }
        let _ = execute!(
            io::stdout(),
            DisableBracketedPaste,
            EnableLineWrap,
            SetCursorStyle::DefaultUserShape,
            Show,
            LeaveAlternateScreen
        );
        let _ = terminal::disable_raw_mode();
    }

    #[allow(dead_code)]
    pub fn get_frame_area(&self) -> Rect {
        self.frame_area
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

impl Drop for InlineTerminal {
    fn drop(&mut self) {
        self.restore();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
