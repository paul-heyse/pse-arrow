// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete typed terminal observations through the existing admitted sidecar path.
mod failures;
#[cfg(test)]
mod tests;
use crate::{CompilerError, PassStatus, passes::dag::invalid};
use miette::Diagnostic;
use pse_catalog::{
    Catalog,
    session::PlanObservation,
    store::{publish::RelationDraft, sidecar::SidecarArtifact},
};
use pse_ids::{CancellationToken, ContentHash, SemanticId, SnapshotId};
use pse_relations::RecordBatch;
use pse_schema::model::{Cell, PassSpec};
use std::{
    fmt::Write,
    sync::Arc,
    time::{Duration, Instant},
};

/// A driver-owned attempt, identified before cancellation checks or execution begin.
#[derive(Debug)]
pub(crate) struct Attempt {
    pub(crate) id: SemanticId,
    pass: SemanticId,
    version: &'static str,
    started: Instant,
}
/// Borrowed actual observations; no identity or digest substitutes for these values.
#[derive(Debug)]
pub(crate) struct Observation<'a> {
    pub(crate) status: PassStatus,
    pub(crate) input: Option<SnapshotId>,
    pub(crate) output: Option<SnapshotId>,
    pub(crate) engine: Option<ContentHash>,
    pub(crate) findings: &'a [RecordBatch],
    pub(crate) plans: &'a [PlanObservation],
}
impl Observation<'_> {
    pub(crate) const fn success(input: Option<SnapshotId>) -> Self {
        Self {
            status: PassStatus::Ok,
            input,
            output: None,
            engine: None,
            findings: &[],
            plans: &[],
        }
    }
}
impl Attempt {
    pub(crate) fn new(pass: &PassSpec) -> Self {
        Self {
            id: pse_authoring::ids::uuid_v7(),
            pass: pass.id,
            version: pass.version,
            started: Instant::now(),
        }
    }
    pub(crate) async fn success(
        &self,
        catalog: &Catalog,
        observation: Observation<'_>,
        cancel: &CancellationToken,
    ) -> Result<SidecarArtifact, CompilerError> {
        if cancel.is_cancelled() {
            return Err(self
                .failed(
                    catalog,
                    observation.input,
                    CompilerError::Cancelled {
                        findings: observation.findings.to_vec(),
                    },
                )
                .await);
        }
        let output = observation.output;
        publish(catalog, self, observation, None, cancel)
            .await
            .map_err(|source| CompilerError::SuccessRecording {
                pass_run_id: self.id,
                output,
                source: Box::new(source),
            })
    }
    pub(crate) async fn failed(
        &self,
        catalog: &Catalog,
        input: Option<SnapshotId>,
        original: CompilerError,
    ) -> CompilerError {
        // The work token may already be cancelled. Cleanup has its own finite deadline
        // and still uses the catalog's actual shared memory reservation limit.
        let cleanup = CancellationToken::new();
        let observation = Observation::success(input);
        let result = tokio::time::timeout(
            Duration::from_secs(10),
            publish(catalog, self, observation, Some(&original), &cleanup),
        )
        .await;
        let result = result.unwrap_or_else(|_| {
            cleanup.cancel();
            Err(CompilerError::Infrastructure {
                op: "terminal attempt recording".to_owned(),
                detail: "ten-second cleanup deadline exceeded".to_owned(),
            })
        });
        match result {
            Ok(record) => CompilerError::AttemptFailed {
                pass_run_id: self.id,
                record: Box::new(record),
                source: Box::new(original),
            },
            Err(recording) => CompilerError::TerminalRecording {
                pass_run_id: self.id,
                errors: vec![original, recording],
            },
        }
    }
}

