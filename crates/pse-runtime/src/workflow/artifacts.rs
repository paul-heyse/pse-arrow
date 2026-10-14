// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Scientific closure admission and independent, bounded retained content.

use super::{Runtime, WorkflowError, contract};
use pse_columnar::{CancellationToken, MemoryConsumer};
use pse_operations::canonical_artifacts::{Artifact, ArtifactLocator};
use pse_relations::columnar::RelationRow;

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

/// A promised scientific product, distinct from arbitrary result row selection.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ScientificArtifactKind {
    /// Actual terminal physical state, including its observed endpoint assessment.
    FinalState,
    /// Complete recorded trajectory prefix, transitions and endpoint interpretation.
    Trajectory,
    /// Original local derivative/uncertainty rows with qualification and assumptions.
    LocalAnalysis,
}
impl ScientificArtifactKind {
    fn name(self) -> &'static str {
        match self {
            Self::FinalState => "final-state",
            Self::Trajectory => "trajectory",
            Self::LocalAnalysis => "local-analysis",
        }
    }
}

/// A restartable independent scientific owner. Each read acquires current permission.
#[derive(Clone, Debug)]
pub struct ScientificArtifact {
    runtime: Runtime,
    header: Artifact,
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
        let owner = std::sync::Arc::new(MemoryConsumer::new("canonical:artifact-metadata")
            .register(&self.runtime.shared.pool()));
        owner
            .try_grow(12 * 1024 * 1024)
            .map_err(pse_engine::EngineError::from)?;
        let row = pse_operations::canonical::response_ownership_scope(
            owner.clone(),
            self.runtime.canonical_store()?.artifact(&self.header.key),
        ).await?;
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
        let owner = MemoryConsumer::new("canonical:scientific-artifact-read")
            .register(&self.runtime.shared.pool());
        owner
            .try_grow(4 * super::result_files::MAX_FILE_BYTES)
            .map_err(pse_engine::EngineError::from)?;
        let response = super::connected_results::response_owner(
            &self.runtime, "canonical:artifact-block-response",
        )?;
        let read = pse_operations::canonical::response_ownership_scope(
            response.clone(),
            self.runtime.canonical_store()?.read_artifact(&self.header.key),
        ).await?;
        let block = pse_operations::canonical::response_ownership_scope(
            response, read.block(ordinal),
        ).await?;
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
    Ok(pse_model::generated::runtime::scientific_contexts::Row {
        run_id,
        step,
        model_id: Some(solved.model()),
        fit_id: None,
        case_id: solved.case(),
        instance_id: Some(solved.instance()),
        source_identity: prepared.source.revision.identity().as_id(),
        physical_identity: prepared.source.physical.identity(),
        preparation_identity: prepared
            .solve
            .preparation_identity()
            .map_err(crate::math::MathRuntimeError::from)?,
        profile_identity: crate::math::solves::profile_key(&prepared.profile)
            .map_err(crate::math::MathRuntimeError::from)?
            .as_id(),
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
        let header = self
            .operations()?
            .promote_seed(solution, owner_kind, owner)
            .await?;
        Ok(ScientificArtifact {
            runtime: self.clone(),
            header,
        })
    }
    /// Reopen an independent artifact without consulting producer result availability.
    pub async fn scientific_artifact(
        &self,
        key: &str,
    ) -> Result<ScientificArtifact, WorkflowError> {
        let activity = self.admit_activity()?;
        activity.scope(async {
        let response = super::connected_results::response_owner(self, "canonical:scientific-artifact-header-response")?;
        let header = pse_operations::canonical::response_ownership_scope(
            response, self.canonical_store()?.artifact(key),
        ).await?;
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
        Ok(ScientificArtifact {
            runtime: self.clone(),
            header,
        })
        }).await
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
        activity.scope(async {
            let memory=MemoryConsumer::new("canonical:scientific-artifact-promotion").register(&self.shared.pool());
            memory.try_grow(128*1024*1024).map_err(pse_engine::EngineError::from)?;
            let response=super::connected_results::response_owner(self,"canonical:artifact-promotion-selection-response")?;
            let input=std::sync::Arc::new(pse_operations::canonical::response_ownership_scope(
                response,self.canonical_store()?.read_results(run,attempt),
            ).await?);
            let mut names=vec!["runtime.scientific_contexts","runtime.candidate_assessments","runtime.run_lineage","runtime.resolved_numerics","runtime.modeling_checks","runtime.modeling_reports","runtime.modeling_findings"];
            match kind {
                ScientificArtifactKind::FinalState=>names.extend(["runtime.trajectory_endpoints","runtime.computation_runs","runtime.solve_variables","runtime.solve_runs"]),
                ScientificArtifactKind::Trajectory=>names.extend(["runtime.trajectory_endpoints","runtime.simulation_samples","runtime.simulation_events","runtime.modeling_trajectory_modes","runtime.response_sensitivities","runtime.computation_runs"]),
                ScientificArtifactKind::LocalAnalysis=>names.extend(["runtime.local_validity","runtime.parametric_sensitivities","runtime.response_sensitivities","runtime.reduced_hessians","runtime.parameter_covariances","runtime.parameter_intervals","runtime.profile_points","runtime.response_directions","runtime.propagated_covariances","runtime.fit_parameters","runtime.fit_observations","runtime.solve_runs","runtime.computation_runs","runtime.solve_metrics"]),
            }
            let response=super::connected_results::response_owner(self,"canonical:artifact-promotion-sources-response")?;
            let sources=pse_operations::canonical::response_ownership_scope(
                std::sync::Arc::new((input.clone(),response)),self.canonical_store()?.analysis_run_sources(run),
            ).await?;
            let revision=sources.iter().find(|source|source.key==input.run().revision).ok_or_else(||contract("artifact primary source unavailable"))?;
            let members=MemoryConsumer::new("canonical:scientific-artifact-member-files").register(&self.shared.pool());
            let mut copied=Vec::new();
            let mut contexts=Vec::new();
            let mut endpoints=Vec::new();
            let mut selected=Vec::new();
            for name in names {
                let spec=self.registry().relation(name).ok_or_else(||contract("scientific artifact relation unavailable"))?;
                let Some(set)=input.sets().iter().find(|set|set.name==spec.id.to_string()) else {continue;};
                if copied.len().saturating_add(set.batch_count.max(1) as usize)>64 { return Err(contract("scientific artifact exceeds bounded selection; choose a smaller scientific product")); }
                selected.push((name,set.row_count,set.batches_digest.clone()));
                if set.batch_count==0 {
                    let schema=pse_schema::arrow::relation_schema_ref(self.registry(),spec).map_err(|error|contract(error.to_string()))?;
                    let payload=super::result_files::encode_result_file(&datafusion::arrow::record_batch::RecordBatch::new_empty(schema))?;
                    members.try_grow(payload.capacity()).map_err(pse_engine::EngineError::from)?;
                    copied.push((name,payload,0,set.batches_digest.clone()));
                }
                if set.batch_count != 0 {
                    let schema=pse_schema::arrow::relation_schema_ref(self.registry(),spec).map_err(|error|contract(error.to_string()))?;
                    let snapshot=super::ResultSnapshot::capture(self,input.clone(),spec.id,&set.key,schema,0,u64::MAX).await?;
                    for block in snapshot.blocks() {
                        cancel.checkpoint().map_err(pse_engine::EngineError::from)?;
                        let checked=snapshot.read_block(block,&cancel).await?;
                        // Encoding scratch is local to this demanded logical batch;
                        // the packed producer file is read through the snapshot cache.
                        let scratch=MemoryConsumer::new("canonical:scientific-artifact-member-encoding").register(&self.shared.pool());
                        scratch.try_grow(3*super::result_files::MAX_FILE_BYTES).map_err(pse_engine::EngineError::from)?;
                        let member=super::result_files::encode_result_file(checked.batch())?;
                        members.try_grow(member.capacity()).map_err(pse_engine::EngineError::from)?;
                        if name=="runtime.scientific_contexts" || name=="runtime.trajectory_endpoints" {
                            memory.try_grow(member.len().saturating_mul(4)).map_err(pse_engine::EngineError::from)?;
                        }
                        if name=="runtime.scientific_contexts" {contexts.extend(pse_relations::generated::runtime::scientific_contexts::Row::rows(&checked).map_err(super::relation)?);}
                        if name=="runtime.trajectory_endpoints" {endpoints.extend(pse_relations::generated::runtime::trajectory_endpoints::Row::rows(&checked).map_err(super::relation)?);}
                        copied.push((name,member,block.rows,block.payload_digest.clone()));
                    }
                }
            }
            if contexts.is_empty() || !selected.iter().any(|(name,rows,_)|*name=="runtime.candidate_assessments" && *rows>0) || !selected.iter().any(|(name,rows,_)|*name=="runtime.run_lineage" && *rows>0) { return Err(contract("scientific artifact lacks original interpretation/assessment/lineage")); }
            for context in &contexts {
                if context.mode_names.len()!=context.mode_identities.len() || context.configuration.is_empty() || context.state.iter().chain(&context.inputs).chain(&context.outputs).chain(&context.integrals).any(|c|!c.scale.is_finite() || c.scale<=0. || !c.offset.is_finite()) { return Err(contract("scientific artifact coordinate interpretation incomplete")); }
            }
            match kind {
                ScientificArtifactKind::FinalState|ScientificArtifactKind::Trajectory if !endpoints.is_empty()=>{
                    for endpoint in &endpoints {
                        let context=contexts.iter().find(|context|context.run_id==endpoint.run_id && context.time.is_some()).ok_or_else(||contract("endpoint scientific context absent"))?;
                        let mode=usize::try_from(endpoint.mode).map_err(|_|contract("endpoint mode extent"))?;
                        if context.mode_names.get(mode)!=Some(&endpoint.mode_name) || context.mode_identities.get(mode)!=Some(&endpoint.mode_identity) || context.state.iter().map(|c|c.id).collect::<Vec<_>>()!=endpoint.state_ids || context.inputs.iter().map(|c|c.id).collect::<Vec<_>>()!=endpoint.input_ids || context.outputs.iter().map(|c|c.id).collect::<Vec<_>>()!=endpoint.output_ids || context.integrals.len()!=endpoint.integrals.len() || endpoint.state.len()!=endpoint.state_ids.len() || endpoint.inputs.len()!=endpoint.input_ids.len() || endpoint.outputs.len()!=endpoint.output_ids.len() {return Err(contract("endpoint scientific closure differs"));}
                    }
                }
                ScientificArtifactKind::FinalState=>{
                    if !selected.iter().any(|(name,rows,_)|*name=="runtime.solve_variables" && *rows>0) || contexts.iter().any(|context|context.time.is_some()) {return Err(contract("final state requires an actual endpoint or complete static candidate"));}
                }
                ScientificArtifactKind::Trajectory=>return Err(contract("trajectory has no actual endpoint interpretation")),
                ScientificArtifactKind::LocalAnalysis=>{
                    if !selected.iter().any(|(name,rows,_)|*name=="runtime.local_validity" && *rows>0) {return Err(contract("local analysis lacks original certification/withheld reasons"));}
                }
            }
            let provenance=serde_json::to_vec(&(1u8,run,attempt,&input.run().revision,input.run().sequence,input.attempt().generation,&input.attempt().outcome,input.manifest(),&selected)).map_err(|error|contract(error.to_string()))?;
            let bytes=copied.iter().map(|(_,payload,_,_)|payload.len() as u64).sum();
            let response=super::connected_results::response_owner(self,"canonical:artifact-promotion-header-response")?;
            let mut header=pse_operations::canonical::response_ownership_scope(
                std::sync::Arc::new((input.clone(),response)),
                self.canonical_store()?.new_artifact(revision,kind.name(),provenance,copied.len() as u64,bytes),
            ).await?;
            let blocks=copied.into_iter().enumerate().map(|(ordinal,(name,payload,rows,digest))| {
                let spec=self.registry().relation(name).ok_or_else(||contract("artifact member relation unavailable"))?;
                Ok::<_,WorkflowError>(pse_operations::canonical_artifacts::artifact_block(&header.key,ordinal as u64,name,payload,&digest,rows).native_file(&spec.id.to_string())?)
            }).collect::<Result<Vec<_>,_>>()?;
            pse_operations::canonical_artifacts::seal_artifact_request(&mut header,&sources,std::slice::from_ref(input.as_ref()),&blocks,owner_kind,owner);
            let response=super::connected_results::response_owner(self,"canonical:artifact-promotion-publication-response")?;
            pse_operations::canonical::response_ownership_scope(
                // The SDK owns metadata and the original read follower only;
                // copied member buffers retain their separate local grant.
                std::sync::Arc::new((input.clone(),response)),
                self.canonical_store()?.persist_artifact(&header,&sources,std::slice::from_ref(input.as_ref()),&blocks,owner_kind,owner),
            ).await?;
            Ok(ScientificArtifact {runtime:self.clone(),header})
        }).await
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
        let rows=pse_authoring::language::parse("package p { def Root { domain t: Time from 0{s} to 1{s}; discretize grid on t using integrated(elements=1,order=1); var x[i in t]: Time; eq rate[i in t]: d(x[i])/di == -x[i]/1{s}; eq initial: x[0{s}] == 1{s}; let area:Time=integral(i in t | x[i]/1{s}); annotation report area(\"integral\"); } }",pse_ids::SemanticId::NIL,pse_authoring::language::IdentityPolicy::Named,pse_authoring::ParseBudget::default()).unwrap();
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
        let result = prepared.start().unwrap().wait().await.unwrap();
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
    #![allow(clippy::unwrap_used, reason = "canonical response controls require an admitted exact fixture and actual pool")]
    use super::*;

    #[tokio::test]
    async fn canonical_public_artifact_entrypoints_refuse_unprepaid_metadata() {
        let runtime = super::super::durable_tests::durable_runtime();
        let pool = runtime.shared.pool();
        let baseline = pool.reserved();
        let metadata = 3 * super::super::result_files::MAX_FILE_BYTES;
        let copy_envelope = 128 * 1024 * 1024;
        let pressure = MemoryConsumer::new("control:artifact-response-pressure").register(&pool);
        let datafusion::execution::memory_pool::MemoryLimit::Finite(capacity) = pool.memory_limit() else {
            unreachable!("canonical fixture requires a finite shared pool");
        };
        // Leave exactly one byte less than the metadata envelope, rather than
        // making all work impossible or changing the actual fixture capacity.
        pressure.try_grow(capacity - baseline - metadata + 1).unwrap();
        let refused = runtime.scientific_artifact("unissued-artifact").await;
        assert!(matches!(refused, Err(WorkflowError::Engine(_))),
            "public acquisition must prepay before querying an absent artifact: {refused:?}");
        // Promotion's existing copy-out allowance can succeed. Its distinct
        // first metadata response must still refuse, without leaking that grant.
        pressure.shrink(copy_envelope);
        let before = pool.reserved();
        let refused = runtime.promote_scientific_artifact(
            "unissued-run", "unissued-attempt", ScientificArtifactKind::FinalState,
            "user", "unissued-owner", CancellationToken::new(),
        ).await;
        assert!(matches!(refused, Err(WorkflowError::Engine(_))),
            "copy-out admission cannot substitute for metadata prepayment: {refused:?}");
        assert_eq!(pool.reserved(), before);
        drop(pressure);
        assert_eq!(pool.reserved(), baseline);
    }
}
