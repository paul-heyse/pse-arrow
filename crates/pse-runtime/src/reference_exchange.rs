// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded generated scientific interchange. This codec never constructs reference answers.
use crate::projection_diagnostics::ProjectionDiagnosticError;
use pse_contract::boundary_projection::{AdapterError, AdapterFailure, ValidateProjection as _};
use pse_contract_proto::support::*;
use pse_model::reference_exchange as native;

/// Source-generated messages and validators, with the complete imported descriptor world.
pub mod generated;
pub use pse_model::reference_exchange::{
    ReferenceCaseContract, ReferenceFixedInput, ReferenceInputArtifact, ReferenceObservableContract,
};
use generated::pb::pse::reference::v1 as pb;
const MAXIMUM_BYTES: usize = 16 << 20;
const UNKNOWN_FIELDS: usize = 256;
const RECURSION: u32 = 32;
#[path = "generated/reference-exchange/native-adapters.rs"]
#[rustfmt::skip]
mod native_projection;

/// Structural exchange refusal; physical comparison remains a separate operation.
#[derive(Debug, thiserror::Error)]
pub enum ExchangeError {
    /// A binary extent, decoding or encoding operation refused.
    #[error("scientific exchange: {0}")]
    Invalid(String),
    /// Original typed local-rule or mechanical-adoption refusal.
    #[error(transparent)]
    Projection(#[from] ProjectionDiagnosticError),
}
impl From<BoundaryProjectionError> for ExchangeError {
    fn from(error: BoundaryProjectionError) -> Self {
        Self::Projection(error.into())
    }
}
pse_diagnostics::impl_diagnostic! {
    ExchangeError,
    code(this) { match this { Self::Invalid(_) => Some(pse_diagnostics::DiagnosticCode::ValidationInvariant), Self::Projection(_) => None } },
    forward(this) { match this { Self::Projection(error) => Some(error), Self::Invalid(_) => None } }, help(_this) { None }, related(_this) { None }, source(_this) { None }
}
fn invalid(error: impl std::fmt::Display) -> ExchangeError {
    ExchangeError::Invalid(error.to_string())
}

/// A checked known interpretation plus its complete original message, including unknown fields.
#[derive(Debug)]
pub struct DecodedReference {
    native: native::ReferenceEnvelope,
    original: pb::ReferenceEnvelope,
}
/// Original bounded case message and its checked known interpretation.
/// Adoption of this correspondence never admits a scientific source or proof.
#[derive(Debug)]
pub struct DecodedCaseContract {
    native: ReferenceCaseContract,
    original: pb::ReferenceCaseContract,
}

/// Independent certificates and their retained original message.
#[derive(Debug)]
pub struct DecodedDegeneracyResult {
    native: native::ReferenceDegeneracyResult,
    original: pb::ReferenceDegeneracyResult,
}

impl DecodedDegeneracyResult {
    /// Known row-support observations. Their scientific interpretation needs the original case.
    pub fn known(&self) -> &native::ReferenceDegeneracyResult {
        &self.native
    }

    /// Forward the original message including bounded unknown fields.
    /// # Errors
    /// Refuses if the original message exceeds the selected finite extent.
    pub fn forward(&self, maximum_bytes: usize) -> Result<Vec<u8>, ExchangeError> {
        encode_bounded_message(&self.original, maximum_bytes)
    }
}

/// Admission of one original scalar population, independently of its values.
///
/// The borrowed native envelope supplies only interpretation, ordered physical
/// metadata and completion kind. Its scalar values are never inspected. Unknown
/// binary fields retain the generic codec's byte/count/recursion allowances.
#[derive(Debug)]
pub struct ScalarAdmission<'a> {
    expected: &'a native::ReferenceEnvelope,
    maximum_bytes: usize,
    element_bytes: usize,
}

fn extent_add(a: usize, b: usize) -> Result<usize, ExchangeError> {
    a.checked_add(b)
        .ok_or_else(|| invalid("scientific custody extent overflow"))
}
fn extent_mul(a: usize, b: usize) -> Result<usize, ExchangeError> {
    a.checked_mul(b)
        .ok_or_else(|| invalid("scientific custody extent overflow"))
}

