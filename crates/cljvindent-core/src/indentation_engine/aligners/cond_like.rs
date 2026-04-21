use tracing::debug;
use tree_sitter::Node;

use crate::indentation_engine::alignable::Alignable;
use super::generic_builder::build_aligned_string;
use crate::indentation_engine::model::Pair;
use crate::indentation_engine::model::{AlignKind, Extracted};
use crate::indentation_engine::helpers::{node_text, non_comment_children};

pub struct CondLikeAligner;

pub fn extract_cond_like_pairs(nd: Node, src: &str) -> Option<Vec<Pair>> {
    if nd.kind() != "list_lit" {
        return None;
    }
    if nd.start_position().row == nd.end_position().row {
        debug!("Skip cond/case-like: extract: single-line form");
        return None;
    }

    let children = non_comment_children(nd);
    let head = children.first()?;
    let head_text = node_text(head.child(0)?, src);

    let clauses = match head_text {
        "cond->" | "cond->>" | "case" => &children[2..],
        _ => &children[1..], // essetially for simple cond
    };

    if clauses.is_empty() {
        debug!(head = head_text, "skip cond/case-like extract: no clauses");
        return None;
    }

    debug!(head = head_text, clauses = clauses.len(), "extract cond/case-like pairs");
    
    let start_of_first_lh = clauses.first()?.start_position().column;
    let mut pairs = Vec::new();
    let mut it = clauses.iter().copied();

    while let Some(lhs) = it.next() {
        match it.next() {
            Some(rhs) => {
                debug!("Cond like branch");
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
            None if head_text == "case" => {
                debug!("Case default clause");
                let lh_string = node_text(lhs, src).to_string();

                pairs.push(Pair {
                    lh_width: lh_string.len(),
                    lh_start_col: start_of_first_lh,
                    lh_string,
                    rh_string: String::new(),
                    lh_start_byte: lhs.start_byte(),
                    lh_end_byte: lhs.end_byte(),
                    rh_start_byte: lhs.end_byte(),
                    rh_end_byte: lhs.end_byte(),
                });
            }
            None => {
                debug!(head = head_text, "invalid cond/case-like form: missing rhs");
                return None;},
        }
    }
    debug!(head = head_text, pairs = pairs.len(), "finished cond/case-like extract");
    Some(pairs)
}

impl Alignable for CondLikeAligner {
    fn kind(&self) -> AlignKind {
        AlignKind::CondLike
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

        matches!(node_text(head, src), "cond" | "cond->" | "cond->>" | "case")
    }

    fn extract(&self, node: Node, src: &str) -> Option<Extracted> {
        extract_cond_like_pairs(node, src).map(Extracted::Pairs)
    }

    fn build(&self, src: &str, extracted: Extracted, base_col: usize) -> String {
        match extracted {
            Extracted::Pairs(pairs) => build_aligned_string(src, &pairs, base_col),
            _ => src.to_string(),
        }
    }
}
