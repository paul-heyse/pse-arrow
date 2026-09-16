// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Occurrence/declaration joins prepare the only name-resolution inventory consumed
//! by lexical interpretation. Candidate multiplicity survives the native joins.
use super::{ParsedExpression, PathMeaning};
use crate::{
    AuthoringError,
    document::{Batches, DocumentBundle, load::contract},
};
use datafusion::{
    common::{Column, ScalarValue, metadata::FieldMetadata},
    logical_expr::{Expr, JoinType, LogicalPlan, LogicalPlanBuilder, col, lit},
};
use pse_catalog::session::SnapshotSession;
use pse_ids::{CancellationToken, SemanticId};
use pse_relations::{
    columnar::FieldCheckedBatch,
    generated::{authored, enums, extension_values, normalized},
};
use pse_schema::{
    Registry,
    model::{ExpressionOwnerKind, FieldContract},
};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default)]
pub(super) struct Lookup {
    pub(super) completed: crate::change_set::plans::Completions,
    pub(super) source_keys: BTreeMap<(SemanticId, String), String>,
    next: BTreeMap<(SemanticId, String, u32), u32>,
    pub(super) local: BTreeMap<(SemanticId, String), Vec<PathMeaning>>,
    pub(super) global: BTreeMap<String, Vec<SemanticId>>,
    pub(super) units: BTreeMap<String, Vec<SemanticId>>,
    pub(super) occurrences: Vec<FieldCheckedBatch>,
    pub(super) resolved: Vec<FieldCheckedBatch>,
}
fn engine(error: datafusion::common::DataFusionError) -> AuthoringError {
    crate::change_set::plans::engine(error)
}
fn qualified(alias: &str, name: &str) -> Expr {
    Expr::Column(Column::new(Some(alias), name))
}
fn typed(
    value: ScalarValue,
    kind: FieldContract,
    registry: &Registry,
) -> Result<Expr, AuthoringError> {
    let field = pse_schema::arrow::field_for(
        registry,
        &FieldContract::payload("literal", kind, "Declared native literal."),
    )
    .map_err(pse_relations::RelationError::from)?;
    Ok(Expr::Literal(value, Some(FieldMetadata::from(&field))))
}
fn kind(value: &str, registry: &Registry) -> Result<Expr, AuthoringError> {
    let column = FieldContract::payload(
        "kind",
        FieldContract::enumeration("SourceBindingKind"),
        "Binding kind.",
    );
    pse_catalog::session::output::checked_literal(
        registry,
        &column,
        ScalarValue::Utf8(Some(value.to_owned())),
    )
    .map_err(engine)
}

