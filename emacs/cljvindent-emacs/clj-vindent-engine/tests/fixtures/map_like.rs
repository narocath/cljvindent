use rstest::fixture;
use clj_vindent_engine::{engine::find_aligner, AlignKind, helpers::{get_tree, get_root_node}};

#[fixture]
pub fn simple_nested_map() -> (&'static str, &'static str){
    let inp = r#"
{:user {:settings {:meta            "Alice"
          :stats 42}
    :profile {:address          "Paris"
                  :preferences true}}
 :logs {:items {:data                   false}}}
"#;
    let expected = r#"
{:user {:settings {:meta  "Alice"
                   :stats 42}
        :profile  {:address     "Paris"
                   :preferences true}}
 :logs {:items {:data false}}}
"#;
    (inp, expected)
}

#[fixture]
pub fn deep_nested_map() -> (&'static str, &'static str){
    let inp = r#"
{:app {:config {:env     "dev"
                :version "0.0.1"
                :features
                {:auth {:enabled true
                  :providers
                  {:google {:client-id     "dummy-google-id"
                            :client-secret "dummy-google-secret"}
                   :github {:client-id     "dummy-github-id"
                            :client-secret "dummy-github-secret"}}}
                 :payments {:enabled false
                            :stripe  {:public-key "pk_test_dummy"
                                      :secret-key "sk_test_dummy"}}}}
       :tenants {:tenant-001
   {:name "Dummy Corp"
    :regions {:eu-west {:datacenter {:id "dc-01"
                                     :clusters {:cluster-a {:nodes {:node-1 {:status "active"
           :services {:api
            {:port   8080
             :health {:status "ok"
                      :checks {:db    true
                               :redis true
                               :disk  true}}}
            :worker {:port 9090
             :queues {:emails {:pending 12
               :last-job {:id "job-1001"
                :payload {:recipient "test@example.com"
                 :template  "welcome"
                 :meta      {:attempt 1
                             :source  "seed-data"}}}}
              :reports  {:pending 3
               :last-job {:id "job-1002"
                :payload {:range  {:from "2026-01-01"
                          :to   "2026-01-31"}
                 :format "pdf"
                 :meta   {:attempt 2
                          :source  "seed-data"}}}}}}}
           :node-2 {:status "standby"
            :services {:api {:port   8081
              :health {:status "warn"
                       :checks {:db    true
                                :redis false
                                :disk  true}}}}}}}}}}}
     :tenant-002 {:name "Example Industries"
      :regions {:us-east {:datacenter {:id "dc-02"
         :clusters {:cluster-b {:nodes {:node-9 {:status "active"
             :services {:analytics {:port 7070
               :pipelines {:daily-sync {:enabled  true
                 :schedule "0 0 * * *"
                 :last-run {:started-at  "2026-04-09T00:00:00Z"
                  :finished-at "2026-04-09T00:14:32Z"
                  :result {:status "success"
                   :stats {:records  10452
                    :errors   0
                    :warnings 2
                    :details {:warning-1 {:code  "MISSING_OPTIONAL_FIELD"
                      :count 17}
                     :warning-2 {:code  "DEPRECATED_SCHEMA_FIELD"
                      :count 4}}}}}}}}}}}}}}}}}
     :users {:admin {:id 1
       :profile {:name  "Admin User"
        :email "admin@example.com"
        :preferences {:theme "dark"
         :notifications {:email true
          :sms   false
          :push {:enabled true
           :channels {:system    {:enabled true}
            :marketing {:enabled false}
            :security {:enabled true
             :rules {:login           {:email true :push true}
              :password-change {:email true :push true}
              :device-approval {:email true :push true}}}}}}}
        :sessions {:current {:token      "dummy-token"
          :created-at "2026-04-10T10:00:00Z"
          :context {:ip "127.0.0.1"
           :device {:type "laptop"
            :os {:family  "macos"
             :version "14.0"
             :build   {:major 23
                       :minor 1
                       :patch 0}}}}}}}}
      :meta {:generated true
       :source    "dummy-data"
       :checksum  "abc123xyz"
       :notes {:purpose "testing"
        :safe    true
        :nested {:level-1 {:level-2 {:level-3 {:level-4 {:level-5 {:level-6 {:level-7 {:level-8 {:level-9
{:level-10 "very deep value"}}}}}}}}}}}}}}}}}}
"#;
    let expected = r#"
{:app {:config  {:env      "dev"
                 :version  "0.0.1"
                 :features {:auth     {:enabled   true
                                       :providers {:google {:client-id     "dummy-google-id"
                                                            :client-secret "dummy-google-secret"}
                                                   :github {:client-id     "dummy-github-id"
                                                            :client-secret "dummy-github-secret"}}}
                            :payments {:enabled false
                                       :stripe  {:public-key "pk_test_dummy"
                                                 :secret-key "sk_test_dummy"}}}}
       :tenants {:tenant-001 {:name    "Dummy Corp"
                              :regions {:eu-west    {:datacenter {:id       "dc-01"
                                                                  :clusters {:cluster-a {:nodes {:node-1 {:status   "active"
                                                                                                          :services {:api    {:port   8080
                                                                                                                              :health {:status "ok"
                                                                                                                                       :checks {:db    true
                                                                                                                                                :redis true
                                                                                                                                                :disk  true}}}
                                                                                                                     :worker {:port   9090
                                                                                                                              :queues {:emails  {:pending  12
                                                                                                                                                 :last-job {:id      "job-1001"
                                                                                                                                                            :payload {:recipient "test@example.com"
                                                                                                                                                                      :template  "welcome"
                                                                                                                                                                      :meta      {:attempt 1
                                                                                                                                                                                  :source  "seed-data"}}}}
                                                                                                                                       :reports {:pending  3
                                                                                                                                                 :last-job {:id      "job-1002"
                                                                                                                                                            :payload {:range  {:from "2026-01-01"
                                                                                                                                                                               :to   "2026-01-31"}
                                                                                                                                                                      :format "pdf"
                                                                                                                                                                      :meta   {:attempt 2
                                                                                                                                                                               :source  "seed-data"}}}}}}}
                                                                                                          :node-2   {:status   "standby"
                                                                                                                     :services {:api {:port   8081
                                                                                                                                      :health {:status "warn"
                                                                                                                                               :checks {:db    true
                                                                                                                                                        :redis false
                                                                                                                                                        :disk  true}}}}}}}}}}}
                                        :tenant-002 {:name    "Example Industries"
                                                     :regions {:us-east {:datacenter {:id       "dc-02"
                                                                                      :clusters {:cluster-b {:nodes {:node-9 {:status   "active"
                                                                                                                              :services {:analytics {:port      7070
                                                                                                                                                     :pipelines {:daily-sync {:enabled  true
                                                                                                                                                                              :schedule "0 0 * * *"
                                                                                                                                                                              :last-run {:started-at  "2026-04-09T00:00:00Z"
                                                                                                                                                                                         :finished-at "2026-04-09T00:14:32Z"
                                                                                                                                                                                         :result      {:status "success"
                                                                                                                                                                                                       :stats  {:records  10452
                                                                                                                                                                                                                :errors   0
                                                                                                                                                                                                                :warnings 2
                                                                                                                                                                                                                :details  {:warning-1 {:code  "MISSING_OPTIONAL_FIELD"
                                                                                                                                                                                                                                       :count 17}
                                                                                                                                                                                                                           :warning-2 {:code  "DEPRECATED_SCHEMA_FIELD"
                                                                                                                                                                                                                                       :count 4}}}}}}}}}}}}}}}}}
                                        :users      {:admin {:id      1
                                                             :profile {:name        "Admin User"
                                                                       :email       "admin@example.com"
                                                                       :preferences {:theme         "dark"
                                                                                     :notifications {:email true
                                                                                                     :sms   false
                                                                                                     :push  {:enabled  true
                                                                                                             :channels {:system    {:enabled true}
                                                                                                                        :marketing {:enabled false}
                                                                                                                        :security  {:enabled true
                                                                                                                                    :rules   {:login           {:email true :push true}
                                                                                                                                              :password-change {:email true :push true}
                                                                                                                                              :device-approval {:email true :push true}}}}}}}
                                                                       :sessions    {:current {:token      "dummy-token"
                                                                                               :created-at "2026-04-10T10:00:00Z"
                                                                                               :context    {:ip     "127.0.0.1"
                                                                                                            :device {:type "laptop"
                                                                                                                     :os   {:family  "macos"
                                                                                                                            :version "14.0"
                                                                                                                            :build   {:major 23
                                                                                                                                      :minor 1
                                                                                                                                      :patch 0}}}}}}}}
                                                     :meta  {:generated true
                                                             :source    "dummy-data"
                                                             :checksum  "abc123xyz"
                                                             :notes     {:purpose "testing"
                                                                         :safe    true
                                                                         :nested  {:level-1 {:level-2 {:level-3 {:level-4 {:level-5 {:level-6 {:level-7 {:level-8 {:level-9 {:level-10 "very deep value"}}}}}}}}}}}}}}}}}}
"#;
    (inp, expected)
}

