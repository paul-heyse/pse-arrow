// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bridge actual source-owned native operations to task admission and observations.
use crate::{
    ProblemError, SolveSeparator,
    solve::{Execution, Metric, SolveReport, WorkEvidence},
};
use pounce_common::observed::{Abort, Event, Layout, Observer, Path, Primitive};
use std::cell::RefCell;
#[derive(Clone, Copy)]
struct Call {
    ordinal: Option<u64>,
    parent: Option<u64>,
    path: Path,
    primitive: Primitive,
    rhs: usize,
    refinement_bound: usize,
}
#[derive(Default)]
struct State {
    phases: [[Vec<pse_diagnostics::observation::PhaseGuard>; 2]; 3],
    layout: Option<Layout>,
    separator: Option<Vec<usize>>,
    factors: [u64; 3],
    failed_factors: [u64; 3],
    pending_factors: [u64; 3],
    solves: [u64; 3],
    failed_solves: [u64; 3],
    path: Option<(bool, &'static str)>,
    known_storage: usize,
    storage_owners: std::collections::BTreeMap<(String, usize), usize>,
    opaque_storage: bool,
    application_opaque: bool,
    linear_seen: bool,
    error: Option<ProblemError>,
    next_call: u64,
    call_overflow: bool,
    next_detail: u64,
    detail_overflow: bool,
    calls: Vec<Call>,
}
pub(super) struct Observation {
    execution: Execution,
    separator: Option<SolveSeparator>,
    allowance: Option<usize>,
    state: RefCell<State>,
    detail: Option<pse_diagnostics::observation::Producer>,
}
fn detail_family(family: &str) -> &str {
    match family {
        "native_factor.v1" => "native_factor.v2",
        "native_refinement.v1" => "native_refinement.v2",
        _ => family,
    }
}
fn index(path: Path) -> usize {
    match path {
        Path::Monolithic => 0,
        Path::SchurF => 1,
        Path::SchurS => 2,
    }
}
impl Observation {
    pub(super) fn new(
        execution: Execution,
        separator: Option<SolveSeparator>,
        allowance: Option<usize>,
    ) -> Self {
        Self {
            execution,
            separator,
            allowance,
            state: RefCell::new(State::default()),
            detail: pse_diagnostics::observation::current().map(|context| {
                let producer = context.producer(
                    pse_diagnostics::observation::ProducerRole::NativeFactor,
                    "pounce.linear.detail",
                );
                for family in [
                    "native_linear_call.v1",
                    "native_factor.v2",
                    "native_refinement.v2",
                ] {
                    producer.activate_record_family(family);
                }
                producer
            }),
        }
    }
    fn fail(&self, error: ProblemError) -> Abort {
        let abort = match crate::callback::classify(&error) {
            crate::callback::Failure::Stopped(crate::solve::Termination::Cancelled) => {
                Abort::Cancelled
            }
            crate::callback::Failure::Stopped(_) => Abort::Resource(error.to_string()),
            _ => Abort::Contract(error.to_string()),
        };
        self.state.borrow_mut().error = Some(error);
        abort
    }
    pub(super) fn take_error(&self) -> Option<ProblemError> {
        self.state.borrow_mut().error.take()
    }
    /// Reconcile begun factors when native unwinding prevents the matching End event.
    pub(super) fn finish_operations(&self) {
        for path in &mut self.state.borrow_mut().phases {
            for phases in path {
                phases.clear();
            }
        }
        self.state.borrow_mut().calls.clear();
        let pending = std::mem::take(&mut self.state.borrow_mut().pending_factors);
        let count: u64 = pending.iter().sum();
        if count == 0 {
            return;
        }
        if let Some(admission) = &self.execution.work_admission
            && let Err(error) = crate::quality::contained(|| {
                admission.observe(WorkEvidence {
                    evaluations: Some(0),
                    iterations: Some(0),
                    factorizations: Some(count),
                    proof_steps: Some(0),
                })
            })
        {
            self.fail(error);
        }
        let mut state = self.state.borrow_mut();
        for (index, (failed, pending)) in state.failed_factors.iter_mut().zip(pending).enumerate() {
            *failed += pending;
            crate::observation::count(
                &format!(
                    "pounce.linear.{}.factor.interrupted",
                    ["monolithic", "schur_f", "schur_s"][index]
                ),
                pending,
            );
        }
    }
    fn call_detail(
        &self,
        stage: &'static str,
        succeeded: Option<bool>,
        rhs: usize,
        refinement_bound: usize,
    ) {
        use pounce_common::observed::{DetailField as Field, DetailValue as Value};
        // Optional publication must never trip the compulsory native admission latch.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            if !self.wants_detail("native_linear_call.v1") {
                return;
            }
            let mut fields = vec![
                Field {
                    name: "stage",
                    value: Value::Text(stage),
                },
                Field {
                    name: "rhs",
                    value: Value::Count(rhs as u64),
                },
                Field {
                    name: "refinement_bound",
                    value: Value::Count(refinement_bound as u64),
                },
            ];
            if let Some(succeeded) = succeeded {
                fields.push(Field {
                    name: "succeeded",
                    value: Value::Flag(succeeded),
                });
            }
            self.detail("native_linear_call.v1", &fields);
        }));
        if result.is_err() {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                self.detail_failed("native_linear_call.v1")
            }));
        }
    }
    pub(super) fn record(&self, report: &mut SolveReport) {
        let state = self.state.borrow();
        report.evidence.work.factorizations = Some(state.factors.iter().sum());
        report.metrics.insert(
            "linear.observed.factor_scope".into(),
            Metric::Text(
                "FERAL Solver::factor invocations; native_factor.v2 carries MC64 numeric retry and MC64-to-InfNorm scaling fallback deltas for selected monolithic calls; other internal numeric retries unknown"
                    .into(),
            ),
        );
        report.metrics.insert(
            "linear.observed.refinement_actual_steps".into(),
            Metric::Text("native_refinement.v2 carries actual RHS, correction batches, residual products and outcomes for selected monolithic calls; aggregate and outer refinement unknown".into()),
        );
        for (i, path) in ["monolithic", "schur.f", "schur.s"].into_iter().enumerate() {
            for (kind, count) in [
                ("factor_attempts", state.factors[i]),
                ("failed_factor_attempts", state.failed_factors[i]),
                ("backsolve_calls", state.solves[i]),
                ("failed_backsolve_calls", state.failed_solves[i]),
            ] {
                report.metrics.insert(
                    format!("linear.observed.{path}.{kind}"),
                    Metric::Integer(i64::try_from(count).unwrap_or(i64::MAX)),
                );
            }
        }
        report.metrics.insert(
            "linear.schur.actual_use".into(),
            Metric::Bool(state.factors[1] > 0 || state.factors[2] > 0),
        );
        report.metrics.insert(
            "linear.storage.complete_extent".into(),
            Metric::Bool(state.linear_seen && !state.opaque_storage),
        );
        if let Some((used, reason)) = state.path {
            report.metrics.insert(
                "linear.schur.actual_use".into(),
                Metric::Bool(state.factors[1] > 0 || state.factors[2] > 0),
            );
            report
                .metrics
                .insert("linear.schur.final_path".into(), Metric::Bool(used));
            report.metrics.insert(
                "linear.schur.path_reason".into(),
                Metric::Text(reason.into()),
            );
        }
        report.metrics.insert(
            "linear.storage.known_extent".into(),
            Metric::Integer(i64::try_from(state.known_storage).unwrap_or(i64::MAX)),
        );
        report.metrics.insert(
            "native.application.storage.opaque".into(),
            Metric::Bool(state.application_opaque),
        );
        report.metrics.insert(
            "linear.storage.reservation_bound".into(),
            Metric::Integer(i64::try_from(state.known_storage).unwrap_or(i64::MAX)),
        );
        report.metrics.insert(
            "linear.storage.scope".into(),
            Metric::Text("factors-actions-scratch-and-retained-fallback".into()),
        );
        report.metrics.insert(
            "linear.storage.opaque".into(),
            Metric::Bool(state.opaque_storage),
        );
        report.metrics.insert(
            "linear.storage.actual_total".into(),
            Metric::Text("unknown".into()),
        );
        if let Some(layout) = &state.layout {
            report.provenance.insert("linear.native_layout".into(),serde_json::json!({"x_original":layout.x,"c_original":layout.c,"d_original":layout.d,"full_to_x":layout.full_to_x,"full_to_c":layout.full_to_c,"full_to_d":layout.full_to_d,"fixed_removed":layout.fixed_removed,"relax_bounds":layout.relaxed_fixed,"separator":state.separator}).to_string());
        }
    }
}
impl Observer for Observation {
    fn wants_detail(&self, family: &str) -> bool {
        self.detail
            .as_ref()
            .is_some_and(|producer| producer.detail_selected(detail_family(family)))
    }
    fn detail(&self, family: &str, fields: &[pounce_common::observed::DetailField<'_>]) {
        use pounce_common::observed::DetailValue;
        let family = detail_family(family);
        let Some(producer) = &self.detail else {
            return;
        };
        let overflow = fields.iter().any(|field| {
            field.name == "counter_overflow" && matches!(field.value, DetailValue::Flag(true))
        });
        let mut identity =
            std::collections::BTreeMap::from([("backend".into(), Metric::Text("pounce".into()))]);
        {
            let mut state = self.state.borrow_mut();
            let ordinal = |value: Option<u64>| {
                value.and_then(|value| i64::try_from(value).ok()).map_or(
                    Metric::Unavailable(crate::solve::UnavailableReason::Unknown),
                    Metric::Integer,
                )
            };
            let group = if state.detail_overflow {
                None
            } else {
                state.next_detail.checked_add(1)
            };
            if let Some(group) = group {
                state.next_detail = group;
            } else {
                state.detail_overflow = true;
            }
            identity.insert("detail_group_ordinal".into(), ordinal(group));
            if let Some(call) = state.calls.last() {
                identity.insert("call_ordinal".into(), ordinal(call.ordinal));
                identity.insert("parent_call_ordinal".into(), ordinal(call.parent));
                identity.insert(
                    "parent_call_present".into(),
                    Metric::Bool(state.calls.len() > 1),
                );
                identity.insert(
                    "call_path".into(),
                    Metric::Text(["monolithic", "schur_f", "schur_s"][index(call.path)].into()),
                );
                identity.insert(
                    "call_primitive".into(),
                    Metric::Text(
                        if call.primitive == Primitive::Factor {
                            "factor"
                        } else {
                            "backsolve"
                        }
                        .into(),
                    ),
                );
            } else {
                identity.insert(
                    "call_ordinal".into(),
                    Metric::Unavailable(crate::solve::UnavailableReason::Unknown),
                );
            }
        }
        // Progress retains at most 4 KiB per event. Bound every coherent part
        // after adding adapter identity; missing parts remain detectable.
        let part_count = fields.len().max(1).div_ceil(12);
        let elapsed = self.execution.started.elapsed();
        for part in 0..part_count {
            let mut values = identity.clone();
            values.insert("detail_part".into(), Metric::Integer(part as i64));
            values.insert("detail_parts".into(), Metric::Integer(part_count as i64));
            for field in fields.iter().skip(part * 12).take(12) {
                let value = match field.value {
                    DetailValue::Count(value) => (if overflow {
                        None
                    } else {
                        i64::try_from(value).ok()
                    })
                    .map_or(
                        Metric::Unavailable(crate::solve::UnavailableReason::Unknown),
                        Metric::Integer,
                    ),
                    DetailValue::Number(value) => Metric::Real(value),
                    DetailValue::Flag(value) => Metric::Bool(value),
                    DetailValue::Text(value) => Metric::Text(value.into()),
                };
                values.insert(field.name.into(), value);
            }
            let mut source = crate::solve::EventSource::new(
                "native_linear_detail",
                Some("native_linear_system"),
                None,
            )
            .with_role(pse_diagnostics::observation::ProducerRole::NativeFactor);
            source.family = family.into();
            self.execution.progress.push_from(
                crate::solve::Event {
                    source: Some(source),
                    phase: format!("pounce.{family}"),
                    elapsed,
                    values,
                    incumbent: None,
                },
                producer,
            );
        }
    }
    fn detail_failed(&self, family: &str) {
        if let Some(producer) = &self.detail {
            producer
                .context()
                .record_source_unavailable(detail_family(family));
            producer
                .context()
                .unavailable("native.detail.failed", detail_family(family));
        }
    }
    fn bind_layout(&self, layout: &Layout) -> Result<Option<Vec<usize>>, Abort> {
        self.execution.check().map_err(|e| self.fail(e))?;
        let mapped = self
            .separator
            .as_ref()
            .map(|separator| {
                let mut indices = Vec::new();
                for &original in &separator.variables {
                    if original >= layout.full_to_x.len() {
                        return Err(ProblemError::Contract(
                            "separator variable outside original layout".into(),
                        ));
                    }
                    if let Some(native) = layout.x_index(original) {
                        indices.push(native);
                    }
                }
                for &original in &separator.rows {
                    if original >= layout.full_to_c.len() {
                        return Err(ProblemError::Contract(
                            "separator row outside original layout".into(),
                        ));
                    }
                    if let Some(native) = layout.row_index(original) {
                        indices.push(native);
                    }
                    if let Some(&d) = layout.full_to_d.get(original)
                        && d >= 0
                    {
                        indices.push(layout.x.len() + d as usize);
                    }
                }
                indices.sort_unstable();
                indices.dedup();
                Ok(indices)
            })
            .transpose()
            .map_err(|e| self.fail(e))?;
        let mut state = self.state.borrow_mut();
        state.layout = Some(layout.clone());
        state.separator = mapped.clone();
        Ok(mapped)
    }
    fn event(&self, event: &Event) -> Result<(), Abort> {
        if !matches!(event, Event::End { .. }) {
            self.execution.check().map_err(|e| self.fail(e))?;
        }
        match event {
            Event::Storage {
                scope,
                owner,
                instance,
                known_bytes,
                opaque,
            } => {
                let scope = match scope {
                    pounce_common::observed::StorageScope::Application => {
                        crate::solve::NativeStorageScope::Application
                    }
                    pounce_common::observed::StorageScope::Linear => {
                        crate::solve::NativeStorageScope::Linear
                    }
                };
                if scope == crate::solve::NativeStorageScope::Linear && !opaque {
                    let state = self.state.borrow();
                    let previous = state
                        .storage_owners
                        .get(&(owner.to_string(), *instance))
                        .copied()
                        .unwrap_or(0);
                    let retained = state
                        .known_storage
                        .checked_add(known_bytes.saturating_sub(previous));
                    drop(state);
                    let retained = retained.ok_or_else(|| {
                        self.fail(ProblemError::memory("linear reservation overflow"))
                    })?;
                    if self.allowance.is_some_and(|allowance| retained > allowance) {
                        return Err(self.fail(ProblemError::memory(format!(
                            "linear reservation {retained} exceeds admitted allowance"
                        ))));
                    }
                }
                let previous = self
                    .state
                    .borrow()
                    .storage_owners
                    .get(&(owner.to_string(), *instance))
                    .copied()
                    .unwrap_or(0);
                let delta = known_bytes.saturating_sub(previous);
                if let Some(admission) = &self.execution.work_admission {
                    admission
                        .admit_storage(scope, &format!("{owner}@{instance}"), delta, *opaque)
                        .map_err(|e| self.fail(e))?;
                }

                let mut state = self.state.borrow_mut();
                if scope == crate::solve::NativeStorageScope::Linear {
                    state.linear_seen = true;
                    let key = (owner.to_string(), *instance);
                    let previous = state.storage_owners.get(&key).copied().unwrap_or(0);
                    state.known_storage = state
                        .known_storage
                        .saturating_add(known_bytes.saturating_sub(previous));
                    state.storage_owners.insert(key, previous.max(*known_bytes));
                    state.opaque_storage |= *opaque;
                } else {
                    state.application_opaque |= *opaque;
                }
            }
            Event::Begin {
                path,
                primitive,
                rhs,
                refinement_bound,
            } => {
                let work = WorkEvidence {
                    evaluations: Some(0),
                    iterations: Some(0),
                    factorizations: Some(u64::from(*primitive == Primitive::Factor)),
                    proof_steps: Some(0),
                };
                if let Some(admission) = &self.execution.work_admission {
                    crate::quality::contained(|| admission.admit(work))
                        .map_err(|e| self.fail(e))?;
                }
                let mut state = self.state.borrow_mut();
                let path_name = ["monolithic", "schur_f", "schur_s"][index(*path)];
                let primitive_name = if *primitive == Primitive::Factor {
                    "factor"
                } else {
                    "backsolve"
                };
                let name = format!("pounce.linear.{path_name}.{primitive_name}");
                crate::observation::count(&format!("{name}.attempts"), 1);
                if let Some(context) = pse_diagnostics::observation::current() {
                    state.phases[index(*path)][usize::from(*primitive == Primitive::Backsolve)]
                        .push(context.phase(&name));
                }
                match primitive {
                    Primitive::Factor => {
                        state.factors[index(*path)] += 1;
                        state.pending_factors[index(*path)] += 1;
                    }
                    Primitive::Backsolve => state.solves[index(*path)] += 1,
                }
                let ordinal = if state.call_overflow {
                    None
                } else {
                    state.next_call.checked_add(1)
                };
                if let Some(ordinal) = ordinal {
                    state.next_call = ordinal;
                } else {
                    state.call_overflow = true;
                }
                let parent = state.calls.last().and_then(|call| call.ordinal);
                state.calls.push(Call {
                    ordinal,
                    parent,
                    path: *path,
                    primitive: *primitive,
                    rhs: *rhs,
                    refinement_bound: *refinement_bound,
                });
                drop(state);
                self.call_detail("begin", None, *rhs, *refinement_bound);
            }
            Event::End {
                path,
                primitive,
                succeeded,
            } => {
                let (rhs, bound) = self
                    .state
                    .borrow()
                    .calls
                    .last()
                    .map_or((0, 0), |call| (call.rhs, call.refinement_bound));
                self.call_detail("end", Some(*succeeded), rhs, bound);
                // Source operations are synchronous and nested. A malformed optional
                // stack loses correlation; it cannot revoke compulsory work permission.
                let mut state = self.state.borrow_mut();
                if state
                    .calls
                    .last()
                    .is_some_and(|call| call.path == *path && call.primitive == *primitive)
                {
                    state.calls.pop();
                } else {
                    state.calls.clear();
                }
                drop(state);
                self.state.borrow_mut().phases[index(*path)]
                    [usize::from(*primitive == Primitive::Backsolve)]
                .pop();
                let name = format!(
                    "pounce.linear.{}.{}.{}",
                    ["monolithic", "schur_f", "schur_s"][index(*path)],
                    if *primitive == Primitive::Factor {
                        "factor"
                    } else {
                        "backsolve"
                    },
                    if *succeeded { "succeeded" } else { "failed" }
                );
                crate::observation::count(&name, 1);
                if *primitive == Primitive::Factor {
                    let mut state = self.state.borrow_mut();
                    state.pending_factors[index(*path)] = state.pending_factors[index(*path)]
                        .checked_sub(1)
                        .ok_or_else(|| {
                            Abort::Contract("factor End has no admitted Begin".into())
                        })?;
                }
                if !succeeded {
                    let mut state = self.state.borrow_mut();
                    match primitive {
                        Primitive::Factor => state.failed_factors[index(*path)] += 1,
                        Primitive::Backsolve => state.failed_solves[index(*path)] += 1,
                    }
                }
                if let Some(admission) = &self.execution.work_admission {
                    crate::quality::contained(|| {
                        admission.observe(WorkEvidence {
                            evaluations: Some(0),
                            iterations: Some(0),
                            factorizations: Some(u64::from(*primitive == Primitive::Factor)),
                            proof_steps: Some(0),
                        })
                    })
                    .map_err(|e| self.fail(e))?;
                }
            }
            Event::Selected { schur, reason } => {
                self.state.borrow_mut().path = Some((*schur, *reason));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    };
    #[derive(Debug, Default)]
    struct Count {
        admitted: AtomicU64,
        observed: AtomicU64,
    }
    impl crate::solve::WorkAdmission for Count {
        fn admit(&self, work: WorkEvidence) -> Result<(), ProblemError> {
            self.admitted
                .fetch_add(work.factorizations.unwrap(), Ordering::SeqCst);
            Ok(())
        }
        fn observe(&self, work: WorkEvidence) -> Result<(), ProblemError> {
            self.observed
                .fetch_add(work.factorizations.unwrap(), Ordering::SeqCst);
            Ok(())
        }
        fn admit_storage(
            &self,
            _: crate::solve::NativeStorageScope,
            _: &str,
            _: usize,
            _: bool,
        ) -> Result<(), ProblemError> {
            Ok(())
        }
    }
    #[derive(Debug, Default)]
    struct Details(std::sync::Mutex<Vec<pse_diagnostics::observation::CoherentRecord>>);
    impl pse_diagnostics::observation::RecordTap for Details {
        fn selected(&self, family: &str) -> bool {
            matches!(
                family,
                "native_linear_call.v1" | "native_factor.v2" | "native_refinement.v2"
            )
        }
        fn observe(&self, record: &pse_diagnostics::observation::CoherentRecord) {
            if self.selected(&record.family) {
                self.0.lock().unwrap().push(record.clone());
            }
        }
    }
    #[test]
    fn selected_detail_has_exact_nested_call_parent_and_balanced_boundaries() {
        use pounce_common::observed::{DetailField, DetailValue};
        use pse_diagnostics::observation::{ObservationHub, Policy, Value};
        let operation = ObservationHub::new(Policy::default()).start("nested-pounce-linear");
        let tap = Arc::new(Details::default());
        let _entered = operation.context().with_record_tap(tap.clone()).enter();
        let observer = Observation::new(
            Execution::new(
                pse_diagnostics::Cancellation::default(),
                &crate::solve::Controls::default(),
            ),
            None,
            None,
        );
        observer
            .event(&Event::Begin {
                path: Path::SchurF,
                primitive: Primitive::Factor,
                rhs: 0,
                refinement_bound: 0,
            })
            .unwrap();
        observer
            .event(&Event::Begin {
                path: Path::Monolithic,
                primitive: Primitive::Factor,
                rhs: 0,
                refinement_bound: 0,
            })
            .unwrap();
        observer.detail(
            "native_factor.v1",
            &[DetailField {
                name: "factor_values_available",
                value: DetailValue::Flag(true),
            }],
        );
        observer
            .event(&Event::End {
                path: Path::Monolithic,
                primitive: Primitive::Factor,
                succeeded: false,
            })
            .unwrap();
        observer
            .event(&Event::End {
                path: Path::SchurF,
                primitive: Primitive::Factor,
                succeeded: false,
            })
            .unwrap();
        observer
            .event(&Event::Begin {
                path: Path::Monolithic,
                primitive: Primitive::Backsolve,
                rhs: 2,
                refinement_bound: 3,
            })
            .unwrap();
        observer.detail(
            "native_refinement.v1",
            &[DetailField {
                name: "initial_rhs_completed",
                value: DetailValue::Count(2),
            }],
        );
        observer
            .event(&Event::End {
                path: Path::Monolithic,
                primitive: Primitive::Backsolve,
                succeeded: true,
            })
            .unwrap();
        let records = tap.0.lock().unwrap();
        let field = |record: &pse_diagnostics::observation::CoherentRecord, name: &str| {
            record
                .values
                .iter()
                .find(|field| field.name == name)
                .unwrap()
                .value
                .clone()
        };
        let factor = records
            .iter()
            .find(|record| record.family == "native_factor.v2")
            .unwrap();
        assert_eq!(field(factor, "call_ordinal"), Value::Integer(2));
        assert_eq!(field(factor, "parent_call_ordinal"), Value::Integer(1));
        assert_eq!(field(factor, "parent_call_present"), Value::Boolean(true));
        assert_eq!(field(factor, "call_path"), Value::Text("monolithic".into()));
        let refinement = records
            .iter()
            .find(|record| record.family == "native_refinement.v2")
            .unwrap();
        assert_eq!(field(refinement, "call_ordinal"), Value::Integer(3));
        assert_eq!(
            field(refinement, "parent_call_present"),
            Value::Boolean(false)
        );
        let boundaries: Vec<_> = records
            .iter()
            .filter(|record| record.family == "native_linear_call.v1")
            .collect();
        assert_eq!(boundaries.len(), 6);
        assert_eq!(field(boundaries[5], "rhs"), Value::Integer(2));
        assert_eq!(field(boundaries[5], "refinement_bound"), Value::Integer(3));
        assert!(observer.state.borrow().calls.is_empty());
        assert!(observer.take_error().is_none());
    }
    #[test]
    fn optional_call_identity_overflow_and_sink_failure_do_not_abort_work() {
        use pse_diagnostics::observation::{ObservationHub, Policy, RecordTap};
        #[derive(Debug)]
        struct Failing;
        impl RecordTap for Failing {
            fn selected(&self, _: &str) -> bool {
                true
            }
            fn observe(&self, _: &pse_diagnostics::observation::CoherentRecord) {
                panic!("optional sink");
            }
        }
        let operation = ObservationHub::new(Policy::default()).start("optional-pounce-linear");
        let _entered = operation
            .context()
            .with_record_tap(Arc::new(Failing))
            .enter();
        let observer = std::rc::Rc::new(Observation::new(
            Execution::new(
                pse_diagnostics::Cancellation::default(),
                &crate::solve::Controls::default(),
            ),
            None,
            None,
        ));
        observer.state.borrow_mut().next_call = u64::MAX;
        observer.state.borrow_mut().next_detail = u64::MAX;
        let scope = pounce_common::observed::Scope::enter(observer.clone());
        assert!(
            pounce_common::observed::Operation::begin(Path::Monolithic, Primitive::Factor, 0, 0)
                .unwrap()
                .finish(true)
        );
        assert!(observer.state.borrow().call_overflow);
        assert!(observer.state.borrow().detail_overflow);
        assert!(observer.state.borrow().calls.is_empty());
        let current = observer.execution.progress.current().unwrap();
        assert!(matches!(
            current.values["call_ordinal"],
            Metric::Unavailable(_)
        ));
        assert!(matches!(
            current.values["detail_group_ordinal"],
            Metric::Unavailable(_)
        ));
        assert!(scope.abort().is_none());
        assert!(observer.take_error().is_none());
    }
    #[test]
    fn optional_progress_tap_panic_cannot_abort_compulsory_native_operations() {
        use pse_diagnostics::observation::{ObservationHub, Policy};
        #[derive(Debug)]
        struct FailingProgress;
        impl crate::solve::ProgressTap for FailingProgress {
            fn observe(&self, _: &crate::solve::Event) {
                panic!("optional progress sink");
            }
        }
        let operation = ObservationHub::new(Policy::default()).start("optional-pounce-progress");
        let tap = Arc::new(Details::default());
        let _entered = operation.context().with_record_tap(tap.clone()).enter();
        let mut execution = Execution::new(
            pse_diagnostics::Cancellation::default(),
            &crate::solve::Controls::default(),
        );
        execution.progress = Arc::new(crate::solve::Progress::tapped(
            32,
            Arc::new(FailingProgress),
        ));
        let observer = std::rc::Rc::new(Observation::new(execution, None, None));
        let scope = pounce_common::observed::Scope::enter(observer.clone());
        let call =
            pounce_common::observed::Operation::begin(Path::Monolithic, Primitive::Factor, 0, 0)
                .unwrap();
        pounce_common::observed::detail("native_factor.v1", &[]);
        assert!(call.finish(true));
        assert!(observer.state.borrow().calls.is_empty());
        assert!(observer.take_error().is_none());
        assert!(scope.abort().is_none());
        let records = tap.0.lock().unwrap();
        assert!(
            records
                .iter()
                .any(|record| record.family == "native_factor.v2")
        );
        assert_eq!(
            records
                .iter()
                .filter(|record| record.family == "native_linear_call.v1")
                .count(),
            2
        );
    }
    #[test]
    fn failed_factor_and_missing_end_reconcile_once() {
        use pse_diagnostics::observation::{ObservationHub, Policy, Value};
        let hub = ObservationHub::new(Policy::default());
        let operation = hub.start("pounce-failed-factor");
        let _entered = operation.context().enter();
        let count = Arc::new(Count::default());
        let mut execution = Execution::new(
            pse_diagnostics::Cancellation::default(),
            &crate::solve::Controls::default(),
        );
        execution.work_admission = Some(count.clone());
        let observer = Observation::new(execution, None, None);
        for end in [true, false] {
            observer
                .event(&Event::Begin {
                    path: Path::Monolithic,
                    primitive: Primitive::Factor,
                    rhs: 0,
                    refinement_bound: 0,
                })
                .unwrap();
            if end {
                observer
                    .event(&Event::End {
                        path: Path::Monolithic,
                        primitive: Primitive::Factor,
                        succeeded: false,
                    })
                    .unwrap();
            }
            observer.finish_operations();
            observer.finish_operations();
        }
        assert_eq!(count.admitted.load(Ordering::SeqCst), 2);
        assert_eq!(count.observed.load(Ordering::SeqCst), 2);
        assert_eq!(observer.state.borrow().failed_factors[0], 2);
        let snapshot = operation
            .reader()
            .snapshot(std::time::Duration::from_secs(60));
        assert!(snapshot.phases.is_empty());
        assert_eq!(
            snapshot
                .metrics
                .iter()
                .find(|m| m.name == "pounce.linear.monolithic.factor.attempts")
                .unwrap()
                .cumulative,
            Value::Count(2)
        );
        assert_eq!(
            snapshot
                .metrics
                .iter()
                .find(|m| m.name == "pounce.linear.monolithic.factor.interrupted")
                .unwrap()
                .cumulative,
            Value::Count(1)
        );
    }
}
