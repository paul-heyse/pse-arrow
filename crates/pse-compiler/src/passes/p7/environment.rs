// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::{Realizer, invalid};
use crate::{CompilerError, mathir_relations::domain::DomainValue};
use pse_ids::SemanticId;
use pse_mathir::{DomainRef, Payload, ValueRef};
use pse_quantity::{BoundIndexId, BoundIndexRef, DomainId, QuantityTypeId, UnitId};
use pse_relations::generated::{enums::ConfigCategory, inferred};
use pse_schema::math::TemplateValueKind;
use pse_templates::{BindingValue, InstantiationEnvironment, identity};

// Both relation projections share the same declared alternative; borrow each directly.
macro_rules! configuration_literal {
    ($engine:expr, $instance:expr, $value:expr, $selection:path) => {{
        (|| -> Result<Option<BindingValue>, CompilerError> {
            use $selection as Selected;
            let engine = $engine;
            let instance = $instance;
            let selected = $value.selected()?;
            let quantity = match selected {
                Selected::Signed(_) | Selected::Unsigned(_) | Selected::Real(_) => {
                    Some(engine.neutral_type(instance)?)
                }
                Selected::Boolean(_) => Some(engine.boolean_types_for_instance(instance)?),
                Selected::Quantity(arm) => Some(QuantityTypeId::from_id(arm.quantity_type_id)),
                _ => None,
            };
            let payload = match selected {
                Selected::Signed(arm) => Payload::IntConst { value: arm.value },
                Selected::Unsigned(arm) => Payload::IntConst {
                    value: i64::try_from(arm.value).map_err(|_| {
                        invalid("unsigned parameter exceeds exact mathematical integer range")
                    })?,
                },
                Selected::Boolean(arm) => Payload::IntConst {
                    value: i64::from(arm.value),
                },
                Selected::Quantity(arm) => Payload::FloatConst {
                    value: arm.value,
                    unit: UnitId::from_id(arm.unit_id),
                },
                Selected::Real(arm) => {
                    let quantity = quantity.ok_or_else(|| {
                        invalid(
                            "real arithmetic parameter needs its explicitly selected physical type",
                        )
                    })?;
                    Payload::FloatConst {
                        value: arm.value,
                        unit: engine.physical.quantity_type(quantity)?.canonical_unit,
                    }
                }
                Selected::SemanticId(arm) => return Ok(Some(BindingValue::Member(arm.value))),
                Selected::Text(_) | Selected::Enum(_) | Selected::Index(_) => return Ok(None),
            };
            Ok(Some(BindingValue::Literal { payload, quantity }))
        })()
    }};
}

