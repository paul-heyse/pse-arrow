// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Registry-generated physical observations retain Arrow allocation ownership.
use super::{RunResult, WorkflowError, contract, relation};
use crate::math::solves::Outcome;
use pse_backend_native::solve::{Metric, OptionValue};
use pse_ids::SemanticId;
use pse_model::HeapUsage;
use pse_relations::{
    columnar::FieldCheckedBatch,
    generated::{
        authored::computation_models as models,
        enums::{NativeAssurance, NativeCandidateKind, NativeMetricKind, NativeRunState},
        runtime::{
            solve_constraints as constraints, solve_metrics as metrics, solve_runs as runs,
            solve_variables as variables,
        },
    },
};
use std::{collections::BTreeMap, sync::Arc};
impl RunResult {
    /// Encode once from joined immutable reports; clones retain the final Arrow buffers.
    pub fn tables(&self) -> Result<&BTreeMap<SemanticId, FieldCheckedBatch>, Arc<WorkflowError>> {
        self.batches
            .get_or_init(|| self.encode().map_err(Arc::new))
            .as_ref()
            .map_err(Arc::clone)
    }
    /// A retained checked table can outlive the result, run handle and model revision.
    pub fn table(&self, name: &str) -> Result<FieldCheckedBatch, Arc<WorkflowError>> {
        let spec = self
            .runtime
            .registry
            .relation(name)
            .ok_or_else(|| Arc::new(contract("unknown result relation")))?;
        self.tables()?
            .get(&spec.id)
            .cloned()
            .ok_or_else(|| Arc::new(contract("relation is not part of this run")))
    }
    fn encode(&self) -> Result<BTreeMap<SemanticId, FieldCheckedBatch>, WorkflowError> {
        let super::run::RunRequest::Solves(steps) = &self.request else {
            return match self.request {
                super::RunRequest::Simulation(_) => self.encode_simulation(),
                _ => self.encode_fit(),
            };
        };
        // Reserve declaration and bounded report expansion before creating Arrow builders.
        // Export subsequently transfers buffer ownership into the shared native pool.
        let bytes = steps
            .iter()
            .try_fold(0usize, |sum, step| {
                let source = step.revision.0.row.owned_bytes();
                let cells = step
                    .compiled()
                    .plan
                    .columns()
                    .len()
                    .checked_add(step.compiled().plan.structure().rows().len())?;
                sum.checked_add(source.checked_mul(8)?)?
                    .checked_add(cells.checked_mul(2048)?)?
                    .checked_add(
                        step.profile
                            .controls
                            .report_allowance()
                            .ok()?
                            .checked_mul(8)?,
                    )?
                    .checked_add(65536)
            })
            .ok_or_else(|| contract("result encoding allocation extent overflow"))?;
        let staging = pse_columnar::MemoryConsumer::new("workflow:result-encoding")
            .register(&self.runtime.shared.pool());
        staging
            .try_grow(bytes)
            .map_err(|e| WorkflowError::Math(e.into()))?;
        let registry = &self.runtime.registry;
        let mut run_rows = runs::Builder::with_registry(registry, steps.len()).map_err(relation)?;
        let mut variable_rows = variables::Builder::with_registry(registry, 0).map_err(relation)?;
        let mut constraint_rows =
            constraints::Builder::with_registry(registry, 0).map_err(relation)?;
        let mut metric_rows = metrics::Builder::with_registry(registry, 0).map_err(relation)?;
        let mut models_rows = models::Builder::with_registry(registry, 0).map_err(relation)?;
        let mut seen = std::collections::BTreeSet::new();
        for (ordinal, request) in steps.iter().enumerate() {
            let step = ordinal as i64;
            let declaration = request
                .revision
                .0
                .row
                .cases
                .iter()
                .find(|c| c.case_id == request.case)
                .ok_or_else(|| contract("selected declaration missing"))?;
            if seen.insert(request.revision.0.row.model_id) {
                models_rows
                    .push(request.revision.0.row.clone())
                    .map_err(relation)?;
            }
            let outcome = self.report.as_ref().ok().and_then(|r| match r {
                super::run::RunReport::Solves(r) => r.outcomes.get(ordinal),
                _ => None,
            });
            let native = match outcome {
                Some(Outcome::Native(r)) => Some(r.as_ref()),
                _ => None,
            };
            let constant = match outcome {
                Some(Outcome::Constant(r)) => Some(r),
                _ => None,
            };
            let quality = native
                .and_then(|r| r.quality.as_ref())
                .or_else(|| constant.map(|r| &r.quality));
            let observation = native
                .and_then(|r| r.observation.as_ref())
                .or_else(|| constant.map(|r| &r.observation));
            let candidate = native.and_then(|r| r.candidate.as_ref());
            let error = match (&self.report, outcome) {
                (Err(e), _) => Some(e.to_string()),
                (_, Some(Outcome::Rejected(e))) => Some(e.to_string()),
                _ => None,
            };
            let state = if native.is_some() {
                NativeRunState::Native
            } else if error.is_some() {
                NativeRunState::Rejected
            } else if constant.is_some() {
                NativeRunState::ConstantEvaluation
            } else {
                NativeRunState::Unattempted
            };
            run_rows
                .push(runs::Row {
                    run_id: self.run_id,
                    step,
                    model_id: Some(request.revision.0.row.model_id),
                    revision: Some(request.revision.identity()),
                    case_id: Some(request.case),
                    backend: native.map(|r| r.backend),
                    native_code: native.map(|r| r.termination.code),
                    native_status: native.map(|r| r.termination.name.clone()),
                    state,
                    termination: native.map(|r| r.termination.category),
                    assurance: native.map_or(NativeAssurance::None, |r| r.termination.assurance),
                    candidate_kind: candidate
                        .map(|c| c.kind)
                        .or_else(|| constant.map(|_| NativeCandidateKind::ConstantEvaluation)),
                    feasible: quality.map(|q| q.feasible()),
                    objective: observation
                        .and_then(|o| o.objective)
                        .or_else(|| candidate.and_then(|c| c.objective)),
                    objective_sense: declaration.objective.as_ref().map(|o| o.sense),
                    objective_quantity_id: declaration.objective.as_ref().map(|o| o.quantity_id),
                    validation_error: native.and_then(|r| r.validation_error.clone()),
                    error,
                    transformation: native
                        .and_then(|r| r.preprocessing.as_ref().map(|p| p.transformation)),
                })
                .map_err(relation)?;
            let coordinates: BTreeMap<_, _> = native
                .map(|r| {
                    r.variables
                        .iter()
                        .enumerate()
                        .map(|(i, id)| (*id, i))
                        .collect()
                })
                .unwrap_or_default();
            for v in &declaration.variables {
                let p = &v.port;
                let ix = coordinates.get(&p.symbol_id).copied();
                let value = if v.fixed {
                    declaration
                        .values
                        .iter()
                        .find(|v| v.symbol_id == p.symbol_id)
                        .map(|v| v.value)
                } else {
                    ix.and_then(|i| candidate.and_then(|c| c.primal.get(i).copied()))
                };
                let tolerance = if v.fixed {
                    None
                } else {
                    let i = request
                        .compiled()
                        .plan
                        .columns()
                        .iter()
                        .position(|id| *id == p.symbol_id);
                    i.and_then(|i| request.solve.tolerances().variables.get(i).copied())
                };
                let dual_status = match observation {
                    Some(o) if o.dual_error.is_none() => "evaluated_kkt_not_sensitivity_certified",
                    Some(o) if o.dual_error.is_some() => "unavailable_or_invalid",
                    _ => "unavailable",
                };
                variable_rows
                    .push(variables::Row {
                        run_id: self.run_id,
                        step,
                        symbol_id: p.symbol_id,
                        quantity_id: Some(p.quantity_id),
                        unit_id: Some(p.unit_id),
                        fixed: v.fixed,
                        parameter: false,
                        value,
                        lower: v.lower,
                        upper: v.upper,
                        lower_violation: value.map(|x| v.lower.map_or(0.0, |l| (l - x).max(0.0))),
                        upper_violation: value.map(|x| v.upper.map_or(0.0, |u| (x - u).max(0.0))),
                        tolerance,
                        lower_dual: ix.and_then(|i| {
                            candidate.and_then(|c| {
                                c.bound_dual.as_ref().and_then(|(l, _)| l.get(i).copied())
                            })
                        }),
                        upper_dual: ix.and_then(|i| {
                            candidate.and_then(|c| {
                                c.bound_dual.as_ref().and_then(|(_, u)| u.get(i).copied())
                            })
                        }),
                        reduced_cost: ix.and_then(|i| {
                            candidate.and_then(|c| {
                                c.reduced_costs.as_ref().and_then(|v| v.get(i).copied())
                            })
                        }),
                        stationarity: ix.and_then(|i| {
                            observation.and_then(|o| {
                                o.stationarity.as_ref().and_then(|v| v.get(i).copied())
                            })
                        }),
                        dual_qualification: dual_status.into(),
                    })
                    .map_err(relation)?;
            }
            for p in &declaration.parameters {
                variable_rows
                    .push(variables::Row {
                        run_id: self.run_id,
                        step,
                        symbol_id: p.symbol_id,
                        quantity_id: Some(p.quantity_id),
                        unit_id: Some(p.unit_id),
                        fixed: true,
                        parameter: true,
                        value: declaration
                            .values
                            .iter()
                            .find(|v| v.symbol_id == p.symbol_id)
                            .map(|v| v.value),
                        lower: None,
                        upper: None,
                        lower_violation: None,
                        upper_violation: None,
                        tolerance: None,
                        lower_dual: None,
                        upper_dual: None,
                        reduced_cost: None,
                        stationarity: None,
                        dual_qualification: "not_applicable_parameter".into(),
                    })
                    .map_err(relation)?;
            }
            let rows = request.compiled().plan.structure().rows();
            for (i, r) in rows.iter().enumerate() {
                let unit = request
                    .revision
                    .0
                    .physical
                    .quantities
                    .quantity_type(r.quantity)
                    .map_err(super::math)?
                    .canonical_unit
                    .as_id();
                constraint_rows
                    .push(constraints::Row {
                        run_id: self.run_id,
                        step,
                        row_id: r.id,
                        quantity_id: Some(r.quantity.as_id()),
                        unit_id: Some(unit),
                        value: observation.and_then(|o| o.values.get(i).copied()),
                        lower: r.lower.is_finite().then_some(r.lower),
                        upper: r.upper.is_finite().then_some(r.upper),
                        equality_residual: observation
                            .and_then(|o| o.equality_residuals.get(i).copied().flatten()),
                        lower_violation: observation
                            .and_then(|o| o.lower_violations.get(i).copied()),
                        upper_violation: observation
                            .and_then(|o| o.upper_violations.get(i).copied()),
                        tolerance: request.solve.tolerances().rows.get(i).copied(),
                        dual: candidate
                            .and_then(|c| c.row_dual.as_ref().and_then(|v| v.get(i).copied())),
                        dual_qualification: observation
                            .map_or("unavailable", |o| {
                                if o.dual_error.is_none() {
                                    "evaluated_kkt_not_sensitivity_certified"
                                } else {
                                    "unavailable_or_invalid"
                                }
                            })
                            .into(),
                    })
                    .map_err(relation)?;
            }
            for (name, value) in [
                (
                    "physical.origin",
                    request.revision.0.physical.origin.to_owned(),
                ),
                (
                    "physical.identity",
                    request.revision.0.physical.key.to_prefixed(),
                ),
                ("registry", registry.fingerprint().to_prefixed()),
                ("build.source", pse_buildinfo::SOURCE_IDENTITY.to_prefixed()),
                (
                    "build.identity",
                    pse_buildinfo::BUILD_IDENTITY.to_prefixed(),
                ),
                ("profile.requested", format!("{:?}", request.profile)),
                (
                    "compiler.structure",
                    request.compiled().plan.structure().key().to_prefixed(),
                ),
                (
                    "compiler.presolve_facts",
                    request.compiled().presolve.key.to_prefixed(),
                ),
            ] {
                push_metric(
                    &mut metric_rows,
                    self.run_id,
                    step,
                    "provenance",
                    name,
                    &Metric::Text(value),
                )?;
            }
            if let Some(proof) = request.solve.quadratic_evidence() {
                use pse_math::convexity::ConvexityAssessment;
                let (state, values, reason) = match proof.assessment() {
                    None | Some(ConvexityAssessment::Exact(_)) => ("exact_gram", vec![], None),
                    Some(ConvexityAssessment::NumericalPsd {
                        minimum,
                        tolerance,
                        uncertainty,
                    }) => (
                        "numerical_psd",
                        vec![
                            ("minimum", *minimum),
                            ("tolerance", *tolerance),
                            ("uncertainty", *uncertainty),
                        ],
                        None,
                    ),
                    Some(ConvexityAssessment::Indefinite {
                        minimum,
                        uncertainty,
                    }) => (
                        "indefinite",
                        vec![("minimum", *minimum), ("uncertainty", *uncertainty)],
                        None,
                    ),
                    Some(ConvexityAssessment::Inconclusive(reason)) => {
                        ("inconclusive", vec![], Some(format!("{reason:?}")))
                    }
                };
                push_metric(
                    &mut metric_rows,
                    self.run_id,
                    step,
                    "convexity",
                    "assessment",
                    &Metric::Text(state.into()),
                )?;
                for (name, value) in values {
                    push_metric(
                        &mut metric_rows,
                        self.run_id,
                        step,
                        "convexity",
                        name,
                        &Metric::Real(value),
                    )?;
                }
                if let Some(reason) = reason {
                    push_metric(
                        &mut metric_rows,
                        self.run_id,
                        step,
                        "convexity",
                        "reason",
                        &Metric::Text(reason),
                    )?;
                }
            }
            for (name, provider) in &request.revision.0.providers {
                push_metric(
                    &mut metric_rows,
                    self.run_id,
                    step,
                    "provider",
                    &format!("{name}.identity"),
                    &Metric::Text(provider.registration.spec().identity().to_prefixed()),
                )?;
                push_metric(
                    &mut metric_rows,
                    self.run_id,
                    step,
                    "provider",
                    &format!("{name}.output"),
                    &Metric::Integer(provider.output as i64),
                )?;
            }
            if let Some(identity) = request.solve.compatibility() {
                push_metric(
                    &mut metric_rows,
                    self.run_id,
                    step,
                    "provenance",
                    "prepared.layout",
                    &Metric::Text(identity.layout.to_prefixed()),
                )?;
                push_metric(
                    &mut metric_rows,
                    self.run_id,
                    step,
                    "provenance",
                    "prepared.data",
                    &Metric::Text(identity.data.to_prefixed()),
                )?;
            }
            if let Some(native) = native {
                push_native_metrics(&mut metric_rows, self.run_id, step, native)?;
                if let Some(o) = observation
                    && let Some(error) = &o.dual_error
                {
                    push_metric(
                        &mut metric_rows,
                        self.run_id,
                        step,
                        "diagnostic",
                        "dual",
                        &Metric::Text(error.clone()),
                    )?;
                }
            }
        }
        let mut batches = BTreeMap::from([
            (runs::RELATION_ID, run_rows.finish().map_err(relation)?),
            (
                variables::RELATION_ID,
                variable_rows.finish().map_err(relation)?,
            ),
            (
                constraints::RELATION_ID,
                constraint_rows.finish().map_err(relation)?,
            ),
            (
                metrics::RELATION_ID,
                metric_rows.finish().map_err(relation)?,
            ),
            (models::RELATION_ID, models_rows.finish().map_err(relation)?),
        ]);
        self.retain_sources(&mut batches)?;
        Ok(batches)
    }
}
pub(super) fn push_metric(
    builder: &mut metrics::Builder,
    run_id: SemanticId,
    step: i64,
    namespace: &str,
    name: &str,
    value: &Metric,
) -> Result<(), WorkflowError> {
    let mut row = metrics::Row {
        run_id,
        step,
        namespace: namespace.into(),
        name: name.into(),
        kind: NativeMetricKind::Text,
        real: None,
        integer: None,
        boolean: None,
        text: None,
        unavailable: None,
    };
    match value {
        Metric::Real(v) if v.is_finite() => {
            row.kind = NativeMetricKind::Real;
            row.real = Some(*v);
        }
        Metric::Real(v) => {
            row.text = Some(v.to_string());
        }
        Metric::Integer(v) => {
            row.kind = NativeMetricKind::Integer;
            row.integer = Some(*v);
        }
        Metric::Bool(v) => {
            row.kind = NativeMetricKind::Boolean;
            row.boolean = Some(*v);
        }
        Metric::Text(v) => row.text = Some(v.clone()),
        Metric::Unavailable(reason) => {
            row.kind = NativeMetricKind::Unavailable;
            row.unavailable = Some(*reason);
        }
    }
    builder.push(row).map_err(relation)
}

