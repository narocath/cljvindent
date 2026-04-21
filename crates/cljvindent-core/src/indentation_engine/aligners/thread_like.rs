use tree_sitter::Node;
use tracing::debug;

use crate::indentation_engine::model::Row;
use crate::indentation_engine::alignable::Alignable;
use crate::indentation_engine::helpers::{absolute_col_in_slice,
                                         node_text,
                                         get_tree,
                                         get_root_node,
                                         line_start_byte,                                         
                                         non_comment_children,
                                         shift_multiline_block};
use crate::indentation_engine::model::{AlignKind, Extracted};

pub struct ThreadLikeAligner;

fn normalized_as_thread_header(src: &str) -> Option<String> {
    let tree = get_tree(src)?;
    let root = get_root_node(&tree)?;
    let form = root.named_child(0)?;

    let children = non_comment_children(form);
    if children.len() < 3 {
        return None;
    }

    let head = node_text(children[0], src);
    if head != "as->" {
        return None;
    }

    Some(format!(
        "({} {} {}",
        head,
        node_text(children[1], src).trim(),
        node_text(children[2], src).trim(),
    ))
}

pub fn extract_thread_layout(nd: Node, src: &str) -> Option<(usize, Vec<Row>)> {
    if nd.kind() != "list_lit" {
        return None;
    }

    let children = non_comment_children(nd);
    let head = node_text(*children.first()?, src);

    debug!(head = head, children = children.len(), "extract thread-like rows");
    
    if nd.start_position().row == nd.end_position().row {
        debug!("Skip thread-like: extarct: single-line-row");
        return None;
    }

    match head {
        "->" | "->>" | "some->" | "some->>" => {
            if children.len() < 3 {
                return None;
            }

            let anchor_byte = children[1].start_byte();

            let rows = children[2..]
                .iter()
                .map(|n| Row {
                    text: node_text(*n, src).to_string(),
                    start_byte: n.start_byte(),
                    end_byte: n.end_byte(),
                })
                .collect();

            Some((anchor_byte, rows))
        }

        "if" | "when" | "when-not" | "if-not" | "while"  => {
            if children.len() < 3 {
                return None;
            }

            let anchor_byte = nd.start_byte() + 2;

            let rows = children[2..]
                .iter()
                .map(|n| Row {
                    text: node_text(*n, src).to_string(),
                    start_byte: n.start_byte(),
                    end_byte: n.end_byte(),
                })
                .collect();

            Some((anchor_byte, rows))
        }

        "as->" => {
            if children.len() < 2 {
                return None;
            }

            let header_row = children[0].start_position().row;

            let body_start = children
                .iter()
                .position(|n| n.start_position().row > header_row)?;

            let anchor_byte = children[body_start].start_byte();

            let rows = children[body_start..]
                .iter()
                .map(|n| Row {
                    text: node_text(*n, src).to_string(),
                    start_byte: n.start_byte(),
                    end_byte: n.end_byte(),
                })
                .collect();

            Some((anchor_byte, rows))
        }

        _ => None,
    }
}

pub fn build_aligned_thread_string(
    src: &str,
    anchor_byte: usize,
    steps: &[Row],
    base_col: usize,
) -> String {
    if steps.is_empty() {
        return src.to_string();
    }

    let is_as_thread = src.trim_start().starts_with("(as->");

    let target_col = if is_as_thread {
        base_col + 2
    } else {
        absolute_col_in_slice(src, base_col, anchor_byte)
    };

    debug!(target_col = target_col, steps = steps.len(), "extract thread-like rows");

    let mut out = String::new();
    let mut last = 0;
    let mut prev_line_start: Option<usize> = None;

    for (i, step) in steps.iter().enumerate() {
        let line_start = line_start_byte(src, step.start_byte);

        if step.start_byte < last || line_start < last {
            return src.to_string();
        }

        if prev_line_start == Some(line_start) {
            return src.to_string();
        }

        let old_step_col = absolute_col_in_slice(src, base_col, step.start_byte);
        let adjusted_step = shift_multiline_block(
            &step.text,
            target_col as isize - old_step_col as isize,
        );

        if i == 0 {
            if is_as_thread {
                let header = normalized_as_thread_header(src).unwrap_or_else(|| {
                    src[last..step.start_byte]
                        .trim_end_matches(char::is_whitespace)
                        .to_string()
                });

                out.push_str(&header);
                out.push('\n');
                out.push_str(&" ".repeat(target_col));
            } else {
                let prefix = src[last..step.start_byte]
                    .trim_end_matches(char::is_whitespace);

                out.push_str(prefix);
                out.push('\n');
                out.push_str(&" ".repeat(target_col));
            }
        } else {
            out.push_str(&src[last..line_start]);
            out.push_str(&" ".repeat(target_col));
        }

        out.push_str(&adjusted_step);

        last = step.end_byte;
        prev_line_start = Some(line_start);
    }

    out.push_str(&src[last..]);
    debug!("Finish build thread-like rowts");
    out
}

impl Alignable for ThreadLikeAligner {
    fn kind(&self) -> AlignKind {
        AlignKind::ThreadLike
    }

    fn matches(&self, node: Node, src: &str) -> bool {
        if node.kind() != "list_lit" {
            return false;
        }

        let children = non_comment_children(node);
        let head = match children.first() {
            Some(h) => *h,
            None => return false,
        };

        matches!(node_text(head, src),
                 "->" | "->>" | "as->" |
                 "some->" | "some->>" |
                 "if" | "when" | "when-not" |
                 "while" | "if-not")
    }

    fn extract(&self, node: Node, src: &str) -> Option<Extracted> {
        extract_thread_layout(node, src)
            .map(|(anchor_byte, rows)| Extracted::Rows { anchor_byte, rows })
    }

    fn build(&self, src: &str, extracted: Extracted, base_col: usize) -> String {
        match extracted {
            Extracted::Rows { anchor_byte, rows } => {
                build_aligned_thread_string(src, anchor_byte, &rows, base_col)
            }
            _ => src.to_string(),
        }
    }
}
