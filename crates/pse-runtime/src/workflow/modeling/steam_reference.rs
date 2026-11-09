// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Compare the actual authored wet turbine's original coordinates with isolated IAPWS95.

#[path = "../../../tests/reference_support/environment.rs"]
mod reference_environment;
#[path = "../../../tests/reference_support/process.rs"]
mod reference_process;

use super::{DeclarationId, ModelingPackage, SemanticId};
use crate::{
    reference_exchange::{self, ReferenceCaseContract, ReferenceFixedInput, ReferenceInputArtifact, ReferenceObservableContract, ScalarAdmission},
    workflow::{ExecutionServices, PhysicalInspectionDocument, PhysicalInspectionQuery, PhysicalInspectionRequest, PhysicalQuantityDescription, modeling::ModelingResult},
};
use pse_model::reference_exchange::{ComparisonAllowance, ReferenceEnvelope, ReferenceMeaning, ReferenceObservable, ReferenceOutcome, compare};
use sha2::{Digest as _, Sha256};
use std::{collections::BTreeMap, path::{Path, PathBuf}, process::Command};

const LIMIT: usize = 1 << 20;
const CASE: &str = "steam_fixtures.wet_water_turbine";
const ARTIFACTS: [&str; 4] = [
    "packages/reference/process/models/pressure-changer.pse",
    "packages/reference/process/models/steam-equipment.pse",
    "packages/reference/seed-data/models/steam-fixtures.pse",
    "packages/reference/thermodynamics/models/steam.pse",
];
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn verify_property_images(python: &Path, evidence: &BTreeMap<String, String>) -> TestResult {
    let selected = std::env::var_os("IDAES_DATA").map(PathBuf::from).or_else(|| {
        std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".idaes"))
    }).ok_or("prepared IDAES data root absent")?;
    let library = selected.join("bin/general_helmholtz_external.so").canonicalize()?;
    let python_home = python.parent().and_then(Path::parent)
        .ok_or("selected reference interpreter has no environment root")?.canonicalize()?;
    for (role, filename) in [
        ("library", "general_helmholtz_external.so"),
        ("eos", "h2o_expressions_eos.nl"),
        ("parameters", "h2o_parameters.json"),
    ] {
        let prefix = format!("steam.wet_turbine.iapws95.{role}");
        let path = evidence.get(&format!("{prefix}.path")).map(Path::new)
            .ok_or("performed IAPWS95 image path absent")?;
        if (role == "library" && path != library)
            || (role != "library" && (!path.starts_with(&python_home) || !path.ends_with(filename)))
            || evidence.get(&format!("{prefix}.sha256")) != Some(&reference_environment::image_digest(path)?)
        {
            return Err(format!("performed IAPWS95 {role} differs from its prepared image").into());
        }
    }
    Ok(())
}

