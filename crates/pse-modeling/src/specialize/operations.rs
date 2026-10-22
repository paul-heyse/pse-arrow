// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Derived numerical operations emit their edges alongside independent inspection syntax.
use super::*;
use crate::resolved::{
    Node, NodeIx, Source,
    construction::{Rewritten, RewrittenEquation},
};
impl Engine<'_, '_> {
    pub(super) fn generated_binary(
        &mut self,
        op: BinaryOp,
        lhs: Rewritten,
        rhs: Rewritten,
        source: Source,
    ) -> Result<Rewritten> {
        self.construction.binary(op, lhs, rhs, source)
    }
    pub(super) fn generated_neg(&mut self, value: Rewritten, source: Source) -> Result<Rewritten> {
        self.construction.neg(value, source)
    }
    pub(super) fn generated_sum(
        &mut self,
        values: Vec<Rewritten>,
        source: Source,
    ) -> Result<Rewritten> {
        let mut values = values.into_iter();
        let mut result = values.next().ok_or_else(|| {
            invalid(
                source.field.declaration,
                "empty generated sum lacks its separately admitted neutral element",
            )
        })?;
        for value in values {
            result = self.generated_binary(BinaryOp::Add, result, value, source.clone())?;
        }
        Ok(result)
    }
    pub(super) fn selected_scalar(&mut self, id: SemanticId, source: Source) -> Result<Rewritten> {
        let slot = self.model.symbols.slot(id).ok_or_else(|| {
            invalid(
                source.field.declaration,
                "required actual scalar selection is absent",
            )
        })?;
        self.construction.pair(
            Node::Reference(crate::resolved::Reference::Symbol(slot)),
            symbol_expr(id).kind,
            source,
        )
    }
    pub(super) fn generated_relation(
        lhs: Rewritten,
        sense: EquationSense,
        rhs: Rewritten,
        span: Span,
    ) -> RewrittenEquation {
        RewrittenEquation {
            lhs: lhs.root,
            sense,
            rhs: rhs.root,
            inspection: Equation {
                kind: EquationKind::Relation {
                    lhs: lhs.inspection,
                    sense,
                    rhs: rhs.inspection,
                },
                span,
            },
        }
    }
    pub(super) fn retain_row(
        &mut self,
        id: SemanticId,
        equation: RewrittenEquation,
        lineage: Lineage,
    ) -> Result<()> {
        let relation = equation.relation();
        if self.model.equation_roots.insert(id, relation).is_some() {
            return Err(invalid(
                lineage.declaration,
                "one numerical row has two constructed relation owners",
            ));
        }
        self.model.equations.push(Row {
            id,
            equation: equation.inspection,
            lineage,
        });
        Ok(())
    }
    /// Equality inside the current receiver preserves actual formal/scalar roles.
    /// Standalone alpha-canonical framing alone would identify x and y as the same
    /// one-coordinate graph and is insufficient for interval/nonaxis binding equality.
    pub(super) fn current_roots_identity(&self, roots: &[NodeIx]) -> Result<ContentHash> {
        let mut hash = FramedHasher::new(pse_ids::Frame::ModelingResolvedSyntaxV1);
        self.frame_current_roots(roots, &mut hash)?;
        Ok(hash.finish_hash())
    }
    /// Framing issues the reached role correspondence once. A current derived occurrence
    /// additionally consumes the actual current scalar bindings; a reusable body does not.
    pub(super) fn frame_current_roots(
        &self,
        roots: &[NodeIx],
        hash: &mut FramedHasher,
    ) -> Result<()> {
        let layout = self.current_layout(roots, &[])?;
        hash.hash(&layout.identity).u64(layout.symbols.len() as u64);
        for binding in layout.symbols {
            hash.str(match binding.role {
                crate::derived::LocalRole::Coordinate => "coordinate",
                crate::derived::LocalRole::Parameter => "parameter",
                crate::derived::LocalRole::Member => "member",
            })
            .u64(binding.ordinal as u64)
            .id(&self.current_symbols()[binding.symbol].id);
        }
        Ok(())
    }
}
