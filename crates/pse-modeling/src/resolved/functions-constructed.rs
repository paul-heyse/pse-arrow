// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Direct synthesized contracts. No source body is visited or resolved here.
use super::*;
use crate::expression::admission::PhysicalAdmission;
use crate::specialize::{Lineage, Symbol, Symbols, finite::ReductionSelection};
use pse_quantity::{PhysicalPreconditions, QuantityRegistry, infer::OpRequest, scheme::Scheme};

impl FunctionProduct {
    pub(crate) fn finite_reduction(
        at: DeclarationId,
        kind: dsl::ReduceKind,
        selection: ReductionSelection,
        signature: (Type, Type),
        terms: usize,
        registry: &QuantityRegistry,
        facts: &PhysicalPreconditions,
    ) -> Result<Arc<Self>> {
        let (prototype, result) = signature;
        let (kind, domain) =
            crate::expression::finite_reduction(kind, &selection.carrier.element, at)?;
        let contract = if let Some(scheme) = prototype.quantity_scheme() {
            scheme
                .resolve_contract_with_evidence(registry, &BTreeMap::new(), facts)
                .map_err(|error| invalid(at, error.to_string()))?
        } else if prototype == Type::Integer {
            pse_quantity::ResolvedPhysicalContract::named(
                registry
                    .neutral_dimensionless()
                    .ok_or_else(|| invalid(at, "neutral type absent"))?,
                pse_quantity::IndexSet::new(),
                registry,
            )
            .map_err(|error| invalid(at, error.to_string()))?
        } else {
            return Err(invalid(
                at,
                "finite reduction requires a physical or exact integer prototype",
            ));
        };
        let request = OpRequest::FiniteReduce { kind, domain };
        let admission = crate::expression::admission::admit_operation(
            &request,
            std::slice::from_ref(&contract),
            registry,
            facts,
            None,
            at,
        )?;
        let obligation = PhysicalAdmission::checked(
            &request,
            vec![Scheme::from_contract(contract.clone())],
            Some(admission),
            at,
        )?;
        let mut symbols = Symbols::default();
        let mut syntax = ResolvedSyntax::default();
        let mut formals = Vec::with_capacity(terms);
        for ordinal in 0..terms {
            let slot = SymbolIx::from(ordinal);
            let mut identity = FramedHasher::new(pse_ids::Frame::ModelingResolvedFunctionV1);
            identity.str("local-formal").u64(ordinal as u64);
            symbols.push(Symbol {
                id: identity.finish_id(),
                ty: prototype.clone(),
                role: ModelingDeclarationKind::Variable,
                domain: ModelingVariableDomain::Continuous,
                definition: None,
                expression: None,
                initial: None,
                lineage: Lineage {
                    declaration: at,
                    instance: SemanticId::NIL.into(),
                    path: format!("term_{ordinal}"),
                    demand: vec![],
                    default_owner: None,
                    is_override: false,
                    presets: vec![],
                },
            });
            let root = NodeIx::from(syntax.nodes.len());
            syntax.nodes.push(Node::Reference(Reference::Symbol(slot)));
            syntax.node_sources.insert(
                root,
                Source {
                    field: crate::expression::occurrences::OccurrenceKey {
                        declaration: at,
                        role: "derived.reduction.arguments".into(),
                        position: ordinal,
                    },
                    span: dsl::Span::default(),
                    coordinate: None,
                },
            );
            syntax.outputs.push(root);
            formals.push(slot);
        }
        syntax.symbols = symbols;
        let scope = crate::PhysicalScope::default();
        let context = crate::TypeContext {
            admissions: None,
            formula_authority: None,
            quantities: registry,
            preconditions: facts,
            scope: &scope,
        };
        let (physical, _, body_inputs) = PhysicalSupport::prepare_function(
            &syntax,
            &context,
            FunctionSupportRequest {
                body: None,
                result: &result,
                literals: &BTreeMap::new(),
                operation: None,
                at,
                obligations: &BTreeMap::new(),
                reduction: Some(&obligation),
                signature: &vec![prototype.clone(); terms],
                formals: &formals,
                meaning: None,
            },
        )?;
        Self {
            identity: ContentHash::from_bytes([0; 32]),
            has_applicability_effects: false,
            declaration: at,
            syntax,
            physical: Arc::new(physical),
            formals,
            signature: vec![prototype; terms],
            arguments: vec![],
            selections: vec![],
            literals: BTreeMap::new(),
            result,
            body: None,
            validity: None,
            envelopes: vec![],
            applicability: vec![],
            prerequisites: vec![],
            admissions: BTreeMap::new(),
            reduction_admission: Some(obligation),
            reduction: Some(crate::FiniteReduction {
                kind,
                domain,
                prototype: contract,
            }),
            reduction_selection: Some(selection),
            physical_operation: None,
            output_boundary: None,
            body_inputs,
            validity_reads: Default::default(),
            external: None,
            continuity: None,
        }
        .seal()
    }
}
