// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Finite template recursion consumes native selected declarations and ordered members.
use super::native::Source;
use super::{Configuration, Origins, invalid, native, selected::SelectedRoot, value};
use crate::CompilerError;
use authored::template_domain_bindings::AuthoredTemplateDomainBindingsFieldSourceSelected as BindingSource;
use datafusion::logical_expr::{col, lit};
use pse_ids::{SemanticId, named_id};
use pse_relations::{
    columnar::RelationRow,
    generated::{authored, enums::MethodFamily, normalized, reference},
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub(super) struct Prospective {
    pub(super) row: authored::instances::Row,
    pub(super) origins: Origins,
    submodel_template: Option<SemanticId>,
    submodel_name: Option<String>,
    index: Vec<SemanticId>,
    path: String,
    pub(super) guard: Option<SemanticId>,
}
impl Prospective {
    pub(super) fn binding(&self) -> normalized::instance_bindings::Row {
        normalized::instance_bindings::Row {
            instance_id: self.row.instance_id,
            parent_instance_id: self.row.parent_instance_id,
            template_id: self.row.template_id,
            submodel_template_id: self.submodel_template,
            submodel_name: self.submodel_name.clone(),
            index: self.index.clone(),
            path: self.path.clone(),
            property_package_id: self.row.property_package_id,
            reaction_package_id: self.row.reaction_package_id,
            guard_source_id: None,
            guard_node_id: None,
            derivation_id: named_id(self.row.instance_id, "pass:P3@1"),
        }
    }
}
struct Pending {
    row: authored::instances::Row,
    submodel: Option<authored::template_submodels::Row>,
    index: Vec<SemanticId>,
    chain: Vec<SemanticId>,
    source_relation: SemanticId,
    source_key: pse_ids::ContentHash,
    origins: Origins,
    bindings: BTreeMap<String, super::Value>,
}

pub(super) async fn expand(
    result: &mut Configuration<'_>,
    selected: &[SelectedRoot],
) -> Result<(), CompilerError> {
    let mut queue = root_queue(result, selected).await?;
    let mut paths = BTreeMap::<SemanticId, String>::new();
    let mut packages = PackageBindings::new();
    let mut seen = BTreeSet::new();
    while !queue.is_empty() {
        let before = queue.len();
        let mut progressed = false;
        for _ in 0..before {
            result.cancel.checkpoint()?;
            let mut pending = queue
                .pop_front()
                .ok_or_else(|| invalid("instance queue changed unexpectedly"))?;
            if pending
                .row
                .parent_instance_id
                .is_some_and(|parent| !seen.contains(&parent))
            {
                queue.push_back(pending);
                continue;
            }
            result.arguments.reserve(
                authored::instances::Row::allocation_size(&pending.row)?
                    .checked_mul(4)
                    .and_then(|size| size.checked_add(4096))
                    .ok_or_else(|| invalid("prospective instance extent overflow"))?,
            )?;
            if !seen.insert(pending.row.instance_id) {
                return Err(invalid("generated/authored instance identity collision"));
            }
            configure_context(result, &mut pending, &packages).await?;
            pending.chain.push(pending.row.template_id);
            let path = if let Some(parent) = pending.row.parent_instance_id {
                format!(
                    "{}.{}",
                    paths
                        .get(&parent)
                        .ok_or_else(|| invalid("parent path absent"))?,
                    pending.row.name
                )
            } else {
                pending.row.name.clone()
            };
            paths.insert(pending.row.instance_id, path.clone());
            packages.insert(
                pending.row.instance_id,
                (
                    pending.row.property_package_id,
                    pending.row.reaction_package_id,
                ),
            );
            result.instances.insert(
                pending.row.instance_id,
                Prospective {
                    submodel_template: pending.submodel.as_ref().map(|row| row.template_id),
                    submodel_name: pending.submodel.as_ref().map(|row| row.name.clone()),
                    guard: pending.submodel.as_ref().and_then(|row| row.guard_id),
                    row: pending.row.clone(),
                    origins: pending.origins.clone(),
                    index: pending.index.clone(),
                    path,
                },
            );
            result
                .bind_values(
                    &pending.row,
                    pending.source_relation,
                    &pending.source_key,
                    &pending.bindings,
                )
                .await?;
            bind_domains(result, &pending.row).await?;
            enqueue_children(result, &pending, &mut queue).await?;
            if pending.submodel.is_some()
                || selected
                    .iter()
                    .any(|root| root.instance.instance_id == pending.row.instance_id)
            {
                result
                    .retain_generated(pending.row.clone(), pending.origins.clone())
                    .await?;
            }
            progressed = true;
        }
        if !progressed {
            return Err(invalid(
                "instance containment has a cycle or a missing parent",
            ));
        }
    }
    Ok(())
}

async fn root_queue(
    result: &mut Configuration<'_>,
    selected: &[SelectedRoot],
) -> Result<VecDeque<Pending>, CompilerError> {
    let roots = result.select::<authored::instances::Row>(vec![]).await?;
    let mut queue = VecDeque::new();
    for source in roots {
        let origins = result.origin(&source)?;
        queue.push_back(Pending {
            row: source.row,
            source_key: source.key,
            source_relation: authored::instances::RELATION_ID,
            submodel: None,
            index: vec![],
            chain: vec![],
            origins,
            bindings: BTreeMap::new(),
        });
    }
    for root in selected {
        queue.push_back(Pending {
            row: root.instance.clone(),
            source_key: root.source_key,
            source_relation: root.source_relation,
            submodel: None,
            index: vec![],
            chain: vec![],
            origins: result.original(root.source_relation, &root.source_key)?,
            bindings: BTreeMap::new(),
        });
    }
    Ok(queue)
}

type PackageBindings = BTreeMap<SemanticId, (Option<SemanticId>, Option<SemanticId>)>;

async fn configure_context(
    result: &mut Configuration<'_>,
    pending: &mut Pending,
    packages: &PackageBindings,
) -> Result<(), CompilerError> {
    let template = result
        .one::<authored::templates::Row>("template_id", pending.row.template_id)
        .await?;
    pending.origins.extend(result.origin(&template)?);
    if pending.chain.contains(&pending.row.template_id) {
        return Err(invalid(
            "recursive template expansion has no finite instance universe",
        ));
    }
    if let Some(parent) = pending.row.parent_instance_id {
        pending.origins.extend(result.owner(parent)?);
        let (property, reaction) = packages
            .get(&parent)
            .ok_or_else(|| invalid("prospective parent configuration absent"))?;
        pending.row.property_package_id = pending.row.property_package_id.or(*property);
        pending.row.reaction_package_id = pending.row.reaction_package_id.or(*reaction);
    }
    if pending.row.property_package_id.is_none() {
        let flowsheets = result
            .by_id::<authored::flowsheets::Row>("instance_id", pending.row.instance_id)
            .await?;
        if flowsheets.len() > 1 {
            return Err(invalid("multiple flowsheet package defaults"));
        }
        pending.row.property_package_id = flowsheets
            .first()
            .and_then(|source| source.row.default_property_package_id);
        for flow in &flowsheets {
            pending.origins.extend(result.origin(flow)?);
        }
    }
    Ok(())
}

async fn enqueue_children(
    result: &mut Configuration<'_>,
    parent: &Pending,
    queue: &mut VecDeque<Pending>,
) -> Result<(), CompilerError> {
    let submodels = result
        .by_id::<authored::template_submodels::Row>("template_id", parent.row.template_id)
        .await?;
    for source in submodels {
        let mut origins = result.owner(parent.row.instance_id)?;
        origins.extend(result.origin(&source)?);
        let submodel = &source.row;
        let (template, package, selection) = match (
            submodel.child_template_id,
            submodel.child_from_param.as_ref(),
        ) {
            (Some(id), None) => (id, None, Origins::new()),
            (None, Some(name)) => select_child_template(result, &parent.row, name).await?,
            _ => {
                return Err(invalid(
                    "submodel requires exactly one template selection alternative",
                ));
            }
        };
        origins.extend(selection);
        if parent.chain.contains(&template) {
            return Err(invalid(
                "recursive template expansion rejected before creating a child",
            ));
        }
        let members = child_members(result, &parent.row, submodel, &mut origins).await?;
        for (index, member_origins) in members {
            let mut child_origins = origins.clone();
            child_origins.extend(member_origins);
            result.arguments.reserve(
                4096_usize
                    .checked_add(
                        index
                            .len()
                            .checked_mul(64)
                            .ok_or_else(|| invalid("child index extent overflow"))?,
                    )
                    .ok_or_else(|| invalid("child extent overflow"))?,
            )?;
            let key = format!(
                "pse:submodel:v1:{}:{}:{}",
                submodel.template_id,
                submodel.name,
                index.iter().map(ToString::to_string).collect::<String>()
            );
            let mut row = authored::instances::Row {
                instance_id: named_id(parent.row.instance_id, &key),
                parent_instance_id: Some(parent.row.instance_id),
                template_id: template,
                name: if index.is_empty() {
                    submodel.name.clone()
                } else {
                    format!(
                        "{}[{}]",
                        submodel.name,
                        index.iter().map(ToString::to_string).collect::<String>()
                    )
                },
                param_values: vec![],
                feature_values: vec![],
                property_package_id: package.or(parent.row.property_package_id),
                reaction_package_id: parent.row.reaction_package_id,
                doc: String::new(),
            };
            let (bindings, binding_origins) =
                bind_child(&mut row, &source, &parent.row, result).await?;
            child_origins.extend(binding_origins);
            queue.push_back(Pending {
                row,
                submodel: Some(submodel.clone()),
                index,
                chain: parent.chain.clone(),
                source_relation: authored::template_submodels::RELATION_ID,
                source_key: source.key,
                origins: child_origins,
                bindings,
            });
        }
    }
    Ok(())
}

type IndexedOrigins = Vec<(Vec<SemanticId>, Origins)>;

async fn child_members(
    result: &mut Configuration<'_>,
    parent: &authored::instances::Row,
    submodel: &authored::template_submodels::Row,
    origins: &mut Origins,
) -> Result<IndexedOrigins, CompilerError> {
    Ok(if let Some(name) = &submodel.multiplicity_domain {
        let domain = binding(result, parent.instance_id, name)?;
        origins.extend(
            result
                .domains
                .get(&(parent.instance_id, name.clone()))
                .ok_or_else(|| invalid("multiplicity domain omitted its source correspondence"))?
                .sources
                .clone(),
        );
        let declaration = result
            .one::<authored::domains::Row>("domain_id", domain)
            .await?;
        origins.extend(result.origin(&declaration)?);
        if declaration.row.continuous {
            return Err(invalid(
                "continuous domain cannot enumerate finite child instances",
            ));
        }
        result
            .by_id::<authored::domain_members::Row>("domain_id", domain)
            .await?
            .into_iter()
            .map(|row| Ok((vec![row.row.member_id], result.origin(&row)?)))
            .collect::<Result<Vec<_>, CompilerError>>()?
    } else {
        vec![(vec![], Origins::new())]
    })
}

async fn bind_child(
    child: &mut authored::instances::Row,
    source: &Source<authored::template_submodels::Row>,
    parent: &authored::instances::Row,
    result: &mut Configuration<'_>,
) -> Result<(BTreeMap<String, super::Value>, Origins), CompilerError> {
    let mut origins = Origins::new();
    let mut bindings = BTreeMap::new();
    let mut assigned = BTreeSet::new();
    for (ordinal, assignment) in source.row.bindings.iter().enumerate() {
        if !assigned.insert(&assignment.child_param) {
            return Err(invalid("duplicate child binding"));
        }
        let param = authored::template_params::Row::relation(result.registry)?;
        let params = result
            .select::<authored::template_params::Row>(vec![
                col("template_id").eq(native::identity(
                    result.registry,
                    param,
                    "template_id",
                    child.template_id,
                )?),
                col("name").eq(lit(&assignment.child_param)),
            ])
            .await?;
        let feature = authored::template_features::Row::relation(result.registry)?;
        let features = result
            .select::<authored::template_features::Row>(vec![
                col("template_id").eq(native::identity(
                    result.registry,
                    feature,
                    "template_id",
                    child.template_id,
                )?),
                col("name").eq(lit(&assignment.child_param)),
            ])
            .await?;
        if params.len() + features.len() != 1 {
            return Err(invalid("child binding target missing or ambiguous"));
        }
        for param in &params {
            origins.extend(result.origin(param)?);
        }
        for feature in &features {
            origins.extend(result.origin(feature)?);
        }
        let is_param = !params.is_empty();
        let value = match result.syntax.binding(source, ordinal)? {
            super::syntax::Binding::Parent(name) => {
                match result.values.get(&(parent.instance_id, name.to_owned())) {
                    Some(value) => {
                        origins.extend(value.sources.clone());
                        bindings.insert(assignment.child_param.clone(), value.value.clone());
                        continue;
                    }
                    None if !is_param && name == assignment.child_param => "inherit".to_owned(),
                    None => {
                        return Err(invalid(
                            "child parameter reference has no actual typed value",
                        ));
                    }
                }
            }
            super::syntax::Binding::Literal(value) => value.into_owned(),
        };
        if is_param {
            child
                .param_values
                .push(authored::instances::AuthoredInstancesFieldParamValuesItem {
                    name: assignment.child_param.clone(),
                    value,
                });
        } else {
            child.feature_values.push(
                authored::instances::AuthoredInstancesFieldFeatureValuesItem {
                    name: assignment.child_param.clone(),
                    value,
                },
            );
        }
    }
    Ok((bindings, origins))
}

async fn bind_domains(
    result: &mut Configuration<'_>,
    instance: &authored::instances::Row,
) -> Result<(), CompilerError> {
    let declarations = result
        .by_id::<authored::template_domains::Row>("template_id", instance.template_id)
        .await?;
    let explicit = result
        .by_id::<authored::instance_domain_bindings::Row>("instance_id", instance.instance_id)
        .await?;
    for binding in &explicit {
        let declaration = authored::template_domains::Row::relation(result.registry)?;
        let matches = result
            .select::<authored::template_domains::Row>(vec![
                col("template_id").eq(native::identity(
                    result.registry,
                    declaration,
                    "template_id",
                    instance.template_id,
                )?),
                col("name").eq(lit(&binding.row.domain_name)),
            ])
            .await?;
        if matches.len() != 1 {
            return Err(invalid("instance binds an undeclared template domain"));
        }
    }
    for declaration in declarations {
        let mut origins = result.owner(instance.instance_id)?;
        origins.extend(result.origin(&declaration)?);
        let domain = declaration.row;
        let mut candidates = domain_candidates(result, instance, &domain, &mut origins).await?;
        if candidates.len() != 1 {
            return Err(invalid("template domain has no unique actual binding"));
        }
        let actual = candidates
            .pop_first()
            .ok_or_else(|| invalid("domain candidate disappeared"))?;
        let actual_domain = result
            .one::<authored::domains::Row>("domain_id", actual)
            .await?;
        origins.extend(result.origin(&actual_domain)?);
        if actual_domain.row.kind != domain.kind
            || actual_domain.row.continuous != domain.continuous
        {
            return Err(invalid(
                "bound domain does not match declared kind and continuity",
            ));
        }
        result.domains.insert(
            (instance.instance_id, domain.name.clone()),
            super::Sourced {
                value: actual,
                sources: origins.clone(),
            },
        );
        result.columns.push(
            normalized::instance_domain_bindings::Row {
                instance_id: instance.instance_id,
                domain_name: domain.name,
                domain_id: actual,
            },
            &origins,
        )?;
    }
    Ok(())
}
async fn domain_candidates(
    result: &mut Configuration<'_>,
    instance: &authored::instances::Row,
    domain: &authored::template_domains::Row,
    origins: &mut Origins,
) -> Result<BTreeSet<SemanticId>, CompilerError> {
    let explicit_spec = authored::instance_domain_bindings::Row::relation(result.registry)?;
    let bindings = result
        .select::<authored::instance_domain_bindings::Row>(vec![
            col("instance_id").eq(native::identity(
                result.registry,
                explicit_spec,
                "instance_id",
                instance.instance_id,
            )?),
            col("domain_name").eq(lit(&domain.name)),
        ])
        .await?;
    let mut candidates = BTreeSet::new();
    for binding in bindings {
        origins.extend(result.origin(&binding)?);
        candidates.insert(binding.row.domain_id);
    }
    let source_spec = authored::template_domain_bindings::Row::relation(result.registry)?;
    let sources = result
        .select::<authored::template_domain_bindings::Row>(vec![
            col("template_id").eq(native::identity(
                result.registry,
                source_spec,
                "template_id",
                instance.template_id,
            )?),
            col("name").eq(lit(&domain.name)),
        ])
        .await?;
    for source in &sources {
        origins.extend(result.origin(source)?);
        let actual = match source.row.source.selected()? {
            BindingSource::Domain(value) => value.domain_id,
            BindingSource::Parameter(parameter) => {
                let value = result
                    .values
                    .get(&(instance.instance_id, parameter.name.clone()))
                    .ok_or_else(|| invalid("domain parameter omitted its sources"))?;
                origins.extend(value.sources.clone());
                value
                    .value
                    .semantic_id
                    .as_ref()
                    .map(|arm| arm.value)
                    .ok_or_else(|| invalid("domain parameter lacks actual identity"))?
            }
            BindingSource::Species
            | BindingSource::Phase
            | BindingSource::PhaseSpecies
            | BindingSource::Element => {
                super::material_domains::bind(result, instance, source.row.source.kind, domain.kind)
                    .await?
            }
        };
        candidates.insert(actual);
    }
    if let Some(source) = &domain.members_from {
        if let Some(name) = source.strip_prefix("parent.") {
            let parent = instance
                .parent_instance_id
                .ok_or_else(|| invalid("domain inheritance has no parent"))?;
            candidates.insert(binding(result, parent, name)?);
            origins.extend(
                result
                    .domains
                    .get(&(parent, name.to_owned()))
                    .ok_or_else(|| invalid("inherited domain omitted its sources"))?
                    .sources
                    .clone(),
            );
        } else if let Some(id) = source.strip_prefix("domain:") {
            candidates.insert(value::id(id)?);
        } else if sources.is_empty() {
            return Err(invalid("domain source lacks its typed declaration"));
        }
    }
    Ok(candidates)
}

fn binding(
    result: &Configuration<'_>,
    instance: SemanticId,
    name: &str,
) -> Result<SemanticId, CompilerError> {
    result
        .domains
        .get(&(instance, name.to_owned()))
        .map(|binding| binding.value)
        .ok_or_else(|| invalid("finite instance domain binding missing"))
}
async fn select_child_template(
    result: &mut Configuration<'_>,
    parent: &authored::instances::Row,
    name: &str,
) -> Result<(SemanticId, Option<SemanticId>, Origins), CompilerError> {
    let value = result
        .values
        .get(&(parent.instance_id, name.to_owned()))
        .ok_or_else(|| invalid("selected child parameter omitted its sources"))?;
    let mut origins = value.sources.clone();
    let selected = value
        .value
        .semantic_id
        .as_ref()
        .map(|arm| arm.value)
        .ok_or_else(|| {
            invalid("parameter-selected child requires an actual typed semantic identity")
        })?;
    let spec = authored::template_params::Row::relation(result.registry)?;
    let declarations = result
        .select::<authored::template_params::Row>(vec![
            col("template_id").eq(native::identity(
                result.registry,
                spec,
                "template_id",
                parent.template_id,
            )?),
            col("name").eq(lit(name)),
        ])
        .await?;
    let [declaration] = declarations.as_slice() else {
        return Err(invalid(
            "child selector parameter declaration absent or ambiguous",
        ));
    };
    origins.extend(result.origin(declaration)?);
    if declaration.row.domain_spec.as_deref() != Some("is_property_package") {
        return Ok((selected, None, origins));
    }
    let package = result
        .one::<authored::property_packages::Row>("property_package_id", selected)
        .await?;
    origins.extend(result.origin(&package)?);
    let method = result
        .one::<reference::method_specs::Row>("method_id", package.row.state_definition_method_id)
        .await?;
    origins.extend(result.origin(&method)?);
    let method = method.row;
    let reference::method_specs::ReferenceMethodSpecsFieldRealizationSelected::EquationTemplate(
        producer,
    ) = method.realization.selected()?
    else {
        return Err(invalid(
            "property package state definition requires an equation template",
        ));
    };
    if method.family != MethodFamily::StateDefinition {
        return Err(invalid(
            "property package state definition requires a state method",
        ));
    }
    Ok((producer.template_id, Some(selected), origins))
}