async fn publish(
    catalog: &Catalog,
    attempt: &Attempt,
    observation: Observation<'_>,
    error: Option<&CompilerError>,
    cancel: &CancellationToken,
) -> Result<SidecarArtifact, CompilerError> {
    let mut workspace = catalog.reserver().open("compiler:terminal-record");
    workspace
        .try_grow(extent(catalog, &observation, error)?)
        .map_err(pse_ids::CanonError::from)?;
    let FindingReport {
        findings,
        status,
        failure_class,
    } = collect_findings(catalog.registry(), &observation, error)?;
    let (explain, rules) = plans(observation.plans)?;
    let count = u64::try_from(findings.len()).map_err(|_| invalid("finding count overflow"))?;
    publish_rows(
        catalog,
        "provenance.pass_records",
        &[vec![
            Cell::Id(attempt.id),
            Cell::Id(attempt.pass),
            Cell::text(attempt.version),
            observation
                .input
                .map_or(Cell::Null, |id| Cell::Hash(id.content_hash())),
            observation
                .output
                .map_or(Cell::Null, |id| Cell::Hash(id.content_hash())),
            observation.engine.map_or(Cell::Null, Cell::Hash),
            Cell::List(vec![]),
            Cell::List(explain),
            Cell::List(rules),
            Cell::F64(attempt.started.elapsed().as_secs_f64() * 1000.0),
            Cell::U64(count),
            Cell::Enum(status.as_str()),
            Cell::List(findings),
            failure_class.map_or(Cell::Null, Cell::Enum),
        ]],
        cancel,
    )
    .await
}
struct FindingReport {
    findings: Vec<Cell>,
    status: PassStatus,
    failure_class: Option<&'static str>,
}
fn collect_findings(
    registry: &pse_schema::Registry,
    observation: &Observation<'_>,
    error: Option<&CompilerError>,
) -> Result<FindingReport, CompilerError> {
    let mut findings = decode_findings(registry, observation.findings)?;
    let mut failure_class = None;
    let mut all_cancelled = true;
    if let Some(error) = error {
        failures::visit(error, registry, &mut |class, diagnostic, batches| {
            if failure_class.is_none()
                || (failure_class == Some("runtime.cancelled") && class != "runtime.cancelled")
            {
                failure_class = Some(class);
            }
            all_cancelled &= class == "runtime.cancelled";
            let supplied = if let Some(batches) = batches {
                decode_findings(registry, batches)?
            } else {
                vec![]
            };
            if supplied.is_empty() {
                findings.push(execution_finding(
                    diagnostic,
                    class,
                    observation.input,
                    error,
                ));
            } else {
                findings.extend(supplied);
            }
            Ok(())
        })?;
        if failure_class.is_none() {
            return Err(invalid("failure contained no actual diagnostic leaves"));
        }
    }
    let status = if error.is_some() {
        if all_cancelled {
            PassStatus::Cancelled
        } else {
            PassStatus::Failed
        }
    } else {
        observation.status
    };
    Ok(FindingReport {
        findings,
        status,
        failure_class,
    })
}
fn decode_findings(
    registry: &pse_schema::Registry,
    batches: &[RecordBatch],
) -> Result<Vec<Cell>, CompilerError> {
    if batches.is_empty() {
        return Ok(Vec::new());
    }
    let spec = registry
        .relation("runtime.diagnostics_findings")
        .ok_or_else(|| invalid("diagnostic relation absent"))?;
    let mut rows = Vec::new();
    for batch in batches {
        pse_relations::validate::validate_batch(registry, spec, batch)
            .map_err(|errors| pse_relations::RelationError::Validation { errors })?;
        rows.extend(
            pse_relations::cells::cells_from_batch(registry, spec, batch)?
                .into_iter()
                .map(Cell::Struct),
        );
    }
    Ok(rows)
}
fn execution_finding(
    error: &dyn Diagnostic,
    class: &'static str,
    input: Option<SnapshotId>,
    context: &CompilerError,
) -> Cell {
    let values = serde_json::json!({"failure_class": class, "diagnostic_code": error.code().map(|code| code.to_string()), "attempt_error": context.to_string()});
    Cell::Struct(vec![
        Cell::Id(pse_authoring::ids::uuid_v7()),
        input.map_or(Cell::Null, |id| Cell::Hash(id.content_hash())),
        Cell::Null,
        Cell::Null,
        Cell::Enum("error"),
        Cell::List(vec![]),
        Cell::text(values.to_string()),
        Cell::text(error.to_string()),
        Cell::List(
            error
                .help()
                .map(|help| Cell::text(help.to_string()))
                .into_iter()
                .collect(),
        ),
    ])
}
fn plans(plans: &[PlanObservation]) -> Result<(Vec<Cell>, Vec<Cell>), CompilerError> {
    let explain = plans
        .iter()
        .map(|plan| Cell::text(plan.explain_pgjson()))
        .collect();
    let mut rules = Vec::new();
    for (plan_ordinal, plan) in plans.iter().enumerate() {
        let plan_ordinal =
            u16::try_from(plan_ordinal).map_err(|_| invalid("too many observed plans"))?;
        for (ordinal, name) in plan.rules_fired().iter().enumerate() {
            let ordinal =
                u16::try_from(ordinal).map_err(|_| invalid("too many observed optimizer rules"))?;
            rules.push(Cell::Struct(vec![
                Cell::U64(u64::from(plan_ordinal)),
                Cell::text(name),
                Cell::U64(u64::from(ordinal)),
            ]));
        }
    }
    Ok((explain, rules))
}
fn extent(
    catalog: &Catalog,
    observation: &Observation<'_>,
    error: Option<&CompilerError>,
) -> Result<usize, CompilerError> {
    let mut bytes = 8192_usize;
    let mut add = |size: usize| -> Result<(), CompilerError> {
        bytes = bytes
            .checked_add(size)
            .ok_or_else(|| invalid("terminal record extent overflow"))?;
        Ok(())
    };
    for batch in observation.findings {
        add(pse_catalog::store::membership::validation_extent(batch)?)?;
    }
    for plan in observation.plans {
        add(plan.explain_pgjson().len().saturating_mul(8))?;
        for name in plan.rules_fired() {
            add(name.len().saturating_add(512))?;
        }
    }
    if let Some(error) = error {
        let context = rendered_len(error)?
            .checked_mul(8)
            .ok_or_else(|| invalid("diagnostic extent overflow"))?;
        failures::visit(error, catalog.registry(), &mut |_, diagnostic, batches| {
            add(context)?;
            add(rendered_len(diagnostic)?
                .checked_mul(8)
                .ok_or_else(|| invalid("diagnostic extent overflow"))?)?;
            add(8192)?;
            if let Some(batches) = batches {
                for batch in batches {
                    add(pse_catalog::store::membership::validation_extent(batch)?)?;
                }
            }
            Ok(())
        })?;
    }
    Ok(bytes)
}
fn rendered_len(value: &dyn std::fmt::Display) -> Result<usize, CompilerError> {
    struct Counter(usize);
    impl Write for Counter {
        fn write_str(&mut self, value: &str) -> std::fmt::Result {
            self.0 = self.0.checked_add(value.len()).ok_or(std::fmt::Error)?;
            Ok(())
        }
    }
    let mut counter = Counter(0);
    write!(&mut counter, "{value}").map_err(|_| invalid("diagnostic rendered extent overflow"))?;
    Ok(counter.0)
}
/// Publish rows under one declared sidecar contract; no handwritten Arrow schema.
/// # Errors
/// Unknown/non-sidecar contract, malformed actual values or bounded store failure.
pub async fn publish_rows(
    catalog: &Catalog,
    name: &str,
    rows: &[Vec<Cell>],
    cancel: &CancellationToken,
) -> Result<SidecarArtifact, CompilerError> {
    let spec = catalog
        .registry()
        .relation(name)
        .ok_or_else(|| invalid("sidecar relation absent"))?;
    let batch = pse_relations::cells::batch_from_cells_owned(
        catalog.registry(),
        spec,
        rows,
        catalog.reserver().as_ref(),
        cancel,
    )?;
    let contract = Arc::new(pse_catalog::RelationContract::from_spec(
        catalog.registry(),
        spec,
        pse_catalog::EncodingPolicy::IpcFile,
    )?);
    Ok(catalog
        .publish_sidecar(
            RelationDraft {
                contract,
                batches: vec![batch],
            },
            cancel,
        )
        .await?)
}