/// Common native options, metrics, certificates, progress and presolve receipt.
pub(super) fn push_native_metrics(
    builder: &mut metrics::Builder,
    run_id: SemanticId,
    step: i64,
    native: &pse_backend_native::solve::SolveReport,
) -> Result<(), WorkflowError> {
    push_metric(
        builder,
        run_id,
        step,
        "qualification",
        "numerical",
        &Metric::Text(native.qualification.as_str().into()),
    )?;
    if let Some(start) = &native.start_receipt {
        let value = serde_json::json!({"previous_attempt":start.previous_attempt,"seed":start.seed.as_ref().map(|s|s.snapshot()),"sparse_seed":start.sparse_seed.as_ref().map(|s|s.iter().map(|(id,v)|(id.to_hex(),*v)).collect::<BTreeMap<_,_>>()),"transformations":start.transformations,"submitted":start.submitted});
        push_metric(
            builder,
            run_id,
            step,
            "start",
            "request",
            &Metric::Text(value.to_string()),
        )?;
    }
    if let Some(seed) = &native.warm_start {
        push_metric(
            builder,
            run_id,
            step,
            "start",
            "available",
            &Metric::Text(seed.snapshot().to_string()),
        )?;
    }
    for (name, value) in &native.metrics {
        push_metric(builder, run_id, step, "metric", name, value)?;
    }
    for (name, value) in &native.provenance {
        push_metric(
            builder,
            run_id,
            step,
            "native_provenance",
            name,
            &Metric::Text(value.clone()),
        )?;
    }
    for (namespace, options) in [
        ("option", &native.options),
        ("native_default", &native.native_defaults),
    ] {
        for (name, value) in options {
            let v = match value {
                OptionValue::Real(v) => Metric::Real(*v),
                OptionValue::Integer(v) => Metric::Integer(i64::from(*v)),
                OptionValue::Text(v) => Metric::Text(v.clone()),
                OptionValue::Bool(v) => Metric::Bool(*v),
            };
            push_metric(builder, run_id, step, namespace, name, &v)?;
        }
    }
    push_metric(
        builder,
        run_id,
        step,
        "progress",
        "dropped_events",
        &Metric::Integer(native.dropped_events.min(i64::MAX as u64) as i64),
    )?;
    for (i, event) in native.events.iter().enumerate() {
        let ns = format!("event.{i}.{}", event.phase);
        push_metric(
            builder,
            run_id,
            step,
            &ns,
            "elapsed_seconds",
            &Metric::Real(event.elapsed.as_secs_f64()),
        )?;
        for (key, value) in &event.values {
            push_metric(builder, run_id, step, &ns, key, value)?;
        }
    }
    if let Some(c) = &native.certificate {
        push_metric(
            builder,
            run_id,
            step,
            "certificate",
            "kind",
            &Metric::Text(c.kind.clone()),
        )?;
        for (family, values) in [("primal", &c.primal), ("dual", &c.dual)] {
            if let Some(values) = values {
                for (i, v) in values.iter().enumerate() {
                    push_metric(
                        builder,
                        run_id,
                        step,
                        "certificate",
                        &format!("{family}.{i}"),
                        &Metric::Real(*v),
                    )?;
                }
            }
        }
    }
    #[cfg(feature = "solver-highs")]
    if let Some(d) = &native.highs_diagnostics {
        for (name, message) in &d.unavailable {
            push_metric(
                builder,
                run_id,
                step,
                "highs.unavailable",
                name,
                &Metric::Text(message.clone()),
            )?;
        }
        for (family, values) in [("primal_ray", &d.primal_ray), ("dual_ray", &d.dual_ray)] {
            if let Some(values) = values {
                for (i, v) in values.iter().enumerate() {
                    push_metric(
                        builder,
                        run_id,
                        step,
                        "highs",
                        &format!("{family}.{i}"),
                        &Metric::Real(*v),
                    )?;
                }
            }
        }
        if let Some(iis) = &d.iis {
            push_metric(
                builder,
                run_id,
                step,
                "highs.iis",
                "relaxation_only",
                &Metric::Bool(iis.relaxation_only),
            )?;
            for (side, rows) in [("column", &iis.columns), ("row", &iis.rows)] {
                for (id, code) in rows {
                    push_metric(
                        builder,
                        run_id,
                        step,
                        "highs.iis",
                        &format!("{side}.{}", id.to_hex()),
                        &Metric::Integer(i64::from(*code)),
                    )?;
                }
            }
            for (side, values) in [
                ("column_status", &iis.column_status),
                ("row_status", &iis.row_status),
            ] {
                for (i, value) in values.iter().enumerate() {
                    push_metric(
                        builder,
                        run_id,
                        step,
                        "highs.iis",
                        &format!("{side}.{i}"),
                        &Metric::Integer(i64::from(*value)),
                    )?;
                }
            }
        }
        for (family, ranges) in &d.ranging {
            for (i, id) in ranges.ids.iter().enumerate() {
                let ns = format!("highs.ranging.{family}.{}", id.to_hex());
                for (name, value) in [
                    ("value", ranges.value[i]),
                    ("objective", ranges.objective[i]),
                ] {
                    push_metric(builder, run_id, step, &ns, name, &Metric::Real(value))?;
                }
                for (name, value) in [
                    ("entering", ranges.entering[i]),
                    ("leaving", ranges.leaving[i]),
                ] {
                    push_metric(
                        builder,
                        run_id,
                        step,
                        &ns,
                        name,
                        &Metric::Integer(i64::from(value)),
                    )?;
                }
            }
        }
        if let Some(r) = &d.relaxation {
            push_metric(
                builder,
                run_id,
                step,
                "highs.relaxation",
                "operation_status",
                &Metric::Integer(i64::from(r.operation_status)),
            )?;
            push_metric(
                builder,
                run_id,
                step,
                "highs.relaxation",
                "native_code",
                &Metric::Integer(r.termination.code),
            )?;
            push_metric(
                builder,
                run_id,
                step,
                "highs.relaxation",
                "native_status",
                &Metric::Text(r.termination.name.clone()),
            )?;
            if let Some(primal) = &r.primal {
                for (i, value) in primal.iter().enumerate() {
                    push_metric(
                        builder,
                        run_id,
                        step,
                        "highs.relaxation",
                        &format!("primal.{i}"),
                        &Metric::Real(*value),
                    )?;
                }
            }
        }
    }
    if let Some(p) = &native.preprocessing {
        if let Some(proof) = &p.proof {
            let kind = proof.kind();
            push_metric(
                builder,
                run_id,
                step,
                "preprocessing_proof",
                "kind",
                &Metric::Text(kind.into()),
            )?;
            push_metric(
                builder,
                run_id,
                step,
                "preprocessing_proof",
                "normalization",
                &Metric::Text(proof.normalization.to_prefixed()),
            )?;
            if let Some(id) = proof.witness_row {
                push_metric(
                    builder,
                    run_id,
                    step,
                    "preprocessing_proof",
                    "witness_row",
                    &Metric::Text(id.to_hex()),
                )?;
            }
            for (kind, ids, budgets) in [
                ("row", &proof.rows, &proof.budgets.rows),
                ("variable", &proof.columns, &proof.budgets.variables),
            ] {
                for (id, budget) in ids.iter().zip(budgets) {
                    push_metric(
                        builder,
                        run_id,
                        step,
                        "preprocessing_proof",
                        &format!("{kind}.{}.normalized_budget", id.to_hex()),
                        &Metric::Real(*budget),
                    )?;
                }
            }
            for (index, (row, instance, output)) in proof.contributions.iter().enumerate() {
                push_metric(
                    builder,
                    run_id,
                    step,
                    "preprocessing_proof",
                    &format!("contribution.{index}"),
                    &Metric::Text(format!(
                        "row={};instance={};output={output}",
                        row.to_hex(),
                        instance.to_hex()
                    )),
                )?;
            }
        }
        for (name, value) in &p.diagnostics {
            push_metric(
                builder,
                run_id,
                step,
                "preprocessing",
                name,
                &Metric::Text(value.clone()),
            )?;
        }
        for (name, value) in &p.passes {
            let prefix = format!("{name:?}");
            if let Some(reason) = &value.reason {
                push_metric(
                    builder,
                    run_id,
                    step,
                    "preprocessing_pass",
                    &format!("{prefix}.reason"),
                    &Metric::Text(reason.clone()),
                )?;
            }

            for (field, state) in [
                ("requested", value.requested),
                ("eligible", value.eligible),
                ("applied", value.applied),
            ] {
                push_metric(
                    builder,
                    run_id,
                    step,
                    "preprocessing_pass",
                    &format!("{prefix}.{field}"),
                    &Metric::Bool(state),
                )?;
            }
        }
    }
    Ok(())
}
