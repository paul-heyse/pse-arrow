// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Generic actual instance realization, retaining indexed equation environments.
mod contributions;
mod coordinates;
mod declarations;
mod environment;
mod equations;
mod evidence;
mod expressions;
mod input_graph;
mod inventory;
mod kernels;
mod masks;
mod methods;
mod outputs;
mod parameters;
mod paths;
mod promises;
mod roots;
mod transfers;

use super::native_outputs::{OutputRows, Sources};
use crate::AlgorithmOutput;
use crate::{AlgorithmContext, AlgorithmInputs, CompilerError};
use pse_catalog::session::SnapshotSession;
use pse_ids::{CancellationToken, MemoryReserver, Reservation, SemanticId};
use pse_mathir::{ExprGraph, NodeId, index::DomainFacts, relations::vec_sink::KernelBinding};
use pse_quantity::{DomainId, DomainKind, QuantityRegistry, UnitId};
use pse_relations::columnar::{FieldCheckedBatch, RelationRow};
use pse_relations::{
    RecordBatch,
    generated::{compiled, normalized},
};
use pse_schema::{
    Registry,
    model::{AlgorithmSpec, RelationKey},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

type Inputs = BTreeMap<RelationKey, FieldCheckedBatch>;
type Support = (RelationKey, usize);
pub(crate) use kernels::KernelMethod;

/// Complete realized rows and their actual typed source derivations.
#[derive(Debug)]
pub(crate) struct RealizationOutput {
    /// Every declared output, including empty relations.
    pub rows: Inputs,
    /// Source-row evidence for every emitted relation row.
    pub derivations: Vec<RecordBatch>,
    pub sources: Sources,
}

type SymbolKey = (SemanticId, SemanticId, Vec<SemanticId>);

/// P7 binds generic declarations under each actual selected instance context.
#[derive(Debug)]
pub struct P7 {
    spec: AlgorithmSpec,
}
impl P7 {
    /// Bind the exact declared P7 implementation specification.
    /// # Errors
    /// P7 has not been declared in the selected registry.
    pub fn new(registry: &Registry) -> Result<Self, CompilerError> {
        Ok(Self {
            spec: registry
                .algorithm("P7@1")
                .ok_or_else(|| invalid("P7 specification absent"))?
                .clone(),
        })
    }
}
impl crate::Algorithm for P7 {
    fn spec(&self) -> &AlgorithmSpec {
        &self.spec
    }
    fn run<'a>(
        &'a self,
        ctx: &'a AlgorithmContext<'a>,
        inputs: &'a AlgorithmInputs,
    ) -> pse_catalog::provider::BoxFut<'a, Result<AlgorithmOutput, CompilerError>> {
        Box::pin(async move {
            inputs.validate(&self.spec, ctx.registry)?;
            let rows = inputs.checked_rows(ctx.registry)?;
            let sources = Sources::from_inputs(inputs, ctx.registry)?;
            let output = realize(&rows, &sources, ctx, &self.spec, ctx.session, None).await?;
            let ports = self
                .spec
                .outputs
                .iter()
                .map(|port| {
                    let spec = ctx
                        .registry
                        .relation(&port.relation)
                        .ok_or_else(|| invalid("P7 output declaration absent"))?;
                    let batch = output
                        .rows
                        .get(&spec.key)
                        .ok_or_else(|| invalid("P7 output inventory incomplete"))?;
                    Ok((port.port.clone(), batch.clone()))
                })
                .collect::<Result<_, CompilerError>>()?;
            Ok(AlgorithmOutput {
                outputs: ports,
                findings: Vec::new(),
                derivations: output.derivations,
                plans: Vec::new(),
            })
        })
    }
}

/// A selected method computation extends the current graph using these actual inputs.
pub(crate) struct SelectedRealization<'a> {
    pub instances: &'a BTreeSet<SemanticId>,
    pub kernels: &'a [KernelMethod],
}

/// Realize the current bound input through the enclosing invocation's owned context.
pub(crate) fn realize<'a>(
    inputs: &'a Inputs,
    sources: &'a Sources,
    ctx: &'a AlgorithmContext<'a>,
    spec: &'a AlgorithmSpec,
    session: &'a SnapshotSession,
    selected: Option<SelectedRealization<'a>>,
) -> pse_catalog::BoxFut<'a, Result<RealizationOutput, CompilerError>> {
    Box::pin(async move {
        let mut state = Box::pin(prepare(
            inputs,
            sources,
            ctx,
            spec,
            session,
            selected.as_ref().map(|selection| selection.instances),
        ))
        .await?;
        if selected.is_some() {
            state.load_input_graph().await?;
        }
        state.declare_symbols()?;
        if spec.name == "P9" {
            state.bind_method_parameters()?;
        }
        state.expressions()?;
        if let Some(selection) = selected {
            state.kernel_methods(selection.kernels)?;
        }
        if spec.name == "P9" {
            state.method_outputs()?;
        }
        state.validate_promises()?;
        Box::pin(state.finish()).await
    })
}

