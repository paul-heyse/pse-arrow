// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Keyed physical input is copied into owned scalars before an asynchronous boundary.
use super::{Runtime, WorkflowError, contract, relation};
use datafusion::arrow::record_batch::RecordBatchReader;
use pse_ids::SemanticId;
use pse_kernels::Port;
use pse_model::generated::{extension_values::QuantityValue, structures::ScientificCoordinateValue, enums::{NativePrimalCoordinateRole, PhysicalInputKind}};
use pse_model::generated::runtime::{diagnostic_samples, initialization_stages, primal_values, state_tolerances};
use pse_quantity::{CanonicalConversionPlan, IndexSet, ResolvedPhysicalContract};
use pse_relations::columnar::{FieldCheckedBatch, RelationRow};
use std::{collections::{BTreeMap, BTreeSet}, sync::Arc};

#[derive(Debug)]
struct Snapshot<R> {
    rows: Vec<R>,
    _owner: Arc<pse_columnar::AllocationLease>,
}

/// Owned physical Points submitted for a complete original primal start.
#[derive(Clone, Debug)]
pub struct PrimalValues(Arc<Snapshot<primal_values::Row>>);
/// Owned sparse physical Points, retaining explicit ordered empty stages.
#[derive(Clone, Debug)]
pub struct InitializationStages(Arc<Snapshot<initialization_stages::Row>>);
/// Owned named diagnostic samples containing sparse physical Points.
#[derive(Clone, Debug)]
pub struct DiagnosticSamples(Arc<Snapshot<diagnostic_samples::Row>>);
/// Owned physical Delta tolerances, pending actual state/scaling admission.
#[derive(Clone, Debug)]
pub struct StateTolerances(Arc<Snapshot<state_tolerances::Row>>);

impl Runtime {
    /// Full registry-owned Arrow schema, including relation identity and extension metadata.
    pub fn physical_input_schema(&self, kind: PhysicalInputKind) -> Result<datafusion::arrow::datatypes::SchemaRef, WorkflowError> {
        let registry = self.registry();
        let spec = match kind {
            PhysicalInputKind::PrimalValues => pse_relations::generated::runtime::primal_values::spec(registry),
            PhysicalInputKind::InitializationStages => pse_relations::generated::runtime::initialization_stages::spec(registry),
            PhysicalInputKind::DiagnosticSamples => pse_relations::generated::runtime::diagnostic_samples::spec(registry),
            PhysicalInputKind::StateTolerances => pse_relations::generated::runtime::state_tolerances::spec(registry),
        }.map_err(relation)?;
        pse_schema::arrow::relation_schema_ref(registry, spec).map_err(|error| contract(error.to_string()))
    }

