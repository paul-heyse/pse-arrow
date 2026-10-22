// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Persisted incidence and retained result-evidence graphs. Reachability is
//! structural; quantitative meaning stays in the original scientific Arrow rows.

use super::{Durability, ModelingSolvePreparation, Runtime, WorkflowError, contract, relation};
use petgraph::{
    graph::{DiGraph, NodeIndex},
    visit::{Dfs, Reversed},
};
use pse_columnar::{CancellationToken, MemoryConsumer, MemoryReservation};
use pse_operations::canonical_analyses::{
    ANALYSIS_EDGES, ANALYSIS_NODES, AnalysisEdge, AnalysisNode, analysis_key,
};
#[cfg(all(test, feature = "canonical-tests"))]
use std::time::Duration;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, atomic::AtomicBool},
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum TransferCheckpoint {
    BeforeDispatch,
    BeforeArtifactRead,
    BeforeGraph,
    AfterGraphAck,
}
#[cfg(all(test, feature = "canonical-tests"))]
tokio::task_local! {
    static CANCEL_TRANSFER_AT: TransferCheckpoint;
    static GRAPH_HANDOFF_REPLY_LOSS: bool;
}
fn transfer_checkpoint(
    phase: TransferCheckpoint,
    cancel: &CancellationToken,
) -> Result<(), WorkflowError> {
    #[cfg(all(test, feature = "canonical-tests"))]
    if CANCEL_TRANSFER_AT
        .try_with(|at| *at == phase)
        .unwrap_or(false)
    {
        cancel.cancel();
    }
    #[cfg(not(all(test, feature = "canonical-tests")))]
    let _ = phase;
    cancel
        .checkpoint()
        .map_err(pse_engine::EngineError::from)
        .map_err(Into::into)
}

fn check_analysis_ack(
    pool: &Arc<dyn pse_columnar::MemoryPool>,
    expected: &pse_operations::canonical_analyses::Analysis,
    actual: &pse_operations::canonical_analyses::Analysis,
) -> Result<(), WorkflowError> {
    use pse_model::HeapUsage;
    let scratch = MemoryConsumer::new("canonical:analysis-ack-equality").register(pool);
    scratch
        .try_grow(
            size_of_val(expected)
                .checked_add(expected.heap_bytes())
                .ok_or_else(|| contract("analysis acknowledgement equality extent"))?,
        )
        .map_err(pse_engine::EngineError::from)?;
    let mut normalized = expected.clone();
    normalized.active = true;
    normalized.retiring = false;
    if &normalized != actual {
        return Err(contract(
            "analysis acknowledgement differs from exact original request",
        ));
    }
    Ok(())
}

/// Direction in the graph's explicit source-to-dependent scientific edges.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisDirection {
    /// Objects contributing to the selected roots.
    Upstream,
    /// Objects depending on the selected roots.
    Downstream,
}
/// Exact semantic roots and direction; an empty root set selects the whole
/// bounded method graph. Roots are exact semantic names from that method's nodes.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AnalysisControls {
    /// Exact selected semantic object names.
    pub roots: Vec<String>,
    /// Traverse contributors or dependents, preserving distinct edge meanings.
    pub direction: AnalysisDirection,
}
impl AnalysisControls {
    fn validate(&self) -> Result<(), WorkflowError> {
        if self.roots.len() > 64 || self.roots.iter().any(|root| root.len() > 128) {
            return Err(contract("analysis root bound"));
        }
        Ok(())
    }
}
fn target_kind(kind: pse_model::generated::enums::NumericalTarget) -> &'static str {
    use pse_model::generated::enums::NumericalTarget;
    match kind {
        NumericalTarget::Variable => "variable",
        NumericalTarget::Row => "row",
        NumericalTarget::Objective => "objective",
        NumericalTarget::Observable => "observable",
        NumericalTarget::Closure => "closure",
    }
}
use pse_operations::canonical::MetadataResponse;
use pse_operations::canonical_artifacts::{ArtifactLocator, ArtifactRead};
type Graph = DiGraph<(String, String), (String, Option<ArtifactLocator>)>;
// Four MiB transport, native/typed copies, a <=128 KiB header configuration
// or <=64 bounded graph rows, and Arrow projection scratch. No IPC verifier.
fn vertex(
    graph: &mut Graph,
    nodes: &mut BTreeMap<String, NodeIndex>,
    semantic: String,
    kind: &str,
) -> Result<NodeIndex, WorkflowError> {
    if let Some(index) = nodes.get(&semantic) {
        return Ok(*index);
    }
    if nodes.len() >= ANALYSIS_NODES {
        return Err(contract("analysis node bound exceeded"));
    }
    let index = graph.add_node((semantic.clone(), kind.into()));
    nodes.insert(semantic, index);
    Ok(index)
}
fn edge(
    graph: &mut Graph,
    source: NodeIndex,
    target: NodeIndex,
    kind: &str,
    evidence: Option<ArtifactLocator>,
) -> Result<(), WorkflowError> {
    // Identical context references can qualify many rows at the same scope.
    // Deduplicate only exact edges; different block/row/field locators survive.
    if graph
        .edges_connecting(source, target)
        .any(|existing| existing.weight() == &(kind.into(), evidence.clone()))
    {
        return Ok(());
    }
    if graph.edge_count() >= ANALYSIS_EDGES {
        return Err(contract("analysis edge bound exceeded"));
    }
    graph.add_edge(source, target, (kind.into(), evidence));
    Ok(())
}
fn selected(
    graph: &Graph,
    nodes: &BTreeMap<String, NodeIndex>,
    controls: &AnalysisControls,
) -> Result<BTreeSet<NodeIndex>, WorkflowError> {
    controls.validate()?;
    if controls.roots.is_empty() {
        return Ok(graph.node_indices().collect());
    }
    let mut result = BTreeSet::new();
    for root in &controls.roots {
        let index = *nodes
            .get(root)
            .ok_or_else(|| contract("analysis root is not a selected scientific object"))?;
        let mut visit = Dfs::new(graph, index);
        match controls.direction {
            AnalysisDirection::Downstream => {
                while let Some(node) = visit.next(graph) {
                    result.insert(node);
                }
            }
            AnalysisDirection::Upstream => {
                while let Some(node) = visit.next(Reversed(graph)) {
                    result.insert(node);
                }
            }
        }
    }
    // A selected claim keeps its admitted context without traversing unrelated
    // responses sharing that immutable evidence owner.
    use petgraph::visit::EdgeRef;
    let context = graph
        .edge_references()
        .filter(|edge| {
            result.contains(&edge.target()) && graph[edge.source()].1 == "retained_context"
        })
        .map(|edge| edge.source())
        .collect::<Vec<_>>();
    result.extend(context);
    Ok(result)
}

// A locator belongs to the independently retained artifact, never the producer's
// reclaimed result file. Family is admitted context; validity only qualifies a claim.
#[derive(Clone, Debug)]
pub(super) struct Located<R> {
    pub(super) row: R,
    locator: ArtifactLocator,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ResponseFamily {
    Root,
    Optimal,
}
impl ResponseFamily {
    fn name(self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Optimal => "optimal",
        }
    }
}
#[derive(Default)]
pub(super) struct ResponseEvidence {
    pub(super) contexts: Vec<Located<pse_model::generated::runtime::scientific_contexts::Row>>,
    routes: Vec<Located<pse_model::generated::runtime::route_decisions::Row>>,
    validity: Vec<Located<pse_model::generated::runtime::local_validity::Row>>,
    pub(super) computations: Vec<Located<pse_model::generated::runtime::computation_runs::Row>>,
    pub(super) endpoints: Vec<Located<pse_model::generated::runtime::trajectory_endpoints::Row>>,
    samples: Vec<Located<pse_model::generated::runtime::simulation_samples::Row>>,
    modes: Vec<Located<pse_model::generated::runtime::modeling_trajectory_modes::Row>>,
    events: Vec<Located<pse_model::generated::runtime::simulation_events::Row>>,
    parametric: Vec<Located<pse_model::generated::runtime::parametric_sensitivities::Row>>,
    responses: Vec<Located<pse_model::generated::runtime::response_sensitivities::Row>>,
    // Other interpretation rows (including original scaling) remain addressable.
    interpretation: Vec<ArtifactLocator>,
}
fn field_locator(locator: &ArtifactLocator, field: &str) -> ArtifactLocator {
    let mut locator = locator.clone();
    locator.field = Some(field.into());
    locator
}
fn located_rows<R: pse_relations::columnar::RelationRow>(
    batch: &pse_relations::columnar::FieldCheckedBatch,
    artifact: &str,
    block: u64,
) -> Result<Vec<Located<R>>, WorkflowError> {
    Ok(R::rows(batch)
        .map_err(relation)?
        .into_iter()
        .enumerate()
        .map(|(row, value)| Located {
            row: value,
            locator: ArtifactLocator {
                artifact: artifact.into(),
                block,
                row: Some(row as u64),
                field: None,
            },
        })
        .collect())
}
impl ResponseEvidence {
    fn push(
        &mut self,
        batch: &pse_relations::columnar::FieldCheckedBatch,
        artifact: &str,
        block: u64,
        bounded: bool,
    ) -> Result<(), WorkflowError> {
        // Refuse complete relations rather than silently graphing a prefix.
        macro_rules! rows {
            ($field:ident) => {{
                let rows = located_rows(batch, artifact, block)?;
                if bounded && self.$field.len().saturating_add(rows.len()) > ANALYSIS_NODES {
                    return Err(contract("analysis evidence relation exceeds bound"));
                }
                self.$field.extend(rows);
            }};
        }
        use pse_relations::generated::runtime as relations;
        match batch.relation_id() {
            relations::scientific_contexts::RELATION_ID => rows!(contexts),
            relations::route_decisions::RELATION_ID => rows!(routes),
            relations::local_validity::RELATION_ID => rows!(validity),
            relations::computation_runs::RELATION_ID => rows!(computations),
            relations::trajectory_endpoints::RELATION_ID => rows!(endpoints),
            relations::simulation_samples::RELATION_ID => rows!(samples),
            relations::modeling_trajectory_modes::RELATION_ID => rows!(modes),
            relations::simulation_events::RELATION_ID => rows!(events),
            relations::parametric_sensitivities::RELATION_ID => rows!(parametric),
            relations::response_sensitivities::RELATION_ID => rows!(responses),
            _ => {
                self.interpretation.push(ArtifactLocator {
                    artifact: artifact.into(),
                    block,
                    row: None,
                    field: None,
                });
            }
        }
        Ok(())
    }
}

