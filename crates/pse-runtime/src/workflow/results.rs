// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Registry-generated physical observations retain Arrow allocation ownership.
use super::{RunResult, WorkflowError, contract, relation};
use pse_backend_native::solve::{Metric, OptionValue};
use pse_ids::SemanticId;
use pse_model::generated::identities::RunId;
use pse_relations::{
    columnar::FieldCheckedBatch,
    generated::{
        enums::NativeMetricKind,
        runtime::{infeasibility_certificates as certificates, solve_metrics as metrics},
    },
};
use std::{collections::BTreeMap, sync::Arc};
impl RunResult {
    /// Advertised relations are completion metadata; discovery does not encode payloads.
    pub fn table_ids(&self) -> Vec<SemanticId> {
        use pse_relations::generated::runtime::*;
        let mut ids = vec![
            candidate_assessments::RELATION_ID,
            resolved_numerics::RELATION_ID,
            run_lineage::RELATION_ID,
        ];
        if self.completion().is_ok() {
            ids.push(accuracy_goal_assessments::RELATION_ID);
        }
        match &self.request {
            super::RunRequest::Modeling(_) => ids.extend([
                solve_runs::RELATION_ID,
                solve_variables::RELATION_ID,
                solve_constraints::RELATION_ID,
                solve_metrics::RELATION_ID,
                solution_pool::RELATION_ID,
                infeasibility_certificates::RELATION_ID,
                incumbents::RELATION_ID,
                modeling_checks::RELATION_ID,
                modeling_reports::RELATION_ID,
                modeling_findings::RELATION_ID,
                solve_strategy_events::RELATION_ID,
                solve_strategy_products::RELATION_ID,
                local_validity::RELATION_ID,
                parametric_sensitivities::RELATION_ID,
                reduced_hessians::RELATION_ID,
                propagated_covariances::RELATION_ID,
                route_decisions::RELATION_ID,
                structural_assessments::RELATION_ID,
            ]),
            super::RunRequest::Simulation(_) => {
                ids.push(computation_runs::RELATION_ID);
                if let Ok(super::RunReport::Simulation(trajectory)) = &self.report {
                    ids.extend(trajectory.table_ids());
                }
            }
            super::RunRequest::Fit(_) => ids.extend([
                computation_runs::RELATION_ID,
                fit_variables::RELATION_ID,
                fit_constraints::RELATION_ID,
                fit_parameters::RELATION_ID,
                fit_observations::RELATION_ID,
                response_sensitivities::RELATION_ID,
                solve_metrics::RELATION_ID,
                infeasibility_certificates::RELATION_ID,
                modeling_checks::RELATION_ID,
                modeling_reports::RELATION_ID,
                solve_strategy_events::RELATION_ID,
                solve_strategy_products::RELATION_ID,
                local_validity::RELATION_ID,
                parameter_covariances::RELATION_ID,
                parameter_intervals::RELATION_ID,
                profile_points::RELATION_ID,
                response_directions::RELATION_ID,
                propagated_covariances::RELATION_ID,
            ]),
            #[cfg(feature = "solver-diffsol")]
            super::RunRequest::Shooting { .. } => {
                ids.extend([
                    computation_runs::RELATION_ID,
                    solve_variables::RELATION_ID,
                    solve_constraints::RELATION_ID,
                    solve_metrics::RELATION_ID,
                    solve_strategy_events::RELATION_ID,
                    solve_strategy_products::RELATION_ID,
                ]);
                if let Ok(super::RunReport::Shooting(report)) = &self.report
                    && report.trajectory.is_some()
                {
                    ids.extend(super::ModelingTrajectory::relation_ids());
                }
            }
        }
        ids.sort_unstable();
        ids.dedup();
        ids
    }
    /// Name discovery requires no payload allocation or field validation.
    pub fn table_names(&self) -> Vec<String> {
        self.table_ids()
            .iter()
            .filter_map(|id| self.runtime.registry.relation_by_id(*id))
            .map(|spec| format!("{}.{}", spec.key.namespace.as_str(), spec.key.name))
            .collect()
    }
    /// Complete convenience access independently aggregates every advertised relation.
    pub fn tables(&self) -> Result<BTreeMap<SemanticId, FieldCheckedBatch>, Arc<WorkflowError>> {
        let ids = self.table_ids();
        let bytes = ids
            .len()
            .checked_mul(128)
            .ok_or_else(|| Arc::new(contract("result map extent")))?;
        let owner = self
            .runtime
            .shared
            .math()
            .reserve("result:complete-map", bytes)
            .map_err(|e| Arc::new(WorkflowError::Math(e)))?;
        ids.into_iter()
            .map(|id| {
                self.table_by_id(id)
                    .map(|table| (id, table.with_export_owner(owner.clone())))
            })
            .collect()
    }
    /// A retained checked table can outlive the result, run handle and model revision.
    pub fn table(&self, name: &str) -> Result<FieldCheckedBatch, Arc<WorkflowError>> {
        let id = self
            .runtime
            .registry
            .relation(name)
            .ok_or_else(|| Arc::new(contract("unknown result relation")))?
            .id;
        self.table_by_id(id)
    }
    fn table_by_id(&self, id: SemanticId) -> Result<FieldCheckedBatch, Arc<WorkflowError>> {
        super::result_export::collect(
            self.cursor_by_id(id, 1024, super::ResultOrder::Public)?,
            &self.runtime,
            id,
        )
    }
    /// Independent bounded traversal of one advertised relation.
    pub fn table_cursor(
        &self,
        name: &str,
        rows: usize,
    ) -> Result<super::ResultCursor<'_>, Arc<WorkflowError>> {
        let id = self
            .runtime
            .registry
            .relation(name)
            .ok_or_else(|| Arc::new(contract("unknown result relation")))?
            .id;
        self.cursor_by_id(id, rows, super::ResultOrder::Public)
    }
    /// An owning selected cursor for foreign-language streams.
    pub fn into_table_cursor(
        self: Arc<Self>,
        name: &str,
        rows: usize,
    ) -> Result<super::ResultCursor<'static>, Arc<WorkflowError>> {
        let id = self
            .runtime
            .registry
            .relation(name)
            .ok_or_else(|| Arc::new(contract("unknown result relation")))?
            .id;
        if !self.table_ids().contains(&id) {
            return Err(Arc::new(contract("relation is not part of this run")));
        }
        if let Some(error) = self.encodings.failure(id) {
            return Err(error);
        }
        let spec = self
            .runtime
            .registry
            .relation_by_id(id)
            .ok_or_else(|| Arc::new(contract("result declaration absent")))?;
        let schema = pse_schema::arrow::relation_schema_ref(&self.runtime.registry, spec)
            .map_err(|e| Arc::new(relation(e.into())))?;
        super::ResultCursor::new(
            schema,
            id,
            rows,
            super::ResultOrder::Public,
            |projection| async move {
                self.encode(&projection)
                    .await
                    .map_err(|error| self.encodings.record(id, error))
            },
        )
        .map_err(Arc::new)
    }
    /// Select original row coordinates without materializing a complete relation.
    pub fn table_range(
        &self,
        name: &str,
        range: std::ops::Range<usize>,
        rows: usize,
    ) -> Result<super::ResultCursor<'_>, Arc<WorkflowError>> {
        let id = self
            .runtime
            .registry
            .relation(name)
            .ok_or_else(|| Arc::new(contract("unknown result relation")))?
            .id;
        self.cursor_by_id_range(id, rows, super::ResultOrder::Public, range)
    }
    pub(super) fn cursor_by_id(
        &self,
        id: SemanticId,
        rows: usize,
        order: super::ResultOrder,
    ) -> Result<super::ResultCursor<'_>, Arc<WorkflowError>> {
        self.cursor_by_id_range(id, rows, order, 0..usize::MAX)
    }
    fn cursor_by_id_range(
        &self,
        id: SemanticId,
        rows: usize,
        order: super::ResultOrder,
        range: std::ops::Range<usize>,
    ) -> Result<super::ResultCursor<'_>, Arc<WorkflowError>> {
        if !self.table_ids().contains(&id) {
            return Err(Arc::new(contract("relation is not part of this run")));
        }
        if let Some(error) = self.encodings.failure(id) {
            return Err(error);
        }
        let spec = self
            .runtime
            .registry
            .relation_by_id(id)
            .ok_or_else(|| Arc::new(contract("result declaration absent")))?;
        let schema =
            pse_schema::arrow::relation_schema_ref(&self.runtime.registry, spec).map_err(|e| {
                Arc::new(WorkflowError::Engine(
                    pse_relations::RelationError::Schema(e).into(),
                ))
            })?;
        super::ResultCursor::new_range(schema, id, rows, order, range, |projection| async move {
            self.encode(&projection)
                .await
                .map_err(|error| self.encodings.record(id, error))
        })
        .map_err(Arc::new)
    }
    async fn encode(
        &self,
        projection: &super::result_export::Projection,
    ) -> Result<(), WorkflowError> {
        use pse_relations::generated::runtime::{
            accuracy_goal_assessments, candidate_assessments, resolved_numerics, run_lineage,
        };
        let id = projection.relation;
        if [
            candidate_assessments::RELATION_ID,
            resolved_numerics::RELATION_ID,
            accuracy_goal_assessments::RELATION_ID,
            run_lineage::RELATION_ID,
        ]
        .contains(&id)
        {
            return self.project_common(projection).await;
        }
        match &self.request {
            super::RunRequest::Simulation(_) => self.encode_simulation(projection).await,
            super::RunRequest::Modeling(_) => self.encode_modeling(projection).await,
            super::RunRequest::Fit(_) => self.encode_fit(projection).await,
            #[cfg(feature = "solver-diffsol")]
            super::RunRequest::Shooting { .. } => self.encode_shooting(projection).await,
        }
    }
}

