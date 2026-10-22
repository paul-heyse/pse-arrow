// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! The same direct graph constructor handles actual calls and disposable scientific witnesses.
use super::*;
use pse_kernels::DerivativeOrder;
use pse_modeling::resolved::{FunctionIx, FunctionProduct, OperationMeaning};
use std::{cell::RefCell, rc::Rc};
#[derive(Clone)]
pub(super) enum WitnessPass {
    Potential {
        root: ContentHash,
        selected: pse_modeling::DeclarationId,
        witness: Rc<RefCell<pse_math::typed::ScientificWitness>>,
    },
    Reconstruction {
        root: ContentHash,
        witness: Rc<RefCell<pse_math::typed::ScientificWitness>>,
    },
}
impl WitnessPass {
    fn root(&self) -> ContentHash {
        match self {
            Self::Potential { root, .. } | Self::Reconstruction { root, .. } => *root,
        }
    }
    fn witness(&self) -> &Rc<RefCell<pse_math::typed::ScientificWitness>> {
        match self {
            Self::Potential { witness, .. } | Self::Reconstruction { witness, .. } => witness,
        }
    }
}
/// One selected scientific call; all facts borrow the same actual direct call owner.
struct ScientificCall<'a> {
    root: NodeIx,
    slot: FunctionIx,
    args: &'a [NodeIx],
    product: &'a FunctionProduct,
    arguments: &'a [TypedValue],
    wrt: &'a [usize],
    support: &'a pse_modeling::resolved::CallSupport,
    source: SemanticId,
}
impl<'a> Lower<'a> {
    fn child(
        &self,
        root: NodeIx,
        product: &'a FunctionProduct,
        arguments: &[TypedValue],
    ) -> Result<Self, MathError> {
        if arguments.len() != product.formals.len() {
            return Err(contract("selected function scalar argument arity differs"));
        }
        let layout = product.syntax.canonical_layout().map_err(contract)?;
        let mut path = self.path.clone();
        path.push(
            *self
                .node_ordinals
                .get(&root)
                .ok_or_else(|| contract("function call has no canonical caller occurrence"))?,
        );
        Ok(Self {
            syntax: &product.syntax,
            support: &product.physical,
            providers: self.providers,
            cancelled: self.cancelled,
            inputs: BTreeMap::new(),
            arguments: product
                .formals
                .iter()
                .copied()
                .zip(arguments.iter().cloned())
                .collect(),
            nodes: BTreeMap::new(),
            lexical: BTreeMap::new(),
            active: BTreeSet::new(),
            path,
            node_ordinals: layout
                .nodes
                .into_iter()
                .enumerate()
                .map(|(ordinal, slot)| (slot, ordinal))
                .collect(),
            predicate_ordinals: layout
                .predicates
                .into_iter()
                .enumerate()
                .map(|(ordinal, slot)| (slot, ordinal))
                .collect(),
            occurrences: vec![],
            ranges: product
                .physical
                .checked_ranges(&product.syntax)
                .map_err(contract)?
                .into_iter()
                .map(|range| (range.symbol, range))
                .collect(),
            depth: self.depth,
            limit: self.limit,
            witness: self.witness.clone(),
        })
    }
    pub(super) fn function(
        &mut self,
        root: NodeIx,
        slot: FunctionIx,
        args: &[NodeIx],
        wrt: &[usize],
        builder: &mut BodyBuilder<'_>,
        source: SemanticId,
    ) -> Result<TypedValue, MathError> {
        let syntax = self.syntax;
        let product = &syntax
            .functions
            .get(slot)
            .ok_or_else(|| contract("selected function is outside its direct owner"))?
            .product;
        let support = self
            .support
            .node(root)
            .map_err(contract)?
            .call
            .as_ref()
            .ok_or_else(|| contract("selected call lacks exact physical boundaries"))?
            .clone();
        // A scientific pass verifies the nominal body before differentiation. Its
        // exact selected root may be a Partial, whose retained nominal result and
        // ordinary input boundaries already admit that value view. No other call
        // may reinterpret the original ordered partial coordinates.
        let nominal_witness = wrt.is_empty()
            && self
                .witness
                .as_ref()
                .is_some_and(|pass| pass.root() == product.identity)
            && matches!(syntax.nodes.get(root),Some(Node::Partial {function,args:original,wrt:partial})
                if *function==slot && original==args && partial==&support.wrt);
        if support.function != slot
            || (support.wrt != wrt && !nominal_witness)
            || support.inputs.len() != args.len()
            || product.formals.len() != args.len()
        {
            return Err(contract(format!(
                "selected call differs from its complete prepared signature: function {}/{}; partial {:?}/{wrt:?}; inputs/arguments/formals {}/{}/{}",
                support.function.position(),
                slot.position(),
                support.wrt,
                support.inputs.len(),
                args.len(),
                product.formals.len()
            )));
        }
        self.verify_response(root, slot, args, product, builder, source)?;
        let abstract_root = self
            .witness
            .as_ref()
            .is_some_and(|pass| pass.root() == product.identity);
        let mut nominal = Vec::with_capacity(args.len());
        for (arg, boundary) in args.iter().zip(&support.inputs) {
            let value = self.node(*arg, builder)?;
            let value = builder.contract_boundary_admitted(value, boundary)?;
            let value = if abstract_root {
                builder.scientific_argument(value)?
            } else {
                value
            };
            nominal.push(if !wrt.is_empty() || product.continuity.is_some() {
                builder.independent(value)?
            } else if product.reduction.is_some() {
                value
            } else {
                builder.bind(value)?
            });
        }
        let mut arguments = nominal.clone();
        if product.body_inputs.len() != arguments.len() {
            return Err(contract(
                "selected body-input authorizations differ from actual signature",
            ));
        }
        for (argument, view) in arguments.iter_mut().zip(&product.body_inputs) {
            if let Some(view) = view {
                *argument = builder.authorized_boundary(argument.clone(), &view.boundary)?;
            }
        }
        if let Some(value) = self.abstract_call(
            ScientificCall {
                root,
                slot,
                args,
                product,
                arguments: &nominal,
                wrt,
                support: &support,
                source,
            },
            builder,
        )? {
            return Ok(value);
        }
        let mut local = self.child(root, product, &arguments)?;
        let mut assumptions = Vec::new();
        use pse_model::generated::enums::ModelingValidityLayer as Layer;
        let domains = product
            .validity
            .iter()
            .map(|predicate| {
                (
                    Layer::Form,
                    product.declaration,
                    *predicate,
                    &product.validity_reads,
                )
            })
            .chain(
                product
                    .envelopes
                    .iter()
                    .map(|guard| (Layer::Data, guard.owner, guard.predicate, &guard.reads)),
            )
            .collect::<Vec<_>>();
        for (layer, owner, predicate, reads) in domains {
            let lineage = Arc::new(pse_model::diagnostic::ValidityLineage {
                layer,
                source: owner.as_id(),
                form: Some(product.declaration.as_id()),
                sets: reads.sets.clone(),
                variables: reads.variables.clone(),
                members: vec![],
            });
            let mut guard = local.fork();
            let assumption =
                builder.domain(lineage, |builder| guard.indicator(predicate, builder))?;
            local.occurrences.extend(guard.occurrences);
            assumptions.push(assumption);
        }
        for usage in &product.applicability {
            let mut capture = local.fork();
            let assumption = builder.applicability(Arc::new(usage.node.clone()), |builder| {
                let predicates = usage
                    .predicates
                    .iter()
                    .map(|predicate| capture.indicator(*predicate, builder))
                    .collect::<Result<Vec<_>, _>>()?;
                let inputs = usage
                    .inputs
                    .iter()
                    .map(|root| capture.node(*root, builder))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok((predicates, inputs))
            })?;
            local.occurrences.extend(capture.occurrences);
            assumptions.push(assumption);
        }
        let scope = builder.function_scope();
        let mut value = if product.external.is_some() {
            let join = self.providers.call(product.identity, wrt)?;
            if join.inputs().len() != arguments.len() {
                return Err(contract("actual provider input boundary arity differs"));
            }
            let values = arguments
                .iter()
                .zip(join.inputs())
                .map(|(value, boundary)| {
                    builder.contract_boundary_admitted(value.clone(), boundary)
                })
                .collect::<Result<Vec<_>, _>>()?;
            let outputs = builder.provider_partial_admitted(join.prepared(), &values, source)?;
            builder.contract_boundary_admitted(
                outputs
                    .get(join.output())
                    .cloned()
                    .ok_or_else(|| contract("selected provider output disappeared"))?,
                join.result(),
            )?
        } else {
            let value = if let Some(reduction) = &product.reduction {
                builder.finite_reduce_admitted(
                    reduction.kind,
                    &reduction.prototype,
                    &arguments,
                    product.physical.reduction().map_err(contract)?.clone(),
                    source,
                )?
            } else {
                local.node(
                    product
                        .body
                        .ok_or_else(|| contract("selected pure function has no direct body"))?,
                    builder,
                )?
            };
            if let Some(order) = product.continuity {
                let order = match order {
                    0 => DerivativeOrder::Value,
                    1 => DerivativeOrder::First,
                    2 => DerivativeOrder::Second,
                    _ => return Err(contract("selected piecewise derivative order is invalid")),
                };
                builder.verify_piecewise(scope, &arguments, order)?;
            }
            let mut value = if let Some(boundary) = &product.output_boundary {
                builder.authorized_boundary(value, boundary)?
            } else if product.reduction.is_some() {
                builder.contract_boundary_admitted(
                    value,
                    product.physical.reduction_output().map_err(contract)?,
                )?
            } else {
                let body = product
                    .body
                    .ok_or_else(|| contract("selected function result has no body root"))?;
                let ordinal = product
                    .syntax
                    .outputs
                    .iter()
                    .position(|root| *root == body)
                    .ok_or_else(|| {
                        contract("selected function body lost its result boundary ordinal")
                    })?;
                builder.contract_boundary_admitted(
                    value,
                    product.physical.output(ordinal).map_err(contract)?,
                )?
            };
            if !wrt.is_empty() {
                let variables = wrt
                    .iter()
                    .map(|ordinal| {
                        nominal.get(*ordinal).cloned().ok_or_else(|| {
                            contract("selected partial coordinate is outside its nominal signature")
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                value = builder.partial_admitted(
                    scope,
                    value,
                    &variables,
                    support.partial.clone().ok_or_else(|| {
                        contract("selected pure partial lacks its exact physical admission")
                    })?,
                    source,
                )?;
            }
            value
        };
        if let Some(view) = &support.body_result {
            value = builder.authorized_boundary(value, &view.boundary)?;
        }
        for assumption in &assumptions {
            value = builder.with_assumption(value, assumption);
        }
        for ordinal in &product.prerequisites {
            value = builder.with_prerequisite(
                value,
                arguments
                    .get(*ordinal)
                    .ok_or_else(|| contract("selected prerequisite is outside formal order"))?,
            );
        }
        self.occurrences.extend(local.occurrences);
        builder.bind(value)
    }
    fn verify_response(
        &mut self,
        root: NodeIx,
        slot: FunctionIx,
        args: &[NodeIx],
        product: &FunctionProduct,
        builder: &BodyBuilder<'_>,
        source: SemanticId,
    ) -> Result<(), MathError> {
        if self.witness.is_some() {
            return Ok(());
        }
        let Some(OperationMeaning::Response { potential }) = &product.physical_operation else {
            return Ok(());
        };
        let mut hash = FramedHasher::new(pse_ids::Frame::MathProofReceiptRequestV1);
        hash.str("direct-scientific-response")
            .hash(&product.identity)
            .id(&potential.as_id())
            .hash(
                &self
                    .syntax
                    .select(args.to_vec(), vec![])
                    .identity()
                    .map_err(contract)?,
            );
        self.providers.frame(&mut hash);
        let receipt = pse_math::typed::receipts::proof(hash.finish_hash(), || {
            let mut shadow = builder.scientific_pass()?;
            let witness = Rc::new(RefCell::new(pse_math::typed::ScientificWitness::default()));
            let mut lower = self.fork();
            lower.witness = Some(WitnessPass::Potential {
                root: product.identity,
                selected: *potential,
                witness: witness.clone(),
            });
            let value = lower.function(root, slot, args, &[], &mut shadow, source)?;
            if !shadow.retains_scientific_witness(&value, &witness.borrow())? {
                return Err(contract(
                    "response must retain its selected potential or partials after normalization",
                ));
            }
            Ok(Vec::new())
        })?;
        if !receipt.is_empty() {
            return Err(contract(
                "direct scientific dependency receipt has unexpected payload",
            ));
        }
        Ok(())
    }
    fn abstract_call(
        &mut self,
        call: ScientificCall<'_>,
        builder: &mut BodyBuilder<'_>,
    ) -> Result<Option<TypedValue>, MathError> {
        let ScientificCall {
            root,
            slot,
            args,
            product,
            arguments,
            wrt,
            support,
            source,
        } = call;
        let intercept = match &self.witness {
            Some(WitnessPass::Potential { selected, .. }) => *selected == product.declaration,
            Some(WitnessPass::Reconstruction { .. }) => matches!(
                &product.physical_operation,
                Some(OperationMeaning::Reconstruction { .. })
            ),
            None => false,
        };
        if !intercept {
            return Ok(None);
        }
        if matches!(self.witness, Some(WitnessPass::Potential { .. })) {
            let mut shadow = builder.scientific_pass()?;
            let witness = Rc::new(RefCell::new(pse_math::typed::ScientificWitness::default()));
            let mut lower = self.fork();
            lower.witness = Some(WitnessPass::Reconstruction {
                root: product.identity,
                witness: witness.clone(),
            });
            let value = lower.function(root, slot, args, &[], &mut shadow, source)?;
            if !shadow.retains_scientific_witness(&value, &witness.borrow())? {
                return Err(contract(
                    "selected potential must retain its physical reconstruction after normalization",
                ));
            }
        }
        let witness = self
            .witness
            .as_ref()
            .ok_or_else(|| contract("scientific witness owner disappeared"))?
            .witness()
            .clone();
        let admission =
            if wrt.is_empty() {
                None
            } else {
                Some(support.partial.clone().ok_or_else(|| {
                    contract("scientific partial lacks its exact source admission")
                })?)
            };
        Ok(Some(builder.scientific_placeholder_admitted(
            &mut witness.borrow_mut(),
            product.declaration.as_id(),
            support.nominal_result.clone(),
            arguments,
            (wrt, admission),
            source,
        )?))
    }
}