/// Facts from the exact admitted batch used to encode one standalone member.
/// Provisional locators never escape this phase or grant graph admission.
pub(super) struct MemberFacts {
    pub(super) evidence: ResponseEvidence,
    relation: String,
    rows: u64,
    admission: Arc<pse_relations::validate::ValidationContext>,
    owner: MemoryReservation,
}
impl MemberFacts {
    pub(super) fn mine(
        runtime: &Runtime,
        name: &str,
        batch: &pse_relations::columnar::FieldCheckedBatch,
        analysis: bool,
    ) -> Result<Option<Self>, WorkflowError> {
        if !analysis
            && batch.relation_id()
                != pse_relations::generated::runtime::scientific_contexts::RELATION_ID
            && batch.relation_id()
                != pse_relations::generated::runtime::trajectory_endpoints::RELATION_ID
        {
            return Ok(None);
        }
        let owner =
            MemoryConsumer::new("canonical:encoded-member-facts").register(&runtime.shared.pool());
        let bytes = batch
            .batch()
            .get_array_memory_size()
            .checked_mul(8)
            .and_then(|n| {
                batch
                    .batch()
                    .num_rows()
                    .checked_mul(2048)
                    .and_then(|rows| n.checked_add(rows))
            })
            .and_then(|n| n.checked_add(64 * 1024))
            .ok_or_else(|| contract("member fact allocation extent"))?;
        owner
            .try_grow(bytes)
            .map_err(pse_engine::EngineError::from)?;
        let mut evidence = ResponseEvidence::default();
        evidence.push(batch, "", 0, analysis)?;
        Ok(Some(Self {
            evidence,
            relation: name.into(),
            rows: batch.batch().num_rows() as u64,
            admission: runtime.admission_context(),
            owner,
        }))
    }
}

/// One bounded fact owner shared by promotion and the response-evidence join.
/// Member grants retain row heaps; this grant owns merged vectors/final locators.
pub(super) struct ArtifactEvidence {
    pub(super) evidence: ResponseEvidence,
    members: Vec<MemoryReservation>,
    coordinates: Vec<(u64, String, u64, String)>,
    admission: Arc<pse_relations::validate::ValidationContext>,
    owner: MemoryReservation,
}
impl ArtifactEvidence {
    pub(super) fn new(runtime: &Runtime) -> Result<Self, WorkflowError> {
        let owner =
            MemoryConsumer::new("canonical:artifact-evidence").register(&runtime.shared.pool());
        owner
            .try_grow(
                size_of::<Self>()
                    + 64 * (size_of::<MemoryReservation>()
                        + size_of::<(u64, String, u64, String)>()),
            )
            .map_err(pse_engine::EngineError::from)?;
        Ok(Self {
            evidence: ResponseEvidence::default(),
            members: Vec::with_capacity(64),
            coordinates: Vec::with_capacity(64),
            admission: runtime.admission_context(),
            owner,
        })
    }
    pub(super) fn append(
        &mut self,
        mut member: MemberFacts,
        ordinal: u64,
        analysis: bool,
        name: &str,
        rows: u64,
    ) -> Result<(), WorkflowError> {
        if !Arc::ptr_eq(&member.admission, &self.admission) {
            return Err(contract(
                "member facts belong to another receiving admission",
            ));
        }
        if member.relation != name || member.rows != rows {
            return Err(contract(
                "encoded member facts differ from emitted descriptor",
            ));
        }
        if self.members.len() >= 64 {
            return Err(contract("artifact fact member bound"));
        }
        macro_rules! merge {
            ($field:ident) => {{
                let target = &mut self.evidence.$field;
                let source = &mut member.evidence.$field;
                let needed = target
                    .len()
                    .checked_add(source.len())
                    .ok_or_else(|| contract("artifact fact row extent"))?;
                if analysis && needed > ANALYSIS_NODES {
                    return Err(contract("analysis evidence relation exceeds bound"));
                }
                if needed > target.capacity() {
                    let old = target.capacity() * element_bytes::<_>(target);
                    let bytes = needed
                        .checked_mul(element_bytes::<_>(target))
                        .ok_or_else(|| contract("artifact fact vector extent"))?;
                    self.owner
                        .try_grow(bytes)
                        .map_err(pse_engine::EngineError::from)?;
                    target
                        .try_reserve_exact(source.len())
                        .map_err(|_| contract("artifact fact allocation refused"))?;
                    self.owner.shrink(old);
                }
                for item in source.iter_mut() {
                    item.locator.block = ordinal;
                }
                target.append(source);
            }};
        }
        merge!(contexts);
        merge!(routes);
        merge!(validity);
        merge!(computations);
        merge!(endpoints);
        merge!(samples);
        merge!(modes);
        merge!(events);
        merge!(parametric);
        merge!(responses);
        self.owner
            .try_grow(member.evidence.interpretation.len() * (size_of::<ArtifactLocator>() * 2))
            .map_err(pse_engine::EngineError::from)?;
        for mut locator in member.evidence.interpretation {
            locator.block = ordinal;
            self.evidence.interpretation.push(locator);
        }
        self.coordinates
            .push((ordinal, member.relation, member.rows, String::new()));
        self.members.push(member.owner);
        Ok(())
    }
    pub(super) fn verify_members(
        &mut self,
        blocks: &[pse_operations::canonical_artifacts::ArtifactBlock],
    ) -> Result<(), WorkflowError> {
        for (ordinal, relation, rows, digest) in &mut self.coordinates {
            let block = usize::try_from(*ordinal)
                .ok()
                .and_then(|index| blocks.get(index))
                .filter(|block| {
                    block.ordinal == *ordinal && block.relation == *relation && block.rows == *rows
                })
                .ok_or_else(|| contract("artifact fact member coordinate differs"))?;
            self.owner
                .try_grow(block.digest.len())
                .map_err(pse_engine::EngineError::from)?;
            digest.clone_from(&block.digest);
        }
        Ok(())
    }
    pub(super) fn bind(
        mut self,
        runtime: &Runtime,
        read: &ArtifactRead,
        expected: &pse_operations::canonical_artifacts::Artifact,
    ) -> Result<Self, WorkflowError> {
        if !Arc::ptr_eq(&self.admission, &runtime.admission_context()) {
            return Err(contract(
                "artifact facts require actual receiving admission",
            ));
        }
        super::artifacts::check_artifact_ack(&runtime.shared.pool(), expected, read.header())?;
        if self
            .coordinates
            .iter()
            .any(|(_, _, _, digest)| digest.is_empty())
        {
            return Err(contract("artifact facts lack exact encoded member binding"));
        }
        let mut count = self.evidence.interpretation.len();
        macro_rules! count { ($($field:ident),*) => { $(count += self.evidence.$field.len();)* }; }
        count!(
            contexts,
            routes,
            validity,
            computations,
            endpoints,
            samples,
            modes,
            events,
            parametric,
            responses
        );
        self.owner
            .try_grow(
                count
                    .checked_mul(expected.key.len())
                    .ok_or_else(|| contract("artifact locator extent"))?,
            )
            .map_err(pse_engine::EngineError::from)?;
        macro_rules! bind { ($($field:ident),*) => { $(for item in &mut self.evidence.$field { item.locator.artifact.clone_from(&expected.key); })* }; }
        bind!(
            contexts,
            routes,
            validity,
            computations,
            endpoints,
            samples,
            modes,
            events,
            parametric,
            responses
        );
        for locator in &mut self.evidence.interpretation {
            locator.artifact.clone_from(&expected.key);
        }
        Ok(self)
    }
}
fn element_bytes<T>(_: &Vec<T>) -> usize {
    size_of::<T>()
}

type EvidenceRun = pse_model::generated::identities::RunId;
type EvidenceSample = (
    EvidenceRun,
    i64,
    u64,
    pse_ids::SemanticId,
    pse_ids::SemanticId,
);

