use tree_sitter::Node;

use crate::indentation_engine::model::{AlignKind, Extracted};

pub trait Alignable {
    fn kind(&self) -> AlignKind;

    fn matches(&self, node: Node, src: &str) -> bool;

    fn extract(&self, node: Node, src: &str) -> Option<Extracted>;

    fn build(&self, src: &str, extracted: Extracted, base_col: usize) -> String;
}
