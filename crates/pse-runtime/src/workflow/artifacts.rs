// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Scientific closure admission and independent, bounded retained content.

use super::{Runtime, WorkflowError, contract};
use pse_columnar::{CancellationToken, MemoryConsumer, MemoryReservation};
use pse_model::HeapUsage;
use pse_model::generated::products::ScientificRelation;
use pse_operations::canonical::MetadataResponse;
use pse_operations::canonical_artifacts::{Artifact, ArtifactLocator};
#[cfg(all(test, feature = "canonical-tests", feature = "solver-diffsol"))]
use pse_relations::columnar::RelationRow;
use std::sync::Arc;

#[cfg(all(test, feature = "canonical-tests"))]
tokio::task_local! { pub(super) static SCIENTIFIC_HANDOFF_REPLY_LOSS: bool; }

/// A bounded original recovery document is a settlement capability, never a reader.
#[derive(Clone, Debug)]
pub struct ArtifactSettlement {
    runtime: Runtime,
    intent: Artifact,
}
impl ArtifactSettlement {
    /// Exact original occurrence; active discovery remains a separate operation.
    pub fn key(&self) -> &str {
        &self.intent.key
    }
    /// Release one explicit owner only after original-intent equality is proved.
    pub async fn release(&self, kind: &str, owner: &str) -> Result<(), WorkflowError> {
        Ok(self
            .runtime
            .canonical_store()?
            .release_artifact_intent(&self.intent, kind, owner)
            .await?)
    }
    /// Settle one page; an unexpired original creation window remains pending.
    pub async fn collect(&self) -> Result<bool, WorkflowError> {
        Ok(self
            .runtime
            .canonical_store()?
            .collect_artifact(&self.intent)
            .await?)
    }
}
const ARTIFACT_INTENT_BYTES: usize = 1024 * 1024;
fn decode_intent(bytes: &[u8]) -> Result<Artifact, WorkflowError> {
    if bytes.len() > ARTIFACT_INTENT_BYTES {
        return Err(contract("artifact original intent document extent"));
    }
    let intent: Artifact = serde_json::from_slice(bytes)
        .map_err(|error| contract(format!("artifact original intent: {error}")))?;
    if intent.provenance.len() > 128 * 1024
        || intent.creation_nonce.len() != 36
        || intent.creation_request_digest.len() != 64
        || intent.primary_authority.is_empty()
        || intent.primary_authority.len() > 128
        || intent
            .creation_request_digest
            .bytes()
            .any(|value| !value.is_ascii_hexdigit())
        || intent.key
            != format!(
                "pse.artifact.v1:{}:{}:{}",
                intent.primary_authority, intent.creation_nonce, intent.creation_expires_at
            )
    {
        return Err(contract("artifact original intent identity differs"));
    }
    Ok(intent)
}

/// Activation is the only permitted mutation of this exact original request.
/// Both publication ACK and the actual subsequent read must establish it.
pub(super) fn check_artifact_ack(
    pool: &Arc<dyn pse_columnar::MemoryPool>,
    expected: &Artifact,
    actual: &Artifact,
) -> Result<(), WorkflowError> {
    // Compare the generated whole-row meaning. Only the acknowledged lifecycle
    // transition differs from the original immutable publication request.
    let scratch = MemoryConsumer::new("canonical:artifact-ack-equality").register(pool);
    scratch
        .try_grow(
            size_of::<Artifact>()
                .checked_add(expected.heap_bytes())
                .ok_or_else(|| contract("artifact acknowledgement equality extent"))?,
        )
        .map_err(pse_engine::EngineError::from)?;
    let mut normalized = expected.clone();
    normalized.active = true;
    normalized.retiring = false;
    if &normalized != actual {
        return Err(contract(
            "artifact acknowledgement differs from exact original request",
        ));
    }
    Ok(())
}

pub(super) struct ProtectedScientificInput {
    pub(super) input: Arc<pse_operations::canonical_results::ResultRead>,
    pub(super) input_owner: Arc<MemoryReservation>,
    pub(super) sources: Vec<pse_operations::canonical::Revision>,
    pub(super) revision: pse_operations::canonical::Revision,
    _sources_owner: MemoryReservation,
}
pub(super) struct PreparedScientificMember {
    name: &'static str,
    payload: Vec<u8>,
    rows: u64,
    original_digest: String,
    pub(super) facts: Option<super::analyses::MemberFacts>,
    _owner: MemoryReservation,
}
pub(super) struct ScientificArtifactSelection {
    pub(super) protected: ProtectedScientificInput,
    pub(super) kind: ScientificArtifactKind,
    pub(super) prepared: Vec<PreparedScientificMember>,
}
fn scientific_artifact_family(
    prepared: &[PreparedScientificMember],
) -> Result<pse_model::generated::enums::ScientificArtifactFamily, WorkflowError> {
    let mut contexts = prepared
        .iter()
        .filter_map(|member| member.facts.as_ref())
        .flat_map(|facts| facts.evidence.contexts.iter().map(|context| &context.row));
    use pse_model::generated::enums::ScientificArtifactFamily;
    let first = contexts
        .next()
        .ok_or_else(|| contract("scientific artifact has no admitted family context"))?;
    let family = if first.time.is_some() {
        ScientificArtifactFamily::Simulation
    } else if first.fit_id.is_some() {
        ScientificArtifactFamily::Fit
    } else {
        ScientificArtifactFamily::Modeling
    };
    if std::iter::once(first)
        .chain(contexts)
        .any(|context| match family {
            ScientificArtifactFamily::Simulation => context.time.is_none(),
            ScientificArtifactFamily::Fit => context.time.is_some() || context.fit_id.is_none(),
            ScientificArtifactFamily::Modeling => {
                context.time.is_some() || context.fit_id.is_some() || context.model_id.is_none()
            }
        })
    {
        return Err(contract("scientific artifact family contexts conflict"));
    }
    Ok(family)
}

pub(super) struct PromotedScientificArtifact {
    pub(super) artifact: ScientificArtifact,
    pub(super) protected: ProtectedScientificInput,
    pub(super) facts: super::analyses::ArtifactEvidence,
}

pub(super) fn prepare_scientific_member(
    runtime: &Runtime,
    relation: ScientificRelation,
    batch: &pse_relations::columnar::FieldCheckedBatch,
    original_digest: &str,
    analysis: bool,
) -> Result<PreparedScientificMember, WorkflowError> {
    if batch.relation_id() != relation.id {
        return Err(contract(
            "scientific member name and admitted relation identity differ",
        ));
    }
    let name = relation.name;
    let owner = MemoryConsumer::new("canonical:prepared-scientific-member")
        .register(&runtime.shared.pool());
    owner
        .try_grow(3 * super::result_files::MAX_FILE_BYTES)
        .map_err(pse_engine::EngineError::from)?;
    if let Some(context) = pse_diagnostics::observation::current() {
        context.count("canonical.artifact.scientific_encodes", "count", 1);
    }
    let payload = super::result_files::encode_result_file(batch.batch())?;
    let facts = super::analyses::MemberFacts::mine(runtime, name, batch, analysis)?;
    let retained = payload.capacity()
        + name.len()
        + original_digest.len()
        + size_of::<PreparedScientificMember>();
    if retained > owner.size() {
        return Err(contract("scientific member allocation exceeds admission"));
    }
    owner.shrink(owner.size() - retained);
    Ok(PreparedScientificMember {
        name,
        payload,
        rows: batch.batch().num_rows() as u64,
        original_digest: original_digest.into(),
        facts,
        _owner: owner,
    })
}

