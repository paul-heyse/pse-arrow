// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bridge actual source-owned native operations to task admission and observations.
use crate::{
    ProblemError, SolveSeparator,
    solve::{Execution, Metric, SolveReport, WorkEvidence},
};
use pounce_common::observed::{Abort, Event, Layout, Observer, Path, Primitive};
use std::cell::RefCell;
#[derive(Default)]
struct State {
    layout: Option<Layout>,
    separator: Option<Vec<usize>>,
    factors: [u64; 3],
    failed_factors: [u64; 3],
    solves: [u64; 3],
    failed_solves: [u64; 3],
    path: Option<(bool, &'static str)>,
    known_storage: usize,
    opaque_storage: bool,
    application_opaque: bool,
    linear_seen: bool,
    error: Option<ProblemError>,
}
pub(super) struct Observation {
    execution: Execution,
    separator: Option<SolveSeparator>,
    allowance: Option<usize>,
    state: RefCell<State>,
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
    pub(super) fn record(&self, report: &mut SolveReport) {
        let state = self.state.borrow();
        report.evidence.work.factorizations = Some(state.factors.iter().sum());
        report.metrics.insert(
            "linear.observed.factor_scope".into(),
            Metric::Text(
                "FERAL Solver::factor invocations; unrestricted internal numeric retries unknown"
                    .into(),
            ),
        );
        report.metrics.insert(
            "linear.observed.refinement_actual_steps".into(),
            Metric::Text("unknown; per-call upper bound emitted before operation".into()),
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
                    let retained = self.state.borrow().known_storage.checked_add(*known_bytes);
                    let retained = retained.ok_or_else(|| {
                        self.fail(ProblemError::memory("linear reservation overflow"))
                    })?;
                    if self.allowance.is_some_and(|allowance| retained > allowance) {
                        return Err(self.fail(ProblemError::memory(format!(
                            "linear reservation {retained} exceeds admitted allowance"
                        ))));
                    }
                }
                if let Some(admission) = &self.execution.work_admission {
                    admission
                        .admit_storage(scope, owner, *known_bytes, *opaque)
                        .map_err(|e| self.fail(e))?;
                }

                let mut state = self.state.borrow_mut();
                if scope == crate::solve::NativeStorageScope::Linear {
                    state.linear_seen = true;
                    state.known_storage = state.known_storage.saturating_add(*known_bytes);
                    state.opaque_storage |= *opaque;
                } else {
                    state.application_opaque |= *opaque;
                }
            }
            Event::Begin {
                path, primitive, ..
            } => {
                let work = WorkEvidence {
                    evaluations: Some(0),
                    iterations: Some(0),
                    factorizations: Some(u64::from(*primitive == Primitive::Factor)),
                    proof_steps: Some(0),
                };
                if let Some(admission) = &self.execution.work_admission {
                    admission.admit(work).map_err(|e| self.fail(e))?;
                }
                let mut state = self.state.borrow_mut();
                match primitive {
                    Primitive::Factor => state.factors[index(*path)] += 1,
                    Primitive::Backsolve => state.solves[index(*path)] += 1,
                }
            }
            Event::End {
                path,
                primitive,
                succeeded,
            } => {
                if let Some(admission) = &self.execution.work_admission {
                    admission
                        .observe(WorkEvidence {
                            evaluations: Some(0),
                            iterations: Some(0),
                            factorizations: Some(u64::from(*primitive == Primitive::Factor)),
                            proof_steps: Some(0),
                        })
                        .map_err(|e| self.fail(e))?;
                }
                if !succeeded {
                    let mut state = self.state.borrow_mut();
                    match primitive {
                        Primitive::Factor => state.failed_factors[index(*path)] += 1,
                        Primitive::Backsolve => state.failed_solves[index(*path)] += 1,
                    }
                }
            }
            Event::Selected { schur, reason } => {
                self.state.borrow_mut().path = Some((*schur, *reason));
            }
        }
        Ok(())
    }
}
