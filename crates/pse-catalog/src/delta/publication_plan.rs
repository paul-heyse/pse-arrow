// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native composition from relation writes to one coherent publication command.
use super::{
    contract::DeclaredCheck, layout::DurableLayout, publish::DeltaPublish, write::DeltaWrite,
};
use datafusion::{
    arrow::array::{Array, ListArray, StructArray},
    common::{DataFusionError, ResolvedTableReference, Result, ScalarValue},
    functions::core::expr_fn::named_struct,
    functions_aggregate::expr_fn::array_agg,
    logical_expr::{
        Aggregate, Expr, ExprFunctionExt, LogicalPlan, LogicalPlanBuilder, Projection, Union, col,
        lit,
    },
};
use deltalake::{DeltaTable, kernel::transaction::CommitProperties, protocol::SaveMode};
use pse_ids::SemanticId;
use pse_relations::generated::runtime::publications;
use pse_schema::Registry;
use std::sync::Arc;

#[cfg(test)]
mod tests;

/// A real relation input and its bound native Delta destination. No predecessor
/// object or historical member version is required to construct this command.
#[derive(Debug)]
pub struct MemberWrite {
    /// Exact catalog/schema/table name in the resulting publication.
    pub reference: ResolvedTableReference,
    /// Registered semantic relation declaration.
    pub relation_id: SemanticId,
    /// Actual destination and snapshot used for native Delta conflict detection.
    pub table: DeltaTable,
    /// Real declared Arrow input, planned and executed once as a native child.
    pub input: LogicalPlan,
}

/// A member selected by a new publication. Both routes are validated together
/// against the complete candidate vector before the control commit.
#[derive(Debug)]
pub enum Member {
    /// Execute a declared native write and select its actual committed version.
    Write(MemberWrite),
    /// Select an unchanged exact version and optional revision slice.
    Retained(publications::RuntimePublicationsFieldMembersItem),
}

/// Compose all member writes, their actual committed versions and the conditional
/// control commit into one native plan. The caller executes only this plan.
/// An error can leave unpublished member versions, never a partial publication.
/// `header.members` must be empty; writes and exact retained selectors supply the vector.
/// # Errors
/// Invalid declarations, duplicate names, empty inputs, incompatible fields or a
/// header that already contains members.
pub fn plan(
    location: url::Url,
    header: publications::Row,
    members: Vec<Member>,
    registry: Arc<Registry>,
) -> Result<LogicalPlan> {
    compose(location, header, members, registry, None)
}

pub(crate) fn plan_bound(
    location: url::Url,
    header: publications::Row,
    members: Vec<Member>,
    registry: Arc<Registry>,
    operation_id: SemanticId,
    dependencies: Vec<pse_relations::generated::runtime::native_dependencies::Row>,
) -> Result<LogicalPlan> {
    compose(
        location,
        header,
        members,
        registry,
        Some(&(operation_id, dependencies)),
    )
}

