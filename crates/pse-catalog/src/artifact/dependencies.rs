// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed dependencies of one retained immutable native composition. Generation
//! identities retain actual implementations; names and fingerprints cannot replace them.
use super::{ArtifactPlan, invalid};
use crate::{
    CatalogError,
    session::{RelationPlan, SnapshotSession},
};
use datafusion::{
    common::{ResolvedTableReference, TableReference},
    logical_expr::{LogicalPlan, LogicalPlanBuilder},
};
use deps::RuntimeNativeDependenciesFieldEvidence as Evidence;
use pse_ids::{CancellationToken, SemanticId};
use pse_relations::generated::{
    enums::NativeDependencyKind as Kind,
    runtime::{native_dependencies as deps, publications},
};
use pse_schema::model::provider::ProviderScope;
use std::collections::BTreeMap;

pub(super) fn capture(artifact: &ArtifactPlan) -> Result<Vec<deps::Row>, CatalogError> {
    let generation = artifact.session.implementation_generation();
    let mut rows = vec![fact(
        Kind::Operation,
        "native",
        "composition",
        None,
        Some(artifact.operation_id),
    )];
    let mut registry = fact(Kind::Contract, "native", "registry", None, None);
    registry.evidence =
        Evidence::from_fingerprint(deps::RuntimeNativeDependenciesFieldEvidenceFingerprint {
            value: artifact.session.registry().fingerprint(),
        });
    rows.push(registry);
    rows.push(fact(
        Kind::Observation,
        "native",
        "fresh_execution",
        Some(artifact.fresh_observations.to_string()),
        None,
    ));
    let semantics = artifact.session.semantic_inputs();
    for (name, value) in [
        ("datafusion", semantics.engine_version),
        ("arrow", semantics.arrow_version),
        ("profile", semantics.profile.version),
    ] {
        rows.push(fact(
            Kind::Operation,
            "implementation",
            name,
            Some(value),
            Some(generation),
        ));
    }
    for (name, value) in semantics.settings {
        rows.push(fact(Kind::Setting, "session", &name, value, None));
    }
    for (family, functions) in semantics.functions {
        for name in functions {
            rows.push(fact(Kind::Function, &family, &name, None, Some(generation)));
        }
    }
    for (family, rules) in [
        ("analyzer", semantics.profile.analyzer_rules),
        ("optimizer", semantics.profile.optimizer_rules),
        ("physical", semantics.profile.physical_optimizer_rules),
    ] {
        for (index, name) in rules.into_iter().enumerate() {
            rows.push(fact(
                Kind::Rule,
                family,
                &index.to_string(),
                Some(name),
                Some(generation),
            ));
        }
    }
    for policy in semantics.policies {
        policy_facts(policy, &mut rows);
    }
    source_facts(artifact, &mut rows)?;
    rows.sort_by(|a, b| (&a.kind, &a.scope, &a.name).cmp(&(&b.kind, &b.scope, &b.name)));
    rows.dedup();
    if rows.windows(2).any(|pair| {
        (&pair[0].kind, &pair[0].scope, &pair[0].name)
            == (&pair[1].kind, &pair[1].scope, &pair[1].name)
    }) {
        return Err(invalid("native dependency key has conflicting facts"));
    }
    Ok(rows)
}

fn policy_facts(policy: pse_schema::model::provider::ProviderPolicy, rows: &mut Vec<deps::Row>) {
    let scope = policy.id.to_string();
    rows.push(fact(
        Kind::Policy,
        &scope,
        "scope",
        Some(scope_name(&policy.scope)),
        Some(policy.id),
    ));
    for id in policy.requirements {
        rows.push(fact(
            Kind::Policy,
            &scope,
            &format!("requirement:{id}"),
            None,
            Some(id),
        ));
    }
    for effect in policy.effects {
        rows.push(fact(
            Kind::Policy,
            &scope,
            &format!("effect:{}", effect.as_str()),
            None,
            None,
        ));
    }
    for (kind, values) in [
        ("default", policy.defaults),
        ("required", policy.required_settings),
    ] {
        for (name, value) in values {
            rows.push(fact(
                Kind::Policy,
                &scope,
                &format!("{kind}:{name}"),
                Some(value),
                None,
            ));
        }
    }
}

