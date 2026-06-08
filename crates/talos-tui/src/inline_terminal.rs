//! Custom terminal wrapper that enables dynamic viewport height.
//!
//! Inspired by Codex's `custom_terminal.rs`, this wrapper replaces ratatui's
//! `Terminal<Viewport::Inline(N)>` with a terminal that:
//! - Initializes with viewport height = 0 (no blank lines)
//! - Accepts a dynamic height parameter on each `draw()` call
//! - Pushes finalized content to scrollback via ANSI escape sequences
//! - Never queries cursor position after initialization

use std::io::{self, Stdout};

use crossterm::{
    cursor::{MoveTo, SetCursorStyle, Show},
    execute, queue,
    style::Print,
    terminal::{self, Clear, ClearType, EnableLineWrap},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    buffer::Buffer,
    layout::{Position, Rect, Size},
    widgets::{StatefulWidget, Widget},
};

/// A frame for rendering within the inline terminal.
pub struct InlineFrame<'a> {
    area: Rect,
    buffer: &'a mut Buffer,
}

impl<'a> InlineFrame<'a> {
    /// Returns the area of the current frame.
    pub const fn area(&self) -> Rect {
        self.area
    }

    /// Render a widget to the current buffer.
    pub fn render_widget<W: Widget>(&mut self, widget: W, area: Rect) {
        widget.render(area, self.buffer);
    }

    /// Render a stateful widget to the current buffer.
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

/// A custom terminal that supports dynamic viewport height.
///
/// Unlike ratatui's `Terminal<Viewport::Inline(N)>`, this terminal:
/// - Starts with viewport height = 0 (no blank lines on init)
/// - Adjusts viewport height on each `draw()` call
/// - Pushes content to scrollback without querying cursor position
pub struct InlineTerminal {
    backend: CrosstermBackend<Stdout>,
    buffers: [Buffer; 2],
    current: usize,
    viewport_area: Rect,
    screen_size: Size,
    last_known_cursor_pos: Position,
    needs_clear: bool,
}

impl InlineTerminal {
    /// Creates a new inline terminal.
    ///
    /// The viewport is anchored at the current cursor position with height = 0,
    /// so no blank lines are produced on initialization.
    ///
    /// # Errors
    ///
    /// Returns an error if the terminal size cannot be read or raw mode fails.
    pub fn new() -> io::Result<Self> {
        let stdout = io::stdout();
        let mut backend = CrosstermBackend::new(stdout);

        let screen_size = backend.size()?;
        let cursor_pos = backend.get_cursor_position().unwrap_or(Position::new(0, 0));

        let viewport_height = 4u16;
        let needed_bottom = cursor_pos.y.saturating_add(viewport_height);
        if needed_bottom > screen_size.height {
            let padding = needed_bottom.saturating_sub(screen_size.height);
            for _ in 0..padding {
                let _ = queue!(backend, Print("\n"));
            }
            let _ = io::Write::flush(&mut backend);
        }

        let cursor_pos = backend.get_cursor_position().unwrap_or(Position::new(0, 0));
        let viewport_area = Rect::new(0, cursor_pos.y, screen_size.width, 0);

        let buffers = [
            Buffer::empty(viewport_area),
            Buffer::empty(viewport_area),
        ];

        Ok(Self {
            backend,
            buffers,
            current: 0,
            viewport_area,
            screen_size,
            last_known_cursor_pos: cursor_pos,
            needs_clear: false,
        })
    }

    /// Returns a reference to the backend.
    #[allow(dead_code)]
    pub const fn backend(&self) -> &CrosstermBackend<Stdout> {
        &self.backend
    }

    /// Returns a mutable reference to the backend.
    pub fn backend_mut(&mut self) -> &mut CrosstermBackend<Stdout> {
        &mut self.backend
    }

    /// Returns the current viewport area.
    #[allow(dead_code)]
    pub const fn viewport_area(&self) -> Rect {
        self.viewport_area
    }

    /// Sets the viewport area, resizing buffers accordingly.
    ///
    /// This does NOT query the cursor position — it directly updates the
    /// internal state and reallocates buffers.
    pub fn set_viewport_area(&mut self, area: Rect) {
        if area == self.viewport_area {
            return;
        }

        if area.height < self.viewport_area.height && self.viewport_area.height > 0 {
            let writer = self.backend_mut();
            let _ = queue!(writer, MoveTo(0, area.bottom()));
            let _ = queue!(writer, Clear(ClearType::FromCursorDown));
            let _ = std::io::Write::flush(writer);
        }

        self.viewport_area = area;
        self.buffers[1 - self.current] = Buffer::empty(area);
        self.buffers[self.current] = Buffer::empty(area);
    }

