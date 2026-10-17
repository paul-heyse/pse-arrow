// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Selected analytical questions over complete source events. Never controls a solver.
use super::{EvidenceRef, NextObservation, Statement, StatementKind};
use crate::observation::{CoherentRecord, OperationSnapshot, RecordField, Value};
use serde::{Deserialize, Serialize};

/// Explicit operator-selected thresholds, not universal convergence criteria.
/// The enclosing inspection request retains this policy beside its statements.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NumericalHistoryPolicy {
    /// Interpretation version; only version 1 is supported.
    pub version: u16,
    /// Required consecutive source events, at least two.
    pub minimum_events: usize,
    /// Tiny native-step question, in the source's declared norm/coordinates.
    pub tiny_step_maximum: Option<f64>,
    /// Maximum relative reduction in the same native primal measure.
    pub primal_progress_maximum: Option<f64>,
    /// Minimum relative decrease in barrier accompanying a primal-stall question.
    pub barrier_progress_minimum: Option<f64>,
    /// Selected inertia-correction fraction, under one complete factor population.
    pub inertia_fraction_minimum: Option<f64>,
    /// Selected consecutive unaccelerated raw-map residual ratio.
    pub raw_recycle_ratio_minimum: Option<f64>,
    /// Ask whether the same owner-supplied original worst row recurs.
    pub repeated_worst_row: bool,
}
impl NumericalHistoryPolicy {
    fn valid(&self) -> bool {
        self.version == 1 && self.minimum_events >= 2
            && self.tiny_step_maximum.is_none_or(|v| v.is_finite() && v >= 0.0)
            && [self.primal_progress_maximum, self.barrier_progress_minimum, self.inertia_fraction_minimum]
                .into_iter().all(|v| v.is_none_or(|v| v.is_finite() && (0.0..=1.0).contains(&v)))
            && self.raw_recycle_ratio_minimum.is_none_or(|v| v.is_finite() && v >= 0.0)
            && self.primal_progress_maximum.is_some() == self.barrier_progress_minimum.is_some()
    }
}
fn field<'a>(record: &'a CoherentRecord, name: &str) -> Option<&'a RecordField> {
    let mut fields = record.values.iter().filter(|f| f.name == name);
    let field = fields.next()?;
    (field.availability.is_none() && fields.next().is_none()).then_some(field)
}
fn text<'a>(record: &'a CoherentRecord, name: &str) -> Option<&'a str> {
    match &field(record, name)?.value { Value::Text(v) => Some(v), _ => None }
}
fn number(record: &CoherentRecord, name: &str) -> Option<f64> {
    match field(record, name)?.value { Value::Number(v) if v.is_finite() => Some(v), _ => None }
}
fn count(record: &CoherentRecord, name: &str) -> Option<u64> {
    // Native integer fields retain their signed source representation. A
    // checked nonnegative projection is sufficient for exact sequence/count
    // questions; negative native sentinels never become unsigned observations.
    match field(record, name)?.value {
        Value::Count(v) => Some(v),
        Value::Integer(v) => u64::try_from(v).ok(),
        _ => None,
    }
}
fn numeric_field<'a>(record: &'a CoherentRecord, ipopt: &'a str, pounce: &'a str) -> Option<&'a str> {
    match text(record, "backend")? { "ipopt" => Some(ipopt), "pounce" => Some(pounce), _ => None }
}
fn statement(snapshot: &OperationSnapshot, records: &[&CoherentRecord], field_name: Option<&str>, kind: StatementKind, code: &str, message: String, partial: bool) -> Statement {
    Statement {
        kind,
        code: format!("policy.native-progress.v1.{code}"),
        message,
        evidence: records.iter().map(|record| EvidenceRef {
            operation: snapshot.id, producer: Some(record.producer), record_sequence: Some(record.sequence),
            metric: field_name.map(str::to_owned), observed_at: Some(record.observed_at), generation: Some(record.generation),
            value: field_name.and_then(|name| field(record, name)).map(|field| field.value.clone()), partial,
        }).collect(),
        next: if kind == StatementKind::Hypothesis { vec![NextObservation::TimelineCapture, NextObservation::InspectRelated] } else { vec![] },
    }
}
fn same_source(a: &CoherentRecord, b: &CoherentRecord) -> bool {
    a.producer == b.producer && a.generation == b.generation && a.family == b.family
        && a.coordinates == b.coordinates && a.norm == b.norm
        && text(a, "backend") == text(b, "backend")
        && text(a, "native_phase") == text(b, "native_phase")
        && text(a, "checkpoint_kind") == text(b, "checkpoint_kind")
        && text(a, "iterate_semantics") == text(b, "iterate_semantics")
        && ["step_norm_kind", "step_coordinates", "step_semantics"].iter()
            .all(|name| text(a,name)==text(b,name))
        && a.observed_at <= b.observed_at
        && a.family_sequence.zip(b.family_sequence).is_some_and(|(a,b)| a.checked_add(1) == Some(b))
        && a.sequence < b.sequence
}
fn consistent_fields(records: &[&CoherentRecord], names: &[&str]) -> bool {
    names.iter().all(|name| records.first().and_then(|r| field(r,name)).is_some_and(|first| {
        !first.unit.is_empty() && records.iter().all(|r| field(r,name).is_some_and(|f| f.unit == first.unit))
    }))
}
/// Answer selected analytical questions using only retained immutable source evidence.
/// Prefix loss is explicit; a gap inside the selected suffix refuses its classification.
/// A hypothesis never grants convergence, scientific result permission or route changes.
pub fn explain_numerical_history(snapshot: &OperationSnapshot, policy: &NumericalHistoryPolicy) -> Vec<Statement> {
    if !policy.valid() {
        return vec![Statement::operation(snapshot, StatementKind::Limitation, "policy.invalid", "The selected native-progress policy version/thresholds are invalid; no numerical classification was performed.", vec![])];
    }
    let mut output = Vec::new();
    for producer in &snapshot.producers {
        let Some(current) = &producer.current else { continue; };
        let partial = producer.coverage.partial || snapshot.coverage.curtailed || snapshot.coverage.dropped_updates != 0;
        if text(current,"native_phase") == Some("restoration") {
            output.push(statement(snapshot, &[current], Some("native_phase"), StatementKind::Established, "restoration.observed", "The native owner reported restoration at this checkpoint. Its origin and cause require an observed transition and original-coordinate evidence.".into(), partial));
        }
        let mut records: Vec<_> = producer.recent.iter().filter(|r| r.family == current.family).collect();
        if records.last().is_none_or(|r| r.sequence != current.sequence) { records.push(current); }
        let Some(start) = records.len().checked_sub(policy.minimum_events) else {
            output.push(statement(snapshot, &[current], None, StatementKind::Limitation, "history.insufficient", format!("The selected policy requires {} consecutive source events; this view retains fewer. Current progress is still available.", policy.minimum_events), true));
            continue;
        };
        let records = &records[start..];
        if !records.windows(2).all(|w| same_source(w[0],w[1])) {
            output.push(statement(snapshot, &[current], None, StatementKind::Limitation, "history.incompatible", "The selected suffix crosses a source gap, generation, phase, coordinate/norm or native interpretation boundary. It cannot establish consecutive behavior.".into(), true));
            continue;
        }
        let native_iterations = text(current,"checkpoint_kind") == Some("native_iteration")
            && text(current,"iterate_semantics") == Some("current_native_iterate")
            && text(current,"backend").is_some()
            && records.windows(2).all(|w| count(w[0],"iteration").zip(count(w[1],"iteration")).is_some_and(|(a,b)| a.checked_add(1)==Some(b)));
        if native_iterations {
            if let Some(threshold) = policy.tiny_step_maximum {
                if let Some(name) = numeric_field(current,"step.norm","d_norm") {
                    let qualified = text(current,"step_norm_kind").is_some()
                        && text(current,"step_coordinates").is_some()
                        && text(current,"step_semantics")==Some("search_direction")
                        && consistent_fields(records,&[name])
                        && records.iter().all(|r| matches!(field(r,"step_available").map(|f| &f.value),Some(Value::Boolean(true)))
                            && number(r,name).is_some_and(|v| v>=0.0));
                    if qualified {
                        if records.iter().all(|r| number(r,name).is_some_and(|v| v<=threshold)) {
                            output.push(statement(snapshot, records, Some(name), StatementKind::Hypothesis, "tiny_step", format!("All {} consecutive native search-direction measures are at most {threshold} in {:?}/{:?}. Inspect scaling or constraints; this is not an alpha-scaled accepted step or physical progress.",records.len(),text(current,"step_coordinates"),text(current,"step_norm_kind")),partial));
                        }
                    } else {
                        output.push(statement(snapshot, &[current], None, StatementKind::Limitation, "tiny_step.unqualified", "The selected tiny-step question lacks complete supplied search directions with established field-specific norm/coordinates. Initial or unknown direction placeholders are not small-step evidence.".into(),partial));
                    }
                }
            }
            if let (Some(primal_max),Some(barrier_min),Some(primal_name),Some(barrier_name)) = (policy.primal_progress_maximum,policy.barrier_progress_minimum,numeric_field(current,"primal.native","inf_pr"),numeric_field(current,"barrier","mu")) {
                if consistent_fields(records,&[primal_name,barrier_name]) {
                    let first=records[0]; let last=records[records.len()-1];
                    if let (Some(p0),Some(p1),Some(b0),Some(b1)) = (number(first,primal_name),number(last,primal_name),number(first,barrier_name),number(last,barrier_name)) {
                        if p0 > 0.0 && p1 >= 0.0 && b0 > 0.0 && b1 >= 0.0 && records.iter().all(|r| number(r,primal_name).is_some() && number(r,barrier_name).is_some()) {
                            let primal=(p0-p1)/p0; let barrier=(b0-b1)/b0;
                            if primal <= primal_max && barrier >= barrier_min {
                                output.push(statement(snapshot, records, Some(primal_name), StatementKind::Hypothesis, "barrier_primal", format!("Across {} consecutive same-phase native checkpoints, barrier relative decrease {barrier} meets selected minimum {barrier_min}, while primal endpoint decrease {primal} is at most {primal_max}. This is an endpoint-based native-measure question, not continuous stagnation or original physical feasibility.",records.len()),partial));
                            }
                        }
                    }
                }
            }
        }
        if let Some(threshold)=policy.inertia_fraction_minimum {
            if let (Some(factors),Some(corrections))=(count(current,"factorizations"),count(current,"inertia_corrections")) {
                if factors > 0 && corrections <= factors && !partial && text(current,"inertia_population") == Some("corrected_factorizations_over_factorizations") {
                    let ratio=corrections as f64/factors as f64;
                    if ratio >= threshold { output.push(statement(snapshot,&[current],Some("inertia_corrections"),StatementKind::Hypothesis,"inertia_fraction",format!("The source's complete common factor population has correction fraction {ratio}, at least the selected {threshold}. Inspect scaling/model curvature before choosing a numerical intervention."),false)); }
                } else { output.push(statement(snapshot,&[current],None,StatementKind::Limitation,"inertia.unavailable","The supplied factor/correction counts do not establish a complete common corrected-factorization population.".into(),true)); }
            } else { output.push(statement(snapshot,&[current],None,StatementKind::Limitation,"inertia.unavailable","A common complete factor/correction population is unavailable; no inertia fraction is inferred from timing or failure counts.".into(),true)); }
        }
        if policy.repeated_worst_row {
            if let Some(source) = text(current,"worst_row.original") {
                if records.iter().all(|r| text(r,"worst_row.original") == Some(source)) {
                    output.push(statement(snapshot,records,Some("worst_row.original"),StatementKind::Hypothesis,"repeated_worst_row",format!("The same supplied original worst-row source {source} recurs at {} consecutive checkpoints. This identifies a repeated owner observation, not its cause or a new evaluation of original equations.",records.len()),partial));
                }
            } else { output.push(statement(snapshot,&[current],None,StatementKind::Limitation,"worst_row.unavailable","The native checkpoint has no qualified original-row pullback. Passive inspection did not evaluate equations to manufacture one.".into(),true)); }
        }
        if let Some(threshold)=policy.raw_recycle_ratio_minimum {
            if text(current,"iterate_semantics")==Some("unaccelerated_raw_map") && current.norm.is_some() && consistent_fields(records,&["raw_map.residual"]) {
                let ratios: Option<Vec<_>>=records.windows(2).map(|w| { let a=number(w[0],"raw_map.residual")?;let b=number(w[1],"raw_map.residual")?;(a>0.0 && b>=0.0).then_some(b/a) }).collect();
                if ratios.as_ref().is_some_and(|r| r.iter().all(|v| v.is_finite() && *v>=threshold)) { output.push(statement(snapshot,records,Some("raw_map.residual"),StatementKind::Hypothesis,"raw_recycle_ratio",format!("Consecutive observed unaccelerated raw-map residual ratios are at least selected {threshold}. These local observations are not a contraction constant or evidence about accelerated iterates."),partial)); }
            }
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observation::{ObservationHub, ProducerRole};
    use std::time::Duration;

    fn policy() -> NumericalHistoryPolicy {
        NumericalHistoryPolicy { version: 1, minimum_events: 3, tiny_step_maximum: Some(1e-6), primal_progress_maximum: Some(0.01), barrier_progress_minimum: Some(0.5), inertia_fraction_minimum: None, raw_recycle_ratio_minimum: None, repeated_worst_row: false }
    }
    fn observed(norm: Option<&str>, rows: &[(f64,f64,f64)]) -> OperationSnapshot {
        observed_iterations(norm, rows, |i| Value::Count(i as u64 + 1))
    }
    fn observed_iterations(norm: Option<&str>, rows: &[(f64,f64,f64)], iteration: impl Fn(usize) -> Value) -> OperationSnapshot {
        let hub=ObservationHub::new(Default::default());
        let op=hub.start("policy.control");
        let producer=op.context().producer(ProducerRole::NativeIteration,"actual.owner");
        for (i,(step,primal,barrier)) in rows.iter().enumerate() {
            let values=[
                ("backend",Value::Text("ipopt".into()),"backend"),
                ("checkpoint_kind",Value::Text("native_iteration".into()),"interpretation"),
                ("iterate_semantics",Value::Text("current_native_iterate".into()),"interpretation"),
                ("native_phase",Value::Text("main".into()),"phase"),
                ("iteration",iteration(i),"count"),
                ("step_norm_kind",Value::Text(norm.unwrap_or("").into()),"norm"),
                ("step_coordinates",Value::Text("native_primal_and_slack".into()),"coordinates"),
                ("step_semantics",Value::Text("search_direction".into()),"interpretation"),
                ("step_available",Value::Boolean(norm.is_some()),"availability"),
                ("step.norm",Value::Number(*step),"normalized_step"),
                ("primal.native",Value::Number(*primal),"native_primal"),
                ("barrier",Value::Number(*barrier),"native_barrier"),
            ].into_iter().map(|(name,value,unit)|RecordField { name:name.into(),unit:unit.into(),value,availability:None }).collect();
            let record=producer.record("native_nlp.v1","iterate",Some("normalized"),norm,values);
            assert!(producer.publish(&record));
        }
        op.reader().snapshot(Duration::from_secs(60))
    }
    #[test]
    fn selected_policy_reports_consecutive_small_steps_as_hypothesis_with_exact_sources() {
        let snapshot=observed(Some("infinity"),&[(1e-8,1.0,0.1),(2e-8,1.0,0.01),(3e-8,1.0,0.001)]);
        let statements=explain_numerical_history(&snapshot,&policy());
        let small=statements.iter().find(|s|s.code.ends_with(".tiny_step")).unwrap();
        assert_eq!(small.kind,StatementKind::Hypothesis);
        assert_eq!(small.evidence.len(),3);
        assert!(small.evidence.windows(2).all(|w|w[0].record_sequence.unwrap()<w[1].record_sequence.unwrap()));
        assert!(small.evidence.iter().all(|e|e.producer.is_some()&&e.observed_at.is_some()));
        assert!(statements.iter().any(|s|s.code.ends_with(".barrier_primal")&&s.message.contains("endpoint-based")));
    }
    #[test]
    fn selected_policy_gap_or_phase_change_refuses_consecutive_classification() {
        for phase in [false,true] {
            let mut snapshot=observed(Some("infinity"),&[(1e-8,1.0,0.1),(1e-8,1.0,0.01),(1e-8,1.0,0.001)]);
            let middle=&mut snapshot.producers[0].recent[1];
            if phase { middle.values.iter_mut().find(|v|v.name=="native_phase").unwrap().value=Value::Text("restoration".into()); }
            else { middle.family_sequence=Some(99); }
            let statements=explain_numerical_history(&snapshot,&policy());
            assert!(statements.iter().all(|s|s.kind!=StatementKind::Hypothesis));
            assert!(statements.iter().any(|s|s.code.ends_with(".history.incompatible")));
        }
    }
    #[test]
    fn selected_policy_does_not_invent_norm_or_continuous_stagnation() {
        let snapshot=observed(None,&[(1e-8,1.0,0.1),(1e-8,0.1,0.01),(1e-8,1.0,0.001)]);
        let statements=explain_numerical_history(&snapshot,&policy());
        assert!(!statements.iter().any(|s|s.code.ends_with(".tiny_step")));
        let endpoint=statements.iter().find(|s|s.code.ends_with(".barrier_primal")).unwrap();
        assert_eq!(endpoint.kind,StatementKind::Hypothesis);
        assert!(endpoint.message.contains("not continuous stagnation"));
    }
    #[test]
    fn invalid_policy_and_missing_factor_population_remain_limitations() {
        let snapshot=observed(Some("infinity"),&[(0.1,1.0,0.1),(0.1,0.5,0.01),(0.1,0.1,0.001)]);
        let mut selected=policy();selected.version=2;
        assert_eq!(explain_numerical_history(&snapshot,&selected)[0].code,"policy.invalid");
        selected.version=1;selected.inertia_fraction_minimum=Some(0.3);
        assert!(explain_numerical_history(&snapshot,&selected).iter().any(|s|s.code.ends_with(".inertia.unavailable")&&s.kind==StatementKind::Limitation));
    }
    #[test]
    fn selected_policy_initial_placeholder_is_not_small_direction_evidence() {
        let mut snapshot=observed(Some("infinity"),&[(0.0,1.0,0.1),(0.0,1.0,0.01),(0.0,1.0,0.001)]);
        snapshot.producers[0].recent[0].values.iter_mut().find(|f|f.name=="step_available").unwrap().value=Value::Boolean(false);
        let statements=explain_numerical_history(&snapshot,&policy());
        assert!(!statements.iter().any(|s|s.code.ends_with(".tiny_step")));
        assert!(statements.iter().any(|s|s.code.ends_with(".tiny_step.unqualified")));
    }
    #[test]
    fn selected_policy_complete_large_directions_are_not_missing_evidence() {
        let snapshot=observed(Some("infinity"),&[(1.0,1.0,0.1),(0.5,1.0,0.01),(0.1,1.0,0.001)]);
        let statements=explain_numerical_history(&snapshot,&policy());
        assert!(!statements.iter().any(|s|s.code.contains("tiny_step")));
    }

    #[test]
    fn native_signed_iteration_suffix_classifies_tiny_steps_and_barrier_without_changing_source_types() {
        let rows=[(1e-8,1.0,0.1),(2e-8,1.0,0.01),(3e-8,1.0,0.001)];
        // Actual native Metric::Integer fields arrive as Value::Integer. Also
        // exercise exact consecutive values beyond float's integer precision.
        for first in [1, i64::MAX-2] {
            let snapshot=observed_iterations(Some("infinity"),&rows,|i|Value::Integer(first+i as i64));
            let original=serde_json::to_vec(&snapshot).unwrap();
            let statements=explain_numerical_history(&snapshot,&policy());
            for suffix in [".tiny_step",".barrier_primal"] {
                let statement=statements.iter().find(|statement|statement.code.ends_with(suffix)).unwrap();
                assert_eq!(statement.kind,StatementKind::Hypothesis);
                assert_eq!(statement.evidence.len(),3);
                assert!(statement.evidence.iter().all(|evidence| evidence.producer.is_some()&&evidence.record_sequence.is_some()));
            }
            assert!(snapshot.producers[0].recent.iter().all(|record|matches!(field(record,"iteration").unwrap().value,Value::Integer(_))));
            assert_eq!(serde_json::to_vec(&snapshot).unwrap(),original,"inspection does not relabel signed native facts into a counter ledger");
        }
    }
    #[test]
    fn negative_or_unavailable_native_integer_is_not_a_consecutive_iteration_suffix() {
        let rows=[(1e-8,1.0,0.1),(1e-8,1.0,0.01),(1e-8,1.0,0.001)];
        for first in [-1,i64::MIN] {
            let snapshot=observed_iterations(Some("infinity"),&rows,|i|Value::Integer(first+i as i64));
            let statements=explain_numerical_history(&snapshot,&policy());
            assert!(!statements.iter().any(|statement|statement.code.ends_with(".tiny_step")||statement.code.ends_with(".barrier_primal")));
        }
        let mut snapshot=observed_iterations(Some("infinity"),&rows,|i|Value::Integer(i as i64+1));
        snapshot.producers[0].recent[1].values.iter_mut().find(|field|field.name=="iteration").unwrap().availability=
            Some(crate::observation::RecordAvailability::NotSupplied);
        assert!(!explain_numerical_history(&snapshot,&policy()).iter().any(|statement|statement.kind==StatementKind::Hypothesis));
    }
    #[test]
    fn native_integer_recognition_does_not_bypass_step_availability_or_field_norm_contracts() {
        let rows=[(1e-8,1.0,0.1),(1e-8,1.0,0.01),(1e-8,1.0,0.001)];
        for missing_norm in [false,true] {
            let mut snapshot=observed_iterations(Some("infinity"),&rows,|i|Value::Integer(i as i64+1));
            let producer=&mut snapshot.producers[0];
            for record in producer.recent.iter_mut().chain(producer.current.iter_mut()) {
                if missing_norm { record.values.retain(|field|field.name!="step_norm_kind"); }
                else { record.values.iter_mut().find(|field|field.name=="step_available").unwrap().value=Value::Boolean(false); }
            }
            let statements=explain_numerical_history(&snapshot,&policy());
            assert!(!statements.iter().any(|statement|statement.code.ends_with(".tiny_step")));
            assert!(statements.iter().any(|statement|statement.code.ends_with(".tiny_step.unqualified")));
            assert!(statements.iter().any(|statement|statement.code.ends_with(".barrier_primal")),
                "the independent native primal/barrier question keeps its own complete source requirements");
        }
    }

}