pub(super) async fn push_metric(
    builder: &mut super::result_export::Rows<'_, metrics::Row>,
    run_id: RunId,
    step: i64,
    namespace: &str,
    name: &str,
    value: &Metric,
) -> Result<(), WorkflowError> {
    if builder.skip_next() {
        return Ok(());
    }
    if !builder.wanted() {
        return Ok(());
    }
    let text = match value {
        Metric::Text(text) => text.len(),
        _ => 0,
    };
    let _copy = builder.working(&[(namespace.len(), 1), (name.len(), 1), (text, 1)])?;
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
        Metric::Real(_) => {
            row.kind = NativeMetricKind::Unavailable;
            row.unavailable =
                Some(pse_model::generated::enums::EvidenceUnavailableReason::Nonfinite);
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
    builder.push(row).await.map_err(relation)
}

/// Metrics projected from a step's retained report observations. The bounded
/// report's dropped count remains explicit; complete canonical event history is
/// streamed separately rather than hydrated into the joined report.
async fn push_events(
    builder: &mut super::result_export::Rows<'_, metrics::Row>,
    run_id: RunId,
    step: i64,
    native: &pse_backend_native::solve::SolveReport,
) -> Result<(), WorkflowError> {
    if !builder.wanted() {
        return Ok(());
    }
    let dropped = native.dropped_events.min(i64::MAX as u64) as i64;
    push_metric(
        builder,
        run_id,
        step,
        "progress",
        "dropped_events",
        &Metric::Integer(dropped),
    )
    .await?;
    for (i, event) in native.events.iter().enumerate() {
        let ns = format!("event.{i}.{}", event.phase);
        push_metric(
            builder,
            run_id,
            step,
            &ns,
            "elapsed_seconds",
            &Metric::Real(event.elapsed.as_secs_f64()),
        )
        .await?;
        for (key, value) in &event.values {
            push_metric(builder, run_id, step, &ns, key, value).await?;
        }
        // Retained incumbent evidence; complete canonical history is read
        // separately through the exact protected progress stream.
        if let Some(incumbent) = &event.incumbent {
            let real = |v: Option<f64>| {
                v.map_or(
                    Metric::Unavailable(
                        pse_backend_native::solve::UnavailableReason::NotApplicable,
                    ),
                    Metric::Real,
                )
            };
            for (key, value) in [
                ("objective", Metric::Real(incumbent.objective)),
                ("dual_bound", real(incumbent.dual_bound)),
                ("gap", real(incumbent.gap)),
                ("nodes", Metric::Integer(incumbent.nodes)),
                ("seconds", Metric::Real(incumbent.seconds)),
            ] {
                push_metric(builder, run_id, step, &ns, key, &value).await?;
            }
        }
    }
    Ok(())
}

/// A native step's typed infeasibility certificate, in original coordinates with its
/// verification (Plan 22 I11); `None` when the step exported no ray.
pub(super) async fn push_certificate(
    builder: &mut super::result_export::Rows<'_, certificates::Row>,
    run_id: RunId,
    step: i64,
    backend: pse_backend_native::solve::Backend,
    certificate: Option<&pse_backend_native::solve::InfeasibilityCertificate>,
) -> Result<(), WorkflowError> {
    if !builder.wanted() {
        return Ok(());
    }
    let Some(c) = certificate else {
        return Ok(());
    };
    if builder.skip_next() {
        return Ok(());
    }
    let _copy = builder.working(&[(
        c.ray.len(),
        size_of::<certificates::RuntimeInfeasibilityCertificatesFieldRayItem>(),
    )])?;
    builder
        .push(certificates::Row {
            run_id,
            step,
            backend,
            kind: c.kind,
            accuracy: c.accuracy,
            ray: c
                .ray
                .iter()
                .map(
                    |e| certificates::RuntimeInfeasibilityCertificatesFieldRayItem {
                        coordinate: e.coordinate,
                        source_id: e.id,
                        value: e.value,
                    },
                )
                .collect(),
            verification: c.verification.map(|v| {
                certificates::RuntimeInfeasibilityCertificatesFieldVerification {
                    residual: v.residual,
                    objective: v.objective,
                    cone: v.cone,
                    margin: v.margin,
                    tolerance: v.tolerance,
                    verified: v.verified,
                }
            }),
        })
        .await
        .map_err(relation)
}

fn warm_snapshot_extent(
    seed: &pse_backend_native::solve::WarmStart,
) -> Result<usize, WorkflowError> {
    use pse_backend_native::solve::WarmPayload;
    let add = |a: usize, b: usize| {
        a.checked_add(b)
            .ok_or_else(|| contract("seed snapshot copy extent"))
    };
    let count = match &seed.payload {
        WarmPayload::Root(primal) => primal.len(),
        WarmPayload::Nlp {
            primal,
            bounds,
            rows,
            working,
            ..
        } => {
            let mut n = primal.len();
            if let Some((lower, upper)) = bounds {
                n = add(add(n, lower.len())?, upper.len())?;
            }
            if let Some(rows) = rows {
                n = add(n, rows.len())?;
            }
            #[cfg(feature = "solver-pounce")]
            if let Some(working) = working {
                n = add(
                    add(n, working.active.bounds.len())?,
                    working.active.constraints.len(),
                )?;
            }
            #[cfg(not(feature = "solver-pounce"))]
            let _ = working;
            n
        }
        WarmPayload::Highs {
            primal,
            dual,
            basis,
        } => {
            let mut n = primal.as_ref().map_or(0, Vec::len);
            if let Some((columns, rows)) = dual {
                n = add(add(n, columns.len())?, rows.len())?;
            }
            if let Some(basis) = basis {
                n = add(add(n, basis.columns.len())?, basis.rows.len())?;
            }
            n
        }
    };
    count
        .checked_mul(2048)
        .and_then(|n| n.checked_add(16384))
        .ok_or_else(|| contract("seed snapshot copy extent"))
}
fn start_snapshot_extent(
    start: &pse_backend_native::solve::StartReceipt,
) -> Result<usize, WorkflowError> {
    let seed = start
        .seed
        .as_ref()
        .map_or(Ok(0usize), warm_snapshot_extent)?;
    start
        .sparse_seed
        .as_ref()
        .map_or(0, |seed| seed.len())
        .checked_mul(2048)
        .and_then(|n| {
            start
                .transformations
                .len()
                .checked_mul(1024)
                .and_then(|transformations| n.checked_add(transformations))
        })
        .and_then(|n| n.checked_add(seed))
        .and_then(|n| n.checked_add(16384))
        .ok_or_else(|| contract("start snapshot copy extent"))
}
pub(super) async fn push_metric_lazy(
    builder: &mut super::result_export::Rows<'_, metrics::Row>,
    run_id: RunId,
    step: i64,
    namespace: &str,
    name: &str,
    bytes: usize,
    value: impl FnOnce() -> Result<Metric, WorkflowError> + Send,
) -> Result<(), WorkflowError> {
    if !builder.wanted() || builder.skip_next() {
        return Ok(());
    }
    let _copy = builder.working(&[(bytes, 1)])?;
    let value = value()?;
    push_metric(builder, run_id, step, namespace, name, &value).await
}

/// Common native options, metrics, progress and presolve receipt.
pub(super) async fn push_native_metrics(
    builder: &mut super::result_export::Rows<'_, metrics::Row>,
    run_id: RunId,
    step: i64,
    native: &pse_backend_native::solve::SolveReport,
) -> Result<(), WorkflowError> {
    if !builder.wanted() {
        return Ok(());
    }
    push_metric_lazy(
        builder,
        run_id,
        step,
        "qualification",
        "numerical",
        4096,
        || Ok(Metric::Text(native.qualification.as_str().into())),
    )
    .await?;
    if let Some(start) = &native.start_receipt
        && !builder.skip_next()
    {
        push_metric_lazy(
            builder,
            run_id,
            step,
            "start",
            "request",
            start_snapshot_extent(start)?,
            || Ok(Metric::Text(start.snapshot().to_string())),
        )
        .await?;
    }
    if let Some(seed) = &native.warm_start
        && !builder.skip_next()
    {
        push_metric_lazy(
            builder,
            run_id,
            step,
            "start",
            "available",
            warm_snapshot_extent(seed)?,
            || {
                serde_json::to_string(&seed.snapshot())
                    .map(Metric::Text)
                    .map_err(|error| contract(format!("seed snapshot encoding: {error}")))
            },
        )
        .await?;
    }
    for (name, value) in &native.metrics {
        push_metric(builder, run_id, step, "metric", name, value).await?;
    }
    for (name, value) in &native.provenance {
        push_metric_lazy(
            builder,
            run_id,
            step,
            "native_provenance",
            name,
            value.len(),
            || Ok(Metric::Text(value.clone())),
        )
        .await?;
    }
    for (namespace, options) in [
        ("option", &native.options),
        ("native_default", &native.native_defaults),
    ] {
        for (name, value) in options {
            if builder.skip_next() {
                continue;
            }
            let _copy = builder.working(&[(
                match value {
                    OptionValue::Text(text) => text.len(),
                    _ => 0,
                },
                1,
            )])?;
            let v = match value {
                OptionValue::Real(v) => Metric::Real(*v),
                OptionValue::Integer(v) => Metric::Integer(i64::from(*v)),
                OptionValue::Text(v) => Metric::Text(v.clone()),
                OptionValue::Bool(v) => Metric::Bool(*v),
            };
            push_metric(builder, run_id, step, namespace, name, &v).await?;
        }
    }
    push_events(builder, run_id, step, native).await?;
    #[cfg(feature = "solver-highs")]
    if let Some(d) = &native.highs_diagnostics {
        for (name, message) in &d.unavailable {
            push_metric_lazy(
                builder,
                run_id,
                step,
                "highs.unavailable",
                name,
                message.len(),
                || Ok(Metric::Text(message.clone())),
            )
            .await?;
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
                    )
                    .await?;
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
            )
            .await?;
            for (side, rows) in [("column", &iis.columns), ("row", &iis.rows)] {
                for (id, code) in rows {
                    push_metric(
                        builder,
                        run_id,
                        step,
                        "highs.iis",
                        &format!("{side}.{}", id.to_hex()),
                        &Metric::Integer(i64::from(*code)),
                    )
                    .await?;
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
                    )
                    .await?;
                }
            }
        }
        for (family, ranges) in d.ranging.iter().flat_map(|m| m.iter()) {
            for (i, id) in ranges.ids.iter().enumerate() {
                let ns = format!("highs.ranging.{family}.{}", id.to_hex());
                for (name, value) in [
                    ("value", ranges.value[i]),
                    ("objective", ranges.objective[i]),
                ] {
                    push_metric(builder, run_id, step, &ns, name, &Metric::Real(value)).await?;
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
                    )
                    .await?;
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
            )
            .await?;
            push_metric(
                builder,
                run_id,
                step,
                "highs.relaxation",
                "restored_model_code",
                &Metric::Integer(r.restored_status.code),
            )
            .await?;
            push_metric_lazy(
                builder,
                run_id,
                step,
                "highs.relaxation",
                "restored_model_status",
                r.restored_status.name.len(),
                || Ok(Metric::Text(r.restored_status.name.clone())),
            )
            .await?;
            if let Some(penalty) = r.penalty {
                push_metric(
                    builder,
                    run_id,
                    step,
                    "highs.relaxation",
                    "weighted_penalty",
                    &Metric::Real(penalty),
                )
                .await?;
            }
            if let Some(primal) = &r.primal {
                for (i, value) in primal.iter().enumerate() {
                    push_metric(
                        builder,
                        run_id,
                        step,
                        "highs.relaxation",
                        &format!("primal.{i}"),
                        &Metric::Real(*value),
                    )
                    .await?;
                }
            }
        }
    }
    if let Some(p) = &native.preprocessing {
        if let Some(proof) = &p.proof {
            let kind = proof.kind();
            push_metric_lazy(
                builder,
                run_id,
                step,
                "preprocessing_proof",
                "kind",
                4096,
                || Ok(Metric::Text(kind.into())),
            )
            .await?;
            push_metric_lazy(
                builder,
                run_id,
                step,
                "preprocessing_proof",
                "normalization",
                4096,
                || Ok(Metric::Text(proof.normalization.to_prefixed())),
            )
            .await?;
            if let Some(id) = proof.witness_row {
                push_metric_lazy(
                    builder,
                    run_id,
                    step,
                    "preprocessing_proof",
                    "witness_row",
                    4096,
                    || Ok(Metric::Text(id.to_hex())),
                )
                .await?;
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
                    )
                    .await?;
                }
            }
            for (index, (row, instance, output)) in proof.contributions.iter().enumerate() {
                push_metric_lazy(
                    builder,
                    run_id,
                    step,
                    "preprocessing_proof",
                    &format!("contribution.{index}"),
                    4096,
                    || {
                        Ok(Metric::Text(format!(
                            "row={};instance={};output={output}",
                            row.to_hex(),
                            instance.to_hex()
                        )))
                    },
                )
                .await?;
            }
        }
        for (name, value) in &p.diagnostics {
            push_metric_lazy(
                builder,
                run_id,
                step,
                "preprocessing",
                name,
                value.len(),
                || Ok(Metric::Text(value.clone())),
            )
            .await?;
        }
        for (name, value) in &p.passes {
            let prefix = name.as_str();
            if let Some(reason) = &value.reason {
                push_metric_lazy(
                    builder,
                    run_id,
                    step,
                    "preprocessing_pass",
                    &format!("{prefix}.reason"),
                    reason.len(),
                    || Ok(Metric::Text(reason.clone())),
                )
                .await?;
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
                )
                .await?;
            }
        }
    }
    Ok(())
}
