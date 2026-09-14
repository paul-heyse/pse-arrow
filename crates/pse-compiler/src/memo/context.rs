// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete durable dependencies, compared as values after an indexed lookup.
mod extent;

use crate::{CompilerError, PassContext, passes::dag::invalid};
use pse_catalog::store::stage_context::{StageContext, StagePolicy, StageSource, heap_extent};
use pse_ids::Reservation;
use pse_schema::model::Cell;
use std::collections::BTreeMap;

/// Keep the complete descriptor's construction and retained heap within a bounded claim.
pub(crate) struct OwnedContext {
    pub(crate) value: StageContext,
    _reservation: Box<dyn Reservation>,
}
pub(crate) fn capture(
    ctx: &PassContext<'_>,
    executes_plans: bool,
) -> Result<OwnedContext, CompilerError> {
    let mut reservation = ctx.reserver.open("compiler:durable-stage-context");
    reservation
        .try_grow(extent::forecast(ctx, executes_plans)?)
        .map_err(|error| CompilerError::Catalog(error.into()))?;
    ctx.cancel.checkpoint()?;
    let declarations = ctx
        .registry
        .schema_rows_ref()
        .iter()
        .map(|(key, rows)| (key.to_string(), encode_rows(rows)))
        .collect();
    let mut sources = ctx
        .documents
        .bundles()
        .iter()
        .flat_map(|bundle| {
            bundle.documents.iter().map(|document| StageSource {
                package_id: bundle.package.package_id,
                document_id: document.id,
                path: document.path.clone(),
                text: document.text.clone(),
            })
        })
        .collect::<Vec<_>>();
    sources.sort_by(|a, b| {
        (a.package_id, a.document_id, &a.path).cmp(&(b.package_id, b.document_id, &b.path))
    });
    let mut policies = BTreeMap::new();
    for (name, policy) in &ctx.policies.0 {
        ctx.cancel.checkpoint()?;
        let spec = ctx
            .registry
            .relation_by_id(policy.input.relation_id())
            .ok_or_else(|| invalid("policy declaration missing"))?;
        let rows = pse_relations::cells::cells_from_batch(
            ctx.registry,
            spec,
            policy.input.relation().batch(),
        )?;
        policies.insert(
            name.clone(),
            StagePolicy {
                policy_id: policy.policy_id,
                relation: spec.key.to_string(),
                rows: encode_rows(&rows),
            },
        );
    }
    let value = StageContext {
        declarations,
        sources,
        policies,
        engine: if executes_plans {
            Some(
                ctx.session
                    .ok_or_else(|| invalid("engine context missing"))?
                    .semantic_inputs(),
            )
        } else {
            None
        },
    };
    let mut counter = Counter(0);
    serde_json::to_writer(&mut counter, &value).map_err(|error| invalid(error.to_string()))?;
    let retained = heap_extent(&value)?;
    if counter.0 > 16 << 20 || retained > 64 << 20 {
        return Err(invalid(
            "durable semantic context exceeds bounded control envelope",
        ));
    }
    if retained > reservation.size() {
        return Err(invalid(
            "durable context construction exceeded its actual-input forecast",
        ));
    }
    reservation.shrink(reservation.size() - retained);
    Ok(OwnedContext {
        value,
        _reservation: reservation,
    })
}
fn encode_rows(rows: &[Vec<Cell>]) -> Vec<Vec<String>> {
    rows.iter()
        .map(|row| row.iter().map(Cell::literal_spec).collect())
        .collect()
}
struct Counter(usize);
impl std::io::Write for Counter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 = self
            .0
            .checked_add(bytes.len())
            .ok_or_else(|| std::io::Error::other("context size overflow"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
