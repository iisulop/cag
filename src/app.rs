use crate::context_finder::{ContextFinder, InputType};
use crate::error::Error;
use crate::input::stream_input;
use crate::search::{search, SearchDirection, SearchState};
use crate::ui::pager;
use crate::utils::{decrement, get_lines, increment};
use crossterm::event::{read, Event, KeyCode};
use ratatui::{backend::Backend, Terminal};
use std::sync::mpsc::TryRecvError;
use std::time::Duration;
use tracing::{trace, warn};
use tui_input::backend::crossterm::EventHandler as _;

const INPUT_STREAM_TIMEOUT: u64 = 1000;

pub enum State {
    Pager,
    Search(SearchState),
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> Result<(), Error> {
    let mut position: usize = 0;
    let mut vertical_size = terminal.size()?.height;
    let (rx, _thread_handle) = stream_input(usize::from(vertical_size) * 4);
    let mut all_lines = rx.recv_timeout(Duration::from_millis(INPUT_STREAM_TIMEOUT))??;
    let cf = ContextFinder::new(&InputType::Git)?;
    let mut state = State::Pager;

    loop {
        all_lines = match rx.try_recv() {
            Ok(maybe_new_lines) => {
                trace!("Got more lines");
                all_lines.extend(maybe_new_lines?);
                all_lines
            }
            Err(TryRecvError::Disconnected) => all_lines,
            Err(e) => {
                warn!("Got error receiving new lines: {e}");
                all_lines
            }
        };
        // TODO: Position needs to be fixed somehow as lines are now displaying twice: in context
        // and in the pager
        let context = cf.get_context(&all_lines[..], position);
        let lines = get_lines(&all_lines[..], position, terminal.size()?.height)?;

        let hilights = match state {
            State::Search(SearchState::GetInput { ref term }) => Some(term.to_string()),
            State::Search(SearchState::Searching { ref term, .. }) => Some(term.to_string()),
            _ => None,
        };
        terminal.draw(|frame| {
            pager(frame, &state, lines, context, &mut vertical_size, hilights);
        })?;

        let event = read()?;
        if let Event::Key(key) = event {
            match state {
                State::Pager => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('j') | KeyCode::Down => {
                        position = increment(position, 1, all_lines.len(), vertical_size);
                    }
                    KeyCode::Char('k') | KeyCode::Up => position = decrement(position, 1),
                    KeyCode::PageDown => {
                        position = increment(
                            position,
                            usize::from(vertical_size),
                            all_lines.len(),
                            vertical_size,
                        );
                    }
                    KeyCode::PageUp => position = decrement(position, usize::from(vertical_size)),
                    KeyCode::Char('/') => {
                        state = State::Search(SearchState::GetInput { term: "".into() });
                    }
                    _ => (),
                },
                State::Search(SearchState::GetInput { ref mut term }) => match key.code {
                    KeyCode::Esc => state = State::Pager,
                    KeyCode::Enter => {
                        state = State::Search(SearchState::Searching {
                            term: term.clone(),
                            position,
                        });
                    }
                    _ => {
                        position = if let Some(new_position) =
                            search(term, position, &all_lines, &SearchDirection::Forward)?
                        {
                            new_position
                        } else {
                            position
                        };
                        term.handle_event(&event);
                    }
                },
                State::Search(SearchState::Searching {
                    ref mut term,
                    position: _position,
                }) => match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => state = State::Pager,
                    KeyCode::Char('n') => {
                        position = if let Some(new_position) =
                            search(term, position + 1, &all_lines, &SearchDirection::Forward)?
                        {
                            new_position
                        } else {
                            position
                        };
                    }
                    KeyCode::Char('N') => {
                        position = if let Some(new_position) =
                            search(term, position, &all_lines, &SearchDirection::Backwards)?
                        {
                            new_position
                        } else {
                            position
                        };
                    }
                    _ => (),
                },
            }
        }
    }
}
