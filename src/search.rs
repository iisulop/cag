use crate::error::Error;
use aho_corasick::AhoCorasick;
use tui_input::Input;

#[derive(Debug, Eq, PartialEq)]
pub enum SearchDirection {
    Backwards,
    Forward,
}

pub enum SearchState {
    GetInput { term: Input },
    Searching { term: Input, position: usize },
}

pub fn search(
    term: &Input,
    position: usize,
    all_lines: &[String],
    direction: &SearchDirection,
) -> Result<Option<usize>, Error> {
    let ac = AhoCorasick::builder()
        .ascii_case_insensitive(true)
        .build([term.value()])?;
    let match_lines: Vec<usize> = match direction {
        SearchDirection::Backwards => all_lines
            .iter()
            .enumerate()
            .rev()
            .skip(all_lines.len() - position)
            .filter_map(|(line_num, line)| {
                if ac.find_iter(line).next().is_some() {
                    Some(line_num)
                } else {
                    None
                }
            })
            .collect(),
        SearchDirection::Forward => all_lines
            .iter()
            .enumerate()
            .skip(position)
            .filter_map(|(line_num, line)| {
                if ac.find_iter(line).next().is_some() {
                    Some(line_num)
                } else {
                    None
                }
            })
            .collect(),
    };
    Ok(match_lines.first().cloned())
}