async fn prepare<'a>(
    inputs: &'a Inputs,
    sources: &Sources,
    ctx: &'a AlgorithmContext<'a>,
    spec: &'a AlgorithmSpec,
    session: &'a SnapshotSession,
    selected: Option<&BTreeSet<SemanticId>>,
) -> Result<Realizer<'a>, CompilerError> {
    let registry = ctx.registry;
    let cancel = ctx.cancel;
    let reserver = ctx.reserver;
    let physical_inventory = ctx.physical()?;
    cancel.checkpoint()?;
    if !registry
        .algorithms()
        .iter()
        .any(|declared| declared == spec)
    {
        return Err(invalid(
            "realization requires the complete exact registered producer declaration",
        ));
    }
    let mut work = reserver.open("P7:realization");
    let extent = inputs.values().try_fold(0_usize, |total, batch| {
        total
            .checked_add(pse_ids::validation_extent(batch.batch())?)
            .ok_or_else(|| invalid("P7 input extent overflow"))
    })?;
    work.try_grow(
        extent
            .checked_mul(4)
            .ok_or_else(|| invalid("P7 decode extent overflow"))?,
    )
    .map_err(|_| resource())?;
    let inventory = inventory::Inventory::load(inputs, registry, session, reserver, cancel).await?;
    let physical = physical_inventory.quantities();
    let domain_facts = domains(&inventory)?;
    let coordinate_evaluator = super::p4::predicates::IndexEvaluator::new(
        session,
        inputs,
        registry,
        reserver,
        cancel,
        physical_inventory,
    )
    .await?;
    let families = coordinate_evaluator.shared_graphs();
    let family_extents = family_extents(registry, inputs)?;
    let output = OutputRows::new(registry, reserver, cancel)?;
    let selected = selected.cloned().unwrap_or_else(|| {
        inventory
            .instances
            .iter()
            .map(|row| row.instance_id)
            .collect()
    });
    if selected.iter().any(|id| {
        inventory
            .instances
            .iter()
            .filter(|row| row.instance_id == *id)
            .count()
            != 1
    }) {
        return Err(invalid(
            "selected realization instance is not one exact actual instance",
        ));
    }
    Ok(Realizer {
        registry,
        cancel,
        inputs,
        reserver,
        coordinate_evaluator: Some(coordinate_evaluator),
        work,
        selected,
        pass: spec.id,
        sources: sources.clone(),
        session: session.clone(),
        spec,
        active_support: BTreeSet::new(),
        symbol_support: BTreeMap::new(),
        node_support: BTreeMap::new(),
        equation_support: BTreeMap::new(),
        kernel_support: BTreeMap::new(),
        family_extents,
        inventory,
        physical,
        boolean_kind: physical_inventory.boolean(),
        domain_facts,
        families,
        graph: ExprGraph::new(),
        symbols: BTreeMap::new(),
        parameter_literals: BTreeMap::new(),
        groups: BTreeMap::new(),
        kernels: BTreeMap::new(),
        equations: Vec::new(),
        selections: Vec::new(),
        output,
    })
}

fn family_extents(
    registry: &Registry,
    inputs: &Inputs,
) -> Result<BTreeMap<&'static str, usize>, CompilerError> {
    let mut family_extents = BTreeMap::new();
    for prefix in ["template", "instance", "display", "contribution", "guard"] {
        let family_extent = registry
            .relations()
            .iter()
            .filter(|spec| spec.key.namespace == pse_schema::model::Namespace::Compiled)
            .filter_map(|spec| pse_schema::catalog::expr_family::target_name(prefix, spec.key.name))
            .try_fold(0_usize, |sum, name| {
                let spec = registry
                    .relation(&format!("normalized.{name}"))
                    .ok_or_else(|| invalid("family extent declaration absent"))?;
                let batch = inputs
                    .get(&spec.key)
                    .ok_or_else(|| invalid("family extent input absent"))?;
                sum.checked_add(pse_ids::validation_extent(batch.batch())?)
                    .ok_or_else(|| invalid("family extent overflow"))
            })?;
        family_extents.insert(prefix, family_extent);
    }
    Ok(family_extents)
}

