// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Dimensions, units, quantity kinds and types, bases, reference states and conversions
//! (blueprint §3.2, §6.2, §8).
//!
//! Physical typing is declared in the registry (blueprint §8.1). A quantity type is a
//! row, resolved at compile time by a pass, and this crate holds the native contracts
//! and algebra used by that computation. Full library eligibility follows §3.3.2.
//!
//! # Why a dimension vector is not a physical type
//!
//! Dimensional analysis answers one question — do the exponents combine? — and §8.1 is
//! built on the observation that the answer is not enough. Torque and energy share
//! `M·L^2·T^-2`. A molar and a mass-specific enthalpy share `L^2·T^-2` after the basis is
//! divided out. An absolute and a gauge pressure share everything but their datum. Each of
//! those pairs is a modelling error that a dimension checker accepts, so a
//! [`QuantityTypeId`] resolves the complete §8.1 tuple — kind, dimension, basis, reference
//! state, point/difference scale, index shape, subject and canonical unit — and
//! composition goes through a registered `quantity_operations` rule rather than through
//! exponent arithmetic (§8.3).
//!
//! [`DimensionVector`] is therefore a *component* of that answer and never the answer.
//!
//! # Layout
//!
//! Landed by the keel (packet K-4):
//!
//! - [`dimension`] — [`Ratio`], [`BaseDimension`] and [`DimensionVector`] with canonical
//!   bytes.
//! - [`ids`] — the identity newtypes, and the [`semantic_id_newtype`] macro that declares
//!   them.
//! - [`enums`] — the closed registry dictionaries, including the 43 [`Opcode`]s of §7.2,
//!   and the [`closed_enum`] macro that gives each member one spelling.
//! - [`index`] — [`BoundIndexRef`] and [`IndexSet`]: free-index identity.
//! - [`error`] — every error enum with its §23.2 class.
//! - [`mod@unit`] — [`UnitConvertSpec`], physically checked representation conversion.
//!
//! Registry admission and representation conversion live in [`registry`], [`unit_set`],
//! [`conversion`], [`kind`], [`basis`], [`reference_state`] and [`quantity_type`].
//! [`operation`], [`infer`] and [`literal`] implement complete contract composition and
//! occurrence-specific literal resolution. [`admission`] and [`numeric`] provide boundary
//! comparisons; the `fixtures` feature exposes the standard test package.
//!
//! [`closed_enum`]: crate::closed_enum
//! [`semantic_id_newtype`]: crate::semantic_id_newtype

pub mod admission;
pub mod basis;
pub mod conversion;
pub mod dimension;
pub mod enums;
pub mod error;
/// Arrow-free projection of explicitly selected reference package documents.
#[cfg(feature = "fixtures")]
#[rustfmt::skip]
pub mod generated;
pub mod ids;
pub mod index;
pub mod infer;
pub mod kind;
pub mod literal;
pub mod numeric;
pub mod operation;
pub mod precondition;
pub mod quantity_type;
pub mod reference_state;
pub mod registry;
pub mod smoothing;
#[cfg(feature = "fixtures")]
pub mod standard;
pub mod unit;
pub mod unit_set;

pub use crate::basis::Basis;
pub use crate::conversion::ConversionRule;
pub use crate::dimension::{BaseDimension, DimensionVector, Ratio};
pub use crate::enums::{
    BasisKind, BasisRule, CompositionBasis, ConversionKind, DomainKind, Opcode,
    QuantityAdditionKind, QuantityScaleRule, QuantityShapeRule, RateBasis, ReductionKind,
    ReferenceRule, ReferenceStateKind, ScaleKind, SubjectKind, SubjectRule, WeightNormalization,
};
pub use crate::error::{ContractComponent, DimensionError, IncompatibilityReason, QuantityError};
pub use crate::ids::{
    BasisId, BoundIndexId, ConstantId, ConversionId, DomainId, InvariantId, OperationId,
    QuantityKindId, QuantityTypeId, ReferenceStateId, UnitId, UnitSetId,
};
pub use crate::index::{BinderConflict, BoundIndexRef, IndexSet};
pub use crate::kind::QuantityKind;
pub use crate::operation::{InputConversion, QuantityOperation};
pub use crate::precondition::{PhysicalPrecondition, PhysicalRequirement};
pub use crate::quantity_type::{QuantityType, QuantityTypeKey};
pub use crate::reference_state::ReferenceState;
pub use crate::registry::{QuantityRegistry, QuantityRegistryBuilder};
pub use crate::unit::{Unit, UnitConvertSpec, convert_spec, convert_spec_for_type, convert_value};
pub use crate::unit_set::{DerivedUnit, UnitSet};

mod preconditions;
pub use preconditions::PhysicalPreconditions;

/// Authored scalar-function and physical-admission handler vocabulary.
pub mod functions;
