use crate::search::SearchState;
use crate::{app::State, error::Error};
use aho_corasick::AhoCorasick;
use ratatui::style::{Style, Stylize as _};
use ratatui::text::{Line, Span};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};
use tracing::{debug, trace};
use tui_input::Input;

/// Renders the pager UI.
///
/// This function is responsible for rendering the pager UI, including the
/// commit message,
/// git log, and search box if applicable. It handles different states such as
/// `Search` and `Pager`.
///
/// # Arguments
/// * `f` - A mutable reference to the `Frame` to render the UI.
/// * `state` - A reference to the current `State` of the application.
/// * `git_log` - A slice of strings representing the git log to display.
/// * `commit` - An optional slice of strings representing the commit message
///   to display.
/// * `vertical_size` - A mutable reference to a `u16` to store the vertical
///   size of the rendered UI.
/// * `hilights` - An optional string containing the search term to highlight
///   in the git log.
///
/// # Errors
/// This function can return errors in the following cases:
/// * If there is an error building the Aho-Corasick automaton (`aho_corasick::Error`).
/// * If there is an error rendering the widgets (`tui::Error`).
pub fn pager(
    f: &mut Frame,
    state: &State,
    git_log: &[String],
    commit: Option<&[String]>,
    vertical_size: &mut u16,
    hilights: Option<String>,
) -> Result<(), Error> {
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
        .split(f.area());

    let commit_paragraph = Paragraph::new(commit.unwrap_or_default()).block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_type(BorderType::Double),
    );
    f.render_widget(commit_paragraph, chunks[0]);

    let paragraph = if let Some(hilights) = hilights {
        let hilight_style = Style::new().bold();
        let hilighted_log: Vec<_> = git_log
            .iter()
            .map(|line| {
                let ac = AhoCorasick::builder()
                    .ascii_case_insensitive(true)
                    .build([hilights.as_str()])?;
                let matches = ac.find_iter(line);
                let hilights: Vec<_> = matches.map(|m| (m.start(), m.end())).collect();
                debug!("Got hilights at: {hilights:?}");
                let line_hilighted = hilights
                    .windows(2)
                    .map(<&[(usize, usize); 2]>::try_from)
                    .collect::<Result<Vec<_>, _>>()?
                    .iter()
                    .fold(
                        vec![Span::from(
                            line[0..hilights.first().map_or(line.chars().count(), |m| m.0)]
                                .to_string(),
                        )],
                        |mut coll, [(start, end), (next_start, _next_end)]| {
                            let hilight = Span::styled(&line[*start..*end], hilight_style);
                            let text_between_hilights = Span::from(&line[*end..*next_start]);
                            coll.append(&mut vec![hilight, text_between_hilights]);
                            coll
                            // debug!("Adding: `{hilight}` and `{text_between_hilights}`");
                        },
                    );
                let line_hilighted = if let Some((last_start, last_end)) = hilights.last() {
                    let hilight = Span::styled(&line[*last_start..*last_end], hilight_style);
                    let rest_of_line = Span::from(&line[*last_end..]);
                    Line::from(
                        vec![line_hilighted, vec![hilight], vec![rest_of_line]]
                            .into_iter()
                            .flatten()
                            .collect::<Vec<_>>(),
                    )
                } else {
                    Line::from(line_hilighted)
                };
                Ok::<Line, Error>(line_hilighted)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Paragraph::new(hilighted_log)
    } else {
        Paragraph::new(git_log.join("\n"))
    };
    f.render_widget(paragraph, chunks[1]);
    *vertical_size = chunks[1].height;

    match state {
        State::Search(SearchState::GetInput { term }) => {
            draw_search_box(f, chunks[2], term);
        }
        State::Search(SearchState::Searching {
            term,
            position: _position,
        }) => {
            draw_search_box(f, chunks[2], term);
        }
        State::Pager => (),
    }
    Ok(())
}

fn draw_search_box(f: &mut Frame, area: Rect, input: &Input) {
    let search_box =
        Paragraph::new(input.value()).block(Block::default().borders(Borders::ALL).title("Search"));
    f.render_widget(search_box, area);
}