impl Realizer<'_> {
    #[expect(
        clippy::too_many_lines,
        reason = "environment keeps the native relation inputs and dependency ordered assembly visible in one place"
    )]
    pub(super) fn environment(
        &self,
        instance: &inferred::instances::Row,
        source: SemanticId,
    ) -> Result<InstantiationEnvironment, CompilerError> {
        let mut env = InstantiationEnvironment::new(instance.instance_id);
        env.domain_facts.clone_from(&self.domain_facts);
        for row in self
            .inventory
            .domain_bindings
            .iter()
            .filter(|row| row.instance_id == instance.instance_id)
        {
            env.domains.insert(
                DomainRef::Template {
                    template_id: instance.template_id,
                    domain_name: row.domain_name.clone(),
                },
                DomainId::from_id(row.domain_id),
            );
        }
        for member in &self.inventory.members {
            if let Some(value) = member.coordinate.filter(|value| value.is_finite())
                && let Ok(integer) = value.to_string().parse::<i64>()
                && env
                    .integer_members
                    .insert(
                        (DomainId::from_id(member.domain_id), integer),
                        member.member_id,
                    )
                    .is_some()
            {
                return Err(invalid("ambiguous actual numeric coordinate"));
            }
        }
        for row in self
            .inventory
            .indices
            .iter()
            .filter(|row| row.source_id == source)
        {
            let reference = row.domain.domain_ref()?;
            let domain = match &reference {
                DomainRef::Actual(domain) => *domain,
                DomainRef::Template { .. } => *env.domains.get(&reference).ok_or_else(|| {
                    invalid("source binder template domain is not bound in this instance")
                })?,
            };
            let facts = env
                .domain_facts
                .get(&domain)
                .ok_or_else(|| invalid("source binder domain facts absent"))?;
            let binding_id = if let Some(position) = row.position {
                let declaration = self
                    .inventory
                    .sources
                    .iter()
                    .find(|decl| decl.source_id == source)
                    .ok_or_else(|| invalid("source declaration absent for outer binder"))?;
                let owner = self
                    .inventory
                    .source_index
                    .owner(declaration.source_relation_id, declaration.source_key)
                    .unwrap_or(source);
                identity::free_index_id(instance.instance_id, owner, position)
            } else {
                identity::bound_index_id(instance.instance_id, row.bound_index_id)
            };
            let actual = BoundIndexRef::new(BoundIndexId::from_id(binding_id), domain, facts.kind);
            if env
                .binders
                .insert(BoundIndexId::from_id(row.bound_index_id), actual)
                .is_some()
            {
                return Err(invalid("duplicate source index binding"));
            }
            if row.position.is_some() {
                env.free_indices.insert(actual).map_err(|error| {
                    pse_templates::TemplateError::Binding {
                        instance: instance.instance_id,
                        detail: format!("source {} outer binder: {error:?}", source.to_hex()),
                    }
                })?;
            }
        }
        for ((owner, declaration, index), row) in &self.symbols {
            if *owner == instance.instance_id && index.is_empty() {
                env.values.insert(
                    ValueRef::ActualSymbol(*declaration),
                    self.parameter_literals
                        .get(&row.symbol_id)
                        .cloned()
                        .unwrap_or(BindingValue::Symbol(row.symbol_id)),
                );
            }
        }
        if self
            .registry
            .algorithm("P9@1")
            .is_some_and(|pass| pass.id == self.pass)
        {
            env.unbound_parameters.extend(
                self.symbols
                    .values()
                    .filter(|symbol| {
                        self.selected.contains(&symbol.owner_instance_id)
                            && symbol.role == pse_relations::generated::enums::SymbolRole::Parameter
                            && !self.parameter_literals.contains_key(&symbol.symbol_id)
                    })
                    .map(|symbol| symbol.symbol_id),
            );
        }
        for (symbol, value) in &self.parameter_literals {
            env.values
                .insert(ValueRef::ActualSymbol(*symbol), value.clone());
        }
        for ((owner, declaration), group) in &self.groups {
            if *owner == instance.instance_id {
                env.groups.insert(*declaration, group.clone());
            }
        }
        for row in self.inventory.configuration.iter().filter(|row| {
            row.owner_id == instance.instance_id && row.category == ConfigCategory::Parameter
        }) {
            let declaration = self
                .inventory
                .parameters
                .iter()
                .filter(|decl| decl.template_id == instance.template_id && decl.name == row.name)
                .count();
            if declaration != 1 {
                return Err(invalid("parameter value lacks exact declaration"));
            }
            if let Some(value) = configuration_literal!(self, instance.instance_id, &row.value, pse_relations::generated::normalized::config_values::NormalizedConfigValuesFieldValueSelected)? {
                env.values.insert(
                    ValueRef::Template {
                        template_id: instance.template_id,
                        kind: TemplateValueKind::Parameter,
                        name: row.name.clone(),
                    },
                    value,
                );
            }
        }
        for row in self
            .inventory
            .feature_values
            .iter()
            .filter(|row| row.instance_id == instance.instance_id)
        {
            if let Some(value) = configuration_literal!(self, instance.instance_id, &row.value, pse_relations::generated::inferred::instance_features::InferredInstanceFeaturesFieldValueSelected)? {
                env.values.insert(
                    ValueRef::Template {
                        template_id: instance.template_id,
                        kind: TemplateValueKind::Feature,
                        name: row.name.clone(),
                    },
                    value,
                );
            }
        }
        for row in self
            .inventory
            .predicates
            .iter()
            .filter(|row| row.instance_id == instance.instance_id && row.index.is_empty())
        {
            match row.outcome.as_str() {
                "true" => {
                    env.predicates
                        .insert((row.source_id, row.predicate_id), true);
                }
                "false" => {
                    env.predicates
                        .insert((row.source_id, row.predicate_id), false);
                }
                "unknown" | "conflict" => {}
                _ => return Err(invalid("unknown predicate truth dictionary member")),
            }
        }
        Ok(env)
    }
    fn neutral_type(&self, instance: SemanticId) -> Result<QuantityTypeId, CompilerError> {
        self.inventory
            .instances
            .iter()
            .find(|row| row.instance_id == instance)
            .ok_or_else(|| invalid("numeric parameter instance absent"))?;
        let id = self.physical.neutral_dimensionless().ok_or_else(|| {
            invalid("numeric parameter requires an explicit selected mathematical context")
        })?;
        Ok(id)
    }
    pub(super) fn guard_enabled(
        &self,
        instance: SemanticId,
        guard: Option<SemanticId>,
    ) -> Result<bool, CompilerError> {
        let Some(guard) = guard else {
            return Ok(true);
        };
        let template = self
            .inventory
            .instances
            .iter()
            .find(|row| row.instance_id == instance)
            .ok_or_else(|| invalid("guard owner instance absent"))?
            .template_id;
        let declarations = self
            .inventory
            .guards
            .iter()
            .filter(|row| row.guard_id == guard)
            .collect::<Vec<_>>();
        let [declaration] = declarations.as_slice() else {
            return Err(invalid("guard declaration missing or ambiguous"));
        };
        if declaration.template_id != template {
            return Err(invalid("guard declaration belongs to a different template"));
        }
        let source = self.source(
            pse_relations::generated::authored::template_guards::RELATION_ID,
            "guard_id",
            guard,
            "predicate",
        )?;
        let outcomes = self
            .inventory
            .predicates
            .iter()
            .filter(|row| {
                row.instance_id == instance
                    && row.source_id == source.source_id
                    && row.predicate_id == source.root_id
                    && row.index.is_empty()
            })
            .collect::<Vec<_>>();
        let [outcome] = outcomes.as_slice() else {
            return Err(pse_templates::TemplateError::GuardUndecidable {
                instance,
                source_id: source.source_id,
                predicate: source.root_id,
            }
            .into());
        };
        match outcome.outcome.as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(pse_templates::TemplateError::GuardUndecidable {
                instance,
                source_id: source.source_id,
                predicate: source.root_id,
            }
            .into()),
        }
    }
}