pub(super) fn compare_wet(
    package: &ModelingPackage,
    result: &ModelingResult,
    selected: DeclarationId,
    services: &ExecutionServices,
) -> TestResult {
    let capture = crate::scientific_inputs::family("steam");
    let files = capture.files();
    let artifacts = ARTIFACTS.into_iter().map(|path| {
        let bytes = files.get(path).ok_or("wet reference artifact absent from the original capture")?;
        Ok(ReferenceInputArtifact { path: path.into(), sha256: Sha256::digest(bytes).to_vec().into() })
    }).collect::<TestResult<Vec<_>>>()?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize()?;
    capture.verify_checkout(&root)?;
    let model = &result.prepared.model.model.compiled().model;
    let instance = result.prepared.model.model.solved().instance();
    let fixture = model.fixtures.get(&instance).ok_or("original wet fixture absent")?;
    assert_eq!(fixture.declaration, selected);
    let original = &model.instances[&instance].path;
    let mut physical_owners = Vec::new();
    let mut physical = |target: SemanticId| -> TestResult<PhysicalQuantityDescription> {
        let context = package.physical_context();
        let quantity = model.symbols[&target].ty.quantity_scheme()
            .ok_or("wet coordinate has no original physical contract")?
            .resolve(context.quantities(), &BTreeMap::new())?;
        let PhysicalInspectionDocument::Page { mut quantities, owner, .. } =
            context.describe(&PhysicalInspectionRequest {
                query: PhysicalInspectionQuery::Quantity { id: quantity, unit: None },
                offset: 0,
                limit: 1,
            })?
        else { return Err("wet reference physical quantity unavailable".into()); };
        assert_eq!(quantities.len(), 1);
        physical_owners.push(owner.ok_or("physical quantity lost its allocation owner")?);
        Ok(quantities.remove(0))
    };
    let specification = [
        ("inlet_temperature", "root.unit.inlet.T"),
        ("inlet_pressure", "root.unit.inlet.pressure"),
        ("inlet_flow", "root.unit.inlet.flow"),
        ("water_fraction", "root.unit.inlet.z[chem.water]"),
        ("pressure_change", "root.unit.delta_pressure"),
        ("efficiency", "root.unit.efficiency"),
    ].into_iter().map(|(key, path)| {
        let input = fixture.specifications.get(path).ok_or_else(|| format!("original wet input {path} absent"))?;
        assert_eq!(input.fixed, Some(true), "original wet input {path}");
        Ok(ReferenceFixedInput {
            key: key.into(),
            value: input.value.ok_or("original wet input magnitude absent")?,
            physical: physical(input.target)?,
        })
    }).collect::<TestResult<Vec<_>>>()?;
    let crate::math::solves::Outcome::Native(native) = &result.outcome else {
        return Err("wet reference requires the actual native outcome".into());
    };
    let candidate = native.candidate.as_ref().ok_or("original wet candidate absent")?;
    let mut observables = Vec::new();
    let mut values = Vec::new();
    let mut allowances = Vec::new();
    for (key, suffix, absolute, relative) in [
        ("outlet_temperature", "root.unit.outlet.T", 0.01, 0.),
        ("isentropic_temperature", "root.unit.reference.T", 0.01, 0.),
        ("outlet_pressure", "root.unit.outlet.pressure", 1., 0.),
        ("shaft_work", "root.unit.work", 0., 0.01),
        ("outlet_quality", "root.unit.outlet.quality", 1e-4, 0.),
        ("isentropic_quality", "root.unit.reference.quality", 1e-4, 0.),
    ] {
        let path = format!("{original}.{suffix}");
        let mut symbols = model.symbols.values().filter(|symbol| symbol.lineage.path == path);
        let target = symbols.next().ok_or_else(|| format!("original wet output {path} absent"))?.id;
        assert!(symbols.next().is_none(), "original wet output {path} ambiguous");
        let value = match native.variables.iter().position(|member| *member == target) {
            Some(column) => candidate.primal[column],
            None => *result.values.scalars.get(&target).ok_or("original wet output value absent")?,
        };
        let physical = physical(target)?;
        values.push(ReferenceObservable {
            id: target,
            quantity: physical.quantity.id.as_id(),
            unit: physical.representation_unit.id.as_id(),
            basis: physical.quantity.key.basis.map(|id| id.as_id()),
            datum: physical.quantity.key.reference_state.map(|id| id.as_id()),
            value,
        });
        allowances.push(ComparisonAllowance { output: target, absolute, relative });
        observables.push(ReferenceObservableContract { output: target, key: key.into(), physical });
    }
    let artifact_bytes = serde_json::to_vec(&artifacts.iter().map(|row| (&row.path, row.sha256.as_slice())).collect::<Vec<_>>())?;
    let meaning = ReferenceMeaning {
        case: selected.as_id(),
        source: pse_ids::preimage::hash(&artifact_bytes),
        formulation: pse_ids::preimage::hash(&serde_json::to_vec(&(1_u32, CASE, &artifact_bytes))?),
        physical: package.physical_context().identity(),
        configuration: pse_ids::preimage::hash(&serde_json::to_vec(&specification.iter().map(|row| (&row.key, row.value.to_bits(), row.physical.quantity.id.as_id())).collect::<Vec<_>>())?),
    };
    let candidate = ReferenceEnvelope {
        version: 1,
        meaning: meaning.clone(),
        required: values.iter().map(|value| value.id).collect(),
        outcome: ReferenceOutcome::Completed { outputs: values, blocks: Vec::new() },
    };
    let contract = ReferenceCaseContract {
        version: Default::default(), case_name: CASE.into(), meaning, observables, specification, artifacts,
    };
    let admission = ScalarAdmission::new(&candidate, LIMIT)?;
    let math = services.preparation().math();
    let response_bytes = admission.custody_bound(reference_process::DIAGNOSTIC_BYTES)?
        .checked_add(admission.native_envelope_bound()?)
        .ok_or("wet Water reference response custody extent overflow")?;
    let _response = math.reserve("wet-water-reference:response", response_bytes)?;
    let encoding = math.reserve("wet-water-reference:encoding", reference_exchange::case_contract_encoding_bound(&contract, LIMIT)?)?;
    let scratch = tempfile::tempdir()?;
    let descriptor = scratch.path().join("descriptor.bin");
    let request = scratch.path().join("contract.pb");
    let response = scratch.path().join("reference.pb");
    std::fs::write(&descriptor, reference_exchange::generated::DESCRIPTORS)?;
    std::fs::write(&request, reference_exchange::encode_case_contract(contract, LIMIT)?)?;
    drop((encoding, physical_owners));
    let python = reference_environment::interpreter("PSE_ORDINARY_REFERENCE_PYTHON");
    let interpreter_digest = reference_environment::image_digest(&python)?;
    let solver = reference_environment::prepared("IPOPT_DIR", "ipopt")?;
    let mut command = Command::new(&python);
    command.arg("-I").arg(root.join("tests/reference/wet_water_reference_driver.py"))
        .arg(descriptor).arg(request).arg(&response).current_dir(scratch.path())
        .env_remove("PYTHONPATH").env_remove("PYTHONHOME");
    let execution = reference_process::execute(&mut command, scratch, reference_process::DIAGNOSTIC_BYTES);
    if let Some((phase, cause)) = &execution.refusal {
        return Err(execution.failure(phase, cause));
    }
    let mut evidence = execution.diagnostics.lines().filter_map(|line| line.strip_prefix("wet_water_reference_environment "));
    let environment: BTreeMap<String, String> = serde_json::from_str(evidence.next().ok_or_else(|| execution.failure("environment refused", "missing wet reference evidence"))?)?;
    if evidence.next().is_some() {
        return Err(execution.failure("environment refused", "duplicate wet reference evidence"));
    }
    reference_environment::verify(&environment, &python, &interpreter_digest, &reference_environment::pins(&root)?)?;
    reference_environment::verify_stage(&environment, "steam.wet_turbine.initialize", &solver, 2..=2)?;
    reference_environment::verify_stage(&environment, "steam.wet_turbine.solve", &solver, 1..=1)?;
    verify_property_images(&python, &environment)?;
    capture.verify_checkout(&root)?;
    let bytes = reference_process::read_response(&response, LIMIT).map_err(|error| execution.failure("response refused", error))?;
    let independent = admission.decode(&bytes).map_err(|error| execution.failure("response refused", error))?;
    compare(&candidate, independent.known(), &allowances).map_err(|error| {
        execution.failure("comparison refused", format!("{error}; native={:?}; reference={:?}", candidate.outcome, independent.known().outcome))
    })?;
    let mut dry_candidate = candidate.clone();
    if let ReferenceOutcome::Completed { outputs, .. } = &mut dry_candidate.outcome {
        outputs[4].value = 1.;
    }
    assert!(compare(&dry_candidate, independent.known(), &allowances).is_err());
    Ok(())
}
