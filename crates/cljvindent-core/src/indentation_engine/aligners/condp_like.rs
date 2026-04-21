use tree_sitter::Node;
use tracing::debug;

use super::generic_builder::build_aligned_string;
use crate::indentation_engine::alignable::Alignable;
use crate::indentation_engine::model::Pair;
use crate::indentation_engine::model::{AlignKind, Extracted};
use crate::indentation_engine::helpers::{node_text, non_comment_children};

pub struct CondPLikeAligner;

pub fn extract_condp_pairs(form: Node, src: &str) -> Option<Vec<Pair>> {
    if form.kind() != "list_lit" {
        return None;
    }

    let children = non_comment_children(form);
    if children.len() < 4 {
        return None;
    }
    if form.start_position().row == form.end_position().row {
        debug!("Skip condp-like: extract: single-line form");
        return None;
    }

    let head = node_text(children[0], src);
    if head != "condp" {
        debug!(head = head, "skip extract no condp head");
        return None;
    }

    let clauses = &children[3..];
    debug!(head = head, clauses = clauses.len(), "extract condp pairs");
    let mut pairs = Vec::new();
    let mut i = 0;


    while i < clauses.len() {
        let remaining = clauses.len() - i;

        // trailing default clause
        if remaining == 1 {
            let default_node = clauses[i];
            let lh_string = node_text(default_node, src).to_string();

            pairs.push(Pair {
                lh_width: lh_string.len(),
                lh_start_col: default_node.start_position().column,
                lh_string,
                rh_string: String::new(),
                lh_start_byte: default_node.start_byte(),
                lh_end_byte: default_node.end_byte(),
                rh_start_byte: default_node.end_byte(),
                rh_end_byte: default_node.end_byte(),
            });

            break;
        }

        let lhs = clauses[i];

        if i + 2 < clauses.len() && node_text(clauses[i + 1], src) == ":>>" {
            debug!("condp :>> clause");
            let rhs = clauses[i + 2];
            let lh_string = node_text(lhs, src).to_string();

            pairs.push(Pair {
                lh_width: lh_string.len(),
                lh_start_col: lhs.start_position().column,
                lh_string,
                rh_string: format!(":>> {}", node_text(rhs, src)),
                lh_start_byte: lhs.start_byte(),
                lh_end_byte: lhs.end_byte(),
                rh_start_byte: clauses[i + 1].start_byte(),
                rh_end_byte: rhs.end_byte(),
            });

            i += 3;
        } else {
            debug!("condp default clause");
            let rhs = clauses[i + 1];
            let lh_string = node_text(lhs, src).to_string();

            pairs.push(Pair {
                lh_width: lh_string.len(),
                lh_start_col: lhs.start_position().column,
                lh_string,
                rh_string: node_text(rhs, src).to_string(),
                lh_start_byte: lhs.start_byte(),
                lh_end_byte: lhs.end_byte(),
                rh_start_byte: rhs.start_byte(),
                rh_end_byte: rhs.end_byte(),
            });

            i += 2;
        }
    }
    debug!(pairs = pairs.len(), "finished condp extract");
    Some(pairs)
}

impl Alignable for CondPLikeAligner {
    fn kind(&self) -> AlignKind {
        AlignKind::CondPLike
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

        node_text(head, src) == "condp"
    }

    fn extract(&self, node: Node, src: &str) -> Option<Extracted> {
        extract_condp_pairs(node, src).map(Extracted::Pairs)
    }

    fn build(&self, src: &str, extracted: Extracted, base_col: usize) -> String {
        match extracted {
            Extracted::Pairs(pairs) => build_aligned_string(src, &pairs, base_col),
            _ => src.to_string(),
        }
    }
}