pub(super) async fn prepare(
    bundles: &[DocumentBundle],
    inputs: &Batches,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<Lookup, AuthoringError> {
    let OccurrenceInputs {
        roles,
        branches,
        batches: occurrences,
    } = OccurrenceInputs::load(bundles, inputs, session, cancel)?;
    if branches.is_empty() {
        return Ok(Lookup {
            occurrences,
            ..Lookup::default()
        });
    }
    let bound = session.with_checked_role_inputs(roles, cancel)?;
    let mut requests = None;
    for (occurrence_role, source_role, spec) in branches {
        let key = pse_catalog::session::scalar::key(
            spec.primary_key
                .iter()
                .map(|name| (*name, col(*name)))
                .collect(),
        );
        let source = LogicalPlanBuilder::from(bound.scan_role(&source_role)?)
            .project(vec![key.alias("source_key")])
            .and_then(LogicalPlanBuilder::build)
            .map_err(engine)?;
        let plan = LogicalPlanBuilder::from(bound.scan_role(&occurrence_role)?)
            .cross_join(source)
            .and_then(LogicalPlanBuilder::build)
            .map_err(engine)?;
        requests = Some(match requests {
            None => plan,
            Some(previous) => LogicalPlanBuilder::from(previous)
                .union(plan)
                .and_then(LogicalPlanBuilder::build)
                .map_err(engine)?,
        });
    }
    let requests = requests.ok_or_else(|| contract(None, "occurrence plan absent"))?;
    let mut result = Lookup {
        occurrences,
        ..Lookup::default()
    };
    bind_source_keys(&bound, requests.clone(), &mut result, cancel).await?;
    let declarations = declarations(&bound, inputs)?;
    let request = LogicalPlanBuilder::from(requests.clone())
        .alias("occurrence")
        .map_err(engine)?;
    let declarations = LogicalPlanBuilder::from(declarations)
        .alias("declaration")
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    let local = request
        .join_on(
            declarations,
            JoinType::Inner,
            [qualified("occurrence", "lookup_name").eq(qualified("declaration", "name"))],
        )
        .and_then(|plan| plan.project(output_columns()))
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    consume(&bound, local, &mut result, cancel).await?;
    bind_globals(inputs, &bound, &requests, &mut result, cancel).await?;
    Ok(result)
}
fn occurrence_columns() -> Vec<Expr> {
    ["document_id", "field_path", "ordinal", "source_key"]
        .into_iter()
        .map(|name| qualified("occurrence", name).alias(name))
        .collect()
}
fn output_columns() -> Vec<Expr> {
    let mut result = occurrence_columns();
    result.extend(
        ["kind", "owner_template_id", "semantic_id", "name"]
            .into_iter()
            .map(|name| qualified("declaration", name).alias(name)),
    );
    result
}
fn declarations(
    session: &SnapshotSession,
    inputs: &Batches,
) -> Result<LogicalPlan, AuthoringError> {
    let mut result = None;
    for (relation, kind_name, identity) in [
        (
            authored::template_symbols::RELATION_ID,
            "symbol",
            Some("symbol_decl_id"),
        ),
        (
            authored::template_equations::RELATION_ID,
            "equation",
            Some("equation_decl_id"),
        ),
        (authored::template_ports::RELATION_ID, "port", None),
        (authored::template_domains::RELATION_ID, "domain", None),
        (authored::template_params::RELATION_ID, "parameter", None),
        (authored::template_features::RELATION_ID, "feature", None),
    ] {
        if !inputs.contains_key(&relation) {
            continue;
        }
        let identity = identity.map_or_else(
            || {
                typed(
                    ScalarValue::FixedSizeBinary(16, None),
                    FieldContract::id(),
                    session.registry(),
                )
            },
            |name| Ok(col(name)),
        )?;
        let plan = LogicalPlanBuilder::from(session.scan_role(&format!("binding_{relation}"))?)
            .project(vec![
                kind(kind_name, session.registry())?.alias("kind"),
                col("template_id").alias("owner_template_id"),
                identity.alias("semantic_id"),
                col("name"),
            ])
            .and_then(LogicalPlanBuilder::build)
            .map_err(engine)?;
        result = Some(match result {
            None => plan,
            Some(previous) => LogicalPlanBuilder::from(previous)
                .union(plan)
                .and_then(LogicalPlanBuilder::build)
                .map_err(engine)?,
        });
    }
    result.ok_or_else(|| contract(None, "no declaration inventory bound"))
}
async fn consume(
    session: &SnapshotSession,
    plan: LogicalPlan,
    output: &mut Lookup,
    cancel: &CancellationToken,
) -> Result<(), AuthoringError> {
    use datafusion::arrow::array::{Array, FixedSizeBinaryArray, StringArray, UInt32Array};
    let mut owners = BTreeMap::new();
    let mut builder =
        normalized::resolved_source_occurrences::Builder::with_registry(session.registry(), 0)?;
    for batch in
        crate::change_set::plans::execute_recorded(session, plan, cancel, &mut output.completed)
            .await?
    {
        let strings = ["field_path", "kind", "name", "source_key"]
            .into_iter()
            .map(|name| {
                let array = batch
                    .column_by_name(name)
                    .ok_or_else(|| contract(None, "binding text absent"))?;
                let array = datafusion::arrow::compute::cast(
                    array,
                    &datafusion::arrow::datatypes::DataType::Utf8,
                )
                .map_err(|error| engine(error.into()))?;
                Ok((name, array))
            })
            .collect::<Result<BTreeMap<_, _>, AuthoringError>>()?;
        for row in 0..batch.num_rows() {
            let id = |name: &str| -> Result<Option<SemanticId>, AuthoringError> {
                let array = batch
                    .column_by_name(name)
                    .and_then(|array| array.as_any().downcast_ref::<FixedSizeBinaryArray>())
                    .ok_or_else(|| contract(None, "binding identity column missing"))?;
                if array.is_null(row) {
                    return Ok(None);
                }
                let bytes: [u8; 16] = array
                    .value(row)
                    .try_into()
                    .map_err(|_| contract(None, "binding identity width"))?;
                Ok(Some(SemanticId::from_bytes(bytes)))
            };
            let text = |name: &str| -> Result<String, AuthoringError> {
                let array = strings
                    .get(name)
                    .ok_or_else(|| contract(None, "binding text absent"))?;
                Ok(array
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .ok_or_else(|| contract(None, "binding text storage"))?
                    .value(row)
                    .to_owned())
            };
            let document_id =
                id("document_id")?.ok_or_else(|| contract(None, "occurrence document absent"))?;
            let field_path = text("field_path")?;
            let ordinal = batch
                .column_by_name("ordinal")
                .and_then(|array| array.as_any().downcast_ref::<UInt32Array>())
                .ok_or_else(|| contract(None, "occurrence ordinal absent"))?
                .value(row);
            let kind_name = text("kind")?;
            let name = text("name")?;
            let template = id("owner_template_id")?;
            let semantic = id("semantic_id")?;
            let occurrence = (document_id, field_path.clone(), ordinal);
            let key = (template, name.clone(), kind_name.clone());
            let first = owners.entry(key).or_insert_with(|| occurrence.clone());
            if *first == occurrence {
                remember_meaning(output, &kind_name, &name, template, semantic)?;
            }
            let match_ordinal = output.next.entry(occurrence).or_default();
            builder.push(normalized::resolved_source_occurrences::Row {
                document_id,
                field_path,
                ordinal,
                match_ordinal: *match_ordinal,
                source_key: text("source_key")?,
                kind: kind_name.parse()?,
                owner_template_id: template,
                semantic_id: semantic,
                name,
            })?;
            *match_ordinal = match_ordinal
                .checked_add(1)
                .ok_or_else(|| contract(None, "binding multiplicity overflow"))?;
        }
    }
    output
        .resolved
        .push(builder.finish()?.retained(session.reserver(), cancel)?);
    Ok(())
}

fn remember_meaning(
    output: &mut Lookup,
    kind_name: &str,
    name: &str,
    template: Option<SemanticId>,
    semantic: Option<SemanticId>,
) -> Result<(), AuthoringError> {
    if let Some(template_id) = template {
        let meaning = match kind_name {
            "symbol" => PathMeaning::Symbol {
                template_id,
                symbol_id: semantic.ok_or_else(|| contract(None, "symbol identity absent"))?,
            },
            "equation" => PathMeaning::Equation {
                template_id,
                equation_id: semantic.ok_or_else(|| contract(None, "equation identity absent"))?,
            },
            "port" => PathMeaning::Port {
                template_id,
                name: name.to_owned(),
            },
            "domain" => PathMeaning::Domain {
                template_id,
                name: name.to_owned(),
            },
            "parameter" => PathMeaning::Parameter {
                template_id,
                name: name.to_owned(),
            },
            "feature" => PathMeaning::Feature {
                template_id,
                name: name.to_owned(),
            },
            _ => return Err(contract(None, "unexpected template binding kind")),
        };
        output
            .local
            .entry((template_id, name.to_owned()))
            .or_default()
            .push(meaning);
    } else {
        let identity = semantic.ok_or_else(|| contract(None, "global binding identity absent"))?;
        if kind_name == "entity" {
            output
                .global
                .entry(name.to_owned())
                .or_default()
                .push(identity);
        } else {
            output
                .units
                .entry(name.to_owned())
                .or_default()
                .push(identity);
        }
    }
    Ok(())
}

struct OccurrenceInputs<'a> {
    roles: BTreeMap<String, FieldCheckedBatch>,
    branches: Vec<(String, String, &'a pse_schema::model::RelationSpec)>,
    batches: Vec<FieldCheckedBatch>,
}
impl<'a> OccurrenceInputs<'a> {
    fn load(
        bundles: &[DocumentBundle],
        inputs: &Batches,
        session: &'a SnapshotSession,
        cancel: &CancellationToken,
    ) -> Result<Self, AuthoringError> {
        let registry = session.registry();
        let mut roles = inputs
            .iter()
            .map(|(id, batch)| (format!("binding_{id}"), batch.clone()))
            .collect::<BTreeMap<_, _>>();
        let mut branches = Vec::new();
        let mut occurrences = Vec::new();
        for bundle in bundles {
            for document in &bundle.documents {
                for section in &document.declaration.sections {
                    let Some(owner_column) = section.expression_owner_column else {
                        continue;
                    };
                    let spec = registry
                        .relation(section.relation)
                        .ok_or_else(|| contract(None, "expression relation absent"))?;
                    let Some(source) = document.batches.get(&spec.id) else {
                        continue;
                    };
                    let Some(super::super::value::Value::List(values)) =
                        document.value.get(section.key)
                    else {
                        continue;
                    };
                    for (row_ordinal, value) in values.iter().enumerate() {
                        let owner = value
                            .value
                            .get(owner_column)
                            .and_then(super::super::value::Value::text)
                            .ok_or_else(|| contract(None, "expression owner absent"))?;
                        let owner_id =
                            crate::ids::parse_id(owner, crate::SourceSpan::head(document.id))?;
                        let batch = occurrence_rows(
                            document,
                            section,
                            row_ordinal,
                            owner_id,
                            session,
                            cancel,
                        )?;
                        if batch.batch().num_rows() == 0 {
                            continue;
                        }
                        let number = occurrences.len();
                        let occurrence_role = format!("occurrences_{number}");
                        let source_role = format!("occurrence_source_{number}");
                        roles.insert(occurrence_role.clone(), batch.clone());
                        roles.insert(source_role.clone(), source.slice(row_ordinal, 1)?);
                        branches.push((occurrence_role, source_role, spec));
                        occurrences.push(batch);
                    }
                }
            }
        }
        Ok(Self {
            roles,
            branches,
            batches: occurrences,
        })
    }
}

fn occurrence_rows(
    document: &crate::document::Document,
    section: &pse_schema::model::DocumentSection,
    row_ordinal: usize,
    owner_id: SemanticId,
    session: &SnapshotSession,
    cancel: &CancellationToken,
) -> Result<FieldCheckedBatch, AuthoringError> {
    let registry = session.registry();
    let spec = registry
        .relation(section.relation)
        .ok_or_else(|| contract(None, "expression relation absent"))?;
    let prefix = format!("/{}/{row_ordinal}/", section.key);
    let mut builder = normalized::source_occurrences::Builder::with_registry(registry, 0)?;
    for (field_path, expression) in document
        .expressions
        .iter()
        .filter(|(path, _)| path.starts_with(&prefix))
    {
        let paths = match expression {
            ParsedExpression::Expr(value) => value.paths(),
            ParsedExpression::Predicate(value) => value.paths(),
            ParsedExpression::Equation(value) => value.paths(),
        };
        let span = document
            .spans
            .span(field_path)
            .ok_or_else(|| contract(None, "source field span absent"))?;
        let root = normalized::source_occurrences::Row {
            document_id: document.id,
            field_path: field_path.clone(),
            ordinal: 0,
            source_relation_id: spec.id,
            source_row_ordinal: u64::try_from(row_ordinal)
                .map_err(|_| contract(None, "source ordinal overflow"))?,
            owner_kind: match section
                .expression_owner_kind
                .ok_or_else(|| contract(None, "source owner kind absent"))?
            {
                ExpressionOwnerKind::Template => enums::ExpressionOwnerKind::Template,
                ExpressionOwnerKind::Instance => enums::ExpressionOwnerKind::Instance,
            },
            owner_id,
            lookup_name: None,
            source_span: extension_values::SourceSpan {
                document_id: span.document_id,
                start: i64::from(span.start),
                end: i64::from(span.end),
            },
        };
        builder.push(root.clone())?;
        let mut ordinal = 1_u32;
        for path in paths {
            let names = path
                .segments
                .iter()
                .map(|segment| segment.name.as_str())
                .collect::<Vec<_>>();
            for position in 0..names.len() {
                let mut lookups = vec![names[position].to_owned()];
                if position != 0 {
                    lookups.push(names[..=position].join("."));
                }
                for lookup_name in lookups {
                    builder.push(normalized::source_occurrences::Row {
                        ordinal,
                        lookup_name: Some(lookup_name),
                        ..root.clone()
                    })?;
                    ordinal = ordinal
                        .checked_add(1)
                        .ok_or_else(|| contract(None, "occurrence ordinal overflow"))?;
                }
            }
        }
    }
    let batch = builder.finish()?.retained(session.reserver(), cancel)?;
    Ok(batch)
}

async fn bind_source_keys(
    bound: &SnapshotSession,
    requests: LogicalPlan,
    result: &mut Lookup,
    cancel: &CancellationToken,
) -> Result<(), AuthoringError> {
    let keys = LogicalPlanBuilder::from(requests)
        .filter(col("lookup_name").is_null())
        .and_then(|plan| {
            plan.project(vec![
                col("document_id"),
                col("field_path"),
                col("source_key"),
            ])
        })
        .and_then(LogicalPlanBuilder::build)
        .map_err(engine)?;
    for batch in
        crate::change_set::plans::execute_recorded(bound, keys, cancel, &mut result.completed)
            .await?
    {
        use datafusion::arrow::array::{FixedSizeBinaryArray, StringArray};
        let ids = batch
            .column(0)
            .as_any()
            .downcast_ref::<FixedSizeBinaryArray>()
            .ok_or_else(|| contract(None, "source document storage"))?;
        let paths = batch
            .column(1)
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| contract(None, "source path storage"))?;
        let keys = batch
            .column(2)
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| contract(None, "source key storage"))?;
        for row in 0..batch.num_rows() {
            let id = SemanticId::from_bytes(
                ids.value(row)
                    .try_into()
                    .map_err(|_| contract(None, "source identity width"))?,
            );
            if result
                .source_keys
                .insert(
                    (id, paths.value(row).to_owned()),
                    keys.value(row).to_owned(),
                )
                .is_some()
            {
                return Err(contract(None, "source field has multiple generated rows"));
            }
        }
    }
    Ok(())
}

