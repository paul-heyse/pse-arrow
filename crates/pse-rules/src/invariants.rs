// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! P2 evaluates declared rules over unpublished candidates, retaining all violating keys.
use crate::errmap::internal;
use crate::{
    RuleError, derivations,
    exec::{RuleFired, execute},
    plan::{PortBinding, compile},
};
use datafusion::arrow::array::RecordBatch;
use pse_catalog::session::{PlanObservation, SnapshotSession};
use pse_ids::{CancellationToken, ContentHash, SemanticId, SnapshotId};
use pse_schema::{
    Registry,
    model::{Cell, InvariantSpec, RelationKey, Severity, SnapshotClass},
};
use std::collections::BTreeMap;

/// Which primitive candidate contract P2 is admitting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InvariantScope {
    /// Authored/reference model facts.
    Model,
    /// Case rows plus their pinned model/reference context.
    Case,
    /// The registry's own declarations.
    Registry,
    /// Every relation explicitly supplied as a candidate.
    Candidate,
    /// Local sidecar predicates only; typed receipt admission owns cross-artifact
    /// references and never treats the individual artifact as a complete snapshot.
    SidecarRelation,
}
/// Actual P2 outputs, before a model/case snapshot may be minted.
#[derive(Debug, Default)]
pub struct InvariantReport {
    /// Generated diagnostic relation batches.
    pub findings: Vec<RecordBatch>,
    /// Generated undecided relation batches.
    pub undecided: Vec<RecordBatch>,
    /// Generated provenance batches, when snapshot lineage is available.
    pub derivations: Vec<RecordBatch>,
    /// Diagnostic-only plan attribution.
    pub explain: Vec<(SemanticId, String)>,
    /// Actual execution observations.
    pub rules_fired: Vec<RuleFired>,
    /// Actual optimized plans and optimizer callback order for all executed rule plans.
    pub plans: Vec<PlanObservation>,
    /// Error severity findings, counted from actual rule results.
    pub error_count: usize,
    /// Complete actual relation bindings, including empty relations needed for negative claims.
    pub dependency_bindings: Vec<RelationKey>,
}

/// Optional attribution from an actual enclosing execution; never an admission certificate.
#[derive(Clone, Copy, Debug)]
pub struct RunEvidence {
    /// Existing stage key or plan-evidence fingerprint supplied by that execution.
    pub fingerprint: ContentHash,
}

/// Execute standalone candidate validation without inventing execution attribution.
///
/// # Errors
/// Missing bindings, invalid plans, rejected values, resource exhaustion or cancellation.
pub async fn run_invariants(
    candidates: &BTreeMap<RelationKey, RecordBatch>,
    session: &SnapshotSession,
    registry: &Registry,
    scope: InvariantScope,
    subject: Option<SnapshotId>,
    cancel: &CancellationToken,
) -> Result<InvariantReport, RuleError> {
    run_invariants_with_evidence(candidates, session, registry, scope, subject, None, cancel).await
}