fn source_facts(artifact: &ArtifactPlan, rows: &mut Vec<deps::Row>) -> Result<(), CatalogError> {
    let generation = artifact.operation_id;
    let mut selected = BTreeMap::new();
    let mut providers = BTreeMap::new();
    for output in artifact.outputs.values() {
        for member in artifact.session.selected_dependencies(&output.plan)? {
            let name = reference(&member).to_quoted_string();
            if selected
                .insert(name, member.clone())
                .is_some_and(|previous| previous != member)
            {
                return Err(invalid("conflicting exact input roles"));
            }
        }
        for binding in artifact.session.source_bindings(&output.plan)? {
            providers.insert(binding.reference.to_quoted_string(), binding);
        }
    }
    let mut consumption = super::consumption::columns(artifact)?;
    for (name, member) in selected {
        let mut row = fact(Kind::Input, "selected", &name, None, None);
        row.evidence = match consumption.remove(&name).filter(|columns| {
            artifact
                .session
                .registry()
                .relation_by_id(member.relation_id)
                .is_some_and(|spec| columns.len() < spec.columns.len())
        }) {
            Some(columns) => {
                Evidence::from_projection(super::consumption::projection(member, columns)?)
            }
            None => Evidence::from_selection(selection(member)?),
        };
        rows.push(row);
    }
    for (name, binding) in providers {
        rows.push(fact(
            Kind::Provider,
            &name,
            "implementation",
            None,
            Some(generation),
        ));
        for effect in &binding.effects {
            rows.push(fact(
                Kind::Observation,
                &name,
                effect.as_str(),
                None,
                Some(generation),
            ));
        }
    }
    for (catalog, resolution) in artifact.session.bindings.resolutions() {
        rows.push(fact(
            Kind::Scope,
            catalog,
            "coverage",
            Some(format!(
                "{:?}:{}",
                resolution.consistency, resolution.exhaustive
            )),
            Some(generation),
        ));
        for name in &resolution.references {
            let present = artifact
                .session
                .bindings
                .iter()
                .any(|(_, binding)| binding.reference == *name);
            rows.push(fact(
                Kind::Scope,
                catalog,
                &name.to_quoted_string(),
                Some(present.to_string()),
                Some(generation),
            ));
        }
    }
    Ok(())
}

fn fact(
    kind: Kind,
    scope: &str,
    name: &str,
    value: Option<String>,
    identity: Option<SemanticId>,
) -> deps::Row {
    let evidence = match (value, identity) {
        (Some(text), Some(identity)) => Evidence::from_identified_text(
            deps::RuntimeNativeDependenciesFieldEvidenceIdentifiedText { identity, text },
        ),
        (Some(value), None) => {
            Evidence::from_text(deps::RuntimeNativeDependenciesFieldEvidenceText { value })
        }
        (None, Some(value)) => {
            Evidence::from_identity(deps::RuntimeNativeDependenciesFieldEvidenceIdentity { value })
        }
        (None, None) if kind == Kind::Setting => Evidence::from_absent(),
        (None, None) => Evidence::from_present(),
    };
    deps::Row {
        kind,
        scope: scope.into(),
        name: name.into(),
        evidence,
    }
}
fn scope_name(scope: &ProviderScope) -> String {
    match scope {
        ProviderScope::Root => "root".into(),
        ProviderScope::Invocation => "invocation".into(),
        ProviderScope::Catalog(c) => format!(
            "catalog:{}",
            TableReference::bare(c.clone()).to_quoted_string()
        ),
        ProviderScope::Schema(c, s) => format!(
            "schema:{}",
            TableReference::partial(c.clone(), s.clone()).to_quoted_string()
        ),
        ProviderScope::Table(c, s, t) => format!(
            "table:{}",
            TableReference::full(c.clone(), s.clone(), t.clone()).to_quoted_string()
        ),
    }
}
fn reference(member: &publications::RuntimePublicationsFieldMembersItem) -> TableReference {
    TableReference::full(
        member.catalog_name.clone(),
        member.schema_name.clone(),
        member.table_name.clone(),
    )
}
fn selection(
    member: publications::RuntimePublicationsFieldMembersItem,
) -> Result<deps::RuntimeNativeDependenciesFieldEvidenceSelection, CatalogError> {
    use publications::RuntimePublicationsFieldMembersItemSelectionSelected as Selected;
    let selection = match member.selection.selected()? {
        Selected::Full => {
            deps::RuntimeNativeDependenciesFieldEvidenceSelectionSelection::from_full()
        }
        Selected::Revision(value) => {
            deps::RuntimeNativeDependenciesFieldEvidenceSelectionSelection::from_revision(
                deps::RuntimeNativeDependenciesFieldEvidenceSelectionSelectionRevision {
                    column: value.column.clone(),
                    revision_id: value.revision_id,
                },
            )
        }
    };
    Ok(deps::RuntimeNativeDependenciesFieldEvidenceSelection {
        catalog_name: member.catalog_name,
        schema_name: member.schema_name,
        table_name: member.table_name,
        relation_id: member.relation_id,
        relation_version: member.relation_version,
        contract_fingerprint: member.contract_fingerprint,
        table_uri: member.table_uri,
        delta_version: member.delta_version,
        selection,
    })
}