fn compose(
    location: url::Url,
    header: publications::Row,
    members: Vec<Member>,
    registry: Arc<Registry>,
    identity: Option<&(
        SemanticId,
        Vec<pse_relations::generated::runtime::native_dependencies::Row>,
    )>,
) -> Result<LogicalPlan> {
    if !header.members.is_empty() || members.is_empty() {
        return Err(invalid(
            "publication composition needs an empty member header and explicit members",
        ));
    }
    let mut names = std::collections::BTreeSet::new();
    let mut candidate = header.clone();
    let mut inputs = Vec::new();
    for member in members {
        let (outcome, descriptor) = match member {
            Member::Write(member) => write_member(member, &header, &registry, &location, identity)?,
            Member::Retained(descriptor) => {
                let version = LogicalPlanBuilder::empty(true)
                    .project([lit(descriptor.delta_version).alias("version")])?
                    .build()?;
                (version, descriptor)
            }
        };
        let reference = (
            descriptor.catalog_name.clone(),
            descriptor.schema_name.clone(),
            descriptor.table_name.clone(),
        );
        if !names.insert(reference) {
            return Err(invalid("duplicate publication member binding"));
        }
        candidate.members.push(descriptor.clone());
        inputs.push(Arc::new(describe(outcome, &header, descriptor)?));
    }
    super::admission::admit_profile(&candidate, &registry)?;
    let union = if inputs.len() == 1 {
        inputs
            .pop()
            .ok_or_else(|| invalid("no publication members"))?
    } else {
        Arc::new(LogicalPlan::Union(Union::try_new_with_loose_types(inputs)?))
    };
    // All expressions below name fields of our immediately preceding native
    // constructors. They require no SQL name resolution through the write DAG.
    let members = LogicalPlan::Aggregate(Aggregate::try_new(
        union,
        Vec::<Expr>::new(),
        vec![
            array_agg(col("member"))
                .order_by(vec![
                    col("catalog_name").sort(true, false),
                    col("schema_name").sort(true, false),
                    col("table_name").sort(true, false),
                ])
                .build()?
                .alias("members"),
        ],
    )?);
    let literal = control_batch(header)?;
    let layout = DurableLayout::new(literal.schema())?;
    let expressions = literal
        .schema()
        .fields()
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let (value, actual) = if field.name() == "members" {
                (
                    col("members"),
                    members.schema().field(0).data_type().clone(),
                )
            } else {
                (
                    lit(ScalarValue::try_from_array(literal.column(index), 0)?),
                    field.data_type().clone(),
                )
            };
            Ok(layout
                .storage_expression(index, value, actual)
                .alias(field.name()))
        })
        .collect::<Result<Vec<_>>>()?;
    let control = layout.decode(LogicalPlan::Projection(Projection::try_new(
        expressions,
        Arc::new(members),
    )?))?;
    DeltaPublish::plan(location, control, registry)
}
fn write_member(
    member: MemberWrite,
    header: &publications::Row,
    registry: &Registry,
    location: &url::Url,
    identity: Option<&(
        SemanticId,
        Vec<pse_relations::generated::runtime::native_dependencies::Row>,
    )>,
) -> Result<(
    LogicalPlan,
    publications::RuntimePublicationsFieldMembersItem,
)> {
    let spec = registry
        .relation_by_id(member.relation_id)
        .ok_or_else(|| invalid("unknown publication member contract"))?;
    let descriptor = publications::RuntimePublicationsFieldMembersItem {
        catalog_name: member.reference.catalog.to_string(),
        schema_name: member.reference.schema.to_string(),
        table_name: member.reference.table.to_string(),
        relation_id: spec.id,
        relation_version: i64::from(spec.key.version),
        contract_fingerprint: spec.fingerprint,
        table_uri: member.table.table_url().to_string(),
        delta_version: 0,
        selection: publications::RuntimePublicationsFieldMembersItemSelection::from_full(),
    };
    let mode = if member.table.version().is_none() {
        SaveMode::ErrorIfExists
    } else {
        SaveMode::Overwrite
    };
    let commit = CommitProperties::default().with_metadata(std::collections::HashMap::from([(
        "pse.attempt".to_owned(),
        serde_json::Value::String(header.attempt_id.to_string()),
    )]));
    let attempt = identity.map(
        |(operation_id, dependencies)| super::attempt::MemberAttempt {
            operation_id: *operation_id,
            publication_uri: location.clone(),
            workspace_id: header.workspace_id,
            publication_id: header.publication_id,
            parent_publication_id: header.parent_publication_id,
            attempt_id: header.attempt_id,
            member: descriptor.clone(),
            inputs: header.inputs.clone(),
            dependencies: dependencies.clone(),
            base_version: member.table.version(),
        },
    );
    let write = DeltaWrite::declared_attempt(
        member.table,
        member.input,
        mode,
        commit,
        DeclaredCheck::new(registry, spec.id)?,
        attempt,
    )?;
    Ok((write, descriptor))
}

fn describe(
    write: LogicalPlan,
    header: &publications::Row,
    descriptor: publications::RuntimePublicationsFieldMembersItem,
) -> Result<LogicalPlan> {
    let mut sample = header.clone();
    sample.members = vec![descriptor.clone()];
    let batch = control_batch(sample)?;
    let list = batch
        .column_by_name("members")
        .and_then(|c| c.as_any().downcast_ref::<ListArray>())
        .ok_or_else(|| invalid("generated members field is not a list"))?;
    let values = list
        .values()
        .as_any()
        .downcast_ref::<StructArray>()
        .ok_or_else(|| invalid("generated member is not a struct"))?;
    let mut arguments = vec![];
    for (index, field) in values.fields().iter().enumerate() {
        arguments.push(lit(field.name().to_owned()));
        arguments.push(if field.name() == "delta_version" {
            col("version")
        } else {
            lit(ScalarValue::try_from_array(values.column(index), 0)?)
        });
    }
    Ok(LogicalPlan::Projection(Projection::try_new(
        vec![
            named_struct(arguments).alias("member"),
            lit(descriptor.catalog_name).alias("catalog_name"),
            lit(descriptor.schema_name).alias("schema_name"),
            lit(descriptor.table_name).alias("table_name"),
        ],
        Arc::new(write),
    )?))
}
fn control_batch(row: publications::Row) -> Result<datafusion::arrow::array::RecordBatch> {
    let mut builder = publications::Builder::new().map_err(external)?;
    builder.push(row).map_err(external)?;
    Ok(builder.finish().map_err(external)?.into_batch())
}
fn external(error: pse_relations::RelationError) -> DataFusionError {
    DataFusionError::External(Box::new(error))
}
fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(reason.to_owned())
}