#[derive(Default)]
struct RunEvidence<'a> {
    contexts: Vec<&'a Located<pse_model::generated::runtime::scientific_contexts::Row>>,
    computations: Vec<&'a Located<pse_model::generated::runtime::computation_runs::Row>>,
    endpoints: Vec<&'a Located<pse_model::generated::runtime::trajectory_endpoints::Row>>,
    events: Vec<&'a Located<pse_model::generated::runtime::simulation_events::Row>>,
    // Some only when every duplicate agrees and the original family/context
    // prerequisites hold. Partial/event completion is deliberately permitted.
    transient: Option<(f64, i64, f64, pse_model::generated::identities::InstanceId)>,
    parameters: BTreeSet<pse_ids::SemanticId>,
}
#[derive(Default)]
struct StepEvidence<'a> {
    scalar_contexts: Vec<&'a Located<pse_model::generated::runtime::scientific_contexts::Row>>,
    routes: Vec<&'a Located<pse_model::generated::runtime::route_decisions::Row>>,
}
#[derive(Default)]
struct CoordinateEvidence<'a> {
    contexts: Vec<&'a Located<pse_model::generated::runtime::scientific_contexts::Row>>,
    validity: Vec<&'a Located<pse_model::generated::runtime::local_validity::Row>>,
    sensitivity_validity: Vec<&'a Located<pse_model::generated::runtime::local_validity::Row>>,
    family: Option<ResponseFamily>,
    certified: bool,
}

// One operation-local typed join; original rows/locators remain in the evidence
// owner. This index contains no replacement scientific permission oracle.
struct ResponseEvidenceIndex<'a> {
    runs: BTreeMap<EvidenceRun, RunEvidence<'a>>,
    steps: BTreeMap<i64, StepEvidence<'a>>,
    coordinates: BTreeMap<(EvidenceRun, i64), CoordinateEvidence<'a>>,
    samples: BTreeSet<EvidenceSample>,
    // Every retained map node/vector slot is prepaid before any index growth.
    _owner: MemoryReservation,
}
fn evidence_time(time: f64) -> Option<u64> {
    if time.is_nan() {
        None
    } else {
        Some(if time == 0.0 { 0 } else { time.to_bits() })
    }
}
impl<'a> ResponseEvidenceIndex<'a> {
    fn new(
        evidence: &'a ResponseEvidence,
        pool: &Arc<dyn pse_columnar::MemoryPool>,
    ) -> Result<Self, WorkflowError> {
        let owner = MemoryConsumer::new("canonical:response-evidence-index").register(pool);
        // Bound a newly allocated BTree node, including vacant element/edge
        // slots, rather than charging just the live key/value. Reference vectors
        // include the initial four slots and geometric growth. These bounds are
        // deliberately independent of how many keys happen to share one node.
        fn tree_node<K, V>() -> usize {
            16 * size_of::<(K, V)>() + 512
        }
        let run = tree_node::<EvidenceRun, RunEvidence<'_>>();
        let step = tree_node::<i64, StepEvidence<'_>>();
        let coordinate = tree_node::<(EvidenceRun, i64), CoordinateEvidence<'_>>();
        let references = 4 * size_of::<&Located<()>>();
        let inputs = evidence
            .contexts
            .iter()
            .try_fold(0usize, |n, c| n.checked_add(c.row.inputs.len()))
            .ok_or_else(|| contract("analysis index input extent"))?;
        let extents = [
            (
                evidence.contexts.len(),
                run + step + coordinate + 3 * references,
            ),
            (evidence.routes.len(), step + references),
            (evidence.validity.len(), coordinate + 2 * references),
            (evidence.computations.len(), run + references),
            (evidence.endpoints.len(), run + references),
            (evidence.events.len(), run + references),
            (evidence.samples.len(), tree_node::<EvidenceSample, ()>()),
            (evidence.parametric.len(), coordinate),
            (inputs, tree_node::<pse_ids::SemanticId, ()>()),
            // Temporary coordinate-key inventory used while validating groups.
            (evidence.contexts.len(), 2 * size_of::<(EvidenceRun, i64)>()),
            (evidence.validity.len(), 2 * size_of::<(EvidenceRun, i64)>()),
            (
                evidence.parametric.len(),
                2 * size_of::<(EvidenceRun, i64)>(),
            ),
        ];
        let bytes = extents
            .into_iter()
            .try_fold(size_of::<Self>(), |bytes, (count, extent)| {
                count
                    .checked_mul(extent)
                    .and_then(|extent| bytes.checked_add(extent))
            })
            .ok_or_else(|| contract("analysis index allocation extent"))?;
        owner
            .try_grow(bytes)
            .map_err(pse_engine::EngineError::from)?;
        let mut index = Self {
            runs: BTreeMap::new(),
            steps: BTreeMap::new(),
            coordinates: BTreeMap::new(),
            samples: BTreeSet::new(),
            _owner: owner,
        };
        for item in &evidence.contexts {
            index
                .runs
                .entry(item.row.run_id)
                .or_default()
                .contexts
                .push(item);
            index
                .coordinates
                .entry((item.row.run_id, item.row.step))
                .or_default()
                .contexts
                .push(item);
            if item.row.time.is_none() {
                index
                    .steps
                    .entry(item.row.step)
                    .or_default()
                    .scalar_contexts
                    .push(item);
            }
        }
        for item in &evidence.routes {
            index
                .steps
                .entry(item.row.step)
                .or_default()
                .routes
                .push(item);
        }
        for item in &evidence.validity {
            let group = index
                .coordinates
                .entry((item.row.run_id, item.row.step))
                .or_default();
            group.validity.push(item);
            if item.row.quantity
                == pse_model::generated::enums::DerivedQuantity::ParametricSensitivity
            {
                group.sensitivity_validity.push(item);
            }
        }
        for item in &evidence.parametric {
            index
                .coordinates
                .entry((item.row.run_id, item.row.step))
                .or_default();
        }
        for item in &evidence.computations {
            index
                .runs
                .entry(item.row.run_id)
                .or_default()
                .computations
                .push(item);
        }
        for item in &evidence.endpoints {
            index
                .runs
                .entry(item.row.run_id)
                .or_default()
                .endpoints
                .push(item);
        }
        for item in &evidence.events {
            index
                .runs
                .entry(item.row.run_id)
                .or_default()
                .events
                .push(item);
        }
        for item in &evidence.samples {
            let row = &item.row;
            if let Some(time) = evidence_time(row.time) {
                index
                    .samples
                    .insert((row.run_id, row.sample, time, row.symbol_id, row.unit_id));
            }
        }
        for group in index.runs.values_mut() {
            use pse_model::generated::enums::ComputationKind;
            let Some(header) = group.computations.first().map(|v| &v.row) else {
                continue;
            };
            let Some(context) = group.contexts.first().map(|v| &v.row) else {
                continue;
            };
            let Some(endpoint) = group.endpoints.first().map(|v| &v.row) else {
                continue;
            };
            if !group.computations.iter().all(|v| v.row == *header)
                || !group.contexts.iter().all(|v| v.row == *context)
                || !group.endpoints.iter().all(|v| v.row == *endpoint)
                || header.kind != ComputationKind::Simulation
                || header.response_available != Some(true)
                || context.time.is_none()
            {
                continue;
            }
            if let (Some(end), Some(count), Some(instance)) = (
                header.completed_time,
                header.completed_samples,
                context.instance_id,
            ) {
                group.transient = Some((end, count, endpoint.time, instance));
                group.parameters.extend(context.inputs.iter().map(|v| v.id));
            }
        }
        // Validate family and local certificates once per exact run/step. Do not
        // cache one run's permission under a step-only route coordinate.
        let keys = index.coordinates.keys().copied().collect::<Vec<_>>();
        for (run, step) in keys {
            let family = index.scalar_family(run, step);
            let certified = family.is_some_and(|f| index.scalar_certified(run, step, f));
            let group = index
                .coordinates
                .get_mut(&(run, step))
                .ok_or_else(|| contract("analysis index coordinate absent"))?;
            group.family = family;
            group.certified = certified;
        }
        Ok(index)
    }
    fn scalar_family(
        &self,
        run: pse_model::generated::identities::RunId,
        step: i64,
    ) -> Option<ResponseFamily> {
        use pse_model::generated::enums::NativeSolveIntent;
        let group = self.steps.get(&step)?;
        let contexts = &group.scalar_contexts;
        // Route rows have no run_id. Ambiguous step ownership cannot borrow a
        // different run's route, even when its numerical rows happen to agree.
        if contexts.is_empty()
            || contexts.iter().any(|context| {
                context.row.run_id != run
                    || context.row.model_id.is_none()
                    || context.row != contexts[0].row
            })
        {
            return None;
        }
        let mut routes = group.routes.iter();
        let first = &routes.next()?.row;
        if routes.any(|route| {
            route.row.request_identity != first.request_identity || route.row.intent != first.intent
        }) {
            return None;
        }
        match first.intent {
            NativeSolveIntent::Root => Some(ResponseFamily::Root),
            NativeSolveIntent::Optimize => Some(ResponseFamily::Optimal),
            _ => None,
        }
    }
    fn scalar_certified(
        &self,
        run: pse_model::generated::identities::RunId,
        step: i64,
        family: ResponseFamily,
    ) -> bool {
        let Some(group) = self.coordinates.get(&(run, step)) else {
            return false;
        };
        let rows = &group.sensitivity_validity;
        !rows.is_empty()
            && rows.iter().all(|row| {
                let v = &row.row.validity;
                if !v.certified || v.reason.is_some() {
                    return false;
                }
                match family {
                    ResponseFamily::Root => {
                        v.root_rank.is_some()
                            && v.root_rank_cutoff.is_some()
                            && v.root_rank_relative_cutoff.is_some()
                            && v.root_backward_error.is_some()
                            && v.root_backward_error_limit.is_some()
                            && v.root_neighborhood
                                .as_ref()
                                .is_some_and(|value| !value.is_empty())
                            && v.licq.is_none()
                            && v.strict_complementarity.is_none()
                            && v.second_order.is_none()
                    }
                    ResponseFamily::Optimal => {
                        v.licq == Some(true)
                            && v.strict_complementarity == Some(true)
                            && v.second_order == Some(true)
                            && v.weakly_active == Some(0)
                            && v.root_rank.is_none()
                            && v.root_rank_cutoff.is_none()
                            && v.root_rank_relative_cutoff.is_none()
                            && v.root_backward_error.is_none()
                            && v.root_backward_error_limit.is_none()
                            && v.root_neighborhood.is_none()
                    }
                }
            })
    }

