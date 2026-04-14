use rstest::rstest;
use pretty_assertions;
use clj_vindent_engine::{indent_bottom_up, AlignKind};

mod fixtures;

#[rstest]
#[case::deep_nested_map(fixtures::map_like::deep_nested_map())]
#[case::simple_nested_map(fixtures::map_like::simple_nested_map())]
#[case::nested_map_with_everything(fixtures::map_like::nested_map_with_everything())]
fn map_like_alignment_test(#[case] case: (&'static str, &'static str)) {
    let (input, expected) = case;
    let result = indent_bottom_up(input, 0);
//    eprintln!("EXPECTED: {}", expected);
//    eprintln!("RESULT: {}", result);
    // we need to trim 
    pretty_assertions::assert_eq!(result, expected,
    "engine should correctly align a ns form as in the expected string");
}



#[rstest]
#[case::match_correctly_form(fixtures::map_like::correctly_match_form_from_string())]
fn map_like_matches_correctly_test(#[case] case: (AlignKind, AlignKind)){
    let (found, expected) = case;

    pretty_assertions::assert_eq!(found, expected,
                                  "engine should correctly match the MapLike aligner");
    
}
