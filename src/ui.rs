use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};
use tracing::trace;
use tui_input::Input;
use crate::app::State;
use crate::search::SearchState;

pub fn pager<B: Backend>(
    f: &mut Frame<B>,
    state: &State,
    git_log: &[String],
    commit: Option<&[String]>,
    vertical_size: &mut u16,
) {
    trace!("Rendering screen");
    let commit_len = commit.map_or(0, |commit| commit.iter().len() + 1);
    let commit = commit.map(|commit| commit.join("\n"));

    let layout = match state {
        State::Search { .. } => vec![
            #[allow(clippy::cast_possible_truncation)]
            Constraint::Max(std::cmp::min(7, commit_len as u16)),
            Constraint::Min(8),
            Constraint::Max(3),
        ],
        State::Pager => vec![
            #[allow(clippy::cast_possible_truncation)]
            Constraint::Max(std::cmp::min(7, commit_len as u16)),
            Constraint::Min(8),
        ],
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(layout)
        .margin(1)
        .split(f.size());

    let commit_paragraph = Paragraph::new(commit.unwrap_or_default()).block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_type(BorderType::Double),
    );
    f.render_widget(commit_paragraph, chunks[0]);

    let paragraph = Paragraph::new(git_log.join("\n"));
    f.render_widget(paragraph, chunks[1]);
    *vertical_size = chunks[1].height;

    match state {
        State::Search(SearchState::GetInput { term }) => {
            draw_search_box(f, chunks[2], term);
        }
        State::Search(SearchState::Searching { term, position: _position }) => {
            draw_search_box(f, chunks[2], term);
        }
        State::Pager => (),
    }
}

fn draw_search_box<B: Backend>(f: &mut Frame<B>, area: Rect, input: &Input) {
    let search_box =
        Paragraph::new(input.value()).block(Block::default().borders(Borders::ALL).title("Search"));
    f.render_widget(search_box, area);
}
