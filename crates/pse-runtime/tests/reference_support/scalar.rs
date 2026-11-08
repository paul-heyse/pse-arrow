// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact case and result acquisition for the dimensionless independent references.

use crate::{reference_environment, reference_process, reference_support::{Result, root}};
use pse_ids::{ContentHash, SemanticId};
use pse_model::reference_exchange::{ReferenceEnvelope, ReferenceMeaning, ReferenceObservable, ReferenceOutcome};
use pse_runtime::{
    reference_exchange::{self, ReferenceCaseContract, ReferenceFixedInput, ReferenceInputArtifact, ReferenceObservableContract},
    workflow::{PhysicalContext, PhysicalInspectionDocument, PhysicalInspectionQuery, PhysicalInspectionRequest},
};
use sha2::{Digest as _, Sha256};
use std::{collections::BTreeMap, ops::RangeInclusive, process::Command, sync::Arc};

pub(super) struct Case<'a> {
    pub(super) name: &'a str,
    pub(super) identity: SemanticId,
    pub(super) source: &'a str,
    pub(super) configuration: ContentHash,
    pub(super) inputs: Vec<(String, f64)>,
    pub(super) outputs: Vec<(String, SemanticId)>,
}

pub(super) struct Inputs {
    pub(super) contract: ReferenceCaseContract,
    _owner: Arc<datafusion::execution::memory_pool::MemoryReservation>,
}

pub(super) fn bind(
    case: Case<'_>,
    context: &PhysicalContext,
    originals: BTreeMap<String, Vec<u8>>,
) -> Result<Inputs> {
    let PhysicalInspectionDocument::Page { mut quantities, owner, .. } = context.describe(&PhysicalInspectionRequest {
        query: PhysicalInspectionQuery::QuantityNamed { name: "Scalar".into() },
        offset: 0,
        limit: 1,
    })? else {
        return Err("scalar reference physical description is unavailable".into());
    };
    if quantities.len() != 1 {
        return Err("scalar reference physical description is absent or ambiguous".into());
    }
    let physical = quantities.remove(0);
    let artifacts = originals.into_iter().map(|(path, content)| {
        ReferenceInputArtifact { path, sha256: Sha256::digest(content).to_vec().into() }
    }).collect::<Vec<_>>();
    let source = pse_ids::preimage::hash(&serde_json::to_vec(&(
        case.source,
        artifacts.iter().map(|artifact| (&artifact.path, artifact.sha256.as_slice())).collect::<Vec<_>>(),
    ))?);
    let specification = case.inputs.into_iter().map(|(key, value)| ReferenceFixedInput {
        key, value, physical: physical.clone(),
    }).collect();
    let observables = case.outputs.into_iter().map(|(key, output)| ReferenceObservableContract {
        output,
        key,
        physical: physical.clone(),
    }).collect();
    Ok(Inputs {
        contract: ReferenceCaseContract {
            version: Default::default(),
            case_name: case.name.into(),
            meaning: ReferenceMeaning {
                case: case.identity,
                source,
                formulation: pse_ids::preimage::hash(case.source.as_bytes()),
                physical: context.identity(),
                configuration: case.configuration,
            },
            observables,
            specification,
            artifacts,
        },
        _owner: owner.ok_or("scalar reference description lost its allocation owner")?,
    })
}

pub(super) struct Oracle<'a> {
    pub(super) role: &'a str,
    pub(super) driver: &'a str,
    pub(super) diagnostic_prefix: &'a str,
    pub(super) provider: &'a str,
    pub(super) solver: &'a str,
    pub(super) stages: &'a [(&'a str, RangeInclusive<usize>)],
}

