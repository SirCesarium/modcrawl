//! Display width, padding, and wrapping for the human-readable renderers.

use std::io::{self, IsTerminal};
use std::mem;

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Terminal width in display columns.
///
/// `None` when stdout is redirected, so files and pipes receive whole lines
/// and keep the text the terminal would otherwise have wrapped for them.
pub(crate) fn terminal_width() -> Option<usize> {
    if !io::stdout().is_terminal() {
        return None;
    }

    terminal_size::terminal_size().map(|(terminal_size::Width(columns), _)| usize::from(columns))
}

/// Display width of the widest line in `text`.
pub(crate) fn width(text: &str) -> usize {
    text.lines().map(UnicodeWidthStr::width).max().unwrap_or(0)
}

/// `text` followed by enough spaces to reach `columns` display columns.
pub(crate) fn pad(text: &str, columns: usize) -> String {
    let mut padded = String::from(text);
    padded.push_str(&" ".repeat(columns.saturating_sub(width(text))));

    padded
}

/// Break `text` into lines of at most `columns` display columns.
///
/// Existing line breaks are kept. Within a line the break goes at a space when
/// one is available; a run without spaces — a method descriptor, a path — is
/// split between characters, so no line is ever left to wrap on its own.
pub(crate) fn wrap(text: &str, columns: usize) -> Vec<String> {
    if text.is_empty() {
        return vec![String::new()];
    }

    let columns = columns.max(1);
    let mut lines = Vec::new();

    for line in text.lines() {
        wrap_line(line, columns, &mut lines);
    }

    lines
}

/// Append the wrapped pieces of one line to `lines`.
fn wrap_line(line: &str, columns: usize, lines: &mut Vec<String>) {
    let mut current = String::new();
    let mut current_width = 0usize;
    // Byte index of the last space in `current`: the candidate break point.
    let mut break_at: Option<usize> = None;

    for ch in line.chars() {
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);

        if current_width + ch_width > columns {
            if ch.is_whitespace() {
                // The line is already full: end it here, and the space we are
                // standing on is the break, so it is not carried over.
                if !current.is_empty() {
                    lines.push(mem::take(&mut current));
                    current_width = 0;
                    break_at = None;
                }
                continue;
            }

            loop {
                match break_at {
                    Some(index) => {
                        lines.push(current[..index].to_string());
                        current.drain(..=index);
                        current_width = width(&current);
                        break_at = current.rfind(' ');
                    }
                    // Wider than a whole line: emit it as it is rather than
                    // loop without progress.
                    None if current.is_empty() => break,
                    None => {
                        lines.push(mem::take(&mut current));
                        current_width = 0;
                    }
                }

                if current_width + ch_width <= columns {
                    break;
                }
            }
        }

        current.push(ch);
        current_width += ch_width;

        if ch == ' ' {
            break_at = current.rfind(' ');
        }
    }

    lines.push(current);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_that_fits_stays_on_one_line() {
        assert_eq!(wrap("abc def", 20), ["abc def"]);
    }

    #[test]
    fn breaks_at_the_last_space_that_fits() {
        assert_eq!(wrap("aaa bbb ccc", 7), ["aaa bbb", "ccc"]);
    }

    #[test]
    fn splits_a_word_wider_than_the_line() {
        assert_eq!(wrap("abcdefgh", 3), ["abc", "def", "gh"]);
    }

    #[test]
    fn splits_a_descriptor_that_has_no_spaces() {
        let descriptor = "(Ljava/lang/invoke/MethodHandles$Lookup;Ljava/lang/String;)Ljava/lang/invoke/CallSite;";
        let wrapped = wrap(descriptor, 40);

        assert!(wrapped.iter().all(|line| width(line) <= 40));
        assert_eq!(wrapped.concat(), descriptor);
    }

    #[test]
    fn keeps_existing_line_breaks() {
        assert_eq!(wrap("first\n\nthird", 10), ["first", "", "third"]);
    }

    #[test]
    fn empty_text_is_one_empty_line() {
        assert_eq!(wrap("", 10), [""]);
    }

    #[test]
    fn a_single_wide_character_does_not_loop() {
        let wrapped = wrap("日本語", 1);

        assert_eq!(wrapped.len(), 3);
        assert_eq!(wrapped.concat(), "日本語");
    }

    #[test]
    fn a_narrow_terminal_still_wraps_at_spaces() {
        assert_eq!(wrap("aa bb", 2), ["aa", "bb"]);
    }
}
