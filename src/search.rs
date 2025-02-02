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

/// Searches for a term in the provided lines of text starting from a given position.
///
/// # Returns
/// * `Result<Option<usize>, Error>` - Returns an Ok result containing an
///   Option with the line number of the first match if found, otherwise None.
///   Returns an Error if the search fails.
///
/// # Errors
/// This function can return errors in the following cases:
/// * If there is an error building the Aho-Corasick automaton
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
    Ok(match_lines.first().copied())
}
