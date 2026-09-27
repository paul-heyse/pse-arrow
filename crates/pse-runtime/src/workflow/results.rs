// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Registry-generated physical observations retain Arrow allocation ownership.
use super::{RunResult, WorkflowError, contract, relation};
use pse_backend_native::solve::{Metric, OptionValue};
use pse_ids::SemanticId;
use pse_relations::{
    columnar::FieldCheckedBatch,
    generated::{
        enums::NativeMetricKind,
        runtime::{
            solve_metrics as metrics,
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
        match &self.request {
            super::RunRequest::Simulation(_) => self.encode_simulation(),
            super::RunRequest::Modeling(_) => self.encode_modeling(),
            super::RunRequest::Fit(_) => self.encode_fit(),
        }
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
                "restored_model_code",
                &Metric::Integer(r.restored_status.code),
            )?;
            push_metric(
                builder,
                run_id,
                step,
                "highs.relaxation",
                "restored_model_status",
                &Metric::Text(r.restored_status.name.clone()),
            )?;
            if let Some(penalty) = r.penalty {
                push_metric(
                    builder,
                    run_id,
                    step,
                    "highs.relaxation",
                    "weighted_penalty",
                    &Metric::Real(penalty),
                )?;
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
            let prefix = name.as_str();
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