    fn snapshot_physical<R: RelationRow>(
        &self,
        batches: impl RecordBatchReader,
    ) -> Result<Arc<Snapshot<R>>, WorkflowError> {
        let context = self.validation_context();
        let registry = self.registry();
        let spec = R::relation(registry).map_err(relation)?;
        pse_relations::validate::validate_schema(registry, spec, &batches.schema())
            .map_err(|errors| relation(pse_relations::RelationError::Validation { errors }))?;
        let token = pse_columnar::CancellationToken::new();
        let pool = self.shared.pool();
        let reservation = pse_columnar::MemoryConsumer::new("workflow:physical-input").register(&pool);
        let mut rows = Vec::new();
        for batch in batches {
            let batch = batch.map_err(|error| contract(error.to_string()))?;
            // Reserve before decoding; nested buffers bound the owned scalar payload too.
            let extent = batch.get_array_memory_size().checked_mul(4).ok_or_else(|| contract("physical input copy extent"))?
                .checked_add(batch.num_rows().checked_mul(size_of::<R>() + 128).ok_or_else(|| contract("physical input row extent"))?)
                .ok_or_else(|| contract("physical input extent"))?;
            reservation.try_grow(extent).map_err(|error| contract(error.to_string()))?;
            let checked = FieldCheckedBatch::admit(registry, spec, batch, &context, &token).map_err(relation)?;
            // Generated codecs copy scalar values only. No external Arrow buffer escapes.
            rows.extend(R::rows(&checked).map_err(relation)?);
        }
        Ok(Arc::new(Snapshot { rows, _owner: pse_columnar::AllocationLease::new(reservation) }))
    }
    /// Check the actual relation and extension metadata, and isolate scalar custody.
    pub fn snapshot_primal_values(&self, batches: impl RecordBatchReader) -> Result<PrimalValues, WorkflowError> {
        self.snapshot_physical(batches).map(PrimalValues)
    }
    /// Check initialization relation metadata before retaining owned stage values.
    pub fn snapshot_initialization_stages(&self, batches: impl RecordBatchReader) -> Result<InitializationStages, WorkflowError> {
        let snapshot: Arc<Snapshot<initialization_stages::Row>> = self.snapshot_physical(batches)?;
        let mut stages = BTreeSet::new();
        for row in &snapshot.rows {
            if !stages.insert(row.stage) { return Err(contract("duplicate initialization stage")); }
            unique_coordinates(&row.values)?;
        }
        if stages.iter().copied().ne(0..i64::try_from(stages.len()).map_err(|_| contract("stage count"))?) {
            return Err(contract("initialization stages must be contiguous from zero"));
        }
        let stages = InitializationStages(snapshot);
        if stages.0.rows.is_empty() { return Err(contract("initialization requires at least one explicit stage")); }
        Ok(stages)
    }
    /// Check named sample uniqueness and retain owned scalar values.
    pub fn snapshot_diagnostic_samples(&self, batches: impl RecordBatchReader) -> Result<DiagnosticSamples, WorkflowError> {
        let snapshot: Arc<Snapshot<diagnostic_samples::Row>> = self.snapshot_physical(batches)?;
        let mut samples = BTreeSet::new();
        for row in &snapshot.rows {
            if !samples.insert(row.sample_id) { return Err(contract("duplicate diagnostic sample identity")); }
            unique_coordinates(&row.values)?;
        }
        Ok(DiagnosticSamples(snapshot))
    }
    /// Check the physical tolerance relation and retain owned scalar values.
    pub fn snapshot_state_tolerances(&self, batches: impl RecordBatchReader) -> Result<StateTolerances, WorkflowError> {
        let snapshot: Arc<Snapshot<state_tolerances::Row>> = self.snapshot_physical(batches)?;
        let mut keys = BTreeSet::new();
        for row in &snapshot.rows {
            if !keys.insert(row.coordinate_id) { return Err(contract("duplicate state tolerance coordinate")); }
            if !row.value.value.is_finite() || row.value.value <= 0.0 { return Err(contract("state tolerances must be finite and positive")); }
        }
        Ok(StateTolerances(snapshot))
    }
}

impl super::ModelingPackage {
    /// Own diagnostic scalar custody before asynchronous source preparation begins.
    pub fn snapshot_diagnostic_samples(&self, batches: impl RecordBatchReader) -> Result<DiagnosticSamples, WorkflowError> {
        self.runtime.snapshot_diagnostic_samples(batches)
    }
    /// Own initialization scalar custody before asynchronous source preparation begins.
    pub fn snapshot_initialization_stages(&self, batches: impl RecordBatchReader) -> Result<InitializationStages, WorkflowError> {
        self.runtime.snapshot_initialization_stages(batches)
    }
}

fn unique_coordinates(values: &[ScientificCoordinateValue]) -> Result<(), WorkflowError> {
    let mut keys = BTreeSet::new();
    for value in values {
        if !keys.insert(value.coordinate_id) { return Err(contract("duplicate physical coordinate")); }
        if !value.value.value.is_finite() { return Err(contract("physical coordinate must be finite")); }
    }
    Ok(())
}

/// Complete named physical meaning comes from the quantity key (basis, datum,
/// subject and shape); representation units never establish that meaning.
fn convert(
    registry: &pse_quantity::QuantityRegistry,
    expected: &Port,
    supplied: &QuantityValue,
    delta: bool,
) -> Result<f64, WorkflowError> {
    let physical = |error| super::math(pse_math::MathError::Quantity(error));
    let expected_contract = ResolvedPhysicalContract::named(expected.quantity, IndexSet::new(), registry).map_err(physical)?;
    let supplied_contract = ResolvedPhysicalContract::named(supplied.quantity_type_id.into(), IndexSet::new(), registry).map_err(physical)?;
    if !expected_contract.same_meaning(&supplied_contract) { return Err(contract("physical coordinate quantity differs from the prepared port")); }
    let source = CanonicalConversionPlan::registered(registry, expected.quantity, supplied.unit_id.into()).map_err(physical)?;
    let target = CanonicalConversionPlan::registered(registry, expected.quantity, expected.unit).map_err(physical)?;
    let value = if delta {
        supplied.value * source.scale() / target.scale()
    } else if source.source_unit() == target.source_unit() {
        supplied.value
    } else {
        (source.apply(supplied.value).map_err(physical)?.value() - target.offset()) / target.scale()
    };
    if !value.is_finite() || (delta && value <= 0.0) { return Err(contract("converted physical coordinate is outside its finite positive domain")); }
    Ok(value)
}