// These walks describe allocation custody of the actual native/binding types,
// not another scientific schema. Exhaustive native destructuring makes an added
// field a compile error until its storage is included. Inline fields are counted
// again deliberately: that also covers temporary field tuples during adoption.
trait EncodingExtent {
    type Wire;
    fn native_extent(&self) -> Result<usize, ExchangeError>;
    fn wire_extent(&self) -> Result<usize, ExchangeError>;
    fn path_extent(&self, _prefix: usize) -> Result<usize, ExchangeError> {
        Ok(0)
    }
}
macro_rules! encoding_record {
    ($native:ty => $wire:ty { $($field:ident),* $(,)? }) => {
        impl EncodingExtent for $native {
            type Wire = $wire;
            fn native_extent(&self) -> Result<usize, ExchangeError> {
                let Self { $($field,)* } = self;
                let bytes = size_of::<Self>();
                $(let bytes = extent_add(bytes, $field.native_extent()?)?;)*
                Ok(bytes)
            }
            fn wire_extent(&self) -> Result<usize, ExchangeError> {
                let Self { $($field,)* } = self;
                let bytes = size_of::<Self::Wire>();
                $(let bytes = extent_add(bytes, $field.wire_extent()?)?;)*
                Ok(bytes)
            }
            fn path_extent(&self, prefix: usize) -> Result<usize, ExchangeError> {
                let Self { $($field,)* } = self;
                let bytes = 0;
                $(let child = extent_add(prefix, 1 + stringify!($field).len())?;
                  let bytes = extent_add(bytes, extent_add(extent_mul(3, child)?, $field.path_extent(child)?)?)?;)*
                Ok(bytes)
            }
        }
    };
}
macro_rules! encoding_scalar {
    ($($native:ty => $wire:ty),* $(,)?) => {$ (
        impl EncodingExtent for $native {
            type Wire = $wire;
            fn native_extent(&self) -> Result<usize, ExchangeError> { Ok(size_of::<Self>()) }
            fn wire_extent(&self) -> Result<usize, ExchangeError> { Ok(size_of::<Self::Wire>()) }
        }
    )*};
}
encoding_scalar! { bool => bool, f64 => u64, i16 => i32, u32 => u32 }
impl EncodingExtent for String {
    type Wire = String;
    fn native_extent(&self) -> Result<usize, ExchangeError> {
        extent_add(size_of::<Self>(), self.capacity())
    }
    fn wire_extent(&self) -> Result<usize, ExchangeError> {
        extent_add(size_of::<Self>(), self.capacity())
    }
}
impl<T: EncodingExtent> EncodingExtent for Vec<T> {
    type Wire = Vec<T::Wire>;
    fn native_extent(&self) -> Result<usize, ExchangeError> {
        self.iter().try_fold(
            extent_add(
                size_of::<Self>(),
                extent_mul(self.capacity(), size_of::<T>())?,
            )?,
            |bytes, value| extent_add(bytes, value.native_extent()?),
        )
    }
    fn wire_extent(&self) -> Result<usize, ExchangeError> {
        let elements = self
            .iter()
            .try_fold(0, |bytes, value| extent_add(bytes, value.wire_extent()?))?;
        extent_add(
            size_of::<Self::Wire>(),
            extent_add(
                extent_mul(3, elements)?,
                extent_mul(4, size_of::<T::Wire>())?,
            )?,
        )
    }
    fn path_extent(&self, prefix: usize) -> Result<usize, ExchangeError> {
        let child = extent_add(prefix, usize::BITS as usize + 2)?;
        self.iter().try_fold(0, |bytes, value| {
            extent_add(
                bytes,
                extent_add(extent_mul(3, child)?, value.path_extent(child)?)?,
            )
        })
    }
}
impl<T: EncodingExtent> EncodingExtent for Option<T> {
    type Wire = Option<T::Wire>;
    fn native_extent(&self) -> Result<usize, ExchangeError> {
        extent_add(
            size_of::<Self>(),
            self.as_ref().map_or(Ok(0), EncodingExtent::native_extent)?,
        )
    }
    fn wire_extent(&self) -> Result<usize, ExchangeError> {
        extent_add(
            size_of::<Self::Wire>(),
            self.as_ref().map_or(Ok(0), EncodingExtent::wire_extent)?,
        )
    }
    fn path_extent(&self, prefix: usize) -> Result<usize, ExchangeError> {
        self.as_ref().map_or(Ok(0), |value| {
            extent_add(extent_mul(3, prefix)?, value.path_extent(prefix)?)
        })
    }
}
macro_rules! encoding_identity {
    ($($native:ty => $wire:ty),* $(,)?) => {$ (
        impl EncodingExtent for $native {
            type Wire = $wire;
            fn native_extent(&self) -> Result<usize, ExchangeError> { Ok(size_of::<Self>()) }
            fn wire_extent(&self) -> Result<usize, ExchangeError> { extent_add(size_of::<Self::Wire>(), self.as_bytes().len()) }
            fn path_extent(&self, prefix: usize) -> Result<usize, ExchangeError> { extent_mul(3, extent_add(prefix, 3)?) }
        }
    )*};
}
encoding_identity! {
    pse_ids::SemanticId => Vec<u8>, pse_ids::ContentHash => Vec<u8>,
    pse_quantity::BasisId => pb::BasisId, pse_quantity::EntityKindId => pb::EntityKindId,
    pse_quantity::QuantityKindId => pb::QuantityKindId, pse_quantity::QuantityTypeId => pb::QuantityTypeId,
    pse_quantity::UnitId => pb::UnitId, pse_quantity::ReferenceStateId => pb::ReferenceStateId,
}
macro_rules! encoding_string_enum {
    ($($native:ty => $wire:ty),* $(,)?) => {$ (
        impl EncodingExtent for $native {
            type Wire = $wire;
            fn native_extent(&self) -> Result<usize, ExchangeError> { Ok(size_of::<Self>()) }
            fn wire_extent(&self) -> Result<usize, ExchangeError> { extent_add(size_of::<Self::Wire>(), self.as_str().len()) }
        }
    )*};
}
encoding_string_enum! {
    pse_quantity::BasisKind => pb::BasisKind, pse_quantity::CompositionBasis => pb::CompositionBasis,
    pse_quantity::RateBasis => pb::RateBasis, pse_quantity::ScaleKind => pb::ScaleKind,
    pse_quantity::ReferenceStateKind => pb::ReferenceStateKind,
    pse_quantity::QuantityAdditionKind => pb::QuantityAdditionKind,
    pse_quantity::QuantityKindCategory => pb::QuantityKindCategory,
}
impl EncodingExtent for pse_model::Bytes {
    type Wire = Vec<u8>;
    fn native_extent(&self) -> Result<usize, ExchangeError> {
        extent_add(size_of::<Self>(), pse_model::HeapUsage::heap_bytes(self))
    }
    fn wire_extent(&self) -> Result<usize, ExchangeError> {
        extent_add(
            size_of::<Self::Wire>(),
            pse_model::HeapUsage::heap_bytes(self),
        )
    }
}
impl EncodingExtent for pse_model::document::Version<1> {
    type Wire = pb::DocumentVersion1;
    fn native_extent(&self) -> Result<usize, ExchangeError> {
        Ok(size_of::<Self>())
    }
    fn wire_extent(&self) -> Result<usize, ExchangeError> {
        Ok(size_of::<Self::Wire>())
    }
    fn path_extent(&self, prefix: usize) -> Result<usize, ExchangeError> {
        extent_mul(3, extent_add(prefix, 6)?)
    }
}
impl EncodingExtent for pse_quantity::Ratio {
    type Wire = pb::PhysicalRatio;
    fn native_extent(&self) -> Result<usize, ExchangeError> {
        let _inline = *self;
        Ok(size_of::<Self>())
    }
    fn wire_extent(&self) -> Result<usize, ExchangeError> {
        Ok(size_of::<Self::Wire>())
    }
    fn path_extent(&self, prefix: usize) -> Result<usize, ExchangeError> {
        extent_mul(6, extent_add(prefix, 4)?)
    }
}
impl EncodingExtent for pse_quantity::DimensionVector {
    type Wire = pb::PhysicalDimensionVector;
    fn native_extent(&self) -> Result<usize, ExchangeError> {
        let _inline = *self;
        Ok(size_of::<Self>())
    }
    fn wire_extent(&self) -> Result<usize, ExchangeError> {
        let elements = self
            .exponents()
            .iter()
            .try_fold(0, |bytes, value| extent_add(bytes, value.wire_extent()?))?;
        extent_add(
            size_of::<Self::Wire>() + size_of::<pb::Array8PhysicalRatio>(),
            extent_add(
                extent_mul(3, elements)?,
                extent_mul(4, size_of::<pb::PhysicalRatio>())?,
            )?,
        )
    }
    fn path_extent(&self, prefix: usize) -> Result<usize, ExchangeError> {
        let exponents = extent_add(prefix, ".exponents".len())?;
        let values = extent_add(exponents, ".values".len())?;
        let index = extent_add(values, usize::BITS as usize + 2)?;
        self.exponents().iter().try_fold(
            extent_mul(3, extent_add(exponents, values)?)?,
            |bytes, value| {
                extent_add(
                    bytes,
                    extent_add(extent_mul(3, index)?, value.path_extent(index)?)?,
                )
            },
        )
    }
}
encoding_record! { pse_quantity::KindFactor => pb::KindFactor { kind, exponent } }
encoding_record! { pse_quantity::KindDefinition => pb::KindDefinition { monomial, canonical_unit, basis, reference_state, scale_kind, subject_kind } }
encoding_record! { pse_quantity::QuantityKind => pb::PhysicalQuantityKind { id, dimension, extensive, addition_kind, category, definition } }
encoding_record! { pse_quantity::QuantityTypeKey => pb::QuantityTypeKey { kind, basis, reference_state, scale_kind, shape, subject_kind } }
encoding_record! { pse_quantity::QuantityType => pb::QuantityType { id, name, key, canonical_unit, nominal_magnitude } }
encoding_record! { pse_quantity::UnitFactor => pb::UnitFactor { unit, exponent } }
encoding_record! { pse_quantity::Unit => pb::PhysicalUnit { id, symbol, dimension, scale_to_canonical, offset_to_canonical, is_affine, reference_state, definition } }
encoding_record! { pse_quantity::Basis => pb::PhysicalBasis { id, kind, composition_basis, rate_basis, reference_conditions } }
encoding_record! { pse_quantity::EntityKind => pb::PhysicalEntityKind { id, name } }
encoding_record! { pse_quantity::ReferenceCondition => pb::ReferenceCondition { value, quantity_type, unit } }
encoding_record! { pse_quantity::ReferenceState => pb::ReferenceState { id, name, kind, temperature, pressure, include_enthalpy_of_formation, subject } }
encoding_record! { crate::workflow::PhysicalQuantityDescription => pb::PhysicalQuantityDescription { quantity, quantity_kind, canonical_unit, representation_unit, scale_to_canonical, offset_to_canonical, basis, basis_reference_state, reference_state, shape, subject } }
encoding_record! { native::ReferenceMeaning => pb::ReferenceMeaning { case, source, formulation, physical, configuration } }
encoding_record! { ReferenceObservableContract => pb::ReferenceObservableContract { output, key, physical } }
encoding_record! { ReferenceFixedInput => pb::ReferenceFixedInput { key, value, physical } }
encoding_record! { ReferenceInputArtifact => pb::ReferenceInputArtifact { path, sha256 } }
encoding_record! { ReferenceCaseContract => pb::ReferenceCaseContract { version, case_name, meaning, observables, specification, artifacts } }

