// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Generated declarations are authoritative; compiler inputs are a checked projection.
use super::{Runtime, WorkflowError, contract, math, relation};
use crate::math::Workspace;
use pse_compiler::{
    typed_math::{Domain, Formal, Group, ProviderCall},
    workspace::{Case, Definition, Inputs, WorkspaceLimits},
};
use pse_ids::{ContentHash, FramedHasher, SemanticId};
use pse_kernels::{Port, Registration};
use pse_math::binding::*;
use pse_model::{HeapUsage, SemanticFrame};
use pse_quantity::{PhysicalPreconditions, QuantityRegistry, QuantityTypeId, UnitId};
use pse_relations::columnar::RelationRow;
use pse_relations::{
    columnar::FieldCheckedBatch,
    generated::{authored::computation_models as wire, enums},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
/// The generated model row, including finite/ragged lexical context.
pub type ModelDeclaration = wire::Row;
/// One generated selected-case declaration.
pub type CaseDeclaration = wire::AuthoredComputationModelsFieldCasesItem;
/// One generated reusable body declaration.
pub type DefinitionDeclaration = wire::AuthoredComputationModelsFieldDefinitionsItem;

/// Immutable physical declarations retaining their actual admitted source rows.
#[derive(Clone, Debug)]
pub struct PhysicalContext {
    pub(crate) quantities: Arc<QuantityRegistry>,
    pub(crate) preconditions: Arc<PhysicalPreconditions>,
    pub(crate) sources: BTreeMap<pse_schema::model::RelationKey, FieldCheckedBatch>,
    pub(crate) key: ContentHash,
    pub(crate) origin: &'static str,
    pub(super) _inventory: Option<Arc<crate::physical::PhysicalInventory>>,
}
impl PhysicalContext {
    /// Custom contexts retain the very rows admitted by PhysicalInventory.
    pub fn admitted(inventory: Arc<crate::physical::PhysicalInventory>) -> Self {
        let quantities = Arc::new(inventory.quantities().clone());
        let preconditions = inventory.compiler_preconditions();
        let key = pse_compiler::workspace::physical_identity(&quantities, &preconditions);
        Self {
            quantities,
            preconditions,
            key,
            sources: inventory.source_batches().clone(),
            origin: "admitted_source_rows",
            _inventory: Some(inventory),
        }
    }
    /// Full exact physical context identity, not a caller's revision assertion.
    pub fn identity(&self) -> ContentHash {
        self.key
    }
}
/// A named native provider selection; Python never supplies callbacks.
#[derive(Clone, Debug)]
pub struct ProviderBinding {
    /// Admitted native implementation factory.
    pub registration: Registration,
    /// Selected scalar output.
    pub output: usize,
}
/// Mutable draft containing only generated declaration values.
#[derive(Debug)]
pub struct ModelBuilder {
    pub(crate) sources: super::sources::Sources,
    runtime: Runtime,
    pub(crate) row: ModelDeclaration,
    physical: PhysicalContext,
    providers: BTreeMap<String, ProviderBinding>,
    workspace: Option<Workspace>,
    document_inventory: BTreeMap<SemanticId, FieldCheckedBatch>,
}
impl ModelBuilder {
    pub(crate) fn physical_context(&self) -> &PhysicalContext {
        &self.physical
    }

    /// Replace the accompanying generated source relations atomically on freeze.
    pub fn sources_mut(&mut self) -> &mut super::SourceDeclarations {
        &mut self.sources
    }

    pub(super) fn new(
        runtime: Runtime,
        id: SemanticId,
        name: String,
        physical: PhysicalContext,
    ) -> Self {
        Self::from_declaration(
            runtime,
            wire::Row {
                model_id: id,
                name,
                definitions: vec![],
                domains: vec![],
                groups: vec![],
                cases: vec![],
            },
            physical,
        )
    }
    /// A typed declaration and a document row enter this identical admission path.
    pub fn from_declaration(
        runtime: Runtime,
        row: ModelDeclaration,
        physical: PhysicalContext,
    ) -> Self {
        Self {
            runtime,
            row,
            physical,
            providers: BTreeMap::new(),
            workspace: None,
            document_inventory: BTreeMap::new(),
            sources: Default::default(),
        }
    }
    /// Add one reusable source definition; duplicates are rejected atomically on freeze.
    pub fn definition(&mut self, row: DefinitionDeclaration) -> &mut Self {
        self.row.definitions.push(row);
        self
    }
    /// Add a complete selected case, including fixed/free treatment and physical bounds.
    pub fn case(&mut self, case: CaseBuilder) -> &mut Self {
        self.row.cases.push(case.row);
        self
    }
    /// Edit the generated draft directly; it never mutates a previously frozen revision.
    pub fn declaration_mut(&mut self) -> &mut ModelDeclaration {
        &mut self.row
    }
    /// Select a registered native provider by the exact name used by the DSL.
    pub fn provider(
        &mut self,
        name: String,
        provider: ProviderBinding,
    ) -> Result<&mut Self, WorkflowError> {
        if self.providers.contains_key(&name) {
            return Err(contract("duplicate native provider name"));
        }
        self.providers.insert(name, provider);
        Ok(self)
    }
    /// Validate generated fields, identities, bindings and values before publishing an immutable revision.
    pub fn freeze(mut self) -> Result<ModelRevision, WorkflowError> {
        let case_ids = self.row.cases.iter().map(|c| c.case_id).collect();
        let (mut selected, mut admission) = self.sources.composition.select(
            self.row.model_id,
            &case_ids,
            &self
                .sources
                .providers
                .iter()
                .filter_map(|p| p.material_system_id)
                .collect(),
        )?;
        selected.extend(self.row.definitions.iter().map(|d| d.definition_id));
        for c in &self.row.cases {
            selected.extend(c.variables.iter().map(|v| v.port.symbol_id));
            selected.extend(c.parameters.iter().map(|p| p.symbol_id));
            selected.extend(c.instances.iter().map(|i| i.instance_id));
            selected.extend(c.rows.iter().map(|r| r.row_id));
        }
        super::composition::classify_documents(
            &self.document_inventory,
            &selected,
            &self.physical,
            &self.runtime.registry,
            &mut admission,
        )?;
        self.sources.canonicalize(self.row.model_id)?;
        for row in &self.sources.providers {
            let provider =
                super::sources::factory(row, &self.physical.quantities, &self.sources.composition)?;
            if self.providers.get(&row.name).is_some_and(|old| {
                old.registration.spec().key() != provider.registration.spec().key()
                    || old.output != provider.output
            }) {
                return Err(contract(
                    "provider factory conflicts with existing registration",
                ));
            }
            self.providers.insert(row.name.clone(), provider);
        }
        for law in &self.sources.valve_laws {
            let neutral = self
                .physical
                .quantities
                .neutral_dimensionless()
                .ok_or_else(|| {
                    contract("directional valve requires an explicitly designated neutral quantity")
                })?;
            let package = pse_kernels::valve::DirectionalValve::new(
                pse_ids::named_id(law.model_id, &law.name),
                neutral,
                &self.physical.quantities,
            )
            .map_err(|e| contract(e.to_string()))?;
            let registration = Registration::new(Arc::new(package), &self.physical.quantities)
                .map_err(|e| contract(e.to_string()))?;
            if self
                .providers
                .insert(
                    law.name.clone(),
                    ProviderBinding {
                        registration,
                        output: 0,
                    },
                )
                .is_some()
            {
                return Err(contract("duplicate valve provider binding"));
            }
        }
        let bytes = self
            .row
            .owned_bytes()
            .saturating_add(self.sources.bytes())
            .saturating_add(super::balances::projection_bytes(&self.sources));
        if bytes > self.runtime.shared.budget().math.workspace_bytes / 4 {
            return Err(contract("model declaration exceeds workspace allowance"));
        }
        let reservation = pse_columnar::MemoryConsumer::new("workflow:model-revision")
            .register(&self.runtime.shared.pool());
        reservation
            .try_grow(bytes.saturating_mul(4).saturating_add(4096))
            .map_err(|e| WorkflowError::Math(e.into()))?;
        // Validate every added generated source family before publishing a revision.
        let source_relations: BTreeSet<_> = self
            .sources
            .tables(&self.runtime.registry)?
            .keys()
            .copied()
            .chain([wire::RELATION_ID])
            .collect();
        let mut builder =
            wire::Builder::with_registry(&self.runtime.registry, 1).map_err(relation)?;
        builder.push(self.row.clone()).map_err(relation)?;
        let batch = builder
            .finish()
            .map_err(relation)?
            .retained(
                &self.runtime.shared.pool(),
                &pse_columnar::CancellationToken::new(),
            )
            .map_err(relation)?;
        if self.row.cases.is_empty() {
            return Err(contract("model must declare at least one selected case"));
        }
        let mut projection = self.row.clone();
        let lowered = super::composition::lower(
            &self.sources.composition,
            &mut projection,
            &self.physical,
            &self.providers,
            &self.runtime.registry,
        )?;
        let mut resolved_sources = self.sources.clone();
        resolved_sources.balances.extend(lowered.balances.clone());
        resolved_sources.numerics.extend(lowered.numerics.clone());
        super::reactions::project(&mut projection, &mut resolved_sources, &self.physical)?;
        resolved_sources.canonicalize(self.row.model_id)?;
        super::balances::project(&mut projection, &resolved_sources)?;
        for case in &mut projection.cases {
            let before = case.instances.len();
            case.instances.retain(|i| !i.contributions.is_empty());
            admission.push(super::AdmissionEntry {
                relation: format!(
                    "authored.computation_models.instances:{}",
                    case.case_id.to_hex()
                ),
                selected: case.instances.len(),
                nonexecuting: before - case.instances.len(),
            });
        }
        let projection_bytes = projection
            .owned_bytes()
            .saturating_add(resolved_sources.bytes());
        if projection_bytes > self.runtime.shared.budget().math.workspace_bytes / 2 {
            return Err(contract("expanded model exceeds workspace allowance"));
        }

        let retained_bytes = bytes
            .saturating_mul(4)
            .saturating_add(projection_bytes)
            .saturating_add(4096);
        if retained_bytes > reservation.size() {
            reservation
                .try_grow(retained_bytes - reservation.size())
                .map_err(|e| WorkflowError::Math(e.into()))?;
        }
        let owner = pse_columnar::AllocationLease::new(reservation);
        let mut cases = BTreeMap::new();
        let mut case_keys = BTreeMap::new();
        let mut workspace = self.workspace;
        for case in &projection.cases {
            let mut inputs = project(&projection, case, &self.physical, &self.providers)?;
            for law in &self.sources.valve_laws {
                if case.instances.iter().any(|i| {
                    projection.definitions.iter().any(|d| {
                        d.definition_id == i.definition_id && d.providers.contains(&law.name)
                    })
                }) {
                    use pse_quantity::{BaseDimension, DimensionVector};
                    let pressure = DimensionVector::base(BaseDimension::Mass)
                        .div(&DimensionVector::base(BaseDimension::Length))
                        .and_then(|d| d.div(&DimensionVector::base(BaseDimension::Time)))
                        .and_then(|d| d.div(&DimensionVector::base(BaseDimension::Time)))
                        .map_err(|e| contract(e.to_string()))?;
                    let width = case
                        .parameters
                        .iter()
                        .find(|p| p.symbol_id == law.transition_width_id)
                        .and_then(|p| self.physical.quantities.unit(p.unit_id.into()).ok());
                    if width.is_none_or(|u| u.dimension != pressure || u.is_affine)
                        || case
                            .instances
                            .iter()
                            .filter(|i| {
                                projection.definitions.iter().any(|d| {
                                    d.definition_id == i.definition_id
                                        && d.providers.contains(&law.name)
                                })
                            })
                            .any(|i| {
                                !i.slots
                                    .iter()
                                    .any(|s| s.source_id == law.transition_width_id)
                            })
                    {
                        return Err(contract(
                            "directional valve width must be a pressure interval bound to each law instance",
                        ));
                    }
                    if !case
                        .parameters
                        .iter()
                        .any(|p| p.symbol_id == law.transition_width_id)
                        || inputs
                            .values
                            .get(&law.transition_width_id)
                            .is_none_or(|v| !v.is_finite() || *v <= 0.0)
                    {
                        return Err(contract(
                            "directional valve transition width must be an authored fixed positive pressure input",
                        ));
                    }
                }
            }
            inputs.flows = lowered.flows.clone();
            if cases
                .insert(case.case_id, Arc::new(inputs.clone()))
                .is_some()
            {
                return Err(contract("duplicate case identity"));
            }
            // Same input admission used by the incremental compiler, without compiling an evaluator.
            let mut admission_workspace = pse_compiler::workspace::CompilerWorkspace::new(
                inputs.clone(),
                WorkspaceLimits::default(),
            )
            .map_err(|e| WorkflowError::Math(e.into()))?;
            let plan = admission_workspace
                .admit_selected_case(case.case_id)
                .map_err(|e| WorkflowError::Math(e.into()))?;
            let mut case_hash = FramedHasher::new("pse.native.selected-case.v1");
            case_hash.id(&case.case_id).hash(&plan.structure().key());
            inputs.values.frame(&mut case_hash);
            case_keys.insert(case.case_id, case_hash.finish_hash());
            if workspace.is_none() {
                workspace = Some(
                    self.runtime
                        .native()
                        .workspace(inputs, WorkspaceLimits::default())?,
                );
            }
        }
        let mut h = FramedHasher::new("pse.native.model-revision.v2");
        h.id(&self.row.model_id).hash(&self.physical.key);
        for (id, key) in &case_keys {
            h.id(id).hash(key);
        }
        for (id, flow) in &lowered.flows {
            let graph = pse_structural::flowsheet::FlowGraph::admit(
                flow.clone(),
                &self.physical.quantities,
                pse_structural::projection::GraphLimits {
                    nodes: 4096,
                    edges: 16384,
                },
            )
            .map_err(|e| super::composition::invalid([*id], e.to_string()))?;
            h.id(id).hash(&graph.key());
        }
        resolved_sources.balances.frame(&mut h);
        resolved_sources.numerics.frame(&mut h);
        resolved_sources.scaling_bindings.frame(&mut h);
        resolved_sources.scaling_defaults.frame(&mut h);
        self.sources.composition.frame_material_contract(&mut h);
        self.sources.dynamics.frame(&mut h);
        self.sources.fits.frame(&mut h);
        self.sources.observations.frame(&mut h);
        self.sources.datasets.frame(&mut h);
        h.hash(
            &pse_schema::fingerprint::semantic_product(&self.runtime.registry, &source_relations)
                .map_err(pse_relations::RelationError::from)
                .map_err(relation)?,
        );
        for (name, p) in &self.providers {
            h.str(name)
                .hash(&p.registration.spec().identity())
                .u64(p.output as u64);
        }
        Ok(ModelRevision(Arc::new(Revision {
            runtime: self.runtime,
            row: self.row,
            sources: self.sources,
            resolved_sources,
            admission,
            projection,
            bindings: lowered.bindings,
            physical: self.physical,
            providers: self.providers,
            cases,
            case_keys,
            key: h.finish_hash(),
            workspace: workspace.ok_or_else(|| contract("missing compiler workspace"))?,
            batch,
            _owner: owner,
        })))
    }
}
/// Case-specific convenience over the same generated case record.
#[derive(Clone, Debug)]
pub struct CaseBuilder {
    row: CaseDeclaration,
}
impl CaseBuilder {
    /// Start a selected case. Sources must explicitly declare all variables/values and rows.
    pub fn new(id: SemanticId, name: String) -> Self {
        Self {
            row: CaseDeclaration {
                case_id: id,
                name,
                variables: vec![],
                parameters: vec![],
                instances: vec![],
                rows: vec![],
                objective: None,
                values: vec![],
            },
        }
    }
    /// Wrap a generated document or typed case declaration.
    pub fn from_declaration(row: CaseDeclaration) -> Self {
        Self { row }
    }
    /// Access the generated draft; no parallel case DTO exists.
    pub fn declaration_mut(&mut self) -> &mut CaseDeclaration {
        &mut self.row
    }
}
#[derive(Debug)]
pub(crate) struct Revision {
    pub case_keys: BTreeMap<SemanticId, ContentHash>,
    pub admission: Vec<super::AdmissionEntry>,
    pub projection: ModelDeclaration,
    pub bindings: Vec<super::composition::lower::Binding>,
    pub sources: super::sources::Sources,
    pub resolved_sources: super::sources::Sources,
    pub runtime: Runtime,
    pub row: ModelDeclaration,
    pub physical: PhysicalContext,
    pub providers: BTreeMap<String, ProviderBinding>,
    pub cases: BTreeMap<SemanticId, Arc<Inputs>>,
    pub key: ContentHash,
    pub workspace: Workspace,
    pub batch: FieldCheckedBatch,
    _owner: Arc<pse_columnar::AllocationLease>,
}
/// Immutable declarations and source identity; revisions may share only the compiler's derived cache.
#[derive(Clone, Debug)]
pub struct ModelRevision(pub(crate) Arc<Revision>);
impl ModelRevision {
    /// Selected values and physical structure, excluding display names and source prose.
    pub fn case_identity(&self, case: SemanticId) -> Option<ContentHash> {
        self.0.case_keys.get(&case).copied()
    }
    /// Exhaustive declaration accounting at selected admission.
    pub fn admission(&self) -> &[super::AdmissionEntry] {
        &self.0.admission
    }
    /// Inspect the derived scalar declarations without replacing authored templates.
    pub fn projection(&self) -> &ModelDeclaration {
        &self.0.projection
    }
    /// Inspect the binding of every active template scalar.
    pub fn scalar_bindings(&self) -> &[super::ScalarBinding] {
        &self.0.bindings
    }
    /// Prepare initialization using this immutable revision under the shared workspace lock.
    #[cfg(feature = "solver-kinsol")]
    pub async fn prepare_initialization(
        &self,
        case: SemanticId,
        profile: crate::math::initialization::InitializationProfile,
        compiler: pse_compiler::workspace::Profile,
    ) -> Result<super::PreparedInitializationStrategy, WorkflowError> {
        let inputs = self
            .0
            .cases
            .get(&case)
            .ok_or_else(|| super::composition::invalid([case], "unknown selected case"))?;
        let order = if profile.controls.hessian == pse_backend_native::solve::HessianMode::Exact {
            pse_kernels::DerivativeOrder::Second
        } else {
            pse_kernels::DerivativeOrder::First
        };
        let prepared = self
            .0
            .runtime
            .native()
            .prepare_initialization(
                self.0.workspace.clone(),
                inputs.as_ref().clone(),
                case,
                compiler,
                order,
            )
            .await?;
        prepared.validate_profile(
            &CaseValues {
                scalars: inputs.values.clone(),
            },
            &profile,
        )?;
        Ok(super::PreparedInitializationStrategy {
            revision: self.clone(),
            case,
            prepared,
            profile,
        })
    }
    /// Prepare this revision's flow, independent of the last use of a shared workspace.
    pub async fn prepare_flow(
        &self,
        case: SemanticId,
        flow: SemanticId,
    ) -> Result<crate::math::flows::PreparedFlow, WorkflowError> {
        let inputs = self
            .0
            .cases
            .get(&case)
            .ok_or_else(|| super::composition::invalid([case], "unknown selected case"))?;
        Ok(self
            .0
            .runtime
            .native()
            .prepare_flow(self.0.workspace.clone(), inputs.as_ref().clone(), flow)
            .await?)
    }
    /// Checked retained source row for Arrow consumers and durable publication.
    pub fn declaration_batch(&self) -> FieldCheckedBatch {
        self.0.batch.clone()
    }
    /// Physical balances derived solely from the retained declarations.
    pub fn resolved_balances(&self) -> &[super::BalanceDeclaration] {
        &self.0.resolved_sources.balances
    }
    /// Complete current-format declaration identity.
    pub fn identity(&self) -> ContentHash {
        self.0.key
    }
    /// Original generated declaration values.
    /// Exact authored provider, dynamics and observation declarations.
    pub fn source_declarations(&self) -> &super::SourceDeclarations {
        &self.0.sources
    }
    /// Original immutable generated model declaration.
    pub fn declaration(&self) -> &ModelDeclaration {
        &self.0.row
    }
    /// A separate editable draft; failed admission leaves this revision untouched.
    pub fn edit(&self) -> ModelBuilder {
        ModelBuilder {
            runtime: self.0.runtime.clone(),
            row: self.0.row.clone(),
            sources: self.0.sources.clone(),
            physical: self.0.physical.clone(),
            providers: self.0.providers.clone(),
            workspace: Some(self.0.workspace.clone()),
            document_inventory: BTreeMap::new(),
        }
    }
}
fn unique<K: Ord, V>(
    rows: impl IntoIterator<Item = (K, V)>,
    name: &str,
) -> Result<BTreeMap<K, V>, WorkflowError> {
    let mut out = BTreeMap::new();
    for (k, v) in rows {
        if out.insert(k, v).is_some() {
            return Err(contract(format!("duplicate {name}")));
        }
    }
    Ok(out)
}
fn port(id: SemanticId, quantity: SemanticId, unit: SemanticId) -> Port {
    Port {
        id,
        quantity: QuantityTypeId::from_id(quantity),
        unit: UnitId::from_id(unit),
    }
}
fn project(
    model: &ModelDeclaration,
    case: &CaseDeclaration,
    physical: &PhysicalContext,
    providers: &BTreeMap<String, ProviderBinding>,
) -> Result<Inputs, WorkflowError> {
    let mut definitions = BTreeMap::new();
    for d in &model.definitions {
        let value = Definition {
            sources: d.sources.clone(),
            formals: d
                .formals
                .iter()
                .map(|f| Formal {
                    path: f.path.clone(),
                    quantity: f.quantity_id.into(),
                })
                .collect(),
            domains: d.domains.clone(),
            groups: d.groups.clone(),
            providers: d.providers.clone(),
            units: unique(
                d.units
                    .iter()
                    .map(|u| (u.spelling.clone(), u.unit_id.into())),
                "unit spelling",
            )?,
            literals: unique(
                d.literals
                    .iter()
                    .map(|l| ((l.start as u32, l.end as u32), l.quantity_id.into())),
                "literal span",
            )?,

            limits: Default::default(),
        };
        if definitions.insert(d.definition_id, value).is_some() {
            return Err(contract("duplicate definition"));
        }
    }
    let mut domains = BTreeMap::new();
    for d in &model.domains {
        let kind = pse_quantity::DomainKind::parse(d.kind.as_str())
            .ok_or_else(|| contract("unknown physical domain kind"))?;
        let members = FiniteDomain::new(d.domain_id, d.members.clone(), 4096).map_err(math)?;
        if domains
            .insert(d.name.clone(), Domain { members, kind })
            .is_some()
        {
            return Err(contract("duplicate domain name"));
        }
    }
    let mut groups = BTreeMap::new();
    for g in &model.groups {
        let slots = unique(
            g.slots.iter().map(|s| (s.members.clone(), s.slot as usize)),
            "group tuple",
        )?;
        if groups
            .insert(
                g.name.clone(),
                Group {
                    quantity: g.quantity_id.into(),
                    axes: g.axes.clone(),
                    slots,
                },
            )
            .is_some()
        {
            return Err(contract("duplicate group name"));
        }
    }
    let variables = case
        .variables
        .iter()
        .map(|v| Variable {
            port: port(v.port.symbol_id, v.port.quantity_id, v.port.unit_id),
            fixed: v.fixed,
            domain: match v.domain {
                enums::NativeVariableDomain::Continuous => VariableDomain::Continuous,
                enums::NativeVariableDomain::Integer => VariableDomain::Integer,
                enums::NativeVariableDomain::Binary => VariableDomain::Binary,
                enums::NativeVariableDomain::SemiContinuous => VariableDomain::SemiContinuous,
                enums::NativeVariableDomain::SemiInteger => VariableDomain::SemiInteger,
            },
            lower: v.lower,
            upper: v.upper,
        })
        .collect::<Vec<_>>();
    let parameters = case
        .parameters
        .iter()
        .map(|v| port(v.symbol_id, v.quantity_id, v.unit_id))
        .collect::<Vec<_>>();
    let ports = unique(
        variables
            .iter()
            .map(|v| (v.port.id, v.port.clone()))
            .chain(parameters.iter().map(|p| (p.id, p.clone()))),
        "scalar identity",
    )?;
    let mut instances = vec![];
    let mut selected = BTreeMap::new();
    for i in &case.instances {
        if !definitions.contains_key(&i.definition_id) {
            return Err(contract("instance names missing definition"));
        }
        let mut slots = vec![];
        for (ordinal, s) in i.slots.iter().enumerate() {
            let source = ports
                .get(&s.source_id)
                .ok_or_else(|| contract("slot names missing source scalar"))?;
            let formal = port(source.id, s.formal_quantity_id, s.formal_unit_id);
            if definitions[&i.definition_id]
                .formals
                .get(ordinal)
                .is_none_or(|f| f.quantity != formal.quantity)
            {
                return Err(contract("slot formal differs from reusable definition"));
            }
            slots.push(SlotBinding::new(source, &formal, &physical.quantities).map_err(math)?);
        }
        instances.push(InstanceBinding {
            instance: i.instance_id,
            body: ContentHash::from_bytes([0; 32]),
            slots,
            contributions: i
                .contributions
                .iter()
                .map(|c| Contribution {
                    output: c.output as usize,
                    target: c.row_id.map_or(Target::Objective, Target::Row),
                    scale: c.scale,
                })
                .collect(),
        });
        if selected.insert(i.instance_id, i.definition_id).is_some() {
            return Err(contract("duplicate instance"));
        }
    }
    let rows = case
        .rows
        .iter()
        .map(|r| Row {
            id: r.row_id,
            quantity: r.quantity_id.into(),
            lower: r.lower.unwrap_or(f64::NEG_INFINITY),
            upper: r.upper.unwrap_or(f64::INFINITY),
        })
        .collect();
    let objective = case.objective.as_ref().map(|o| Objective {
        quantity: o.quantity_id.into(),
        sense: match o.sense {
            enums::NativeObjectiveSense::Minimize => ObjectiveSense::Minimize,
            enums::NativeObjectiveSense::Maximize => ObjectiveSense::Maximize,
        },
    });
    let structure = Arc::new(
        CaseStructure::new(
            variables,
            parameters,
            instances,
            rows,
            objective,
            CaseLimits::default(),
        )
        .map_err(math)?,
    );
    let values = unique(
        case.values.iter().map(|v| (v.symbol_id, v.value)),
        "case value",
    )?;
    structure
        .validate_values(&CaseValues {
            scalars: values.clone(),
        })
        .map_err(math)?;
    let providers = providers
        .iter()
        .map(|(name, p)| {
            (
                name.clone(),
                ProviderCall {
                    descriptor: p.registration.descriptor(),
                    output: p.output,
                },
            )
        })
        .collect();
    Ok(Inputs {
        flows: BTreeMap::new(),
        quantities: physical.quantities.clone(),
        preconditions: physical.preconditions.clone(),
        definitions,
        domains,
        groups,
        providers,
        cases: BTreeMap::from([(
            case.case_id,
            Case {
                structure,
                definitions: selected,
            },
        )]),
        values,
    })
}
impl Runtime {
    /// Project registry-parsed documents into the same typed builders, without a YAML round trip.
    pub fn models_from_documents(
        &self,
        documents: &crate::authoring_driver::document::OwnedDocumentSet,
        physical: PhysicalContext,
    ) -> Result<Vec<ModelBuilder>, WorkflowError> {
        documents.validate_registry(&self.registry)?;
        let batches =
            crate::authoring_driver::p1::source_batches(documents.bundles(), &self.registry)?;
        let batch = batches
            .get(&wire::RELATION_ID)
            .ok_or_else(|| contract("documents contain no native model declarations"))?;
        let view = wire::View::from_checked(batch).map_err(relation)?;
        if batch.batch().num_rows() == 0 {
            return Err(contract(
                "native workflow requires computation_models declarations; legacy template pipelines are not executed",
            ));
        }
        (0..batch.batch().num_rows())
            .map(|i| {
                let mut builder = ModelBuilder::from_declaration(
                    self.clone(),
                    view.row(i).map_err(relation)?,
                    physical.clone(),
                );
                macro_rules! load {
                    ($module:ident,$field:ident,$filter:expr) => {
                        if let Some(batch) =
                            batches.get(&pse_relations::generated::authored::$module::RELATION_ID)
                        {
                            let rows =
                                pse_relations::generated::authored::$module::Row::rows(batch)
                                    .map_err(relation)?;
                            builder.sources.$field = rows.into_iter().filter($filter).collect();
                        }
                    };
                }
                let model = builder.row.model_id;
                load!(physical_balances, balances, |r| r.model_id == model);
                load!(numerical_requirements, numerics, |r| r.model_id == model);
                load!(provider_scaling_bindings, scaling_bindings, |r| r.model_id
                    == model);
                let packages: BTreeSet<_> = builder
                    .sources
                    .scaling_bindings
                    .iter()
                    .map(|b| b.property_package_id)
                    .collect();
                load!(default_scaling, scaling_defaults, |r| packages
                    .contains(&r.property_package_id));
                load!(dynamic_cases, dynamics, |r| r.model_id == model);
                load!(fit_cases, fits, |r| r.model_id == model);
                load!(native_providers, providers, |r| r.model_id == model);
                load!(directional_valve_laws, valve_laws, |r| r.model_id == model);
                load!(observations, observations, |_| true);
                load!(datasets, datasets, |_| true);
                builder.sources.composition =
                    super::composition::CompositionDeclarations::load(&batches)?;
                builder.document_inventory = batches.clone();
                Ok(builder)
            })
            .collect()
    }
}
impl Runtime {
    /// Admit the exact physical sources through the existing registry/session boundary.
    pub async fn physical_from_documents(
        &self,
        documents: &crate::authoring_driver::document::OwnedDocumentSet,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<PhysicalContext, WorkflowError> {
        documents.validate_registry(&self.registry)?;
        let batches =
            crate::authoring_driver::p1::source_batches(documents.bundles(), &self.registry)?;
        let keys = crate::physical::input_keys(&self.registry);
        let roots = keys
            .iter()
            .filter_map(|key| {
                self.registry
                    .relation(&key.qualified_name())
                    .map(|spec| spec.id)
            })
            .collect();
        let support = pse_schema::product::support_closure(&self.registry, &roots)
            .map_err(pse_relations::RelationError::from)
            .map_err(relation)?;
        let mut physical = BTreeMap::new();
        let mut retained_support = BTreeMap::new();
        for (id, batch) in batches {
            let spec = self
                .registry
                .relation_by_id(id)
                .ok_or_else(|| contract("unknown physical declaration"))?;
            if keys.contains(&spec.key) {
                physical.insert(spec.key, batch);
            } else if support.contains(&id) {
                retained_support.insert(
                    spec.key,
                    batch
                        .retained(&self.shared.pool(), cancel)
                        .map_err(relation)?,
                );
            }
        }
        let session = self
            .sessions
            .candidate_checked(physical, self.registry.clone(), cancel)?;
        let inventory = crate::physical::PhysicalInventory::load(&session, &self.registry, cancel)
            .await
            .map_err(|e| pse_engine::EngineError::Semantic(Arc::new(e)))?;
        let mut context = PhysicalContext::admitted(Arc::new(inventory));
        context.sources.extend(retained_support);
        Ok(context)
    }
}
