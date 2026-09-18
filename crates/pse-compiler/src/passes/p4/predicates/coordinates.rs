// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Shared source-coordinate evaluation over retained native typed arguments.
use super::{
    Inventory, invalid,
    inventory::{Inputs, Support},
    scalar::{Evaluation, Value},
};
use crate::{CompilerError, mathir_relations::domain::DomainValue};
use pse_catalog::session::SnapshotSession;
use pse_ids::{CancellationToken, MemoryReserver, ReservationLease, SemanticId};
use pse_mathir::{NodeId, relations::LoadedMath};
use pse_schema::Registry;
use pse_templates::paths::Coordinate;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

/// One admitted source graph and the reservation retained by every algorithm view.
#[derive(Debug)]
pub(crate) struct SourceGraph {
    math: LoadedMath,
    _lease: Arc<ReservationLease>,
}
impl std::ops::Deref for SourceGraph {
    type Target = LoadedMath;
    fn deref(&self) -> &Self::Target {
        &self.math
    }
}

/// Actual native argument owners and the single normalized graph algorithm.
pub struct IndexEvaluator<'a> {
    pub(crate) inventory: Inventory<'a>,
    graphs: BTreeMap<String, Arc<SourceGraph>>,
    cancel: &'a CancellationToken,
}
impl std::fmt::Debug for IndexEvaluator<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IndexEvaluator")
            .field("families", &self.graphs.len())
            .finish_non_exhaustive()
    }
}
impl<'a> IndexEvaluator<'a> {
    /// Bind complete checked inputs through actual native projections once.
    /// # Errors
    /// Missing source/physical context, malformed graphs or exhausted resources.
    pub async fn new(
        session: &SnapshotSession,
        inputs: &Inputs,
        registry: &'a Registry,
        reserver: &dyn MemoryReserver,
        cancel: &'a CancellationToken,
        physical: &'a crate::quantity_relations::PhysicalInventory,
    ) -> Result<Self, CompilerError> {
        let inventory =
            Inventory::load(inputs, registry, session, reserver, cancel, physical).await?;
        let mut graphs = BTreeMap::new();
        for source in &inventory.sources {
            cancel.checkpoint()?;
            let family = source.row.family.as_str();
            if !graphs.contains_key(family) {
                let relation_source = crate::mathir_relations::RelationSource::from_checked(
                    inputs,
                    registry,
                    crate::mathir_relations::SourceFamily::Normalized { prefix: family },
                )?;
                let extent =
                    relation_source
                        .batches()
                        .values()
                        .try_fold(0usize, |sum, input| {
                            sum.checked_add(pse_ids::validation_extent(input.batch())?)
                                .ok_or_else(|| invalid("index graph extent overflow"))
                        })?;
                let mut memory = reserver.open("semantic:source-graph");
                memory
                    .try_grow(
                        extent
                            .checked_mul(8)
                            .ok_or_else(|| invalid("index graph extent overflow"))?,
                    )
                    .map_err(pse_ids::CanonError::from)?;
                graphs.insert(
                    family.to_owned(),
                    Arc::new(SourceGraph {
                        math: pse_mathir::relations::load_untyped(&relation_source, &[])?,
                        _lease: ReservationLease::new(memory),
                    }),
                );
            }
        }
        Ok(Self {
            inventory,
            graphs,
            cancel,
        })
    }
    pub(crate) fn shared_graphs(&self) -> BTreeMap<String, Arc<SourceGraph>> {
        self.graphs.clone()
    }
    /// Resolve a binder even when its finite domain has no tuples.
    /// # Errors
    /// The owner, binder or domain binding disagrees.
    pub fn binding_domain(
        &self,
        instance: SemanticId,
        source: SemanticId,
        binder: SemanticId,
    ) -> Result<SemanticId, CompilerError> {
        self.cancel.checkpoint()?;
        let owner = self.inventory.owner(instance, source)?;
        let declaration = &self.inventory.binder(source, binder)?.row;
        self.inventory.domain(
            &owner.row,
            &declaration.domain.domain_ref()?,
            &mut Support::new(),
        )
    }
    /// Validate explicit assignments against their actual source domains.
    /// # Errors
    /// An assignment names a foreign binder, domain or member.
    pub fn validate_bindings(
        &self,
        instance: SemanticId,
        source: SemanticId,
        bindings: &BTreeMap<SemanticId, SemanticId>,
    ) -> Result<(), CompilerError> {
        for (binder, member) in bindings {
            let domain = self.binding_domain(instance, source, *binder)?;
            if !self
                .inventory
                .members(domain, &mut Support::new())?
                .iter()
                .any(|row| row.row.member_id == *member)
            {
                return Err(invalid(
                    "source binder assignment is outside its actual domain",
                ));
            }
        }
        Ok(())
    }
    fn graph(&self, source: SemanticId) -> Result<&LoadedMath, CompilerError> {
        self.graphs
            .get(self.inventory.source(source)?.row.family.as_str())
            .map(|graph| &graph.math)
            .ok_or_else(|| invalid("source coordinate graph absent"))
    }
    /// Read one source payload in its original normalized numbering.
    /// # Errors
    /// The source, node or original reference is absent.
    pub fn payload(
        &self,
        source: SemanticId,
        node: NodeId,
    ) -> Result<pse_mathir::Payload, CompilerError> {
        self.cancel.checkpoint()?;
        let graph = self.graph(source)?;
        let mapped = graph
            .node_mapping
            .get(&node)
            .ok_or_else(|| invalid("source coordinate node absent"))?;
        let mut payload = graph.graph.node(*mapped)?.payload.clone();
        payload.map_node_references(|mapped| {
            graph
                .node_mapping
                .iter()
                .find_map(|(original, actual)| (*actual == mapped).then_some(*original))
                .ok_or_else(|| pse_mathir::MathIrError::Malformed {
                    node: Some(mapped),
                    detail: "original source coordinate reference absent".to_owned(),
                })
        })?;
        Ok(payload)
    }
    /// Collect actual free binder identities used by a read and its index expressions.
    /// # Errors
    /// A referenced source/node is absent or cancellation was requested.
    pub fn free_indices(
        &self,
        source: SemanticId,
        node: NodeId,
    ) -> Result<BTreeSet<SemanticId>, CompilerError> {
        self.cancel.checkpoint()?;
        let mut free = BTreeSet::new();
        super::collect_node(self.graph(source)?, node, &mut free)?;
        Ok(free)
    }
    /// Evaluate an original node under exact actual member assignments.
    /// # Errors
    /// Wrong owner, free/out-of-domain indices, unknown configuration, noninteger or physical coordinates.
    pub fn evaluate(
        &self,
        instance: SemanticId,
        source: SemanticId,
        node: NodeId,
        bindings: &BTreeMap<SemanticId, SemanticId>,
    ) -> Result<Coordinate, CompilerError> {
        Ok(self.evaluate_supported(instance, source, node, bindings)?.0)
    }
    pub(crate) fn evaluate_supported(
        &self,
        instance: SemanticId,
        source: SemanticId,
        node: NodeId,
        bindings: &BTreeMap<SemanticId, SemanticId>,
    ) -> Result<(Coordinate, Support), CompilerError> {
        self.cancel.checkpoint()?;
        let owner = self.inventory.owner(instance, source)?;
        let source_row = self.inventory.source(source)?;
        let mut support = BTreeSet::from([
            self.inventory.origin(owner)?,
            self.inventory.origin(source_row)?,
        ]);
        for (id, member) in bindings {
            let binder = self.inventory.binder(source, *id)?;
            let domain = self.inventory.domain(
                &owner.row,
                &binder.row.domain.domain_ref()?,
                &mut support,
            )?;
            let actual = self
                .inventory
                .members(domain, &mut support)?
                .into_iter()
                .find(|row| row.row.member_id == *member)
                .ok_or_else(|| invalid("index assignment is outside its actual bound domain"))?;
            support.insert(self.inventory.origin(binder)?);
            support.insert(self.inventory.origin(actual)?);
        }
        let mut evaluation = Evaluation {
            inventory: &self.inventory,
            graph: self.graph(source)?,
            instance: &owner.row,
            source_id: source,
            bindings,
            support: &mut support,
            cancel: self.cancel,
        };
        let coordinate = match evaluation.expression(node)? {
            Value::Id(id) => Coordinate::Member(id),
            Value::Integer(value) => Coordinate::Integer(
                i64::try_from(value)
                    .map_err(|_| invalid("index coordinate exceeds exact Int64"))?,
            ),
            Value::Real {
                value, unit: None, ..
            } => Coordinate::Integer(
                pse_quantity::numeric::exact_i64_from_f64(value)
                    .ok_or_else(|| invalid("index expression is not an exact finite integer"))?,
            ),
            Value::Unknown => {
                return Err(invalid(
                    "index expression has unresolved configuration or free indices",
                ));
            }
            _ => {
                return Err(invalid(
                    "index expression is not a member identity or dimensionless integer",
                ));
            }
        };
        Ok((coordinate, support))
    }
}
