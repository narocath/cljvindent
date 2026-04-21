use rstest::fixture;
use cljvindent_core::{engine::find_aligner, AlignKind, helpers::{get_tree, get_root_node}};

#[fixture]
pub fn simple_thread_first() -> (&'static str, &'static str) {
    let inp = r#"
(->> [1 2 3 4 5] (filter odd?)
     (map inc)
     (reduce +))
"#;
    let expected = r#"
(->> [1 2 3 4 5]
     (filter odd?)
     (map inc)
     (reduce +))
"#;
    (inp, expected)
}

#[fixture]
pub fn simple_thread_last() -> (&'static str, &'static str){
    let inp = r#"
(->> [1 2 3 4 5] (filter odd?)
            (map inc)
     (reduce +))
"#;
    let expected = r#"
(->> [1 2 3 4 5]
     (filter odd?)
     (map inc)
     (reduce +))
"#;
    (inp, expected)
}

#[fixture]
pub fn simple_thread_as() -> (&'static str, &'static str){
    let inp = r#"
(as-> {:a 1 :b 2}           $
                    (assoc $ :c 3)
(update $ :a inc)
  (select-keys $   [:a :c]))
"#;
    let expected = r#"
(as-> {:a 1 :b 2} $
  (assoc $ :c 3)
  (update $ :a inc)
  (select-keys $   [:a :c]))
"#;
    (inp, expected)
}

#[fixture]
pub fn simple_thread_some() -> (&'static str, &'static str){
    let inp = r#"
(some-> {:user {:name "  Alice "
                    :foo [1 2 3 4]}}
        :user
                    :name
                    clojure.string/trim
        clojure.string/lower-case
        (foo))
"#;
    let expected = r#"
(some-> {:user {:name "  Alice "
                :foo  [1 2 3 4]}}
        :user
        :name
        clojure.string/trim
        clojure.string/lower-case
        (foo))
"#;
    (inp, expected)
}

#[fixture]
pub fn nested_thread_like_with_almost_everything() -> (&'static str, &'static str){
    let inp = r#"
(-> {:user {:profile {:name "  Alice  "
                                    :address {:city "athens"}
    :tier :gold}
             :scores [1 nil 2 3]
                                :active? true
             :country "GR"}}
    :user
    (some-> (assoc :seen true))
    (as-> $
(let [clean-name (some-> $ :profile :name clojure.string/trim clojure.string/lower-case)
            city       (some-> $ :profile :address :city clojure.string/upper-case)
            tier-label (condp = (get-in $ [:profile :tier])
                         :gold :vip
                         :silver :preferred
                         :bronze :standard
                         :unknown)]
     (-> $
            (assoc-in [:profile :name] clean-name)
            (assoc-in [:profile :city] city)
            (assoc :tier-label tier-label))))
   (as-> 
      (cond-> $
        (:active? $) (assoc :active-label :active)
        (not (:active? $)) (assoc :active-label :inactive)
        true (update :scores
                     (fn [xs]
                       (some->> xs
                                        (filter number?)
            (map inc)
                                vec)))) $
     $)
    ((fn [user]
       (let [total (->> (:scores user)
                                    (map identity)
                        (reduce + 0))
             avg   (if (seq (:scores user))
                     (/ total (count (:scores user)))
                     0)
             bucket (cond
                      (>= avg 4) :high
                      (>= avg 2) :medium
                      :else :low)
             region (condp = (:country user)
                      "GR"              :greece
                                    "DE" :europe
                      "US" :>> (fn [_] :united-states)
                      :other)]
         (assoc user
                :total total
                        :avg avg
                :bucket bucket
                :region region))))
    (as-> $
                (assoc $ :display-name (some-> $ :profile :name))
      (select-keys $ [:profile
                                :scores
                      :total
                      :avg
                        :bucket
                      :region
                      :tier-label
                      :active-label
                      :display-name
                      :seen])))
"#;
    let expected = r#"
(-> {:user {:profile {:name    "  Alice  "
                      :address {:city "athens"}
                      :tier    :gold}
            :scores  [1 nil 2 3]
            :active? true
            :country "GR"}}
    :user
    (some-> (assoc :seen true))
    (as-> $ (let [clean-name (some-> $ :profile :name clojure.string/trim clojure.string/lower-case)
      city       (some-> $ :profile :address :city clojure.string/upper-case)
      tier-label (condp = (get-in $ [:profile :tier])
                   :gold   :vip
                   :silver :preferred
                   :bronze :standard
                   :unknown)]
  (-> $
      (assoc-in [:profile :name] clean-name)
      (assoc-in [:profile :city] city)
      (assoc :tier-label tier-label)))
      (let [clean-name (some-> $ :profile :name clojure.string/trim clojure.string/lower-case)
            city       (some-> $ :profile :address :city clojure.string/upper-case)
            tier-label (condp = (get-in $ [:profile :tier])
                         :gold   :vip
                         :silver :preferred
                         :bronze :standard
                         :unknown)]
        (-> $
            (assoc-in [:profile :name] clean-name)
            (assoc-in [:profile :city] city)
            (assoc :tier-label tier-label))))
    (as-> 
       (cond-> $
         (:active? $)       (assoc :active-label :active)
         (not (:active? $)) (assoc :active-label :inactive)
         true               (update :scores
                                    (fn [xs]
                                      (some->> xs
                                               (filter number?)
                                               (map inc)
                                               vec)))) $
      $)
    ((fn [user]
       (let [total  (->> (:scores user)
                         (map identity)
                         (reduce + 0))
             avg    (if (seq (:scores user))
                      (/ total (count (:scores user)))
                      0)
             bucket (cond
                      (>= avg 4) :high
                      (>= avg 2) :medium
                      :else      :low)
             region (condp = (:country user)
                      "GR" :greece
                      "DE" :europe
                      "US" :>> (fn [_] :united-states)
                      :other)]
         (assoc user
                :total total
                        :avg avg
                :bucket bucket
                :region region))))
    (as-> $ (assoc $ :display-name (some-> $ :profile :name))
      (assoc $ :display-name (some-> $ :profile :name))
      (select-keys $ [:profile
                      :scores
                      :total
                      :avg
                      :bucket
                      :region
                      :tier-label
                      :active-label
                      :display-name
                      :seen])))
"#;
    (inp, expected)
}

