use crate::{app::State, error::Error};
use crate::search::SearchState;
use aho_corasick::AhoCorasick;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};
use tracing::{debug, trace};
use tui_input::Input;

pub fn pager<B: Backend>(
    f: &mut Frame<B>,
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
        .split(f.size());

    let commit_paragraph = Paragraph::new(commit.unwrap_or_default()).block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_type(BorderType::Double),
    );
    f.render_widget(commit_paragraph, chunks[0]);

    let paragraph = if let Some(hilights) = hilights {
        let hilighted_log: Vec<_> = git_log
            .iter()
            .map(|line| {
                let ac = AhoCorasick::builder()
                    .ascii_case_insensitive(true)
                    .build([hilights.as_str()])
                    .unwrap();
                let matches = ac.find_iter(line);
                let hilights: Vec<_> = matches.map(|m| (m.start(), m.end())).collect();
                debug!("Got hilights at: {hilights:?}");
                let line_hilighted = hilights
                    .windows(2)
                    .map(<&[(usize, usize); 2]>::try_from)
                    .collect::<Result<Vec<_>, _>>()?
                    .iter()
                    .fold(
                        line[0..hilights
                            .first()
                            .map(|m| m.0)
                            .unwrap_or(line.chars().count())]
                            .to_string(),
                        |coll, [(start, end), (next_start, _next_end)]| {
                            let hilight = &line[*start..*end];
                            let text_between_hilights = &line[*end..*next_start];
                            debug!("Adding: `{hilight}` and `{text_between_hilights}`");
                            coll + hilight + text_between_hilights
                        },
                    );
                let line_hilighted = if let Some((last_start, last_end)) =  hilights.last() {
                    let hilight = &line[*last_start..*last_end];
                    let rest_of_line = &line[*last_end..];
                    line_hilighted + hilight + rest_of_line
                } else {
                    line_hilighted
                };
                Ok::<String, Error>(line_hilighted)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Paragraph::new(hilighted_log.as_slice().join("\n"))
        //Paragraph::new(git_log.join("\n"))
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

fn draw_search_box<B: Backend>(f: &mut Frame<B>, area: Rect, input: &Input) {
    let search_box =
        Paragraph::new(input.value()).block(Block::default().borders(Borders::ALL).title("Search"));
    f.render_widget(search_box, area);
}
