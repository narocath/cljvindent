use tree_sitter::Node;
use tracing::debug;
use crate::indentation_engine::model::Pair;
use crate::indentation_engine::alignable::Alignable;
use crate::indentation_engine::helpers::{absolute_col_in_slice,
                                         node_text,
                                         line_start_byte,
                                         get_tree,
                                         get_root_node,
                                         non_comment_children,
                                         shift_multiline_block};
use crate::indentation_engine::model::{AlignKind, Extracted};

pub struct LetLikeAligner;

fn last_line_width(s: &str) -> usize {
    s.lines().last().map(|l| l.chars().count()).unwrap_or(0)
}

pub fn extract_let_like_pairs(nd: Node, src: &str) -> Option<Vec<Pair>> {
    if nd.kind() != "list_lit" {
        return None;
    }

    let children = non_comment_children(nd);
    if children.len() < 2 {
        return None;
    }

    let binding_vec = children[1];
    if binding_vec.kind() != "vec_lit" {
        debug!("skip let-like: extract: not a binding vector");
        return None;
    }

    let bindings = non_comment_children(binding_vec);

    if bindings.is_empty() || bindings.len() % 2 != 0 {
        debug!("Skip let-like: extract: binding vector empty");
        return None;
    }

    let target_lhs_col = bindings.first()?.start_position().column;
    let mut pairs = Vec::new();

    for chunk in bindings.chunks(2) {
        let lhs = chunk[0];
        let rhs = chunk[1];
        let lh_string = node_text(lhs, src).to_string();
        let lh_width = if lh_string.contains('\n') {
            last_line_width(&lh_string)
        } else {
            lh_string.chars().count()
        };
        pairs.push(Pair {
            lh_width,
            lh_start_col: target_lhs_col,
            lh_string,
            rh_string: node_text(rhs, src).to_string(),
            lh_start_byte: lhs.start_byte(),
            lh_end_byte: lhs.end_byte(),
            rh_start_byte: rhs.start_byte(),
            rh_end_byte: rhs.end_byte(),
        });
    }
    debug!(pairs = pairs.len(), "finished let-like pairs extract");
    Some(pairs)
}
pub fn build_aligned_let_body(
    src: &str,
    base_col: usize,
    out: &mut String,
    children: &[Node],
    last_byte: usize) -> Option<usize>{
    let mut last = last_byte;
    let target_body_col = base_col + 2;
    let body_children = &children[2..];

    for child in body_children {
        let start = child.start_byte();
        let end = child.end_byte();
        let line_start = line_start_byte(src, start);

        if start < last || line_start < last {
            debug!("let body bail-out: overlapping/reversed ranges");
            return None;
        }

        // Preserve text before this body form up to the start of its line
        out.push_str(&src[last..line_start]);

        let child_src = &src[start..end];

        // place the first line explicitly at target body column.
        out.push_str(&" ".repeat(target_body_col));

        let old_child_col = absolute_col_in_slice(src, base_col, start);

        let adjusted_child = shift_multiline_block(
            child_src,
            target_body_col as isize - old_child_col as isize,
        );

        out.push_str(&adjusted_child);

        last = end;
    }
    out.push_str(&src[last..]);
    Some(last)
}
pub fn build_aligned_let_binding_vec(
    src: &str,
    pairs: &[Pair],
    base_col: usize,
    out: &mut String,
    last_byte: usize,
    binding_vec: &Node,
) -> Option<usize> {
    let mut last = last_byte;
    let target_lhs_col = absolute_col_in_slice(src, base_col, pairs[0].lh_start_byte);
    let target_rhs_col = pairs
        .iter()
        .map(|p| target_lhs_col + p.lh_width)
        .max()
        .unwrap_or(target_lhs_col)
        + 1;

    let mut prev_line_start: Option<usize> = None;

    for (i, pair) in pairs.iter().enumerate() {
        let line_start = line_start_byte(src, pair.lh_start_byte);

        if pair.lh_start_byte < last || line_start < last {
            debug!("let binding bail-out: overlapping/reversed ranges");
            return None;
        }

        if let Some(prev) = prev_line_start {
            if prev == line_start {
                return None;
            }
        }

        if i == 0 {
            out.push_str(&src[last..pair.lh_start_byte]);
        } else {
            out.push_str(&src[last..line_start]);
            out.push_str(&" ".repeat(target_lhs_col));
        }

        out.push_str(&pair.lh_string);

        let current_rhs_anchor = if pair.lh_string.contains('\n') {
            pair.lh_width
        } else {
            target_lhs_col + pair.lh_width
        };

        let spaces = target_rhs_col.saturating_sub(current_rhs_anchor);
        
        out.push_str(&" ".repeat(spaces));

        if pair.rh_string.contains('\n') {
            let old_rhs_col = absolute_col_in_slice(src, base_col, pair.rh_start_byte);
            let adjusted_rhs = shift_multiline_block(
                &pair.rh_string,
                target_rhs_col as isize - old_rhs_col as isize,
            );
            out.push_str(&adjusted_rhs);
        } else {
            out.push_str(pair.rh_string.trim_start());
        }

        last = pair.rh_end_byte;
        prev_line_start = Some(line_start);
    }

    let binding_vec_end = binding_vec.end_byte();
    if last < binding_vec_end {
        out.push_str(&src[last..binding_vec_end]);
    }
    last = binding_vec_end;
    Some(last)
}
pub fn build_aligned_let_string(src: &str, pairs: &[Pair], base_col: usize) -> String {
    if pairs.is_empty() {
        return src.to_string();
    }

    let tree = match get_tree(src) {
        Some(t) => t,
        None => return src.to_string(),
    };
    let root = match get_root_node(&tree) {
        Some(r) => r,
        None => return src.to_string(),
    };
    let form = match root.named_child(0) {
        Some(f) => f,
        None => return src.to_string(),
    };
    
    debug!(pairs = pairs.len(), base_col, "build let-like");
    if form.kind() != "list_lit" {
        debug!("let builder bail-out: not list_lit");
        return src.to_string();
    }

    let children = non_comment_children(form);
    if children.len() < 2 {
        return src.to_string();
    }

    let binding_vec = children[1];
    if binding_vec.kind() != "vec_lit" {
        return src.to_string();
    }

    let mut out = String::new();
    let last = match build_aligned_let_binding_vec(
        src,
        pairs,
        base_col,
        &mut out,
        0,
        &binding_vec,
    ) {
        Some(last) => last,
        None => return src.to_string(),
    };

    let result = match build_aligned_let_body(src, base_col, &mut out, &children, last) {
        Some(_) => out,
        None => src.to_string(),
    };
    
    debug!("finished let-like build");
    result
    
}

impl Alignable for LetLikeAligner {
    fn kind(&self) -> AlignKind {
        AlignKind::LetLike
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

        matches!(
            node_text(head, src),
            "let" | "when-let" | "if-let" | "binding" | "loop" | "with-open" | "with-redefs"
        )
    }

    fn extract(&self, node: Node, src: &str) -> Option<Extracted> {
        extract_let_like_pairs(node, src).map(Extracted::Pairs)
    }

    fn build(&self, src: &str, extracted: Extracted, base_col: usize) -> String {
        match extracted {
            Extracted::Pairs(pairs) => build_aligned_let_string(src, &pairs, base_col),
            _ => src.to_string(),
        }
    }
}
