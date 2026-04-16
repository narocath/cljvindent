use tree_sitter::{Node, Parser, Tree};

pub fn get_tree(source: &str) -> Option<Tree> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_clojure::LANGUAGE.into())
        .ok()?;
    parser.parse(source, None)
}

pub fn non_comment_children(node: Node) -> Vec<Node> {
    // filter out `;;` and `#_`
    named_children(node)
        .into_iter()
        .filter(|n| n.kind() != "comment" && n.kind() != "dis_expr")
        .collect()
}

pub fn get_root_node<'a>(tree: &'a Tree) -> Option<Node<'a>>{
    Some(tree.root_node())
}

pub fn node_text<'a>(node: Node, src: &'a str) -> &'a str {
    &src[node.start_byte()..node.end_byte()]
}

pub fn named_children(node: Node) -> Vec<Node> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor).collect()
}

pub fn local_col(src: &str, byte_pos: usize) -> usize {
    byte_pos - line_start_byte(src, byte_pos)
}

pub fn absolute_col_in_slice(src: &str, base_col: usize, byte_pos: usize) -> usize {
    if line_start_byte(src, byte_pos) == 0 {
        base_col + local_col(src, byte_pos)
    } else {
        local_col(src, byte_pos)
    }
}

pub fn line_start_byte(src: &str, byte_pos: usize) -> usize {
    src[..byte_pos]
        .rfind('\n')
        .map(|i| i + 1)
        .unwrap_or(0)
}

pub fn is_traversable(node: Node) -> bool {
    matches!(node.kind(), "list_lit" | "vec_lit" | "map_lit")
}

pub fn shift_multiline_block(text: &str, delta: isize) -> String {
    if !text.contains('\n') || delta == 0 {
        return text.to_string();
    }

    let mut out = String::new();
    let mut lines = text.split('\n');

    if let Some(first) = lines.next() {
        out.push_str(first);
    }

    for line in lines {
        out.push('\n');

        if line.is_empty() {
            continue;
        }

        let indent = line.bytes().take_while(|b| *b == b' ').count();
        let content = &line[indent..];

        let new_indent = if delta >= 0 {
            indent + delta as usize
        } else {
            indent.saturating_sub((-delta) as usize)
        };

        out.push_str(&" ".repeat(new_indent));
        out.push_str(content);
    }

    out
}
pub fn shift_block_all_lines(text: &str, delta: isize) -> String {
    if delta == 0 || text.is_empty() {
        return text.to_string();
    }

    let mut out = String::new();

    for (i, line) in text.split('\n').enumerate() {
        if i > 0 {
            out.push('\n');
        }

        if line.is_empty() {
            continue;
        }

        let indent = line.bytes().take_while(|b| *b == b' ').count();
        let content = &line[indent..];

        let new_indent = if delta >= 0 {
            indent + delta as usize
        } else {
            indent.saturating_sub((-delta) as usize)
        };

        out.push_str(&" ".repeat(new_indent));
        out.push_str(content);
    }

    out
}
