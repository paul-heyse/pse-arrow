// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Reuse native P4/P5 producers for the actual selected method contexts.
use super::{execute_selected_program, invalid, predicates};
use crate::{
    CompilerError, InputBundle, PassContext,
    passes::{native_outputs::Sources, native_rows::workspace},
};
use datafusion::logical_expr::{LogicalPlanBuilder, col, lit};
use pse_catalog::session::SnapshotSession;
use pse_relations::{RecordBatch, columnar::FieldCheckedBatch};
use pse_rules::strata::{
    LocatedRuleInput, RuleInputLocation,
    completed::CompletedRelation,
    native_input::{NativeInput, NativeWitness},
};
use pse_schema::model::{PassSpec, RelationKey};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Debug)]
pub(crate) struct AugmentedInference {
    pub(crate) relations: BTreeMap<RelationKey, FieldCheckedBatch>,
    pub(crate) sources: Sources,
    pub(crate) derivations: Vec<RecordBatch>,
}

/// Overrides retain their actual native completions; the shared producers run once.
pub(crate) async fn evaluate_augmented(
    spec: &PassSpec,
    ctx: &PassContext<'_>,
    inputs: &InputBundle,
    overrides: &BTreeMap<RelationKey, Arc<NativeInput>>,
) -> Result<AugmentedInference, CompilerError> {
    inputs.validate(spec, ctx.registry)?;
    let base = ctx.session;
    let p4 = ctx
        .registry
        .pass("P4@1")
        .ok_or_else(|| invalid("P4 declaration absent"))?;
    let mut rows = inputs.checked_rows(ctx.registry)?;
    let mut sources = Sources::from_inputs(inputs, ctx.registry)?;
    let mut replacements = BTreeMap::new();
    for (key, input) in overrides {
        if !spec
            .outputs
            .iter()
            .any(|port| port.relation == key.qualified_name())
        {
            return Err(invalid(format!(
                "augmented result {key} is outside the actual pass output inventory"
            )));
        }
        rows.insert(*key, input.checked().clone());
        replacements.insert(*key, input.checked().clone());
        sources.replace_native(*key, Arc::clone(input))?;
    }
    let session = workspace(base, &replacements, ctx.cancel)?;
    let first =
        execute_selected_program(p4, spec, ctx, inputs, &session, &sources.locations()?).await?;
    let mut derivations = first.derivations;
    let mut complete = BTreeMap::new();
    for key in first.completed.keys() {
        let relation = first.completed.relation(key)?;
        rows.insert(key, relation.checked().clone());
        complete.insert(key, relation.checked().clone());
        sources.replace_location(key, RuleInputLocation::Completed(relation))?;
    }
    let session = workspace(&session, &complete, ctx.cancel)?;
    let predicate = predicates::evaluate_rows(ctx, spec, &rows, &sources, &session).await?;
    for (key, input) in predicate.relations {
        rows.insert(key, input.checked().clone());
        complete.insert(key, input.checked().clone());
        sources.replace_native(key, input)?;
    }
    derivations.extend(predicate.derivations);
    let session = workspace(&session, &complete, ctx.cancel)?;
    let second = Box::pin(crate::passes::p5::evaluate(
        spec, ctx, inputs, rows, sources, &session,
    ))
    .await?;
    let mut sources = second.sources;
    for (key, batch) in second.relations {
        match complete.entry(key) {
            std::collections::btree_map::Entry::Occupied(mut entry) => {
                // Both are actual completed rule families. Project original evidence
                // fields unchanged, while retaining each branch's exact source rows.
                let right = match &sources
                    .get(&key)
                    .ok_or_else(|| invalid("shared evidence source absent"))?
                    .1
                    .location
                {
                    RuleInputLocation::Completed(input) => Arc::clone(input),
                    _ => {
                        return Err(invalid(
                            "shared rule evidence lacks an actual completed producer",
                        ));
                    }
                };
                let merged = merge_evidence(
                    key,
                    [first.completed.relation(key)?, right],
                    spec,
                    &session,
                    ctx,
                )
                .await?;
                entry.insert(merged.checked().clone());
                derivations.push(merged.derivations().clone().into_batch());
                sources.replace_native(key, merged)?;
            }
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(batch);
            }
        }
    }
    derivations.extend(second.derivations);
    Ok(AugmentedInference {
        relations: complete,
        sources,
        derivations,
    })
}

async fn merge_evidence(
    key: RelationKey,
    inputs: [Arc<CompletedRelation>; 2],
    pass: &PassSpec,
    session: &SnapshotSession,
    ctx: &PassContext<'_>,
) -> Result<Arc<NativeInput>, CompilerError> {
    if !matches!(
        key.qualified_name().as_str(),
        "inferred.rule_outcomes" | "provenance.rule_support_edges"
    ) {
        return Err(invalid("P4 and P5 unexpectedly share a domain output"));
    }
    let target = ctx
        .registry
        .relation(&key.qualified_name())
        .ok_or_else(|| invalid("shared evidence contract absent"))?;
    let roles = inputs
        .iter()
        .enumerate()
        .map(|(index, input)| {
            (
                format!("shared-rule-evidence:{index}"),
                input.checked().clone(),
            )
        })
        .collect();
    let owner = session.with_checked_role_inputs(roles, ctx.cancel)?;
    let mut branches = Vec::new();
    let mut witnesses = Vec::new();
    for (index, input) in inputs.into_iter().enumerate() {
        let role = format!("shared-rule-evidence:{index}");
        let mut columns = target
            .columns
            .iter()
            .map(|field| col(field.name()))
            .collect::<Vec<_>>();
        columns.push(lit(index as u64).alias("__evidence_branch"));
        branches.push(
            LogicalPlanBuilder::from(owner.scan_role(&role)?)
                .project(columns)
                .and_then(LogicalPlanBuilder::build)
                .map_err(crate::passes::native_construction::error)?,
        );
        witnesses.push(NativeWitness {
            port: role,
            input: LocatedRuleInput {
                relation: key,
                location: RuleInputLocation::Completed(input),
            },
            key_columns: target
                .primary_key
                .iter()
                .map(|name| (*name).to_owned())
                .collect(),
            when: Some(col("__evidence_branch").eq(lit(index as u64))),
        });
    }
    Ok(NativeInput::build(
        crate::passes::native_construction::union(branches)?,
        key,
        pass.id,
        target
            .columns
            .iter()
            .map(|column| (column.name().to_owned(), column.name().to_owned()))
            .collect(),
        witnesses,
        &owner,
        ctx.cancel,
    )
    .await?)
}
