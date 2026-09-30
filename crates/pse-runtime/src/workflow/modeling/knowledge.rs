// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded lowering of immutable admitted knowledge through registry-owned schemas.
use super::*;
use pse_model::HeapUsage;
use pse_model::generated::enums::ModelingKnowledgeValueKind as Kind;
use pse_model::generated::runtime::modeling_knowledge::Row;
use pse_model::generated::structures::{
    ModelingKnowledgeLineage, ModelingKnowledgeUncertainty, ModelingKnowledgeValueNode as Node,
};
use pse_modeling::{entity::Uncertainty, provenance::Provenance, specialize::Value};
use pse_relations::columnar::{Collection, FieldCheckedBatch, RelationRow};
use std::sync::Arc;

/// A read-only projection retaining the immutable source owner and accounted Arrow batch.
#[derive(Clone, Debug)]
pub struct ModelingKnowledge {
    runtime: Runtime,
    revision: ModelingRevision,
    table: FieldCheckedBatch,
    metadata: BTreeMap<pse_schema::model::RelationKey, FieldCheckedBatch>,
}
impl ModelingKnowledge {
    /// Identity of the exact admitted source, physical environment and data documents.
    pub fn source_revision(&self) -> pse_ids::ContentHash {
        self.revision.identity()
    }
    /// Registry-declared cells, including their origins and typed values.
    pub fn table(&self) -> &FieldCheckedBatch {
        &self.table
    }
    /// Read-only DataFusion scope over `workspace.runtime.modeling_knowledge`.
    pub fn query_session(
        &self,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<pse_engine::session::EngineSession, WorkflowError> {
        let session = self.runtime.sessions.candidate(
            BTreeMap::new(),
            Arc::clone(&self.runtime.registry),
            cancel,
        )?;
        let mut tables = self.metadata.clone();
        tables.insert(
            Row::relation(&self.runtime.registry).map_err(relation)?.key,
            self.table.clone(),
        );
        Ok(session.with_checked_workspace(tables, cancel)?)
    }
}
struct Cell<'a> {
    owner: DeclarationId,
    row: usize,
    slot: &'a str,
    record_kind: Option<DeclarationId>,
    origin: DeclarationId,
    keys: &'a [Value],
    value: &'a Value,
    uncertainty: Option<&'a Uncertainty>,
    provenance: Option<&'a Provenance>,
    test_only: bool,
}
impl ModelingPackage {
    /// Project admitted cells with an optional exact owner selection. Refuse an oversized
    /// projection before owned growth; inspection does not authorize a production read.
    pub fn knowledge(
        &self,
        owner: Option<DeclarationId>,
        maximum_cells: usize,
        maximum_bytes: usize,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<ModelingKnowledge, WorkflowError> {
        if maximum_cells == 0 || maximum_cells > 1_000_000 || maximum_bytes == 0 {
            return Err(contract("bounded knowledge projection policy"));
        }
        let view = self.revision.checked().knowledge();
        let visit = |consumer: &mut dyn FnMut(Cell<'_>) -> Result<(), WorkflowError>| -> Result<(),WorkflowError> {
            let matches = |id| owner.is_none_or(|owner| owner == id);
            for (id, record) in view.records().filter(|(id, _)| matches(*id)) {
                for (name, value) in &record.values {
                    cancel.checkpoint().map_err(pse_relations::RelationError::from).map_err(relation)?;
                    consumer(Cell { owner:id, row:0, slot:name, record_kind:Some(record.kind), origin:record.origin,
                        keys:&[], value, uncertainty:record.uncertainties.get(name),
                        provenance:view.attribute_provenance(id,name), test_only:view.is_test_only(id) })?;
                }
            }
            for (id, typed) in view.constants().filter(|(id, _)| matches(*id)) {
                cancel.checkpoint().map_err(pse_relations::RelationError::from).map_err(relation)?;
                consumer(Cell { owner:id, row:0, slot:"value", record_kind:None, origin:id, keys:&[], value:&typed.value,
                    uncertainty:typed.uncertainty.as_ref(), provenance:view.provenance(id), test_only:view.is_test_only(id) })?;
            }
            for (id, table) in view.tables().filter(|(id, _)| matches(*id)) {
                for (index, (keys,row)) in table.rows.iter().enumerate() {
                    for (position,value) in row.cells.iter().enumerate() {
                        cancel.checkpoint().map_err(pse_relations::RelationError::from).map_err(relation)?;
                        consumer(Cell { owner:id, row:index, slot:table.names.get(position).map_or("value", String::as_str),
                            record_kind:None, origin:row.origin, keys, value, uncertainty:None,
                            provenance:view.provenance(row.origin), test_only:row.test_only })?;
                    }
                }
            }
            Ok(())
        };
        let mut cells = 0usize;
        let mut bytes = self
            .revision
            .declarations()
            .iter()
            .fold(4096usize, |bytes, row| {
                bytes.saturating_add(row.owned_bytes().saturating_mul(4))
            });
        for (name, _) in view.names() {
            bytes = bytes.saturating_add(4 * (name.len() + 96));
        }
        if bytes > maximum_bytes {
            return Err(contract("knowledge metadata exceeds maximum_bytes"));
        }
        visit(&mut |cell| {
            cells = cells.saturating_add(1);
            if cells > maximum_cells {
                return Err(contract("knowledge projection exceeds maximum_cells"));
            }
            bytes = bytes.saturating_add(4 * size_of::<Row>() + 4 * cell.slot.len());
            for value in cell.keys.iter().chain(std::iter::once(cell.value)) {
                bytes = bytes.saturating_add(extent(value, 0)?);
            }
            bytes = bytes.saturating_add(4 * size_of::<Node>());
            if let Some(provenance) = cell.provenance {
                bytes = bytes.saturating_add(
                    provenance
                        .lineage
                        .len()
                        .saturating_mul(4 * size_of::<ModelingKnowledgeLineage>()),
                );
            }
            if bytes > maximum_bytes {
                return Err(contract("knowledge projection exceeds maximum_bytes"));
            }
            Ok(())
        })?;
        if owner.is_some() && cells == 0 {
            return Err(contract("knowledge owner has no admitted cells"));
        }
        let _scratch = self
            .runtime
            .shared
            .math()
            .reserve("modeling:knowledge-projection", bytes)?;
        let pool = self.runtime.shared.pool();
        let mut columns = Collection::new(&self.runtime.registry, &pool, cancel);
        columns.ensure::<Row>().map_err(relation)?;
        visit(&mut |cell| {
            let mut keys = Vec::new();
            let children = cell
                .keys
                .iter()
                .map(|value| node(value, &mut keys, 0))
                .collect::<Result<Vec<_>, _>>()?;
            let mut root = empty(Kind::Tuple);
            root.children = children;
            keys.push(root);
            let mut value = Vec::new();
            node(cell.value, &mut value, 0)?;
            for node in keys.iter_mut().chain(value.iter_mut()) {
                if let Some(quantity) = node.quantity_type_id {
                    node.canonical_unit_id = Some(
                        self.quantities
                            .quantity_type(quantity.into())
                            .map_err(|error| contract(error.to_string()))?
                            .canonical_unit
                            .as_id(),
                    );
                }
            }
            let provenance = cell.provenance;
            columns
                .push(Row {
                    source_revision: self.revision.identity(),
                    owner_id: cell.owner,
                    row_index: i64::try_from(cell.row)
                        .map_err(|_| contract("knowledge row index"))?,
                    slot: cell.slot.into(),
                    record_kind_id: cell.record_kind,
                    origin_id: cell.origin,
                    keys,
                    value,
                    uncertainty: cell.uncertainty.map(|u| ModelingKnowledgeUncertainty {
                        kind: u.kind,
                        magnitude: u.magnitude,
                    }),
                    source_id: provenance.map(|p| p.source),
                    role_enumeration_id: provenance.map(|p| p.role.enumeration),
                    role_member_id: provenance.map(|p| p.role.member),
                    lineage: provenance.map_or_else(Vec::new, |p| {
                        p.lineage
                            .iter()
                            .map(|(kind, id)| ModelingKnowledgeLineage {
                                kind: *kind,
                                target_id: id.as_id(),
                            })
                            .collect()
                    }),
                    test_only: cell.test_only,
                })
                .map_err(relation)
        })?;
        let table = columns
            .finish()
            .map_err(relation)?
            .into_values()
            .next()
            .ok_or_else(|| contract("knowledge relation absent"))?;
        let mut columns = Collection::new(&self.runtime.registry, &pool, cancel);
        use pse_model::generated::runtime::modeling_knowledge_names::Row as Name;
        columns.ensure::<Name>().map_err(relation)?;
        columns.ensure::<Declaration>().map_err(relation)?;
        for (name, declaration) in view.names() {
            cancel
                .checkpoint()
                .map_err(pse_relations::RelationError::from)
                .map_err(relation)?;
            columns
                .push(Name {
                    source_revision: self.revision.identity(),
                    name: name.into(),
                    declaration_id: declaration,
                })
                .map_err(relation)?;
        }
        for row in self.revision.declarations() {
            cancel
                .checkpoint()
                .map_err(pse_relations::RelationError::from)
                .map_err(relation)?;
            columns.push(row.clone()).map_err(relation)?;
        }
        let mut metadata = self.physical.sources.clone();
        metadata.extend(columns.finish().map_err(relation)?);
        Ok(ModelingKnowledge {
            runtime: self.runtime.clone(),
            revision: self.revision.clone(),
            table,
            metadata,
        })
    }
}
fn empty(kind: Kind) -> Node {
    Node {
        kind,
        boolean: None,
        integer: None,
        magnitude: None,
        quantity_type_id: None,
        canonical_unit_id: None,
        reference_id: None,
        type_id: None,
        text: None,
        labels: Vec::new(),
        children: Vec::new(),
    }
}
fn extent(value: &Value, depth: usize) -> Result<usize, WorkflowError> {
    if depth > 64 {
        return Err(contract("knowledge value depth"));
    }
    let mut bytes = 4 * size_of::<Node>() + 4 * value.retained_bytes();
    match value {
        Value::Set(values) | Value::Tuple(values) => {
            for value in values {
                bytes = bytes.saturating_add(extent(value, depth + 1)?);
            }
        }
        Value::Row { fields, .. } => {
            for value in fields.iter() {
                bytes = bytes.saturating_add(extent(value, depth + 1)?);
            }
        }
        Value::Definition { bindings, .. } => {
            for value in bindings.values() {
                bytes = bytes.saturating_add(extent(value, depth + 1)?);
            }
        }
        _ => {}
    }
    Ok(bytes)
}
fn node(value: &Value, arena: &mut Vec<Node>, depth: usize) -> Result<u32, WorkflowError> {
    if depth > 64 {
        return Err(contract("knowledge value depth"));
    }
    let mut result = empty(Kind::Missing);
    match value {
        Value::Missing => {}
        Value::Boolean(value) => {
            result.kind = Kind::Boolean;
            result.boolean = Some(*value);
        }
        Value::Integer(value) => {
            result.kind = Kind::Integer;
            result.integer = Some(*value);
        }
        Value::Number { bits, quantity } => {
            result.kind = Kind::Quantity;
            result.magnitude = Some(f64::from_bits(*bits));
            result.quantity_type_id = Some(quantity.as_id());
        }
        Value::Coordinate { id, bits, quantity } => {
            result.kind = Kind::Coordinate;
            result.reference_id = Some(*id);
            result.magnitude = Some(f64::from_bits(*bits));
            result.quantity_type_id = Some(quantity.as_id());
        }
        Value::Text(value) => {
            result.kind = Kind::Text;
            result.text = Some(value.clone());
        }
        Value::Entity { id, kind } => {
            result.kind = Kind::Entity;
            result.reference_id = Some(id.as_id());
            result.type_id = Some(kind.as_id());
        }
        Value::Enum {
            enumeration,
            member,
        } => {
            result.kind = Kind::Enumeration;
            result.reference_id = Some(*member);
            result.type_id = Some(enumeration.as_id());
        }
        Value::Identifier { scheme, value } => {
            result.kind = Kind::Identifier;
            result.type_id = Some(scheme.as_id());
            result.text = Some(value.clone());
        }
        Value::Definition { id, bindings } => {
            result.kind = Kind::Definition;
            result.reference_id = Some(id.as_id());
            for (label, value) in bindings {
                result.labels.push(label.clone());
                result.children.push(node(value, arena, depth + 1)?);
            }
        }
        Value::Function(id) => {
            result.kind = Kind::Function;
            result.reference_id = Some(id.as_id());
        }
        Value::Row {
            table,
            names,
            fields,
        } => {
            result.kind = Kind::Row;
            result.reference_id = Some(table.as_id());
            result.labels = names.to_vec();
            for value in fields.iter() {
                result.children.push(node(value, arena, depth + 1)?);
            }
        }
        Value::Set(values) | Value::Tuple(values) => {
            result.kind = if matches!(value, Value::Set(_)) {
                Kind::Set
            } else {
                Kind::Tuple
            };
            for value in values {
                result.children.push(node(value, arena, depth + 1)?);
            }
        }
        Value::QuantityType(id) => {
            result.kind = Kind::QuantityType;
            result.reference_id = Some(id.as_id());
        }
        Value::ReferenceState(id) => {
            result.kind = Kind::ReferenceState;
            result.reference_id = Some(id.as_id());
        }
    }
    let index = u32::try_from(arena.len()).map_err(|_| contract("knowledge arena extent"))?;
    arena.push(result);
    Ok(index)
}
