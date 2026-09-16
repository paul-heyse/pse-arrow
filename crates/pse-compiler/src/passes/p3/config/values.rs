// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed scalar parsing follows native declaration/default/override selection.
use super::{Configuration, Origins, Value, choices, invalid, native, value};
use crate::CompilerError;
use datafusion::logical_expr::col;
use pse_ids::{SemanticId, named_id};
use pse_relations::{
    columnar::RelationRow,
    generated::{
        authored,
        enums::{ConfigCategory, FeatureKind, TriState},
        normalized,
    },
};

impl Configuration<'_> {
    pub(super) async fn bind_values(
        &mut self,
        instance: &authored::instances::Row,
        source_relation: SemanticId,
        source_key: &pse_ids::ContentHash,
        bindings: &std::collections::BTreeMap<String, Value>,
    ) -> Result<(), CompilerError> {
        self.bind_parameters(instance, source_relation, source_key, bindings)
            .await?;
        self.bind_features(instance, source_relation, source_key, bindings)
            .await
    }

    async fn bind_parameters(
        &mut self,
        instance: &authored::instances::Row,
        source_relation: SemanticId,
        source_key: &pse_ids::ContentHash,
        bindings: &std::collections::BTreeMap<String, Value>,
    ) -> Result<(), CompilerError> {
        let params =
            choices::select::<authored::template_params::Row>(self, instance, "param_values")
                .await?;
        for (parameter, assigned) in params {
            let mut origins = self.owner(instance.instance_id)?;
            origins.extend(self.origin(&parameter)?);
            let row = parameter.row;
            let text = assigned.as_deref().or(row.default.as_deref());
            let binding = bindings.get(&row.name);
            if text.is_none() && binding.is_none() {
                if row.required {
                    return Err(invalid(
                        "required instance parameter has no value or default",
                    ));
                }
                continue;
            }
            let logical = self
                .registry
                .logical_types()
                .iter()
                .find(|logical| logical.id == row.logical_type_id)
                .ok_or_else(|| invalid("parameter logical type is not declared"))?;
            let parsed = if let Some(binding) = binding {
                value::bind(binding, &logical.name, row.enum_id, self.registry)?
            } else {
                value::parse(
                    text.ok_or_else(|| invalid("selected parameter value absent"))?,
                    &logical.name,
                    row.enum_id,
                    self.registry,
                    Some(self.physical.quantities()),
                )?
            };
            origins.extend(self.constraint(&parsed, row.domain_spec.as_deref()).await?);
            let (relation, key) = if assigned.is_some() || binding.is_some() {
                (source_relation, source_key)
            } else {
                (authored::template_params::RELATION_ID, &parameter.key)
            };
            origins.extend(self.original(relation, key)?);
            self.record(
                instance.instance_id,
                ConfigCategory::Parameter,
                &row.name,
                parsed,
                (relation, key),
                &origins,
            )?;
        }
        Ok(())
    }

    async fn bind_features(
        &mut self,
        instance: &authored::instances::Row,
        source_relation: SemanticId,
        source_key: &pse_ids::ContentHash,
        bindings: &std::collections::BTreeMap<String, Value>,
    ) -> Result<(), CompilerError> {
        let flowsheet = self
            .by_id::<authored::flowsheets::Row>("instance_id", instance.instance_id)
            .await?;
        if flowsheet.len() > 1 {
            return Err(invalid("instance has multiple flowsheet declarations"));
        }
        let features =
            choices::select::<authored::template_features::Row>(self, instance, "feature_values")
                .await?;
        let mut dynamic_origins = Origins::new();
        for (feature, assigned) in features {
            let mut origins = self.owner(instance.instance_id)?;
            origins.extend(self.origin(&feature)?);
            let row = feature.row;
            if row.name == "dynamic" && !flowsheet.is_empty() {
                if row.kind != FeatureKind::Bool {
                    return Err(invalid("flowsheet dynamic feature must be Boolean"));
                }
                if assigned
                    .as_ref()
                    .is_some_and(|actual| actual != flowsheet[0].row.dynamic.as_str())
                {
                    return Err(invalid(
                        "flowsheet dynamic and instance feature assignments conflict",
                    ));
                }
                dynamic_origins.extend(origins);
                continue;
            }
            let text = assigned.as_deref().or(row.default.as_deref());
            let binding = bindings.get(&row.name);
            let (relation, key) = if assigned.is_some() || binding.is_some() {
                (source_relation, source_key)
            } else {
                (authored::template_features::RELATION_ID, &feature.key)
            };
            origins.extend(self.original(relation, key)?);
            let logical = match row.kind {
                FeatureKind::Bool => "bool",
                FeatureKind::Enum | FeatureKind::Choice => "enum",
            };
            let parsed = if let Some(binding) = binding {
                value::bind(binding, logical, row.enum_id, self.registry)?
            } else if text == Some("inherit")
                || text.is_some_and(|text| text.starts_with("= "))
                || (text.is_none() && row.inherit_from.is_some())
            {
                let path = text.and_then(|text| text.strip_prefix("= ")).map_or_else(
                    || {
                        row.inherit_from
                            .clone()
                            .unwrap_or_else(|| format!("parent.{}", row.name))
                    },
                    |name| format!("self.{name}"),
                );
                self.inherit(instance, &row.name, &path, relation, key, &origins)?;
                continue;
            } else if let Some(text) = text {
                value::parse(
                    text,
                    logical,
                    row.enum_id,
                    self.registry,
                    Some(self.physical.quantities()),
                )?
            } else {
                continue;
            };
            self.record(
                instance.instance_id,
                ConfigCategory::Feature,
                &row.name,
                parsed,
                (relation, key),
                &origins,
            )?;
        }
        self.bind_dynamic(instance, flowsheet, &dynamic_origins)
    }

    fn bind_dynamic(
        &mut self,
        instance: &authored::instances::Row,
        flowsheet: Vec<super::Source<authored::flowsheets::Row>>,
        dynamic_origins: &Origins,
    ) -> Result<(), CompilerError> {
        for flow in flowsheet {
            let mut origins = self.owner(instance.instance_id)?;
            origins.extend(self.origin(&flow)?);
            origins.extend(dynamic_origins.iter().cloned());
            if flow.row.dynamic == TriState::Inherit {
                self.inherit(
                    instance,
                    "dynamic",
                    "parent.dynamic",
                    authored::flowsheets::RELATION_ID,
                    &flow.key,
                    &origins,
                )?;
            } else {
                let parsed = Value::from_boolean(
                    normalized::config_values::NormalizedConfigValuesFieldValueBoolean {
                        value: flow.row.dynamic == TriState::True,
                    },
                );
                self.record(
                    instance.instance_id,
                    ConfigCategory::Feature,
                    "dynamic",
                    parsed,
                    (authored::flowsheets::RELATION_ID, &flow.key),
                    &origins,
                )?;
            }
        }
        Ok(())
    }

    fn record(
        &mut self,
        owner: SemanticId,
        category: ConfigCategory,
        name: &str,
        value: Value,
        source: (SemanticId, &pse_ids::ContentHash),
        origins: &Origins,
    ) -> Result<(), CompilerError> {
        let (relation, source_key) = source;
        if self
            .values
            .insert(
                (owner, name.to_owned()),
                super::Sourced {
                    value: value.clone(),
                    sources: origins.clone(),
                },
            )
            .is_some()
        {
            return Err(invalid(
                "parameter and feature names collide in one instance binding",
            ));
        }
        self.columns.push(
            normalized::config_values::Row {
                owner_id: owner,
                category,
                name: name.to_owned(),
                value,
                source_relation_id: relation,
                source_key: source_key.to_owned(),
                derivation_id: named_id(owner, &format!("pse:P3:config:v1:{relation}:{name}")),
            },
            origins,
        )?;
        Ok(())
    }

    fn inherit(
        &mut self,
        instance: &authored::instances::Row,
        name: &str,
        path: &str,
        relation: SemanticId,
        source_key: &pse_ids::ContentHash,
        origins: &Origins,
    ) -> Result<(), CompilerError> {
        let (scope, source_name) = path
            .split_once('.')
            .filter(|(_, name)| !name.is_empty() && !name.contains('.'))
            .ok_or_else(|| invalid("feature inheritance requires parent.name or self.name"))?;
        let source_instance_id = match scope {
            "parent" => instance.parent_instance_id,
            "self" => Some(instance.instance_id),
            _ => return Err(invalid("unknown feature inheritance scope")),
        };
        self.columns.push(
            normalized::feature_inheritance::Row {
                instance_id: instance.instance_id,
                name: name.to_owned(),
                source_instance_id,
                source_name: source_name.to_owned(),
                source_relation_id: relation,
                source_key: source_key.to_owned(),
                derivation_id: named_id(
                    instance.instance_id,
                    &format!("pse:P3:config:v1:{relation}:{name}"),
                ),
            },
            origins,
        )?;
        Ok(())
    }

    async fn constraint(
        &mut self,
        value: &Value,
        spec: Option<&str>,
    ) -> Result<Origins, CompilerError> {
        let mut origins = Origins::new();
        let Some(spec) = spec else {
            return Ok(origins);
        };
        if value::numeric_constraint(value, spec)? {
            return Ok(origins);
        }
        let target = value
            .semantic_id
            .as_ref()
            .map(|arm| arm.value)
            .ok_or_else(|| invalid("identity constraint requires a semantic ID value"))?;
        match spec {
            "is_property_package" => {
                let source = self
                    .one::<authored::property_packages::Row>("property_package_id", target)
                    .await?;
                origins.extend(self.origin(&source)?);
            }
            "is_reaction_package" => {
                let source = self
                    .one::<authored::reaction_packages::Row>("reaction_package_id", target)
                    .await?;
                origins.extend(self.origin(&source)?);
            }
            "is_template" => {
                let source = self
                    .one::<authored::templates::Row>("template_id", target)
                    .await?;
                origins.extend(self.origin(&source)?);
            }
            spec if spec.starts_with("member_of:") => {
                let domain = value::id(&spec[10..])?;
                let declaration = authored::domain_members::Row::relation(self.registry)?;
                let rows = self
                    .select::<authored::domain_members::Row>(vec![
                        col("domain_id").eq(native::identity(
                            self.registry,
                            declaration,
                            "domain_id",
                            domain,
                        )?),
                        col("member_id").eq(native::identity(
                            self.registry,
                            declaration,
                            "member_id",
                            target,
                        )?),
                    ])
                    .await?;
                if rows.len() != 1 {
                    return Err(invalid(
                        "configuration identity is absent from its declared domain",
                    ));
                }
                for source in &rows {
                    origins.extend(self.origin(source)?);
                }
            }
            _ => return Err(invalid("unknown configuration domain validator")),
        }
        Ok(origins)
    }
}