fn retain_prepared_member(
    mut member: PreparedScientificMember,
    copied: &mut Vec<(&'static str, Vec<u8>, u64, String)>,
    members: &MemoryReservation,
    facts: &mut super::analyses::ArtifactEvidence,
    analysis: bool,
) -> Result<(), WorkflowError> {
    if copied.len() >= 64 {
        return Err(contract("scientific artifact member bound"));
    }
    members
        .try_grow(
            member.payload.capacity()
                + member.original_digest.capacity()
                + 2 * size_of::<(&str, Vec<u8>, u64, String)>(),
        )
        .map_err(pse_engine::EngineError::from)?;
    copied
        .try_reserve_exact(1)
        .map_err(|_| contract("scientific artifact member allocation refused"))?;
    if let Some(evidence) = member.facts.take() {
        facts.append(
            evidence,
            copied.len() as u64,
            analysis,
            member.name,
            member.rows,
        )?;
    }
    copied.push((
        member.name,
        member.payload,
        member.rows,
        member.original_digest,
    ));
    Ok(())
}

/// A promised scientific product, distinct from arbitrary result row selection.
pub use pse_model::generated::enums::ScientificArtifactKind;

/// Response dispatch consumes admitted scientific ownership, never the presence
/// of a sensitivity relation or a successful primal candidate.
pub(super) fn response_artifact_kind<'a>(
    contexts: impl Iterator<Item = &'a pse_model::generated::runtime::scientific_contexts::Row>,
    computations: impl Iterator<Item = &'a pse_model::generated::runtime::computation_runs::Row>,
) -> Result<ScientificArtifactKind, WorkflowError> {
    use pse_model::generated::enums::ComputationKind;
    let contexts = contexts.collect::<Vec<_>>();
    let computations = computations.collect::<Vec<_>>();
    if contexts.is_empty() {
        return Err(contract(
            "response analysis lacks admitted scientific context",
        ));
    }
    if contexts.iter().all(|context| context.time.is_some())
        && !computations.is_empty()
        && computations
            .iter()
            .all(|header| header.kind == ComputationKind::Simulation)
        && contexts.iter().all(|context| {
            computations
                .iter()
                .any(|header| header.run_id == context.run_id)
        })
    {
        return Ok(ScientificArtifactKind::Trajectory);
    }
    if contexts.iter().all(|context| {
        context.time.is_none() && (context.model_id.is_some() || context.fit_id.is_some())
    }) && computations
        .iter()
        .all(|header| header.kind != ComputationKind::Simulation)
    {
        return Ok(ScientificArtifactKind::LocalAnalysis);
    }
    Err(contract(
        "response analysis scientific family is missing or conflicting",
    ))
}

