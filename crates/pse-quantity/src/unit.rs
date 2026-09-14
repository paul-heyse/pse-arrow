// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Units and the unit-conversion edge (blueprint §6.2, §8.2).
//!
//! Filled by packet Q-1: `Unit`, `convert_spec` — the single source for every
//! `UnitConvert` edge P3, P9 and P10 emit or check — and `convert_value`, which applies
//! the two roundings of `(v * scale) + offset` in that order and never contracts them into
//! an FMA (§7.3, ADR-0047).
//!
//! [`UnitConvertSpec`] lands with the keel rather than with Q-1 because
//! `pse_mathir::Payload::UnitConvert` carries it: the payload of a node and the value a
//! conversion function returns are the same four numbers, and declaring them twice would
//! be the second copy prime directive 3 refuses.

use crate::ids::UnitId;

/// A resolved unit-conversion edge: `to = (from * scale) + offset` (blueprint §7.2, §8.2).
///
/// The offset is nonzero only when a *point* quantity is converted out of an affine unit
/// (°C, psig) — a difference converts by scale alone, because the datum cancels. §7.2
/// makes the consequence explicit: after P10 no node carries an affine unit, because every
/// such conversion is a visible `UnitConvert` node rather than an implicit reinterpretation.
///
/// The two `f64` fields are compared through [`pse_ids::canonical_f64_bits`] wherever two
/// specs must be judged equal, never with `==` (ADR-0030); the derived `PartialEq` here is
/// the ordinary IEEE comparison and is intended for tests and debugging.
///
/// ```
/// use pse_ids::SemanticId;
/// use pse_quantity::{UnitConvertSpec, UnitId};
///
/// // °C to K, as a point quantity: scale 1, offset 273.15.
/// let celsius = UnitId::from_id(SemanticId::from_bytes([1; 16]));
/// let kelvin = UnitId::from_id(SemanticId::from_bytes([2; 16]));
/// let spec = UnitConvertSpec {
///     from: celsius,
///     to: kelvin,
///     scale: 1.0,
///     offset: 273.15,
/// };
/// assert_eq!(spec.from, celsius);
/// ```
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct UnitConvertSpec {
    /// The unit the value is in before the conversion.
    pub from: UnitId,
    /// The unit the value is in after the conversion.
    pub to: UnitId,
    /// The multiplicative factor, applied first.
    pub scale: f64,
    /// The additive offset, applied second; zero for a difference conversion.
    pub offset: f64,
}