/// Actual native/binding field custody plus the bounded output buffer for encoding
/// this locally constructed case contract. Traversal borrows all native storage.
/// # Errors
/// Invalid byte extent or checked allocation arithmetic overflow.
pub fn case_contract_encoding_bound(
    value: &ReferenceCaseContract,
    maximum_bytes: usize,
) -> Result<usize, ExchangeError> {
    if maximum_bytes == 0 || maximum_bytes > MAXIMUM_BYTES {
        return Err(invalid(
            "select a scientific envelope allowance in 1..=16MiB",
        ));
    }
    let paths = extent_mul(2, value.path_extent("ReferenceCaseContract".len())?)?;
    extent_add(
        extent_add(
            extent_add(value.native_extent()?, value.wire_extent()?)?,
            paths,
        )?,
        maximum_bytes,
    )
}

/// Native top-level containers/keys before physical descriptions are constructed.
/// Physical pages keep their original allocation owners through subsequent encode.
/// `key_storage` includes the actual source mapping/key String capacities.
/// # Errors
/// Checked allocation arithmetic overflow.
pub fn case_contract_container_bound(
    observables: usize,
    inputs: usize,
    artifacts: &[ReferenceInputArtifact],
    key_storage: usize,
) -> Result<usize, ExchangeError> {
    let slots = extent_add(
        extent_mul(
            extent_add(observables, 4)?,
            size_of::<ReferenceObservableContract>(),
        )?,
        extent_add(
            extent_mul(extent_add(inputs, 4)?, size_of::<ReferenceFixedInput>())?,
            extent_mul(
                extent_add(artifacts.len(), 4)?,
                size_of::<ReferenceInputArtifact>(),
            )?,
        )?,
    )?;
    let payloads = artifacts.iter().try_fold(key_storage, |bytes, artifact| {
        extent_add(
            bytes,
            extent_add(artifact.path.len(), artifact.sha256.as_slice().len())?,
        )
    })?;
    let pages = extent_mul(
        3,
        extent_mul(
            extent_add(extent_add(observables, inputs)?, 4)?,
            size_of::<std::sync::Arc<pse_columnar::MemoryReservation>>(),
        )?,
    )?;
    extent_add(
        size_of::<ReferenceCaseContract>(),
        extent_add(
            pages,
            extent_add(extent_mul(3, slots)?, extent_mul(3, payloads)?)?,
        )?,
    )
}

impl<'a> ScalarAdmission<'a> {
    /// Borrow a source-owned completed or failed scalar shape with no block products.
    /// # Errors
    /// Invalid shared native rules, unsupported shape, empty population or byte extent.
    pub fn new(
        expected: &'a native::ReferenceEnvelope,
        maximum_bytes: usize,
    ) -> Result<Self, ExchangeError> {
        if maximum_bytes == 0 || maximum_bytes > MAXIMUM_BYTES {
            return Err(invalid(
                "select a scientific envelope allowance in 1..=16MiB",
            ));
        }
        expected
            .validate_projection()
            .map_err(BoundaryProjectionError::OutgoingNative)?;
        let outputs = match &expected.outcome {
            native::ReferenceOutcome::Completed { outputs, blocks }
                if blocks.is_empty() && outputs.len() == expected.required.len() =>
            {
                outputs.len()
            }
            native::ReferenceOutcome::Failed { .. } => 0,
            _ => {
                return Err(invalid(
                    "selected scalar shape requires complete scalar outputs or an attributed failure",
                ));
            }
        };
        // This is Buffa's actual repeated-element charge, not a heap bound.
        let element_bytes = extent_add(
            extent_mul(expected.required.len(), size_of::<Vec<u8>>())?,
            extent_mul(outputs, size_of::<pb::ReferenceObservable>())?,
        )?;
        Ok(Self {
            expected,
            maximum_bytes,
            element_bytes,
        })
    }

