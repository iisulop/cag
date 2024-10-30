use crate::error::Error;
use tracing::trace;

pub fn decrement(scroll: usize, count: usize) -> usize {
    scroll.checked_sub(count).unwrap_or_default()
}

pub fn increment(scroll: usize, count: usize, max_val: usize, vertical_size: u16) -> usize {
    if let Some(pos) = scroll.checked_add(count) {
        if pos > (max_val - usize::from(vertical_size)) {
            max_val - usize::from(vertical_size)
        } else {
            pos
        }
    } else {
        usize::MAX
    }
}

pub fn get_lines(
    log_lines: &[String],
    position: usize,
    vertical_size: u16,
) -> Result<&[String], Error> {
    trace!("Getting screenful of lines");
    let lines = if log_lines.len() > (position + usize::from(vertical_size)) {
        log_lines.get(position..(position + usize::from(vertical_size)))
    } else {
        log_lines.get(position..(log_lines.len() - 1))
    };
    lines.ok_or(Error::GetLines)
}
