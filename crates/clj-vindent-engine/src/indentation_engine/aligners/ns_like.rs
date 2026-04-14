use tree_sitter::Node;
use tracing::debug;

use crate::indentation_engine::alignable::Alignable;
use crate::indentation_engine::model::Row;
use crate::indentation_engine::model::{AlignKind, Extracted};
use crate::indentation_engine::helpers::{node_text,
                                         line_start_byte,                                         
                                         non_comment_children,
                                         shift_multiline_block};

pub struct NsLikeAligner;

fn compact_ns_entry_text(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalized_ns_clause_text(clause: Node, src: &str) -> String {
    if clause.kind() != "list_lit" {
        return node_text(clause, src).to_string();
    }

    let children = non_comment_children(clause);
    if children.is_empty() {
        return node_text(clause, src).to_string();
    }

    let head = node_text(children[0], src);

    match head {
        ":require" | ":import" | ":use" => rebuild_sorted_ns_clause(clause, src),
        _ => node_text(clause, src).to_string(),
    }
}

fn rebuild_sorted_ns_clause(clause: Node, src: &str) -> String {
    let children = non_comment_children(clause);
    if children.is_empty() {
        return node_text(clause, src).to_string();
    }

    let head = node_text(children[0], src);

    if children.len() == 1 {
        return format!("({head})");
    }

    let mut items: Vec<String> = children[1..]
        .iter()
        .map(|n| compact_ns_entry_text(node_text(*n, src)))
        .collect();

    items.sort_by(|a, b| {
        a.chars()
         .count()
         .cmp(&b.chars().count())
         .then_with(|| a.cmp(b))
    });

    let mut out = String::new();
    out.push('(');
    out.push_str(head);

    for item in items {
        out.push('\n');
        out.push_str(" "); // relative indent inside ns clause
        out.push_str(&item);
    }

    out.push(')');
    out
}

pub fn extract_ns_layout(nd: Node, src: &str) -> Option<(usize, Vec<Row>)> {
    if nd.kind() != "list_lit" {
        return None;
    }

    let children = non_comment_children(nd);
    let head = node_text(*children.first()?, src);

    if head != "ns" {
        return None;
    }

    // Ignore single-line ns forms.
    if nd.start_position().row == nd.end_position().row {
        debug!("Bail out single line ns form");
        return None;
    }
    
    if children.len() < 3 {
        return None;
    }

    // Anchor to first clause after namespace symbol.
    let anchor_byte = children[2].start_byte();

    let rows = children[2..]
        .iter()
        .map(|clause| Row {
            text: normalized_ns_clause_text(*clause, src),
            start_byte: clause.start_byte(),
            end_byte: clause.end_byte(),
        })
        .collect::<Vec<Row>>();

    debug!(rows = rows.len(), "finish ns like extract");
    Some((anchor_byte, rows))
}

pub fn build_aligned_ns_string(
    src: &str,
    _anchor_byte: usize,
    rows: &[Row],
    base_col: usize,
) -> String {
    if rows.is_empty() {
        return src.to_string();
    }

    let target_col = base_col + 2;

    debug!(rows = rows.len(), target_col, "build ns form");

    let mut out = String::new();
    let mut last = 0;
    let mut prev_line_start: Option<usize> = None;

    for row in rows {
        let line_start = line_start_byte(src, row.start_byte);

        if row.start_byte < last || line_start < last {
            return src.to_string();
        }

        if prev_line_start == Some(line_start) {
            return src.to_string();
        }

        // preserve text before clause, just to be sure
        out.push_str(&src[last..line_start]);

        // place first line of clause at base_col(initial column number for the form) + 2
        out.push_str(&" ".repeat(target_col));

        let adjusted_row = shift_multiline_block(&row.text, target_col as isize);
        out.push_str(&adjusted_row);

        last = row.end_byte;
        prev_line_start = Some(line_start);
    }

    out.push_str(&src[last..]);
    debug!("finished ns build");
    out
}

impl Alignable for NsLikeAligner {
    fn kind(&self) -> AlignKind {
        AlignKind::NsLike
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

        node_text(head, src) == "ns"
    }

    fn extract(&self, node: Node, src: &str) -> Option<Extracted> {
        extract_ns_layout(node, src)
            .map(|(anchor_byte, rows)| Extracted::Rows { anchor_byte, rows })
    }

    fn build(&self, src: &str, extracted: Extracted, base_col: usize) -> String {
        match extracted {
            Extracted::Rows { anchor_byte, rows } => {
                build_aligned_ns_string(src, anchor_byte, &rows, base_col)
            }
            _ => src.to_string(),
        }
    }
}
