use rayon::prelude::*;
use std::fs;
use tree_sitter::Node;
use std::time::Instant;

use crate::indentation_engine::alignable::Alignable;
use crate::indentation_engine::aligners::{
    cond_like::CondLikeAligner,
    condp_like::CondPLikeAligner,
    let_like::LetLikeAligner,
    map_like::MapLikeAligner,
    thread_like::ThreadLikeAligner,
    vec_like::VecLikeAligner,
    ns_like::NsLikeAligner
};
use crate::indentation_engine::helpers::{
    absolute_col_in_slice, get_root_node, get_tree, is_traversable, named_children,
};

use tracing::{info, debug, instrument, error};

static LET_ALIGNER: LetLikeAligner = LetLikeAligner;
static MAP_ALIGNER: MapLikeAligner = MapLikeAligner;
static VEC_ALIGNER: VecLikeAligner = VecLikeAligner;
static COND_ALIGNER: CondLikeAligner = CondLikeAligner;
static CONDP_ALIGNER: CondPLikeAligner = CondPLikeAligner;
static THREAD_ALIGNER: ThreadLikeAligner = ThreadLikeAligner;
static NS_ALIGNER: NsLikeAligner = NsLikeAligner;

pub fn find_aligner(node: Node, src: &str) -> Option<&'static dyn Alignable> {
    let aligners: [&'static dyn Alignable; 7] = [
        &NS_ALIGNER,
        &LET_ALIGNER,
        &MAP_ALIGNER,
        &VEC_ALIGNER,
        &COND_ALIGNER,
        &CONDP_ALIGNER,
        &THREAD_ALIGNER,
    ];

    aligners.into_iter().find(|a| a.matches(node, src))
}

pub fn indent_current_form_once(source: &str, base_col: usize) -> String {
    let tree = get_tree(source).expect("parse failed");
    let root = get_root_node(&tree).unwrap();
    let form = match root.named_child(0) {
        Some(f) => f,
        None => return source.to_string(),
    };

    let aligner = match find_aligner(form, source) {
        Some(a) => a,
        None => return source.to_string(),
    };

    debug!(kind = %aligner.kind(), base_col, "selected aligner");
    
    match aligner.extract(form, source) {
        Some(extracted) => aligner.build(source, extracted, base_col),
        None => source.to_string(),
    }
}
fn has_missing_or_error(node: Node) -> bool {
    let result = node.is_error() || node.is_missing() || node.has_error();
    if result {error!("Whole Node parse error: malformed, missing, or unbalanced form");};
    result
}

pub fn format_form_recursive(src: &str, base_col: usize) -> String {
    let tree = match get_tree(src) {
        Some(t) => t,
        None => return src.to_string(),
    };

    let root = tree.root_node();
    assert!(!has_missing_or_error(root),
    "Format-form: parse error: malformed, missing, or unbalanced form");
    
    let form = match root.named_child(0) {
        Some(f) => f,
        None => return src.to_string(),
    };

    let mut current = src.to_string();
    let mut replacements: Vec<(usize, usize, String)> = Vec::new();

    for child in named_children(form) {
        if is_traversable(child) {
            let start = child.start_byte();
            let end = child.end_byte();

            let old_child = &src[start..end];
            let child_base_col = absolute_col_in_slice(src, base_col, start);
            let new_child = format_form_recursive(old_child, child_base_col);

            if old_child != new_child {
                replacements.push((start, end, new_child));
            }
        }
    }

    replacements.sort_by_key(|(start, _, _)| std::cmp::Reverse(*start));

    for (start, end, new_child) in replacements {
        current.replace_range(start..end, &new_child);
    }

    indent_current_form_once(&current, base_col)
}


pub fn indent_bottom_up(src: &str, base_col: usize) -> String {
    format_form_recursive(src, base_col)
}

#[instrument(skip(src))]
pub fn indent_whole_file_parallel(src: &str) -> String {
    let tree = get_tree(src).expect("parse failed");
    info!("Parsing root node");
    let root = get_root_node(&tree).unwrap(); 
    let top_forms = named_children(root);
    let num_of_top_forms = top_forms.len();

    if top_forms.par_iter().any(|form| has_missing_or_error(*form)) {
        panic!("whole-file parse error: malformed, missing, or unbalanced top-level form");
    }
    
    info!("Number of top forms {}", num_of_top_forms);

    let ranges: Vec<(usize, usize)> = top_forms
        .into_par_iter()
        .map(|form| (form.start_byte(), form.end_byte()))
        .collect();

    let mut pieces: Vec<(usize, usize, String)> = ranges
        .into_par_iter()
        .map(|(start, end)| {
            let slice = &src[start..end];
            let formatted = indent_bottom_up(slice, 0);
            (start, end, formatted)
        })
        .collect();

    pieces.sort_by_key(|(start, _, _)| *start);

    let mut out = String::with_capacity(src.len());
    let mut last = 0;

    for (start, end, replacement) in pieces {
        out.push_str(&src[last..start]);
        out.push_str(&replacement);
        last = end;
    }

    out.push_str(&src[last..]);
    out
}

#[instrument(skip(path))]
pub fn indent_clojure_file(path: &str) -> String {
    info!("Start formatting {path} ...");
    
    let start = Instant::now();
    let src = fs::read_to_string(path).unwrap();
    let result = indent_whole_file_parallel(&src);
    let elapsed = start.elapsed();
    
    info!("Done formmating {path}!");
    info!("Elapsed: {:.3?}", elapsed);
    
    result
}

#[instrument(skip(path))]
pub fn indent_clojure_file_no_return(path: String) -> Result<(), std::io::Error> {
    info!("Reading file from path");
    
    let src = fs::read_to_string(&path).unwrap();
    
    info!("File was read successfully!");
    info!("Start formatting {path} ...");
    
    let start = Instant::now();
    let result = indent_whole_file_parallel(&src);
    let elapsed = start.elapsed();
    
    info!("Done formmating {path}!");
    info!("Elapsed: {:.3?}", elapsed);
    info!("Start writing to file the result...");
    
    fs::write(path, result)?;
    
    info!("Done writing to file the result!");
    
    Ok(())
}

#[instrument(skip(src, base_col))]
pub fn indent_clojure_string(src: &str, base_col: usize) -> String {
    info!("Start formatting string");
    debug!("Start formatting string with starting column {base_col} and source {src}");
    
    let start = Instant::now();
    let result = indent_bottom_up(src, base_col);
    let elapsed = start.elapsed();
    
    info!("Done formmating string!");
    debug!("Result string {result}");
    info!("Elapsed: {:.3?}", elapsed);
    
    result
}

#[instrument(skip(cols))]
pub fn indent_clojure_string_collection(cols: &[(String, usize)]) -> Vec<String> {
    info!("Start indent string collection forms in parrallel.\nNumber of strings: {}", cols.len());
    
    let start = Instant::now();
    let result = cols.into_par_iter()
                     .map(|(form, base_col)| indent_bottom_up(form, *base_col))
                     .collect();
    let elapsed = start.elapsed();
    
    info!("Done formatting collection of strings!");
    debug!("Result string {result}");
    info!("Elapsed: {:.3?}", elapsed);
    
    result
}