#[fixture]
pub fn simple_thread_with_rest() -> (&'static str, &'static str){
    let inp = r#"
(as-> {:count 0
       :enabled? true
                        :blocked? false
       :done?                           false
       :items [1 2 3]}
$
  (when (:enabled? $)
    (assoc $ :seen true))

  (if (:seen $)
                            (update $ :count inc)
    $)

  (if-not (:blocked? $)
                (assoc $ :status :allowed)
    (assoc $ :status :blocked))

  (when-not (:done? $)
    (assoc $ :pending true))

  (let [state (atom $)]
    (while (< (:count @state) 3)
        (swap! state update :count inc))
    @state))
"#;
    let expected = r#"
(as-> {:count    0
       :enabled? true
       :blocked? false
       :done?    false
       :items    [1 2 3]} $
  $
  (when (:enabled? $)
    (assoc $ :seen true))

  (if (:seen $)
    (update $ :count inc)
    $)

  (if-not (:blocked? $)
    (assoc $ :status :allowed)
    (assoc $ :status :blocked))

  (when-not (:done? $)
    (assoc $ :pending true))

  (let [state (atom $)]
    (while (< (:count @state) 3)
      (swap! state update :count inc))
    @state))
"#;
    (inp, expected)
}

#[fixture]
pub fn correctly_match_thread_like_from_string() -> (Vec<AlignKind>, AlignKind) {
    let inp_thread_fist = r#"
(-> [1 2 3 4 5] (filter odd?)
     (map inc)
     (reduce +))
"#;
    let inp_thread_last = r#"
(->> [1 2 3 4 5] (filter odd?)
     (map inc)
     (reduce +))
"#;
    let inp_thread_as = r#"
(as-> {:a 1 :b 2}           $
                    (assoc $ :c 3)
(update $ :a inc)
  (select-keys $   [:a :c]))
"#;
    let inp_thread_some_first = r#"
(some-> {:a 1 :b 2}        
                    (assoc :c 3)
(update  :a inc)
  (select-keys   [:a :c]))
"#;
    let inp_thread_some_last = r#"
(some->> {:a 1 :b 2}        
                    (map :c 3)
(reduce  :a inc)
  :b)
"#;
    let inp_when = r#"
(when (:foo something)
(println "something"))
"#;
    let inp_when_not = r#"
(when-not (:foo something)
(println "something"))
"#;
    let inp_if = r#"
(if (:foo something)
(println "something")
    (println "none"))
"#;
    let inp_if_not = r#"
(if-not (:foo something)
(println "something")
    (println "none"))
"#;
    let inp_while = r#"
(while (:foo something)
(println "something"))
"#;
    let results = vec![inp_if, inp_if_not, inp_thread_fist, inp_thread_last,
                       inp_thread_some_first, inp_thread_some_last, inp_when, inp_when_not,
                       inp_while, inp_thread_as]
        .iter().map(|i| {
            let tree = get_tree(i).unwrap();
            let root = get_root_node(&tree).unwrap();
            let form = root.named_child(0).unwrap();
            find_aligner(form, i).unwrap().kind()})
               .collect::<Vec<AlignKind>>(); 
    
    let expected = AlignKind::ThreadLike;
    (results, expected)
}