/// Execute applicable invariants with optional real execution attribution.
///
/// # Errors
/// Requires complete exact session bindings and executes actual rule predicates.
pub async fn run_invariants_with_evidence(
    candidates: &BTreeMap<RelationKey, RecordBatch>,
    session: &SnapshotSession,
    registry: &Registry,
    scope: InvariantScope,
    subject: Option<SnapshotId>,
    evidence: Option<RunEvidence>,
    cancel: &CancellationToken,
) -> Result<InvariantReport, RuleError> {
    session.validate_bindings(candidates)?;
    let mut report = InvariantReport {
        dependency_bindings: candidates.keys().copied().collect(),
        ..InvariantReport::default()
    };
    let context = OutputContext {
        registry,
        session,
        subject,
        evidence,
        cancel,
    };
    for invariant in registry.invariants() {
        let target = registry
            .relation(&invariant.relation)
            .ok_or_else(|| internal("invariant target absent"))?;
        if !candidates.contains_key(&target.key) {
            continue;
        }
        let applies = match scope {
            InvariantScope::Candidate => true,
            InvariantScope::SidecarRelation => target.snapshot_class == SnapshotClass::Sidecar,
            InvariantScope::Model => target.snapshot_class == SnapshotClass::Model,
            InvariantScope::Case => target.snapshot_class == SnapshotClass::Case,
            InvariantScope::Registry => {
                target.key.namespace == pse_schema::model::Namespace::Reference
            }
        };
        if !applies {
            continue;
        }
        let rule = registry
            .rule(&invariant.rule)
            .ok_or_else(|| internal("invariant rule absent"))?;
        if matches!(scope, InvariantScope::SidecarRelation)
            && rule
                .plan
                .dependencies()
                .iter()
                .any(|(name, _, _)| *name != target.key.qualified_name())
        {
            continue;
        }
        let binding = bind_invariant(invariant, rule, candidates, registry)?;
        let compiled = compile(rule, &binding, session, registry)?;
        let outcome = execute(&compiled, session, registry, cancel).await?;
        let names = rule
            .plan
            .dependencies()
            .into_iter()
            .map(|(name, _, _)| name)
            .collect::<std::collections::BTreeSet<_>>();
        let dependencies = candidates
            .iter()
            .filter(|(key, _)| names.contains(key.qualified_name().as_str()))
            .map(|(_, batch)| batch);
        let _workspace = crate::allocation::reserve_evidence(
            outcome.head.iter().chain(&outcome.undecided),
            dependencies,
            session,
            cancel,
        )?;
        let mut keys = vec![];
        let mut unknown = vec![];
        for batch in &outcome.head {
            keys.extend(pse_relations::cells::decode_columns(registry, batch)?);
        }
        for batch in &outcome.undecided {
            unknown.extend(pse_relations::cells::decode_columns(registry, batch)?);
        }
        if invariant.severity == Severity::Error {
            report.error_count += keys.len() + unknown.len();
        }
        let supporting = if keys.is_empty() && unknown.is_empty() {
            vec![]
        } else {
            supporting_rows(rule, candidates, registry)?
        };
        append_outputs(
            &mut report,
            invariant,
            &keys,
            &unknown,
            &supporting,
            &context,
        )?;
        report.explain.push((rule.id, outcome.explain_pgjson));
        report.rules_fired.extend(outcome.rules_fired);
        report.plans.extend(outcome.plans);
    }
    Ok(report)
}
struct OutputContext<'a> {
    registry: &'a Registry,
    session: &'a SnapshotSession,
    subject: Option<SnapshotId>,
    evidence: Option<RunEvidence>,
    cancel: &'a CancellationToken,
}
fn supporting_rows(
    rule: &pse_schema::model::RuleSpec,
    candidates: &BTreeMap<RelationKey, RecordBatch>,
    registry: &Registry,
) -> Result<Vec<Cell>, RuleError> {
    let names = rule
        .plan
        .dependencies()
        .into_iter()
        .map(|(name, _, _)| name)
        .collect::<std::collections::BTreeSet<_>>();
    let mut supporting = vec![];
    for name in names {
        let spec = registry
            .relation(name)
            .ok_or_else(|| internal("support relation absent"))?;
        let batch = candidates
            .get(&spec.key)
            .ok_or_else(|| internal("support binding absent"))?;
        for row in pse_relations::cells::decode_columns(registry, batch)? {
            supporting.push(Cell::Struct(vec![
                Cell::Id(spec.id),
                Cell::text(derivations::encode_relation_key(spec, registry, &row)?),
            ]));
        }
    }
    Ok(supporting)
}
fn append_outputs(
    report: &mut InvariantReport,
    invariant: &InvariantSpec,
    keys: &[Vec<Cell>],
    unknown: &[Vec<Cell>],
    supporting: &[Cell],
    context: &OutputContext<'_>,
) -> Result<(), RuleError> {
    let registry = context.registry;
    let rule = registry
        .rule(&invariant.rule)
        .ok_or_else(|| internal("invariant rule absent"))?;
    let target = registry
        .relation(&invariant.relation)
        .ok_or_else(|| internal("invariant target absent"))?;
    let mut findings = vec![];
    let mut undecided = vec![];
    let mut provenance = vec![];
    for (values, is_unknown) in keys
        .iter()
        .map(|row| (row, false))
        .chain(unknown.iter().map(|row| (row, true)))
    {
        let key = derivations::encode_key(rule, registry, values)?;
        let id = derivations::finding_id(invariant.id, &key);
        let derivation = pse_ids::named_id(rule.id, &format!("derivation:{key}"));
        let reason = if is_unknown {
            format!(
                "{}: invariant predicate is unknown",
                invariant.qualified_name()
            )
        } else {
            invariant.doc.to_owned()
        };
        findings.push(vec![
            Cell::Id(id),
            context
                .subject
                .map_or(Cell::Null, |id| Cell::Hash(id.content_hash())),
            Cell::Null,
            Cell::Id(invariant.id),
            Cell::Enum(invariant.severity.as_str()),
            Cell::List(
                values
                    .iter()
                    .filter(|value| matches!(value, Cell::Id(_)))
                    .cloned()
                    .collect(),
            ),
            Cell::text(&key),
            Cell::text(&reason),
            Cell::List(vec![]),
        ]);
        provenance.push(vec![
            Cell::Id(derivation),
            Cell::Id(target.id),
            Cell::text(&key),
            Cell::Id(rule.id),
            Cell::Null,
            Cell::List(supporting.to_vec()),
            context
                .subject
                .map_or(Cell::Null, |id| Cell::Hash(id.content_hash())),
            context
                .evidence
                .map_or(Cell::Null, |value| Cell::Hash(value.fingerprint)),
        ]);
        if is_unknown {
            undecided.push(vec![
                Cell::Id(pse_ids::named_id(rule.id, &format!("undecided:{key}"))),
                Cell::Id(rule.id),
                Cell::Id(target.id),
                Cell::text(&key),
                Cell::Enum("unknown"),
                Cell::text(reason),
                Cell::List(supporting.to_vec()),
                Cell::Id(derivation),
            ]);
        }
    }
    append_batch(
        &mut report.findings,
        "runtime.diagnostics_findings",
        &findings,
        context,
    )?;
    append_batch(
        &mut report.undecided,
        "inferred.undecided",
        &undecided,
        context,
    )?;
    append_batch(
        &mut report.derivations,
        "provenance.derivations",
        &provenance,
        context,
    )
}
fn append_batch(
    output: &mut Vec<RecordBatch>,
    name: &str,
    rows: &[Vec<Cell>],
    context: &OutputContext<'_>,
) -> Result<(), RuleError> {
    if rows.is_empty() {
        return Ok(());
    }
    let spec = context
        .registry
        .relation(name)
        .ok_or_else(|| internal("diagnostic head undeclared"))?;
    output.push(pse_relations::cells::batch_from_cells_owned(
        context.registry,
        spec,
        rows,
        context.session.reserver(),
        context.cancel,
    )?);
    Ok(())
}

fn bind_invariant(
    invariant: &InvariantSpec,
    rule: &pse_schema::model::RuleSpec,
    candidates: &BTreeMap<RelationKey, RecordBatch>,
    registry: &Registry,
) -> Result<PortBinding, RuleError> {
    let mut binding = PortBinding::default();
    for (relation, port, _) in rule.plan.dependencies() {
        let spec = registry
            .relation(relation)
            .ok_or_else(|| internal("invariant dependency undeclared"))?;
        if !candidates.contains_key(&spec.key) {
            return Err(internal(format!(
                "invariant {} lacks explicit dependency {}",
                invariant.qualified_name(),
                spec.key
            )));
        }
        if binding
            .ports
            .insert(port.to_owned(), spec.key)
            .is_some_and(|prior| prior != spec.key)
        {
            return Err(internal(
                "one rule input port is bound to different relations",
            ));
        }
    }
    Ok(binding)
}
