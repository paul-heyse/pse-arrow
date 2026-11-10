// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded original indexed products captured by the finite source producer.
use super::*;
use crate::resolved::{ContractionAxis, ContractionFactor, OriginalContraction, Source};
impl Engine<'_, '_> {
    /// The initial product is exactly two indexed family occurrences, with one
    /// occurrence of the original dummy in each. Other reductions keep their
    /// ordinary scalar meaning and have no admitted indexed analytical product.
    pub(super) fn original_contraction(
        &self,
        instance: InstanceId,
        at: DeclarationId,
        binder: &dsl::Binder,
        body: &Expr,
        env: &Environment,
        selection: &finite::ReductionSelection,
        prototype: &Type,
        source: &Source,
    ) -> Result<Option<OriginalContraction>> {
        let ExprKind::Binary {
            op: BinaryOp::Mul,
            lhs,
            rhs,
        } = &body.kind
        else {
            return Ok(None);
        };
        let (ExprKind::Path(left), ExprKind::Path(right)) = (&lhs.kind, &rhs.kind) else {
            return Ok(None);
        };
        if selection.selected.is_empty() {
            return Ok(None);
        }
        let mut factors = Vec::new();
        for (expression, path) in [(&**lhs, left), (&**rhs, right)] {
            let [segment] = path.segments.as_slice() else {
                return Ok(None);
            };
            if segment.indices.iter().filter(|index|matches!(&index.kind,ExprKind::Path(path) if path.ident()==Some(binder.var.as_str()))).count()!=1{return Ok(None)}
            // A transformed dummy such as j+1 is outside this contraction profile.
            if segment.indices.iter().any(|index| {
                let mut refers=false;index.walk(|node|{refers|=matches!(&node.kind,ExprKind::Path(path) if path.ident()==Some(binder.var.as_str()));});
                refers&&!matches!(&index.kind,ExprKind::Path(path) if path.ident()==Some(binder.var.as_str()))
            }){return Ok(None)}
            let factor_source = self.original_expression_source(expression, at)?;
            let mut factor = None::<ContractionFactor>;
            for member in selection.selected.iter() {
                let mut local = env.clone();
                local.insert(binder.var.clone(), member.clone());
                let (owner, declaration, coordinates) =
                    self.resolve_path(instance, at, path, &local, true)?;
                if coordinates.len() != segment.indices.len() {
                    return Ok(None);
                }
                let axes=segment.indices.iter().zip(&coordinates).map(|(index,(_,value))|
                    if matches!(&index.kind,ExprKind::Path(path) if path.ident()==Some(binder.var.as_str())){ContractionAxis::Dummy}else{ContractionAxis::Fixed(value.clone())}).collect::<Vec<_>>();
                let id = member_id(owner, declaration, &coordinates);
                if !self.model.symbols.contains_key(&id) {
                    return Err(invalid(
                        at,
                        "selected contraction member has no actual scalar realization",
                    ));
                }
                let factor = factor.get_or_insert_with(|| ContractionFactor {
                    instance: owner,
                    declaration,
                    axes: axes.clone(),
                    components: Vec::new(),
                    source: factor_source.clone(),
                });
                if factor.instance != owner
                    || factor.declaration != declaration
                    || factor.axes != axes
                {
                    return Err(invalid(
                        at,
                        "contraction changes a fixed external family/coordinate across its dummy domain",
                    ));
                }
                factor.components.push(id);
            }
            let Some(factor) = factor else {
                return Ok(None);
            };
            factors.push(factor);
        }
        let factors: [ContractionFactor; 2] = factors
            .try_into()
            .map_err(|_| invalid(at, "contraction factor extent"))?;
        Ok(Some(OriginalContraction {
            source: source.clone(),
            binder_spelling: binder.var.clone(),
            carrier: selection.carrier.clone(),
            selected: selection.selected.clone(),
            factors,
            prototype: prototype.clone(),
            filtered: binder.filter.is_some(),
        }))
    }
}