#[fixture]
pub fn nested_map_with_everything() -> (&'static str, &'static str){
    let inp = r#"
{:app
   {:tenant-1 {:event-router
        (fn [event]
          (let [kind (:kind event)
                status (:status event)
                payload                        (:payload event)
                tags (:tags event [])
                user-id (:user-id event)
                normalized (cond-> {:id (:id event)
                                                :kind kind
                                    :status status}
                             user-id (assoc :user-id user-id)
                             (seq tags) (assoc :tags (vec tags))
                             (:urgent? event) (assoc :priority :high))]
            (when-let [user (:user event)]
              (println "routing for user" (:name user)))
            (if (:enabled? event)
              {:route
               (case kind
                 :create :create-flow
                                    :update :update-flow
                 :delete :delete-flow
                 :unknown-flow)

               :meta
               (cond
          (= status :new) :demo/fresh
                 (= status :processing) :demo/active
                 (= status :failed) :demo/failed
                 :else :demo/other)

               :payload
               (condp = (:country payload)
                 "GR" :local
                                           "DE" :eu
                 "US" :>> (fn [_] :demo/us-special)
                 "UK" :>> (fn [_] {:region :uk :vat true})
                 :intl)

               :normalized normalized}
              {:route :disabled
               :reason :demo/event-disabled})))

        :batch-builder
        (fn [rows]
          (let [prepared (cond->> rows
                           true (filter map?)
                                           true (remove #(= :deleted (:status %)))
                           true (map #(update % :score (fnil identity 0)))
                           true vec)
                summary (cond
                          (empty? prepared) :empty
                          (= 1 (count prepared)) :single
                          (< (count prepared) 5) :small
                          :else :bulk)]
            {:rows prepared
             :summary summary
             :labels
             (condp = summary
               :empty [:skip]
      :single [:one]
               :small :>> (fn [_] [:few :demo/batched])
               :bulk :>> (fn [_] [:many :demo/batched])
               [:unknown])}))

        :permission-check
        (fn [user action]
          (let [role (:role user)
                          flags (:flags user [])]
            (when (seq flags)
              (println "user-flags" flags))
            {:allowed?
        (condp = [role action]
               [:admin :read] true
               [:admin :write]              true
               [:manager :read]             true
                                [:manager :write] :>> (fn [_] (boolean (:can-write user)))
               [:guest :read] true
               false)

             :reason
             (case action
               :read :demo/read-check
               :write               :demo/write-check
    :delete :demo/delete-check
               :demo/unknown-action)}))

        :maybe-enrich
        (fn [m]
          (when-let [profile (:profile m)]
            (let [country (:country profile)
                  tier (:tier profile)]
              (cond-> m
                country (assoc :country country)
                                tier (assoc :tier tier)
                (= tier :gold) (assoc :discount 20)
                (= tier :silver) (assoc :discount 10)))))

        :shape-response
        (fn [resp]
          (let [base {:ok true
                      :data (:data resp)}]
            (if-not (:skip? resp)
              (cond
                (:error resp)
                {:ok false
                 :code (case (:error resp)
                         :timeout 504
                                   :not-found 404
                         :conflict 409
                         500)}

                (:warning resp)
                     (assoc base :warning (:warning resp))

                :else
                (assoc base :code 200))
              (assoc base :code 204 :skipped true))))

        :config
        {:compile
         (fn [cfg]
           (let [mode (:mode cfg)
                 steps (:steps cfg [])
                 result (cond-> {:mode mode
                                 :step-count (count steps)}
                          (:debug cfg) (assoc :debug true)
                                           (= mode :prod) (assoc :optimizations :full)
                          (= mode :dev) (assoc :optimizations :minimal))]
             (if-not (empty? steps)
               (assoc result :steps (vec steps))
               (assoc result :steps []))))

         :runtime
                                 (fn [cfg]
           (let [hooks (cond->> (:hooks cfg [])
                                       true (filter keyword?)
                          true distinct
                          true vec)]
             (when (:trace? cfg)
               (println "runtime-trace-enabled"))
             {:env
              (condp = (:env cfg)
                :dev :local
                           :stage :>> (fn [_] {:name :stage :level 2})
                :prod :>> (fn [_] {:name :prod :level 3})
                :unknown)

              :hooks hooks}))}}}}
"#;
    let expected = r#"
{:app {:tenant-1 {:event-router     (fn [event]
                                      (let [kind       (:kind event)
                                            status     (:status event)
                                            payload    (:payload event)
                                            tags       (:tags event [])
                                            user-id    (:user-id event)
                                            normalized (cond-> {:id     (:id event)
                                                                :kind   kind
                                                                :status status}
                                                         user-id          (assoc :user-id user-id)
                                                         (seq tags)       (assoc :tags (vec tags))
                                                         (:urgent? event) (assoc :priority :high))]
                                        (when-let [user (:user event)]
                                          (println "routing for user" (:name user)))
                                        (if (:enabled? event)
                                          {:route      (case kind
                                                         :create :create-flow
                                                         :update :update-flow
                                                         :delete :delete-flow
                                                         :unknown-flow)
                                           :meta       (cond
                                                         (= status :new)        :demo/fresh
                                                         (= status :processing) :demo/active
                                                         (= status :failed)     :demo/failed
                                                         :else                  :demo/other)
                                           :payload    (condp = (:country payload)
                                                         "GR" :local
                                                         "DE" :eu
                                                         "US" :>> (fn [_] :demo/us-special)
                                                         "UK" :>> (fn [_] {:region :uk :vat true})
                                                         :intl)
                                           :normalized normalized}
                                          {:route  :disabled
                                           :reason :demo/event-disabled})))
                  :batch-builder    (fn [rows]
                                      (let [prepared (cond->> rows
                                                       true (filter map?)
                                                       true (remove #(= :deleted (:status %)))
                                                       true (map #(update % :score (fnil identity 0)))
                                                       true vec)
                                            summary  (cond
                                                       (empty? prepared)      :empty
                                                       (= 1 (count prepared)) :single
                                                       (< (count prepared) 5) :small
                                                       :else                  :bulk)]
                                        {:rows    prepared
                                         :summary summary
                                         :labels  (condp = summary
                                                    :empty  [:skip]
                                                    :single [:one]
                                                    :small  :>> (fn [_] [:few :demo/batched])
                                                    :bulk   :>> (fn [_] [:many :demo/batched])
                                                    [:unknown])}))
                  :permission-check (fn [user action]
                                      (let [role  (:role user)
                                            flags (:flags user [])]
                                        (when (seq flags)
                                          (println "user-flags" flags))
                                        {:allowed? (condp = [role action]
                                                     [:admin :read]    true
                                                     [:admin :write]   true
                                                     [:manager :read]  true
                                                     [:manager :write] :>> (fn [_] (boolean (:can-write user)))
                                                     [:guest :read]    true
                                                     false)
                                         :reason   (case action
                                                     :read   :demo/read-check
                                                     :write  :demo/write-check
                                                     :delete :demo/delete-check
                                                     :demo/unknown-action)}))
                  :maybe-enrich     (fn [m]
                                      (when-let [profile (:profile m)]
                                        (let [country (:country profile)
                                              tier    (:tier profile)]
                                          (cond-> m
                                            country          (assoc :country country)
                                            tier             (assoc :tier tier)
                                            (= tier :gold)   (assoc :discount 20)
                                            (= tier :silver) (assoc :discount 10)))))
                  :shape-response   (fn [resp]
                                      (let [base {:ok   true
                                                  :data (:data resp)}]
                                        (if-not (:skip? resp)
                                          (cond
                                            (:error resp)   {:ok   false
                                                             :code (case (:error resp)
                                                                     :timeout   504
                                                                     :not-found 404
                                                                     :conflict  409
                                                                     500)}

                                            (:warning resp) (assoc base :warning (:warning resp))

                                            :else           (assoc base :code 200))
                                          (assoc base :code 204 :skipped true))))
                  :config           {:compile (fn [cfg]
                                                (let [mode   (:mode cfg)
                                                      steps  (:steps cfg [])
                                                      result (cond-> {:mode       mode
                                                                      :step-count (count steps)}
                                                               (:debug cfg)   (assoc :debug true)
                                                               (= mode :prod) (assoc :optimizations :full)
                                                               (= mode :dev)  (assoc :optimizations :minimal))]
                                                  (if-not (empty? steps)
                                                    (assoc result :steps (vec steps))
                                                    (assoc result :steps []))))
                                     :runtime (fn [cfg]
                            (let [hooks (cond->> (:hooks cfg [])
                                      true (filter keyword?)
                                      true distinct
                                      true vec)]
                            (when (:trace? cfg)
                            (println "runtime-trace-enabled"))
                            {:env   (condp = (:env cfg)
                                    :dev   :local
                                    :stage :>> (fn [_] {:name :stage :level 2})
                                    :prod  :>> (fn [_] {:name :prod :level 3})
                                    :unknown)
                            :hooks hooks}))}}}}
"#;
    (inp, expected)
}

#[fixture]
pub fn correctly_match_form_from_string() -> (AlignKind, AlignKind) {
    let inp = r#"
{:user {:settings {:meta            "Alice"
          :stats 42}
    :profile {:address          "Paris"
                  :preferences true}}
 :logs {:items {:data                   false}}}
"#;
    let tree = get_tree(inp).unwrap();
    let root = get_root_node(&tree).unwrap();
    let form = root.named_child(0).unwrap();

    let found_aligner = find_aligner(form, inp).unwrap().kind();
    
    let expected = AlignKind::MapLike;
    (found_aligner, expected)
}