    /// Conservative custody of one native envelope of this original population.
    /// Includes old/new Vec storage during geometric growth and failed-state strings.
    /// # Errors
    /// Checked extent arithmetic overflow.
    pub fn native_envelope_bound(&self) -> Result<usize, ExchangeError> {
        let outputs = match &self.expected.outcome {
            native::ReferenceOutcome::Completed { outputs, .. } => outputs.len(),
            _ => 0,
        };
        let elements = extent_add(
            extent_mul(
                self.expected.required.len(),
                size_of::<pse_ids::SemanticId>(),
            )?,
            extent_mul(outputs, size_of::<native::ReferenceObservable>())?,
        )?;
        // Four slots cover Vec's minimum allocation; three footprints cover
        // old+new storage at a doubling reallocation. Failed strings match the
        // original before adoption, so their lengths are independently known.
        let strings = match &self.expected.outcome {
            native::ReferenceOutcome::Failed { code, stage } => {
                extent_add(code.len(), stage.len())?
            }
            _ => 0,
        };
        extent_add(
            size_of::<native::ReferenceEnvelope>(),
            extent_add(
                extent_mul(
                    3,
                    extent_add(
                        elements,
                        extent_mul(
                            4,
                            size_of::<native::ReferenceObservable>()
                                + size_of::<pse_ids::SemanticId>(),
                        )?,
                    )?,
                )?,
                strings,
            )?,
        )
    }

    /// Before-allocation custody for bounded input, owned decode, retained original,
    /// clone/adoption, one forwarded encoding and bounded diagnostic rendering.
    ///
    /// `diagnostic_bytes` includes any caller-added truncation marker. It prices
    /// raw Vec old/new storage plus UTF-8 replacement (at most three bytes per
    /// input byte) and String growth. Callers must stream diagnostic errors rather
    /// than cloning the rendered log into another owned formatted String.
    ///
    /// This bounds owned Rust payload/storage, not allocator bookkeeping or RSS.
    /// The selected metadata preflight below precedes generated validation, so
    /// private-child violations cannot allocate an error tree: every known local
    /// rule is established by equality to the owner-validated native shape.
    /// # Errors
    /// Checked extent arithmetic overflow.
    pub fn custody_bound(&self, diagnostic_bytes: usize) -> Result<usize, ExchangeError> {
        let repeated_slack = extent_mul(
            4,
            size_of::<Vec<u8>>()
                + size_of::<pb::ReferenceObservable>()
                + size_of::<pb::ReferenceBlock>(),
        )?;
        // The unknown count is GLOBAL across the decode tree. Each nonempty
        // group Vec can reserve a minimum of four UnknownField slots. Six slots
        // per charged field additionally cover old+new doubling storage; its
        // length-delimited contents are accounted separately by the B-byte term.
        let unknown = extent_mul(6 * UNKNOWN_FIELDS, size_of::<buffa::UnknownField>())?;
        let alternative = [
            size_of::<pb::reference_outcome::Alternative1>(),
            size_of::<pb::reference_outcome::Alternative2>(),
            size_of::<pb::reference_outcome::Alternative3>(),
            size_of::<pb::reference_outcome::Alternative4>(),
        ]
        .into_iter()
        .max()
        .unwrap_or(0);
        // MessageField is inline; the outcome's alternative is the only known
        // non-repeated box. Two alternatives cover replacement during a merge.
        let fixed = extent_add(
            size_of::<pb::ReferenceEnvelope>(),
            extent_mul(2, alternative)?,
        )?;
        let wire = extent_add(
            extent_add(
                extent_add(extent_mul(3, self.element_bytes)?, repeated_slack)?,
                unknown,
            )?,
            extent_add(fixed, self.maximum_bytes)?,
        )?;
        let input = extent_add(self.maximum_bytes, 1)?;
        // At lossy conversion the raw Vec retains at most twice Q. A replacement
        // emits at most three UTF-8 bytes per input byte; old+new String storage
        // during doubling costs at most three times that expanded payload.
        let diagnostics = extent_add(
            extent_mul(2, diagnostic_bytes)?,
            extent_mul(3, extent_mul(3, diagnostic_bytes)?)?,
        )?;
        extent_add(
            extent_add(extent_add(extent_mul(2, wire)?, input)?, self.maximum_bytes)?,
            extent_add(self.native_envelope_bound()?, diagnostics)?,
        )
    }

    fn preflight(&self, wire: &pb::ReferenceEnvelope) -> Result<(), ExchangeError> {
        let refuse = || {
            invalid(
                "reference scalar shape differs from the original interpretation or ordered physical population",
            )
        };
        let expected = self.expected;
        let meaning = wire.meaning.as_option().ok_or_else(refuse)?;
        if wire.version != expected.version
            || meaning.case.as_slice() != expected.meaning.case.as_bytes()
            || meaning.source.as_slice() != expected.meaning.source.as_bytes()
            || meaning.formulation.as_slice() != expected.meaning.formulation.as_bytes()
            || meaning.physical.as_slice() != expected.meaning.physical.as_bytes()
            || meaning.configuration.as_slice() != expected.meaning.configuration.as_bytes()
            || wire.required.len() != expected.required.len()
            || !wire
                .required
                .iter()
                .zip(&expected.required)
                .all(|(wire, native)| wire.as_slice() == native.as_bytes())
        {
            return Err(refuse());
        }
        let outcome = wire
            .outcome
            .as_option()
            .and_then(|outcome| outcome.kind.as_ref())
            .ok_or_else(refuse)?;
        match (&expected.outcome, outcome) {
            (
                native::ReferenceOutcome::Completed { outputs, .. },
                pb::reference_outcome::Kind::Completed(actual),
            ) => {
                if actual.outputs.len() != outputs.len() || !actual.blocks.is_empty() {
                    return Err(refuse());
                }
                for (actual, expected) in actual.outputs.iter().zip(outputs) {
                    let same_optional =
                        |actual: &Option<Vec<u8>>, expected: Option<pse_ids::SemanticId>| match (
                            actual, expected,
                        ) {
                            (None, None) => true,
                            (Some(actual), Some(expected)) => {
                                actual.as_slice() == expected.as_bytes()
                            }
                            _ => false,
                        };
                    if actual.id.as_slice() != expected.id.as_bytes()
                        || actual.quantity.as_slice() != expected.quantity.as_bytes()
                        || actual.unit.as_slice() != expected.unit.as_bytes()
                        || !same_optional(&actual.basis, expected.basis)
                        || !same_optional(&actual.datum, expected.datum)
                    {
                        return Err(refuse());
                    }
                }
            }
            (
                native::ReferenceOutcome::Failed { code, stage },
                pb::reference_outcome::Kind::Failed(actual),
            ) if actual.code == *code && actual.stage == *stage => {}
            _ => return Err(refuse()),
        }
        Ok(())
    }

