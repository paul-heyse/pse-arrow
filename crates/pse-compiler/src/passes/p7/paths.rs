// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::{Realizer, invalid};
use crate::CompilerError;
use pse_ids::SemanticId;
use pse_quantity::{DomainId, QuantityTypeId};
use pse_relations::generated::{compiled, inferred};
use pse_schema::model::ExpressionPathSegmentKind;
use pse_templates::{
    BindingValue, GroupBinding, InstantiationEnvironment, PathBinding, identity,
    paths::{Coordinate, PathInventory, PathRequest, ResolvedMember},
};
use std::collections::BTreeMap;

impl Realizer<'_> {
    pub(super) fn bind_paths(
        &mut self,
        instance: &inferred::instances::Row,
        source: SemanticId,
        env: &mut InstantiationEnvironment,
    ) -> Result<(), CompilerError> {
        let paths = self
            .inventory
            .paths
            .iter()
            .filter(|path| path.source_id == source)
            .cloned()
            .collect::<Vec<_>>();
        for path in paths {
            self.cancel.checkpoint()?;
            let segments = path
                .segments
                .iter()
                .map(|segment| {
                    let kind = ExpressionPathSegmentKind::ALL
                        .into_iter()
                        .find(|kind| kind.as_str() == segment.kind.as_str())
                        .ok_or_else(|| invalid("source path step kind differs"))?;
                    Ok((kind, segment.name.as_str(), segment.index_count))
                })
                .collect::<Result<Vec<_>, CompilerError>>()?;
            let targets = self
                .inventory
                .targets
                .iter()
                .filter(|row| {
                    row.requester_instance_id == instance.instance_id
                        && row.source_id == source
                        && row.path_id == path.path_id
                })
                .cloned()
                .collect::<Vec<_>>();
            if targets.is_empty() {
                return Err(invalid(
                    "demanded source path has no actual P5 target binding",
                ));
            }
            let mut members = BTreeMap::new();
            let mut axes = None;
            let mut scalar_type = None;
            for target in &targets {
                let coordinates = target
                    .path_index
                    .iter()
                    .copied()
                    .map(Coordinate::Member)
                    .collect::<Vec<_>>();
                let resolved = pse_templates::paths::resolve(
                    self.path_inventory()?,
                    PathRequest {
                        requester: instance.instance_id,
                        root_instance: path.root_instance_id,
                        segments: &segments,
                        coordinates: &coordinates,
                    },
                    self.cancel,
                )?;
                if resolved.owner != target.owner_instance_id
                    || resolved.path_index != target.path_index
                    || resolved.path_domains != target.path_domains
                {
                    return Err(invalid(
                        "P5 path target owner/index differs from actual path traversal",
                    ));
                }
                let ResolvedMember::Symbol { declaration, index } = resolved.member else {
                    self.bind_configuration_path(
                        env,
                        source,
                        path.path_id,
                        target,
                        &resolved.member,
                    )?;
                    continue;
                };
                if target.target_kind.as_str() != "symbol"
                    || target.symbol_decl_id != Some(declaration)
                    || target.name.is_some()
                    || target.source_index != index
                {
                    return Err(invalid(
                        "P5 path target declaration differs from actual selected member",
                    ));
                }
                let symbol = self
                    .symbols
                    .get(&(target.owner_instance_id, declaration, index))
                    .ok_or_else(|| {
                        invalid("path target symbol was not realized from its actual declaration")
                    })?;
                let current = self
                    .physical
                    .quantity_type(QuantityTypeId::from_id(symbol.quantity_type_id))?;
                if scalar_type.as_ref().is_some_and(|key| key != &current.key) {
                    return Err(invalid(
                        "collected path members have different complete physical types",
                    ));
                }
                scalar_type = Some(current.key.clone());
                if axes
                    .as_ref()
                    .is_some_and(|prior| prior != &resolved.path_domains)
                {
                    return Err(invalid(
                        "collected path axes vary between selected actual instances",
                    ));
                }
                axes = Some(resolved.path_domains);
                if members
                    .insert(target.path_index.clone(), symbol.symbol_id)
                    .is_some()
                {
                    return Err(invalid("duplicate collected source path tuple"));
                }
            }
            if members.is_empty() {
                continue;
            }
            let factors = axes.ok_or_else(|| invalid("collected path domain factors absent"))?;
            if factors.is_empty() {
                let symbols = members.values().copied().collect::<Vec<_>>();
                let [symbol] = symbols.as_slice() else {
                    return Err(invalid("scalar path has multiple actual targets"));
                };
                env.paths.insert(
                    (source, path.path_id),
                    PathBinding::Value(BindingValue::Symbol(*symbol)),
                );
            } else {
                let axes = factors
                    .into_iter()
                    .map(DomainId::from_id)
                    .collect::<Vec<_>>();
                let product = self.product(&axes)?;
                let mut key =
                    scalar_type.ok_or_else(|| invalid("path physical contract absent"))?;
                key.shape = axes
                    .iter()
                    .map(|domain| {
                        self.domain_facts
                            .get(domain)
                            .map(|facts| facts.kind)
                            .ok_or_else(|| invalid("path domain facts absent"))
                    })
                    .collect::<Result<_, _>>()?;
                let quantity = self.physical.resolve_key(&key)?;
                let group = GroupBinding {
                    group: identity::group_collection_id(
                        instance.instance_id,
                        source,
                        path.path_id,
                    ),
                    axes,
                    members,
                };
                self.emit_collection(
                    instance.instance_id,
                    source,
                    path.path_id,
                    product,
                    quantity.as_id(),
                    &group,
                    &targets,
                )?;
                env.paths
                    .insert((source, path.path_id), PathBinding::Group(group));
            }
        }
        Ok(())
    }
    fn path_inventory(&self) -> Result<PathInventory<'_>, CompilerError> {
        Ok(PathInventory {
            instances: &self.inventory.instances,
            prospective: &self.inventory.prospective,
            semantic_id_type: self
                .registry
                .logical_type(
                    &pse_schema::model::FieldContract::id()
                        .type_name()
                        .map_err(|e| invalid(e.to_string()))?,
                )
                .ok_or_else(|| invalid("SemanticId declaration absent"))?
                .id,
            submodels: &self.inventory.submodels,
            domains: &self.inventory.domain_bindings,
            members: &self.inventory.members,
            products: &self.inventory.products,
            valid_tuples: &self.inventory.tuples,
            symbols: &self.inventory.symbols,
            parameters: &self.inventory.parameters,
            features: &self.inventory.features,
            ports: &self.inventory.ports,
            configuration: &self.inventory.configuration,
        })
    }
    fn emit_collection(
        &mut self,
        instance: SemanticId,
        source: SemanticId,
        path: u64,
        product: SemanticId,
        quantity: SemanticId,
        group: &GroupBinding,
        targets: &[inferred::path_targets::Row],
    ) -> Result<(), CompilerError> {
        self.append(
            "compiled.symbol_groups",
            compiled::symbol_groups::Row {
                group_id: group.group,
                owner_instance_id: instance,
                name: format!("source:{}:{path}", source.to_hex()),
                product_id: product,
            },
        )?;
        self.append(
            "compiled.group_collections",
            compiled::group_collections::Row {
                group_id: group.group,
                root_instance_id: instance,
                source_id: source,
                path_id: path,
                product_id: product,
                quantity_type_id: quantity,
                derivation_id: Self::derivation(instance, source, "collection"),
            },
        )?;
        for target in targets {
            let symbol = *group
                .members
                .get(&target.path_index)
                .ok_or_else(|| invalid("collection target inventory differs"))?;
            self.append(
                "compiled.symbol_group_members",
                compiled::symbol_group_members::Row {
                    group_id: group.group,
                    tuple: target.path_index.clone(),
                    symbol_id: symbol,
                },
            )?;
            self.append(
                "compiled.group_collection_members",
                compiled::group_collection_members::Row {
                    group_id: group.group,
                    index: target.path_index.clone(),
                    symbol_id: symbol,
                    owner_instance_id: target.owner_instance_id,
                    symbol_decl_id: target
                        .symbol_decl_id
                        .ok_or_else(|| invalid("collection member declaration absent"))?,
                    source_index: target.source_index.clone(),
                    derivation_id: Self::derivation(instance, source, "collection_member"),
                },
            )?;
        }
        Ok(())
    }
    fn bind_configuration_path(
        &self,
        env: &mut InstantiationEnvironment,
        source: SemanticId,
        path: u64,
        target: &inferred::path_targets::Row,
        member: &ResolvedMember,
    ) -> Result<(), CompilerError> {
        let (name, category) = match member {
            ResolvedMember::Parameter(name) => (name, "parameter"),
            ResolvedMember::Feature(name) => (name, "feature"),
            ResolvedMember::Port(_) => {
                return Err(invalid(
                    "an arithmetic port path needs an exact declared port-member mapping",
                ));
            }
            ResolvedMember::Symbol { .. } => {
                return Err(invalid("symbol path was sent to configuration binding"));
            }
        };
        if target.target_kind.as_str() != category
            || target.name.as_ref() != Some(name)
            || target.symbol_decl_id.is_some()
            || !target.source_index.is_empty()
            || !target.path_index.is_empty()
        {
            return Err(invalid(
                "configuration path target differs from its actual composite member",
            ));
        }
        let instance = self
            .inventory
            .instances
            .iter()
            .find(|row| row.instance_id == target.owner_instance_id)
            .ok_or_else(|| invalid("configuration owner absent"))?;
        let actual = self.environment(instance, source)?;
        let kind = if category == "parameter" {
            pse_schema::math::TemplateValueKind::Parameter
        } else {
            pse_schema::math::TemplateValueKind::Feature
        };
        let value = actual
            .values
            .get(&pse_mathir::ValueRef::Template {
                template_id: instance.template_id,
                kind,
                name: name.clone(),
            })
            .ok_or_else(|| invalid("path selected a non-arithmetic configuration value"))?
            .clone();
        if env
            .paths
            .insert((source, path), PathBinding::Value(value))
            .is_some()
        {
            return Err(invalid("configuration path has multiple actual owners"));
        }
        Ok(())
    }
}
