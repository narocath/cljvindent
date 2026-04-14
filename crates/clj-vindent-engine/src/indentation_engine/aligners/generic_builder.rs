use crate::indentation_engine::model::Pair;

use crate::indentation_engine::helpers::{absolute_col_in_slice,
                                         line_start_byte,
                                         shift_multiline_block};
use tracing::debug;

pub fn build_aligned_string(src: &str, pairs: &[Pair], base_col: usize) -> String {
    if pairs.is_empty() {
        return src.to_string();
    }

    //let max_lhs_width = pairs.iter().map(|p| p.lh_width).max().unwrap_or(0);
    let max_lhs_width = pairs
        .iter()
        .filter(|p| !p.rh_string.is_empty())
        .map(|p| p.lh_width)
        .max()
        .unwrap_or(0);
    let target_lhs_col = base_col + 2;

    debug!(
        pairs = pairs.len(),
        max_lhs_width,
        target_lhs_col,
        "build aligned string"
    );

    let mut out = String::new();
    let mut last = 0;
    let mut prev_line_start: Option<usize> = None;

    for (i, pair) in pairs.iter().enumerate() {
        let line_start = line_start_byte(src, pair.lh_start_byte);

        if pair.lh_start_byte < last || line_start < last {
            debug!("builder bail-out: overlapping/reversed ranges");
            return src.to_string();
        }

        if prev_line_start == Some(line_start) {
            debug!("builder bail-out: multiple pairs on same line");
            return src.to_string();
        }

        if i == 0 {
            let prefix = src[last..pair.lh_start_byte]
                .trim_end_matches(char::is_whitespace);

            out.push_str(prefix);
            out.push('\n');
            out.push_str(&" ".repeat(target_lhs_col));
        } else {
            out.push_str(&src[last..line_start]);
            out.push_str(&" ".repeat(target_lhs_col));
        }

        let old_lhs_col = absolute_col_in_slice(src, base_col, pair.lh_start_byte);
        let adjusted_lhs = if pair.rh_string.is_empty() && pair.lh_string.contains('\n') {
            shift_multiline_block(
                &pair.lh_string,
                target_lhs_col as isize - old_lhs_col as isize,
            )
        } else {
            pair.lh_string.clone()
        };

        out.push_str(&adjusted_lhs);

        if !pair.rh_string.is_empty() {
            let spaces = (max_lhs_width - pair.lh_width) + 1;
            out.push_str(&" ".repeat(spaces));

            let old_rhs_col = absolute_col_in_slice(src, base_col, pair.rh_start_byte);
            let new_rhs_col = target_lhs_col + pair.lh_width + spaces;

            let adjusted_rhs = shift_multiline_block(
                &pair.rh_string,
                new_rhs_col as isize - old_rhs_col as isize,
            );

            out.push_str(&adjusted_rhs);
            last = pair.rh_end_byte;
        } else {
            debug!("lhs-only row");
            last = pair.lh_end_byte;
        }
        
        prev_line_start = Some(line_start);
    }

    out.push_str(&src[last..]);
    debug!("finished building aligned string");
    out
}
