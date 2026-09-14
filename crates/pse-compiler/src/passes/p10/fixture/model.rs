// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete model fixture loaded through the actual source/identity authority.
use super::{arithmetic_inputs, invalid, physical_rows};
use crate::CompilerError;
use pse_authoring::document::{DocumentBundle, load_package_texts};
use pse_relations::RecordBatch;
use pse_schema::{
    Registry,
    model::{RelationKey, SnapshotClass},
};
use std::collections::BTreeMap;

/// Parse the explicit physical package through the generated document contracts.
/// Entity registration, source spans and integrity values come from the actual loader.
/// # Errors
/// Missing fixture declarations, generated-row conversion or authoring admission failure.
pub fn physical_package(registry: &Registry) -> Result<DocumentBundle, CompilerError> {
    let rows = physical_rows(registry)?;
    let document = registry
        .documents()
        .iter()
        .find(|document| document.name == "materials")
        .ok_or_else(|| invalid("materials document undeclared"))?;
    let mut sections = serde_json::Map::new();
    for section in &document.sections {
        let spec = registry
            .relation(section.relation)
            .ok_or_else(|| invalid("fixture material relation undeclared"))?;
        let Some(batch) = rows.get(&spec.key).filter(|batch| batch.num_rows() != 0) else {
            continue;
        };
        let cells = pse_relations::cells::cells_from_batch(registry, spec, batch)?;
        macro_rules! typed {
            ($module:ident) => {{
                let values = cells
                    .into_iter()
                    .map(pse_relations::generated::reference::$module::Row::from_cells)
                    .collect::<Result<Vec<_>, _>>()?;
                serde_json::to_value(values)
                    .map_err(|error| invalid(format!("fixture JSON: {error}")))?
            }};
        }
        let values = match spec.key.name {
            "units" => typed!(units),
            "unit_sets" => typed!(unit_sets),
            "quantity_kinds" => typed!(quantity_kinds),
            "quantity_types" => typed!(quantity_types),
            _ => {
                return Err(invalid(
                    "nonempty fixture relation lacks its explicit generated document binding",
                ));
            }
        };
        sections.insert(section.key.to_owned(), values);
    }
    let package = pse_ids::SemanticId::from_bytes([90; 16]);
    let header = format!(
        "[package]\npackage_id = \"{package}\"\nname = \"p10_fixture\"\nversion = \"1.0.0\"\nkind = \"model\"\nid_policy = \"explicit\"\ndependencies = []\ndoc = \"Complete physical P10 predecessor fixture\"\n"
    );
    let text = serde_json::to_string_pretty(&sections)
        .map_err(|error| invalid(format!("fixture JSON: {error}")))?;
    Ok(load_package_texts(
        BTreeMap::from([
            ("package.toml".to_owned(), header),
            ("materials/physical.yaml".to_owned(), text),
        ]),
        registry,
        pse_authoring::ParseBudget::default(),
    )?)
}

/// Every Model-class relation, including actual registry rows and parsed physical source.
/// # Errors
/// Source/registry/value admission failure or an undeclared P10 fixture context.
pub fn model_inputs(
    registry: &Registry,
) -> Result<BTreeMap<RelationKey, RecordBatch>, CompilerError> {
    let mut rows = pse_relations::registry_relations::materialize(registry)?;
    let physical = physical_package(registry)?;
    for (id, values) in physical.rows {
        let spec = registry
            .relation_by_id(id)
            .ok_or_else(|| invalid("loaded fixture relation undeclared"))?;
        rows.insert(
            spec.key,
            pse_relations::cells::batch_from_cells(registry, spec, &values)?,
        );
    }
    let (actual, _) = arithmetic_inputs(registry)?;
    let context = registry
        .relation("reference.p10_fixture_context")
        .ok_or_else(|| invalid("P10 fixture context undeclared"))?;
    rows.insert(context.key, actual[&context.key].clone());
    for spec in registry
        .relations()
        .iter()
        .filter(|spec| spec.snapshot_class == SnapshotClass::Model)
    {
        if let std::collections::btree_map::Entry::Vacant(entry) = rows.entry(spec.key) {
            entry.insert(pse_relations::cells::batch_from_cells(registry, spec, &[])?);
        }
    }
    rows.retain(|key, _| {
        registry
            .relation(&key.qualified_name())
            .is_some_and(|spec| spec.snapshot_class == SnapshotClass::Model)
    });
    Ok(rows)
}