    // Recorded sensitivity is weaker than a scalar local certificate. Preserve
    // actual endpoint/sample coverage and IEEE equality (NaN never matches).
    fn transient_recorded(
        &self,
        row: &pse_model::generated::runtime::response_sensitivities::Row,
    ) -> bool {
        let Some(time) = row.time else {
            return false;
        };
        let Some(bits) = evidence_time(time) else {
            return false;
        };
        let Some(group) = self.runs.get(&row.run_id) else {
            return false;
        };
        let Some((end, count, endpoint, instance)) = group.transient else {
            return false;
        };
        row.parameter_column >= 0
            && row.sample >= 0
            && row.sample < count
            && time <= end
            && time <= endpoint
            && row.experiment_id == instance
            && group.parameters.contains(&row.parameter_id)
            && self.samples.contains(&(
                row.run_id,
                row.sample,
                bits,
                row.output_id,
                row.output_unit_id,
            ))
    }
}

fn evidence_context(
    graph: &mut Graph,
    nodes: &mut BTreeMap<String, NodeIndex>,
    attempt: NodeIndex,
    target: NodeIndex,
    kind: &str,
    locator: &ArtifactLocator,
) -> Result<(), WorkflowError> {
    let mut identity = pse_ids::FramedHasher::new(pse_ids::Frame::CanonicalPayloadV1);
    identity
        .str("pse.analysis.evidence-coordinate.v2")
        .str(&locator.artifact)
        .u64(locator.block);
    // Fields are distinct edges over the same retained row/block owner.
    identity.part(&serde_json::to_vec(&locator.row).map_err(|error| contract(error.to_string()))?);
    let context = vertex(
        graph,
        nodes,
        format!("evidence:{}", identity.finish_hash().to_hex()),
        "retained_context",
    )?;
    if target == attempt {
        return edge(graph, attempt, context, kind, Some(locator.clone()));
    }
    edge(
        graph,
        attempt,
        context,
        "produced_context",
        Some(locator.clone()),
    )?;
    edge(graph, context, target, kind, Some(locator.clone()))
}
fn project_response_evidence(
    evidence: &ResponseEvidence,
    graph: &mut Graph,
    nodes: &mut BTreeMap<String, NodeIndex>,
    attempt: NodeIndex,
    pool: &Arc<dyn pse_columnar::MemoryPool>,
) -> Result<(), WorkflowError> {
    // Admission precedes even speculative graph mutation.
    let index = ResponseEvidenceIndex::new(evidence, pool)?;
    use pse_model::generated::enums::NumericalTarget;
    // Interpretation/qualification evidence survives even with no derivative
    // rows, including original withheld reasons and partial event coverage.
    macro_rules! retain {
        ($rows:expr, $kind:expr) => {
            for item in $rows {
                evidence_context(graph, nodes, attempt, attempt, $kind, &item.locator)?;
            }
        };
    }
    retain!(&evidence.contexts, "scientific_context");
    retain!(&evidence.routes, "admitted_route");
    retain!(&evidence.validity, "local_validity");
    retain!(&evidence.computations, "computation_context");
    retain!(&evidence.endpoints, "trajectory_endpoint");
    retain!(&evidence.events, "trajectory_event");
    retain!(&evidence.modes, "trajectory_mode");
    for locator in &evidence.interpretation {
        evidence_context(
            graph,
            nodes,
            attempt,
            attempt,
            "retained_interpretation",
            locator,
        )?;
    }
    for item in &evidence.parametric {
        let row = &item.row;
        let coordinate = index
            .coordinates
            .get(&(row.run_id, row.step))
            .ok_or_else(|| contract("analysis response coordinate absent"))?;
        let family = coordinate.family;
        let family_name = family.map_or("unclassified", ResponseFamily::name);
        let parameter = vertex(graph, nodes, row.parameter_id.to_string(), "parameter")?;
        let target_name = if row.target_kind == NumericalTarget::Objective {
            "objective".into()
        } else {
            row.target_id.to_string()
        };
        let original = vertex(
            graph,
            nodes,
            target_name.clone(),
            target_kind(row.target_kind),
        )?;
        let target = vertex(
            graph,
            nodes,
            format!(
                "{family_name}:{}:step:{}:{target_name}",
                row.run_id, row.step
            ),
            "scoped_response",
        )?;
        edge(
            graph,
            target,
            original,
            "scientific_coordinate",
            Some(item.locator.clone()),
        )?;
        edge(
            graph,
            attempt,
            target,
            "produced_evidence",
            Some(item.locator.clone()),
        )?;
        for context in &coordinate.contexts {
            evidence_context(
                graph,
                nodes,
                attempt,
                target,
                "response_coordinates",
                &context.locator,
            )?;
        }
        for route in index
            .steps
            .get(&row.step)
            .into_iter()
            .flat_map(|group| &group.routes)
        {
            evidence_context(
                graph,
                nodes,
                attempt,
                target,
                "response_family",
                &field_locator(&route.locator, "intent"),
            )?;
        }
        for validity in &coordinate.validity {
            evidence_context(
                graph,
                nodes,
                attempt,
                target,
                "response_validity",
                &field_locator(&validity.locator, "validity"),
            )?;
        }
        for (field, value) in [("primal", row.primal), ("dual", row.dual)] {
            if value.is_none() {
                continue;
            }
            let certified = family.is_some_and(|family| {
                coordinate.certified
                    && (family != ResponseFamily::Root
                        || (field == "primal" && row.target_kind == NumericalTarget::Variable))
            });
            let kind = if certified {
                format!("retained_{family_name}_parametric_{field}")
            } else {
                format!("withheld_{family_name}_parametric_{field}")
            };
            edge(
                graph,
                if certified { parameter } else { attempt },
                target,
                &kind,
                Some(field_locator(&item.locator, field)),
            )?;
        }
    }
    for item in &evidence.responses {
        let row = &item.row;
        let recorded = index.transient_recorded(row);
        let parameter = vertex(graph, nodes, row.parameter_id.to_string(), "parameter")?;
        // Each sample retains its own quantitative row locator even when the
        // semantic output node is shared. Sample/time/unit and parameter-column
        // scope are in that row; scheduled axes are never summed here.
        let target = vertex(graph, nodes, row.output_id.to_string(), "output")?;
        // Production provenance describes the output's membership in this
        // retained block. Quantitative claims below still identify every row.
        let mut produced = item.locator.clone();
        produced.row = None;
        produced.field = None;
        edge(graph, attempt, target, "produced_evidence", Some(produced))?;
        edge(
            graph,
            if recorded { parameter } else { attempt },
            target,
            if recorded {
                "recorded_transient_response"
            } else {
                "withheld_response_claim"
            },
            Some(field_locator(&item.locator, "value")),
        )?;
        for context in index
            .runs
            .get(&row.run_id)
            .into_iter()
            .flat_map(|group| &group.contexts)
        {
            evidence_context(
                graph,
                nodes,
                attempt,
                target,
                "response_coordinates",
                &context.locator,
            )?;
        }
        for header in index
            .runs
            .get(&row.run_id)
            .into_iter()
            .flat_map(|group| &group.computations)
        {
            evidence_context(
                graph,
                nodes,
                attempt,
                target,
                "response_completion",
                &header.locator,
            )?;
        }
        for endpoint in index
            .runs
            .get(&row.run_id)
            .into_iter()
            .flat_map(|group| &group.endpoints)
        {
            evidence_context(
                graph,
                nodes,
                attempt,
                target,
                "response_endpoint",
                &endpoint.locator,
            )?;
        }
        // The quantitative row itself preserves exact sample/time/output/unit
        // coordinates. Its matching sample and mode remain in the same retained
        // trajectory; avoid duplicating a graph edge per output/sample metadata.
        for event in index
            .runs
            .get(&row.run_id)
            .into_iter()
            .flat_map(|group| &group.events)
        {
            evidence_context(
                graph,
                nodes,
                attempt,
                target,
                "response_event_context",
                &event.locator,
            )?;
        }
    }
    Ok(())
}