fn admit_values<'a>(
    registry: &pse_quantity::QuantityRegistry,
    ports: &[Port],
    values: impl Iterator<Item = (SemanticId, &'a QuantityValue)>,
    complete: bool,
    delta: bool,
) -> Result<BTreeMap<SemanticId, f64>, WorkflowError> {
    let ports: BTreeMap<_, _> = ports.iter().map(|port| (port.id, port)).collect();
    let mut admitted = BTreeMap::new();
    for (id, value) in values {
        let port = ports.get(&id).ok_or_else(|| contract("unknown physical coordinate"))?;
        let value = convert(registry, port, value, delta)?;
        if admitted.insert(id, value).is_some() { return Err(contract("duplicate physical coordinate")); }
    }
    if complete && admitted.len() != ports.len() { return Err(contract("physical input requires every prepared coordinate exactly once")); }
    Ok(admitted)
}
impl PrimalValues {
    pub(super) fn admit(&self, registry: &pse_quantity::QuantityRegistry, ports: &[Port]) -> Result<BTreeMap<SemanticId, f64>, WorkflowError> {
        if self.0.rows.iter().any(|row| row.coordinate_role != NativePrimalCoordinateRole::Original) {
            return Err(contract("primal starts require original scientific coordinates"));
        }
        admit_values(registry, ports, self.0.rows.iter().map(|row| (row.coordinate_id, &row.value)), true, false)
    }
}
impl InitializationStages {
    #[cfg(feature = "solver-kinsol")]
    pub(super) fn admit(&self, registry: &pse_quantity::QuantityRegistry, ports: &[Port]) -> Result<Vec<BTreeMap<SemanticId, f64>>, WorkflowError> {
        let mut rows: Vec<_> = self.0.rows.iter().collect();
        rows.sort_by_key(|row| row.stage);
        rows.into_iter().map(|row| admit_values(registry, ports, row.values.iter().map(|value| (value.coordinate_id, &value.value)), false, false)).collect()
    }
}
impl DiagnosticSamples {
    pub(super) fn admit(&self, registry: &pse_quantity::QuantityRegistry, ports: &[Port]) -> Result<Vec<(SemanticId, BTreeMap<SemanticId, f64>)>, WorkflowError> {
        self.0.rows.iter().map(|row| Ok((row.sample_id, admit_values(registry, ports, row.values.iter().map(|value| (value.coordinate_id, &value.value)), false, false)?))).collect()
    }
}
impl StateTolerances {
    /// Owned pending document projection; physical admission is still contextual.
    pub fn coordinates(&self) -> Vec<ScientificCoordinateValue> {
        self.0.rows.iter().map(|row| ScientificCoordinateValue { coordinate_id: row.coordinate_id, value: row.value.clone() }).collect()
    }
}

pub(super) fn admit_tolerances(registry: &pse_quantity::QuantityRegistry, ports: &[Port], values: &[ScientificCoordinateValue]) -> Result<BTreeMap<SemanticId, f64>, WorkflowError> {
    admit_values(registry, ports, values.iter().map(|value| (value.coordinate_id, &value.value)), true, true)
}

#[cfg(test)]
pub(in crate::workflow) fn test_primal_batch(prepared: &super::ModelingSolvePreparation, values: BTreeMap<SemanticId, f64>) -> datafusion::arrow::record_batch::RecordBatch {
    let context = prepared.source.runtime.validation_context();
    let mut rows = pse_relations::columnar::RowBuilder::<primal_values::Row>::new(&context).unwrap();
    for (id, value) in values {
        let port = &prepared.model.case.compiled().plan.structure().variables().iter().find(|variable| variable.port.id == id).unwrap().port;
        rows.push(primal_values::Row { coordinate_id: id, coordinate_role: NativePrimalCoordinateRole::Original, value: QuantityValue { value, quantity_type_id: port.quantity.as_id(), unit_id: port.unit.as_id() } }).unwrap();
    }
    rows.finish().unwrap().batch().clone()
}

