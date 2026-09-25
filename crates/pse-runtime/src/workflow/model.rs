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
use std::{collections::BTreeMap, sync::Arc};
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
        self.sources.canonicalize(self.row.model_id)?;
        for row in &self.sources.providers {
            let provider = super::sources::factory(row, &self.physical.quantities)?;
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
        let owner = pse_columnar::AllocationLease::new(reservation);
        // Validate every added generated source family before publishing a revision.
        drop(self.sources.tables(&self.runtime.registry)?);
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
        super::balances::project(&mut projection, &self.sources)?;
        let mut cases = BTreeMap::new();
        let mut workspace = self.workspace;
        for case in &projection.cases {
            let inputs = project(&projection, case, &self.physical, &self.providers)?;
            if cases
                .insert(case.case_id, Arc::new(inputs.clone()))
                .is_some()
            {
                return Err(contract("duplicate case identity"));
            }
            // Same input admission used by the incremental compiler, without compiling an evaluator.
            pse_compiler::workspace::CompilerWorkspace::new(
                inputs.clone(),
                WorkspaceLimits::default(),
            )
            .map_err(|e| WorkflowError::Math(e.into()))?;
            if workspace.is_none() {
                workspace = Some(
                    self.runtime
                        .native()
                        .workspace(inputs, WorkspaceLimits::default())?,
                );
            }
        }
        let mut h = FramedHasher::new("pse.native.model-revision.v1");
        self.row.frame(&mut h);
        self.sources.frame(&mut h);
        h.hash(&self.physical.key)
            .hash(&self.runtime.registry.fingerprint());
        for (name, p) in &self.providers {
            h.str(name)
                .hash(&p.registration.spec().identity())
                .u64(p.output as u64);
        }
        Ok(ModelRevision(Arc::new(Revision {
            runtime: self.runtime,
            row: self.row,
            sources: self.sources,
            physical: self.physical,
            providers: self.providers,
            cases,
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
    pub sources: super::sources::Sources,
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
    /// Checked retained source row for Arrow consumers and durable publication.
    pub fn declaration_batch(&self) -> FieldCheckedBatch {
        self.0.batch.clone()
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
                load!(dynamic_cases, dynamics, |r| r.model_id == model);
                load!(fit_cases, fits, |r| r.model_id == model);
                load!(native_providers, providers, |r| r.model_id == model);
                load!(observations, observations, |_| true);
                load!(datasets, datasets, |_| true);
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
