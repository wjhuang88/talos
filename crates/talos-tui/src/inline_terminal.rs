use std::io::{self, Stdout};

use crossterm::{
    cursor::{Hide, MoveTo, SetCursorStyle, Show},
    event::{DisableBracketedPaste, EnableBracketedPaste},
    execute, queue,
    style::{Color as CColor, Print, SetBackgroundColor},
    terminal::{self, Clear, ClearType, EnableLineWrap},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    buffer::Buffer,
    layout::{Position, Rect, Size},
    widgets::{StatefulWidget, Widget},
};

pub struct InlineFrame<'a> {
    area: Rect,
    buffer: &'a mut Buffer,
}

impl<'a> InlineFrame<'a> {
    pub const fn area(&self) -> Rect {
        self.area
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
            let h = component.height_hint(available_width);
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
    viewport_area: Rect,
    screen_size: Size,
    last_known_cursor_pos: Position,
    needs_clear: bool,
}

impl InlineTerminal {
    pub fn new() -> io::Result<Self> {
        let stdout = io::stdout();
        let mut backend = CrosstermBackend::new(stdout);

        let screen_size = backend.size()?;
        let cursor_pos = backend.get_cursor_position().unwrap_or(Position::new(0, 0));

        let _ = execute!(backend, Hide, EnableBracketedPaste);
        let viewport_area = Rect::new(0, cursor_pos.y, screen_size.width, 0);

        let buffers = [Buffer::empty(viewport_area), Buffer::empty(viewport_area)];

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

    #[allow(dead_code)]
    pub const fn backend(&self) -> &CrosstermBackend<Stdout> {
        &self.backend
    }

    pub fn backend_mut(&mut self) -> &mut CrosstermBackend<Stdout> {
        &mut self.backend
    }

    #[allow(dead_code)]
    pub const fn viewport_area(&self) -> Rect {
        self.viewport_area
    }

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

    #[allow(dead_code)]
    pub const fn screen_size(&self) -> Size {
        self.screen_size
    }

    pub fn draw(&mut self, height: u16, draw_fn: impl FnOnce(&mut InlineFrame)) -> io::Result<()> {
        let screen_size = self.backend.size()?;
        self.screen_size = screen_size;

        let mut area = self.viewport_area;
        area.height = height.min(screen_size.height);
        area.width = screen_size.width;

        if area.bottom() > screen_size.height {
            area.y = screen_size.height.saturating_sub(area.height);
        }

        let area_changed = area != self.viewport_area;
        let shrinking = area.height < self.viewport_area.height;
        if area_changed {
            self.set_viewport_area(area);
            if shrinking {
                self.needs_clear = true;
            }
        }

        let force_clear = self.needs_clear;
        if force_clear {
            self.needs_clear = false;
        }

        self.draw_inner(draw_fn, force_clear, height)
    }

    fn draw_inner(
        &mut self,
        draw_fn: impl FnOnce(&mut InlineFrame),
        force_clear: bool,
        render_height: u16,
    ) -> io::Result<()> {
        let area = self.viewport_area;
        let prev_idx = 1 - self.current;

        let render_area = Rect {
            y: area.bottom().saturating_sub(render_height),
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

    pub fn insert_history(&mut self, line: &str, bg: Option<CColor>) -> io::Result<()> {
        let screen_height = self.screen_size.height;
        let line_width = self.screen_size.width;
        let mut area = self.viewport_area;
        let writer = self.backend_mut();

        let styled_print =
            |w: &mut CrosstermBackend<Stdout>, text: &str, bg: Option<CColor>| -> io::Result<()> {
                if let Some(color) = bg {
                    queue!(w, SetBackgroundColor(color))?;
                    queue!(w, Print(text))?;
                    let text_width = unicode_width::UnicodeWidthStr::width(text);
                    if text_width < line_width as usize {
                        queue!(w, Print(" ".repeat(line_width as usize - text_width)))?;
                    }
                    queue!(w, SetBackgroundColor(CColor::Reset))?;
                } else {
                    queue!(w, Print(text))?;
                }
                Ok(())
            };

        if area.bottom() < screen_height {
            let top_1based = area.top() + 1;
            queue!(
                writer,
                crossterm::style::Print(format!("\x1b[{};{}r", top_1based, screen_height))
            )?;
            queue!(writer, MoveTo(0, area.top()))?;
            queue!(writer, crossterm::style::Print("\x1bM"))?;
            queue!(writer, crossterm::style::Print("\x1b[r"))?;
            area.y += 1;
            queue!(writer, MoveTo(0, area.top() - 1))?;
            queue!(writer, Clear(ClearType::UntilNewLine))?;
            styled_print(writer, line, bg)?;
        } else if area.top() > 1 {
            queue!(
                writer,
                crossterm::style::Print(format!("\x1b[1;{}r", area.top()))
            )?;
            queue!(writer, MoveTo(0, area.top() - 1))?;
            queue!(writer, crossterm::style::Print("\r\n"))?;
            queue!(writer, Clear(ClearType::UntilNewLine))?;
            styled_print(writer, line, bg)?;
            queue!(writer, crossterm::style::Print("\x1b[r"))?;
        }

        std::io::Write::flush(writer)?;

        let area_changed = area != self.viewport_area;
        if area_changed {
            self.set_viewport_area(area);
        }
        self.needs_clear = true;

        Ok(())
    }

    pub fn set_cursor(&mut self, col: u16, row: u16) -> io::Result<()> {
        self.last_known_cursor_pos = Position::new(col, row);
        let writer = self.backend_mut();
        queue!(writer, MoveTo(col, row))?;
        queue!(writer, Show)?;
        io::Write::flush(writer)?;
        Ok(())
    }

    pub fn restore(&self) {
        let _ = execute!(
            io::stdout(),
            MoveTo(0, self.viewport_area.top()),
            Clear(ClearType::FromCursorDown),
        );
        let _ = terminal::disable_raw_mode();
        let _ = execute!(
            io::stdout(),
            DisableBracketedPaste,
            EnableLineWrap,
            SetCursorStyle::DefaultUserShape,
            Show
        );
    }

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
