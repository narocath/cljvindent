use rstest::rstest;
use pretty_assertions;
use clj_vindent_engine::{indent_bottom_up, AlignKind};

mod fixtures;

#[rstest]
#[case::nested_let(fixtures::let_like::nested_let())]
#[case::simple_let(fixtures::let_like::simple_let())]
#[case::with_nested_map(fixtures::let_like::nested_let_with_nested_map())]
#[case::nested_let_with_cond_map(fixtures::let_like::nested_let_with_cond_map())]
#[case::with_destracturing_symbol(fixtures::let_like::simple_let_with_destracturing_symbol())]
#[case::nested_let_with_thread_macro_cond(fixtures::let_like::nested_let_with_thread_macro_cond())]
#[case::with_multiline_destracturing(fixtures::let_like::simple_let_with_multiline_destrucaturing_symbol())]
#[case::nested_let_with_thread_macros_cond_condp_map(fixtures::let_like::nested_let_with_thread_macros_cond_condp_map())]
fn let_like_alignment_test(#[case] case: (&'static str, &'static str)) {
    let (input, expected) = case;
    let result = indent_bottom_up(input, 0);
//    eprintln!("EXPECTED: {}", expected);
//    eprintln!("RESULT: {}", result);
    pretty_assertions::assert_eq!(result, expected,
    "engine should correctly align a let form as in the expected string");
}


#[rstest]
#[case::match_correctly_form(fixtures::let_like::correctly_match_form_from_string())]
fn let_like_matches_correctly_test(#[case] case: (Vec<AlignKind>, AlignKind)){
    let (found, expected) = case;

    pretty_assertions::assert_eq!(found, vec![expected; found.len()],
                                  "engine should correctly match the LetLike aligner");
    
}
