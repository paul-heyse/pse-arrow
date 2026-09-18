// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact authored key fields locate the normalized declaration and its guard source.
use super::{
    BTreeSet, CompilerError, Inventory, LogicalPlanBuilder, ScalarValue, SemanticId, Support, col,
    error, invalid, n, scalar, unique,
};
use datafusion::arrow::array::{Array, FixedSizeBinaryArray};
use pse_catalog::session::output::checked_literal;
use pse_ids::CancellationToken;

type Guard = (Option<(SemanticId, i64)>, Support);
#[expect(
    clippy::too_many_lines,
    reason = "lookup keeps the native relation inputs and dependency ordered assembly visible in one place"
)]
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
    let token = checked_literal(
        inventory.registry,
        &pse_schema::model::FieldContract::row_key(),
        ScalarValue::FixedSizeBinary(32, Some(source.source_key.as_bytes().to_vec())),
    )
    .map_err(error)?;
    // Normalization preserves the actual declaration key. Compare it in its
    // authored scope; the selected normalized row remains the support owner.
    let plan = LogicalPlanBuilder::from(inventory.scan_key(relation.key)?)
        .filter(
            scalar::key(
                authored.id,
                authored
                    .primary_key
                    .iter()
                    .map(|name| (*name, col(*name)))
                    .collect(),
            )
            .eq(token),
        )
        .map_err(error)?;
    let guard_key = scalar::key(
        pse_relations::generated::authored::template_guards::RELATION_ID,
        vec![("guard_id", col("guard_id"))],
    );
    let plan = plan
        .project([
            col("guard_id"),
            guard_key.alias("guard_key"),
            scalar::key(
                relation.id,
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
        let guard_keys = batch
            .column(1)
            .as_any()
            .downcast_ref::<FixedSizeBinaryArray>()
            .ok_or_else(|| invalid("guard declaration key storage differs"))?;
        let keys = batch
            .column(2)
            .as_any()
            .downcast_ref::<FixedSizeBinaryArray>()
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
            found.push((
                guard,
                crate::passes::native_rows::key_value(guard_keys.value(row))?,
                crate::passes::native_rows::key_value(keys.value(row))?,
            ));
        }
    }
    let [(guard, guard_key, key)] = found.as_slice() else {
        return Err(invalid("source key has no unique guarded declaration"));
    };
    let mut support = BTreeSet::from([(relation.key, *key)]);
    let Some(_) = guard else {
        return Ok((None, support));
    };
    let guard_source = unique(
        &inventory.sources,
        |row| {
            row.source_relation_id
                == pse_relations::generated::authored::template_guards::RELATION_ID
                && row.field_path.rsplit('/').next() == Some("predicate")
                && row.source_key == *guard_key
        },
        "declaration guard source",
    )?;
    support.insert(inventory.origin(guard_source)?);
    Ok((
        Some((guard_source.row.source_id, guard_source.row.root_id)),
        support,
    ))
}