    /// Decode only this original scalar shape, preserving every bounded unknown field.
    /// # Errors
    /// Byte/element/unknown/recursion limits, original correspondence or generated adoption.
    /// The caller reserves [`Self::custody_bound`] before reading or decoding bytes.
    pub fn decode(&self, bytes: &[u8]) -> Result<DecodedReference, ExchangeError> {
        let original: pb::ReferenceEnvelope =
            decode_options(self.maximum_bytes, self.element_bytes)?
                .decode_from_slice(bytes)
                .map_err(invalid)?;
        self.preflight(&original)?;
        let native = native_projection::decode_reference_envelope(original.clone())?;
        Ok(DecodedReference { native, original })
    }
}
impl DecodedCaseContract {
    /// Original known metadata and fixed input values.
    pub fn known(&self) -> &ReferenceCaseContract {
        &self.native
    }
    /// Forward the complete binary message, retaining future unknown fields.
    /// # Errors
    /// The whole message must fit the selected finite output allowance.
    pub fn forward(&self, maximum_bytes: usize) -> Result<Vec<u8>, ExchangeError> {
        encode_bounded_message(&self.original, maximum_bytes)
    }
}
impl DecodedReference {
    /// Known versioned scientific metadata. This is not contextual scientific admission.
    pub fn known(&self) -> &native::ReferenceEnvelope {
        &self.native
    }
    /// Forward the original complete binary message without dropping unknown binary fields.
    /// # Errors
    /// An extent outside the caller's finite allowance refuses before any output is returned.
    pub fn forward(&self, maximum_bytes: usize) -> Result<Vec<u8>, ExchangeError> {
        encode_message(&self.original, maximum_bytes)
    }
}

/// Encode a source-owned scientific envelope, preserving scalar bits and presence.
/// # Errors
/// Shared local rules, generated validation and output extent must all hold.
pub fn encode(
    value: native::ReferenceEnvelope,
    maximum_bytes: usize,
) -> Result<Vec<u8>, ExchangeError> {
    let wire = native_projection::encode_reference_envelope(value)?;
    encode_message(&wire, maximum_bytes)
}
/// Encode original inputs and their actual physical correspondence before the
/// independent calculation. Candidate values are carried by a separate envelope.
/// # Errors
/// Shared local rules and the finite output extent must hold in full.
pub fn encode_case_contract(
    value: ReferenceCaseContract,
    maximum_bytes: usize,
) -> Result<Vec<u8>, ExchangeError> {
    let wire = native_projection::encode_reference_case_contract(value)?;
    encode_bounded_message(&wire, maximum_bytes)
}

/// Encode typed independent degeneracy observations for exchange.
/// # Errors
/// Shared local rules, generated validation or finite extent refusal.
pub fn encode_degeneracy_result(
    value: native::ReferenceDegeneracyResult,
    maximum_bytes: usize,
) -> Result<Vec<u8>, ExchangeError> {
    let wire = native_projection::encode_reference_degeneracy_result(value)?;
    encode_bounded_message(&wire, maximum_bytes)
}
fn encode_message(
    value: &pb::ReferenceEnvelope,
    maximum_bytes: usize,
) -> Result<Vec<u8>, ExchangeError> {
    encode_bounded_message(value, maximum_bytes)
}
fn encode_bounded_message<M: buffa::Message>(
    value: &M,
    maximum_bytes: usize,
) -> Result<Vec<u8>, ExchangeError> {
    if maximum_bytes == 0
        || maximum_bytes > MAXIMUM_BYTES
        || value.try_encoded_len().map_err(invalid)? as usize > maximum_bytes
    {
        return Err(invalid(
            "scientific envelope exceeds the selected output extent",
        ));
    }
    value.try_encode_to_vec().map_err(invalid)
}

/// Decode with finite byte, element, recursion and unknown-field budgets.
/// # Errors
/// Malformed/oversized wire, unsupported version, local rules and absent alternatives refuse.
pub fn decode(bytes: &[u8], maximum_bytes: usize) -> Result<DecodedReference, ExchangeError> {
    let original: pb::ReferenceEnvelope = decode_bounded_message(bytes, maximum_bytes)?;
    let native = native_projection::decode_reference_envelope(original.clone())?;
    Ok(DecodedReference { native, original })
}
/// Decode the shared case correspondence with the same finite binary limits as results.
/// # Errors
/// Malformed wire, unsupported interpretation, local rules and byte extents refuse.
pub fn decode_case_contract(
    bytes: &[u8],
    maximum_bytes: usize,
) -> Result<DecodedCaseContract, ExchangeError> {
    let original: pb::ReferenceCaseContract = decode_bounded_message(bytes, maximum_bytes)?;
    let native = native_projection::decode_reference_case_contract(original.clone())?;
    Ok(DecodedCaseContract { native, original })
}

/// Decode independent certificates under bounded byte, element and unknown-field allowances.
/// # Errors
/// Malformed wire, unsupported alternatives, local rules and extent refusal.
pub fn decode_degeneracy_result(
    bytes: &[u8],
    maximum_bytes: usize,
) -> Result<DecodedDegeneracyResult, ExchangeError> {
    let original: pb::ReferenceDegeneracyResult = decode_bounded_message(bytes, maximum_bytes)?;
    let native = native_projection::decode_reference_degeneracy_result(original.clone())?;
    Ok(DecodedDegeneracyResult { native, original })
}
fn decode_bounded_message<M: buffa::Message>(
    bytes: &[u8],
    maximum_bytes: usize,
) -> Result<M, ExchangeError> {
    decode_options(
        maximum_bytes,
        maximum_bytes
            .checked_mul(8)
            .ok_or_else(|| invalid("decode extent overflow"))?,
    )?
    .decode_from_slice(bytes)
    .map_err(invalid)
}
fn decode_options(
    maximum_bytes: usize,
    element_bytes: usize,
) -> Result<buffa::DecodeOptions, ExchangeError> {
    if maximum_bytes == 0 || maximum_bytes > MAXIMUM_BYTES {
        return Err(invalid(
            "select a scientific envelope allowance in 1..=16MiB",
        ));
    }
    Ok(buffa::DecodeOptions::new()
        .with_max_message_size(maximum_bytes)
        .with_element_memory_limit(element_bytes)
        .with_recursion_limit(RECURSION)
        .with_unknown_field_limit(UNKNOWN_FIELDS))
}