    /// Returns the current screen size.
    #[allow(dead_code)]
    pub const fn screen_size(&self) -> Size {
        self.screen_size
    }

    /// Draws a frame with the given viewport height.
    ///
    /// The viewport height is adjusted before rendering. The draw function
    /// receives a `Frame` that covers the adjusted viewport area.
    ///
    /// # Errors
    ///
    /// Returns an error if the terminal size cannot be read or output fails.
    pub fn draw(
        &mut self,
        height: u16,
        draw_fn: impl FnOnce(&mut InlineFrame),
    ) -> io::Result<()> {
        let screen_size = self.backend.size()?;
        self.screen_size = screen_size;

        let mut area = self.viewport_area;
        area.height = height.min(screen_size.height);
        area.width = screen_size.width;

        if area.bottom() > screen_size.height {
            area.y = screen_size.height.saturating_sub(area.height);
        }

        let area_changed = area != self.viewport_area;
        if area_changed {
            self.set_viewport_area(area);
            self.needs_clear = true;
        }

        let force_clear = self.needs_clear;
        if force_clear {
            self.needs_clear = false;
        }

        self.draw_inner(draw_fn, force_clear)
    }

    fn draw_inner(
        &mut self,
        draw_fn: impl FnOnce(&mut InlineFrame),
        force_clear: bool,
    ) -> io::Result<()> {
        let area = self.viewport_area;
        let prev_idx = 1 - self.current;

        {
            let buffer = &mut self.buffers[self.current];
            buffer.reset();
            buffer.resize(area);

            let mut frame = InlineFrame { area, buffer };
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

    pub fn insert_history(&mut self, lines: &[String]) -> io::Result<()> {
        if lines.is_empty() {
            return Ok(());
        }

        let screen_height = self.screen_size.height;
        let mut area = self.viewport_area;
        let last_cursor_pos = self.last_known_cursor_pos;
        let mut should_update_area = false;

        // Phase 1: If viewport is not at screen bottom, scroll it down
        // using Reverse Index so it reaches the bottom.
        let cursor_top = if area.bottom() < screen_height {
            let scroll_amount = (lines.len() as u16).min(screen_height - area.bottom());
            if scroll_amount > 0 {
                let top_1based = area.top() + 1;
                let bottom_1based = screen_height;
                if top_1based < bottom_1based {
                    let _ = queue!(
                        self.backend_mut(),
                        crossterm::style::Print(format!(
                            "\x1b[{};{}r",
                            top_1based, bottom_1based
                        ))
                    );
                }
                let _ = queue!(self.backend_mut(), MoveTo(0, area.top()));
                for _ in 0..scroll_amount {
                    let _ = queue!(self.backend_mut(), crossterm::style::Print("\x1bM"));
                }
                let _ = queue!(self.backend_mut(), crossterm::style::Print("\x1b[r"));
            }
            let ct = area.top().saturating_sub(1);
            area.y += scroll_amount;
            should_update_area = true;
            ct
        } else {
            area.top().saturating_sub(1)
        };

        // Phase 2: Set scroll region to rows above the viewport and write
        // history lines. Each \r\n inside this region pushes old content
        // into the terminal's native scrollback.
        if area.top() > 0 {
            let _ = queue!(
                self.backend_mut(),
                crossterm::style::Print(format!("\x1b[1;{}r", area.top()))
            );
        }

        let _ = queue!(self.backend_mut(), MoveTo(0, cursor_top));
        for line in lines {
            let _ = queue!(self.backend_mut(), Print("\r\n"));
            let _ = queue!(self.backend_mut(), Print(line));
        }

        let _ = queue!(self.backend_mut(), crossterm::style::Print("\x1b[r"));

        // Phase 3: Update viewport position only when it was scrolled.
        if should_update_area {
            self.viewport_area = area;
            self.buffers = [
                Buffer::empty(self.viewport_area),
                Buffer::empty(self.viewport_area),
            ];
            self.needs_clear = true;
        }

        let _ = queue!(
            self.backend_mut(),
            MoveTo(last_cursor_pos.x, last_cursor_pos.y)
        );

        let _ = std::io::Write::flush(self.backend_mut());

        Ok(())
    }

    /// Restores the terminal to its original state.
    ///
    /// Disables raw mode, shows the cursor, and resets cursor style.
    pub fn restore(&self) {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(
            io::stdout(),
            EnableLineWrap,
            SetCursorStyle::DefaultUserShape,
            Show
        );
    }

    /// Returns the area of the last completed frame.
    #[allow(dead_code)]
    pub fn get_frame_area(&self) -> Rect {
        self.viewport_area
    }
}

impl Drop for InlineTerminal {
    fn drop(&mut self) {
        self.restore();
    }
}