struct Realizer<'a> {
    registry: &'a Registry,
    cancel: &'a CancellationToken,
    inputs: &'a Inputs,
    reserver: &'a dyn MemoryReserver,
    coordinate_evaluator: Option<super::p4::predicates::IndexEvaluator<'a>>,
    work: Box<dyn Reservation>,
    family_extents: BTreeMap<&'static str, usize>,
    selected: BTreeSet<SemanticId>,
    pass: SemanticId,
    sources: Sources,
    session: SnapshotSession,
    spec: &'a AlgorithmSpec,
    active_support: BTreeSet<Support>,
    symbol_support: BTreeMap<SemanticId, BTreeSet<Support>>,
    node_support: BTreeMap<NodeId, BTreeSet<Support>>,
    equation_support: BTreeMap<SemanticId, BTreeSet<Support>>,
    kernel_support: BTreeMap<SemanticId, BTreeSet<Support>>,
    inventory: inventory::Inventory,
    physical: &'a QuantityRegistry,
    boolean_kind: Option<pse_quantity::QuantityKindId>,
    domain_facts: BTreeMap<DomainId, DomainFacts>,
    families: BTreeMap<String, Arc<super::p4::predicates::SourceGraph>>,
    graph: ExprGraph,
    symbols: BTreeMap<SymbolKey, compiled::symbols::Row>,
    parameter_literals: BTreeMap<SemanticId, pse_templates::BindingValue>,
    groups: BTreeMap<(SemanticId, SemanticId), pse_templates::GroupBinding>,
    kernels: BTreeMap<SemanticId, KernelBinding>,
    equations: Vec<pse_mathir::equation::EquationRecord>,
    selections: Vec<pse_mathir::relations::vec_sink::QuantitySelection>,
    output: OutputRows<'a>,
}
impl Realizer<'_> {
    fn append<T: RelationRow>(&mut self, name: &str, row: T) -> Result<(), CompilerError> {
        if T::relation(self.registry)?.key.qualified_name() != name {
            return Err(invalid("typed output differs from its declared relation"));
        }
        let sources = self.source_keys(&self.active_support)?;
        self.output.push(row, &sources)
    }
    fn derivation(instance: SemanticId, source: SemanticId, role: &str) -> SemanticId {
        pse_ids::named_id(instance, &format!("pse:P7:v1:{role}:{}", source.to_hex()))
    }
    fn source(
        &self,
        relation: SemanticId,
        key: &str,
        id: SemanticId,
        field: &str,
    ) -> Result<normalized::expression_sources::Row, CompilerError> {
        let spec = self
            .registry
            .relation_by_id(relation)
            .filter(|spec| spec.primary_key == [key])
            .ok_or_else(|| {
                invalid("expression source lookup requires its complete identity key")
            })?;
        let token = self
            .inventory
            .source_index
            .key(spec.id, id)
            .ok_or_else(|| invalid("expression source declaration absent"))?;
        let matches = self
            .inventory
            .sources
            .iter()
            .filter(|source| {
                source.source_relation_id == relation
                    && source.source_key == token
                    && source.field_path.rsplit('/').next() == Some(field)
            })
            .collect::<Vec<_>>();
        let [source] = matches.as_slice() else {
            return Err(invalid(
                "exact expression source key/field missing or ambiguous",
            ));
        };
        Ok((*source).clone())
    }
}

fn domains(
    inventory: &inventory::Inventory,
) -> Result<BTreeMap<DomainId, DomainFacts>, CompilerError> {
    inventory
        .domains
        .iter()
        .map(|row| {
            let mut members = inventory
                .members
                .iter()
                .filter(|member| member.domain_id == row.domain_id)
                .collect::<Vec<_>>();
            members.sort_by_key(|member| member.ordinal);
            if members
                .windows(2)
                .any(|pair| pair[0].ordinal == pair[1].ordinal)
            {
                return Err(invalid("domain member ordinal repeated"));
            }
            Ok((
                DomainId::from_id(row.domain_id),
                DomainFacts {
                    kind: DomainKind::parse(row.kind.as_str())
                        .ok_or_else(|| invalid("domain kind unknown"))?,
                    continuous: row.continuous,
                    unit: row.unit_id.map(UnitId::from_id),
                    members: members.into_iter().map(|member| member.member_id).collect(),
                },
            ))
        })
        .collect()
}
fn invalid(detail: impl Into<String>) -> CompilerError {
    pse_templates::TemplateError::Binding {
        instance: SemanticId::NIL,
        detail: detail.into(),
    }
    .into()
}
fn resource() -> CompilerError {
    CompilerError::ResourceLimit {
        consumer: "P7 realization workspace".to_owned(),
        config_keys: vec!["memory_limit".to_owned()],
    }
}