#[cfg(test)]
mod tests {
    use super::*;
    use buffa::Message as _;
    use pse_ids::{ContentHash, SemanticId};
    fn original() -> native::ReferenceEnvelope {
        native::ReferenceEnvelope {
            version: 1,
            meaning: native::ReferenceMeaning {
                case: SemanticId::from_bytes([1; 16]),
                source: ContentHash::from_bytes([2; 32]),
                formulation: ContentHash::from_bytes([3; 32]),
                physical: ContentHash::from_bytes([4; 32]),
                configuration: ContentHash::from_bytes([5; 32]),
            },
            required: vec![SemanticId::from_bytes([6; 16])],
            outcome: native::ReferenceOutcome::Completed {
                outputs: vec![native::ReferenceObservable {
                    id: SemanticId::from_bytes([6; 16]),
                    quantity: SemanticId::from_bytes([7; 16]),
                    unit: SemanticId::from_bytes([8; 16]),
                    basis: None,
                    datum: Some(SemanticId::NIL),
                    value: -0.0,
                }],
                blocks: vec![],
            },
        }
    }
    #[test]
    fn reference_wire_has_independently_specified_bits_presence_and_unknown_binary_forwarding() {
        let bytes = encode(original(), 1 << 20).unwrap();
        let wire = pb::ReferenceEnvelope::decode_from_slice(&bytes).unwrap();
        let pb::reference_outcome::Kind::Completed(value) = wire.outcome.kind.as_ref().unwrap()
        else {
            panic!("completed source alternative")
        };
        assert_eq!(
            value.outputs[0].value, 0x8000_0000_0000_0000,
            "literal IEEE-754 signed-zero payload"
        );
        assert_eq!(value.outputs[0].basis, None);
        assert_eq!(
            value.outputs[0].datum,
            Some(vec![0; 16]),
            "absent and explicit identity stay distinct"
        );
        assert_eq!(
            pb::ReferenceObservable {
                value: 0x8000_0000_0000_0000,
                ..Default::default()
            }
            .encode_to_vec(),
            [0x31, 0, 0, 0, 0, 0, 0, 0, 0x80],
            "independent tag6 fixed64 little-endian bytes"
        );
        let mut future = bytes;
        future.extend([0xc0, 0x3e, 0x2a]); // unknown field1000 varint42
        let decoded = decode(&future, 1 << 20).unwrap();
        let forwarded = decoded.forward(1 << 20).unwrap();
        let forwarded = pb::ReferenceEnvelope::decode_from_slice(&forwarded).unwrap();
        assert!(!forwarded.__buffa_unknown_fields.is_empty());
        let native::ReferenceOutcome::Completed { outputs, .. } = &decoded.known().outcome else {
            panic!("known interpretation")
        };
        assert_eq!(outputs[0].value.to_bits(), 0x8000_0000_0000_0000);
    }
    #[test]
    fn reference_native_adoption_keeps_nested_unknowns_in_the_retained_original() {
        let mut wire = native_projection::encode_reference_envelope(original()).unwrap();
        let pb::reference_outcome::Kind::Completed(completed) =
            wire.outcome.as_option_mut().unwrap().kind.as_mut().unwrap()
        else {
            panic!("completed source alternative")
        };
        let mut future_output = completed.outputs[0].encode_to_vec();
        future_output.extend([0xc0, 0x3e, 0x2a]);
        completed.outputs[0] = pb::ReferenceObservable::decode_from_slice(&future_output).unwrap();
        let decoded = decode(&wire.encode_to_vec(), 1 << 20).unwrap();
        let forwarded =
            pb::ReferenceEnvelope::decode_from_slice(&decoded.forward(1 << 20).unwrap()).unwrap();
        let pb::reference_outcome::Kind::Completed(completed) =
            forwarded.outcome.kind.as_ref().unwrap()
        else {
            panic!("retained alternative")
        };
        assert!(!completed.outputs[0].__buffa_unknown_fields.is_empty());
        let native::ReferenceOutcome::Completed { outputs, .. } = &decoded.known().outcome else {
            panic!("known interpretation")
        };
        assert_eq!(outputs[0].value.to_bits(), (-0.0_f64).to_bits());
    }
    #[test]
    fn selected_scalar_admission_preserves_values_presence_and_unknown_forwarding() {
        let expected = original();
        let admission = ScalarAdmission::new(&expected, MAXIMUM_BYTES).unwrap();
        let mut wire = native_projection::encode_reference_envelope(expected.clone()).unwrap();
        let pb::reference_outcome::Kind::Completed(completed) =
            wire.outcome.as_option_mut().unwrap().kind.as_mut().unwrap()
        else {
            panic!("completed original");
        };
        completed.outputs[0].value = 3.25f64.to_bits();
        completed.outputs[0]
            .__buffa_unknown_fields
            .push(buffa::UnknownField {
                number: 1000,
                data: buffa::UnknownFieldData::Varint(42),
            });
        wire.__buffa_unknown_fields.push(buffa::UnknownField {
            number: 1001,
            data: buffa::UnknownFieldData::LengthDelimited(vec![0xff; 1024]),
        });
        let decoded = admission.decode(&wire.encode_to_vec()).unwrap();
        let native::ReferenceOutcome::Completed { outputs, .. } = &decoded.known().outcome else {
            panic!("completed reference");
        };
        assert_eq!(
            outputs[0].value.to_bits(),
            3.25f64.to_bits(),
            "shape admission never substitutes its unused numeric placeholder"
        );
        assert_eq!(outputs[0].basis, None);
        assert_eq!(outputs[0].datum, Some(SemanticId::NIL));
        let forwarded =
            pb::ReferenceEnvelope::decode_from_slice(&decoded.forward(MAXIMUM_BYTES).unwrap())
                .unwrap();
        assert_eq!(
            forwarded, wire,
            "all root/nested unknown data retain their original ownership"
        );
    }

    #[test]
    fn selected_scalar_admission_refuses_million_byte_empty_output_amplification() {
        let expected = original();
        let admission = ScalarAdmission::new(&expected, MAXIMUM_BYTES).unwrap();
        let mut wire = native_projection::encode_reference_envelope(expected.clone()).unwrap();
        wire.outcome = Default::default();
        let empty_outputs = [0x0a, 0].repeat(524_288);
        let mut outcome = vec![0x0a];
        buffa::encoding::encode_varint(empty_outputs.len() as u64, &mut outcome);
        outcome.extend_from_slice(&empty_outputs);
        let mut bytes = wire.encode_to_vec();
        bytes.push(0x22);
        buffa::encoding::encode_varint(outcome.len() as u64, &mut bytes);
        bytes.extend_from_slice(&outcome);
        assert!(bytes.len() < MAXIMUM_BYTES);
        // The former128MiB element allowance would admit76MiB of output
        // headers here, then clone them and collect per-element errors.
        let error = admission.decode(&bytes).unwrap_err();
        assert!(
            matches!(&error, ExchangeError::Invalid(_)),
            "refuses in Buffa admission before clone/validator: {error}"
        );
        assert_eq!(
            admission.element_bytes,
            size_of::<Vec<u8>>() + size_of::<pb::ReferenceObservable>()
        );
    }