/// Compact restartable exact analysis handle. Graph pages come from native
/// relations; this owner never retains a hydrated run report or trajectory.
#[derive(Clone, Debug)]
pub struct AnalysisHandle {
    runtime: Runtime,
    key: String,
}
impl AnalysisHandle {
    /// Immutable method/configuration and graph identity.
    pub fn key(&self) -> &str {
        &self.key
    }
    /// Registry-declared method and input-lineage header.
    pub async fn header(
        &self,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, WorkflowError> {
        let activity = self.runtime.admit_activity()?;
        activity
            .scope(async {
                use pse_relations::generated::runtime::canonical_analyses as rows;
                let owner = super::connected_results::response_owner(
                    &self.runtime,
                    "canonical:analysis-header",
                    MetadataResponse::Header,
                )?;
                let row = pse_operations::canonical::response_ownership_scope(
                    owner.clone(),
                    self.runtime.canonical_store()?.analysis(&self.key),
                )
                .await?;
                let validation = self.runtime.validation_context();
                let mut builder =
                    rows::Builder::with_registry(self.runtime.registry(), 1, &validation)
                        .map_err(relation)?;
                builder.push(row).map_err(relation)?;
                builder
                    .finish()
                    .map_err(relation)?
                    .retained(&self.runtime.shared.pool(), &CancellationToken::new())
                    .map_err(relation)
            })
            .await
    }
    /// Native ordered node page, at most 64 rows, retaining actual Arrow buffers.
    pub async fn nodes(
        &self,
        after: Option<&str>,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, WorkflowError> {
        let activity = self.runtime.admit_activity()?;
        activity
            .scope(async {
                use pse_relations::generated::runtime::canonical_analysis_nodes as rows;
                let owner = super::connected_results::response_owner(
                    &self.runtime,
                    "canonical:analysis-page",
                    MetadataResponse::Page,
                )?;
                let page = pse_operations::canonical::response_ownership_scope(
                    owner.clone(),
                    self.runtime
                        .canonical_store()?
                        .analysis_node_page(&self.key, after),
                )
                .await?;
                let validation = self.runtime.validation_context();
                let mut builder =
                    rows::Builder::with_registry(self.runtime.registry(), page.len(), &validation)
                        .map_err(relation)?;
                for row in page {
                    builder.push(row).map_err(relation)?;
                }
                builder
                    .finish()
                    .map_err(relation)?
                    .retained(&self.runtime.shared.pool(), &CancellationToken::new())
                    .map_err(relation)
            })
            .await
    }
    /// Native directed graph edge page with original result evidence references.
    pub async fn edges(
        &self,
        after: Option<&str>,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, WorkflowError> {
        let activity = self.runtime.admit_activity()?;
        activity
            .scope(async {
                use pse_relations::generated::runtime::canonical_analysis_edges as rows;
                let owner = super::connected_results::response_owner(
                    &self.runtime,
                    "canonical:analysis-page",
                    MetadataResponse::Page,
                )?;
                let page = pse_operations::canonical::response_ownership_scope(
                    owner.clone(),
                    self.runtime
                        .canonical_store()?
                        .analysis_edge_page(&self.key, after),
                )
                .await?;
                let validation = self.runtime.validation_context();
                let mut builder =
                    rows::Builder::with_registry(self.runtime.registry(), page.len(), &validation)
                        .map_err(relation)?;
                for row in page {
                    builder.push(row).map_err(relation)?;
                }
                builder
                    .finish()
                    .map_err(relation)?
                    .retained(&self.runtime.shared.pool(), &CancellationToken::new())
                    .map_err(relation)
            })
            .await
    }
}
impl Runtime {
    /// Read bounded original completion metadata from one exact retained attempt.
    /// Scientific reports and trajectories remain in their canonical result sets.
    pub async fn stored_completion(
        &self,
        run: &str,
        attempt: &str,
    ) -> Result<Arc<pse_columnar::Leased<super::durable::StoredCompletion>>, WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async { self.operations()?.completion(run, attempt).await })
            .await
    }
    /// Reopen only an active, retained exact analysis; no latest-run substitution.
    pub async fn analysis(&self, key: &str) -> Result<AnalysisHandle, WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                let response = super::connected_results::response_owner(
                    self,
                    "canonical:analysis-selection-response",
                    MetadataResponse::Header,
                )?;
                pse_operations::canonical::response_ownership_scope(
                    response,
                    self.canonical_store()?.analysis(key),
                )
                .await?;
                Ok(AnalysisHandle {
                    runtime: self.clone(),
                    key: key.into(),
                })
            })
            .await
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "one analysis admission binds exact source and result lineage, method/configuration, graph membership and selection controls"
    )]
    async fn store_analysis_graph(
        &self,
        revision: &pse_operations::canonical::Revision,
        sources: &[pse_operations::canonical::Revision],
        inputs: &[ArtifactRead],
        method: &str,
        configuration: Vec<u8>,
        graph: Graph,
        nodes: BTreeMap<String, NodeIndex>,
        controls: &AnalysisControls,
    ) -> Result<AnalysisHandle, WorkflowError> {
        use petgraph::visit::EdgeRef;
        let selected = selected(&graph, &nodes, controls)?;
        let mut input_hash = pse_ids::FramedHasher::new(pse_ids::Frame::CanonicalPayloadV1);
        input_hash.str("pse.analysis.inputs.v1");
        for source in sources {
            input_hash.str(&source.key);
        }
        for input in inputs {
            input_hash
                .str(&input.header().key)
                .str(&input.header().creation_request_digest);
        }
        for index in &selected {
            let (semantic, kind) = &graph[*index];
            input_hash.str(semantic).str(kind);
        }
        let mut included = graph
            .edge_references()
            .filter(|edge| selected.contains(&edge.source()) && selected.contains(&edge.target()))
            .map(|edge| {
                let (kind, evidence) = edge.weight();
                (
                    graph[edge.source()].0.clone(),
                    graph[edge.target()].0.clone(),
                    kind.clone(),
                    evidence.clone(),
                )
            })
            .collect::<Vec<_>>();
        included.sort();
        included.dedup();
        for (source, target, kind, evidence) in &included {
            input_hash
                .str(source)
                .str(target)
                .str(kind)
                .part(&serde_json::to_vec(evidence).map_err(|error| contract(error.to_string()))?);
        }
        let input_digest = input_hash.finish_hash().to_hex();
        let mut header = self
            .canonical_store()?
            .new_analysis(
                revision,
                method,
                configuration,
                input_digest,
                selected.len() as u64,
                included.len() as u64,
            )
            .await?;
        let key = header.key.clone();
        let mut mapping = BTreeMap::new();
        let mut retained_nodes = Vec::with_capacity(selected.len());
        for index in selected {
            let (semantic, kind) = &graph[index];
            let node_key = analysis_key(
                "pse.analysis.node.v1",
                &[key.as_bytes(), semantic.as_bytes()],
            );
            mapping.insert(semantic.clone(), node_key.clone());
            retained_nodes.push(AnalysisNode {
                key: node_key,
                analysis: key.clone(),
                semantic: semantic.clone(),
                kind: kind.clone(),
            });
        }
        let mut retained_edges = Vec::with_capacity(included.len());
        for (source, target, kind, evidence) in included {
            let source = mapping
                .get(&source)
                .ok_or_else(|| contract("analysis source absent"))?
                .clone();
            let target = mapping
                .get(&target)
                .ok_or_else(|| contract("analysis target absent"))?
                .clone();
            let evidence_bytes =
                serde_json::to_vec(&evidence).map_err(|error| contract(error.to_string()))?;
            let edge_key = analysis_key(
                "pse.analysis.edge.v1",
                &[
                    key.as_bytes(),
                    source.as_bytes(),
                    target.as_bytes(),
                    kind.as_bytes(),
                    &evidence_bytes,
                ],
            );
            retained_edges.push(AnalysisEdge {
                key: edge_key,
                analysis: key.clone(),
                source,
                target,
                kind,
                evidence_artifact: evidence.as_ref().map(|locator| locator.artifact.clone()),
                evidence_block: evidence.as_ref().map(|locator| locator.block),
                evidence_row: evidence.as_ref().and_then(|locator| locator.row),
                evidence_field: evidence.and_then(|locator| locator.field),
            });
        }
        pse_operations::canonical_analyses::seal_analysis_request(
            &mut header,
            sources,
            inputs,
            &retained_nodes,
            &retained_edges,
        );
        let acknowledged = self
            .canonical_store()?
            .persist_analysis(&header, sources, inputs, &retained_nodes, &retained_edges)
            .await?;
        let valid = check_analysis_ack(&self.shared.pool(), &header, &acknowledged);
        #[cfg(all(test, feature = "canonical-tests"))]
        let lost = GRAPH_HANDOFF_REPLY_LOSS
            .try_with(|lost| *lost)
            .unwrap_or(false);
        #[cfg(not(all(test, feature = "canonical-tests")))]
        let lost = false;
        if valid.is_err() || lost {
            return Err(
                pse_operations::canonical::CanonicalError::AnalysisUnsettled {
                    intent: Box::new(header),
                    source: Box::new(pse_operations::canonical::CanonicalError::IncompleteResponse),
                }
                .into(),
            );
        }
        Ok(AnalysisHandle {
            runtime: self.clone(),
            key,
        })
    }

    /// Construct the protected selection once, before its caller polls it.
    /// The borrowed grant and cancellation owner outlive this boxed state. Keeping
    /// construction out of the polling frame avoids moving its concrete RPC state
    /// through each surrounding ownership scope; the same read leases remain held.
    #[inline(never)]
    fn select_result_analysis<'a>(
        &'a self,
        run: &'a str,
        attempt: &'a str,
        _owner: &'a MemoryReservation,
        cancel: &'a CancellationToken,
    ) -> futures_util::future::BoxFuture<
        'a,
        Result<super::artifacts::ScientificArtifactSelection, WorkflowError>,
    > {
        Box::pin(async move {
            let protected = self.select_scientific_input(run, attempt).await?;
            let mut prepared = Vec::new();
            use pse_model::generated::products::ScientificRelation;
            use pse_relations::generated::runtime::{computation_runs, scientific_contexts};
            for member in [
                ScientificRelation {
                    id: scientific_contexts::RELATION_ID,
                    name: scientific_contexts::QUALIFIED_NAME,
                },
                ScientificRelation {
                    id: computation_runs::RELATION_ID,
                    name: computation_runs::QUALIFIED_NAME,
                },
            ] {
                let spec = self
                    .registry()
                    .relation_by_id(member.id)
                    .ok_or_else(|| contract("response dispatch relation absent"))?;
                let Some(set) = protected
                    .input
                    .sets()
                    .iter()
                    .find(|set| set.name == spec.id.to_string())
                else {
                    continue;
                };
                let schema = pse_schema::arrow::relation_schema_ref(self.registry(), spec)
                    .map_err(|error| contract(error.to_string()))?;
                let snapshot = super::ResultSnapshot::capture(
                    self,
                    protected.input.clone(),
                    protected.input_owner.clone(),
                    spec.id,
                    &set.key,
                    schema,
                    0,
                    u64::MAX,
                )
                .await?;
                for block in snapshot.blocks() {
                    if prepared.len() >= 64 {
                        return Err(contract("scientific dispatch member bound"));
                    }
                    let batch = snapshot.read_block(block, cancel).await?;
                    prepared.push(super::artifacts::prepare_scientific_member(
                        self,
                        member,
                        &batch,
                        &block.payload_digest,
                        true,
                    )?);
                }
            }
            let kind = super::artifacts::response_artifact_kind(
                prepared
                    .iter()
                    .filter_map(|member| member.facts.as_ref())
                    .flat_map(|facts| facts.evidence.contexts.iter().map(|c| &c.row)),
                prepared
                    .iter()
                    .filter_map(|member| member.facts.as_ref())
                    .flat_map(|facts| facts.evidence.computations.iter().map(|c| &c.row)),
            )?;
            Ok(super::artifacts::ScientificArtifactSelection {
                protected,
                kind,
                prepared,
            })
        })
    }

    /// Persist family-qualified response evidence and its exact provenance.
    /// Scalar derivatives require their admitted route and original validity;
    /// transient recordings retain their weaker sample/endpoint meaning. Graph
    /// reachability never invents a derivative or promotes outcome-class success.
    pub async fn result_analysis(
        &self,
        run: &str,
        attempt: &str,
        controls: &AnalysisControls,
        cancel: CancellationToken,
    ) -> Result<AnalysisHandle, WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                controls.validate()?;
                transfer_checkpoint(TransferCheckpoint::BeforeDispatch, &cancel)?;
                let owner =
                    MemoryConsumer::new("canonical:result-analysis").register(&self.shared.pool());
                owner
                    .try_grow(128 * 1024 * 1024)
                    .map_err(pse_engine::EngineError::from)?;
                self.result_analysis_body(run, attempt, controls, &owner, &cancel)
                    .await
            })
            .await
    }

    /// The activity scope owns this future through return or cancellation. Its
    /// prepaid grant stays in the small enclosing future until this borrowed body
    /// drops. The factory returns before polling, so concrete state construction
    /// cannot remain live underneath nested transport and artifact polls.
    #[inline(never)]
    fn result_analysis_body<'a>(
        &'a self,
        run: &'a str,
        attempt: &'a str,
        controls: &'a AnalysisControls,
        owner: &'a MemoryReservation,
        cancel: &'a CancellationToken,
    ) -> futures_util::future::BoxFuture<'a, Result<AnalysisHandle, WorkflowError>> {
        Box::pin(async move {
            let selection = self
                .select_result_analysis(run, attempt, owner, cancel)
                .await?;
            let staging_owner = format!(
                "analysis-staging:{}",
                pse_operations::mint_id::<pse_model::generated::identities::RunId>()
            );
            let promotion = match self
                .promote_selected_scientific_artifact(
                    selection,
                    "audit",
                    &staging_owner,
                    cancel,
                    owner,
                    true,
                )
                .await
            {
                Ok(product) => product,
                Err(source) => {
                    if let WorkflowError::Canonical(
                        pse_operations::canonical::CanonicalError::ArtifactUnsettled {
                            intent, ..
                        },
                    ) = &source
                    {
                        return Err(WorkflowError::ArtifactTransferUnsettled {
                            analysis: None,
                            artifact: intent.clone(),
                            owner_kind: "audit".into(),
                            owner: staging_owner,
                            read: None,
                            source: Box::new(source),
                        });
                    }
                    return Err(source);
                }
            };
            let artifact = promotion.artifact;
            let protected_input = promotion.protected;
            let sources = &protected_input.sources;
            let revision = &protected_input.revision;
            let facts = promotion.facts;
            let transfer_error =
                |source: WorkflowError, analysis: Option<String>, read: Option<String>| {
                    WorkflowError::ArtifactTransferUnsettled {
                        analysis,
                        artifact: Box::new(artifact.header().clone()),
                        owner_kind: "audit".into(),
                        owner: staging_owner.clone(),
                        read,
                        source: Box::new(source),
                    }
                };
            transfer_checkpoint(TransferCheckpoint::BeforeArtifactRead, cancel)
                .map_err(|source| transfer_error(source, None, None))?;
            let response = super::connected_results::response_owner(
                self,
                "canonical:analysis-artifact-selection-response",
                MetadataResponse::Selection,
            )
            .map_err(|source| transfer_error(source, None, None))?;
            let artifact_read = pse_operations::canonical::response_ownership_scope(
                response.clone(),
                self.canonical_store()?
                    .read_artifact(&artifact.header().key),
            )
            .await
            .map_err(|source| transfer_error(source.into(), None, None))?;
            let read_owner = Arc::new(
                super::connected_results::retain_protected_metadata(
                    &self.shared.pool(),
                    response,
                    artifact_read.retained_bytes().map_err(|source| {
                        transfer_error(
                            source.into(),
                            None,
                            Some(artifact_read.read_key().to_owned()),
                        )
                    })?,
                    |owner| artifact_read.retain_metadata_owner(owner),
                )
                .map_err(|source| {
                    transfer_error(source, None, Some(artifact_read.read_key().to_owned()))
                })?,
            );
            artifact_read
                .retain_metadata_owner(read_owner.clone())
                .map_err(|source| {
                    transfer_error(
                        source.into(),
                        None,
                        Some(artifact_read.read_key().to_owned()),
                    )
                })?;
            let read_key = artifact_read.read_key().to_owned();
            let graph_result: Result<AnalysisHandle, WorkflowError> = Box::pin(async {
                // Only exact acknowledged independent content plus the actual
                // artifact acquisition can bind provisional scientific locators.
                transfer_checkpoint(TransferCheckpoint::BeforeGraph, cancel)?;
                let facts = facts.bind(self, &artifact_read, artifact.header())?;

                let mut graph = Graph::new();
                let mut nodes = BTreeMap::new();
                let attempt_node = vertex(
                    &mut graph,
                    &mut nodes,
                    format!("attempt:{attempt}"),
                    "attempt",
                )?;
                for source in sources {
                    let node = vertex(
                        &mut graph,
                        &mut nodes,
                        format!("revision:{}", source.key),
                        "source_revision",
                    )?;
                    edge(&mut graph, node, attempt_node, "input_provenance", None)?;
                }
                project_response_evidence(
                    &facts.evidence,
                    &mut graph,
                    &mut nodes,
                    attempt_node,
                    &self.shared.pool(),
                )?;
                cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
                let configuration = serde_json::to_vec(&(2_u8, controls, run, attempt))
                    .map_err(|error| contract(error.to_string()))?;
                let handle = self
                    .store_analysis_graph(
                        revision,
                        sources,
                        std::slice::from_ref(&artifact_read),
                        "result-sensitivity-provenance-reachability:v2",
                        configuration,
                        graph,
                        nodes,
                        controls,
                    )
                    .await?;
                Ok(handle)
            })
            .await;
            let handle = graph_result.map_err(|source| {
                let analysis = match &source {
                    WorkflowError::Canonical(
                        pse_operations::canonical::CanonicalError::AnalysisUnsettled {
                            intent, ..
                        },
                    ) => Some(intent.key.clone()),
                    _ => None,
                };
                transfer_error(source, analysis, Some(read_key.clone()))
            })?;
            transfer_checkpoint(TransferCheckpoint::AfterGraphAck, cancel).map_err(|source| {
                transfer_error(
                    source,
                    Some(handle.key().to_owned()),
                    Some(read_key.clone()),
                )
            })?;
            self.canonical_store()?
                .release_artifact_intent(artifact.header(), "audit", &staging_owner)
                .await
                .map_err(|source| {
                    transfer_error(
                        source.into(),
                        Some(handle.key().to_owned()),
                        Some(read_key.clone()),
                    )
                })?;
            artifact_read.release().await.map_err(|source| {
                transfer_error(source.into(), Some(handle.key().to_owned()), Some(read_key))
            })?;
            drop(protected_input);
            Ok(handle)
        })
    }
}
impl ModelingSolvePreparation {
    /// Persist original all-branch numerical incidence and conservative execution
    /// dependencies before dispatch. Neither edge kind certifies derivatives.
    pub async fn dependency_analysis(
        &self,
        controls: &AnalysisControls,
        cancel: CancellationToken,
    ) -> Result<AnalysisHandle, WorkflowError> {
        let activity = self.source.runtime.admit_activity()?;
        activity
            .scope(async {
                controls.validate()?;
                let runtime = &self.source.runtime;
                let owner = MemoryConsumer::new("canonical:dependency-analysis")
                    .register(&runtime.shared.pool());
                owner
                    .try_grow(128 * 1024 * 1024)
                    .map_err(pse_engine::EngineError::from)?;
                cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
                let plan = &self.model.case.compiled().plan;
                let structure = plan.structure();
                if structure.variables().len()
                    + structure.parameters().len()
                    + structure.rows().len()
                    + structure.objectives().len()
                    > ANALYSIS_NODES
                {
                    return Err(contract("analysis original structure exceeds bound"));
                }
                let mut graph = Graph::new();
                let mut nodes = BTreeMap::new();
                for variable in structure.variables() {
                    vertex(
                        &mut graph,
                        &mut nodes,
                        variable.port.id.to_string(),
                        "variable",
                    )?;
                }
                for parameter in structure.parameters() {
                    vertex(
                        &mut graph,
                        &mut nodes,
                        parameter.id.to_string(),
                        "parameter",
                    )?;
                }
                for row in structure.rows() {
                    vertex(&mut graph, &mut nodes, row.id.to_string(), "row")?;
                }
                for level in 0..structure.objectives().len() {
                    vertex(
                        &mut graph,
                        &mut nodes,
                        format!("objective:{level}"),
                        "objective",
                    )?;
                }
                let flow = plan
                    .visit_dependencies(&Arc::new(AtomicBool::new(false)), |dependency| {
                        let projected = (|| {
                            cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
                            let target_name = match dependency.target {
                                pse_math::binding::Target::Row(id) => id.to_string(),
                                pse_math::binding::Target::Objective(level) => {
                                    format!("objective:{level}")
                                }
                            };
                            let target = *nodes.get(&target_name).ok_or_else(|| {
                                contract("analysis target absent from original structure")
                            })?;
                            for (kind, symbols) in [
                                ("numerical_incidence", dependency.numerical),
                                ("execution_dependency", dependency.execution),
                            ] {
                                for symbol in symbols {
                                    let source = *nodes.get(&symbol.to_string()).ok_or_else(|| {
                                contract("analysis dependency absent from original selected values")
                            })?;
                                    edge(&mut graph, source, target, kind, None)?;
                                }
                            }
                            Ok::<(), WorkflowError>(())
                        })();
                        match projected {
                            Ok(()) => std::ops::ControlFlow::Continue(()),
                            Err(error) => std::ops::ControlFlow::Break(error),
                        }
                    })
                    .map_err(super::math)?;
                if let std::ops::ControlFlow::Break(error) = flow {
                    return Err(error);
                }
                let revision = self.source.recorded_revision()?.clone();
                let mut sources = vec![revision.clone()];
                let operations = match runtime.durability() {
                    Durability::Durable(operations) => operations.as_ref().clone(),
                    Durability::Ephemeral => super::Operations::from_store(
                        runtime.canonical_store()?.clone(),
                        "analysis",
                        super::LeasePolicy::default(),
                        runtime.shared.pool(),
                    ),
                };
                let physical_rows = operations
                    .put_physical_rows(runtime, &self.source.physical)
                    .await?;
                sources.extend(physical_rows.revisions.iter().cloned());
                sources.sort_by(|a, b| a.key.cmp(&b.key));
                sources.dedup_by(|a, b| a.key == b.key);
                let configuration =
                    serde_json::to_vec(&(1_u8, controls, self.admission_identity()?.to_hex()))
                        .map_err(|error| contract(error.to_string()))?;
                let analysis = runtime
                    .store_analysis_graph(
                        &revision,
                        &sources,
                        &[],
                        "original-incidence-execution-reachability:v1",
                        configuration,
                        graph,
                        nodes,
                        controls,
                    )
                    .await?;
                // Keep the admitted physical source protection until analysis roots acknowledge.
                drop(physical_rows);
                Ok(analysis)
            })
            .await
    }
}

