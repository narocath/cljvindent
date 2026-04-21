use rstest::rstest;
use pretty_assertions;
use cljvindent_core::{indent_bottom_up, AlignKind};

mod fixtures;

#[rstest]
#[case::simple_case(fixtures::cond_like::simple_case())]
#[case::simple_cond(fixtures::cond_like::simple_cond())]
#[case::complex_cond(fixtures::cond_like::complex_cond())]
#[case::simple_thread_cond(fixtures::cond_like::simple_thread_cond())]
fn cond_like_alignment_test(#[case] case: (&'static str, &'static str)) {
    let (input, expected) = case;
    let result = indent_bottom_up(input, 0);
    //eprintln!("EXPECTED: {}", expected);
    //eprintln!("RESULT: {}", result);
    // we need to trim 
    pretty_assertions::assert_eq!(result.trim(), expected.trim(),
                                  "engine should correctly align a ns form as in the expected string");
}

#[rstest]
#[case::match_correctly_form(fixtures::cond_like::correctly_match_form_from_string())]
fn cond_like_matches_correctly_test(#[case] case: (Vec<AlignKind>, AlignKind)){
    let (found, expected) = case;

    pretty_assertions::assert_eq!(found, vec![expected; found.len()],
                                  "engine should correctly match the CondLike aligner");
    
}
