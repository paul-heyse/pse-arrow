// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Offline historical reconstruction through the ordinary scientific owners.
use super::{result_blocks, result_projection};
use pse_operations::{
    canonical::CanonicalError,
    canonical_preparation_activation::{
        HistoricalProjection, HistoricalResultRebuilder as Rebuilder,
    },
};
use std::sync::Arc;

/// Actual registry/function owner and prepaid decoding allowance retained by the
/// offline supervisor's native operation. This is a codec capability, never a
/// database write or activation permit.
#[derive(Debug)]
pub struct HistoricalResultRebuilder {
    model: pse_engine::session::BoundModel,
    _allocation: Arc<pse_columnar::MemoryReservation>,
}
impl HistoricalResultRebuilder {
    /// Retain the actual bound native context and existing operation reservation.
    /// # Errors
    /// The operation has not prepaid the bounded decoder and validation scratch.
    pub fn new(
        model: pse_engine::session::BoundModel,
        allocation: Arc<pse_columnar::MemoryReservation>,
    ) -> Result<Self, CanonicalError> {
        if allocation.size() < 128 * 1024 * 1024 {
            return Err(CanonicalError::PayloadLimit);
        }
        Ok(Self {
            model,
            _allocation: allocation,
        })
    }
}
impl Rebuilder for HistoricalResultRebuilder {
    fn rebuild(
        &self,
        relation: &str,
        batch: &pse_model::generated::runtime::canonical_result_batches::Row,
        metadata: &pse_model::generated::runtime::canonical_result_blocks::Row,
    ) -> Result<HistoricalProjection, CanonicalError> {
        let invalid = |error: &dyn std::fmt::Display| {
            CanonicalError::Configuration(format!("historical scientific block: {error}"))
        };
        let id = pse_ids::SemanticId::parse_hex(relation).map_err(|error| invalid(&error))?;
        let registry = self.model.registry();
        let spec = registry.relation_by_id(id).ok_or_else(|| {
            CanonicalError::Configuration("unknown historical scientific relation".into())
        })?;
        if spec.key.namespace != pse_schema::model::Namespace::Runtime {
            return Err(CanonicalError::Configuration(
                "historical result relation namespace".into(),
            ));
        }
        let schema = pse_schema::arrow::relation_schema_ref(registry, spec)
            .map_err(|error| invalid(&error))?;
        let rows = usize::try_from(batch.row_count).map_err(|_| CanonicalError::PayloadLimit)?;
        let decoded = result_blocks::decode_result_block(batch.payload.as_slice(), schema, rows)
            .map_err(|error| invalid(&error))?;
        if decoded.num_columns() as u64 != metadata.columns {
            return Err(CanonicalError::Configuration(
                "historical scientific column extent differs".into(),
            ));
        }
        let checked = pse_relations::columnar::FieldCheckedBatch::admit(
            registry,
            spec,
            decoded,
            &self.model.validation_context(),
            &pse_columnar::CancellationToken::new(),
        )
        .map_err(|error| invalid(&error))?;
        let outputs = if id == pse_relations::generated::runtime::simulation_samples::RELATION_ID {
            result_projection::trajectory_outputs(
                &checked,
                &batch.key,
                &batch.result_set,
                metadata.start,
                0,
                rows,
            )
            .map_err(|error| invalid(&error))?
        } else {
            Vec::new()
        };
        let rows = result_projection::scalar_rows_at(
            &checked,
            &batch.attempt,
            &batch.result_set,
            &batch.key,
            0,
            rows,
            metadata.start,
        )
        .map_err(|error| invalid(&error))?;
        Ok(HistoricalProjection { rows, outputs })
    }
}

