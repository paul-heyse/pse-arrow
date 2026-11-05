// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Shared structure for independent comparisons; expected answers stay external.
use pse_contract::boundary_projection::{Direction, LocalRule, ProjectionError, ProjectionGraph};
use pse_ids::{ContentHash, SemanticId};

pse_contract::projection_struct! {
    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    /// Complete scientific interpretation of the independently constructed case.
    pub struct ReferenceMeaning {
        projection: { name: "ReferenceMeaning", version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
        /// Authored case identity, never a display label or vector position.
        pub case: SemanticId => { tag: 1, direction: Direction::Both, rules: [] };
        /// Source closure, including scientific parameter/data documents.
        pub source: ContentHash => { tag: 2, direction: Direction::Both, rules: [] };
        /// Scientific formulation interpretation, separate from wire version.
        pub formulation: ContentHash => { tag: 3, direction: Direction::Both, rules: [] };
        /// Physical definitions, units and reference anchors.
        pub physical: ContentHash => { tag: 4, direction: Direction::Both, rules: [] };
        /// Independently agreed case input/configuration closure.
        pub configuration: ContentHash => { tag: 5, direction: Direction::Both, rules: [] };
    }
}
pse_contract::projection_struct! {
    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    /// Original scalar observation and its physical correspondence.
    pub struct ReferenceObservable {
        projection: { name: "ReferenceObservable", version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
        /// Mapped output identity; order is meaningful and checked.
        pub id: SemanticId => { tag: 1, direction: Direction::Both, rules: [] };
        /// Original physical type.
        pub quantity: SemanticId => { tag: 2, direction: Direction::Both, rules: [] };
        /// Unit of the unmodified observed number.
        pub unit: SemanticId => { tag: 3, direction: Direction::Both, rules: [] };
        /// Amount/composition basis.
        pub basis: Option<SemanticId> => { tag: 4, direction: Direction::Both, rules: [] };
        /// Declared datum; absence remains distinct from an explicit reference state.
        pub datum: Option<SemanticId> => { tag: 5, direction: Direction::Both, rules: [] };
        /// IEEE-754 bits cross binary wire without decimal normalization.
        pub value: f64 => { tag: 6, direction: Direction::Both, rules: [] };
    }
}
pse_contract::projection_struct! {
    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    /// Retained columnar block; acquired data needs an independent block comparator.
    pub struct ReferenceBlock {
        projection: { name: "ReferenceBlock", version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
        /// Existing Arrow/Parquet block content identity; no copied scientific array.
        pub content: ContentHash => { tag: 1, direction: Direction::Both, rules: [] };
        /// Exact columnar schema interpretation.
        pub schema: ContentHash => { tag: 2, direction: Direction::Both, rules: [] };
        /// Original block population.
        pub rows: u64 => { tag: 3, direction: Direction::Both, rules: [] };
    }
}
pse_contract::projection_enum! {
    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    /// Different completion states retain their distinct evidence meanings.
    pub enum ReferenceOutcome {
        projection: { name: "ReferenceOutcome", version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
        /// All requested original products completed.
        Completed {
            /// Complete ordered scalar observations.
            outputs: Vec<ReferenceObservable> => { tag: 1, direction: Direction::Both, rules: [LocalRule::Length { min: 1, max: 65536 }] };
            /// Existing retained scientific blocks.
            blocks: Vec<ReferenceBlock> => { tag: 2, direction: Direction::Both, rules: [] };
        } => 1;
        /// Actual attributed failure at the intended scientific stage.
        Failed {
            /// Original typed failure code.
            code: String => { tag: 1, direction: Direction::Both, rules: [LocalRule::Length { min: 1, max: 256 }] };
            /// Stage reached before failure.
            stage: String => { tag: 2, direction: Direction::Both, rules: [LocalRule::Length { min: 1, max: 256 }] };
        } => 2;
        /// An unavailable prerequisite cannot supply passing evidence.
        Unavailable(String) => 3;
        /// Selected work has no execution evidence.
        NotRun(String) => 4;
    }
}
pse_contract::projection_struct! {
    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    /// Shared structure independent of numerical answer construction.
    pub struct ReferenceEnvelope {
        projection: { name: "ReferenceEnvelope", version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
        /// Explicit envelope major, distinct from scientific interpretation.
        pub version: u32 => { tag: 1, direction: Direction::Both, rules: [LocalRule::UnsignedRange { min: 1, max: 1 }] };
        /// Exact shared case interpretation.
        pub meaning: ReferenceMeaning => { tag: 2, direction: Direction::Both, rules: [LocalRule::Required] };
        /// Entire ordered required output population, including on failed/unrun cases.
        pub required: Vec<SemanticId> => { tag: 3, direction: Direction::Both, rules: [LocalRule::Length { min: 1, max: 65536 }] };
        /// Attributed completed/failed/unavailable/not-run state.
        pub outcome: ReferenceOutcome => { tag: 4, direction: Direction::Both, rules: [LocalRule::Required] };
    }
}

/// Compiled structural schema, with no numerical reference implementation.
pub fn reference_projection() -> Result<ProjectionGraph, ProjectionError> {
    ProjectionGraph::of::<ReferenceEnvelope>()
}

/// One independently chosen observable's dimensional allowances.
#[derive(Clone, Copy, Debug)]
pub struct ComparisonAllowance {
    /// Required output identity.
    pub output: SemanticId,
    /// Absolute allowance in the observable's declared unit.
    pub absolute: f64,
    /// Relative allowance applied to the independent reference magnitude.
    pub relative: f64,
}

/// Semantic mismatch is separate from protobuf structural compatibility.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("scientific reference correspondence: {0}")]
pub struct CorrespondenceError(pub String);
pse_diagnostics::impl_diagnostic! {
    CorrespondenceError,
    code(_this) { Some(pse_diagnostics::DiagnosticCode::ValidationInvariant) },
    forward(_this) { None }, help(_this) { None }, related(_this) { None }, source(_this) { None }
}

/// Compare against independently produced expected scalar results.
/// Columnar comparisons remain with the independently acquired-data comparator.
pub fn compare(
    candidate: &ReferenceEnvelope,
    reference: &ReferenceEnvelope,
    allowances: &[ComparisonAllowance],
) -> Result<(), CorrespondenceError> {
    let refuse = |reason: &str| CorrespondenceError(reason.into());
    if candidate.version != 1 || reference.version != 1 || candidate.meaning != reference.meaning {
        return Err(refuse(
            "case/source/formulation/physical/configuration interpretation differs",
        ));
    }
    if candidate.required != reference.required
        || reference.required.is_empty()
        || reference
            .required
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != reference.required.len()
    {
        return Err(refuse(
            "required ordered output population differs or repeats",
        ));
    }
    match (&candidate.outcome, &reference.outcome) {
        (
            ReferenceOutcome::Completed {
                outputs: actual,
                blocks: actual_blocks,
            },
            ReferenceOutcome::Completed {
                outputs: expected,
                blocks: expected_blocks,
            },
        ) => {
            if actual.len() != reference.required.len()
                || expected.len() != reference.required.len()
                || allowances.len() != reference.required.len()
            {
                return Err(refuse(
                    "completion is missing a required output or allowance",
                ));
            }
            if !actual_blocks.is_empty() || !expected_blocks.is_empty() {
                return Err(refuse(
                    "columnar blocks require their independent acquired-data comparator; metadata is not numerical evidence",
                ));
            }
            for (((actual, expected), required), allowance) in actual
                .iter()
                .zip(expected)
                .zip(&reference.required)
                .zip(allowances)
            {
                if actual.id != *required
                    || expected.id != *required
                    || allowance.output != *required
                    || (actual.quantity, actual.unit, actual.basis, actual.datum)
                        != (
                            expected.quantity,
                            expected.unit,
                            expected.basis,
                            expected.datum,
                        )
                {
                    return Err(refuse(
                        "ordered output identity, quantity/unit/basis/datum correspondence differs",
                    ));
                }
                if !allowance.absolute.is_finite()
                    || allowance.absolute < 0.
                    || !allowance.relative.is_finite()
                    || allowance.relative < 0.
                    || !actual.value.is_finite()
                    || !expected.value.is_finite()
                {
                    return Err(refuse("nonfinite value or invalid independent allowance"));
                }
                let difference = (actual.value - expected.value).abs();
                if difference > allowance.absolute
                    && difference / expected.value.abs() > allowance.relative
                {
                    return Err(refuse(
                        "observable exceeds its independent absolute and relative allowances",
                    ));
                }
            }
            Ok(())
        }
        (
            ReferenceOutcome::Failed { code: a, stage: sa },
            ReferenceOutcome::Failed { code: b, stage: sb },
        ) if a == b && sa == sb => Ok(()),
        _ => Err(refuse(
            "completion/failure stage differs, or selected work is unavailable/not run",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn id(byte: u8) -> SemanticId {
        SemanticId::from_bytes([byte; 16])
    }
    fn hash(byte: u8) -> ContentHash {
        ContentHash::from_bytes([byte; 32])
    }
    fn envelope() -> ReferenceEnvelope {
        ReferenceEnvelope {
            version: 1,
            meaning: ReferenceMeaning {
                case: id(1),
                source: hash(2),
                formulation: hash(3),
                physical: hash(4),
                configuration: hash(5),
            },
            required: vec![id(6), id(7)],
            outcome: ReferenceOutcome::Completed {
                outputs: [6, 7]
                    .map(|key| ReferenceObservable {
                        id: id(key),
                        quantity: id(8),
                        unit: id(9),
                        basis: Some(id(10)),
                        datum: Some(id(11)),
                        value: f64::from(key),
                    })
                    .to_vec(),
                blocks: vec![],
            },
        }
    }
    fn allowances() -> [ComparisonAllowance; 2] {
        [6, 7].map(|key| ComparisonAllowance {
            output: id(key),
            absolute: 1e-9,
            relative: 1e-8,
        })
    }
    #[test]
    fn independent_correspondence_refuses_wrong_missing_reordered_and_changed_meaning() {
        let reference = envelope();
        assert!(compare(&envelope(), &reference, &allowances()).is_ok());
        for mutation in 0..7 {
            let mut candidate = envelope();
            match mutation {
                0 => {
                    if let ReferenceOutcome::Completed { outputs, .. } = &mut candidate.outcome {
                        outputs[0].value += 1.;
                    }
                }
                1 => {
                    if let ReferenceOutcome::Completed { outputs, .. } = &mut candidate.outcome {
                        outputs.pop();
                    }
                }
                2 => {
                    if let ReferenceOutcome::Completed { outputs, .. } = &mut candidate.outcome {
                        outputs.swap(0, 1);
                    }
                }
                3 => {
                    if let ReferenceOutcome::Completed { outputs, .. } = &mut candidate.outcome {
                        outputs[0].datum = Some(id(99));
                    }
                }
                4 => candidate.meaning.formulation = hash(99),
                5 => candidate.meaning.source = hash(99),
                _ => candidate.outcome = ReferenceOutcome::NotRun("missing prerequisite".into()),
            }
            assert!(
                compare(&candidate, &reference, &allowances()).is_err(),
                "mutation {mutation}"
            );
        }
        let mut unavailable = envelope();
        unavailable.outcome = ReferenceOutcome::Unavailable("solver missing".into());
        assert!(compare(&unavailable, &unavailable, &allowances()).is_err());
    }
    #[test]
    fn dimensional_allowances_handle_zero_and_refuse_nonfinite_duplicate_and_unacquired_blocks() {
        let mut reference = envelope();
        let ReferenceOutcome::Completed { outputs, .. } = &mut reference.outcome else {
            panic!("reference fixture must have completed outputs")
        };
        outputs[0].value = 0.;
        let mut candidate = reference.clone();
        let ReferenceOutcome::Completed { outputs, .. } = &mut candidate.outcome else {
            panic!("candidate cloned from reference must have completed outputs")
        };
        outputs[0].value = 0.5e-9;
        assert!(compare(&candidate, &reference, &allowances()).is_ok());
        let ReferenceOutcome::Completed { outputs, .. } = &mut candidate.outcome else {
            panic!("candidate must retain completed outputs after value mutation")
        };
        outputs[0].value = f64::NAN;
        assert!(compare(&candidate, &reference, &allowances()).is_err());
        let mut block = envelope();
        let ReferenceOutcome::Completed { blocks, .. } = &mut block.outcome else {
            panic!("block fixture must have a completed outcome")
        };
        blocks.push(ReferenceBlock {
            content: hash(1),
            schema: hash(2),
            rows: 1,
        });
        assert!(compare(&block, &block, &allowances()).is_err());
        reference.required[1] = reference.required[0];
        assert!(compare(&reference, &reference, &allowances()).is_err());
    }
}
