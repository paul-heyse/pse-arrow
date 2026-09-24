// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Member recovery from actual Delta actions. `SetTransaction` is a durable witness;
//! at this pin it does not deduplicate replay. An attempt label is never authority.
use datafusion::{
    common::{DataFusionError, Result},
    execution::{
        memory_pool::{MemoryConsumer, MemoryReservation},
        session_state::SessionState,
    },
};
use deltalake::{
    DeltaTable,
    kernel::{Action, Transaction, transaction::CommitProperties},
};
use pse_ids::SemanticId;
use pse_relations::generated::runtime::publications::RuntimePublicationsFieldMembersItem as Member;
use serde::{Deserialize, Serialize};

const KEY: &str = "pse.member_attempt.v3";

/// A member failure whose settlement differs from an ordinary computation failure.
#[derive(Debug, thiserror::Error)]
pub enum MemberAttemptError {
    /// Retrying with different inputs, implementations, policies or destination refuses.
    #[error("member attempt identity was reused with a different native operation")]
    IdentityReused,
    /// Complete readable history establishes that no data commit was made.
    #[error("member write was rejected without a data commit: {source}")]
    Rejected {
        /// Native operation diagnostic.
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    /// Incomplete history or failed observation cannot establish whether data committed.
    #[error("member write settlement is unresolved: {source}")]
    Unresolved {
        /// Observation failure. Never interpreted as rollback or empty history.
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// Minted by an immutable `ArtifactPlan`, never accepted as a caller-supplied digest.
/// Opaque native implementations deliberately require the same retained composition.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MemberAttempt {
    pub operation_id: SemanticId,
    pub publication_uri: url::Url,
    pub workspace_id: SemanticId,
    pub publication_id: SemanticId,
    pub parent_publication_id: Option<SemanticId>,
    pub attempt_id: SemanticId,
    pub member: Member,
    pub inputs:
        Vec<pse_relations::generated::runtime::publications::RuntimePublicationsFieldInputsItem>,
    pub dependencies: Vec<pse_relations::generated::runtime::native_dependencies::Row>,
    pub base_version: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Phase {
    Provisioned,
    Written,
    Reclaimed,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    request: MemberAttempt,
    phase: Phase,
}

#[derive(Serialize)]
struct ReceiptRef<'a> {
    request: &'a MemberAttempt,
    phase: Phase,
}

/// Count the serialization without retaining a second metadata representation.
#[derive(Default)]
struct SerializedExtent(usize);
impl std::io::Write for SerializedExtent {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 = self
            .0
            .checked_add(bytes.len())
            .ok_or_else(|| std::io::Error::other("member receipt extent overflow"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum MemberState {
    Unpublished,
    Provisioned(u64),
    Committed(u64),
}

impl MemberAttempt {
    pub(super) fn commit(
        &self,
        phase: Phase,
        state: &SessionState,
    ) -> Result<(CommitProperties, MemoryReservation)> {
        let receipt = ReceiptRef {
            request: self,
            phase,
        };
        let mut extent = SerializedExtent::default();
        serde_json::to_writer(&mut extent, &receipt)
            .map_err(|error| DataFusionError::External(Box::new(error)))?;
        let owner = MemoryConsumer::new("pse.delta.receipt_encode")
            .register(&state.runtime_env().memory_pool);
        owner.try_grow(
            extent
                .0
                .checked_mul(96)
                .and_then(|size| size.checked_add(4096))
                .ok_or_else(|| unresolved("member receipt extent overflow"))?,
        )?;
        let mut commit = super::operation::commit_policy(
            CommitProperties::default().with_metadata([(
                KEY.into(),
                serde_json::to_value(receipt)
                    .map_err(|error| DataFusionError::External(Box::new(error)))?,
            )]),
            super::operation::CommitKind::Publication,
        );
        if phase == Phase::Written {
            commit =
                commit.with_application_transaction(Transaction::new(self.transaction_id(), 1));
        }
        Ok((commit, owner))
    }
    fn transaction_id(&self) -> String {
        format!(
            "pse.member:{}:{}:{}:{}",
            self.attempt_id,
            self.member.catalog_name,
            self.member.schema_name,
            self.member.table_name
        )
    }
    /// Observe every relevant native commit. A missing file is an unknown outcome,
    /// not evidence that a write is safe to replay. Version zero is schema creation.
    pub(super) async fn inspect(
        &self,
        table: &DeltaTable,
        state: &SessionState,
    ) -> Result<MemberState> {
        let Some(latest) = table.version() else {
            return Ok(MemberState::Unpublished);
        };
        let first = self
            .base_version
            .map_or(0, |version| version.saturating_add(1));
        let mut found = MemberState::Unpublished;
        for version in first..=latest {
            let actions = super::actions::read(
                table,
                version,
                state,
                &pse_columnar::CancellationToken::new(),
            )
            .await?;
            let transaction_id = self.transaction_id();
            let mut transaction_count = 0;
            let mut receipt = None;
            actions.visit(|action| {
                match action {
                    Action::Txn(transaction)
                        if transaction.app_id == transaction_id && transaction.version == 1 =>
                    {
                        transaction_count += 1;
                    }
                    Action::CommitInfo(mut info) => {
                        if let Some(value) = info.info.remove(KEY) {
                            if receipt.is_some() {
                                return Err(unresolved("duplicate member receipt"));
                            }
                            receipt =
                                Some(serde_json::from_value::<Receipt>(value).map_err(unresolved)?);
                        }
                    }
                    _ => {}
                }
                Ok(())
            })?;
            let has_transaction = transaction_count == 1;
            if let Some(receipt) = receipt {
                if receipt.request.attempt_id != self.attempt_id {
                    continue;
                }
                self.compare(&receipt.request)?;
                found = match receipt.phase {
                    Phase::Reclaimed => {
                        return Err(rejected("member attempt was explicitly reclaimed"));
                    }
                    Phase::Provisioned if found == MemberState::Unpublished => {
                        MemberState::Provisioned(version)
                    }
                    Phase::Written => {
                        if !has_transaction || matches!(found, MemberState::Committed(_)) {
                            return Err(unresolved(
                                "missing or duplicate native application transaction",
                            ));
                        }
                        MemberState::Committed(version)
                    }
                    Phase::Provisioned => {
                        return Err(unresolved("invalid member attempt phase sequence"));
                    }
                };
            }
        }
        Ok(found)
    }
    fn compare(&self, actual: &Self) -> Result<()> {
        if self == actual {
            Ok(())
        } else {
            Err(external(MemberAttemptError::IdentityReused))
        }
    }
}

pub(super) fn unresolved(
    source: impl Into<Box<dyn std::error::Error + Send + Sync>>,
) -> DataFusionError {
    external(MemberAttemptError::Unresolved {
        source: source.into(),
    })
}
pub(super) fn rejected(
    source: impl Into<Box<dyn std::error::Error + Send + Sync>>,
) -> DataFusionError {
    external(MemberAttemptError::Rejected {
        source: source.into(),
    })
}

/// Reclamation requires an actual native member receipt at the current head.
/// An unrelated table, missing commit or later unaccounted mutation refuses.
pub(super) async fn admit_reclamation(
    table: &DeltaTable,
    control: &url::Url,
    state: &SessionState,
) -> Result<(CommitProperties, MemoryReservation)> {
    let version = table
        .version()
        .ok_or_else(|| unresolved("attempt table has no version"))?;
    let actions = super::actions::read(
        table,
        version,
        state,
        &pse_columnar::CancellationToken::new(),
    )
    .await?;
    let mut receipt = None;
    for action in actions.iter() {
        if let Action::CommitInfo(mut info) = action?
            && let Some(value) = info.info.remove(KEY)
        {
            if receipt.is_some() {
                return Err(unresolved("duplicate member receipt"));
            }
            receipt = Some(
                serde_json::from_value::<Receipt>(value)
                    .map_err(|error| DataFusionError::External(Box::new(error)))?,
            );
        }
    }
    let receipt = receipt.ok_or_else(|| rejected("table head is not an owned member attempt"))?;
    let canonical = |location: &url::Url| {
        location
            .to_file_path()
            .map_err(|()| rejected("remote attempt reclamation is unqualified"))?
            .canonicalize()
            .map_err(external)
    };
    if canonical(&receipt.request.publication_uri)? != canonical(control)?
        || canonical(
            &url::Url::parse(&receipt.request.member.table_uri)
                .map_err(|error| DataFusionError::External(Box::new(error)))?,
        )? != canonical(table.table_url())?
    {
        return Err(rejected(
            "member attempt belongs to a different control root or destination",
        ));
    }
    receipt.request.commit(Phase::Reclaimed, state)
}

pub(super) async fn read_dependencies(
    table: &DeltaTable,
    member: &Member,
    state: &SessionState,
) -> Result<(
    Vec<pse_relations::generated::runtime::native_dependencies::Row>,
    MemoryReservation,
)> {
    let version = super::provider::delta_version(member.delta_version)?;
    let actions = super::actions::read(
        table,
        version,
        state,
        &pse_columnar::CancellationToken::new(),
    )
    .await?;
    let mut found = None;
    for action in actions.iter() {
        if let Action::CommitInfo(mut info) = action?
            && let Some(value) = info.info.remove(KEY)
        {
            let receipt: Receipt = serde_json::from_value(value)
                .map_err(|error| DataFusionError::External(Box::new(error)))?;
            let mut recorded = receipt.request.member;
            recorded.delta_version = member.delta_version;
            if receipt.phase != Phase::Written || recorded != *member || found.is_some() {
                return Err(rejected(
                    "member dependency receipt does not describe this exact selected output",
                ));
            }
            found = Some(receipt.request.dependencies);
        }
    }
    found.map(|rows| (rows, actions.owner)).ok_or_else(|| {
        rejected("member has no retained native dependency receipt; recompute explicitly")
    })
}

fn external(error: impl Into<DataFusionError>) -> DataFusionError {
    error.into()
}

pse_diagnostics::impl_diagnostic! {
    MemberAttemptError,
    code(this) { match this {
            Self::IdentityReused => Some(pse_diagnostics::DiagnosticCode::ConfigInvalid),
            Self::Rejected { .. } | Self::Unresolved { .. } => Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure),

            _ => None,
        } },
    forward(_this) { None },
    help(_this) { None },
    related(_this) { None },
    source(_this) { None }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn request() -> MemberAttempt {
        MemberAttempt {
            operation_id: SemanticId::from_bytes([1; 16]),
            publication_uri: url::Url::parse("memory:///workspace/control/").unwrap(),
            workspace_id: SemanticId::from_bytes([2; 16]),
            publication_id: SemanticId::from_bytes([3; 16]),
            parent_publication_id: None,
            attempt_id: SemanticId::from_bytes([4; 16]),
            member: Member {
                catalog_name: "artifact".into(), schema_name: "authored".into(), table_name: "a".into(),
                relation_id: SemanticId::from_bytes([5; 16]), relation_version: 1,
                contract_fingerprint: pse_ids::ContentHash::NIL,
                table_uri: "memory:///workspace/member/".into(), delta_version: 0,
                selection: pse_relations::generated::runtime::publications::RuntimePublicationsFieldMembersItemSelection::from_full(),
            },
            inputs: vec![],
            dependencies: vec![],
            base_version: None,
        }
    }
    #[test]
    fn attempt_equality_requires_actual_operation_inputs_contract_and_policy() {
        let original = request();
        let encoded = serde_json::to_value(Receipt {
            request: original.clone(),
            phase: Phase::Written,
        })
        .unwrap();
        let recovered: Receipt = serde_json::from_value(encoded).unwrap();
        original.compare(&recovered.request).unwrap();
        let mut changed = original.clone();
        changed.operation_id = SemanticId::from_bytes([8; 16]);
        assert!(original.compare(&changed).is_err());
        changed = original.clone();
        changed.member.relation_version += 1;
        assert!(original.compare(&changed).is_err());
        changed = original.clone();
        changed.dependencies.push(
            pse_relations::generated::runtime::native_dependencies::Row {
                kind: pse_relations::generated::enums::NativeDependencyKind::Setting,
                scope: "session".into(),
                name: "qualified.setting".into(),
                evidence: pse_relations::generated::runtime::native_dependencies::RuntimeNativeDependenciesFieldEvidence::from_text(
                    pse_relations::generated::runtime::native_dependencies::RuntimeNativeDependenciesFieldEvidenceText { value: "changed".into() }),
            },
        );
        assert!(original.compare(&changed).is_err());
        changed = original.clone();
        changed
            .inputs
            .push(serde_json::from_value(serde_json::to_value(&original.member).unwrap()).unwrap());
        assert!(original.compare(&changed).is_err());
        changed = original.clone();
        changed.base_version = Some(1);
        assert!(original.compare(&changed).is_err());
        assert!(
            original
                .commit(
                    Phase::Written,
                    &datafusion::prelude::SessionContext::new().state()
                )
                .is_ok()
        );
    }

    #[test]
    fn receipt_serialization_reserves_before_growth_and_releases_with_commit() {
        use datafusion::execution::{
            memory_pool::GreedyMemoryPool, runtime_env::RuntimeEnvBuilder,
        };
        let pool = std::sync::Arc::new(GreedyMemoryPool::new(1 << 20));
        let runtime = RuntimeEnvBuilder::new()
            .with_memory_pool(pool.clone())
            .build_arc()
            .unwrap();
        let state = datafusion::prelude::SessionContext::new_with_config_rt(
            datafusion::execution::config::SessionConfig::new(),
            runtime,
        )
        .state();
        let (commit, owner) = request().commit(Phase::Written, &state).unwrap();
        assert!(state.runtime_env().memory_pool.reserved() > 0);
        drop((commit, owner));
        assert_eq!(state.runtime_env().memory_pool.reserved(), 0);
        let mut large = request();
        large.member.table_name = "x".repeat(1 << 20);
        assert!(large.commit(Phase::Written, &state).is_err());
        assert_eq!(state.runtime_env().memory_pool.reserved(), 0);
    }
}

pse_columnar::impl_native_error!(MemberAttemptError);
