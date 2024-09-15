use std::{
    collections::{HashSet, VecDeque},
    io::{self},
};
mod connect_four;
mod tui;

use connect_four::{CFGameState, CFMove, Position};
use ratatui::{
    crossterm::event::KeyEvent,
    style::{Style, Stylize},
    text::Line,
    widgets::{block::Title, Widget},
    Frame,
};
use tui::{Spinner, SpinnerState};
use yamcts::{rng::DefaultRng, BestResultHandle, GameState};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum AppState {
    ColumnSelection,
    AiThinking,
    GameOver,
}

struct App<'a> {
    column_selection: u8,
    exit: bool,
    game: CFGameState,
    messages: VecDeque<Line<'a>>,
    app_state: AppState,
    best_move: Option<BestResultHandle<CFGameState>>,

    spinner_state: SpinnerState,
}

impl<'a> Widget for &mut App<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        use ratatui::layout::*;
        use ratatui::prelude::*;
        use ratatui::widgets::*;

        let horiz_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(2 * 7 + 1), Constraint::Min(0)])
            .margin(1)
            .spacing(2)
            .split(area);

        let board_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(6 + 2), Constraint::Min(0)])
            .split(horiz_layout[0]);

        let title = Title::from(" Board ".bold());

        let (ix, iy) = (board_layout[0].x, board_layout[0].y);

        for x in 0..6 {
            for y in 0..6 {
                buf.set_span(
                    ix + x * 2 + 2,
                    iy + y + 1,
                    &Span::styled("|", Style::default().light_blue()),
                    1,
                );
            }
        }

        Block::bordered()
            .title(title.alignment(Alignment::Center))
            .border_set(symbols::border::PLAIN)
            .render(board_layout[0], buf);

        for x in 0..7 {
            for y in 0..6 {
                use connect_four::Position::*;
                let bg = Color::from_hsl(0.0, 0.0, 25.0);
                let span = match self.game.pos(x, y) {
                    Red => Span::from("O").style(Style::default().light_blue().bg(bg)),
                    Black => Span::from("X").style(Style::default().light_red().bg(bg)),
                    Empty => {
                        Span::from(symbols::line::HORIZONTAL).style(Style::default().gray().bg(bg))
                    }
                };

                buf.set_span(ix + 1 + (x * 2) as u16, iy + 1 + y as u16, &span, 1);
            }
        }

        // determine last empty row
        let max_row = {
            let mut m = 5;
            for y in 1..6 {
                if self.game.pos(self.column_selection as usize, y) != connect_four::Position::Empty
                {
                    m = y - 1;
                    break;
                }
            }
            m as u16
        };

        for y in 0..=max_row {
            let style = if y == max_row {
                Style::default().black().on_white()
            } else {
                Style::default().black().on_yellow()
            };

            buf.set_line(
                ix + 1 + 2 * self.column_selection as u16,
                iy + 1 + y,
                &Line::from(" ").style(style),
                1,
            );
        }

        let title = Title::from(" Messages ".bold());

        let messages_area = Block::new()
            .borders(Borders::TOP)
            .title(title.alignment(Alignment::Center))
            .border_set(symbols::border::PLAIN);

        let messages_inner_area = messages_area.inner(horiz_layout[1]);
        let msgs = self
            .messages
            .iter()
            .take(10)
            .rev()
            .cloned()
            .collect::<Vec<_>>();
        Paragraph::new(msgs)
            .wrap(Wrap { trim: true })
            .render(messages_inner_area, buf);
        messages_area.render(horiz_layout[1], buf);

        let selection_text_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Max(1), Constraint::Min(0)])
            .split(board_layout[1]);

        Text::from(format!("column {}", self.column_selection + 1))
            .style(Style::default().green())
            .centered()
            .render(selection_text_layout[1], buf);

        if self.app_state == AppState::AiThinking {
            let mut area = area;
            area.width = 25;

            Spinner::default().render(area, buf, &mut self.spinner_state);
        }
    }
}

impl<'a> App<'a> {
    fn new(game: CFGameState) -> Self {
        let mut messages = VecDeque::new();

        let msg_play = "Use the arrow keys to choose where to play. Then press enter or spacebar.";
        let line = Line::from(msg_play).style(Style::default().light_green());
        messages.push_front(line);

        let msg = "Press Escape key at any time to exit.";
        let line = Line::from(msg).style(Style::default().red());
        messages.push_front(line);

        Self {
            column_selection: 0,
            exit: false,
            game,
            messages,
            best_move: None,
            app_state: AppState::ColumnSelection,
            spinner_state: SpinnerState::new(15),
        }
    }

    fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| {
                self.render_frame(frame);
            })?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn render_frame(&mut self, frame: &mut Frame) {
        let area = frame.size();
        frame.render_widget(self, area);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        use ratatui::crossterm::event::*;
        if poll(std::time::Duration::from_millis(16))? {
            match read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    let state = self.app_state;
                    self.handle_key_event(key, state)
                }
                _ => {}
            }
        }

        // check if done processing
        let best_move_handle = self.best_move.take();
        if best_move_handle.is_some() {
            let mut best_move_handle = best_move_handle.unwrap();
            if best_move_handle.is_finished() {
                let result = best_move_handle.join();

                let line = Line::from(format!(
                    "AI plays to column {} after thinking for {} moves.",
                    result.best_move.col + 1,
                    result.iterations
                ))
                .style(Style::default().light_red());
                self.messages.push_front(line);

                self.game = self.game.apply_move(result.best_move);

                if let Some(win) = self.game.is_terminal_state() {
                    if win == Position::Black {
                        let line = Line::from("AI Wins!").style(Style::default().light_red());
                        self.messages.push_front(line);
                    } else {
                        let line = Line::from("Tie").style(Style::default().light_blue());
                        self.messages.push_front(line);
                    }

                    self.app_state = AppState::GameOver;
                } else {
                    self.app_state = AppState::ColumnSelection;
                }
            } else {
                self.best_move = Some(best_move_handle);
            }
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent, state: AppState) {
        use ratatui::crossterm::event::KeyCode::*;
        match key.code {
            Char('q') | Esc => {
                self.exit = true;
                return;
            }
            _ => {}
        };

        if state == AppState::ColumnSelection {
            match key.code {
                Left => self.move_left(),
                Right => self.move_right(),
                Enter | Char(' ') => {
                    let player_move = CFMove {
                        color: Position::Red,
                        col: self.column_selection as usize,
                    };
                    self.game = self.game.apply_move(player_move);

                    let line =
                        Line::from(format!("Playing to column {}", self.column_selection + 1))
                            .style(Style::default().light_blue());

                    self.messages.push_front(line);

                    if let Some(win) = self.game.is_terminal_state() {
                        if win == Position::Red {
                            let line = Line::from("You win!").style(Style::default().light_blue());
                            self.messages.push_front(line);
                        } else {
                            let line = Line::from("Tie").style(Style::default().light_blue());
                            self.messages.push_front(line);
                        }

                        self.app_state = AppState::GameOver;
                    } else {
                        let mcts = yamcts::MCTS::<DefaultRng>::default();

                        self.best_move =
                            Some(mcts.run_with_duration(
                                self.game.clone(),
                                chrono::TimeDelta::seconds(1),
                            ));

                        self.maybe_move_column_selection();

                        self.app_state = AppState::AiThinking;
                        self.spinner_state = SpinnerState::new(15);
                    }
                }
                _ => {}
            }
        }
    }

    fn maybe_move_column_selection(&mut self) {
        let set = self
            .game
            .all_moves()
            .iter()
            .map(|m| m.col)
            .collect::<HashSet<usize>>();

        if !set.contains(&(self.column_selection as usize)) {
            self.column_selection = *set.iter().next().unwrap() as u8;
        }
    }

    fn move_left(&mut self) {
        let set = self
            .game
            .all_moves()
            .iter()
            .map(|m| m.col as i32)
            .collect::<HashSet<i32>>();

        // loop down
        let mut selection = self.column_selection as i32;
        loop {
            selection -= 1;
            if selection < 0 {
                selection = self.column_selection as i32;
                break;
            }

            if set.contains(&selection) {
                break;
            }
        }
        self.column_selection = selection as u8;
    }

    fn move_right(&mut self) {
        let set = self
            .game
            .all_moves()
            .iter()
            .map(|m| m.col as i32)
            .collect::<HashSet<i32>>();

        // loop up
        let mut selection = self.column_selection as i32;
        loop {
            selection += 1;
            if selection > 6 {
                selection = self.column_selection as i32;
                break;
            }

            if set.contains(&selection) {
                break;
            }
        }
        self.column_selection = selection as u8;
    }
}

fn main() -> io::Result<()> {
    env_logger::init();

    tui::init_panic_hook();
    let mut terminal = tui::init()?;

    App::new(CFGameState::new(Position::Red, Position::Black)).run(&mut terminal)?;

    tui::restore()?;

    Ok(())
}
