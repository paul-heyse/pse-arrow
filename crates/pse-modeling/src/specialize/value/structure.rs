// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One binding contract for instantiation and structural definition projection.
use super::{Environment, Evaluator, Value, conforms};
use crate::{Result, Selected, Type, invalid};
use pse_ids::SemanticId;
use pse_model::generated::enums::ModelingDeclarationKind as Kind;
use std::collections::BTreeSet;

impl Evaluator<'_, '_> {
    pub(crate) fn definition_environment(
        &self,
        definition: SemanticId,
        arguments: &Environment,
        mut env: Environment,
    ) -> Result<(SemanticId, Environment)> {
        if self.stack.len() >= 64 || self.stack.contains(&definition) {
            return Err(invalid(
                definition,
                "recursive structural definition projection",
            ));
        }
        let row = &self.package.declarations[&definition];
        let mut stack = self.stack.clone();
        stack.push(definition);
        let evaluate = |at, env: &Environment, source: &str, expected: Option<&Type>| {
            Evaluator {
                package: self.package,
                physical: self.physical,
                at,
                env,
                limit: self.limit,
                stack: stack.clone(),
            }
            .text(source, expected)
        };
        if row.value.kind == Kind::Preset {
            let source = row
                .value
                .binding
                .as_ref()
                .and_then(|b| b.expression.as_deref())
                .ok_or_else(|| invalid(definition, "preset requires application"))?;
            let Value::Definition { id, mut bindings } =
                evaluate(definition, arguments, source, None)?
            else {
                return Err(invalid(definition, "preset target"));
            };
            bindings.extend(arguments.clone());
            return Evaluator {
                package: self.package,
                physical: self.physical,
                at: definition,
                env: &env,
                limit: self.limit,
                stack,
            }
            .definition_environment(id, &bindings, env.clone());
        }
        let contract = match row
            .value
            .selected()
            .map_err(|e| invalid(definition, e.to_string()))?
        {
            Selected::Definition(c)
            | Selected::Test(c)
            | Selected::Case(c)
            | Selected::Implicit(c)
            | Selected::Interface(c) => c,
            _ => return Err(invalid(definition, "structural definition required")),
        };
        let members = self.package.members.get(&definition);
        for name in members.into_iter().flat_map(|m| m.keys()) {
            env.remove(name);
        }
        let facts = env
            .iter()
            .filter(|(n, _)| n.starts_with("analysis.") || n.starts_with("stage."))
            .map(|(n, v)| (n.clone(), v.clone()))
            .collect::<Environment>();
        let names = self.package.named_types(definition);
        let vars = BTreeSet::new();
        let declared = contract
            .parameters
            .iter()
            .map(|p| p.name.as_str())
            .collect::<BTreeSet<_>>();
        for (name, value) in arguments {
            if !declared.contains(name.as_str())
                && !name.starts_with("analysis.")
                && !name.starts_with("stage.")
            {
                return Err(invalid(definition, format!("unknown parameter {name}")));
            }
            env.insert(name.clone(), value.clone());
        }
        let mut pending = contract
            .parameters
            .iter()
            .filter(|p| !arguments.contains_key(&p.name))
            .collect::<Vec<_>>();
        while !pending.is_empty() {
            let before = pending.len();
            let mut next = Vec::new();
            let mut failure = None;
            for parameter in pending {
                let ty = self
                    .physical
                    .resolve(&parameter.type_name, &vars, &names, definition)?;
                let source = parameter.default_value.as_deref().ok_or_else(|| {
                    invalid(definition, format!("missing argument {}", parameter.name))
                })?;
                match evaluate(definition, &env, source, Some(&ty)) {
                    Ok(value) => {
                        env.insert(parameter.name.clone(), value);
                    }
                    Err(error) => {
                        failure = Some(error);
                        next.push(parameter);
                    }
                }
            }
            if next.len() == before {
                return Err(
                    failure.unwrap_or_else(|| invalid(definition, "recursive parameter defaults"))
                );
            }
            pending = next;
        }
        for parameter in &contract.parameters {
            let ty = self
                .physical
                .resolve(&parameter.type_name, &vars, &names, definition)?;
            if !conforms(&env[&parameter.name], &ty, self.package) {
                return Err(invalid(definition, "binding complete type differs"));
            }
        }
        env.extend(facts);
        let mut pending = members
            .into_iter()
            .flat_map(|m| m.values())
            .copied()
            .filter(|member| {
                self.package.declarations[member].value.kind == Kind::Parameter
                    && !matches!(
                        self.package.types.get(member),
                        Some(Type::Quantity(_) | Type::Function { .. })
                    )
            })
            .collect::<Vec<_>>();
        if pending.len().saturating_add(env.len()) > self.limit {
            return Err(invalid(
                definition,
                "structural binding extent exceeds budget",
            ));
        }
        while !pending.is_empty() {
            let before = pending.len();
            let mut next = Vec::new();
            let mut failure = None;
            for member in pending {
                let row = &self.package.declarations[&member];
                let binding = row
                    .value
                    .binding
                    .as_ref()
                    .ok_or_else(|| invalid(member, "structural parameter binding"))?;
                if !binding.indices.is_empty() || binding.defined_by.is_some() {
                    return Err(invalid(
                        member,
                        "structural parameter requires a scalar explicit value",
                    ));
                }
                let source = binding.expression.as_deref().ok_or_else(|| {
                    invalid(member, "structural parameter implementation missing")
                })?;
                match evaluate(member, &env, source, self.package.types.get(&member)) {
                    Ok(value) => {
                        env.insert(row.name.clone(), value);
                    }
                    Err(error) => {
                        failure = Some(error);
                        next.push(member);
                    }
                }
            }
            if next.len() == before {
                return Err(failure
                    .unwrap_or_else(|| invalid(definition, "recursive structural parameters")));
            }
            pending = next;
        }
        Ok((definition, env))
    }

    pub(super) fn definition_member(
        &self,
        definition: SemanticId,
        bindings: &Environment,
        name: &str,
    ) -> Result<Value> {
        // Only explicit ambient facts travel into another definition. Caller locals
        // cannot fill a missing constructor argument or implementation member.
        let ambient = self
            .env
            .iter()
            .filter(|(n, _)| {
                n.starts_with("analysis.") || n.starts_with("stage.") || n.starts_with("scope.")
            })
            .map(|(n, v)| (n.clone(), v.clone()))
            .collect();
        let (definition, env) = self.definition_environment(definition, bindings, ambient)?;
        let member = self
            .package
            .members
            .get(&definition)
            .and_then(|m| m.get(name))
            .copied()
            .ok_or_else(|| invalid(definition, format!("definition has no member {name}")))?;
        if self.package.declarations[&member].value.kind != Kind::Parameter
            || matches!(
                self.package.types.get(&member),
                Some(Type::Quantity(_) | Type::Function { .. })
            )
        {
            return Err(invalid(
                member,
                "only structural parameters may be projected from a definition",
            ));
        }
        env.get(name)
            .cloned()
            .ok_or_else(|| invalid(member, "structural parameter unresolved"))
    }
}
