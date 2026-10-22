// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual finite-reduction selection enters one direct immutable function product.
use super::*;
use crate::resolved::{FunctionProduct, Node, Source, construction::Rewritten};
use crate::specialize::finite::ReductionSelection;
impl Engine<'_, '_> {
    pub(super) fn finish_finite_reduction(
        &mut self,
        kind: dsl::ReduceKind,
        selection: ReductionSelection,
        signature: (Type, Type),
        terms: Vec<Rewritten>,
        source: Source,
        at: DeclarationId,
    ) -> Result<Rewritten> {
        let (prototype, result) = signature;
        let product = FunctionProduct::finite_reduction(
            at,
            kind,
            selection,
            (prototype.clone(), result.clone()),
            terms.len(),
            self.c.quantities,
            self.c.preconditions,
        )?;
        let function = self.construction.function(product.clone())?;
        let name = format!("f_{}", product.identity.to_hex());
        let args = terms.iter().map(|term| term.root).collect();
        let inspection = terms.into_iter().map(|term| term.inspection).collect();
        // This contract is retained exclusively for independent source/scientific inspection.
        self.model
            .functions
            .entry(name.clone())
            .or_insert(crate::Function {
                applicability: vec![],
                applicability_uses: vec![],
                prerequisites: vec![],
                physical_admissions: BTreeMap::new(),
                physical_operation: None,
                reduction: product.reduction.clone(),
                validity: None,
                envelopes: vec![],
                validity_reads: Default::default(),
                external: None,
                continuity: None,
                id: at,
                variables: BTreeSet::new(),
                arguments: product
                    .signature
                    .iter()
                    .enumerate()
                    .map(|(ordinal, ty)| (format!("term_{ordinal}"), ty.clone()))
                    .collect(),
                result,
                body: None,
            });
        self.construction.pair(
            Node::Apply { function, args },
            ExprKind::NamedCall {
                name: Path::single(name),
                args: inspection,
            },
            source,
        )
    }
}