/// A restartable independent scientific owner. Each read acquires current permission.
#[derive(Clone, Debug)]
pub struct ScientificArtifact {
    runtime: Runtime,
    header: Arc<Artifact>,
    _metadata_owner: Arc<MemoryReservation>,
}
fn retained_artifact_header(
    runtime: &Runtime,
    header: &Artifact,
) -> Result<Arc<MemoryReservation>, WorkflowError> {
    let bytes = size_of::<Artifact>()
        .checked_add(header.heap_bytes())
        .and_then(|bytes| bytes.checked_add(2 * size_of::<usize>()))
        .ok_or_else(|| contract("artifact retained header extent"))?;
    let owner =
        MemoryConsumer::new("canonical:artifact-retained-header").register(&runtime.shared.pool());
    owner
        .try_grow(bytes)
        .map_err(pse_engine::EngineError::from)?;
    Ok(Arc::new(owner))
}
impl ScientificArtifact {
    /// Immutable creation/provenance and original settlement intent.
    pub fn header(&self) -> &Artifact {
        &self.header
    }
    /// Observe current active metadata through its generated relation contract.
    pub async fn metadata(
        &self,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, WorkflowError> {
        use pse_relations::generated::runtime::canonical_artifacts as rows;
        let owner = super::connected_results::response_owner(
            &self.runtime,
            "canonical:artifact-metadata",
            MetadataResponse::Header,
        )?;
        let row = pse_operations::canonical::response_ownership_scope(
            owner.clone(),
            self.runtime.canonical_store()?.artifact(&self.header.key),
        )
        .await?;
        let validation = self.runtime.validation_context();
        let mut builder = rows::Builder::with_registry(self.runtime.registry(), 1, &validation)
            .map_err(super::relation)?;
        builder.push(row).map_err(super::relation)?;
        builder
            .finish()
            .map_err(super::relation)?
            .retained(&self.runtime.shared.pool(), &CancellationToken::new())
            .map_err(super::relation)
    }
    /// Acquire the original qualified seed under current independent ownership.
    pub async fn seed(&self) -> Result<super::AdmittedSeed, WorkflowError> {
        self.runtime
            .operations()?
            .artifact_seed(&self.header.key)
            .await
    }
    /// Read one independently decoded IPC member; returned buffers own their memory.
    pub async fn block(
        &self,
        ordinal: u64,
        cancel: CancellationToken,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, WorkflowError> {
        let owner = Arc::new(
            MemoryConsumer::new("canonical:scientific-artifact-read")
                .register(&self.runtime.shared.pool()),
        );
        owner
            .try_grow(4 * super::result_files::MAX_FILE_BYTES)
            .map_err(pse_engine::EngineError::from)?;
        let response = super::connected_results::response_owner(
            &self.runtime,
            "canonical:artifact-block-selection-response",
            MetadataResponse::Selection,
        )?;
        let read = pse_operations::canonical::response_ownership_scope(
            response.clone(),
            self.runtime
                .canonical_store()?
                .read_artifact(&self.header.key),
        )
        .await?;
        let read_owner = Arc::new(super::connected_results::retain_protected_metadata(
            &self.runtime.shared.pool(),
            response,
            read.retained_bytes()?,
            |owner| read.retain_metadata_owner(owner),
        )?);
        read.retain_metadata_owner(read_owner.clone())?;
        let response = super::connected_results::response_owner(
            &self.runtime,
            "canonical:artifact-block-response",
            MetadataResponse::Header,
        )?;
        let block = pse_operations::canonical::response_ownership_scope(
            Arc::new((response, owner.clone(), read_owner)),
            read.block(ordinal),
        )
        .await?;
        let spec = self
            .runtime
            .registry()
            .relation(&block.relation)
            .ok_or_else(|| contract("artifact member relation unavailable"))?;
        let schema = pse_schema::arrow::relation_schema_ref(self.runtime.registry(), spec)
            .map_err(|error| contract(error.to_string()))?;
        let batch = super::result_files::decode_result_file(
            block.payload.as_slice(),
            schema,
            usize::try_from(block.rows).map_err(|_| contract("artifact row extent"))?,
        )?;
        let batch = pse_relations::columnar::FieldCheckedBatch::admit(
            self.runtime.registry(),
            spec,
            batch,
            &self.runtime.validation_context(),
            &cancel,
        )
        .map_err(pse_engine::EngineError::from)?;
        let retained = batch
            .retained(&self.runtime.shared.pool(), &cancel)
            .map_err(super::relation)?;
        read.release().await?;
        Ok(retained)
    }
    /// Acquire current seed authority and validate the target's scientific contract.
    pub async fn apply_start(
        &self,
        target: &super::ModelingSolvePreparation,
    ) -> Result<super::ModelingSolvePreparation, WorkflowError> {
        target.clone().with_admitted_start(&self.seed().await?)
    }
    /// Add exactly one independent consumer obligation.
    pub async fn retain(&self, kind: &str, owner: &str) -> Result<(), WorkflowError> {
        Ok(self
            .runtime
            .canonical_store()?
            .retain_artifact(&self.header.key, kind, owner)
            .await?)
    }
    /// Release only this attributed obligation; other owners retain future reads.
    pub async fn release(&self, kind: &str, owner: &str) -> Result<(), WorkflowError> {
        Ok(self
            .runtime
            .canonical_store()?
            .release_artifact(&self.header.key, kind, owner)
            .await?)
    }
    /// Settle one bounded cleanup page under the original immutable intent.
    pub async fn collect(&self) -> Result<bool, WorkflowError> {
        Ok(self
            .runtime
            .canonical_store()?
            .collect_artifact(&self.header)
            .await?)
    }
    /// Verify the typed graph coordinate selects an actual generated field and row.
    pub async fn evidence(
        &self,
        locator: &ArtifactLocator,
        cancel: CancellationToken,
    ) -> Result<pse_relations::columnar::FieldCheckedBatch, WorkflowError> {
        if locator.artifact != self.header.key {
            return Err(contract("artifact evidence owner differs"));
        }
        let batch = self.block(locator.block, cancel.clone()).await?;
        if locator
            .row
            .is_some_and(|row| row >= batch.batch().num_rows() as u64)
            || locator
                .field
                .as_ref()
                .is_some_and(|field| batch.batch().schema().index_of(field).is_err())
        {
            return Err(contract("artifact evidence coordinate absent"));
        }
        Ok(batch)
    }
}

pub(super) fn coordinate(
    id: pse_ids::SemanticId,
    quantity: pse_quantity::QuantityTypeId,
    quantities: &pse_quantity::QuantityRegistry,
    scale: f64,
    offset: f64,
) -> Result<pse_model::generated::structures::ScientificArtifactCoordinate, WorkflowError> {
    let unit = quantities
        .quantity_type(quantity)
        .map_err(super::math)?
        .canonical_unit;
    Ok(
        pse_model::generated::structures::ScientificArtifactCoordinate {
            id,
            quantity_id: quantity.as_id(),
            unit_id: unit.as_id(),
            scale,
            offset,
        },
    )
}

pub(super) fn solve_context(
    prepared: &super::ModelingSolvePreparation,
    run_id: pse_model::generated::identities::RunId,
    step: i64,
) -> Result<pse_model::generated::runtime::scientific_contexts::Row, WorkflowError> {
    let coordinates = solve_coordinates(prepared, run_id, step)?;
    Ok(pse_model::generated::runtime::scientific_contexts::Row {
        run_id: coordinates.run_id,
        step: coordinates.step,
        model_id: coordinates.model_id,
        fit_id: coordinates.fit_id,
        case_id: coordinates.case_id,
        instance_id: coordinates.instance_id,
        source_identity: coordinates.source_identity,
        physical_identity: coordinates.physical_identity,
        preparation_identity: prepared
            .solve
            .preparation_identity()
            .map_err(crate::math::MathRuntimeError::from)?,
        profile_identity: crate::math::solves::profile_key(&prepared.profile)
            .map_err(crate::math::MathRuntimeError::from)?
            .as_id(),
        configuration: coordinates.configuration,
        binding: coordinates.binding,
        state: coordinates.state,
        inputs: coordinates.inputs,
        outputs: coordinates.outputs,
        integrals: coordinates.integrals,
        time: coordinates.time,
        mode_names: coordinates.mode_names,
        mode_identities: coordinates.mode_identities,
    })
}

/// Shared factual gathering does not ask executable providers for configuration identities.
pub(super) fn solve_coordinates(
    prepared: &super::ModelingSolvePreparation,
    run_id: pse_model::generated::identities::RunId,
    step: i64,
) -> Result<super::review_context::ReviewScientificCoordinates, WorkflowError> {
    let solved = prepared.model.model.solved();
    let structure = prepared.model.case.compiled().plan.structure();
    let quantities = &prepared.source.quantities;
    let state = structure
        .variables()
        .iter()
        .map(|v| coordinate(v.port.id, v.port.quantity, quantities, 1., 0.))
        .collect::<Result<Vec<_>, _>>()?;
    let inputs = structure
        .parameters()
        .iter()
        .map(|v| coordinate(v.id, v.quantity, quantities, 1., 0.))
        .collect::<Result<Vec<_>, _>>()?;
    let configuration = serde_json::to_vec(&(
        1u8,
        &prepared.profile.sensitivity,
        &prepared.starts,
        &prepared.model.values.scalars,
    ))
    .map_err(|error| contract(error.to_string()))?;
    let binding = prepared
        .bound_source
        .as_ref()
        .map(|value| {
            serde_json::to_vec(&***value)
                .map(Into::into)
                .map_err(|error| contract(error.to_string()))
        })
        .transpose()?;
    Ok(super::review_context::ReviewScientificCoordinates {
        run_id,
        step,
        model_id: Some(solved.model()),
        fit_id: None,
        case_id: solved.case(),
        instance_id: Some(solved.instance()),
        source_identity: prepared.source.revision.identity().as_id(),
        physical_identity: prepared.source.physical.identity(),
        preparation_identity: None,
        profile_identity: None,
        configuration: configuration.into(),
        binding,
        state,
        inputs,
        outputs: Vec::new(),
        integrals: Vec::new(),
        time: None,
        mode_names: Vec::new(),
        mode_identities: Vec::new(),
    })
}

impl Runtime {
    /// Recover settlement from exact owner-emitted bytes, without issuing authority
    /// or opening inactive/absent content. Server effects recheck the original intent.
    pub fn recover_artifact_intent(
        &self,
        bytes: &[u8],
    ) -> Result<ArtifactSettlement, WorkflowError> {
        Ok(ArtifactSettlement {
            runtime: self.clone(),
            intent: decode_intent(bytes)?,
        })
    }
    /// Preserve a qualified start as a copied capsule with an independent owner.
    pub async fn promote_seed_artifact(
        &self,
        solution: pse_model::generated::identities::SolutionId,
        owner_kind: &str,
        owner: &str,
    ) -> Result<ScientificArtifact, WorkflowError> {
        let response = super::connected_results::response_owner(
            self,
            "canonical:seed-artifact-promotion",
            MetadataResponse::Header,
        )?;
        let header = pse_operations::canonical::response_ownership_scope(
            response.clone(),
            self.operations()?.promote_seed(solution, owner_kind, owner),
        )
        .await?;
        let metadata_owner = match retained_artifact_header(self, &header) {
            Ok(owner) => owner,
            Err(source) => {
                return Err(WorkflowError::ArtifactTransferUnsettled {
                    analysis: None,
                    artifact: Box::new(header),
                    owner_kind: owner_kind.into(),
                    owner: owner.into(),
                    read: None,
                    source: Box::new(source),
                });
            }
        };
        Ok(ScientificArtifact {
            runtime: self.clone(),
            header: Arc::new(header),
            _metadata_owner: metadata_owner,
        })
    }
    /// Reopen an independent artifact without consulting producer result availability.
    pub async fn scientific_artifact(
        &self,
        key: &str,
    ) -> Result<ScientificArtifact, WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                let response = super::connected_results::response_owner(
                    self,
                    "canonical:scientific-artifact-header-response",
                    MetadataResponse::Header,
                )?;
                let header = pse_operations::canonical::response_ownership_scope(
                    response.clone(),
                    self.canonical_store()?.artifact(key),
                )
                .await?;
                if ![
                    "seed",
                    "final-state",
                    "trajectory",
                    "local-analysis",
                    "graph-evidence",
                ]
                .contains(&header.kind.as_str())
                {
                    return Err(contract("artifact kind has no scientific IPC reader"));
                }
                let metadata_owner = retained_artifact_header(self, &header)?;
                Ok(ScientificArtifact {
                    runtime: self.clone(),
                    header: Arc::new(header),
                    _metadata_owner: metadata_owner,
                })
            })
            .await
    }
    pub(super) async fn select_scientific_input(
        &self,
        run: &str,
        attempt: &str,
    ) -> Result<ProtectedScientificInput, WorkflowError> {
        let response = super::connected_results::response_owner(
            self,
            "canonical:scientific-selection-response",
            MetadataResponse::Selection,
        )?;
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count(
                "canonical.artifact.scientific_selection_requests",
                "count",
                1,
            );
        }
        let input = Arc::new(
            pse_operations::canonical::response_ownership_scope(
                response.clone(),
                self.canonical_store()?.read_results(run, attempt),
            )
            .await?,
        );
        let input_owner = Arc::new(super::connected_results::retain_protected_metadata(
            &self.shared.pool(),
            response,
            input.retained_bytes()?,
            |owner| input.retain_metadata_owner(owner),
        )?);
        input.retain_metadata_owner(input_owner.clone())?;
        let response = super::connected_results::response_owner(
            self,
            "canonical:scientific-sources-response",
            MetadataResponse::Selection,
        )?;
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("canonical.artifact.scientific_source_requests", "count", 1);
        }
        let sources = pse_operations::canonical::response_ownership_scope(
            Arc::new((input.clone(), input_owner.clone(), response.clone())),
            self.canonical_store()?.analysis_run_sources(run),
        )
        .await?;
        let revision = sources
            .iter()
            .find(|source| source.key == input.run().revision)
            .ok_or_else(|| contract("scientific primary result source absent"))?;
        let extent = sources
            .iter()
            .try_fold(
                sources.capacity() * size_of::<pse_operations::canonical::Revision>(),
                |n, source| n.checked_add(source.heap_bytes()),
            )
            .and_then(|n| n.checked_add(revision.heap_bytes()))
            .ok_or_else(|| contract("scientific source allocation extent"))?;
        let owner = MemoryConsumer::new("canonical:scientific-protected-sources")
            .register(&self.shared.pool());
        owner
            .try_grow(extent)
            .map_err(pse_engine::EngineError::from)?;
        let revision = revision.clone();
        drop(response);
        Ok(ProtectedScientificInput {
            input,
            input_owner,
            sources,
            revision,
            _sources_owner: owner,
        })
    }

    /// Copy a scientifically sufficient immutable product without rerunning science.
    /// The trajectory product retains its entire recorded prefix; no row-level
    /// reclamation is promised for a physical IPC block.
    pub async fn promote_scientific_artifact(
        &self,
        run: &str,
        attempt: &str,
        kind: ScientificArtifactKind,
        owner_kind: &str,
        owner: &str,
        cancel: CancellationToken,
    ) -> Result<ScientificArtifact, WorkflowError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                let memory = MemoryConsumer::new("canonical:scientific-artifact-promotion")
                    .register(&self.shared.pool());
                memory
                    .try_grow(128 * 1024 * 1024)
                    .map_err(pse_engine::EngineError::from)?;
                cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
                let mut selection = self
                    .select_result_analysis(run, attempt, &memory, &cancel)
                    .await?;
                selection.kind = kind;
                let product = self
                    .promote_selected_scientific_artifact(
                        selection, owner_kind, owner, &cancel, &memory, false,
                    )
                    .await?;
                Ok(product.artifact)
            })
            .await
    }

    /// Keep the existing activity and prepaid grant around the complete transfer.
    /// This borrowed boxed body is owned by that scope, including its drop path;
    /// its non-inlined construction returns before any nested transport is polled.
    #[allow(
        clippy::too_many_arguments,
        reason = "artifact selection and durable owner association remain explicit alongside the borrowed cancellation token and prepaid grant"
    )]
    #[inline(never)]
    pub(super) fn promote_selected_scientific_artifact<'a>(
        &'a self,
        selection: ScientificArtifactSelection,
        owner_kind: &'a str,
        owner: &'a str,
        cancel: &'a CancellationToken,
        _memory: &'a MemoryReservation,
        analysis: bool,
    ) -> futures_util::future::BoxFuture<'a, Result<PromotedScientificArtifact, WorkflowError>>
    {
        Box::pin(async move {
            cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
            let ScientificArtifactSelection {
                protected,
                kind,
                mut prepared,
            } = selection;
            let input = &protected.input;
            let input_owner = &protected.input_owner;
            let sources = &protected.sources;
            let revision = &protected.revision;
            let run = input.run().key.as_str();
            let attempt = input.attempt().key.as_str();
            let family = scientific_artifact_family(&prepared)?;
            let required = kind.required_relations(family).ok_or_else(|| {
                contract("scientific artifact kind is unavailable for this family")
            })?;
            let members = MemoryConsumer::new("canonical:scientific-artifact-member-files")
                .register(&self.shared.pool());
            let mut copied = Vec::new();
            let mut facts = super::analyses::ArtifactEvidence::new(self)?;
            let mut selected = Vec::new();
            for member in required {
                let name = member.name;
                let spec = self
                    .registry()
                    .relation_by_id(member.id)
                    .ok_or_else(|| contract("scientific artifact relation unavailable"))?;
                let Some(set) = input
                    .sets()
                    .iter()
                    .find(|set| set.name == spec.id.to_string())
                else {
                    return Err(contract(format!(
                        "scientific artifact is missing required relation {name}"
                    )));
                };
                if copied.len().saturating_add(set.batch_count.max(1) as usize) > 64 {
                    return Err(contract(
                        "scientific artifact exceeds bounded selection; choose a smaller scientific product",
                    ));
                }
                selected.push((name, set.row_count, set.batches_digest.clone()));
                if set.batch_count == 0 {
                    let schema = pse_schema::arrow::relation_schema_ref(self.registry(), spec)
                        .map_err(|error| contract(error.to_string()))?;
                    let payload = super::result_files::encode_result_file(
                        &datafusion::arrow::record_batch::RecordBatch::new_empty(schema),
                    )?;
                    members
                        .try_grow(payload.capacity())
                        .map_err(pse_engine::EngineError::from)?;
                    copied.push((name, payload, 0, set.batches_digest.clone()));
                }
                if set.batch_count != 0 {
                    if prepared.iter().any(|member| member.name == name) {
                        while let Some(position) =
                            prepared.iter().position(|member| member.name == name)
                        {
                            let member = prepared.remove(position);
                            retain_prepared_member(
                                member,
                                &mut copied,
                                &members,
                                &mut facts,
                                analysis,
                            )?;
                        }
                        continue;
                    }
                    let schema = pse_schema::arrow::relation_schema_ref(self.registry(), spec)
                        .map_err(|error| contract(error.to_string()))?;
                    let snapshot = super::ResultSnapshot::capture(
                        &self.source_services(),
                        input.clone(),
                        input_owner.clone(),
                        spec.id,
                        &set.key,
                        schema,
                        0,
                        u64::MAX,
                    )
                    .await?;
                    for block in snapshot.blocks() {
                        cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
                        let checked = snapshot.read_block(block, cancel).await?;
                        let member = prepare_scientific_member(
                            self,
                            *member,
                            &checked,
                            &block.payload_digest,
                            analysis,
                        )?;
                        retain_prepared_member(
                            member,
                            &mut copied,
                            &members,
                            &mut facts,
                            analysis,
                        )?;
                    }
                }
            }
            if !prepared.is_empty() {
                return Err(contract(
                    "scientific dispatch was not consumed by exact family",
                ));
            }
            let contexts = facts
                .evidence
                .contexts
                .iter()
                .map(|item| &item.row)
                .collect::<Vec<_>>();
            let endpoints = facts
                .evidence
                .endpoints
                .iter()
                .map(|item| &item.row)
                .collect::<Vec<_>>();
            if contexts.is_empty()
                || !selected
                    .iter()
                    .any(|(name, rows, _)| *name == "runtime.candidate_assessments" && *rows > 0)
                || !selected
                    .iter()
                    .any(|(name, rows, _)| *name == "runtime.run_lineage" && *rows > 0)
            {
                return Err(contract(
                    "scientific artifact lacks original interpretation/assessment/lineage",
                ));
            }
            for context in &contexts {
                if context.mode_names.len() != context.mode_identities.len()
                    || context.configuration.is_empty()
                    || context
                        .state
                        .iter()
                        .chain(&context.inputs)
                        .chain(&context.outputs)
                        .chain(&context.integrals)
                        .any(|c| !c.scale.is_finite() || c.scale <= 0. || !c.offset.is_finite())
                {
                    return Err(contract(
                        "scientific artifact coordinate interpretation incomplete",
                    ));
                }
            }
            match kind {
                ScientificArtifactKind::FinalState | ScientificArtifactKind::Trajectory
                    if !endpoints.is_empty() =>
                {
                    for endpoint in &endpoints {
                        let context = contexts
                            .iter()
                            .find(|context| {
                                context.run_id == endpoint.run_id && context.time.is_some()
                            })
                            .ok_or_else(|| contract("endpoint scientific context absent"))?;
                        let mode = usize::try_from(endpoint.mode)
                            .map_err(|_| contract("endpoint mode extent"))?;
                        if context.mode_names.get(mode) != Some(&endpoint.mode_name)
                            || context.mode_identities.get(mode) != Some(&endpoint.mode_identity)
                            || context.state.iter().map(|c| c.id).collect::<Vec<_>>()
                                != endpoint.state_ids
                            || context.inputs.iter().map(|c| c.id).collect::<Vec<_>>()
                                != endpoint.input_ids
                            || context.outputs.iter().map(|c| c.id).collect::<Vec<_>>()
                                != endpoint.output_ids
                            || context.integrals.len() != endpoint.integrals.len()
                            || endpoint.state.len() != endpoint.state_ids.len()
                            || endpoint.inputs.len() != endpoint.input_ids.len()
                            || endpoint.outputs.len() != endpoint.output_ids.len()
                        {
                            return Err(contract("endpoint scientific closure differs"));
                        }
                    }
                }
                ScientificArtifactKind::FinalState => {
                    if !selected
                        .iter()
                        .any(|(name, rows, _)| *name == "runtime.solve_variables" && *rows > 0)
                        || contexts.iter().any(|context| context.time.is_some())
                    {
                        return Err(contract(
                            "final state requires an actual endpoint or complete static candidate",
                        ));
                    }
                }
                ScientificArtifactKind::Trajectory => {
                    return Err(contract("trajectory has no actual endpoint interpretation"));
                }
                ScientificArtifactKind::LocalAnalysis => {
                    if !selected
                        .iter()
                        .any(|(name, rows, _)| *name == "runtime.local_validity" && *rows > 0)
                    {
                        return Err(contract(
                            "local analysis lacks original certification/withheld reasons",
                        ));
                    }
                }
            }
            let provenance = serde_json::to_vec(&(
                1u8,
                run,
                attempt,
                &input.run().revision,
                input.run().sequence,
                input.attempt().generation,
                &input.attempt().outcome,
                input.manifest(),
                &selected,
            ))
            .map_err(|error| contract(error.to_string()))?;
            let bytes = copied
                .iter()
                .map(|(_, payload, _, _)| payload.len() as u64)
                .sum();
            let response = super::connected_results::response_owner(
                self,
                "canonical:artifact-promotion-header-response",
                MetadataResponse::Header,
            )?;
            let mut header = pse_operations::canonical::response_ownership_scope(
                Arc::new((input.clone(), input_owner.clone(), response)),
                self.canonical_store()?.new_artifact(
                    revision,
                    kind.as_str(),
                    provenance,
                    copied.len() as u64,
                    bytes,
                ),
            )
            .await?;
            let blocks = copied
                .into_iter()
                .enumerate()
                .map(|(ordinal, (name, payload, rows, digest))| {
                    let spec = self
                        .registry()
                        .relation(name)
                        .ok_or_else(|| contract("artifact member relation unavailable"))?;
                    Ok::<_, WorkflowError>(
                        pse_operations::canonical_artifacts::artifact_block(
                            &header.key,
                            ordinal as u64,
                            name,
                            payload,
                            &digest,
                            rows,
                        )
                        .native_file(&spec.id.to_string())?,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            facts.verify_members(&blocks)?;
            pse_operations::canonical_artifacts::seal_artifact_request(
                &mut header,
                sources,
                std::slice::from_ref(input.as_ref()),
                &blocks,
                owner_kind,
                owner,
            );
            // Prepay the continuing header before publication can acknowledge
            // an independent durable owner; handle clones share this allocation.
            let metadata_owner = retained_artifact_header(self, &header)?;
            let response = super::connected_results::response_owner(
                self,
                "canonical:artifact-promotion-publication-response",
                MetadataResponse::Selection,
            )?;
            let publication = pse_operations::canonical::response_ownership_scope(
                // The SDK owns metadata and the original read follower only;
                // copied member buffers retain their separate local grant.
                Arc::new((input.clone(), input_owner.clone(), response)),
                self.canonical_store()?.persist_artifact(
                    &header,
                    sources,
                    std::slice::from_ref(input.as_ref()),
                    &blocks,
                    owner_kind,
                    owner,
                ),
            );
            let acknowledged = match cancel.until_cancelled(publication).await {
                Ok(reply) => reply?,
                Err(_) => {
                    return Err(
                        pse_operations::canonical::CanonicalError::ArtifactUnsettled {
                            intent: Box::new(header),
                            source: Box::new(
                                pse_operations::canonical::CanonicalError::IncompleteResponse,
                            ),
                        }
                        .into(),
                    );
                }
            };
            if check_artifact_ack(&self.shared.pool(), &header, &acknowledged).is_err() {
                return Err(
                    pse_operations::canonical::CanonicalError::ArtifactUnsettled {
                        intent: Box::new(header),
                        source: Box::new(
                            pse_operations::canonical::CanonicalError::IncompleteResponse,
                        ),
                    }
                    .into(),
                );
            }
            #[cfg(all(test, feature = "canonical-tests"))]
            if SCIENTIFIC_HANDOFF_REPLY_LOSS
                .try_with(|lost| *lost)
                .unwrap_or(false)
            {
                return Err(
                    pse_operations::canonical::CanonicalError::ArtifactUnsettled {
                        intent: Box::new(header),
                        source: Box::new(
                            pse_operations::canonical::CanonicalError::IncompleteResponse,
                        ),
                    }
                    .into(),
                );
            }
            let artifact = ScientificArtifact {
                runtime: self.clone(),
                header: Arc::new(header),
                _metadata_owner: metadata_owner,
            };
            Ok(PromotedScientificArtifact {
                artifact,
                protected,
                facts,
            })
        })
    }
}

#[cfg(all(test, feature = "canonical-tests", feature = "solver-diffsol"))]
mod tests {
    use super::*;
    use std::sync::Arc;
    #[tokio::test]
    async fn canonical_scientific_trajectory_and_endpoint_survive_producer_retirement() {
        let runtime = super::super::durable_tests::durable_runtime();
        let mut physical = super::super::tests::physical();
        physical.preconditions = Arc::new(
            pse_quantity::PhysicalPreconditions::new(
                pse_quantity::generated::standard_preconditions(),
            )
            .unwrap(),
        );
        physical.key = pse_compiler::workspace::physical_identity(
            &physical.quantities,
            &physical.preconditions,
        );
        let rows=pse_authoring::language::parse("package p { def Root { domain t: Time from 0{s} to 1{s}; discretize grid on t using integrated(elements=1,order=1); var x[i in t]: Time; eq rate[i in t]: d(x[i])/di == -x[i]/1{s}; eq initial: x[0{s}] == 1{s}; let area:Time=integral(i in t | x[i]/1{s}); annotation report area(\"integral\"); } }",pse_ids::SemanticId::from_bytes([71; 16]),pse_authoring::language::IdentityPolicy::Named,pse_authoring::ParseBudget::default()).unwrap();
        let root = rows
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        let package = runtime.modeling_package(rows, physical).await.unwrap();
        let cancel = crate::CancelSource::new();
        let profile = pse_backend_native::dynamics::Profile {
            method: pse_backend_native::dynamics::Method::Diffsol,
            samples: vec![0., 0.5, 1.],
            rtol: 1e-7,
            atol: vec![1e-9],
            out_rtol: Some(1e-7),
            out_atol: vec![1e-9],
            ..Default::default()
        };
        let prepared = package
            .prepare_simulation(
                &runtime.preparation_services(),
                root,
                pse_modeling::specialize::root_instance(root),
                Default::default(),
                Default::default(),
                Default::default(),
                super::super::tests::compiler_profile(),
                profile,
                pse_kernels::DerivativeOrder::First,
                &cancel,
            )
            .await
            .unwrap();
        let result = prepared
            .start(&runtime.execution_services())
            .unwrap()
            .wait()
            .await
            .unwrap();
        let run = result.canonical_run_key().unwrap();
        let attempt = result.canonical_attempt_key().unwrap();
        let original = pse_relations::generated::runtime::trajectory_endpoints::Row::rows(
            &result.table("runtime.trajectory_endpoints").unwrap(),
        )
        .unwrap();
        assert_eq!(original.len(), 1);
        let trajectory = runtime
            .promote_scientific_artifact(
                run,
                attempt,
                ScientificArtifactKind::Trajectory,
                "user",
                "trajectory-consumer",
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let endpoint = runtime
            .promote_scientific_artifact(
                run,
                attempt,
                ScientificArtifactKind::FinalState,
                "audit",
                "endpoint-consumer",
                CancellationToken::new(),
            )
            .await
            .unwrap();
        assert!(
            runtime
                .promote_scientific_artifact(
                    run,
                    attempt,
                    ScientificArtifactKind::LocalAnalysis,
                    "user",
                    "insufficient-analysis",
                    CancellationToken::new()
                )
                .await
                .is_err(),
            "a trajectory without original local validity cannot promise a local analysis"
        );
        // The same producer can supply a truthful trajectory graph while the
        // stronger scalar artifact promise above remains unavailable.
        let analysis = runtime
            .result_analysis(
                run,
                attempt,
                &super::super::analyses::AnalysisControls {
                    roots: vec![],
                    direction: super::super::analyses::AnalysisDirection::Downstream,
                },
                CancellationToken::new(),
            )
            .await
            .unwrap();
        let graph_rows = pse_relations::generated::runtime::canonical_analysis_edges::Row::rows(
            &analysis.edges(None).await.unwrap(),
        )
        .unwrap();
        assert!(
            graph_rows
                .iter()
                .any(|edge| edge.kind == "scientific_context")
        );
        assert!(
            !graph_rows
                .iter()
                .any(|edge| edge.kind.contains("parametric"))
        );
        let graph_artifact = runtime
            .scientific_artifact(
                graph_rows
                    .iter()
                    .find_map(|edge| edge.evidence_artifact.as_deref())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(graph_artifact.header().kind, "trajectory");
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
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        assert!(retired);
        for artifact in [trajectory, endpoint] {
            let reopened = runtime
                .scientific_artifact(&artifact.header().key)
                .await
                .unwrap();
            let read = runtime
                .canonical_store()
                .unwrap()
                .read_artifact(&artifact.header().key)
                .await
                .unwrap();
            let mut seen = false;
            for ordinal in 0..artifact.header().block_count {
                let block = read.block(ordinal).await.unwrap();
                if block.relation == "runtime.trajectory_endpoints" {
                    let retained = reopened
                        .block(ordinal, CancellationToken::new())
                        .await
                        .unwrap();
                    assert_eq!(
                        pse_relations::generated::runtime::trajectory_endpoints::Row::rows(
                            &retained
                        )
                        .unwrap(),
                        original
                    );
                    seen = true;
                }
            }
            read.release().await.unwrap();
            assert!(seen);
        }
    }

    #[tokio::test]
    async fn canonical_partial_event_analysis_retains_numeric_prefix_without_scalar_claims() {
        use super::super::analyses::{AnalysisControls, AnalysisDirection};
        use pse_relations::generated::runtime::{
            canonical_analysis_edges, computation_runs, scientific_contexts, simulation_events,
            simulation_samples, trajectory_endpoints,
        };
        let runtime = super::super::durable_tests::durable_runtime();
        let mut physical = super::super::tests::physical();
        physical.preconditions = Arc::new(
            pse_quantity::PhysicalPreconditions::new(
                pse_quantity::generated::standard_preconditions(),
            )
            .unwrap(),
        );
        physical.key = pse_compiler::workspace::physical_identity(
            &physical.quantities,
            &physical.preconditions,
        );
        // x(t)=1+2t reaches the terminal guard x=2 at t=0.5,
        // before the fixed-horizon observations at 0.75 and 2 seconds.
        let source = "package p { def Root { domain t:Time from 0{s} to 2{s}; discretize grid on t using integrated(elements=1,order=1); param p:Scalar=2; var x[i in t]:Time; eq rate[i in t]:d(x[i])/di==p; eq initial:x[0{s}]==1{s}; let hit[i in t]:Time=x[i]-2{s}; } test stopped fixture { dof 0; route integrated; procedure integrate; endpoint fixed_horizon; integrate samples(0{s},0.25{s},0.75{s},2{s}) relative(1e-8) normalized_absolute(1e-8) step(1e-4{s}); mode stop; event root.hit[0{s}] direction(either) tolerance(1e-8{s}) terminal; } { child root:Root=Root(); } }";
        let rows = pse_authoring::language::parse(
            source,
            pse_ids::SemanticId::from_bytes([71; 16]),
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = rows
            .iter()
            .find(|row| row.name == "stopped")
            .unwrap()
            .declaration_id;
        let package = runtime.modeling_package(rows, physical).await.unwrap();
        let profile = pse_backend_native::dynamics::Profile {
            method: pse_backend_native::dynamics::Method::Diffsol,
            end: 2.,
            samples: vec![0., 0.25, 0.75, 2.],
            parameter_scales: vec![1.],
            sensitivity: pse_backend_native::dynamics::DynamicSensitivity::None,
            ..Default::default()
        };
        let prepared = package
            .declared_simulation(
                &runtime.preparation_services(),
                root,
                super::super::tests::compiler_profile(),
                Some(profile),
                Default::default(),
                &crate::CancelSource::new(),
            )
            .await
            .unwrap();
        let result = prepared
            .start(&runtime.execution_services())
            .unwrap()
            .wait()
            .await
            .unwrap();
        assert!(
            !result.usable(),
            "the fixed-horizon completion remains unsatisfied after the event"
        );
        let headers =
            computation_runs::Row::rows(&result.table("runtime.computation_runs").unwrap())
                .unwrap();
        assert_eq!(headers.len(), 1);
        assert_eq!(
            headers[0].trajectory_termination,
            Some(pse_model::generated::enums::TrajectoryTermination::Event)
        );
        assert_eq!(headers[0].response_available, Some(false));
        assert!((headers[0].completed_time.unwrap() - 0.5).abs() < 1e-5);
        let contexts =
            scientific_contexts::Row::rows(&result.table("runtime.scientific_contexts").unwrap())
                .unwrap();
        let state = contexts[0].state[0].id;
        let mut samples =
            simulation_samples::Row::rows(&result.table("runtime.simulation_samples").unwrap())
                .unwrap();
        let state_samples = samples
            .iter()
            .filter(|sample| sample.symbol_id == state)
            .collect::<Vec<_>>();
        assert_eq!(state_samples.len(), 2);
        for sample in state_samples {
            assert!((sample.value - (1. + 2. * sample.time)).abs() < 1e-6);
            assert!(sample.time < 0.5);
        }
        let events =
            simulation_events::Row::rows(&result.table("runtime.simulation_events").unwrap())
                .unwrap();
        assert_eq!(events.len(), 1);
        assert!((events[0].time - 0.5).abs() < 1e-5);
        assert!((events[0].before - 2.).abs() < 1e-5);
        let endpoints =
            trajectory_endpoints::Row::rows(&result.table("runtime.trajectory_endpoints").unwrap())
                .unwrap();
        assert_eq!(endpoints.len(), 1);
        assert!(!endpoints[0].endpoint_satisfied);
        assert!((endpoints[0].time - events[0].time).abs() < 1e-8);
        assert!((endpoints[0].state[0] - 2.).abs() < 1e-5);
        let run = result.canonical_run_key().unwrap();
        let attempt = result.canonical_attempt_key().unwrap();
        let graph = runtime
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
        let edges = canonical_analysis_edges::Row::rows(&graph.edges(None).await.unwrap()).unwrap();
        let event_edge = edges
            .iter()
            .find(|edge| edge.kind == "trajectory_event")
            .unwrap();
        assert!(event_edge.evidence_row.is_some());
        assert!(
            edges.iter().all(|edge| !edge.kind.contains("parametric")
                && edge.kind != "recorded_transient_response")
        );
        let artifact = runtime
            .scientific_artifact(event_edge.evidence_artifact.as_deref().unwrap())
            .await
            .unwrap();
        assert_eq!(artifact.header().kind, "trajectory");
        let retained_event = artifact
            .evidence(
                &ArtifactLocator {
                    artifact: artifact.header().key.clone(),
                    block: event_edge.evidence_block.unwrap(),
                    row: event_edge.evidence_row,
                    field: event_edge.evidence_field.clone(),
                },
                CancellationToken::new(),
            )
            .await
            .unwrap();
        assert_eq!(
            simulation_events::Row::rows(&retained_event).unwrap(),
            events
        );
        let read = runtime
            .canonical_store()
            .unwrap()
            .read_artifact(&artifact.header().key)
            .await
            .unwrap();
        let mut retained_samples = Vec::new();
        let mut retained_endpoints = Vec::new();
        for ordinal in 0..artifact.header().block_count {
            let block = read.block(ordinal).await.unwrap();
            match block.relation.as_str() {
                "runtime.simulation_samples" => retained_samples.extend(
                    simulation_samples::Row::rows(
                        &artifact
                            .block(ordinal, CancellationToken::new())
                            .await
                            .unwrap(),
                    )
                    .unwrap(),
                ),
                "runtime.trajectory_endpoints" => retained_endpoints.extend(
                    trajectory_endpoints::Row::rows(
                        &artifact
                            .block(ordinal, CancellationToken::new())
                            .await
                            .unwrap(),
                    )
                    .unwrap(),
                ),
                _ => {}
            }
        }
        read.release().await.unwrap();
        retained_samples.sort_by_key(|row| (row.symbol_id, row.sample));
        samples.sort_by_key(|row| (row.symbol_id, row.sample));
        assert_eq!(retained_samples, samples);
        assert_eq!(retained_endpoints, endpoints);
        assert!(
            runtime
                .promote_scientific_artifact(
                    run,
                    attempt,
                    ScientificArtifactKind::LocalAnalysis,
                    "user",
                    "partial-event-scalar-refusal",
                    CancellationToken::new()
                )
                .await
                .is_err()
        );
    }
}

#[cfg(test)]
mod intent_unit {
    #![allow(
        clippy::unwrap_used,
        reason = "strict original document boundary controls"
    )]
    use super::*;
    #[test]
    fn artifact_recovery_refuses_malformed_and_oversized_documents() {
        assert!(decode_intent(b"{}").is_err());
        assert!(decode_intent(&vec![b' '; ARTIFACT_INTENT_BYTES + 1]).is_err());
        let authority = "original-authority";
        let nonce = "00000000-0000-0000-0000-000000000001";
        let mut intent = Artifact {
            key: format!("pse.artifact.v1:{authority}:{nonce}:123"),
            kind: "seed".into(),
            revision: "original-revision".into(),
            primary_problem: "problem".into(),
            primary_authority: authority.into(),
            creation_nonce: nonce.into(),
            creation_expires_at: 123,
            creation_request_digest: "0".repeat(64),
            interpretation: String::new(),
            provenance: Vec::new().into(),
            block_count: 1,
            payload_bytes: 1,
            active: false,
            retiring: false,
        };
        let bytes = serde_json::to_vec(&intent).unwrap();
        assert_eq!(decode_intent(&bytes).unwrap(), intent);
        intent.creation_expires_at += 1;
        assert!(
            decode_intent(&serde_json::to_vec(&intent).unwrap()).is_err(),
            "altered window does not match original identity"
        );
        let mut unknown: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        unknown
            .as_object_mut()
            .unwrap()
            .insert("fresh_intent".into(), true.into());
        assert!(
            decode_intent(&serde_json::to_vec(&unknown).unwrap()).is_err(),
            "generated original document remains strict"
        );
    }
}

#[cfg(all(test, feature = "canonical-tests"))]
mod response_ownership_tests {
    #![allow(
        clippy::unwrap_used,
        reason = "canonical response controls require an admitted exact fixture and actual pool"
    )]
    use super::*;

    #[tokio::test]
    async fn canonical_public_artifact_entrypoints_refuse_unprepaid_metadata() {
        let runtime = super::super::durable_tests::durable_runtime();
        let pool = runtime.shared.pool();
        let baseline = pool.reserved();
        let metadata = MetadataResponse::Header.allocation_bytes();
        let selection_metadata = MetadataResponse::Selection.allocation_bytes();
        let copy_envelope = 128 * 1024 * 1024;
        let pressure = MemoryConsumer::new("control:artifact-response-pressure").register(&pool);
        let datafusion::execution::memory_pool::MemoryLimit::Finite(capacity) = pool.memory_limit()
        else {
            panic!("canonical fixture requires a finite shared pool");
        };
        // Leave exactly one byte less than the metadata envelope, rather than
        // making all work impossible or changing the actual fixture capacity.
        pressure
            .try_grow(capacity - baseline - metadata + 1)
            .unwrap();
        let refused = runtime.scientific_artifact("unissued-artifact").await;
        assert!(
            matches!(refused, Err(WorkflowError::Engine(_))),
            "public acquisition must prepay before querying an absent artifact: {refused:?}"
        );
        // Promotion's existing copy-out allowance can succeed. Its distinct
        // first metadata response must still refuse, without leaking that grant.
        pressure.shrink(copy_envelope + selection_metadata - metadata);
        let before = pool.reserved();
        let refused = runtime
            .promote_scientific_artifact(
                "unissued-run",
                "unissued-attempt",
                ScientificArtifactKind::FinalState,
                "user",
                "unissued-owner",
                CancellationToken::new(),
            )
            .await;
        assert!(
            matches!(refused, Err(WorkflowError::Engine(_))),
            "copy-out admission cannot substitute for metadata prepayment: {refused:?}"
        );
        assert_eq!(pool.reserved(), before);
        drop(pressure);
        assert_eq!(pool.reserved(), baseline);
    }
}

#[cfg(test)]
mod scientific_ack_unit {
    use super::*;
    fn intent() -> Artifact {
        Artifact {
            key: "original-occurrence".into(),
            kind: "trajectory".into(),
            revision: "source".into(),
            primary_problem: "problem".into(),
            primary_authority: "authority".into(),
            creation_nonce: "nonce".into(),
            creation_expires_at: 1,
            creation_request_digest: "exact-byte-and-member-request".into(),
            interpretation: "original-interpretation".into(),
            provenance: vec![1, 2].into(),
            block_count: 2,
            payload_bytes: 10,
            active: false,
            retiring: false,
        }
    }
    #[test]
    fn scientific_ack_requires_exact_original_request_and_active_independent_content() {
        let original = intent();
        let mut ack = original.clone();
        ack.active = true;
        let pool: Arc<dyn pse_columnar::MemoryPool> =
            Arc::new(pse_columnar::GreedyMemoryPool::new(4096));
        check_artifact_ack(&pool, &original, &ack).unwrap();
        assert_eq!(
            pool.reserved(),
            0,
            "temporary equality copy releases its charge"
        );
        let refused: Arc<dyn pse_columnar::MemoryPool> =
            Arc::new(pse_columnar::GreedyMemoryPool::new(1));
        assert!(
            check_artifact_ack(&refused, &original, &ack).is_err(),
            "header clone must be prepaid"
        );
        assert_eq!(refused.reserved(), 0);
        for kind in 0..8 {
            let mut wrong = ack.clone();
            match kind {
                0 => wrong.active = false,
                1 => wrong.retiring = true,
                2 => wrong.key.push('x'),
                3 => wrong.creation_request_digest.push('x'),
                4 => wrong.block_count += 1,
                5 => wrong.payload_bytes += 1,
                6 => wrong.provenance = vec![9].into(),
                _ => wrong.kind = "local-analysis".into(),
            }
            assert!(check_artifact_ack(&pool, &original, &wrong).is_err());
        }
    }
}
