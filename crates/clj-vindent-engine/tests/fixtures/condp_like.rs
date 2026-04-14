use rstest::fixture;
use clj_vindent_engine::{engine::find_aligner, AlignKind, helpers::{get_tree, get_root_node}};

#[fixture]
pub fn simple_condp() -> (&'static str, &'static str) {
    let inp = r#"
(condp = action
  :read             :allowed

  :write
  :>> (fn [_]
        (let [result {:ok true
                      :action :write}]
          (if admin?
            (assoc result :role :admin)
            result)))

  :delete
(let [m {:ok false
           :action :delete}]
    (if-not protected?
            (assoc m :allowed true)
      (assoc m :reason :protected)))

  :unknown)
"#;
    let expected = r#"
(condp = action
  :read   :allowed

  :write  :>> (fn [_]
                (let [result {:ok     true
                              :action :write}]
                  (if admin?
                    (assoc result :role :admin)
                    result)))

  :delete (let [m {:ok     false
                   :action :delete}]
            (if-not protected?
              (assoc m :allowed true)
              (assoc m :reason :protected)))

  :unknown)
"#;
    (inp, expected)
}

#[fixture]
pub fn complex_condp() -> (&'static str, &'static str) {
    let inp = r#"
(condp = action
  :create
  (let [title (some-> payload
                      :title
                      clojure.string/trim)
        body  (some-> payload
                      :body
                      clojure.string/trim)
        base  {:status :ok
               :action :create
               :payload {:title title
                         :body body}}]
    (if title
      (assoc base :valid true)
      (assoc base :valid false :reason :missing-title)))

  :update
  :>> (fn [_]
        (let [base {:status :ok
                                        :action :update
                    :id id
                    :changes changes}
              normalized (-> base
                             (assoc :seen true)
                             (update :changes #(or % {})))
              enriched   (cond-> normalized
                                            user-id (assoc :user-id user-id)
                           (:admin? opts) (assoc :mode :admin)
                           (not (:admin? opts)) (assoc :mode :user))]
          (when-not (:silent? opts)
            (println "updating record" id))
          enriched))

  :delete
  :>> (fn [_]
        (let [ids (->> items
                       (filter map?)
                                                (map :id)
                       (remove nil?)
                       vec)
              result {:status :ok
                      :action :delete
                      :ids ids}]
          (if-not (empty? ids)
            result
            (assoc result :status :noop :reason :no-ids))))

  :archive
  (let [entry {:status :ok
                                :action :archive
               :meta {:source :dummy
                      :request-id request-id}}]
    (when-not dry-run?
                (println "archiving" request-id))
    (if flagged?
      (assoc entry :flag :manual-review)
      entry))

  :publish
:>> (fn [_]
        (let [doc (-> payload
                      (assoc :published true)
                      (update :tags (fnil vec [])))
              preview (->> (:tags doc)
                           (filter keyword?)
                           (map name)
                           vec)]
          {:status :ok
           :action :publish
           :doc doc
           :preview preview}))

                :sync
  (let [jobs (->> queue
                  (filter map?)
                  (map #(select-keys % [:id :kind :status]))
                  vec)
        summary (cond
                  (empty? jobs) :empty
                  (= 1 (count jobs)) :single
                  :else :many)]
    {:status :ok
     :action :sync
     :jobs jobs
     :summary summary})

            :export
  :>> (fn [_]
        (let [base {:status :ok
                    :action :export
                    :format format}
              export-map (cond-> base
                           (= format :csv) (assoc :content-type "text/csv")
                           (= format :json) (assoc :content-type "application/json")
                           path (assoc :path path))]
          (if (:compressed? opts)
            (assoc export-map :compressed true)
            export-map)))

  {:status :error
   :action :unknown
   :input action})
"#;
    let expected = r#"
(condp = action
  :create  (let [title (some-> payload
                               :title
                               clojure.string/trim)
                 body  (some-> payload
                               :body
                               clojure.string/trim)
                 base  {:status  :ok
                        :action  :create
                        :payload {:title title
                                  :body  body}}]
             (if title
               (assoc base :valid true)
               (assoc base :valid false :reason :missing-title)))

  :update  :>> (fn [_]
                 (let [base       {:status  :ok
                                   :action  :update
                                   :id      id
                                   :changes changes}
                       normalized (-> base
                                      (assoc :seen true)
                                      (update :changes #(or % {})))
                       enriched   (cond-> normalized
                                    user-id              (assoc :user-id user-id)
                                    (:admin? opts)       (assoc :mode :admin)
                                    (not (:admin? opts)) (assoc :mode :user))]
                   (when-not (:silent? opts)
                     (println "updating record" id))
                   enriched))

  :delete  :>> (fn [_]
                 (let [ids    (->> items
                                   (filter map?)
                                   (map :id)
                                   (remove nil?)
                                   vec)
                       result {:status :ok
                               :action :delete
                               :ids    ids}]
                   (if-not (empty? ids)
                     result
                     (assoc result :status :noop :reason :no-ids))))

  :archive (let [entry {:status :ok
                        :action :archive
                        :meta   {:source     :dummy
                                 :request-id request-id}}]
             (when-not dry-run?
               (println "archiving" request-id))
             (if flagged?
               (assoc entry :flag :manual-review)
               entry))

  :publish :>> (fn [_]
                   (let [doc     (-> payload
                                     (assoc :published true)
                                     (update :tags (fnil vec [])))
                         preview (->> (:tags doc)
                                      (filter keyword?)
                                      (map name)
                                      vec)]
                     {:status  :ok
                      :action  :publish
                      :doc     doc
                      :preview preview}))

  :sync    (let [jobs    (->> queue
                              (filter map?)
                              (map #(select-keys % [:id :kind :status]))
                              vec)
                 summary (cond
                           (empty? jobs)      :empty
                           (= 1 (count jobs)) :single
                           :else              :many)]
             {:status  :ok
              :action  :sync
              :jobs    jobs
              :summary summary})

  :export  :>> (fn [_]
                 (let [base       {:status :ok
                                   :action :export
                                   :format format}
                       export-map (cond-> base
                                    (= format :csv)  (assoc :content-type "text/csv")
                                    (= format :json) (assoc :content-type "application/json")
                                    path             (assoc :path path))]
                   (if (:compressed? opts)
                     (assoc export-map :compressed true)
                     export-map)))

  {:status :error
   :action :unknown
   :input  action})
"#;
    (inp, expected)
}

#[fixture]
pub fn correctly_match_form_from_string() -> (AlignKind, AlignKind) {
    let inp = r#"
(condp = action
  :read             :allowed

  :write
  :>> (fn [_]
        (let [result {:ok true
                      :action :write}]
          (if admin?
            (assoc result :role :admin)
            result)))

  :delete
(let [m {:ok false
           :action :delete}]
    (if-not protected?
            (assoc m :allowed true)
      (assoc m :reason :protected)))

  :unknown)
"#;
    let tree = get_tree(inp).unwrap();
    let root = get_root_node(&tree).unwrap();
    let form = root.named_child(0).unwrap();

    let found_aligner = find_aligner(form, inp).unwrap().kind();
    
    let expected = AlignKind::CondPLike;
    (found_aligner, expected)
}
