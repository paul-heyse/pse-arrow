// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The element table (blueprint §6.4, `reference.elements`).
//!
//! Atomic masses and explicit elemental compositions; formulas never infer identities.

use crate::{ElementId, MaterialError};
use std::collections::{BTreeMap, BTreeSet};

/// One declared `reference.elements` row. Atomic mass is in kg/mol.
#[derive(Clone, Debug, PartialEq)]
pub struct Element {
    /// Actual element identity.
    pub id: ElementId,
    /// Declared display symbol, not a lookup substitute for the identity.
    pub symbol: String,
    /// Declared display name.
    pub name: String,
    /// Finite positive atomic mass in kg/mol.
    pub atomic_mass: f64,
}

/// One species' explicit `authored.species_elements` entry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ElementCount {
    /// Actual element identity.
    pub element: ElementId,
    /// Finite nonnegative count, including fractional empirical compositions.
    pub count: f64,
}

/// An admitted element inventory. No built-in periodic table is platform authority.
#[derive(Clone, Debug, Default)]
pub struct ElementTable(BTreeMap<ElementId, Element>);

impl ElementTable {
    /// Admit actual rows, rejecting duplicate identities and invalid atomic masses.
    /// # Errors
    /// Duplicate identities or nonpositive/nonfinite atomic masses.
    pub fn new(elements: impl IntoIterator<Item = Element>) -> Result<Self, MaterialError> {
        let mut rows = BTreeMap::new();
        for element in elements {
            if !element.atomic_mass.is_finite() || element.atomic_mass <= 0.0 {
                return Err(invalid(
                    element.id,
                    "atomic mass must be finite and positive",
                ));
            }
            let id = element.id;
            if rows.insert(id, element).is_some() {
                return Err(invalid(id, "duplicate element identity"));
            }
        }
        Ok(Self(rows))
    }

    /// Resolve the actual declared element row.
    /// # Errors
    /// The identity is absent from this inventory.
    pub fn get(&self, id: ElementId) -> Result<&Element, MaterialError> {
        self.0.get(&id).ok_or(MaterialError::UnknownId {
            kind: "element",
            id: id.as_id(),
        })
    }

    /// Actual admitted rows in identity order.
    pub fn elements(&self) -> impl ExactSizeIterator<Item = &Element> {
        self.0.values()
    }
}

/// Compute kg/mol from explicit elemental counts and actual declared atomic masses.
/// An absent composition returns `None`; it never becomes a guessed formula or zero mass.
/// Accumulation follows element identity order, independent of input row arrival order.
/// # Errors
/// Unknown/duplicate element, negative/nonfinite count, or nonfinite arithmetic.
pub fn molecular_weight(
    elements: &ElementTable,
    composition: &[ElementCount],
) -> Result<Option<f64>, MaterialError> {
    let ordered = admit_composition(elements, composition)?;
    if ordered.is_empty() {
        return Ok(None);
    }
    let mut total = 0.0;
    for item in ordered {
        total += elements.get(item.element)?.atomic_mass * item.count;
        if !total.is_finite() {
            return Err(invalid(
                item.element,
                "molecular-weight arithmetic is nonfinite",
            ));
        }
    }
    Ok(Some(total))
}

pub(crate) fn admit_composition<'a>(
    elements: &ElementTable,
    composition: &'a [ElementCount],
) -> Result<Vec<&'a ElementCount>, MaterialError> {
    let mut seen = BTreeSet::new();
    for item in composition {
        elements.get(item.element)?;
        if !item.count.is_finite() || item.count < 0.0 {
            return Err(invalid(
                item.element,
                "element count must be finite and nonnegative",
            ));
        }
        if !seen.insert(item.element) {
            return Err(invalid(item.element, "duplicate species/element entry"));
        }
    }
    let mut ordered: Vec<_> = composition.iter().collect();
    ordered.sort_by_key(|item| item.element);
    Ok(ordered)
}

fn invalid(element: ElementId, detail: &str) -> MaterialError {
    MaterialError::Invariant {
        rule: "material.element",
        subject: element.as_id(),
        detail: detail.to_owned(),
    }
}
