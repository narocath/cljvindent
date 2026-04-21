use clj_vindent_engine::{engine::find_aligner, AlignKind, helpers::{get_tree, get_root_node}};
use rstest::fixture;

#[fixture]
pub fn simple_cond() -> (&'static str, &'static str) {
    let inp = r#"
(cond
  (= status :new)  :fresh
  (= status :processing)                {:fooo "aoeuaoeeuaoeuoeauaoeu"
            :fii "foeuoeu"
                          :fooo/faaaaaaaaa 4}
  (= status :failed)                                :failed
  :else                  :unknown)
"#;
    let expected = r#"
(cond
  (= status :new)        :fresh
  (= status :processing) {:fooo            "aoeuaoeeuaoeuoeauaoeu"
                          :fii             "foeuoeu"
                          :fooo/faaaaaaaaa 4}
  (= status :failed)     :failed
  :else                  :unknown)
"#;
    (inp, expected)
}

#[fixture]
pub fn simple_thread_cond() -> (&'static str, &'static str) {
    let inp = r#"
(cond-> {:id 42
         :tags [:a :b]
         :active? true
         :premium? true}
  (:active? profile)                          (assoc :status :active)

  (:premium? profile)
  (assoc :tier :gold)

  (seq (:tags profile))
  (update :tags conj :seen))
"#;
    let expected = r#"
(cond-> {:id       42
         :tags     [:a :b]
         :active?  true
         :premium? true}
  (:active? profile)    (assoc :status :active)

  (:premium? profile)   (assoc :tier :gold)

  (seq (:tags profile)) (update :tags conj :seen))
"#;
    (inp, expected)
}

#[fixture]
pub fn simple_case() -> (&'static str, &'static str) {
    let inp = r#"
(case kind
:create {:status :ok
     :route :create-flow}
                (:update foo) {:status :ok
                           :route :update-flow}
                        :delete {:status :ok
           :route :delete-flow}
                                {:status :error
   :route :unknown})
"#;
    let expected = r#"
(case kind
  :create       {:status :ok
                 :route  :create-flow}
  (:update foo) {:status :ok
                 :route  :update-flow}
  :delete       {:status :ok
                 :route  :delete-flow}
  {:status :error
   :route  :unknown})
"#;
    (inp, expected)
}

#[fixture]
pub fn complex_cond() -> (&'static str, &'static str) {
    let inp = r#"
(cond
  (= mode :user)
  (let [profile (-> data
                    :user
                    (select-keys [:id :name :email :active?]))
        clean   (-> profile
                                    (update :name #(some-> % clojure.string/trim))
                    (assoc :kind :user))]
    (when-not (:active? clean)
      (println "inactive user"))
    (if (:email clean)
      {:ok true
                            :entity clean
       :meta {:source :user-branch}}
      {:ok false
       :entity clean
       :error :missing-email}))

(= mode :orders)
  (let [orders (->> (:orders data)
                    (filter map?)
                    (map #(select-keys % [:id :total :status]))
                    vec)
        paid   (->> orders
                                (filter #(= :paid (:status %)))
                    count)]
    (if-not (empty? orders)
      {:ok true
       :orders orders
       :stats {:paid-count paid
               :total-count (count orders)}}
      {:ok false
       :orders []
       :stats {:paid-count 0
               :total-count 0}}))

  (= mode :summary)
  (let [summary (cond-> {:source :demo
                         :mode mode}
                                (:debug? data) (assoc :debug true)
                  (:trace-id data) (assoc :trace-id (:trace-id data)))]
    (when-not silent?
                        (println "building summary"))
    {:ok true:summary summary})

  :else
  (let [fallback {:ok false
                  :reason :unsupported-mode
                  :meta {:mode mode}}]
    fallback))
"#;
    let expected = r#"
(cond
  (= mode :user)    (let [profile (-> data
                                      :user
                                      (select-keys [:id :name :email :active?]))
                          clean   (-> profile
                                      (update :name #(some-> % clojure.string/trim))
                                      (assoc :kind :user))]
                      (when-not (:active? clean)
                        (println "inactive user"))
                      (if (:email clean)
                        {:ok     true
                         :entity clean
                         :meta   {:source :user-branch}}
                        {:ok     false
                         :entity clean
                         :error  :missing-email}))

  (= mode :orders)  (let [orders (->> (:orders data)
                                      (filter map?)
                                      (map #(select-keys % [:id :total :status]))
                                      vec)
                          paid   (->> orders
                                      (filter #(= :paid (:status %)))
                                      count)]
                      (if-not (empty? orders)
                        {:ok     true
                         :orders orders
                         :stats  {:paid-count  paid
                                  :total-count (count orders)}}
                        {:ok     false
                         :orders []
                         :stats  {:paid-count  0
                                  :total-count 0}}))

  (= mode :summary) (let [summary (cond-> {:source :demo
                                           :mode   mode}
                                    (:debug? data)   (assoc :debug true)
                                    (:trace-id data) (assoc :trace-id (:trace-id data)))]
                      (when-not silent?
                        (println "building summary"))
                      {:ok true:summary summary})

  :else             (let [fallback {:ok     false
                                    :reason :unsupported-mode
                                    :meta   {:mode mode}}]
                      fallback))
"#;
    (inp, expected)
}

#[fixture]
pub fn correctly_match_form_from_string() -> (Vec<AlignKind>, AlignKind) {
    let inp_cond = r#"
(cond
  (= status :new)        :fresh
  (= status :processing) {:fooo            "aoeuaoeeuaoeuoeauaoeu"
                          :fii             "foeuoeu"
                          :fooo/faaaaaaaaa 4}
  (= status :failed)     :failed
  :else                  :unknown)
"#;
    let inp_cond_thread_first = r#"
(cond->> {:id 42
         :tags [:a :b]
         :active? true
         :premium? true}
  (:active? profile)                          (assoc :status :active)

  (:premium? profile)
  (assoc :tier :gold)

  (seq (:tags profile))
  (update :tags conj :seen))
"#;
    let inp_cond_thread_last = r#"
(cond->> {:id 42
         :tags [:a :b]
         :active? true
         :premium? true}
  (:active? profile)                          (assoc :status :active)

  (:premium? profile)
  (assoc :tier :gold)

  (seq (:tags profile))
  (update :tags conj :seen))
"#;
    let inp_case = r#"
(case       foo
:status                 200
:error  400
201)
"#;
    let results = vec![inp_cond,inp_cond_thread_first, inp_cond_thread_last, inp_case]
        .iter().map(|i| {
            let tree = get_tree(i).unwrap();
            let root = get_root_node(&tree).unwrap();
            let form = root.named_child(0).unwrap();
            find_aligner(form, i).unwrap().kind()
        })
        .collect::<Vec<AlignKind>>(); 
    
    let expected = AlignKind::CondLike;
    (results, expected)
}
