// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Primitive normalization is an actual projection over exact immutable sources.
use super::invalid;
use crate::{
    CompilerError,
    passes::{
        native_outputs::Sources,
        native_rows::{column, engine, scan},
    },
};
use datafusion::logical_expr::LogicalPlanBuilder;
use pse_catalog::session::SnapshotSession;
use pse_ids::CancellationToken;
use pse_rules::strata::native_input::{NativeInput, NativeWitness};
use pse_schema::model::{AlgorithmSpec, Namespace, RelationKey};
use std::{collections::BTreeMap, sync::Arc};

pub(super) async fn emit(
    sources: &Sources,
    pass: &AlgorithmSpec,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<BTreeMap<RelationKey, Arc<NativeInput>>, CompilerError> {
    let registry = session.registry();
    let mut output = BTreeMap::new();
    for source in registry
        .relations()
        .iter()
        .filter(|spec| spec.key.namespace == Namespace::Authored)
    {
        let Some(target) = registry.relation(&format!("normalized.{}", source.key.name)) else {
            continue;
        };
        if !pass
            .outputs
            .iter()
            .any(|port| port.relation == target.key.qualified_name())
        {
            continue;
        }
        let (port, input) = sources
            .get(&source.key)
            .ok_or_else(|| invalid("P3 primitive source is not bound"))?;
        let mut fields = source
            .columns
            .iter()
            .map(|field| column("source", field.name()).alias(field.name()))
            .collect::<Vec<_>>();
        let key_columns = source
            .primary_key
            .iter()
            .map(|name| {
                let output = format!("source_key_{name}");
                fields.push(column("source", name).alias(&output));
                output
            })
            .collect();
        let plan = LogicalPlanBuilder::from(scan(session, source, "source")?)
            .project(fields)
            .and_then(LogicalPlanBuilder::build)
            .map_err(engine)?;
        let columns = target
            .columns
            .iter()
            .map(|field| (field.name().to_owned(), field.name().to_owned()))
            .collect();
        let native = NativeInput::build(
            plan,
            target.key,
            pass.id,
            columns,
            vec![NativeWitness {
                port: port.clone(),
                input: input.clone(),
                key_columns: Some(key_columns),
                when: None,
            }],
            session,
            cancel,
        )
        .await?;
        output.insert(target.key, native);
    }
    Ok(output)
}
