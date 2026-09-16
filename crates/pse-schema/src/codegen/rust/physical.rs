// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Arrow-free projections of admitted source package data (ADR-0064).

use proc_macro2::TokenStream;
use pse_ids::SemanticId;
use pse_quantity::{
    DimensionVector, PhysicalPrecondition, PhysicalRequirement, QuantityOperation, QuantityRegistry,
};
use quote::{format_ident, quote};

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
    let units = registry.units().map(unit);
    let kinds = registry.kinds().map(|value| {
        let id = typed_id("QuantityKindId", value.id.as_id());
        let dimension = dimension(value.dimension);
        let extensive = value.extensive;
        let addition = enumeration("QuantityAdditionKind", value.addition_kind.as_str());
        quote! { b.kind(crate::QuantityKind { id: #id, dimension: #dimension,
        extensive: #extensive, addition_kind: #addition }); }
    });
    let bases = registry.bases().map(basis);
    let references = registry.reference_states().map(reference);
    let quantities = registry.quantity_types().map(quantity);
    let conversions = registry.conversions().map(conversion);
    let operations = registry.operations().map(operation);
    let reductions = registry.reduction_domains().map(|(operation, domain)| {
        let operation = typed_id("OperationId", operation.as_id());
        let domain = enumeration("DomainKind", domain.as_str());
        quote! { b.reduction_domain(#operation, #domain); }
    });
    let sets = registry.unit_sets().map(|value| {
        let id = typed_id("UnitSetId", value.id.as_id());
        let base = value
            .base
            .map(|id| optional(id.map(|x| typed_id("UnitId", x.as_id()))));
        quote! { b.unit_set(crate::UnitSet { id: #id, base: [#(#base),*] }); }
    });
    let neutral = registry.neutral_dimensionless().map(|id| {
        let id = typed_id("QuantityTypeId", id.as_id());
        quote! { b.neutral_dimensionless(#id); }
    });
    let preconditions = preconditions.iter().map(precondition);
    super::emit(
        tree,
        path,
        quote! {
            /// Admit the full physical projection of the explicitly selected YAML packages.
            ///
        /// # Errors
        /// Invalid declarations are reported by the same production registry builder.
        #[allow(clippy::too_many_lines, reason = "mechanical complete package projection")]
        pub fn standard_registry() -> Result<crate::QuantityRegistry, crate::QuantityError> {
                let mut b = crate::QuantityRegistryBuilder::new();
                #(#units)* #(#kinds)* #(#bases)* #(#references)* #(#quantities)*
                #(#conversions)* #(#operations)* #(#reductions)* #(#sets)* #neutral
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

fn precondition(value: &PhysicalPrecondition) -> TokenStream {
    let id = typed_id("InvariantId", value.id.as_id());
    let positions = &value.operand_positions;
    let requirement = match value.requirement {
        PhysicalRequirement::EqualOperandBases { required } => {
            let required = optional_id("BasisId", required.map(pse_quantity::BasisId::as_id));
            quote! { crate::PhysicalRequirement::EqualOperandBases { required: #required } }
        }
        PhysicalRequirement::OperandQuantityContract {
            required,
            match_shape,
        } => {
            let required = typed_id("QuantityTypeId", required.as_id());
            quote! { crate::PhysicalRequirement::OperandQuantityContract {
                required: #required, match_shape: #match_shape,
            } }
        }
    };
    quote! { crate::PhysicalPrecondition { id: #id,
    operand_positions: vec![#(#positions),*], requirement: #requirement } }
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
    let rows = elements.elements().map(|element| {
        let id = typed_id("ElementId", element.id.as_id());
        let symbol = &element.symbol;
        let name = &element.name;
        let mass = number(element.atomic_mass);
        quote! { crate::Element { id: #id, symbol: #symbol.to_owned(),
        name: #name.to_owned(), atomic_mass: #mass } }
    });
    super::emit(
        tree,
        path,
        quote! {
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

fn id(value: SemanticId) -> TokenStream {
    let bytes = value.as_bytes();
    quote! { pse_ids::SemanticId::from_bytes([#(#bytes),*]) }
}
fn typed_id(name: &str, value: SemanticId) -> TokenStream {
    let name = format_ident!("{name}");
    let id = id(value);
    quote! { crate::#name::from_id(#id) }
}
fn optional(value: Option<TokenStream>) -> TokenStream {
    value.map_or_else(|| quote! { None }, |value| quote! { Some(#value) })
}
fn optional_id(name: &str, value: Option<SemanticId>) -> TokenStream {
    optional(value.map(|value| typed_id(name, value)))
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
fn enumeration(name: &str, value: &str) -> TokenStream {
    let name = format_ident!("{name}");
    quote! { crate::#name::parse(#value).ok_or_else(|| crate::QuantityError::Registry {
        rule: "fixture.enum", subject: pse_ids::SemanticId::NIL,
        detail: format!("unknown generated enum spelling {}", #value),
    })? }
}
fn optional_enum(name: &str, value: Option<&str>) -> TokenStream {
    optional(value.map(|value| enumeration(name, value)))
}
fn dimension(value: DimensionVector) -> TokenStream {
    let exponents = value.exponents().iter().map(|exponent| {
        let num = exponent.num();
        let den = exponent.den();
        quote! { crate::Ratio::from_parts(#num, #den)? }
    });
    quote! { crate::DimensionVector::new([#(#exponents),*]) }
}
fn unit(value: &pse_quantity::Unit) -> TokenStream {
    let id = typed_id("UnitId", value.id.as_id());
    let symbol = &value.symbol;
    let dimension = dimension(value.dimension);
    let scale = number(value.scale_to_canonical);
    let offset = number(value.offset_to_canonical);
    let affine = value.is_affine;
    let reference = optional_id(
        "ReferenceStateId",
        value
            .reference_state
            .map(pse_quantity::ReferenceStateId::as_id),
    );
    quote! { b.unit(crate::Unit { id: #id, symbol: #symbol.to_owned(), dimension: #dimension,
    scale_to_canonical: #scale, offset_to_canonical: #offset, is_affine: #affine,
    reference_state: #reference }); }
}
fn basis(value: &pse_quantity::Basis) -> TokenStream {
    let id = typed_id("BasisId", value.id.as_id());
    let kind = enumeration("BasisKind", value.kind.as_str());
    let composition = optional_enum(
        "CompositionBasis",
        value
            .composition_basis
            .map(pse_quantity::CompositionBasis::as_str),
    );
    let rate = optional_enum(
        "RateBasis",
        value.rate_basis.map(pse_quantity::RateBasis::as_str),
    );
    let reference = optional_id(
        "ReferenceStateId",
        value
            .reference_conditions
            .map(pse_quantity::ReferenceStateId::as_id),
    );
    quote! { b.basis(crate::Basis { id: #id, kind: #kind, composition_basis: #composition,
    rate_basis: #rate, reference_conditions: #reference }); }
}
fn reference(value: &pse_quantity::ReferenceState) -> TokenStream {
    let id = typed_id("ReferenceStateId", value.id.as_id());
    let kind = enumeration("ReferenceStateKind", value.kind.as_str());
    let temperature = optional_number(value.temperature);
    let pressure = optional_number(value.pressure);
    let formation = value.include_enthalpy_of_formation;
    let phase = optional(value.phase.map(self::id));
    quote! { b.reference_state(crate::ReferenceState { id: #id, kind: #kind,
    temperature: #temperature, pressure: #pressure,
    include_enthalpy_of_formation: #formation, phase: #phase }); }
}
fn quantity(value: &pse_quantity::QuantityType) -> TokenStream {
    let id = typed_id("QuantityTypeId", value.id.as_id());
    let kind = typed_id("QuantityKindId", value.key.kind.as_id());
    let basis = optional_id("BasisId", value.key.basis.map(pse_quantity::BasisId::as_id));
    let reference = optional_id(
        "ReferenceStateId",
        value
            .key
            .reference_state
            .map(pse_quantity::ReferenceStateId::as_id),
    );
    let scale = enumeration("ScaleKind", value.key.scale_kind.as_str());
    let shape = value
        .key
        .shape
        .iter()
        .map(|x| enumeration("DomainKind", x.as_str()));
    let subject = optional_enum(
        "SubjectKind",
        value
            .key
            .subject_kind
            .map(pse_quantity::SubjectKind::as_str),
    );
    let unit = typed_id("UnitId", value.canonical_unit.as_id());
    let nominal = optional_number(value.nominal_magnitude);
    quote! { b.quantity_type(crate::QuantityType { id: #id, key: crate::QuantityTypeKey {
        kind: #kind, basis: #basis, reference_state: #reference, scale_kind: #scale,
        shape: vec![#(#shape),*], subject_kind: #subject,
    }, canonical_unit: #unit, nominal_magnitude: #nominal }); }
}
fn conversion(value: &pse_quantity::ConversionRule) -> TokenStream {
    let id = typed_id("ConversionId", value.id.as_id());
    let from = typed_id("QuantityTypeId", value.from.as_id());
    let to = typed_id("QuantityTypeId", value.to.as_id());
    let kind = enumeration("ConversionKind", value.kind.as_str());
    let kernel = optional(value.kernel.map(self::id));
    let parameters = &value.required_parameters;
    let scale = optional_number(value.scale);
    let offset = optional_number(value.offset);
    quote! { b.conversion(crate::ConversionRule { id: #id, from: #from, to: #to, kind: #kind,
    kernel: #kernel, required_parameters: vec![#(#parameters.to_owned()),*],
    scale: #scale, offset: #offset }); }
}
fn operation(value: &QuantityOperation) -> TokenStream {
    let id = typed_id("OperationId", value.id.as_id());
    let opcode = enumeration("Opcode", value.opcode.as_str());
    let inputs = value
        .input_kinds
        .iter()
        .map(|x| typed_id("QuantityKindId", x.as_id()));
    let result = typed_id("QuantityKindId", value.result_kind.as_id());
    let basis = enumeration("BasisRule", value.basis_rule.as_str());
    let reference = enumeration("ReferenceRule", value.reference_rule.as_str());
    let scale = enumeration("QuantityScaleRule", value.scale_rule.as_str());
    let shape = enumeration("QuantityShapeRule", value.shape_rule.as_str());
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
    let subject = enumeration("SubjectRule", value.subject_rule.as_str());
    let result_subject = optional_enum(
        "SubjectKind",
        value
            .result_subject_kind
            .map(pse_quantity::SubjectKind::as_str),
    );
    let result_basis = optional_id(
        "BasisId",
        value.result_basis.map(pse_quantity::BasisId::as_id),
    );
    let result_reference = optional_id(
        "ReferenceStateId",
        value
            .result_reference_state
            .map(pse_quantity::ReferenceStateId::as_id),
    );
    let conversions = value.input_conversions.iter().map(|value| {
        let operand = value.operand;
        let conversion = typed_id("ConversionId", value.conversion.as_id());
        quote! { crate::InputConversion { operand: #operand, conversion: #conversion } }
    });
    let invariants = value
        .precondition_invariants
        .iter()
        .map(|x| typed_id("InvariantId", x.as_id()));
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
