// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Physical maps lower by ordinary finite argument expansion and composed functions.
use super::*;

impl Engine<'_, '_> {
    pub(super) fn physical_operation_call(
        &mut self,
        instance: InstanceId,
        at: DeclarationId,
        name: &str,
        args: &[Expr],
        env: &Environment,
        chain: &[DeclarationId],
    ) -> Result<Option<Expr>> {
        if name != "reconstruct" {
            return Ok(None);
        }
        let (family, arguments) = args.split_first().ok_or_else(|| {
            invalid(
                at,
                "reconstruct requires its family, selected law and physical arguments",
            )
        })?;
        let family = self
            .p
            .resolve(at, &dsl::render_expr(family))
            .filter(|id| matches!(self.p.types.get(id), Some(Type::Reconstruction { .. })))
            .ok_or_else(|| invalid(at, "reconstruct names a declared reconstruction family"))?;
        let kind = self.function_call(instance, family, arguments, &[], env, chain)?;
        Ok(Some(Expr {
            kind,
            span: Span::default(),
        }))
    }

    /// Enumerate the declared slot axes from actual finite-set arguments, then use exactly
    /// the scalar slot function for every coordinate. Empty groups retain their signature.
    pub(super) fn coordinate_group(
        &mut self,
        instance: InstanceId,
        at: DeclarationId,
        name: &Path,
        args: &[Expr],
        env: &Environment,
        chain: &[DeclarationId],
    ) -> Result<Vec<(Vec<Value>, Expr)>> {
        let slot = self
            .p
            .resolve_segments(at, &name.segments)
            .ok_or_else(|| invalid(at, "indexed call is not a coordinate-map slot"))?;
        let declaration = self.p.declarations[&slot]
            .value
            .coordinate_slot
            .as_ref()
            .ok_or_else(|| invalid(at, "indexed call is not a coordinate-map slot"))?;
        let contract = self.p.functions[&slot].clone();
        if declaration.indices.is_empty()
            || args.len() + declaration.indices.len() != contract.arguments.len()
        {
            return Err(invalid(
                slot,
                "coordinate group supplies exactly its map arguments",
            ));
        }
        let mut domains = Vec::new();
        for index in &declaration.indices {
            let position = contract
                .arguments
                .iter()
                .position(|(name, _)| name == &index.domain)
                .ok_or_else(|| invalid(slot, "coordinate slot domain is not a map argument"))?;
            let value = self.eval_ast(at, env, &args[position])?;
            let Value::Set(values) = value else {
                return Err(invalid(slot, "coordinate slot domain is not a finite set"));
            };
            domains.push(values);
        }
        let mut coordinates = vec![Vec::new()];
        for domain in domains {
            let needed = coordinates
                .len()
                .checked_mul(domain.len())
                .ok_or_else(|| ModelingError::Budget("coordinate map expansion".into()))?;
            if needed > self.limits.members {
                return Err(ModelingError::Budget("coordinate map members".into()));
            }
            coordinates = coordinates
                .into_iter()
                .flat_map(|coordinate| {
                    domain.iter().cloned().map(move |value| {
                        let mut coordinate = coordinate.clone();
                        coordinate.push(value);
                        coordinate
                    })
                })
                .collect();
        }
        coordinates
            .into_iter()
            .map(|coordinate| {
                let mut actual = args.to_vec();
                let mut lexical = env.clone();
                for (position, value) in coordinate.iter().enumerate() {
                    let name = format!("coordinate_map_index_{position}");
                    lexical.insert(name.clone(), value.clone());
                    actual.push(Expr {
                        kind: ExprKind::Path(Path {
                            segments: vec![PathSegment {
                                name,
                                indices: Vec::new(),
                            }],
                        }),
                        span: Span::default(),
                    });
                }
                let kind = self.function_call(instance, slot, &actual, &[], &lexical, chain)?;
                Ok((
                    coordinate,
                    Expr {
                        kind,
                        span: Span::default(),
                    },
                ))
            })
            .collect()
    }
}
