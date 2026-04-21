use rstest::rstest;
use pretty_assertions;
use cljvindent_core::{indent_bottom_up, AlignKind};

mod fixtures;

#[rstest]
#[case::simple_ns(fixtures::ns_like::simple_ns())]
#[case::simple_ns_ignore_new_lines(fixtures::ns_like::simple_ns_ignore_new_lines())]
fn ns_like_alignment_and_ordering_test(#[case] case: (&'static str, &'static str)) {
    let (input, expected) = case;
    let result = indent_bottom_up(input, 0);
//    eprintln!("EXPECTED: {}", expected);
//    eprintln!("RESULT: {}", result);
    pretty_assertions::assert_eq!(result, expected,
    "engine should correctly align a ns form as in the expected string");
}

#[rstest]
#[case::match_correctly_form(fixtures::ns_like::correctly_match_form_from_string())]
fn ns_like_matches_correctly_test(#[case] case: (AlignKind, AlignKind)){
    let (found, expected) = case;

    pretty_assertions::assert_eq!(found, expected,
                                  "engine should correctly match the NsLike aligner");
    
}