#[cfg(all(test, feature = "canonical-tests"))]
mod canonical_analyses_server_unit {
    use super::*;
    use pse_relations::columnar::RelationRow;
    use pse_relations::generated::runtime::{
        canonical_analysis_edges, canonical_analysis_nodes, parametric_sensitivities,
    };
    #[tokio::test]
    async fn canonical_original_dependencies_and_retained_sensitivity_persist_exact_sources() {
        let runtime = super::super::durable_tests::durable_runtime();
        let (package, mut analysis) =
            super::super::durable_tests::package_on(&runtime, super::super::durable_tests::LINEAR)
                .await;
        analysis.bindings.demand.push("t".into());
        let cancel = crate::CancelSource::new();
        let original = package.prepare_analysis(&analysis, &cancel).await.unwrap();
        let parameter = original.model.case.compiled().plan.structure().parameters()[0].id;
        let variable = original.model.case.compiled().plan.structure().variables()[0]
            .port
            .id;
        analysis.solver.sensitivity = Some(crate::math::settings::SensitivityRequest {
            parameters: vec![parameter],
            reduced_hessian: false,
            propagation: None,
        });
        let prepared = package.prepare_analysis(&analysis, &cancel).await.unwrap();
        let controls = AnalysisControls {
            roots: vec![parameter.to_string()],
            direction: AnalysisDirection::Downstream,
        };
        let pre = prepared
            .dependency_analysis(&controls, CancellationToken::new())
            .await
            .unwrap();
        let receipt = runtime
            .canonical_store()
            .unwrap()
            .analysis(pre.key())
            .await
            .unwrap();
        assert!(receipt.active);
        assert_eq!(
            receipt.method,
            "original-incidence-execution-reachability:v1"
        );
        assert_eq!(receipt.revision, package.canonical_revision().unwrap().key);
        let nodes = canonical_analysis_nodes::Row::rows(&pre.nodes(None).await.unwrap()).unwrap();
        assert!(
            nodes
                .iter()
                .any(|node| node.semantic == parameter.to_string())
        );
        assert!(
            !nodes
                .iter()
                .any(|node| node.semantic == variable.to_string())
        );
        let edges = canonical_analysis_edges::Row::rows(&pre.edges(None).await.unwrap()).unwrap();
        assert!(edges.iter().any(|edge| edge.kind == "numerical_incidence"));
        assert!(edges.iter().any(|edge| edge.kind == "execution_dependency"));
        assert!(edges.iter().all(|edge| edge.evidence_artifact.is_none()));
        let replay = prepared
            .dependency_analysis(&controls, CancellationToken::new())
            .await
            .unwrap();
        assert_ne!(replay.key(), pre.key());
        let replay_receipt = runtime
            .canonical_store()
            .unwrap()
            .analysis(replay.key())
            .await
            .unwrap();
        assert_eq!(replay_receipt.revision, receipt.revision);
        assert_eq!(replay_receipt.method, receipt.method);
        assert_eq!(replay_receipt.configuration, receipt.configuration);
        assert_eq!(replay_receipt.input_digest, receipt.input_digest);
        assert_eq!(replay_receipt.node_count, receipt.node_count);
        assert_eq!(replay_receipt.edge_count, receipt.edge_count);
        let result = prepared.start().unwrap().wait().await.unwrap();
        assert!(result.usable(), "{:?}", result.report());
        let quantified = parametric_sensitivities::Row::rows(
            &result.table("runtime.parametric_sensitivities").unwrap(),
        )
        .unwrap();
        assert_eq!(quantified.len(), 1);
        assert_eq!(quantified[0].parameter_id, parameter);
        assert_eq!(quantified[0].target_id, variable);
        assert!((quantified[0].primal.unwrap() - 1.0).abs() < 1e-10);
        assert!(quantified[0].dual.is_none());
        let run = result.canonical_run_key().unwrap();
        let attempt = result.canonical_attempt_key().unwrap();
        let final_state = runtime
            .promote_scientific_artifact(
                run,
                attempt,
                super::super::artifacts::ScientificArtifactKind::FinalState,
                "user",
                "retained-final-state",
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let post = runtime
            .result_analysis(
                run,
                attempt,
                &AnalysisControls {
                    roots: vec![],
                    direction: AnalysisDirection::Downstream,
                },
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let sources = runtime
            .canonical_store()
            .unwrap()
            .analysis_run_sources(run)
            .await
            .unwrap();
        assert!(sources.len() >= 2);
        let nodes = canonical_analysis_nodes::Row::rows(&post.nodes(None).await.unwrap()).unwrap();
        assert_eq!(
            nodes
                .iter()
                .filter(|node| node.kind == "source_revision")
                .count(),
            sources.len()
        );
        let edges = canonical_analysis_edges::Row::rows(&post.edges(None).await.unwrap()).unwrap();
        let evidence = edges
            .iter()
            .find(|edge| edge.kind == "retained_root_parametric_primal")
            .unwrap();
        assert!(evidence.evidence_artifact.is_some());
        assert_eq!(evidence.evidence_row, Some(0));
        assert_eq!(evidence.evidence_field.as_deref(), Some("primal"));
        assert!(
            !edges
                .iter()
                .any(|edge| edge.kind == "retained_root_parametric_dual")
        );
        let reopened = runtime.analysis(post.key()).await.unwrap();
        assert_eq!(reopened.key(), post.key());
        assert_eq!(
            canonical_analysis_edges::Row::rows(&reopened.edges(None).await.unwrap()).unwrap(),
            edges
        );
        let cancelled = CancellationToken::new();
        cancelled.cancel();
        assert!(
            runtime
                .result_analysis(run, attempt, &controls, cancelled)
                .await
                .is_err()
        );
        let mut retired = false;
        for _ in 0..32 {
            if runtime
                .canonical_store()
                .unwrap()
                .forget_run_results(run)
                .await
                .is_ok()
            {
                retired = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert!(
            retired,
            "independent graph evidence does not retain the producer run"
        );
        let final_read = runtime
            .canonical_store()
            .unwrap()
            .read_artifact(&final_state.header().key)
            .await
            .unwrap();
        let mut preserved_variables = false;
        for ordinal in 0..final_state.header().block_count {
            let block = final_read.block(ordinal).await.unwrap();
            if block.relation == "runtime.solve_variables" {
                preserved_variables |= block.rows > 0;
                assert!(
                    final_state
                        .block(ordinal, CancellationToken::new())
                        .await
                        .is_ok()
                );
            }
        }
        final_read.release().await.unwrap();
        assert!(preserved_variables);
        let evidence_artifact = runtime
            .scientific_artifact(evidence.evidence_artifact.as_deref().unwrap())
            .await
            .unwrap();
        let retained = evidence_artifact
            .evidence(
                &ArtifactLocator {
                    artifact: evidence.evidence_artifact.clone().unwrap(),
                    block: evidence.evidence_block.unwrap(),
                    row: evidence.evidence_row,
                    field: evidence.evidence_field.clone(),
                },
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let retained = parametric_sensitivities::Row::rows(&retained).unwrap();
        assert_eq!(
            retained, quantified,
            "actual quantitative evidence remains identical after producer retirement"
        );
        // An independently retained evidence artifact survives removal of its
        // graph owner as well as the producer's result retirement.
        runtime
            .canonical_store()
            .unwrap()
            .retain_artifact(
                &evidence_artifact.header().key,
                "user",
                "independent-response-consumer",
            )
            .await
            .unwrap();
        let mut declarations = pse_authoring::language::parse(
            super::super::durable_tests::LINEAR,
            pse_ids::SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        declarations
            .iter_mut()
            .find(|row| row.name == "t" && row.value.binding.is_some())
            .unwrap()
            .value
            .binding
            .as_mut()
            .unwrap()
            .expression = Some("2".into());
        let revised = package.with_declarations(declarations).await.unwrap();
        let changed = revised
            .prepare_analysis(&analysis, &cancel)
            .await
            .unwrap()
            .dependency_analysis(&controls, CancellationToken::new())
            .await
            .unwrap();
        assert_ne!(changed.key(), pre.key());
        assert_eq!(
            runtime
                .canonical_store()
                .unwrap()
                .analysis(pre.key())
                .await
                .unwrap()
                .revision,
            receipt.revision
        );
        assert_eq!(
            runtime
                .canonical_store()
                .unwrap()
                .analysis(changed.key())
                .await
                .unwrap()
                .revision,
            revised.canonical_revision().unwrap().key
        );
        for _ in 0..5 {
            if runtime.forget_analysis_results(post.key()).await.unwrap() {
                break;
            }
        }
        assert!(runtime.analysis(post.key()).await.is_err());
        assert!(
            runtime
                .scientific_artifact(&evidence_artifact.header().key)
                .await
                .unwrap()
                .evidence(
                    &ArtifactLocator {
                        artifact: evidence_artifact.header().key.clone(),
                        block: evidence.evidence_block.unwrap(),
                        row: evidence.evidence_row,
                        field: evidence.evidence_field.clone()
                    },
                    CancellationToken::new(),
                )
                .await
                .is_ok(),
            "graph retirement cannot release another consumer's retained artifact"
        );

        let mut retired = false;
        for _ in 0..32 {
            if runtime
                .canonical_store()
                .unwrap()
                .forget_run_results(run)
                .await
                .is_ok()
            {
                retired = true;
                break;
            }
            tokio::task::yield_now().await;
        }
        assert!(
            retired,
            "retirement remains blocked after exact analysis withdrawal and dropped read owners"
        );
    }
}

#[cfg(test)]
mod analysis_bounds_unit {
    use super::*;

    #[test]
    fn analysis_controls_refuse_before_graph_preparation() {
        let mut controls = AnalysisControls {
            roots: vec!["r".repeat(128); 64],
            direction: AnalysisDirection::Upstream,
        };
        assert!(controls.validate().is_ok());
        controls.roots.push("extra".into());
        assert!(controls.validate().is_err());
        controls.roots = vec!["r".repeat(129)];
        assert!(controls.validate().is_err());
    }

    #[test]
    fn analysis_exact_caps_preserve_complete_membership_and_refuse_overflow() {
        let mut graph = Graph::new();
        let mut nodes = BTreeMap::new();
        for ordinal in 0..ANALYSIS_NODES {
            vertex(&mut graph, &mut nodes, format!("v:{ordinal}"), "variable").unwrap();
        }
        assert_eq!(graph.node_count(), ANALYSIS_NODES);
        // An existing vertex remains valid at the cap; a small selection cannot
        // turn an incompletely admitted source graph into complete evidence.
        assert!(vertex(&mut graph, &mut nodes, "v:0".into(), "variable").is_ok());
        assert!(vertex(&mut graph, &mut nodes, "extra".into(), "variable").is_err());
        let root = nodes["v:0"];
        let evidence = |row| ArtifactLocator {
            artifact: "cap-fixture".into(),
            block: 0,
            row: Some(row),
            field: Some("value".into()),
        };
        for ordinal in 0..ANALYSIS_EDGES {
            edge(
                &mut graph,
                root,
                root,
                "incidence",
                Some(evidence(ordinal as u64)),
            )
            .unwrap();
        }
        assert_eq!(graph.edge_count(), ANALYSIS_EDGES);
        // An exact duplicate remains valid at the cap, while distinct scientific
        // row evidence must consume its own membership slot.
        assert!(edge(&mut graph, root, root, "incidence", Some(evidence(0))).is_ok());
        assert_eq!(graph.edge_count(), ANALYSIS_EDGES);
        assert!(edge(&mut graph, root, root, "incidence", None).is_err());
        let controls = AnalysisControls {
            roots: vec!["v:0".into()],
            direction: AnalysisDirection::Upstream,
        };
        assert_eq!(
            selected(&graph, &nodes, &controls).unwrap(),
            BTreeSet::from([root])
        );
        assert_eq!(graph.edge_count(), ANALYSIS_EDGES);
    }
}

#[cfg(test)]
#[path = "analyses-response-tests.rs"]
mod response_evidence_unit;

#[cfg(all(test, feature = "canonical-tests"))]
#[path = "analyses-transfer-tests.rs"]
mod transfer_unit;

#[cfg(test)]
mod analysis_ack_unit {
    use super::*;
    #[test]
    fn analysis_ack_requires_generated_whole_row_equality_and_prepaid_clone() {
        let original = pse_operations::canonical_analyses::Analysis {
            key: "original".into(),
            revision: "source".into(),
            method: "original-method".into(),
            configuration: vec![1, 2].into(),
            input_digest: "input".into(),
            primary_problem: "problem".into(),
            primary_authority: "authority".into(),
            creation_nonce: "nonce".into(),
            creation_request_digest: "exact-request".into(),
            creation_expires_at: 1,
            interpretation: "original-interpretation".into(),
            node_count: 1,
            edge_count: 2,
            active: false,
            retiring: false,
        };
        let pool: Arc<dyn pse_columnar::MemoryPool> =
            Arc::new(pse_columnar::GreedyMemoryPool::new(4096));
        let mut ack = original.clone();
        ack.active = true;
        check_analysis_ack(&pool, &original, &ack).unwrap();
        for change in 0..5 {
            let mut wrong = ack.clone();
            match change {
                0 => wrong.active = false,
                1 => wrong.retiring = true,
                2 => wrong.node_count += 1,
                3 => wrong.configuration = vec![9].into(),
                _ => wrong.method.push('x'),
            }
            assert!(check_analysis_ack(&pool, &original, &wrong).is_err());
        }
        assert_eq!(pool.reserved(), 0);
        let refused: Arc<dyn pse_columnar::MemoryPool> =
            Arc::new(pse_columnar::GreedyMemoryPool::new(1));
        assert!(check_analysis_ack(&refused, &original, &ack).is_err());
        assert_eq!(refused.reserved(), 0);
    }
}
