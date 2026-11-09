// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Recorded trajectory use from one original protected completion and its coordinates.
//! This interprets original permission; it never restores a live scientific capability.

use super::{ObservedProjection, ResultReader, WorkflowError, contract};
use crate::workflow::{PackScientificState as State, ResultSnapshot};
use datafusion::arrow::array::Array;
use pse_columnar::MemoryConsumer;
use pse_ids::{ContentHash, SemanticId};
use pse_model::generated::{
    enums::ComputationKind,
    identities::{InstanceId, RunId},
    structures::ScientificArtifactCoordinate,
};
use pse_relations::{
    columnar::ArrowValue,
    generated::runtime::{
        scientific_contexts,
        simulation_samples::{self, columns as sample},
    },
};
use std::{collections::BTreeMap, sync::Arc};

pub(super) const SAMPLE_ROOTS: &[&str] = &[
    sample::RUN_ID.name,
    sample::SAMPLE.name,
    sample::TIME.name,
    sample::SYMBOL_ID.name,
    sample::QUANTITY_ID.name,
    sample::UNIT_ID.name,
    sample::VALUE.name,
];

// Deliberately passive projection of the existing native profile document. Other
// controls and the applied parameter vector are skipped, not adopted or restored.
#[derive(serde::Deserialize)]
struct RecordedSampleTimes {
    samples: Vec<f64>,
}

fn times(bytes: &[u8]) -> Result<Vec<f64>, WorkflowError> {
    let (version, profile, _): (u8, RecordedSampleTimes, serde::de::IgnoredAny) =
        serde_json::from_slice(bytes)
            .map_err(|error| contract(format!("recorded trajectory coordinates: {error}")))?;
    if version != 1
        || profile.samples.is_empty()
        || profile.samples.iter().any(|time| !time.is_finite())
        || profile.samples.windows(2).any(|pair| pair[0] >= pair[1])
    {
        return Err(contract("recorded trajectory sample interpretation"));
    }
    Ok(profile.samples)
}

fn read<T: ArrowValue>(column: &dyn Array, row: usize) -> Result<T, WorkflowError> {
    T::read(column, row).map_err(crate::workflow::relation)
}

/// Match the recorded coordinate interpretation; this does not restore native proof.
pub(in crate::workflow) fn context_matches(
    context: &scientific_contexts::View<'_>,
    header: &pse_model::generated::runtime::computation_runs::Row,
    lineage: &pse_model::generated::runtime::run_lineage::Row,
) -> Result<bool, WorkflowError> {
    Ok(context.len() == 1
        && read::<RunId>(context.run_id_column(), 0)? == header.run_id
        && read::<i64>(context.step_column(), 0)? == 0
        && read::<Option<InstanceId>>(context.instance_id_column(), 0)? == lineage.instance_id
        && read::<ContentHash>(context.source_identity_column(), 0)? == lineage.revision
        && read::<ContentHash>(context.physical_identity_column(), 0)? == lineage.physical_identity
        && read::<ContentHash>(context.preparation_identity_column(), 0)?
            == lineage.preparation_identity
        && read::<ContentHash>(context.profile_identity_column(), 0)? == header.profile_identity
        && read::<Option<pse_model::generated::identities::DeclarationId>>(
            context.model_id_column(),
            0,
        )? == lineage.model_id
        && read::<Option<pse_model::generated::identities::DeclarationId>>(
            context.case_id_column(),
            0,
        )? == lineage.case_id
        && context.fit_id_column().is_null(0)
        && !context.time_column().is_null(0))
}

/// Compare original physical coordinates only; this predicate cannot issue permission.
pub(in crate::workflow) fn sample_matches(
    row: &simulation_samples::Row,
    run: RunId,
    times: &[f64],
    observed: usize,
    outputs: &BTreeMap<SemanticId, (SemanticId, SemanticId)>,
) -> bool {
    let Ok(sample) = usize::try_from(row.sample) else {
        return false;
    };
    let output = pse_modeling::derived::Relationship::MemberOutput {
        symbol: row.symbol_id,
    }
    .identity();
    row.run_id == run
        && sample < observed
        && times.get(sample).map(|at| at.to_bits()) == Some(row.time.to_bits())
        && outputs.get(&output) == Some(&(row.quantity_id, row.unit_id))
        && row.value.is_finite()
}

