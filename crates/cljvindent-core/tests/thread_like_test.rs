use rstest::rstest;
use pretty_assertions;
use cljvindent_core::{indent_bottom_up, AlignKind};

mod fixtures;

#[rstest]
#[case::simple_therad_some(fixtures::thread_like::simple_thread_some())]
#[case::simple_therad_as(fixtures::thread_like::simple_thread_as())]
#[case::simple_therad_last(fixtures::thread_like::simple_thread_last())]
#[case::simple_thread_first(fixtures::thread_like::simple_thread_first())]
#[case::simple_thread_with_rest(fixtures::thread_like::simple_thread_with_rest())]
#[case::nested_thread_like_with_almost_everything(fixtures::thread_like::nested_thread_like_with_almost_everything())]
fn thread_like_alignment_test(#[case] case: (&'static str, &'static str)) {
    let (input, expected) = case;
    let result = indent_bottom_up(input, 0);
//    eprintln!("EXPECTED: {}", expected);
//    eprintln!("RESULT: {}", result);
    // we need to trim 
    pretty_assertions::assert_eq!(result.trim(), expected.trim(),
                                  "engine should correctly align a ns form as in the expected string");
}


#[rstest]
#[case::match_thread_like(fixtures::thread_like::correctly_match_thread_like_from_string())]
fn thread_like_matches_correctly_test(#[case] case: (Vec<AlignKind>, AlignKind)){
    let (found, expected) = case;

    pretty_assertions::assert_eq!(found, vec![expected; found.len()],
                                  "engine should correctly match the NsLike aligner");
    
}
