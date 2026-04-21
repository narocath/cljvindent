use tree_sitter::Node;
use tracing::debug;

use crate::indentation_engine::alignable::Alignable;
use crate::indentation_engine::model::Row;
use crate::indentation_engine::helpers::{absolute_col_in_slice,                                         
                                         line_start_byte,
                                         non_comment_children,
                                         node_text,
                                         shift_multiline_block};
use crate::indentation_engine::model::{AlignKind, Extracted};

pub struct VecLikeAligner;
pub fn extract_vector_layout(nd: Node, src: &str) -> Option<(usize, Vec<Row>)> {
    if nd.kind() != "vec_lit" {
        return None;
    }

    let items = non_comment_children(nd);
    if items.is_empty() || nd.start_position().row == nd.end_position().row {
        debug!("Skip extract vec-like, no child items");
        return None;
    }

    let first_row = items[0].start_position().row;

    let body_start = items.iter().position(|n| n.start_position().row > first_row)?;
    let anchor_byte = items[body_start].start_byte();

    let rows = items[body_start..]
        .iter()
        .map(|n| Row {
            text: node_text(*n, src).to_string(),
            start_byte: n.start_byte(),
            end_byte: n.end_byte(),
        })
        .collect::<Vec<Row>>();
    
    debug!(rows = rows.len(), "finished extract vec rows");
    
    Some((anchor_byte, rows))
}

pub fn build_aligned_vector_string(
    src: &str,
    _anchor_byte: usize,
    steps: &[Row],
    base_col: usize,
) -> String {
    if steps.is_empty() {
        return src.to_string();
    }

    //let target_col = absolute_col_in_slice(src, base_col, anchor_byte);
    let target_col = base_col + 1;

    let mut out = String::new();
    let mut last = 0;
    let mut prev_line_start: Option<usize> = None;

    for (i, step) in steps.iter().enumerate() {
        let line_start = line_start_byte(src, step.start_byte);

        if step.start_byte < last || line_start < last {
            return src.to_string();
        }

        if let Some(prev) = prev_line_start {
            if prev == line_start {
                return src.to_string();
            }
        }

        let old_step_col = absolute_col_in_slice(src, base_col, step.start_byte);
        let adjusted_step = shift_multiline_block(
            &step.text,
            target_col as isize - old_step_col as isize,
        );
        if i == 0 {
            out.push_str(&src[last..line_start]);
            out.push_str(&" ".repeat(target_col));
            out.push_str(&adjusted_step);
        } else {
            out.push('\n');
            out.push_str(&" ".repeat(target_col));
            out.push_str(&adjusted_step);
        }
        
        last = step.end_byte;
        prev_line_start = Some(line_start);
    }

    out.push_str(&src[last..]);
    debug!(steps = steps.len(), "finished build vec-like rows");
    out
}

impl Alignable for VecLikeAligner {
    fn kind(&self) -> AlignKind {
        AlignKind::VecLike
    }

    fn matches(&self, node: Node, _src: &str) -> bool {
        node.kind() == "vec_lit"
    }

    fn extract(&self, node: Node, src: &str) -> Option<Extracted> {
        extract_vector_layout(node, src)
            .map(|(anchor_byte, rows)| Extracted::Rows { anchor_byte, rows })
    }

    fn build(&self, src: &str, extracted: Extracted, base_col: usize) -> String {
        match extracted {
            Extracted::Rows { anchor_byte, rows } => {
                build_aligned_vector_string(src, anchor_byte, &rows, base_col)
            }
            _ => src.to_string(),
        }
    }
}