impl ResultReader {
    pub(super) async fn recorded_trajectory_usability(
        &self,
        projection: Option<&Arc<pse_relations::validate::PreparedInspectionProjection>>,
    ) -> Result<State, WorkflowError> {
        let Some(stored) = crate::workflow::durable::result_scientific_completion(
            self.services.canonical_store()?,
            &self.services.pool(),
            &self.read,
        )
        .await?
        else {
            return Ok(State::Unknown);
        };
        let Some(completion) = &stored.completion else {
            return Ok(State::Unknown);
        };
        let Some(header) = &completion.computation else {
            return Ok(State::Unknown);
        };
        // Fitting spans independently interpreted experiments. It cannot borrow
        // this standalone trajectory's one-context interpretation.
        if header.kind != ComputationKind::Simulation || completion.lineage.len() != 1 {
            return Ok(State::Unknown);
        }
        let lineage = &completion.lineage[0];
        let Some(member) = lineage.instance_id else {
            return Ok(State::Unknown);
        };
        let [permission] = completion.member_permissions.as_slice() else {
            return Ok(State::Unknown);
        };
        if self.read.run().key != format!("run:{}", header.run_id)
            || header.run_id != lineage.run_id
            || lineage.step != 0
            || header.source_identity != lineage.revision
            || header.profile_identity != lineage.profile_identity
            || !permission.records_original(
                header.run_id,
                Some(stored.attempt_id),
                member,
                lineage.revision,
                lineage.physical_identity,
                lineage.preparation_identity,
            )
        {
            return Ok(State::Unknown);
        }
        // Descriptive historical permission is distinct from native custody.
        // Exact protected provenance allows its original decision to be read.
        let decision = permission.recorded_permission();
        if !decision.report_succeeded || !decision.permits_result() {
            return Ok(State::Unusable);
        }
        if decision.observed == 0
            || decision.observed > decision.requested
            || header
                .completed_samples
                .and_then(|count| usize::try_from(count).ok())
                != Some(decision.observed)
        {
            return Ok(State::Unknown);
        }
        let context_name = scientific_contexts::RELATION_ID.to_string();
        let Some(context_set) = self.read.sets().iter().find(|set| set.name == context_name) else {
            return Ok(State::Unknown);
        };
        if context_set.row_count != 1 {
            return Ok(State::Unknown);
        }
        let spec = self
            .services
            .registry()
            .relation(scientific_contexts::QUALIFIED_NAME)
            .ok_or_else(|| contract("original trajectory context relation absent"))?;
        let schema = pse_schema::arrow::relation_schema_ref(self.services.registry(), spec)
            .map_err(|error| pse_engine::EngineError::Semantic(Arc::new(error)))?;
        // Share this exact ResultRead and its prepaid metadata owner. No fresh
        // selection, current source lookup, provider factory or unprotected query.
        let context_snapshot = ResultSnapshot::capture(
            &self.services,
            self.read.clone(),
            self.read_owner.clone(),
            spec.id,
            &context_set.key,
            schema,
            0,
            1,
        )
        .await?;
        let [block] = context_snapshot.blocks() else {
            return Ok(State::Unknown);
        };
        let context_batch = context_snapshot.read_block(block, &self.cancel).await?;
        let context = scientific_contexts::View::from_checked(&context_batch)
            .map_err(crate::workflow::relation)?;
        if !context_matches(&context, header, lineage)? {
            return Ok(State::Unknown);
        }
        let configuration = context.configuration_column().value(0);
        let output_offsets = context.outputs_column().value_offsets();
        let start =
            usize::try_from(output_offsets[0]).map_err(|_| contract("trajectory output offset"))?;
        let end =
            usize::try_from(output_offsets[1]).map_err(|_| contract("trajectory output offset"))?;
        let owner = MemoryConsumer::new("result:recorded-trajectory-coordinates")
            .register(&self.services.pool());
        // JSON times can grow their Vec geometrically; eight bytes per input
        // byte cover that copy. Applied inputs and other profile fields are skipped.
        let bytes = configuration
            .len()
            .checked_mul(8)
            .and_then(|bytes| {
                (end - start)
                    .checked_mul(128)
                    .and_then(|outputs| bytes.checked_add(outputs))
            })
            .ok_or_else(|| contract("recorded trajectory coordinate extent"))?;
        owner
            .try_grow(bytes)
            .map_err(pse_engine::EngineError::from)?;
        let sample_times = times(configuration)?;
        if sample_times.len() != decision.requested {
            return Ok(State::Unknown);
        }
        let mut outputs = BTreeMap::new();
        for index in start..end {
            let coordinate: ScientificArtifactCoordinate =
                read(context.outputs_column().values().as_ref(), index)?;
            if coordinate.scale != 1.
                || coordinate.offset != 0.
                || outputs
                    .insert(coordinate.id, (coordinate.quantity_id, coordinate.unit_id))
                    .is_some()
            {
                return Ok(State::Unknown);
            }
        }
        if outputs.is_empty() {
            return Ok(State::Unknown);
        }
        let expected_rows = outputs
            .len()
            .checked_mul(decision.observed)
            .ok_or_else(|| contract("recorded trajectory population extent"))?;
        let mut rows = self.fresh_selection("result:trajectory-permission-read")?;
        let mut previous = None;
        let mut count = 0usize;
        let projection = projection.map(ObservedProjection::roots_only);
        while let Some(decoded) = rows.next_native_batch(projection).await? {
            let batch = decoded.batch();
            let column = |name| {
                batch
                    .column_by_name(name)
                    .map(|array| array.as_ref())
                    .ok_or_else(|| contract("original trajectory permission root absent"))
            };
            let run_id = column(sample::RUN_ID.name)?;
            let sample_index = column(sample::SAMPLE.name)?;
            let time = column(sample::TIME.name)?;
            let symbol = column(sample::SYMBOL_ID.name)?;
            let quantity = column(sample::QUANTITY_ID.name)?;
            let unit = column(sample::UNIT_ID.name)?;
            let value = column(sample::VALUE.name)?;
            for row in 0..batch.num_rows() {
                let sample = simulation_samples::Row {
                    run_id: read(run_id, row)?,
                    sample: read(sample_index, row)?,
                    time: read(time, row)?,
                    symbol_id: read(symbol, row)?,
                    quantity_id: read(quantity, row)?,
                    unit_id: read(unit, row)?,
                    value: read(value, row)?,
                };
                if !sample_matches(
                    &sample,
                    header.run_id,
                    &sample_times,
                    decision.observed,
                    &outputs,
                ) {
                    return Ok(State::Unknown);
                }
                let coordinate = (sample.symbol_id, sample.sample);
                if previous.is_some_and(|previous| previous >= coordinate) {
                    return Ok(State::Unknown);
                }
                // Canonical producer order is symbol then original sample. This
                // streaming uniqueness check neither renumbers nor deduplicates rows.
                previous = Some(coordinate);
                count = count
                    .checked_add(1)
                    .ok_or_else(|| contract("trajectory selected population extent"))?;
            }
        }
        Ok(if count == 0 {
            State::Unknown
        } else if count == expected_rows {
            State::Usable
        } else {
            State::Partial
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn recorded_sample_projection_does_not_adopt_profile_or_inputs() {
        assert_eq!(
            super::times(
                br#"[1,{"samples":[5,5.25,10],"unknown_control":{"opaque":true}},[999,888]]"#
            )
            .unwrap(),
            [5., 5.25, 10.]
        );
        for bytes in [
            br#"[2,{"samples":[5,10]},[]]"#.as_slice(),
            br#"[1,{"samples":[5,5]},[]]"#,
            br#"[1,{"samples":[10,5]},[]]"#,
            br#"[1,{"samples":[]},[]]"#,
            br#"[1,{"samples":[5],"samples":[10]},[]]"#,
        ] {
            assert!(super::times(bytes).is_err());
        }
    }
}