/// Original or constructor-declared auxiliary coordinates, retained with their actual owner.
#[derive(Clone, Debug)]
pub struct PrimalCoordinates {
    runtime: Runtime,
    ports: Arc<Vec<Port>>,
    role: NativePrimalCoordinateRole,
    _owner: Arc<pse_columnar::AllocationLease>,
}
impl PrimalCoordinates {
    pub(in crate::workflow) fn original(runtime: &Runtime, ports: Vec<Port>) -> Result<Self, WorkflowError> {
        Self::new(runtime, ports, NativePrimalCoordinateRole::Original)
    }
    pub(in crate::workflow) fn auxiliary(runtime: &Runtime, registry: &pse_quantity::QuantityRegistry, ids: &[SemanticId]) -> Result<Self, WorkflowError> {
        let quantity = registry.neutral_dimensionless().ok_or_else(|| contract("auxiliary primal physical meaning has no designated neutral dimensionless quantity"))?;
        let unit = registry.quantity_type(quantity).map_err(|error| contract(error.to_string()))?.canonical_unit;
        Self::new(runtime, ids.iter().map(|id| Port { id: *id, quantity, unit }).collect(), NativePrimalCoordinateRole::Auxiliary)
    }
    fn new(runtime: &Runtime, ports: Vec<Port>, role: NativePrimalCoordinateRole) -> Result<Self, WorkflowError> {
        let extent = ports.len().checked_mul(size_of::<Port>() + 64).and_then(|bytes| bytes.checked_add(512)).ok_or_else(|| contract("primal coordinate inventory extent"))?;
        let owner = runtime.shared.math().reserve("workflow:primal-coordinate-inventory", extent)?;
        let unique = ports.iter().map(|port| port.id).collect::<BTreeSet<_>>();
        if unique.len() != ports.len() { return Err(contract("duplicate primal coordinate declaration")); }
        Ok(Self { runtime: runtime.clone(), ports: Arc::new(ports), role, _owner: owner })
    }
    /// Copy a candidate into its registered keyed relation without inventing quantity metadata.
    pub fn table(&self, report: &pse_backend_native::solve::SolveReport) -> Result<FieldCheckedBatch, WorkflowError> {
        let candidate = report.candidate.as_ref();
        if candidate.is_some_and(|candidate| candidate.primal.len() != report.variables.len()) {
            return Err(contract("native primal extent differs from original coordinates"));
        }
        let scratch = report.variables.len().checked_mul(256).and_then(|bytes| bytes.checked_add(4096)).ok_or_else(|| contract("primal transport extent"))?;
        let _scratch = self.runtime.shared.math().reserve("workflow:primal-transport", scratch)?;
        let pool = self.runtime.shared.pool();
        let cancel = pse_columnar::CancellationToken::new();
        let context = self.runtime.validation_context();
        let mut columns = pse_relations::columnar::Collection::new(self.runtime.registry(), &pool, &cancel, &context);
        columns.ensure::<primal_values::Row>().map_err(relation)?;
        if let Some(candidate) = candidate {
            let mut seen = BTreeSet::new();
            for (id, value) in report.variables.iter().zip(&candidate.primal) {
                if !seen.insert(*id) || !value.is_finite() { return Err(contract("native primal has duplicate or nonfinite coordinates")); }
                let port = self.ports.iter().find(|port| port.id == *id).ok_or_else(|| contract("native primal lacks producer-owned physical meaning"))?;
                columns.push(primal_values::Row { coordinate_id: *id, coordinate_role: self.role, value: QuantityValue { value: *value, quantity_type_id: port.quantity.as_id(), unit_id: port.unit.as_id() } }).map_err(relation)?;
            }
        }
        columns.finish().map_err(relation)?.into_values().next().ok_or_else(|| contract("primal transport absent"))
    }
}
impl super::ModelingSolvePreparation {
    /// Actual original variable ports, independent of a native report's ID list.
    pub fn primal_coordinates(&self) -> Result<PrimalCoordinates, WorkflowError> {
        PrimalCoordinates::original(&self.source.runtime, self.model.case.compiled().plan.structure().variables().iter().map(|variable| variable.port.clone()).collect())
    }
}
impl super::ModelingPackage {
    /// Resolve an explicitly constructor-declared dimensionless auxiliary inventory.
    pub fn auxiliary_primal_coordinates(&self, ids: &[SemanticId]) -> Result<PrimalCoordinates, WorkflowError> {
        PrimalCoordinates::auxiliary(&self.runtime, self.physical.quantities(), ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::arrow::{datatypes::Schema, record_batch::{RecordBatch, RecordBatchIterator}};
    use pse_relations::columnar::RowBuilder;

    fn batch<R: RelationRow>(runtime: &Runtime, rows: impl IntoIterator<Item = R>) -> RecordBatch {
        let context = runtime.validation_context();
        let mut builder = RowBuilder::<R>::new(&context).unwrap();
        for row in rows { builder.push(row).unwrap(); }
        builder.finish().unwrap().batch().clone()
    }
    fn reader(batch: RecordBatch) -> impl RecordBatchReader {
        RecordBatchIterator::new([Ok(batch.clone())], batch.schema())
    }
    fn coordinate(port: &Port, value: f64) -> QuantityValue {
        QuantityValue { value, quantity_type_id: port.quantity.as_id(), unit_id: port.unit.as_id() }
    }
    fn scalar(id: u8, registry: &pse_quantity::QuantityRegistry) -> Port {
        let quantity = registry.neutral_dimensionless().unwrap();
        Port { id: SemanticId::from_bytes([id;16]), quantity, unit: registry.quantity_type(quantity).unwrap().canonical_unit }
    }
    #[test]
    fn physical_point_and_delta_use_complete_meaning_and_distinct_offset_rules() {
        let physical = super::super::tests::physical();
        let registry = physical.quantities();
        let quantity = registry.quantity_types().find(|quantity| quantity.name.as_deref() == Some("Temperature")).unwrap();
        let celsius = registry.units().find(|unit| unit.symbol == "degC").unwrap();
        let port = Port { id: SemanticId::NIL, quantity: quantity.id, unit: quantity.canonical_unit };
        let point = QuantityValue { value: 25., quantity_type_id: quantity.id.as_id(), unit_id: celsius.id.as_id() };
        assert!((convert(registry, &port, &point, false).unwrap() - 298.15).abs() < 1e-12);
        assert_eq!(convert(registry, &port, &point, true).unwrap(), 25.);
        let same = Port { unit: celsius.id, ..port.clone() };
        let negative_zero = QuantityValue { value: -0., ..point.clone() };
        assert_eq!(convert(registry, &same, &negative_zero, false).unwrap().to_bits(), (-0f64).to_bits());
        assert!(convert(registry, &same, &negative_zero, true).is_err());
        let unrelated = coordinate(&scalar(8, registry), 25.);
        assert!(convert(registry, &port, &unrelated, false).is_err());
    }
    #[test]
    fn physical_primal_keys_roles_coverage_and_snapshot_custody_are_admitted() {
        let runtime = super::super::tests::runtime();
        let physical = super::super::tests::physical();
        let ports = [scalar(1, physical.quantities()), scalar(2, physical.quantities())];
        let row = |port: &Port, value| primal_values::Row { coordinate_id: port.id, coordinate_role: NativePrimalCoordinateRole::Original, value: coordinate(port, value) };
        let input = batch(&runtime, [row(&ports[1], 9.), row(&ports[0], -0.)]);
        let snapshot = runtime.snapshot_primal_values(reader(input.clone())).unwrap();
        drop(input);
        let lease = Arc::downgrade(&snapshot.0._owner);
        let clone = snapshot.clone();
        drop(snapshot);
        assert!(lease.upgrade().is_some());
        let values = clone.admit(physical.quantities(), &ports).unwrap();
        assert_eq!(values[&ports[1].id], 9.);
        assert_eq!(values[&ports[0].id].to_bits(), (-0f64).to_bits());
        drop(clone);
        assert!(lease.upgrade().is_none());
        for rows in [vec![row(&ports[0], 1.)], vec![row(&ports[0], 1.), row(&ports[0], 2.)], vec![row(&scalar(3, physical.quantities()), 1.), row(&ports[1], 2.)]] {
            let snapshot = runtime.snapshot_primal_values(reader(batch(&runtime, rows)));
            assert!(snapshot.and_then(|snapshot| snapshot.admit(physical.quantities(), &ports)).is_err());
        }
        let auxiliary = primal_values::Row { coordinate_role: NativePrimalCoordinateRole::Auxiliary, ..row(&ports[0], 1.) };
        assert!(runtime.snapshot_primal_values(reader(batch(&runtime, [auxiliary]))).unwrap().admit(physical.quantities(), &ports[..1]).is_err());
    }
    #[test]
    fn physical_schema_metadata_is_authoritative_even_for_empty_streams() {
        let runtime = super::super::tests::runtime();
        for kind in [PhysicalInputKind::PrimalValues, PhysicalInputKind::InitializationStages, PhysicalInputKind::DiagnosticSamples, PhysicalInputKind::StateTolerances] {
            let schema = runtime.physical_input_schema(kind).unwrap();
            assert!(schema.metadata().contains_key(pse_schema::arrow::KEY_CONTRACT_FINGERPRINT));
            assert!(schema.metadata().contains_key(pse_schema::arrow::KEY_CONTRACT_ID));
        }
        let schema = runtime.physical_input_schema(PhysicalInputKind::StateTolerances).unwrap();
        let empty = RecordBatchIterator::new(std::iter::empty(), schema.clone());
        assert!(runtime.snapshot_state_tolerances(empty).is_ok());
        let stripped = Arc::new(Schema::new(schema.fields().clone()));
        let empty = RecordBatchIterator::new(std::iter::empty(), stripped);
        assert!(runtime.snapshot_state_tolerances(empty).is_err());
    }
    #[test]
    fn physical_stages_retain_empty_stages_and_reject_gaps_duplicates_and_nonfinite_points() {
        let runtime = super::super::tests::runtime();
        let physical = super::super::tests::physical();
        let port = scalar(1, physical.quantities());
        let point = ScientificCoordinateValue { coordinate_id: port.id, value: coordinate(&port, -5.) };
        let input = batch(&runtime, [initialization_stages::Row { stage: 1, values: vec![point.clone()] }, initialization_stages::Row { stage: 0, values: vec![] }]);
        let stages = runtime.snapshot_initialization_stages(reader(input)).unwrap();
        assert_eq!(stages.0.rows.len(), 2);
        assert!(stages.0.rows.iter().find(|row| row.stage == 0).unwrap().values.is_empty());
        for rows in [vec![], vec![initialization_stages::Row { stage: 1, values: vec![] }], vec![initialization_stages::Row { stage: 0, values: vec![point.clone(), point.clone()] }]] {
            assert!(runtime.snapshot_initialization_stages(reader(batch(&runtime, rows))).is_err());
        }
        let nonfinite = ScientificCoordinateValue { value: QuantityValue { value: f64::NAN, ..point.value }, ..point };
        let input = batch(&runtime, [diagnostic_samples::Row { sample_id: SemanticId::NIL, values: vec![nonfinite] }]);
        assert!(runtime.snapshot_diagnostic_samples(reader(input)).is_err());
    }
    #[test]
    fn physical_simulation_delta_admission_resolves_scales_once_and_preserves_signed_time() {
        let physical = super::super::tests::physical();
        let registry = physical.quantities();
        let quantity = registry.quantity_types().find(|quantity| quantity.name.as_deref() == Some("Temperature")).unwrap();
        let celsius = registry.units().find(|unit| unit.symbol == "degC").unwrap();
        let state = Port { id: SemanticId::from_bytes([1;16]), quantity: quantity.id, unit: quantity.canonical_unit };
        let parameter = scalar(2, registry);
        let mut controls = pse_backend_native::dynamics::Profile::default().with_coordinates(
            vec![ScientificCoordinateValue { coordinate_id: state.id, value: QuantityValue { value: 0.2, quantity_type_id: state.quantity.as_id(), unit_id: celsius.id.as_id() } }], vec![],
            vec![ScientificCoordinateValue { coordinate_id: parameter.id, value: coordinate(&parameter, 3.) }]);
        controls.start = -1.000_000_000_1;
        controls.end = 0.000_000_000_1;
        controls.samples = vec![controls.start, -0., controls.end];
        let request = super::super::PhysicalSimulationRequest::new(controls);
        let decoded: super::super::PhysicalSimulationRequest = serde_json::from_slice(&serde_json::to_vec(&request).unwrap()).unwrap();
        let admitted = decoded.admit(registry, &[state.clone()], &[], &[parameter], &[2.]).unwrap();
        assert_eq!(admitted.atol, vec![0.1]);
        assert_eq!(admitted.parameter_scales, vec![3.]);
        assert_eq!(admitted.start.to_bits(), request.controls.start.to_bits());
        assert_eq!(admitted.end.to_bits(), request.controls.end.to_bits());
        assert_eq!(admitted.samples[1].to_bits(), (-0f64).to_bits());
        assert!(decoded.admit(registry, &[state], &[], &[], &[2.]).is_err());
    }
}
