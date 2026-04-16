use tree_sitter::Node;
use tracing::debug;
use crate::indentation_engine::model::Pair;
use crate::indentation_engine::alignable::Alignable;
use crate::indentation_engine::helpers::{absolute_col_in_slice,                                         
                                         line_start_byte,
                                         node_text,
                                         non_comment_children,
                                         shift_multiline_block};
use crate::indentation_engine::model::{AlignKind, Extracted};

pub struct MapLikeAligner;

pub fn extract_map_pairs(nd: Node, src: &str) -> Option<Vec<Pair>> {
    if nd.kind() != "map_lit" {
        return None;
    }
    // if nd.start_position().row == nd.end_position().row {
    //     return None;
    // }

    let children = non_comment_children(nd);

    if children.is_empty() || children.len() % 2 != 0 {
        return None;
    }

    if children.len() < 2 {
        debug!("Bail out map like extraction {}: no pair", children.len());
        return None;
    }

    let mut prev_line_start: Option<usize> = None;
    let start_of_first_lh = children.first()?.start_position().column;
    let mut pairs = Vec::new();

    debug!("Extracting map children");
    
    for chunk in children.chunks(2) {
        let lhs = chunk[0];
        let rhs = chunk[1];
        let key = node_text(lhs, src);

        if matches!(key, ":keys" | ":strs" | ":syms" | ":as" | ":or") {
            debug!("Bail out map is destructing form");
            return None;
        }

        let line_start = line_start_byte(src, lhs.start_byte());
        if let Some(prev) = prev_line_start {
            if prev == line_start {
                return None;
            }
        }
        prev_line_start = Some(line_start);

        let lh_string = node_text(lhs, src).to_string();
        pairs.push(Pair {
            lh_width: lh_string.len(),
            lh_start_col: start_of_first_lh,
            lh_string,
            rh_string: node_text(rhs, src).to_string(),
            lh_start_byte: lhs.start_byte(),
            lh_end_byte: lhs.end_byte(),
            rh_start_byte: rhs.start_byte(),
            rh_end_byte: rhs.end_byte(),
        });
    }
    debug!(pairs = pairs.len(), "finished map-like extraction");
    
    Some(pairs)
}

pub fn build_aligned_map_string(src: &str, pairs: &[Pair], base_col: usize) -> String {
    if pairs.is_empty() {
        return src.to_string();
    }

    let max_lhs_width = pairs.iter().map(|p| p.lh_width).max().unwrap_or(0);
    let target_lhs_col = base_col + 1;

    debug!(
        pairs = pairs.len(),
        max_lhs_width,
        target_lhs_col,
        "build map"
    );

    let mut out = String::new();
    let mut last = 0;
    let mut prev_line_start: Option<usize> = None;

    for (i, pair) in pairs.iter().enumerate() {
        let line_start = line_start_byte(src, pair.lh_start_byte);

        if pair.lh_start_byte < last || line_start < last {
            return src.to_string();
        }

        if prev_line_start == Some(line_start) {
            return src.to_string();
        }

        if i == 0 {
            let prefix = &src[last..pair.lh_start_byte];

            if let Some(open_brace_idx) = prefix.rfind('{') {
                out.push_str(&prefix[..=open_brace_idx]);
            } else {
                out.push('{');
            }
        } else {
            out.push('\n');
            out.push_str(&" ".repeat(target_lhs_col));
        }

        out.push_str(pair.lh_string.trim());

        let spaces = (max_lhs_width - pair.lh_width) + 1;
        out.push_str(&" ".repeat(spaces));

        let raw_rhs = pair.rh_string.as_str();
        let rhs = raw_rhs.trim_start_matches(char::is_whitespace);

        let trimmed_prefix_len = raw_rhs.len() - rhs.len();
        let trimmed_rhs_start_byte = pair.rh_start_byte + trimmed_prefix_len;

        let old_rhs_col = absolute_col_in_slice(src, base_col, trimmed_rhs_start_byte);
        let new_rhs_col = target_lhs_col + pair.lh_width + spaces;

        let adjusted_rhs = shift_multiline_block(
            rhs,
            new_rhs_col as isize - old_rhs_col as isize,
        );

        out.push_str(&adjusted_rhs);

        last = pair.rh_end_byte;
        prev_line_start = Some(line_start);
    }

    out.push_str(&src[last..]);
    debug!("finished map build");
    out
}

impl Alignable for MapLikeAligner {
    fn kind(&self) -> AlignKind {
        AlignKind::MapLike
    }

    fn matches(&self, node: Node, _src: &str) -> bool {
        node.kind() == "map_lit"
    }

    fn extract(&self, node: Node, src: &str) -> Option<Extracted> {
        extract_map_pairs(node, src).map(Extracted::Pairs)
    }

    fn build(&self, src: &str, extracted: Extracted, base_col: usize) -> String {
        match extracted {
            Extracted::Pairs(pairs) => build_aligned_map_string(src, &pairs, base_col),
            _ => src.to_string(),
        }
    }
}