    #[test]
    fn selected_scalar_preflight_refuses_malformed_metadata_before_bulk_validation() {
        let expected = original();
        let admission = ScalarAdmission::new(&expected, MAXIMUM_BYTES).unwrap();
        let mut wire = native_projection::encode_reference_envelope(expected.clone()).unwrap();
        let pb::reference_outcome::Kind::Completed(completed) =
            wire.outcome.as_option_mut().unwrap().kind.as_mut().unwrap()
        else {
            panic!("completed original");
        };
        completed.outputs[0].quantity.clear();
        let error = admission.decode(&wire.encode_to_vec()).unwrap_err();
        assert!(
            matches!(&error, ExchangeError::Invalid(reason) if reason.contains("shape differs")),
            "{error}"
        );
        wire.required.clear();
        assert!(matches!(
            admission.decode(&wire.encode_to_vec()),
            Err(ExchangeError::Invalid(_))
        ));
    }

    #[test]
    fn selected_scalar_unknown_count_is_global_across_known_messages() {
        let expected = original();
        let admission = ScalarAdmission::new(&expected, MAXIMUM_BYTES).unwrap();
        let mut wire = native_projection::encode_reference_envelope(expected.clone()).unwrap();
        let pb::reference_outcome::Kind::Completed(completed) =
            wire.outcome.as_option_mut().unwrap().kind.as_mut().unwrap()
        else {
            panic!("completed original");
        };
        completed.outputs[0]
            .__buffa_unknown_fields
            .push(buffa::UnknownField {
                number: 1000,
                data: buffa::UnknownFieldData::Varint(42),
            });
        for _ in 0..UNKNOWN_FIELDS - 1 {
            wire.__buffa_unknown_fields.push(buffa::UnknownField {
                number: 1001,
                data: buffa::UnknownFieldData::Group(buffa::UnknownFields::new()),
            });
        }
        admission.decode(&wire.encode_to_vec()).unwrap();
        wire.__buffa_unknown_fields.push(buffa::UnknownField {
            number: 1002,
            data: buffa::UnknownFieldData::Varint(42),
        });
        assert!(matches!(
            admission.decode(&wire.encode_to_vec()),
            Err(ExchangeError::Invalid(_))
        ));
    }

    #[test]
    fn selected_scalar_nested_nonempty_groups_share_the_global_slot_budget() {
        let group = || {
            let mut leaf = buffa::UnknownFields::new();
            leaf.push(buffa::UnknownField {
                number: 1000,
                data: buffa::UnknownFieldData::Varint(42),
            });
            let mut inner = buffa::UnknownFields::new();
            inner.push(buffa::UnknownField {
                number: 1001,
                data: buffa::UnknownFieldData::Group(leaf),
            });
            buffa::UnknownField {
                number: 1002,
                data: buffa::UnknownFieldData::Group(inner),
            }
        };
        let expected = original();
        let admission = ScalarAdmission::new(&expected, MAXIMUM_BYTES).unwrap();
        let mut wire = native_projection::encode_reference_envelope(expected.clone()).unwrap();
        let pb::reference_outcome::Kind::Completed(completed) =
            wire.outcome.as_option_mut().unwrap().kind.as_mut().unwrap()
        else {
            panic!("completed original");
        };
        completed.outputs[0].__buffa_unknown_fields.push(group());
        let root_groups = (UNKNOWN_FIELDS - 3) / 3;
        for _ in 0..root_groups {
            wire.__buffa_unknown_fields.push(group());
        }
        for _ in 0..UNKNOWN_FIELDS - 3 - root_groups * 3 {
            wire.__buffa_unknown_fields.push(buffa::UnknownField {
                number: 1003,
                data: buffa::UnknownFieldData::Varint(1),
            });
        }
        let decoded = admission.decode(&wire.encode_to_vec()).unwrap();
        let forwarded =
            pb::ReferenceEnvelope::decode_from_slice(&decoded.forward(MAXIMUM_BYTES).unwrap())
                .unwrap();
        assert_eq!(
            forwarded, wire,
            "nonempty group Vecs and their nested payloads remain owned"
        );
        wire.__buffa_unknown_fields.push(buffa::UnknownField {
            number: 1004,
            data: buffa::UnknownFieldData::Varint(1),
        });
        assert!(matches!(
            admission.decode(&wire.encode_to_vec()),
            Err(ExchangeError::Invalid(_))
        ));
    }

    #[test]
    fn selected_scalar_oneof_replacement_keeps_the_final_original_completion() {
        let expected = original();
        let admission = ScalarAdmission::new(&expected, MAXIMUM_BYTES).unwrap();
        let mut wire = native_projection::encode_reference_envelope(expected.clone()).unwrap();
        let final_outcome = wire.outcome.take().unwrap();
        let alternatives = [
            pb::ReferenceOutcome {
                kind: Some(pb::reference_outcome::Kind::Completed(Box::default())),
                ..Default::default()
            },
            pb::ReferenceOutcome {
                kind: Some(pb::reference_outcome::Kind::Failed(Box::new(
                    pb::reference_outcome::Alternative2 {
                        code: "x".repeat(1024),
                        stage: "y".repeat(1024),
                        ..Default::default()
                    },
                ))),
                ..Default::default()
            },
            pb::ReferenceOutcome {
                kind: Some(pb::reference_outcome::Kind::Unavailable(Box::new(
                    pb::reference_outcome::Alternative3 {
                        value: "replaced diagnostic".repeat(128),
                        ..Default::default()
                    },
                ))),
                ..Default::default()
            },
            final_outcome,
        ];
        let mut bytes = wire.encode_to_vec();
        for outcome in alternatives {
            let encoded = outcome.encode_to_vec();
            bytes.push(0x22);
            buffa::encoding::encode_varint(encoded.len() as u64, &mut bytes);
            bytes.extend_from_slice(&encoded);
        }
        let decoded = admission.decode(&bytes).unwrap();
        assert_eq!(
            decoded.known(),
            &expected,
            "discarded alternatives do not become an original scientific failure"
        );
    }

