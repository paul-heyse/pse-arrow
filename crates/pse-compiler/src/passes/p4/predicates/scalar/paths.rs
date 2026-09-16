// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::{CompilerError, Evaluation, Keyed, SemanticId, Value, invalid, inventory};
use pse_relations::generated::inferred;
use pse_templates::paths::{Coordinate, PathInventory, PathRequest, ResolvedMember};

impl Evaluation<'_, '_> {
    pub(super) fn path(
        &mut self,
        source: SemanticId,
        path: u64,
        values: &[Value],
    ) -> Result<Value, CompilerError> {
        if values.iter().any(|value| matches!(value, Value::Unknown)) {
            return Ok(Value::Unknown);
        }
        let coordinates = values
            .iter()
            .map(|value| match value {
                Value::Id(id) => Ok(Coordinate::Member(*id)),
                Value::Integer(value) => Ok(Coordinate::Integer(
                    i64::try_from(*value).map_err(|_| invalid("path coordinate exceeds Int64"))?,
                )),
                Value::Real {
                    value, unit: None, ..
                } => pse_quantity::numeric::exact_i64_from_f64(*value)
                    .map(Coordinate::Integer)
                    .ok_or_else(|| invalid("path coordinate is not exact integer")),
                _ => Err(invalid(
                    "path coordinate is not an actual member or integer",
                )),
            })
            .collect::<Result<Vec<_>, CompilerError>>()?;
        let path_row = inventory::unique(
            &self.inventory.paths,
            |row| row.source_id == source && row.path_id == path,
            "predicate path",
        )?;
        self.support.insert(self.inventory.origin(path_row)?);
        let path = &path_row.row;
        let prospective = typed_rows(&self.inventory.instances);
        let instances = prospective
            .iter()
            .map(|row| inferred::instances::Row {
                instance_id: row.instance_id,
                parent_instance_id: row.parent_instance_id,
                template_id: row.template_id,
                path: row.path.clone(),
                index: row.index.clone(),
                derivation_id: row.derivation_id,
            })
            .collect::<Vec<_>>();
        let domains = typed_rows(&self.inventory.domain_bindings);
        let members = typed_rows(&self.inventory.members);
        let products = typed_rows(&self.inventory.products);
        let valid_tuples = self
            .inventory
            .tuples
            .iter()
            .map(|row| inferred::valid_index_tuples::Row {
                product_id: row.row.product_id,
                tuple: row.row.tuple.clone(),
                derivation_id: row.row.derivation_id,
            })
            .collect::<Vec<_>>();
        let submodels = typed_rows(&self.inventory.submodels);
        let symbols = typed_rows(&self.inventory.symbols);
        let parameters = typed_rows(&self.inventory.parameters);
        let features = typed_rows(&self.inventory.features);
        let ports = typed_rows(&self.inventory.ports);
        let configuration = typed_rows(&self.inventory.configuration);
        let segments = path
            .segments
            .iter()
            .map(|segment| {
                let kind = pse_schema::model::ExpressionPathSegmentKind::ALL
                    .into_iter()
                    .find(|kind| kind.as_str() == segment.kind.as_str())
                    .ok_or_else(|| invalid("path segment kind is undeclared"))?;
                Ok((kind, segment.name.as_str(), segment.index_count))
            })
            .collect::<Result<Vec<_>, CompilerError>>()?;
        let semantic_id_type = self
            .inventory
            .registry
            .logical_type("semantic_id")
            .ok_or_else(|| invalid("SemanticId logical type absent"))?
            .id;
        let target = pse_templates::paths::resolve(
            PathInventory {
                instances: &instances,
                semantic_id_type,
                prospective: &prospective,
                submodels: &submodels,
                domains: &domains,
                members: &members,
                products: &products,
                valid_tuples: &valid_tuples,
                symbols: &symbols,
                parameters: &parameters,
                features: &features,
                ports: &ports,
                configuration: &configuration,
            },
            PathRequest {
                requester: self.instance.instance_id,
                root_instance: path.root_instance_id,
                segments: &segments,
                coordinates: &coordinates,
            },
            self.cancel,
        )?;
        self.support.insert(
            self.inventory
                .origin(self.inventory.instance(target.owner)?)?,
        );
        match target.member {
            ResolvedMember::Parameter(name) => self.config(target.owner, "parameter", &name),
            ResolvedMember::Feature(name) => self.config(target.owner, "feature", &name),
            ResolvedMember::Symbol { .. } | ResolvedMember::Port(_) => Ok(Value::Unknown),
        }
    }
}
fn typed_rows<T: Clone>(rows: &[Keyed<T>]) -> Vec<T> {
    rows.iter().map(|row| row.row.clone()).collect()
}
