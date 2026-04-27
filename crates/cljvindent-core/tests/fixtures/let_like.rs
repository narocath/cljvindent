use rstest::fixture;
use cljvindent_core::{engine::find_aligner, AlignKind, helpers::{get_tree, get_root_node}};

#[fixture]
pub fn simple_let() -> (&'static str, &'static str) {
    let inp = r#"
(let [dummy-1 0
                dummy-really-long-sym-really-long "foooooooo"
a (fooo call foooo)]
  (do
    (foo1 a)
    (foo2 a)))
"#;
    let expected = r#"
(let [dummy-1                           0
      dummy-really-long-sym-really-long "foooooooo"
      a                                 (fooo call foooo)]
  (do
    (foo1 a)
    (foo2 a)))
"#;
    (inp, expected)
}

#[fixture]
pub fn simple_let_with_destructuring_symbol() -> (&'static str, &'static str) {
    let inp = r#"
(let [dummy-1 0
      dummy-really-long-sym-really-long "foooooooo"
a                                           (fooo call foooo)
      {:keys [fooo faaa] :as res} a]
  (do
    (foo1 a)
    (foo2 a)))
"#;
    let expected = r#"
(let [dummy-1                           0
      dummy-really-long-sym-really-long "foooooooo"
      a                                 (fooo call foooo)
      {:keys [fooo faaa] :as res}        a]
  (do
    (foo1 a)
    (foo2 a)))
"#;
    (inp, expected)
} 
#[fixture]
pub fn simple_let_with_multiline_destructuring_symbol() -> (&'static str, &'static str){
    let inp = r#"
(let [dummy-1            0
dummy-really-long-sym-really-long                                     "foooooooo"
      a               (fooo call foooo)
      {:keys [fooo faaa]
       :as res}     a]
  (do
    (foo1 a)
    (foo2 a)))
"#;
    let expected = r#"
(let [dummy-1                           0
      dummy-really-long-sym-really-long "foooooooo"
      a                                 (fooo call foooo)
      {:keys [fooo faaa]
       :as   res}                       a]
  (do
    (foo1 a)
    (foo2 a)))
"#;
    (inp, expected)
}


#[fixture]
pub fn nested_let() -> (&'static str, &'static str){
    let inp = r#"
(let [dummy-1                               0
 dummy-really-long-sym-really-long          "foooooooo"
      a  (fooo call foooo)
      b (let [dummy-1                               0
 dummy-really-long-sym-really-long          "foooooooo"
      a  (fooo call foooo)
              c (let [dummy-1                               0
 dummy-really-long-sym-really-long          "foooooooo"
      a  (fooo call foooo)
      {:keys [fooo faaa] :as res}       a]
  (do
    (foo1 a)
    (foo2 a)))]
  (do
    (foo1 a)
    (foo2 a)))]
  (do
    (foo1 a)
    (foo2 a)))
"#;
    let expected = r#"
(let [dummy-1                           0
      dummy-really-long-sym-really-long "foooooooo"
      a                                 (fooo call foooo)
      b                                 (let [dummy-1                           0
                                              dummy-really-long-sym-really-long "foooooooo"
                                              a                                 (fooo call foooo)
                                              c                                 (let [dummy-1                           0
                                                                                      dummy-really-long-sym-really-long "foooooooo"
                                                                                      a                                 (fooo call foooo)
                                                                                      {:keys [fooo faaa] :as res}       a]
                                                                                  (do
                                                                                    (foo1 a)
                                                                                    (foo2 a)))]
                                          (do
                                            (foo1 a)
                                            (foo2 a)))]
  (do
    (foo1 a)
    (foo2 a)))
"#;
    (inp, expected)
}

#[fixture]
pub fn nested_let_with_nested_map() -> (&'static str, &'static str){
    let inp = r#"
(let [dummy-1                                                               0
      dummy-really-long-sym-really-long "foooooooo"
a        (fooo call foooo)
      b                                 (let [dummy-1                           0
                                              dummy-really-long-sym-really-long "foooooooo"
                                              a                                 (fooo call foooo)
                                              c                                 {:fooo fooo1
                                                                    :faaaa faaa2
                                                                                        :fiiii {:fiii1 (fooo a)
                                                                        :fpppppp            4
                                                                                                :ccccc {:faa feo
                                        :ggggggggggggg "gggggggggg"}}}]                                          
                                                    (foo1 a)
                                         (foo2 a))]
                    (foo1 a)
 (foo2 a))
"#;
    let expected = r#"
(let [dummy-1                           0
      dummy-really-long-sym-really-long "foooooooo"
      a                                 (fooo call foooo)
      b                                 (let [dummy-1                           0
                                              dummy-really-long-sym-really-long "foooooooo"
                                              a                                 (fooo call foooo)
                                              c                                 {:fooo  fooo1
                                                                                 :faaaa faaa2
                                                                                 :fiiii {:fiii1   (fooo a)
                                                                                         :fpppppp 4
                                                                                         :ccccc   {:faa           feo
                                                                                                   :ggggggggggggg "gggggggggg"}}}]                                          
                                          (foo1 a)
                                          (foo2 a))]
  (foo1 a)
  (foo2 a))
"#;
    (inp, expected)
}

