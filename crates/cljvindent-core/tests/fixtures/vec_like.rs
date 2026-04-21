use rstest::fixture;
use cljvindent_core::{engine::find_aligner, AlignKind, helpers::{get_tree, get_root_node}};

#[fixture]
pub fn simple_vector() -> (&'static str, &'static str) {
    let inp = r#"
[:fooo
:faaaa
        :fiiii
:fooo
            :fuuuuu]
"#;
    let expected = r#"
[:fooo
 :faaaa
 :fiiii
 :fooo
 :fuuuuu]
"#;
    (inp, expected)
}

#[fixture]
pub fn simple_vector_ignore_same_lines_elements() -> (&'static str, &'static str) {
    let inp = r#"
[:fooo :faaaa
        :fiiii
:fooo
            :fuuuuu]
"#;
    let expected = r#"
[:fooo :faaaa
 :fiiii
 :fooo
 :fuuuuu]
"#;
    (inp, expected)
}

#[fixture]
pub fn simple_vector_of_maps() -> (&'static str, &'static str) {
    let inp = r#"
[{:id 1
  :name "Alice"
  :active? true}
 {:id 2
  :name "Bob"
  :active? false}
 {:id 3
  :name "Eve"
  :active? true}]
"#;
    let expected = r#"
[{:id      1
  :name    "Alice"
  :active? true}
 {:id      2
  :name    "Bob"
  :active? false}
 {:id      3
  :name    "Eve"
  :active? true}]
"#;
    (inp, expected)
}

#[fixture]
pub fn simple_vector_ignore_oneliner() -> (&'static str, &'static str) {
    let inp = r#"
[:fooo :faaaa :fiiii :fooo :fuuuuu]
"#;
    let expected = r#"
[:fooo :faaaa :fiiii :fooo :fuuuuu]
"#;
    (inp, expected)
}

#[fixture]
pub fn correctly_match_form_from_string() -> (AlignKind, AlignKind) {
    let inp = r#"
[:foo :faa :fiii :fhhhhhh]
"#;
    let tree = get_tree(inp).unwrap();
    let root = get_root_node(&tree).unwrap();
    let form = root.named_child(0).unwrap();

    let found_aligner = find_aligner(form, inp).unwrap().kind();
    
    let expected = AlignKind::VecLike;
    (found_aligner, expected)
}
