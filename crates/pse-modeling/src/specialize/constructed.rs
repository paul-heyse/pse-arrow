// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Library-declared primitives construct local typed functions without an AST admission pass.
use super::*;
use crate::resolved::{
    FunctionProduct, Node, NodeIx, PhysicalSupport, Reference, Source, SymbolIx,
    construction::{Construction, Rewritten, RewrittenPredicate},
    operation::{OperationMeaning, OperationRoots},
};
use std::sync::Arc;

/// Exact construction facts for one synthesized, physically checked function.
pub(super) struct ConstructedFunction {
    pub(super) at: DeclarationId,
    pub(super) arguments: Vec<(String, Type)>,
    pub(super) result: Type,
    pub(super) operation: Option<crate::PhysicalOperation>,
    pub(super) actual: Vec<Rewritten>,
    pub(super) source: Source,
    pub(super) closed: OperationRoots,
}
impl Engine<'_, '_> {
    /// The builder receives actual local formal roots and already relocated closed metadata.
    /// Its inspection syntax is built simultaneously and is never parsed or inferred here.
    pub(super) fn constructed_function(
        &mut self,
        request: ConstructedFunction,
        build: impl FnOnce(
            &mut Construction,
            &[Rewritten],
            &[NodeIx],
            &Source,
        ) -> Result<(Rewritten, Option<RewrittenPredicate>)>,
    ) -> Result<Rewritten> {
        let ConstructedFunction {
            at,
            arguments,
            result,
            operation,
            actual,
            source,
            closed,
        } = request;
        if arguments.len() != actual.len() {
            return Err(invalid(
                at,
                "constructed primitive actual/signature arity differs",
            ));
        }
        self.reserve(1)?;
        let mut construction = Construction::new(self.limits.items);
        let body_arguments = operation.as_ref().map_or_else(
            || arguments.clone(),
            |operation| operation.body_arguments(&arguments),
        );
        let mut symbols = Symbols::default();
        let mut formals = Vec::new();
        let mut formal_values = Vec::new();
        let mut outputs = Vec::new();
        let mut origin = source.clone();
        origin.coordinate = None;
        for (ordinal, (name, ty)) in body_arguments.iter().enumerate() {
            let slot = SymbolIx::from(ordinal);
            let mut hash = FramedHasher::new(pse_ids::Frame::ModelingResolvedFunctionV1);
            hash.str("local-formal").u64(ordinal as u64);
            symbols.push(Symbol {
                id: hash.finish_id(),
                ty: ty.clone(),
                role: Kind::Variable,
                domain: Domain::Continuous,
                definition: None,
                expression: None,
                initial: None,
                lineage: Lineage {
                    declaration: at,
                    instance: SemanticId::NIL.into(),
                    path: name.clone(),
                    demand: vec![],
                    default_owner: None,
                    is_override: false,
                    presets: vec![],
                },
            });
            let formal = construction.pair(
                Node::Reference(Reference::Symbol(slot)),
                ExprKind::Path(Path::single(name.clone())),
                origin.clone(),
            )?;
            outputs.push(formal.root);
            formals.push(slot);
            formal_values.push(formal);
        }
        let closed_roots = match &closed {
            OperationRoots::None => vec![],
            OperationRoots::Reference { state, anchors } => state
                .iter()
                .copied()
                .chain(anchors.iter().flatten().copied())
                .collect(),
        };
        let imported = construction.import_closed(&self.construction, &closed_roots)?;
        let meaning_roots = match closed {
            OperationRoots::None => OperationRoots::None,
            OperationRoots::Reference { anchors, .. } => {
                if imported.len() != 2 + 2 * anchors.len() {
                    return Err(invalid(
                        at,
                        "closed contextual metadata import changed root arity",
                    ));
                }
                OperationRoots::Reference {
                    state: [imported[0], imported[1]],
                    anchors: imported[2..].as_chunks::<2>().0.to_vec(),
                }
            }
        };
        let (body, validity) = build(&mut construction, &formal_values, &imported, &origin)?;
        outputs.push(body.root);
        outputs.extend(&imported);
        let physical_operation = operation
            .as_ref()
            .map(|operation| OperationMeaning::capture(operation, meaning_roots, at))
            .transpose()?;
        let (admissions, literals) = construction.take_support();
        let syntax = construction.finish(
            symbols,
            outputs,
            validity.iter().map(|guard| guard.root).collect(),
        )?;
        let (physical, output_boundary, body_inputs) = PhysicalSupport::prepare_function(
            &syntax,
            self.c,
            crate::resolved::FunctionSupportRequest {
                body: Some(body.root),
                result: &result,
                literals: &literals,
                operation: operation.as_ref(),
                at,
                obligations: &admissions,
                reduction: None,
                signature: &arguments
                    .iter()
                    .map(|(_, ty)| ty.clone())
                    .collect::<Vec<_>>(),
                formals: &formals,
                meaning: physical_operation.as_ref(),
            },
        )?;
        for input in body_inputs.iter().flatten() {
            self.model.derivations.publish(input.relationship)?;
        }
        for relationship in physical.result_views() {
            self.model.derivations.publish(relationship)?;
        }
        let product = FunctionProduct {
            identity: ContentHash::from_bytes([0; 32]),
            has_applicability_effects: false,
            declaration: at,
            syntax,
            physical: Arc::new(physical),
            formals,
            signature: arguments.iter().map(|(_, ty)| ty.clone()).collect(),
            arguments: vec![],
            selections: vec![],
            literals,
            result: result.clone(),
            body: Some(body.root),
            validity: validity.as_ref().map(|guard| guard.root),
            envelopes: vec![],
            applicability: vec![],
            prerequisites: vec![],
            admissions,
            reduction_admission: None,
            reduction: None,
            reduction_selection: None,
            physical_operation,
            output_boundary,
            body_inputs,
            validity_reads: Default::default(),
            external: None,
            continuity: None,
        }
        .seal()?;
        let name = if let Some(context) = self.inspection_context {
            format!(
                "f_{}_inspection_{}_{}_{}",
                product.identity.to_hex(),
                at.as_id(),
                context.to_hex(),
                self.model.functions.len()
            )
        } else {
            format!(
                "f_{}_inspection_{}",
                product.identity.to_hex(),
                self.model.functions.len()
            )
        };
        self.model.functions.insert(
            name.clone(),
            crate::Function {
                applicability: vec![],
                applicability_uses: vec![],
                prerequisites: vec![],
                physical_admissions: BTreeMap::new(),
                physical_operation: operation,
                reduction: None,
                validity: validity.map(|guard| guard.inspection),
                envelopes: vec![],
                validity_reads: Default::default(),
                external: None,
                continuity: None,
                id: at,
                variables: BTreeSet::new(),
                arguments,
                result,
                body: Some(body.inspection),
            },
        );
        let function = self.construction.function(product)?;
        let roots = actual.iter().map(|arg| arg.root).collect();
        let args = actual.into_iter().map(|arg| arg.inspection).collect();
        self.construction.pair(
            Node::Apply {
                function,
                args: roots,
            },
            ExprKind::NamedCall {
                name: Path::single(name),
                args,
            },
            source,
        )
    }
}
