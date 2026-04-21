use rstest::rstest;
use pretty_assertions;
use cljvindent_core::{indent_bottom_up, AlignKind};

mod fixtures;

#[rstest]
#[case::simple_vector(fixtures::vec_like::simple_vector())]
#[case::simple_vector_ignore_oneliner(fixtures::vec_like::simple_vector_ignore_oneliner())]
#[case::simple_vector_of_maps(fixtures::vec_like::simple_vector_of_maps())]
#[case::simple_vector_ignore_same_lines_elements(fixtures::vec_like::simple_vector_ignore_same_lines_elements())]
fn vec_like_alignment_test(#[case] case: (&'static str, &'static str)) {
    let (input, expected) = case;
    let result = indent_bottom_up(input, 0);
    //eprintln!("EXPECTED: {}", expected);
    //eprintln!("RESULT: {}", result);
    // we need to trim 
    pretty_assertions::assert_eq!(result.trim(), expected.trim(),
                                  "engine should correctly align a ns form as in the expected string");
}
#[rstest]
#[case::match_vec_like(fixtures::vec_like::correctly_match_form_from_string())]
fn vec_like_matches_correctly_test(#[case] case: (AlignKind, AlignKind)){
    let (found, expected) = case;

    pretty_assertions::assert_eq!(found, expected,
                                  "engine should correctly match the MapLike aligner");
    
}
