// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Borrowed inspection of admitted knowledge, without parsing, effects or a second owner.
use crate::{
    CheckedPackage, DeclarationId, Type,
    data::Table,
    entity::{Record, Typed},
    provenance::Provenance,
};

/// An immutable view of exactly one admitted package. Values remain owned by admission.
#[derive(Clone, Copy, Debug)]
pub struct Knowledge<'a> {
    package: &'a CheckedPackage,
}
impl CheckedPackage {
    /// Inspect admitted records, constants and relations, including derived values.
    /// Inspection preserves test-only status; it does not authorize a scientific read.
    pub fn knowledge(&self) -> Knowledge<'_> {
        Knowledge { package: self }
    }
}
impl<'a> Knowledge<'a> {
    /// Declared and keyed records, in identity order.
    pub fn records(self) -> impl Iterator<Item = (DeclarationId, &'a Record)> {
        self.package
            .entities
            .iter()
            .map(|(id, record)| (*id, record))
    }
    /// Typed constants, retaining declared uncertainty.
    pub fn constants(self) -> impl Iterator<Item = (DeclarationId, &'a Typed)> {
        self.package
            .constants
            .iter()
            .map(|(id, value)| (*id, value))
    }
    /// Admitted table schemas and rows, in declaration and canonical key order.
    pub fn tables(self) -> impl Iterator<Item = (DeclarationId, &'a Table)> {
        self.package.tables.iter().map(|(id, table)| (*id, table))
    }
    /// Resolved names, without interpreting identifier values or source text.
    pub fn names(self) -> impl Iterator<Item = (&'a str, DeclarationId)> {
        self.package
            .names
            .iter()
            .map(|(name, id)| (name.as_str(), *id))
    }
    /// Checked types, including parameterized quantity and reference contracts.
    pub fn types(self) -> impl Iterator<Item = (DeclarationId, &'a Type)> {
        self.package.types.iter().map(|(id, ty)| (*id, ty))
    }
    /// Source, role and lineage of an admitted origin.
    pub fn provenance(self, origin: DeclarationId) -> Option<&'a Provenance> {
        self.package.provenance(origin)
    }
    /// Explicit attribute origin, falling back to its record's origin.
    pub fn attribute_provenance(
        self,
        entity: DeclarationId,
        attribute: &str,
    ) -> Option<&'a Provenance> {
        self.package.attribute_provenance(entity, attribute)
    }
    /// Whether a record, constant or keyed row is test-only.
    pub fn is_test_only(self, id: DeclarationId) -> bool {
        self.package.is_test_only(id)
    }
}

#[cfg(test)]
mod tests {
    use crate::kernel_types::{physical, source};
    use crate::specialize::Value;
    use crate::{PhysicalScope, TypeContext, check};

    #[test]
    fn inspection_borrows_admitted_records_constants_tables_and_origins() {
        let (registry, _) = physical();
        let preconditions = pse_quantity::PhysicalPreconditions::new(vec![]).unwrap();
        let scope = PhysicalScope::default();
        let context = TypeContext {
            formula_authority: None,
            quantities: &registry,
            preconditions: &preconditions,
            scope: &scope,
        };
        let package = check(
            &source(
                r#"package p {
            entity kind source provenance { attribute title:Text; }
            enum role { given, synthetic facets(test_only) }
            entity source s { title="inspection" }
            entity kind sample { attribute energy:Energy; }
            entity sample a provenance(s,role.synthetic) { energy=2{kJ} ± standard(0.1) }
            constant offset:Energy=3{kJ} provenance(s,role.given);
            table bank[n:Integer]:{energy:Energy,derived twice:Energy=2*energy} missing optional;
            dataset values:bank provenance(s,role.given) { [0]=[1{kJ}]; }
        }"#,
            ),
            &context,
        )
        .unwrap();
        let knowledge = package.knowledge();
        let id = package.names["p.a"];
        let record = knowledge.records().find(|(key, _)| *key == id).unwrap().1;
        assert!(std::ptr::eq(record, &package.entities[&id]));
        let Value::Number { bits, .. } = record.values["energy"] else {
            panic!("quantity")
        };
        assert_eq!(f64::from_bits(bits), 2000.);
        assert_eq!(record.uncertainties["energy"].magnitude, 100.);
        assert!(knowledge.is_test_only(id));
        assert!(
            knowledge
                .attribute_provenance(id, "energy")
                .unwrap()
                .test_only()
        );
        assert_eq!(knowledge.constants().count(), 1);
        let table = knowledge.tables().next().unwrap().1;
        let row = table.rows.values().next().unwrap();
        let Value::Number { bits, .. } = row.cells[1] else {
            panic!("derived quantity")
        };
        assert_eq!(f64::from_bits(bits), 2000.);
        assert!(!row.test_only);
        assert!(knowledge.provenance(row.origin).is_some());
        assert_eq!(
            knowledge
                .names()
                .find(|(name, _)| *name == "p.a")
                .unwrap()
                .1,
            id
        );
    }
}
