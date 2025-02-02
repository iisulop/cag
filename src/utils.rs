use crate::error::Error;
use tracing::trace;

/// Decrements the scroll position by a specified count ensuring the result does
/// not underflow.
///
/// # Returns
/// * `usize` - The new scroll position, or 0 if the result would be negative.
#[must_use]
pub fn decrement_scroll_position(scroll: usize, count: usize) -> usize {
    scroll.checked_sub(count).unwrap_or_default()
}

/// Increments the scroll position by a specified count, ensuring it does not
/// exceed the available display size.
///
/// # Arguments
/// * `scroll` - The current scroll position.
/// * `count` - The amount to increment the scroll position by.
/// * `max_val` - The maximum allowable value for the scroll position.
/// * `vertical_size` - The vertical size of the display.
///
/// # Returns
///
/// * `usize` - The new scroll position, or `usize::MAX` if the result would overflow.
#[must_use]
pub fn increment_scroll_position(
    scroll: usize,
    count: usize,
    max_val: usize,
    vertical_size: u16,
) -> usize {
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

/// Retrieves a screenful of lines from the log starting at a specified position.
///
/// This function returns a slice of strings representing a screenful of lines
/// from the log, starting at the given position and extending for the vertical
/// size of the display.
///
/// # Errors
/// This function can return errors in the following cases:
/// * If the specified range is out of bounds of the log lines.
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