#[allow(dead_code, reason = "degeneracy reuses case acquisition with its own typed result")]
pub(super) fn exchange(
    inputs: Inputs,
    oracle: Oracle<'_>,
    pool: Arc<dyn pse_columnar::MemoryPool>,
) -> Result<BTreeMap<String, f64>> {
    const LIMIT: usize = 1 << 20;
    let contract = &inputs.contract;
    let keys = contract.observables.iter().map(|coordinate| coordinate.key.clone()).collect::<Vec<_>>();
    let outputs = contract.observables.iter().map(|coordinate| {
        let physical = &coordinate.physical;
        ReferenceObservable {
            id: coordinate.output,
            quantity: physical.quantity.id.as_id(),
            unit: physical.representation_unit.id.as_id(),
            basis: physical.quantity.key.basis.map(|id| id.as_id()),
            datum: physical.quantity.key.reference_state.map(|id| id.as_id()),
            value: 0.0,
        }
    }).collect::<Vec<_>>();
    let expected = ReferenceEnvelope {
        version: 1,
        meaning: contract.meaning.clone(),
        required: outputs.iter().map(|output| output.id).collect(),
        outcome: ReferenceOutcome::Completed { outputs, blocks: Vec::new() },
    };
    let admission = reference_exchange::ScalarAdmission::new(&expected, LIMIT)?;
    let reserve = pse_columnar::MemoryConsumer::new("test:scalar-reference").register(&pool);
    reserve.try_grow(admission.custody_bound(reference_process::DIAGNOSTIC_BYTES)?)?;
    let (bytes, execution) = perform(inputs, oracle, LIMIT)?;
    let decoded = admission.decode(&bytes)
        .map_err(|error| execution.failure("response refused", error))?;
    let ReferenceOutcome::Completed { outputs, .. } = &decoded.known().outcome else {
        return Err(execution.failure("response refused", "scalar reference did not complete"));
    };
    let mut values = BTreeMap::new();
    for (key, output) in keys.into_iter().zip(outputs) {
        if !output.value.is_finite() || values.insert(key, output.value).is_some() {
            return Err(execution.failure("response refused", "scalar reference value is nonfinite or repeated"));
        }
    }
    Ok(values)
}


pub(super) fn perform(
    inputs: Inputs,
    oracle: Oracle<'_>,
    maximum_response: usize,
) -> Result<(Vec<u8>, reference_process::Execution)> {
    let python = reference_environment::interpreter(oracle.role);
    let interpreter_digest = reference_environment::image_digest(&python)?;
    let solver = reference_environment::prepared(oracle.provider, oracle.solver)?;
    let scratch = tempfile::tempdir()?;
    let descriptors = scratch.path().join("descriptor.bin");
    let input = scratch.path().join("contract.pb");
    let output = scratch.path().join("reference.pb");
    std::fs::write(&descriptors, reference_exchange::generated::DESCRIPTORS)?;
    let Inputs { contract, _owner } = inputs;
    std::fs::write(&input, reference_exchange::encode_case_contract(contract, 1 << 20)?)?;
    let mut command = Command::new(&python);
    command
        .arg("-I")
        .arg(root().join("tests/reference").join(oracle.driver))
        .arg(&descriptors)
        .arg(&input)
        .arg(&output)
        .current_dir(scratch.path())
        .env_remove("PYTHONPATH")
        .env_remove("PYTHONHOME");
    let execution = reference_process::execute(&mut command, scratch, reference_process::DIAGNOSTIC_BYTES);
    if let Some((phase, cause)) = &execution.refusal {
        return Err(execution.failure(phase, cause));
    }
    let mut evidence = execution.diagnostics.lines().filter_map(|line| line.strip_prefix(oracle.diagnostic_prefix));
    let environment: BTreeMap<String, String> = serde_json::from_str(
        evidence.next().ok_or_else(|| execution.failure("environment refused", "missing solver evidence"))?,
    )?;
    if evidence.next().is_some() {
        return Err(execution.failure("environment refused", "duplicate solver evidence"));
    }
    reference_environment::verify(&environment, &python, &interpreter_digest, &reference_environment::pins(&root())?)
        .map_err(|error| execution.failure("environment refused", error))?;
    for (stage, solves) in oracle.stages {
        reference_environment::verify_stage(&environment, stage, &solver, solves.clone())
            .map_err(|error| execution.failure("solver evidence refused", error))?;
    }
    let bytes = reference_process::read_response(&output, maximum_response)
        .map_err(|error| execution.failure("response refused", error))?;
    Ok((bytes, execution))
}
