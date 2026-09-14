// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Compare complete admitted logical rows, including duplicates, without hash witnesses.
use crate::{CompilerError, passes::dag::invalid};
use pse_catalog::{Catalog, store::membership::validation_extent};
use pse_ids::CancellationToken;
use pse_relations::RecordBatch;

pub(super) fn rows(
    catalog: &Catalog,
    relation: &str,
    expected: &[RecordBatch],
    actual: &[RecordBatch],
    cancel: &CancellationToken,
) -> Result<(), CompilerError> {
    let registry = catalog.registry();
    let spec = registry
        .relation(relation)
        .ok_or_else(|| invalid("comparison declaration missing"))?;
    let mut reservation = catalog.reserver().open("compiler:stage-output-values");
    for batch in expected.iter().chain(actual) {
        cancel.checkpoint()?;
        reservation
            .try_grow(validation_extent(batch)?)
            .map_err(|error| CompilerError::Catalog(error.into()))?;
    }
    let values = |batches: &[RecordBatch]| -> Result<Vec<Vec<String>>, CompilerError> {
        let mut values = Vec::new();
        for batch in batches {
            cancel.checkpoint()?;
            values.extend(
                pse_relations::cells::cells_from_batch(registry, spec, batch)?
                    .into_iter()
                    .map(|row| {
                        row.into_iter()
                            .map(|cell| cell.literal_spec())
                            .collect::<Vec<_>>()
                    }),
            );
        }
        values.sort();
        Ok(values)
    };
    if values(expected)? != values(actual)? {
        return Err(invalid(format!(
            "actual stage rows differ from executed producer: {relation}"
        )));
    }
    Ok(())
}