pub(super) fn bind(
    session: &SnapshotSession,
    role: &str,
    rows: &[deps::Row],
    cancel: &CancellationToken,
) -> Result<(SnapshotSession, RelationPlan), CatalogError> {
    let mut builder = deps::Builder::with_registry(session.registry(), rows.len())?;
    for row in rows {
        builder.push(row.clone())?;
    }
    let session = session.with_checked_role_inputs(
        [(role.into(), builder.finish()?)].into_iter().collect(),
        cancel,
    )?;
    let plan = session.relation_plan(&ResolvedTableReference {
        catalog: "roles".into(),
        schema: "inputs".into(),
        table: role.to_owned().into(),
    })?;
    Ok((session, plan))
}

/// Both directions use native null-safe set difference. Group counts preserve
/// multiplicity, so duplicate supplied facts cannot hide behind set equality.
pub(super) fn difference(
    left: LogicalPlan,
    right: LogicalPlan,
) -> datafusion::common::Result<LogicalPlan> {
    use datafusion::{
        functions_aggregate::expr_fn::count,
        logical_expr::{Expr, lit},
    };
    let counts = |plan: LogicalPlan| {
        let fields = plan
            .schema()
            .columns()
            .into_iter()
            .map(Expr::Column)
            .collect::<Vec<_>>();
        LogicalPlanBuilder::from(plan)
            .aggregate(fields, [count(lit(1_i64)).alias("multiplicity")])?
            .build()
    };
    let left = counts(left)?;
    let right = counts(right)?;
    LogicalPlanBuilder::from(LogicalPlanBuilder::except(
        left.clone(),
        right.clone(),
        false,
    )?)
    .union(LogicalPlanBuilder::except(right, left, false)?)?
    .limit(0, Some(1))?
    .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::{
        datasource::{MemTable, provider_as_source},
        execution::context::SessionContext,
        physical_plan::collect,
    };
    use std::sync::Arc;

    fn input(rows: &[deps::Row]) -> LogicalPlan {
        let mut builder = deps::Builder::new().unwrap();
        for row in rows {
            builder.push(row.clone()).unwrap();
        }
        let batch = builder.finish().unwrap().into_batch();
        let provider = Arc::new(MemTable::try_new(batch.schema(), vec![vec![batch]]).unwrap());
        LogicalPlanBuilder::scan("dependencies", provider_as_source(provider), None)
            .unwrap()
            .build()
            .unwrap()
    }
    #[tokio::test]
    async fn native_difference_checks_null_values_generation_and_multiplicity() {
        let expected = vec![
            fact(
                Kind::Operation,
                "native",
                "generation",
                None,
                Some(SemanticId::from_bytes([1; 16])),
            ),
            fact(Kind::Setting, "session", "optional", None, None),
        ];
        let state = SessionContext::new().state();
        let mut reordered = expected.clone();
        reordered.reverse();
        let mut null_changed = expected.clone();
        null_changed[1].evidence =
            Evidence::from_text(deps::RuntimeNativeDependenciesFieldEvidenceText {
                value: String::new(),
            });
        let mut generation_changed = expected.clone();
        generation_changed[0].evidence =
            Evidence::from_identity(deps::RuntimeNativeDependenciesFieldEvidenceIdentity {
                value: SemanticId::from_bytes([2; 16]),
            });
        let mut duplicate = expected.clone();
        duplicate.push(expected[0].clone());
        let missing = expected[..1].to_vec();
        for (actual, equal) in [
            (expected.clone(), true),
            (reordered, true),
            (null_changed, false),
            (generation_changed, false),
            (duplicate, false),
            (missing, false),
        ] {
            let logical = difference(input(&expected), input(&actual)).unwrap();
            let batches = collect(
                state.create_physical_plan(&logical).await.unwrap(),
                state.task_ctx(),
            )
            .await
            .unwrap();
            assert_eq!(batches.iter().all(|batch| batch.num_rows() == 0), equal);
        }
    }
}
