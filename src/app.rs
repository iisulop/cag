use crate::context_finder::{ContextFinder, InputType};
use crate::error::Error;
use crate::input::stream_input;
use crate::search::{search, SearchDirection, SearchState};
use crate::ui::pager;
use crate::utils::{decrement_scroll_position, get_lines, increment_scroll_position};
use crossterm::event::{read, Event, KeyCode, KeyEvent};
use ratatui::{backend::Backend, Terminal};
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::Duration;
use tracing::{trace, warn};
use tui_input::backend::crossterm::EventHandler as _;

const INPUT_STREAM_TIMEOUT: u64 = 1000;

pub enum State {
    Exit,
    Pager,
    Search(SearchState),
}

/// Runs the application.
///
/// This function initializes the terminal, sets up the input stream, and enters a loop to handle
/// user input and update the terminal display accordingly.
///
/// # Errors
/// This function can return errors in the following cases:
/// * If there is an error initializing the terminal size.
/// * If there is an error receiving input from the input stream.
/// * If there is an error drawing to the terminal.
/// * If there is an error reading user input.
pub fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> Result<(), Error> {
    let mut position: usize = 0;
    let mut vertical_size = terminal.size()?.height;
    let (rx, _thread_handle) = stream_input(usize::from(vertical_size) * 4);
    let mut all_lines = rx.recv_timeout(Duration::from_millis(INPUT_STREAM_TIMEOUT))??;
    let cf = ContextFinder::new(&InputType::Git)?;
    let mut state = State::Pager;

    loop {
        all_lines = handle_new_lines(&rx, all_lines)?;
        let context = cf.get_context(&all_lines[..], position);
        let lines = get_lines(&all_lines[..], position, terminal.size()?.height)?;

        let hilights = get_hilights(&state);
        terminal
            .try_draw(|frame| pager(frame, &state, lines, context, &mut vertical_size, hilights))?;

        let event = read()?;
        if let Event::Key(key) = event {
            handle_key_event(key, &mut state, &mut position, &all_lines, vertical_size)?;
            if let State::Exit = state {
                return Ok(());
            }
        }
    }
}

fn handle_new_lines(
    rx: &Receiver<Result<Vec<String>, Error>>,
    mut all_lines: Vec<String>,
) -> Result<Vec<String>, Error> {
    match rx.try_recv() {
        Ok(maybe_new_lines) => {
            trace!("Got more lines");
            all_lines.extend(maybe_new_lines?);
            Ok(all_lines)
        }
        Err(TryRecvError::Disconnected) => Ok(all_lines),
        Err(e) => {
            warn!("Got error receiving new lines: {e}");
            Ok(all_lines)
        }
    }
}

fn get_hilights(state: &State) -> Option<String> {
    match state {
        State::Search(
            SearchState::GetInput { ref term } | SearchState::Searching { ref term, .. },
        ) => Some(term.to_string()),
        _ => None,
    }
}

fn handle_key_event(
    key: KeyEvent,
    state: &mut State,
    position: &mut usize,
    all_lines: &[String],
    vertical_size: u16,
) -> Result<(), Error> {
    match state {
        State::Pager => match key.code {
            KeyCode::Char('q') | KeyCode::Esc => *state = State::Exit,
            KeyCode::Char('j') | KeyCode::Down => {
                *position = increment_scroll_position(*position, 1, all_lines.len(), vertical_size);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                *position = decrement_scroll_position(*position, 1);
            }
            KeyCode::PageDown => {
                *position = increment_scroll_position(
                    *position,
                    usize::from(vertical_size),
                    all_lines.len(),
                    vertical_size,
                );
            }
            KeyCode::PageUp => {
                *position = decrement_scroll_position(*position, usize::from(vertical_size));
            }
            KeyCode::Char('/') => {
                *state = State::Search(SearchState::GetInput { term: "".into() });
            }
            _ => (),
        },
        State::Search(SearchState::GetInput { ref mut term }) => match key.code {
            KeyCode::Esc => *state = State::Pager,
            KeyCode::Enter => {
                *state = State::Search(SearchState::Searching {
                    term: term.clone(),
                    position: *position,
                });
            }
            _ => {
                *position = if let Some(new_position) =
                    search(term, *position, all_lines, &SearchDirection::Forward)?
                {
                    new_position
                } else {
                    *position
                };
                term.handle_event(&Event::Key(key));
            }
        },
        State::Search(SearchState::Searching {
            ref mut term,
            position: _,
        }) => match key.code {
            KeyCode::Esc | KeyCode::Char('q') => *state = State::Pager,
            KeyCode::Char('n') => {
                *position = if let Some(new_position) =
                    search(term, *position + 1, all_lines, &SearchDirection::Forward)?
                {
                    new_position
                } else {
                    *position
                };
            }
            KeyCode::Char('N') => {
                *position = if let Some(new_position) =
                    search(term, *position, all_lines, &SearchDirection::Backwards)?
                {
                    new_position
                } else {
                    *position
                };
            }
            KeyCode::Char('/') => {
                *state = State::Search(SearchState::GetInput { term: "".into() });
            }
            _ => (),
        },
        State::Exit => unreachable!(),
    }
    Ok(())
}
