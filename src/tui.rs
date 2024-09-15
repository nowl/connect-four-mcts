use std::{
    io::{self, stdout, Stdout},
    panic::{set_hook, take_hook},
};

use chrono::{DateTime, TimeDelta, Utc};
use ratatui::{
    crossterm::{
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    prelude::*,
    Terminal,
};

pub type Tui = Terminal<CrosstermBackend<Stdout>>;

pub fn init_panic_hook() {
    let original_hook = take_hook();
    set_hook(Box::new(move |panic_info| {
        // intentionally ignore errors here since we're already in a panic
        let _ = restore();
        original_hook(panic_info);
    }));
}

pub fn init() -> io::Result<Tui> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

pub fn restore() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;
    Ok(())
}

#[derive(Debug)]
pub struct SpinnerState {
    next_draw: DateTime<Utc>,
    pos: u16,
    width: u16,
}

impl SpinnerState {
    pub fn new(width: u16) -> Self {
        Self {
            next_draw: Utc::now(),
            pos: 0,
            width,
        }
    }
}

#[derive(Default)]
pub struct Spinner {}

impl StatefulWidget for Spinner {
    type State = SpinnerState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let width = state.width.min(area.width);

        let max_count = (area.width - width) * 2 + 1;

        if Utc::now() >= state.next_draw {
            state.next_draw += TimeDelta::milliseconds(50);

            state.pos += 1;

            if state.pos >= max_count {
                state.pos = 0;
            }
        }

        let span = Span::default().content(" ").on_cyan();
        let offset = if state.pos > max_count / 2 {
            max_count - state.pos - 1
        } else {
            state.pos
        };
        let start = offset;
        let end = offset + width;
        for x in start..end {
            buf.set_span(x, area.y, &span, width);
        }
    }
}
