// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Physical maps use actual finite arguments and the one selected function constructor.
use super::functions::{ActualGroup, PreparedArgument};
use super::rewrite::RewriteContext;
use super::*;
use crate::resolved::{Carrier, CarrierKind, Source, construction::Rewritten};

impl Engine<'_, '_> {
    pub(super) fn physical_operation_call(
        &mut self,
        rewrite: RewriteContext<'_>,
        at: DeclarationId,
        name: &str,
        args: &[Expr],
        source: Source,
    ) -> Result<Option<Rewritten>> {
        let RewriteContext {
            instance,
            env,
            chain,
        } = rewrite;
        if name != "reconstruct" {
            return Ok(None);
        }
        let (family, arguments) = args.split_first().ok_or_else(|| {
            invalid(
                at,
                "reconstruct requires its family, selected law and physical arguments",
            )
        })?;
        let ExprKind::Path(path) = &family.kind else {
            return Err(invalid(
                at,
                "reconstruct names a declared reconstruction family",
            ));
        };
        let family = self
            .p
            .resolve_segments(at, &path.segments)
            .filter(|id| matches!(self.p.types.get(id), Some(Type::Reconstruction { .. })))
            .ok_or_else(|| invalid(at, "reconstruct names a declared reconstruction family"))?;
        let selected = self.function_call(instance, family, arguments, &[], env, chain)?;
        Ok(Some(self.pair_expression(
            selected.node,
            selected.inspection,
            source,
            None,
        )?))
    }
    /// Prepare an actual original argument once. Generated map coordinates subsequently pass
    /// values directly; they never create synthetic source expressions for interpretation.
    fn map_argument(
        &mut self,
        instance: InstanceId,
        at: DeclarationId,
        arg: &Expr,
        expected: &Type,
        env: &Environment,
        chain: &[DeclarationId],
    ) -> Result<PreparedArgument> {
        let types = self.source_types(at, env)?;
        let ty = crate::expression::infer(arg, &types, self.p, self.c, at, Some(expected))?;
        let source = self.original_expression_source(arg, at)?;
        Ok(match &ty {
            Type::Quantity(_) | Type::RefinedQuantity { .. } => PreparedArgument::Scalar {
                value: self.rewrite(instance, arg, env, chain)?,
                ty,
            },
            Type::Indexed { .. } => {
                let value = match &arg.kind {
                    ExprKind::Path(path) => {
                        self.argument_group(instance, path, env, chain, source)?
                    }
                    ExprKind::NamedCall { name, args } => self.coordinate_group(
                        RewriteContext {
                            instance,
                            env,
                            chain,
                        },
                        at,
                        name,
                        args,
                        source,
                    )?,
                    _ => {
                        return Err(invalid(
                            at,
                            "indexed map argument requires its actual admitted group",
                        ));
                    }
                };
                PreparedArgument::Indexed { value, ty }
            }
            _ => {
                let value = if matches!(ty, Type::Function { .. }) {
                    let ExprKind::Path(path) = &arg.kind else {
                        return Err(invalid(
                            at,
                            "static map function argument names its selected declaration",
                        ));
                    };
                    Value::Function(self.resolve_function(instance, at, path, env)?)
                } else {
                    self.eval_ast_with(at, env, arg, Some(&ty))?
                };
                self.compatible(&value, &ty, at)?;
                PreparedArgument::Static { value, ty }
            }
        })
    }
    /// Preserve declared axis kinds and full actual carrier membership even when the
    /// Cartesian product is empty. Only actual scalar slots enter the numerical graph.
    pub(super) fn coordinate_group(
        &mut self,
        rewrite: RewriteContext<'_>,
        at: DeclarationId,
        name: &Path,
        args: &[Expr],
        mut source: Source,
    ) -> Result<ActualGroup> {
        let RewriteContext {
            instance,
            env,
            chain,
        } = rewrite;
        let slot = self
            .p
            .resolve_segments(at, &name.segments)
            .ok_or_else(|| invalid(at, "indexed call is not a coordinate-map slot"))?;
        let indices = self.p.declarations[&slot]
            .value
            .coordinate_slot
            .as_ref()
            .ok_or_else(|| invalid(at, "indexed call is not a coordinate-map slot"))?
            .indices
            .clone();
        let contract = self.p.functions[&slot].clone();
        if indices.is_empty() || args.len() + indices.len() != contract.arguments.len() {
            return Err(invalid(
                slot,
                "coordinate group supplies exactly its map arguments",
            ));
        }
        let prepared = args
            .iter()
            .zip(&contract.arguments)
            .map(|(arg, (_, ty))| self.map_argument(instance, at, arg, ty, env, chain))
            .collect::<Result<Vec<_>>>()?;
        let mut domains = Vec::with_capacity(indices.len());
        for (axis, index) in indices.iter().enumerate() {
            let position = contract
                .arguments
                .iter()
                .position(|(name, _)| name == &index.domain)
                .ok_or_else(|| invalid(slot, "coordinate slot domain is not a map argument"))?;
            let Some(PreparedArgument::Static {
                value: Value::Set(values),
                ty: Type::Set(element),
            }) = prepared.get(position)
            else {
                return Err(invalid(
                    slot,
                    "coordinate slot domain is not an actual typed finite set",
                ));
            };
            if contract.arguments[args.len() + axis].1 != **element {
                return Err(invalid(
                    slot,
                    "coordinate slot index type differs from its actual carrier",
                ));
            }
            domains.push((
                axis,
                vec![],
                Carrier {
                    kind: CarrierKind::OrderedSet,
                    element: (**element).clone(),
                    members: values.clone().into(),
                },
            ));
        }
        let mut coordinates = vec![Vec::new()];
        for (_, _, domain) in &domains {
            let needed = coordinates
                .len()
                .checked_mul(domain.members.len())
                .ok_or_else(|| ModelingError::Budget("coordinate map expansion".into()))?;
            if needed > self.limits.members {
                return Err(ModelingError::Budget("coordinate map members".into()));
            }
            coordinates = coordinates
                .into_iter()
                .flat_map(|coordinate| {
                    domain.members.iter().cloned().map(move |value| {
                        let mut tuple = coordinate.clone();
                        tuple.push(value);
                        tuple
                    })
                })
                .collect();
        }
        source.coordinate = None;
        let mut members = Vec::with_capacity(coordinates.len());
        for coordinate in coordinates {
            self.checkpoint()?;
            let mut actual = prepared.clone();
            for (axis, value) in coordinate.iter().enumerate() {
                actual.push(PreparedArgument::Static {
                    value: value.clone(),
                    ty: domains[axis].2.element.clone(),
                });
            }
            let value = self.function_call_prepared(instance, slot, &actual, source.clone())?;
            members.push((coordinate, value));
        }
        Ok(ActualGroup { members, domains })
    }
}
