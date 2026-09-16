// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact authored key fields locate the normalized declaration and its guard source.
use super::{
    BTreeSet, CompilerError, Inventory, LogicalPlanBuilder, ScalarValue, SemanticId, Support, col,
    error, invalid, n, scalar, unique,
};
use datafusion::arrow::array::{Array, FixedSizeBinaryArray, StringArray};
use pse_catalog::session::output::checked_literal;
use pse_ids::CancellationToken;

type Guard = (Option<(SemanticId, u64)>, Support);
pub(super) async fn lookup(
    inventory: &mut Inventory<'_>,
    source: Option<&n::expression_sources::Row>,
    cancel: &CancellationToken,
) -> Result<Guard, CompilerError> {
    let Some(source) = source else {
        return Ok((None, Support::new()));
    };
    let authored = inventory
        .registry
        .relations()
        .iter()
        .find(|relation| relation.id == source.source_relation_id)
        .ok_or_else(|| invalid("read authored source relation absent"))?;
    if authored.column("guard_id").is_none() {
        return Ok((None, Support::new()));
    }
    let name = format!("normalized.{}", authored.key.name);
    let relation = inventory
        .registry
        .relation(&name)
        .ok_or_else(|| invalid("guarded normalized source absent"))?;
    if source.source_key.len() != relation.primary_key.len() {
        return Err(invalid("guarded source does not carry its complete key"));
    }
    let mut plan = LogicalPlanBuilder::from(inventory.session.scan_role(&name)?);
    for key in &relation.primary_key {
        let parts = source
            .source_key
            .iter()
            .filter(|part| part.column_name == *key)
            .collect::<Vec<_>>();
        let [part] = parts.as_slice() else {
            return Err(invalid("guarded source key is absent or repeated"));
        };
        let field = relation
            .column(key)
            .ok_or_else(|| invalid("guarded source key column absent"))?;
        let mut values = Vec::new();
        if let Some(value) = part.semantic_id {
            values.push(ScalarValue::FixedSizeBinary(
                16,
                Some(value.as_bytes().to_vec()),
            ));
        }
        if let Some(value) = part.content_hash {
            values.push(ScalarValue::FixedSizeBinary(
                32,
                Some(value.as_bytes().to_vec()),
            ));
        }
        if let Some(value) = &part.text {
            values.push(ScalarValue::Utf8(Some(value.clone())));
        }
        if let Some(value) = part.signed_integer {
            values.push(ScalarValue::Int64(Some(value)));
        }
        if let Some(value) = part.unsigned_integer {
            values.push(ScalarValue::UInt64(Some(value)));
        }
        if let Some(value) = part.boolean {
            values.push(ScalarValue::Boolean(Some(value)));
        }
        let value = if let Some(index) = &part.index_tuple {
            if !values.is_empty() {
                return Err(invalid("source key has multiple typed values"));
            }
            scalar::id_list(
                index
                    .iter()
                    .map(|id| {
                        checked_literal(
                            inventory.registry,
                            &pse_schema::model::FieldContract::payload(
                                "id",
                                pse_schema::model::FieldContract::id(),
                                "Actual source key part.",
                            ),
                            ScalarValue::FixedSizeBinary(16, Some(id.as_bytes().to_vec())),
                        )
                        .map_err(error)
                    })
                    .collect::<Result<_, _>>()?,
            )
        } else {
            let [value] = values.as_slice() else {
                return Err(invalid("source key must have exactly one typed value"));
            };
            let declared = pse_schema::arrow::field_for(inventory.registry, field)
                .map_err(|error| invalid(error.to_string()))?;
            let value = value.cast_to(declared.data_type()).map_err(error)?;
            checked_literal(inventory.registry, field, value).map_err(error)?
        };
        plan = plan.filter(col(*key).eq(value)).map_err(error)?;
    }
    let plan = plan
        .project([
            col("guard_id"),
            scalar::key(
                relation
                    .primary_key
                    .iter()
                    .map(|name| (*name, col(*name)))
                    .collect(),
            )
            .alias("source_key"),
        ])
        .and_then(LogicalPlanBuilder::build)
        .map_err(error)?;
    let complete = inventory
        .arguments
        .execute(plan, &inventory.session, cancel)
        .await?;
    let mut found = Vec::new();
    for batch in complete.batches() {
        let guards = batch
            .column(0)
            .as_any()
            .downcast_ref::<FixedSizeBinaryArray>()
            .ok_or_else(|| invalid("guard identity storage differs"))?;
        let keys = batch
            .column(1)
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| invalid("guard source key storage differs"))?;
        for row in 0..batch.num_rows() {
            if keys.is_null(row) {
                return Err(invalid("guard source key is null"));
            }
            let guard = if guards.is_null(row) {
                None
            } else {
                Some(
                    SemanticId::try_from_slice(guards.value(row))
                        .map_err(|_| invalid("guard identity width differs"))?,
                )
            };
            found.push((guard, keys.value(row).to_owned()));
        }
    }
    let [(guard, key)] = found.as_slice() else {
        return Err(invalid("source key has no unique guarded declaration"));
    };
    let mut support = BTreeSet::from([(relation.key, key.clone())]);
    let Some(guard) = guard else {
        return Ok((None, support));
    };
    let guard_source = unique(
        &inventory.sources,
        |row| {
            row.source_relation_id
                == pse_relations::generated::authored::template_guards::RELATION_ID
                && row.field_path.rsplit('/').next() == Some("predicate")
                && row
                    .source_key
                    .iter()
                    .any(|part| part.column_name == "guard_id" && part.semantic_id == Some(*guard))
        },
        "declaration guard source",
    )?;
    support.insert(inventory.origin(guard_source)?);
    Ok((
        Some((guard_source.row.source_id, guard_source.row.root_id)),
        support,
    ))
}