/// Real registry-built IPC for the explicit disposable offline fixture route.
/// The caller consumes one bounded block before requesting the next.
#[cfg(any(test, feature = "canonical-tests"))]
impl HistoricalResultRebuilder {
    /// # Errors
    /// Actual registry validation or bounded IPC encoding fails.
    pub fn fixture_block(
        &self,
        trajectory: bool,
        attempt: &str,
        ordinal: u64,
        origin: u64,
    ) -> Result<
        (
            pse_model::generated::runtime::canonical_result_batches::Row,
            pse_model::generated::runtime::canonical_result_blocks::Row,
            Vec<pse_model::generated::runtime::canonical_result_block_outputs::Row>,
        ),
        CanonicalError,
    > {
        use pse_operations::canonical_execution::{
            result_batch_key, result_payload_digest, result_set_key,
        };
        use pse_relations::generated::runtime::{simulation_samples, solve_variables};
        let fail = |error: &dyn std::fmt::Display| CanonicalError::Configuration(error.to_string());
        let registry = self.model.registry();
        let validation = self.model.validation_context();
        let run = pse_ids::SemanticId::from_bytes([77; 16]).into();
        let owner = pse_ids::SemanticId::from_bytes([78; 16]);
        let table = if trajectory {
            let mut builder =
                simulation_samples::Builder::with_registry(registry, 4096, &validation)
                    .map_err(|error| fail(&error))?;
            for index in 0..4096_u64 {
                builder
                    .push(simulation_samples::Row {
                        run_id: run,
                        sample: (origin + index) as i64,
                        time: (origin + index) as f64,
                        symbol_id: owner,
                        quantity_id: pse_ids::SemanticId::NIL,
                        unit_id: pse_ids::SemanticId::NIL,
                        value: if index == 0 { -0.0 } else { index as f64 },
                    })
                    .map_err(|error| fail(&error))?;
            }
            builder.finish().map_err(|error| fail(&error))?
        } else {
            let mut builder = solve_variables::Builder::with_registry(registry, 2048, &validation)
                .map_err(|error| fail(&error))?;
            for index in 0..2048_u64 {
                builder
                    .push(solve_variables::Row {
                        run_id: run,
                        step: 4_000_000_001 + origin as i64 + index as i64,
                        symbol_id: owner,
                        quantity_id: None,
                        unit_id: None,
                        fixed: false,
                        parameter: false,
                        domain: None,
                        value: Some(if index == 0 { -0.0 } else { index as f64 }),
                        lower: None,
                        upper: None,
                        lower_violation: None,
                        upper_violation: None,
                        tolerance: None,
                        lower_dual: None,
                        upper_dual: None,
                        reduced_cost: None,
                        stationarity: None,
                        dual_qualification:
                            pse_model::generated::enums::DualQualification::Unavailable,
                    })
                    .map_err(|error| fail(&error))?;
            }
            builder.finish().map_err(|error| fail(&error))?
        };
        let name = table.relation_id().to_string();
        let set = result_set_key(attempt, &name);
        let key = result_batch_key(attempt, &set, ordinal);
        let payload =
            result_blocks::encode_result_block(table.batch()).map_err(|error| fail(&error))?;
        let digest = result_payload_digest(&payload);
        let count = table.batch().num_rows() as u64;
        let block = pse_model::generated::runtime::canonical_result_blocks::Row {
            key: key.clone(),
            result_set: set.clone(),
            batch: key.clone(),
            output: name,
            partition: "0".into(),
            ordinal,
            start: origin,
            end: origin + count,
            rows: count,
            columns: table.batch().num_columns() as u64,
            coordinate_min: None,
            coordinate_max: None,
            payload_bytes: payload.len() as u64,
            payload_digest: digest.clone(),
            interpretation: "pse.substrate.v3".into(),
        };
        let outputs = if trajectory {
            result_projection::trajectory_outputs(&table, &key, &set, origin, 0, count as usize)
                .map_err(|error| fail(&error))?
                .into_iter()
                .map(|mut row| {
                    row.interpretation = "pse.substrate.v3".into();
                    row
                })
                .collect()
        } else {
            Vec::new()
        };
        let batch = pse_model::generated::runtime::canonical_result_batches::Row {
            key,
            attempt: attempt.into(),
            result_set: set,
            ordinal,
            digest,
            payload: payload.into(),
            row_count: count,
        };
        Ok((batch, block, outputs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn historical_rebuilder_uses_original_ipc_exact_integer_keys_and_dense_absolute_rows() {
        let fixture = pse_testkit::NativeFixture::new(
            std::num::NonZeroUsize::new(512 * 1024 * 1024).unwrap(),
        )
        .unwrap();
        let model = Arc::new(fixture.factory)
            .bind_model(pse_schema::shared_registry().unwrap())
            .unwrap();
        let owner = pse_columnar::MemoryConsumer::new("historical-ipc-oracle")
            .register(&fixture.resources.pool);
        owner.try_grow(384 * 1024 * 1024).unwrap();
        let rebuilder = HistoricalResultRebuilder::new(model, Arc::new(owner)).unwrap();
        let (batch, block, _) = rebuilder
            .fixture_block(false, "original-attempt", 7, 100_000)
            .unwrap();
        let original = batch.payload.as_slice().to_vec();
        let projected = rebuilder.rebuild(&block.output, &batch, &block).unwrap();
        assert_eq!(projected.rows.len(), 2048);
        assert!(projected.outputs.is_empty());
        let first = projected.rows.iter().find(|row| row.offset == 0).unwrap();
        assert_eq!(
            first.run_id,
            pse_ids::SemanticId::from_bytes([77; 16]).into()
        );
        assert_eq!(first.owner, pse_ids::SemanticId::from_bytes([78; 16]));
        assert_eq!(first.step, Some(4_000_100_001));
        assert_eq!(first.partition, "step:4000100001");
        assert_eq!(first.row, 100_000);
        assert_eq!(first.key, format!("{}:100000", batch.key));
        assert_eq!(batch.payload.as_slice(), original);
        let (batch, block, outputs) = rebuilder
            .fixture_block(true, "trajectory-attempt", 3, 8000)
            .unwrap();
        let projected = rebuilder.rebuild(&block.output, &batch, &block).unwrap();
        assert!(projected.rows.is_empty());
        assert_eq!(projected.outputs.len(), 1);
        assert_eq!(
            (projected.outputs[0].start, projected.outputs[0].end),
            (8000, 12096)
        );
        assert_eq!(
            (
                projected.outputs[0].coordinate_min,
                projected.outputs[0].coordinate_max
            ),
            (Some(8000.0), Some(12095.0))
        );
        assert_eq!(outputs[0].output, projected.outputs[0].output);
        let mut malformed = batch.clone();
        malformed.payload = vec![0, 1, 2].into();
        assert!(
            rebuilder
                .rebuild(&block.output, &malformed, &block)
                .is_err()
        );
        assert!(
            rebuilder
                .rebuild("old:text:coordinate", &batch, &block)
                .is_err()
        );
    }
}