    #[test]
    fn selected_scalar_failed_shape_and_custody_use_original_source_limits() {
        let mut expected = original();
        expected.outcome = native::ReferenceOutcome::Failed {
            code: "trial_rejected".into(),
            stage: "overheated_tear".into(),
        };
        let admission = ScalarAdmission::new(&expected, MAXIMUM_BYTES).unwrap();
        let decoded = admission
            .decode(&encode(expected.clone(), MAXIMUM_BYTES).unwrap())
            .unwrap();
        assert_eq!(decoded.known(), &expected);
        assert_eq!(admission.element_bytes, size_of::<Vec<u8>>());
        let raw = extent_add(MAXIMUM_BYTES, 1).unwrap();
        assert!(
            admission.custody_bound(1 << 20).unwrap() > raw + 2 * MAXIMUM_BYTES,
            "retained wire/clone contents, native adoption, forwarding and diagnostics are separate terms"
        );
        assert!(admission.custody_bound(usize::MAX).is_err());
        assert!(ScalarAdmission::new(&expected, 0).is_err());
        assert!(ScalarAdmission::new(&expected, MAXIMUM_BYTES + 1).is_err());
    }

    #[test]
    fn case_contract_encoding_custody_tracks_actual_native_capacity_and_wire_fields() {
        let context = crate::workflow::tests::physical();
        let document = context
            .describe(&crate::workflow::PhysicalInspectionRequest {
                query: crate::workflow::PhysicalInspectionQuery::Quantity {
                    id: pse_quantity::standard::ids::quantity("temperature.point"),
                    unit: None,
                },
                offset: 0,
                limit: 1,
            })
            .unwrap();
        let crate::workflow::PhysicalInspectionDocument::Page {
            mut quantities,
            owner,
            ..
        } = document
        else {
            panic!("actual physical page");
        };
        let _page_owner = owner.expect("physical allocation owner through encoding");
        let physical = quantities.remove(0);
        let mut contract = ReferenceCaseContract {
            version: Default::default(),
            case_name: "independent.physical-shape".into(),
            meaning: original().meaning,
            observables: vec![ReferenceObservableContract {
                output: SemanticId::from_bytes([6; 16]),
                key: "temperature".into(),
                physical,
            }],
            specification: Vec::new(),
            artifacts: vec![ReferenceInputArtifact {
                path: "source/original.pse".into(),
                sha256: vec![1; 32].into(),
            }],
        };
        let before = case_contract_encoding_bound(&contract, MAXIMUM_BYTES).unwrap();
        let capacity = contract.case_name.capacity();
        contract.case_name.reserve(4096);
        let extra = contract.case_name.capacity() - capacity;
        let after_string = case_contract_encoding_bound(&contract, MAXIMUM_BYTES).unwrap();
        assert!(
            after_string >= before + extra,
            "source String spare capacity is owned storage, independently of its encoded length"
        );
        let capacity = contract.observables.capacity();
        contract.observables.reserve(16);
        let extra =
            (contract.observables.capacity() - capacity) * size_of::<ReferenceObservableContract>();
        let after_vector = case_contract_encoding_bound(&contract, MAXIMUM_BYTES).unwrap();
        assert!(after_vector >= after_string + extra);
        let encoded = encode_case_contract(contract, MAXIMUM_BYTES).unwrap();
        assert!(
            after_vector > encoded.len() + MAXIMUM_BYTES,
            "known native/wire storage and output buffer are distinct custody terms"
        );
        assert!(case_contract_container_bound(usize::MAX, 1, &[], 0).is_err());
    }
    #[test]
    fn reference_wire_refuses_missing_unknown_alternative_bad_version_and_extent_before_output() {
        let bytes = encode(original(), 1 << 20).unwrap();
        assert!(decode(&bytes, bytes.len() - 1).is_err());
        assert!(decode(&bytes[..bytes.len() - 1], 1 << 20).is_err());
        assert!(encode(original(), 1).is_err());
        let mut wire = native_projection::encode_reference_envelope(original()).unwrap();
        wire.version = 2;
        assert!(decode(&wire.encode_to_vec(), 1 << 20).is_err());
        wire.version = 1;
        wire.outcome = pb::ReferenceOutcome::default().into();
        assert!(decode(&wire.encode_to_vec(), 1 << 20).is_err());
        let unknown_alternative = [0xa2, 0x06, 0x00]; // field100 length-delimited empty
        let outcome = pb::ReferenceOutcome::decode_from_slice(&unknown_alternative).unwrap();
        wire.outcome = outcome.into();
        assert!(decode(&wire.encode_to_vec(), 1 << 20).is_err());
    }

    #[tokio::test]
    async fn native_buf_structural_checks_and_scientific_datum_controls_remain_distinct() {
        use pse_host::schema::{Bundle, compare};
        let original_schema = Bundle::from_bytes(
            "reference-baseline.bin".into(),
            generated::DESCRIPTORS.to_vec(),
        )
        .unwrap();
        let current_schema = Bundle::from_bytes(
            "reference-current.bin".into(),
            generated::DESCRIPTORS.to_vec(),
        )
        .unwrap();
        let classifications = compare::classify(&current_schema, &original_schema)
            .await
            .unwrap();
        assert_eq!(classifications.len(), 3);
        assert!(
            classifications
                .iter()
                .all(|classification| classification.outcome == "compatible"),
            "missing Buf or tool error fails selected compatibility work: {classifications:?}"
        );

        let reference = original();
        let mut incompatible = original();
        let native::ReferenceOutcome::Completed { outputs, .. } = &mut incompatible.outcome else {
            panic!("completed source")
        };
        outputs[0].datum = None;
        assert!(
            native::compare(
                &incompatible,
                &reference,
                &[native::ComparisonAllowance {
                    output: SemanticId::from_bytes([6; 16]),
                    absolute: 0.,
                    relative: 0.,
                }]
            )
            .is_err(),
            "wire compatibility does not establish scientific datum equivalence"
        );

        let mut changed =
            buffa_descriptor::generated::descriptor::FileDescriptorSet::decode_from_slice(
                generated::DESCRIPTORS,
            )
            .unwrap();
        let file = changed
            .file
            .iter_mut()
            .find(|file| file.package.as_deref() == Some("pse.reference.v1"))
            .unwrap();
        let message = file
            .message_type
            .iter_mut()
            .find(|message| message.name.as_deref() == Some("ReferenceEnvelope"))
            .unwrap();
        message
            .field
            .iter_mut()
            .find(|field| field.name.as_deref() == Some("outcome"))
            .unwrap()
            .number = Some(20);
        let changed_schema =
            Bundle::from_bytes("reference-breaking.bin".into(), changed.encode_to_vec()).unwrap();
        let classifications = compare::classify(&changed_schema, &original_schema)
            .await
            .unwrap();
        assert!(
            classifications
                .iter()
                .any(|classification| classification.category == "WIRE"
                    && classification.outcome == "breaking"),
            "independent Buf must detect the changed stable field tag: {classifications:?}"
        );
    }
}
