// This is derived from Codex's custom_terminal.rs, which is itself derived from
// ratatui::Terminal (MIT, Florian Dehau + Ratatui Developers).
//
// The key insight: inline-by-default TUI needs to track viewport position and
// history rows separately from ratatui's standard Terminal.

use std::io::{self, Write};

use anyhow::Result;
use crossterm::{
    cursor::{MoveTo, position},
    event::{DisableMouseCapture, EnableMouseCapture},
    execute, queue,
    terminal::{self, ClearType, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Rect,
    Terminal,
};

/// Custom terminal wrapper that tracks inline viewport position.
///
/// Unlike ratatui's standard Terminal, this tracks:
/// - `viewport_area`: The region where the TUI renders (starts at cursor y on startup)
/// - `visible_history_rows`: How many history rows have been pushed above the viewport
pub struct CustomTerminal {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    pub viewport_area: Rect,
    pub visible_history_rows: u16,
}

impl CustomTerminal {
    /// Creates a new CustomTerminal, capturing the current cursor position as viewport top.
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnableMouseCapture)?;
        
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        
        let (_cursor_x, cursor_y) = position().unwrap_or((0, 0));
        let (width, height) = terminal::size().unwrap_or((80, 24));
        
        let viewport_area = Rect::new(
            0,
            cursor_y,
            width,
            height.saturating_sub(cursor_y),
        );
        
        Ok(Self {
            terminal,
            viewport_area,
            visible_history_rows: 0,
        })
    }
    
    /// Returns the number of visible history rows rendered above the viewport.
    pub fn visible_history_rows(&self) -> u16 {
        self.visible_history_rows
    }
    
    /// Records that history rows were inserted into the scrollback.
    pub fn note_history_rows_inserted(&mut self, inserted_rows: u16) {
        self.visible_history_rows = self
            .visible_history_rows
            .saturating_add(inserted_rows)
            .min(self.viewport_area.top());
    }
    
    /// Updates the viewport area (e.g., after scrolling).
    pub fn set_viewport_area(&mut self, area: Rect) {
        self.viewport_area = area;
        self.visible_history_rows = self.visible_history_rows.min(area.top());
    }
    
    /// Clears the viewport only.
    pub fn clear(&mut self) -> Result<()> {
        self.terminal.clear()?;
        Ok(())
    }
    
    /// Clears the scrollback only.
    pub fn clear_scrollback(&mut self) -> Result<()> {
        let mut stdout = io::stdout();
        execute!(stdout, terminal::Clear(ClearType::Purge))?;
        Ok(())
    }
    
    /// Draws the UI using the viewport area as the render region.
    pub fn draw<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut ratatui::Frame),
    {
        let mut stdout = io::stdout();
        queue!(stdout, MoveTo(0, self.viewport_area.top()))?;
        stdout.flush()?;
        
        self.terminal.draw(f)?;
        Ok(())
    }
    
    /// Restores the terminal to its original state.
    pub fn restore(&mut self) -> Result<()> {
        disable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, DisableMouseCapture)?;
        
        let bottom_y = self.viewport_area.bottom().saturating_sub(1);
        execute!(stdout, MoveTo(0, bottom_y))?;
        
        Ok(())
    }
}

impl Drop for CustomTerminal {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}