#[fixture]
pub fn nested_let_with_cond_map() -> (&'static str, &'static str){
    let inp = r#"
(let [dummy-1         0
      dummy-really-long-sym-really-long "foooooooo"
      a                                 (fooo call foooo)
      b                                 (let [dummy-1                           0
                                              dummy-really-long-sym-really-long "foooooooo"
                                              a                                 (fooo call foooo)
                                              c                                            {:fooo  fooo1
                                                                  :faaaa faaa2
                                                                                 :fiiii {:fiii1   (fooo a)
                                                                            :fpppppp                       4
                                                                                         :ccccc   {:faa           feo
                                                                                           :ggggggggggggg "gggggggggg"}}}]                                          
                                          (cond (= 1 1) (fooo a)
                            (= 2 2eeeee) (fooo11 b)
                                                :else nil)
                                          (foo2 a))]
  (foo1 a)
  (cond
(= 1 1) (fooo a)
                                        (= 2 2eeeee) (fooo11 b)
                                                :else nil))
"#;
    let expected = r#"
(let [dummy-1                           0
      dummy-really-long-sym-really-long "foooooooo"
      a                                 (fooo call foooo)
      b                                 (let [dummy-1                           0
                                              dummy-really-long-sym-really-long "foooooooo"
                                              a                                 (fooo call foooo)
                                              c                                 {:fooo  fooo1
                                                                                 :faaaa faaa2
                                                                                 :fiiii {:fiii1   (fooo a)
                                                                                         :fpppppp 4
                                                                                         :ccccc   {:faa           feo
                                                                                                   :ggggggggggggg "gggggggggg"}}}]                                          
                                          (cond
                                            (= 1 1)      (fooo a)
                                            (= 2 2eeeee) (fooo11 b)
                                            :else        nil)
                                          (foo2 a))]
  (foo1 a)
  (cond
    (= 1 1)      (fooo a)
    (= 2 2eeeee) (fooo11 b)
    :else        nil))
"#;
    (inp, expected)
}
#[fixture]
pub fn nested_let_with_thread_macro_cond() -> (&'static str, &'static str){
    let inp = r#"
(let [dummy-1         0
      dummy-really-long-sym-really-long "foooooooo"
      a                                 (fooo call foooo)
      b                                 (let [dummy-1                           0
                                              dummy-really-long-sym-really-long "foooooooo"
                                              a                                 (fooo call foooo)
                                              c                                            (->> foo
                                        (assoc :foo o)
                                    (assoc :fiii o)
                                          (assoc :fuuuu aaa))]
                                          (-> foo
                                        (assoc :foo o)
                                    (assoc :fiii o)
                                          (assoc :fuuuu aaa))
                                          (foo2 a))]
  (foo1 a)
  (cond (= 1 1)      (fooo a)
    (= 2 2eeeee) (fooo11 b)
    :else        nil))
"#;
    let expected = r#"
(let [dummy-1                           0
      dummy-really-long-sym-really-long "foooooooo"
      a                                 (fooo call foooo)
      b                                 (let [dummy-1                           0
                                              dummy-really-long-sym-really-long "foooooooo"
                                              a                                 (fooo call foooo)
                                              c                                 (->> foo
                                                                                     (assoc :foo o)
                                                                                     (assoc :fiii o)
                                                                                     (assoc :fuuuu aaa))]
                                          (-> foo
                                              (assoc :foo o)
                                              (assoc :fiii o)
                                              (assoc :fuuuu aaa))
                                          (foo2 a))]
  (foo1 a)
  (cond
    (= 1 1)      (fooo a)
    (= 2 2eeeee) (fooo11 b)
    :else        nil))
"#;
    (inp, expected)
}
#[fixture]
pub fn nested_let_with_thread_macros_cond_condp_map() -> (&'static str, &'static str){
    let inp = r#"
(let [dummy-1         0
      dummy-really-long-sym-really-long "foooooooo"
      a                                 (fooo call foooo)
      x {:fooo   fooo1
                                         :faaaaa faaa2
                         :fiiii  {:fiii1   (fooo a)
                :fpppppp                      4
                            :ccccc   {:faa          feo
                                                           :ggggggggggggg "gggggggggg"}}}
      b                                 (let [dummy-1                           0
                                              dummy-really-long-sym-really-long "foooooooo"
                                              a                                 (fooo call foooo)
                                              c                                            (as-> [:foo]          $
                                                                                             (assoc $ :foo o)
                                                                                                          (assoc $ :fiii o)
                                                                                             (assoc $ :fuuuu aaa))]
                                          (-> foo
                                              (assoc :foo o)
                                              (assoc :fiii o)
                                              (assoc :fuuuu aaa))
                                          (foo2 a))
      c (condp = some-value
  :foo "plain-foo"
  :bar :>> (fn [v]
             (handle-bar v))
  :baz "plain-baz"
  :qux :>> (fn [v]
             {:value v
              :handled true})
  nil)]
  (condp = some-value
  :foo "plain-foo"
  :bar :>> (fn [v]
             (handle-bar v))
  :baz "plain-baz"
  :qux :>> (fn [v]
             {:value v
              :handled true})
  nil)
  (foo1 a)
  (cond (= 1 1)      (fooo a)
    (= 2 2eeeee) (fooo11 b)
    :else        nil))
"#;
    let expected = r#"
(let [dummy-1                           0
      dummy-really-long-sym-really-long "foooooooo"
      a                                 (fooo call foooo)
      x                                 {:fooo   fooo1
                                         :faaaaa faaa2
                                         :fiiii  {:fiii1   (fooo a)
                                                  :fpppppp 4
                                                  :ccccc   {:faa           feo
                                                            :ggggggggggggg "gggggggggg"}}}
      b                                 (let [dummy-1                           0
                                              dummy-really-long-sym-really-long "foooooooo"
                                              a                                 (fooo call foooo)
                                              c                                 (as-> [:foo] $
                                                                                  (assoc $ :foo o)
                                                                                  (assoc $ :fiii o)
                                                                                  (assoc $ :fuuuu aaa))]
                                          (-> foo
                                              (assoc :foo o)
                                              (assoc :fiii o)
                                              (assoc :fuuuu aaa))
                                          (foo2 a))
      c                                 (condp = some-value
                                          :foo "plain-foo"
                                          :bar :>> (fn [v]
                                                     (handle-bar v))
                                          :baz "plain-baz"
                                          :qux :>> (fn [v]
                                                     {:value   v
                                                      :handled true})
                                          nil)]
  (condp = some-value
    :foo "plain-foo"
    :bar :>> (fn [v]
               (handle-bar v))
    :baz "plain-baz"
    :qux :>> (fn [v]
               {:value   v
                :handled true})
    nil)
  (foo1 a)
  (cond
    (= 1 1)      (fooo a)
    (= 2 2eeeee) (fooo11 b)
    :else        nil))
"#;
    (inp, expected)
}

