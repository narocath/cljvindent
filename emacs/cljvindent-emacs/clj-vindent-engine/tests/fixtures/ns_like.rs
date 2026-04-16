use rstest::fixture;
use clj_vindent_engine::{engine::find_aligner, AlignKind, helpers::{get_tree, get_root_node}};

#[fixture]
pub fn simple_ns() -> (&'static str, &'static str) {
    let inp = r#"
(ns dummy.fooo
(:require [foo1.foo2 :as f]
    [fooooooooo1.fooooo2.fooo3 :as fff]
           [fi.fi :as fi]
        [faaaaaaaa.foooooo.fiiiiiii.dummy :as fid ])
        (:import
         [foo1.foo2 :as f]
    (fooooooooo1.fooooo2.fooo3 :as fff)
           (fi.fi :as fi)
        [faaaaaaaa.foooooo.fiiiiiii.dummy :as fid ])
(:use [clojure.string :only [trim lower-case split]]
                        [clojure.contrib.shell-out]
   [clojure.pprint]
        [clojure.test]))
"#;
    let expected = r#"
(ns dummy.fooo
  (:require
   [fi.fi :as fi]
   [foo1.foo2 :as f]
   [fooooooooo1.fooooo2.fooo3 :as fff]
   [faaaaaaaa.foooooo.fiiiiiii.dummy :as fid ])
  (:import
   (fi.fi :as fi)
   [foo1.foo2 :as f]
   (fooooooooo1.fooooo2.fooo3 :as fff)
   [faaaaaaaa.foooooo.fiiiiiii.dummy :as fid ])
  (:use
   [clojure.test]
   [clojure.pprint]
   [clojure.contrib.shell-out]
   [clojure.string :only [trim lower-case split]]))
"#;
    (inp, expected)
}

#[fixture]
pub fn simple_ns_ignore_new_lines() -> (&'static str, &'static str) {
    let inp = r#"
(ns dummy.fooo
(:require [foo1.foo2 :as f]
    [fooooooooo1.fooooo2.fooo3 :as fff]
           [fi.fi
            :as fi]
        [faaaaaaaa.foooooo.fiiiiiii.dummy :as fid ])
        (:import
         [foo1.foo2 :as f]
    (fooooooooo1.fooooo2.fooo3 :as fff)
           (fi.fi :as fi)
        [faaaaaaaa.foooooo.fiiiiiii.dummy
:as fid
:refer [fooo
        haaa
        hooo]]))
"#;
    let expected = r#"
(ns dummy.fooo
  (:require
   [fi.fi :as fi]
   [foo1.foo2 :as f]
   [fooooooooo1.fooooo2.fooo3 :as fff]
   [faaaaaaaa.foooooo.fiiiiiii.dummy :as fid ])
  (:import
   (fi.fi :as fi)
   [foo1.foo2 :as f]
   (fooooooooo1.fooooo2.fooo3 :as fff)
   [faaaaaaaa.foooooo.fiiiiiii.dummy :as fid :refer [fooo haaa hooo]]))
"#;
    (inp, expected)
}

#[fixture]
pub fn correctly_match_form_from_string() -> (AlignKind, AlignKind) {
    let inp = r#"
(ns dummy.fooo
(:require [foo1.foo2 :as f]
    [fooooooooo1.fooooo2.fooo3 :as fff]
           [fi.fi :as fi]
        [faaaaaaaa.foooooo.fiiiiiii.dummy :as fid ])
        (:import
         [foo1.foo2 :as f]
    (fooooooooo1.fooooo2.fooo3 :as fff)
           (fi.fi :as fi)
        [faaaaaaaa.foooooo.fiiiiiii.dummy :as fid ])
(:use [clojure.string :only [trim lower-case split]]
                        [clojure.contrib.shell-out]
   [clojure.pprint]
        [clojure.test]))
"#;
    let tree = get_tree(inp).unwrap();
    let root = get_root_node(&tree).unwrap();
    let form = root.named_child(0).unwrap();

    let found_aligner = find_aligner(form, inp).unwrap().kind();
    
    let expected = AlignKind::NsLike;
    (found_aligner, expected)
}
