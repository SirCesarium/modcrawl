//! The borderless table primitive every listing renders through.

use anstream::println;
use anstyle::Style;

use super::style::{dim, paint};
use super::text;

/// One table cell: the plain text plus the style it is painted with, so the
/// text can be wrapped before it is painted.
pub(crate) struct Cell {
    text: String,
    style: Style,
}

/// A plain (unstyled) cell.
pub(crate) fn cell(text: &str) -> Cell {
    styled_cell(Style::new(), text)
}

/// A cell painted with `style`.
pub(crate) fn styled_cell(style: Style, text: &str) -> Cell {
    Cell {
        text: text.to_owned(),
        style,
    }
}

/// Print a borderless table: dimmed uppercase header, a rule, then one row per
/// entry.
///
/// Columns are shrunk to the terminal and long cells wrap, so a continuation
/// line starts under its own column instead of at the left margin. Without a
/// terminal (a pipe or a file) the columns keep their natural width and no
/// cell is wrapped.
pub(crate) fn print_table(headers: &[&str], rows: &[Vec<Cell>]) {
    if rows.is_empty() {
        return;
    }

    let floors: Vec<usize> = headers.iter().map(|h| text::width(h)).collect();
    let mut widths = floors.clone();

    for row in rows {
        for (index, cell) in row.iter().enumerate() {
            if let Some(w) = widths.get_mut(index) {
                *w = (*w).max(text::width(&cell.text));
            }
        }
    }

    if let Some(available) = text::terminal_width() {
        shrink(&mut widths, &floors, available);
    }

    let header_line: Vec<String> = headers
        .iter()
        .enumerate()
        .map(|(i, h)| paint(dim(), &text::pad(&h.to_uppercase(), widths[i])))
        .collect();

    println!("{}", header_line.join("  "));
    print_rule(&widths);

    for row in rows {
        let wrapped: Vec<Vec<String>> = row
            .iter()
            .enumerate()
            .map(|(i, cell)| text::wrap(&cell.text, widths[i]))
            .collect();
        let height = wrapped.iter().map(Vec::len).max().unwrap_or(1);

        for line in 0..height {
            let cells: Vec<String> = wrapped
                .iter()
                .enumerate()
                .map(|(i, lines)| {
                    let value = lines.get(line).map_or("", String::as_str);
                    paint(row[i].style, &text::pad(value, widths[i]))
                })
                .collect();

            println!("{}", cells.join("  "));
        }
    }
}

/// Shrink `widths` until the columns and their two-space gaps fit in
/// `available`, giving up space on the widest column first and never going
/// below `floors`. When every column is already at its floor the table keeps
/// its natural width rather than losing its header.
fn shrink(widths: &mut [usize], floors: &[usize], available: usize) {
    loop {
        let total = widths.iter().sum::<usize>() + 2 * widths.len().saturating_sub(1);
        let Some(excess) = total.checked_sub(available) else {
            return;
        };
        if excess == 0 {
            return;
        }

        let Some(index) = (0..widths.len())
            .filter(|&i| widths[i] > floors[i])
            .max_by_key(|&i| widths[i])
        else {
            return;
        };

        let room = widths[index] - floors[index];
        widths[index] -= room.min(excess);
    }
}

/// Dashed rule under the header, stopping where the header text stops: the
/// padding after the last column is not counted. Dashed so it can't be
/// confused with the solid rule `file_header` prints between JARs.
fn print_rule(widths: &[usize]) {
    let columns = widths
        .iter()
        .take(widths.len().saturating_sub(1))
        .map(|w| w + 2);
    let tail = widths.last().copied().unwrap_or(0);
    let total = columns.sum::<usize>() + tail;

    println!("{}", paint(dim(), &"╌".repeat(total)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shrink_takes_space_from_the_widest_column() {
        let floors = [5, 4, 5];
        let mut widths = [40, 10, 120];

        shrink(&mut widths, &floors, 100);

        assert_eq!(widths.iter().sum::<usize>() + 4, 100);
        assert_eq!(widths, [40, 10, 46]);
    }

    #[test]
    fn shrink_never_goes_below_the_floors() {
        let floors = [5, 4, 7];
        let mut widths = [6, 4, 8];

        shrink(&mut widths, &floors, 10);

        assert_eq!(widths, [5, 4, 7]);
    }

    #[test]
    fn shrink_leaves_a_table_that_fits_untouched() {
        let floors = [3, 3];
        let mut widths = [40, 10];

        shrink(&mut widths, &floors, 100);

        assert_eq!(widths, [40, 10]);
    }
}
