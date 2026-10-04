// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Observation-only benchmark accounting. Inclusive owners are never added together.
use pse_backend_native::solve::{Backend, Metric, SolveReport};
use pse_model::generated::{enums::NumericalEventKind, runtime::solve_strategy_events::Row};
use pse_runtime::{math::solves::Outcome, workflow::ModelingResult};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
struct Count {
    known: u64,
    unknown: u64,
    subtotal: u64,
}
impl Count {
    fn observe(&mut self, value: Option<u64>) {
        match value {
            Some(value) => {
                self.known = self.known.checked_add(1).unwrap();
                self.subtotal = self.subtotal.checked_add(value).unwrap();
            }
            None => self.unknown = self.unknown.checked_add(1).unwrap(),
        }
    }
    fn json(&self) -> Value {
        json!({"total":(self.known>0&&self.unknown==0).then_some(self.subtotal),"known_subtotal":self.subtotal,"known_observations":self.known,"unknown_observations":self.unknown})
    }
}
fn count(value: Option<i64>) -> Option<u64> {
    value.map(|value| u64::try_from(value).expect("generated work observations are nonnegative"))
}
#[derive(Default)]
struct Work {
    attempts: Count,
    evaluations: Count,
    iterations: Count,
    factorizations: Count,
    proof_steps: Count,
}
impl Work {
    fn row(&mut self, row: &Row) {
        self.attempts.observe(count(row.attempts));
        self.evaluations.observe(count(row.evaluations));
        self.iterations.observe(count(row.iterations));
        self.factorizations.observe(count(row.factorizations));
        self.proof_steps.observe(count(row.proof_steps));
    }
    fn missing(&mut self) {
        self.attempts.observe(None);
        self.evaluations.observe(None);
        self.iterations.observe(None);
        self.factorizations.observe(None);
        self.proof_steps.observe(None);
    }
    fn native(&mut self, report: &SolveReport) {
        self.attempts.observe(None);
        self.evaluations.observe(report.evidence.work.evaluations);
        self.iterations.observe(report.evidence.work.iterations);
        self.factorizations
            .observe(report.evidence.work.factorizations);
        self.proof_steps.observe(report.evidence.work.proof_steps);
    }
    fn json(&self) -> Value {
        json!({"attempts":self.attempts.json(),"evaluations":self.evaluations.json(),"iterations":self.iterations.json(),"factorizations":self.factorizations.json(),"proof_steps":self.proof_steps.json()})
    }
}
#[derive(Default)]
pub(super) struct Observations {
    results: u64,
    strategies: u64,
    charged_operations: u64,
    native_reports: u64,
    reused_native_reports: u64,
    ledger: Work,
    native: Work,
    kinsol_refreshes: Count,
    kinsol_initial_reused: Count,
    profiles: BTreeMap<String, Value>,
    unknown_profiles: u64,
    persisted_points: u64,
    persisted_refreshes: Count,
    persisted_initial_reused: Count,
    trajectories: Vec<Value>,
    path_probes: u64,
    path_work: BTreeMap<&'static str, u64>,
    preparations: BTreeMap<&'static str, u64>,
}
impl Observations {
    pub(super) fn modeling(&mut self, result: &ModelingResult, step: usize) {
        self.results = self.results.checked_add(1).unwrap();
        if let Some(trace) = &result.strategy {
            self.rows(&trace.rows(result.run_id, step).unwrap());
        } else {
            self.unobserved_strategy();
        }
        if let Outcome::Native(report) = &result.outcome {
            self.native(report);
        }
    }
    pub(super) fn rows(&mut self, rows: &[Row]) {
        if rows.is_empty() {
            self.unobserved_strategy();
            return;
        }
        self.strategies = self.strategies.checked_add(1).unwrap();
        let mut owners = BTreeSet::new();
        for row in rows {
            if row.kind == NumericalEventKind::Started {
                if let Some(profile) = row.profile_identity {
                    self.profiles
                        .entry(format!("{profile}:{:?}", row.backend))
                        .or_insert_with(|| json!({"identity":profile,"backend":row.backend}));
                } else {
                    self.unknown_profiles = self.unknown_profiles.checked_add(1).unwrap();
                }
            }
            if !matches!(
                row.kind,
                NumericalEventKind::Finished | NumericalEventKind::Abandoned
            ) {
                continue;
            }
            if let Some(owner) = row.charging_owner {
                // A terminal accounting refusal may repeat the same charged operation.
                if !owners.insert((row.run_id, row.step, owner)) {
                    continue;
                }
                self.charged_operations = self.charged_operations.checked_add(1).unwrap();
                self.ledger.row(row);
            } else {
                self.ledger.missing();
            }
            if let Some(events) = &row.path_events {
                for event in events {
                    self.path_probes = self.path_probes.checked_add(1).unwrap();
                    for (name, value) in [
                        ("values", event.values),
                        ("state_actions", event.state_actions),
                        ("parameter_actions", event.parameter_actions),
                        ("curvature_actions", event.curvature_actions),
                        ("factorizations", event.factorizations),
                        ("backsolves", event.backsolves),
                        ("rank_probes", event.rank_probes),
                    ] {
                        let total = self.path_work.entry(name).or_default();
                        *total = total
                            .checked_add(
                                u64::try_from(value)
                                    .expect("actual path probe counters are nonnegative"),
                            )
                            .unwrap();
                    }
                }
            }
        }
    }
    pub(super) fn unobserved_strategy(&mut self) {
        self.ledger.missing();
        self.unknown_profiles = self.unknown_profiles.checked_add(1).unwrap();
    }
    pub(super) fn run(&mut self, result: &pse_runtime::workflow::RunResult) {
        match result.report().unwrap() {
            pse_runtime::workflow::RunReport::Modeling(reports) => {
                for (step, report) in reports.iter().enumerate() {
                    self.modeling(report, step);
                }
            }
            pse_runtime::workflow::RunReport::Fit(report) => {
                if let Some(trace) = &report.strategy {
                    self.rows(&trace.rows(result.run_id, 0).unwrap());
                } else {
                    self.unobserved_strategy();
                }
                if let Some(solve) = &report.solve {
                    self.native(solve);
                }
            }
            _ => self.unobserved_strategy(),
        }
    }
    pub(super) fn trajectory(&mut self, report: &pse_backend_native::dynamics::Report) {
        self.unobserved_strategy();
        self.trajectories.push(json!({"termination":report.termination,"completed_time":report.completed_time,"segment_statistics":report.statistics}));
    }
    pub(super) fn persisted_metrics(
        &mut self,
        rows: &[pse_relations::generated::runtime::solve_metrics::Row],
    ) {
        self.persisted_points = self.persisted_points.checked_add(1).unwrap();
        let metric = |name: &str| {
            rows.iter()
                .find(|row| row.namespace == "metric" && row.name == name)
        };
        self.persisted_refreshes
            .observe(metric("setup.refreshes").and_then(|row| count(row.integer)));
        self.persisted_initial_reused
            .observe(metric("setup.initial_reused").and_then(|row| row.boolean.map(u64::from)));
    }
    pub(super) fn native(&mut self, report: &SolveReport) {
        self.native_reports = self.native_reports.checked_add(1).unwrap();
        self.native.native(report);
        self.reused_native_reports = self
            .reused_native_reports
            .checked_add(u64::from(report.evidence.reused_native_state))
            .unwrap();
        if report.backend == Backend::Kinsol {
            self.kinsol_refreshes
                .observe(match report.metrics.get("setup.refreshes") {
                    Some(Metric::Integer(value)) => Some(
                        u64::try_from(*value)
                            .expect("actual KINSOL setup refresh count is nonnegative"),
                    ),
                    _ => None,
                });
            self.kinsol_initial_reused
                .observe(match report.metrics.get("setup.initial_reused") {
                    Some(Metric::Bool(value)) => Some(u64::from(*value)),
                    _ => None,
                });
        }
    }
    pub(super) fn preparations(
        &mut self,
        before: pse_runtime::math::PreparationCounts,
        after: pse_runtime::math::PreparationCounts,
    ) {
        for (name, before, after) in [
            ("views", before.views, after.views),
            ("observations", before.observations, after.observations),
            ("rebuilt", before.rebuilt, after.rebuilt),
            ("shared", before.shared, after.shared),
        ] {
            let value = u64::try_from(
                after
                    .checked_sub(before)
                    .expect("preparation counters cannot reset within an observed operation"),
            )
            .unwrap();
            let total = self.preparations.entry(name).or_default();
            *total = total.checked_add(value).unwrap();
        }
    }
    pub(super) fn json(&self) -> Value {
        json!({
            "modeling_results":self.results,"observed_strategies":self.strategies,
            "strategy_ledger":{"scope":"actual terminal generated strategy rows; each charging owner counted once per original result, including failed operations","charging_owners":self.charged_operations,"work":self.ledger.json()},
            "persisted_native_metrics":{"scope":"reopened point metrics only; native Evidence work and reuse flag are not persisted here and remain unobserved; do not add to returned report counts","points":self.persisted_points,"native_work":null,"reused_native_state":null,"kinsol_setup_refreshes":self.persisted_refreshes.json(),"kinsol_initial_setup_reused":self.persisted_initial_reused.json()},
            "trajectories":{"scope":"actual segment statistics preserved with their native names; no inferred cross-backend work totals; nested fit trajectories are not exported","observations":self.trajectories},
            "returned_native_reports":{"scope":"inclusive work of returned native reports only; excludes earlier reports visible only in the strategy; do not add to strategy totals","reports":self.native_reports,"work":self.native.json(),"reused_native_state_reports":self.reused_native_reports,"kinsol_setup_refreshes":self.kinsol_refreshes.json(),"kinsol_initial_setup_reused":self.kinsol_initial_reused.json(),"setup_scope":"actual KINSOL setup metrics; refreshes are not factorization counts"},
            "effective_profiles":{"scope":"identities/backend from actual Started strategy rows","observed":self.profiles.values().collect::<Vec<_>>(),"unknown_observations":self.unknown_profiles},
            "path_probe_breakdown":{"scope":"inclusive successful localized probe observations already included in strategy work; do not add to ledger totals; failed or unexported probe breakdowns are unavailable","probes":self.path_probes,"counts":self.path_work},
            "preparations":{"scope":"actual MathService counter deltas per measured operation","counts":self.preparations},
            "aggregation":"totals span collected result observations: main and extended aggregate benchmark invocations including warmup, while each K4 record contains one iteration; null total means no observations or at least one unknown; known subtotals do not qualify unknown work; reporting is included in result observation time"
        })
    }
}
