// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Arrow-free projections of admitted source package data (ADR-0064).

use proc_macro2::TokenStream;
use pse_ids::SemanticId;
use pse_quantity::{
    DimensionVector, PhysicalPrecondition, PhysicalRequirement, QuantityOperation, QuantityRegistry,
};
use quote::{format_ident, quote};
use std::collections::BTreeMap;

use crate::SchemaError;
use crate::codegen::GeneratedTree;

/// Add the complete physical fixture projection to a Rust output tree.
/// The caller loads ordinary package documents and admits them before invoking this
/// pure renderer. Identity equality is never used instead of physical admission.
///
/// # Errors
/// A generated Rust syntax error or an output path already supplied by another emitter.
pub fn append_quantity_fixture(
    tree: &mut GeneratedTree,
    registry: &QuantityRegistry,
    preconditions: &[PhysicalPrecondition],
) -> Result<(), SchemaError> {
    let path = "crates/pse-quantity/src/generated/mod.rs";
    if tree.files.contains_key(std::path::Path::new(path)) {
        return Err(super::error(
            "quantity fixture output is duplicated".to_owned(),
        ));
    }
    let mut values = FixtureValues::default();
    let units = registry.units().map(|v| values.unit(v)).collect::<Vec<_>>();
    let kinds = registry
        .kinds()
        .map(|value| {
            let id = values.typed_id("QuantityKindId", value.id.as_id());
            let dimension = values.dimension(value.dimension);
            let extensive = value.extensive;
            let addition = enumeration("QuantityAdditionKind", value.addition_kind);
            quote! { b.kind(crate::QuantityKind { id: #id, dimension: #dimension,
            extensive: #extensive, addition_kind: #addition }); }
        })
        .collect::<Vec<_>>();
    let bases = registry
        .bases()
        .map(|v| values.basis(v))
        .collect::<Vec<_>>();
    let references = registry
        .reference_states()
        .map(|v| values.reference(v))
        .collect::<Vec<_>>();
    let quantities = registry
        .quantity_types()
        .map(|v| values.quantity(v))
        .collect::<Vec<_>>();
    let conversions = registry
        .conversions()
        .map(|v| values.conversion(v))
        .collect::<Vec<_>>();
    let operations = registry
        .operations()
        .map(|v| values.operation(v))
        .collect::<Vec<_>>();
    let reductions = registry
        .reduction_domains()
        .map(|(operation, domain)| {
            let operation = values.typed_id("OperationId", operation.as_id());
            let domain = enumeration("DomainKind", domain);
            quote! { b.reduction_domain(#operation, #domain); }
        })
        .collect::<Vec<_>>();
    let sets = registry
        .unit_sets()
        .map(|value| {
            let id = values.typed_id("UnitSetId", value.id.as_id());
            let base = value
                .base
                .map(|id| optional(id.map(|x| values.typed_id("UnitId", x.as_id()))));
            quote! { b.unit_set(crate::UnitSet { id: #id, base: [#(#base),*] }); }
        })
        .collect::<Vec<_>>();
    let neutral = registry.neutral_dimensionless().map(|id| {
        let id = values.typed_id("QuantityTypeId", id.as_id());
        quote! { b.neutral_dimensionless(#id); }
    });
    let preconditions = preconditions
        .iter()
        .map(|v| values.precondition(v))
        .collect::<Vec<_>>();
    let ids = values.declarations();
    let dimensions = values.dimension_values();
    let records = [
        units,
        kinds,
        bases,
        references,
        quantities,
        conversions,
        operations,
        reductions,
        sets,
    ]
    .concat();
    let mut helpers = Vec::new();
    let mut calls = Vec::new();
    for (index, records) in records.chunks(64).enumerate() {
        let name = format_ident!("append_{index}");
        helpers.push(quote! {
            fn #name(b: &mut crate::QuantityRegistryBuilder, dimensions: &[crate::DimensionVector]) {
                let _ = dimensions;
                #(#records)*
            }
        });
        calls.push(quote! { #name(&mut b, &dimensions); });
    }
    super::emit(
        tree,
        path,
        quote! {
            #(#ids)* #(#helpers)*
            /// Admit the full physical projection of the explicitly selected YAML packages.
            ///
        /// # Errors
        /// Invalid declarations are reported by the same production registry builder.
        #[allow(clippy::too_many_lines, reason = "mechanical complete package projection")]
        pub fn standard_registry() -> Result<crate::QuantityRegistry, crate::QuantityError> {
                let mut b = crate::QuantityRegistryBuilder::new();
                let dimensions = [#(#dimensions),*];
                #(#calls)* #neutral
                b.build()
            }
            /// Exact physical prerequisites projected from admitted source rows.
            #[allow(clippy::too_many_lines, reason = "mechanical complete package projection")]
            pub fn standard_preconditions() -> Vec<crate::PhysicalPrecondition> {
                vec![#(#preconditions),*]
            }
        },
    )
}

/// Add the element-table projection without making the quantity leaf depend on material.
///
/// # Errors
/// Invalid generated syntax or duplicate output ownership.
pub fn append_element_fixture(
    tree: &mut GeneratedTree,
    elements: &pse_material::ElementTable,
) -> Result<(), SchemaError> {
    let path = "crates/pse-material/src/generated/mod.rs";
    if tree.files.contains_key(std::path::Path::new(path)) {
        return Err(super::error(
            "element fixture output is duplicated".to_owned(),
        ));
    }
    let mut values = FixtureValues::default();
    let rows = elements
        .elements()
        .map(|element| {
            let id = values.typed_id("ElementId", element.id.as_id());
            let symbol = &element.symbol;
            let name = &element.name;
            let mass = number(element.atomic_mass);
            quote! { crate::Element { id: #id, symbol: #symbol.to_owned(),
            name: #name.to_owned(), atomic_mass: #mass } }
        })
        .collect::<Vec<_>>();
    let ids = values.declarations();
    super::emit(
        tree,
        path,
        quote! {
            #(#ids)*
            /// Admit the complete element projection of the selected physical package.
            ///
            /// # Errors
            /// The ordinary element-table constructor reports invalid declarations.
            #[allow(clippy::too_many_lines, reason = "mechanical complete package projection")]
            pub fn standard_elements() -> Result<crate::ElementTable, crate::MaterialError> {
                crate::ElementTable::new([#(#rows),*])
            }
        },
    )
}

#[derive(Default)]
struct FixtureValues {
    ids: BTreeMap<SemanticId, syn::Ident>,
    dimensions: BTreeMap<[u8; 32], usize>,
}
impl FixtureValues {
    fn declarations(&self) -> Vec<TokenStream> {
        self.ids.iter().map(|(value, name)| {
            let value = syn::LitInt::new(&format!("0x{:032x}_u128", u128::from_be_bytes(*value.as_bytes())), proc_macro2::Span::call_site());
            quote! { const #name: pse_ids::SemanticId = pse_ids::SemanticId::from_bytes(#value.to_be_bytes()); }
        }).collect()
    }
    fn dimension_values(&self) -> Vec<TokenStream> {
        let mut values = self.dimensions.iter().collect::<Vec<_>>();
        values.sort_by_key(|(_, index)| **index);
        values
            .into_iter()
            .map(|(bytes, _)| {
                quote! {
                    crate::DimensionVector::from_canonical_bytes(&[#(#bytes),*])?
                }
            })
            .collect()
    }
    fn precondition(&mut self, value: &PhysicalPrecondition) -> TokenStream {
        let id = self.typed_id("InvariantId", value.id.as_id());
        let positions = &value.operand_positions;
        let requirement = match value.requirement {
            PhysicalRequirement::EqualOperandBases { required } => {
                let required =
                    self.optional_id("BasisId", required.map(pse_quantity::BasisId::as_id));
                quote! { crate::PhysicalRequirement::EqualOperandBases { required: #required } }
            }
            PhysicalRequirement::OperandQuantityContract {
                required,
                match_shape,
            } => {
                let required = self.typed_id("QuantityTypeId", required.as_id());
                quote! { crate::PhysicalRequirement::OperandQuantityContract {
                    required: #required, match_shape: #match_shape,
                } }
            }
        };
        quote! { crate::PhysicalPrecondition { id: #id,
        operand_positions: vec![#(#positions),*], requirement: #requirement } }
    }

    fn id(&mut self, value: SemanticId) -> TokenStream {
        let next = self.ids.len();
        let name = self
            .ids
            .entry(value)
            .or_insert_with(|| format_ident!("ID_{next}"));
        quote! { #name }
    }
    fn typed_id(&mut self, name: &str, value: SemanticId) -> TokenStream {
        let name = format_ident!("{name}");
        let id = self.id(value);
        quote! { crate::#name::from_id(#id) }
    }

    fn optional_id(&mut self, name: &str, value: Option<SemanticId>) -> TokenStream {
        optional(value.map(|value| self.typed_id(name, value)))
    }

    fn dimension(&mut self, value: DimensionVector) -> TokenStream {
        let next = self.dimensions.len();
        let index = *self
            .dimensions
            .entry(value.canonical_bytes())
            .or_insert(next);
        quote! { dimensions[#index] }
    }
    fn unit(&mut self, value: &pse_quantity::Unit) -> TokenStream {
        let id = self.typed_id("UnitId", value.id.as_id());
        let symbol = &value.symbol;
        let dimension = self.dimension(value.dimension);
        let scale = number(value.scale_to_canonical);
        let offset = number(value.offset_to_canonical);
        let affine = value.is_affine;
        let reference = self.optional_id(
            "ReferenceStateId",
            value
                .reference_state
                .map(pse_quantity::ReferenceStateId::as_id),
        );
        quote! { b.unit(crate::Unit { id: #id, symbol: #symbol.to_owned(), dimension: #dimension,
        scale_to_canonical: #scale, offset_to_canonical: #offset, is_affine: #affine,
        reference_state: #reference }); }
    }
    fn basis(&mut self, value: &pse_quantity::Basis) -> TokenStream {
        let id = self.typed_id("BasisId", value.id.as_id());
        let kind = enumeration("BasisKind", value.kind);
        let composition = optional_enum("CompositionBasis", value.composition_basis);
        let rate = optional_enum("RateBasis", value.rate_basis);
        let reference = self.optional_id(
            "ReferenceStateId",
            value
                .reference_conditions
                .map(pse_quantity::ReferenceStateId::as_id),
        );
        quote! { b.basis(crate::Basis { id: #id, kind: #kind, composition_basis: #composition,
        rate_basis: #rate, reference_conditions: #reference }); }
    }
    fn reference(&mut self, value: &pse_quantity::ReferenceState) -> TokenStream {
        let id = self.typed_id("ReferenceStateId", value.id.as_id());
        let kind = enumeration("ReferenceStateKind", value.kind);
        let temperature = optional_number(value.temperature);
        let pressure = optional_number(value.pressure);
        let formation = value.include_enthalpy_of_formation;
        let phase = optional(value.phase.map(|value| self.id(value)));
        quote! { b.reference_state(crate::ReferenceState { id: #id, kind: #kind,
        temperature: #temperature, pressure: #pressure,
        include_enthalpy_of_formation: #formation, phase: #phase }); }
    }
    fn quantity(&mut self, value: &pse_quantity::QuantityType) -> TokenStream {
        let id = self.typed_id("QuantityTypeId", value.id.as_id());
        let kind = self.typed_id("QuantityKindId", value.key.kind.as_id());
        let basis = self.optional_id("BasisId", value.key.basis.map(pse_quantity::BasisId::as_id));
        let reference = self.optional_id(
            "ReferenceStateId",
            value
                .key
                .reference_state
                .map(pse_quantity::ReferenceStateId::as_id),
        );
        let scale = enumeration("ScaleKind", value.key.scale_kind);
        let shape = value.key.shape.iter().map(|x| enumeration("DomainKind", x));
        let subject = optional_enum("SubjectKind", value.key.subject_kind);
        let unit = self.typed_id("UnitId", value.canonical_unit.as_id());
        let nominal = optional_number(value.nominal_magnitude);
        quote! { b.quantity_type(crate::QuantityType { id: #id, key: crate::QuantityTypeKey {
            kind: #kind, basis: #basis, reference_state: #reference, scale_kind: #scale,
            shape: vec![#(#shape),*], subject_kind: #subject,
        }, canonical_unit: #unit, nominal_magnitude: #nominal }); }
    }
    fn conversion(&mut self, value: &pse_quantity::ConversionRule) -> TokenStream {
        let id = self.typed_id("ConversionId", value.id.as_id());
        let from = self.typed_id("QuantityTypeId", value.from.as_id());
        let to = self.typed_id("QuantityTypeId", value.to.as_id());
        let kind = enumeration("ConversionKind", value.kind);
        let kernel = optional(value.kernel.map(|value| self.id(value)));
        let parameters = &value.required_parameters;
        let scale = optional_number(value.scale);
        let offset = optional_number(value.offset);
        quote! { b.conversion(crate::ConversionRule { id: #id, from: #from, to: #to, kind: #kind,
        kernel: #kernel, required_parameters: vec![#(#parameters.to_owned()),*],
        scale: #scale, offset: #offset }); }
    }
    fn operation(&mut self, value: &QuantityOperation) -> TokenStream {
        let id = self.typed_id("OperationId", value.id.as_id());
        let opcode = enumeration("Opcode", value.opcode);
        let inputs = value
            .input_kinds
            .iter()
            .map(|x| self.typed_id("QuantityKindId", x.as_id()))
            .collect::<Vec<_>>();
        let result = self.typed_id("QuantityKindId", value.result_kind.as_id());
        let basis = enumeration("BasisRule", value.basis_rule);
        let reference = enumeration("ReferenceRule", value.reference_rule);
        let scale = enumeration("QuantityScaleRule", value.scale_rule);
        let shape = enumeration("QuantityShapeRule", value.shape_rule);
        let basis_source = value.basis_source.map(|x| quote! { #x });
        let reference_source = value.reference_source.map(|x| quote! { #x });
        let scale_source = value.scale_source.map(|x| quote! { #x });
        let shape_source = value.shape_source.map(|x| quote! { #x });
        let subject_source = value.subject_source.map(|x| quote! { #x });
        let sources = [
            basis_source,
            reference_source,
            scale_source,
            shape_source,
            subject_source,
        ]
        .map(optional);
        let [
            basis_source,
            reference_source,
            scale_source,
            shape_source,
            subject_source,
        ] = sources;
        let subject = enumeration("SubjectRule", value.subject_rule);
        let result_subject = optional_enum("SubjectKind", value.result_subject_kind);
        let result_basis = self.optional_id(
            "BasisId",
            value.result_basis.map(pse_quantity::BasisId::as_id),
        );
        let result_reference = self.optional_id(
            "ReferenceStateId",
            value
                .result_reference_state
                .map(pse_quantity::ReferenceStateId::as_id),
        );
        let conversions = value
            .input_conversions
            .iter()
            .map(|value| {
                let operand = value.operand;
                let conversion = self.typed_id("ConversionId", value.conversion.as_id());
                quote! { crate::InputConversion { operand: #operand, conversion: #conversion } }
            })
            .collect::<Vec<_>>();
        let invariants = value
            .precondition_invariants
            .iter()
            .map(|x| self.typed_id("InvariantId", x.as_id()))
            .collect::<Vec<_>>();
        quote! { b.operation(crate::QuantityOperation { id: #id, opcode: #opcode,
            input_kinds: vec![#(#inputs),*], result_kind: #result, basis_rule: #basis,
            reference_rule: #reference, scale_rule: #scale, shape_rule: #shape,
            basis_source: #basis_source, reference_source: #reference_source,
            scale_source: #scale_source, shape_source: #shape_source, subject_rule: #subject,
            subject_source: #subject_source, result_subject_kind: #result_subject,
            result_basis: #result_basis, result_reference_state: #result_reference,
            input_conversions: vec![#(#conversions),*], precondition_invariants: vec![#(#invariants),*],
        }); }
    }
}
fn optional(value: Option<TokenStream>) -> TokenStream {
    value.map_or_else(|| quote! { None }, |value| quote! { Some(#value) })
}
fn number(value: f64) -> TokenStream {
    let bits = value.to_bits();
    let bits = syn::LitInt::new(
        &format!(
            "0x{:04x}_{:04x}_{:04x}_{:04x}_u64",
            bits >> 48,
            (bits >> 32) & 0xffff,
            (bits >> 16) & 0xffff,
            bits & 0xffff
        ),
        proc_macro2::Span::call_site(),
    );
    quote! { f64::from_bits(#bits) }
}
fn optional_number(value: Option<f64>) -> TokenStream {
    optional(value.map(number))
}
fn enumeration(name: &str, value: impl std::fmt::Debug) -> TokenStream {
    let name = format_ident!("{name}");
    let variant = format_ident!("{value:?}");
    quote! { crate::#name::#variant }
}
fn optional_enum(name: &str, value: Option<impl std::fmt::Debug>) -> TokenStream {
    optional(value.map(|value| enumeration(name, value)))
}