async fn bind_globals(
    inputs: &Batches,
    bound: &SnapshotSession,
    requests: &LogicalPlan,
    result: &mut Lookup,
    cancel: &CancellationToken,
) -> Result<(), AuthoringError> {
    let registry = bound.registry();
    for (relation, key, name, kind_name) in [
        (
            authored::entities::RELATION_ID,
            "entity_id",
            "qualified_name",
            "entity",
        ),
        (
            pse_relations::generated::reference::units::RELATION_ID,
            "unit_id",
            "symbol",
            "unit",
        ),
    ] {
        let Some(_) = inputs.get(&relation) else {
            continue;
        };
        let target = LogicalPlanBuilder::from(bound.scan_role(&format!("binding_{relation}"))?)
            .alias("declaration")
            .and_then(LogicalPlanBuilder::build)
            .map_err(engine)?;
        let name_match = if kind_name == "entity" {
            qualified("declaration", name)
                .eq(qualified("occurrence", "lookup_name"))
                .or(datafusion::functions::string::expr_fn::ends_with(
                    qualified("declaration", name),
                    datafusion::functions::string::expr_fn::concat(vec![
                        lit("."),
                        qualified("occurrence", "lookup_name"),
                    ]),
                ))
        } else {
            qualified("declaration", name).eq(qualified("occurrence", "lookup_name"))
        };
        let mut projection = occurrence_columns();
        projection.extend([
            kind(kind_name, registry)?.alias("kind"),
            typed(
                ScalarValue::FixedSizeBinary(16, None),
                FieldContract::id(),
                registry,
            )?
            .alias("owner_template_id"),
            qualified("declaration", key).alias("semantic_id"),
            qualified("occurrence", "lookup_name").alias("name"),
        ]);
        let plan = LogicalPlanBuilder::from(requests.clone())
            .alias("occurrence")
            .and_then(|plan| plan.join_on(target, JoinType::Inner, [name_match]))
            .and_then(|plan| plan.project(projection))
            .and_then(LogicalPlanBuilder::build)
            .map_err(engine)?;
        consume(bound, plan, result, cancel).await?;
    }
    Ok(())
}