#[fixture]
pub fn correctly_match_form_from_string() -> (Vec<AlignKind>, AlignKind) {
    let inp_let = r#"
(let [dummy-1 0
                dummy-really-long-sym-really-long "foooooooo"
a (fooo call foooo)]
  (do
    (foo1 a)
    (foo2 a)))
"#;
    let inp_when_let = r#"
(when-let [dummy-1 0]
  (do
    (foo1 a)
    (foo2 a)))
"#;
    let inp_if_let = r#"
(if-let [dummy-1 0]
  (do
    (foo1 a)
    (foo2 a))
"foo")
"#;
    let inp_binding = r#"
(binding [dummy-1 0
                dummy-really-long-sym-really-long "foooooooo"
a (fooo call foooo)]
  (do
    (foo1 a)
    (foo2 a)))
"#;
    let inp_loop = r#"
(loop [dummy-1 0
                dummy-really-long-sym-really-long "foooooooo"
a (fooo call foooo)]
  (do
    (foo1 a)
    (foo2 a)))
"#;
    let inp_with_open = r#"
(with-open [dummy-1 0
                dummy-really-long-sym-really-long "foooooooo"
a (fooo call foooo)]
  (do
    (foo1 a)
    (foo2 a)))
"#;
    let inp_with_redefs = r#"
(with-redefs [dummy-1 0
                dummy-really-long-sym-really-long "foooooooo"
a (fooo call foooo)]
  (do
    (foo1 a)
    (foo2 a)))
"#;
    let results = vec![inp_let, inp_when_let, inp_if_let, inp_binding, inp_loop,
                       inp_with_open, inp_with_redefs]
        .iter().map(|i| {
            let tree = get_tree(i).unwrap();
            let root = get_root_node(&tree).unwrap();
            let form = root.named_child(0).unwrap();
            find_aligner(form, i).unwrap().kind()})
               .collect::<Vec<AlignKind>>(); 
    
    let expected = AlignKind::LetLike;
    (results, expected)
}
