// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Pre-effect recovery tickets. Settlement observes native witnesses and never executes producers.
use super::{
    attempt::{MemberAttempt, MemberState},
    publication::PublicationRoot,
};
use datafusion::{
    common::{DataFusionError, Result},
    execution::session_state::SessionState,
};
use pse_columnar::CancellationToken;
use pse_relations::generated::runtime::publications;
use std::sync::Arc;

/// Serializable complete publication request, minted before any native write.
/// Deserializing it grants no admission: settlement compares every recorded witness.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicationTicket {
    version: u32,
    location: url::Url,
    candidate: publications::Row,
    attempts: Vec<MemberAttempt>,
}
/// Read-only settlement result. Missing history is always unresolved.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum PublicationSettlement {
    /// Exact native control commit and complete request witnesses agree.
    Committed {
        /// Immutable root, never a latest alias.
        root: PublicationRoot,
    },
    /// Positive evidence excludes this conditional publication.
    ProvedNoncommit {
        /// Durable evidence establishing exclusion.
        reason: String,
    },
    /// Identity or parent conflicts with positive native observations.
    Conflict {
        /// Observed conflict.
        reason: String,
    },
    /// No conclusion is safe; retain the ticket and all unresolved members.
    Unresolved {
        /// Missing or failed observation.
        reason: String,
    },
}
impl PublicationTicket {
    pub(super) fn new(
        location: url::Url,
        mut candidate: publications::Row,
        attempts: Vec<MemberAttempt>,
    ) -> Self {
        candidate.members.sort_by(|a, b| {
            (&a.catalog_name, &a.schema_name, &a.table_name).cmp(&(
                &b.catalog_name,
                &b.schema_name,
                &b.table_name,
            ))
        });
        Self {
            version: 1,
            location,
            candidate,
            attempts,
        }
    }
    /// Identity of the original attempt, not a new retry.
    pub fn attempt_id(&self) -> pse_ids::SemanticId {
        self.candidate.attempt_id
    }
    /// Intended immutable publication identity.
    pub fn publication_id(&self) -> pse_ids::SemanticId {
        self.candidate.publication_id
    }
    /// Complete planned member inventory. New members receive actual versions at settlement.
    pub fn members(&self) -> &[publications::RuntimePublicationsFieldMembersItem] {
        &self.candidate.members
    }
    /// Inspect a completed or interrupted attempt through the caller's native policy and pool.
    /// This method issues no writes, producer executions, schema initialization, or solver calls.
    pub async fn settle(
        &self,
        registry: Arc<pse_schema::Registry>,
        state: Arc<SessionState>,
        cancel: &CancellationToken,
    ) -> PublicationSettlement {
        match self.observe(registry, state, cancel).await {
            Ok(result) => result,
            Err(error) => {
                let conflict = pse_columnar::observe(&error, pse_columnar::PlanOrigin::Analytics)
                    .iter()
                    .any(|observation| {
                        observation.domain_cause.is_some_and(|cause| {
                            let cause: &dyn std::error::Error = cause;
                            matches!(
                                cause.downcast_ref::<super::attempt::MemberAttemptError>(),
                                Some(super::attempt::MemberAttemptError::IdentityReused)
                            )
                        })
                    });
                if conflict {
                    PublicationSettlement::Conflict {
                        reason: error.to_string(),
                    }
                } else {
                    PublicationSettlement::Unresolved {
                        reason: error.to_string(),
                    }
                }
            }
        }
    }
    async fn observe(
        &self,
        registry: Arc<pse_schema::Registry>,
        state: Arc<SessionState>,
        cancel: &CancellationToken,
    ) -> Result<PublicationSettlement> {
        let invalid = |message: &str| DataFusionError::Plan(message.into());
        cancel
            .checkpoint()
            .map_err(|e| DataFusionError::External(Box::new(e)))?;
        if self.version != 1 || self.candidate.members.is_empty() || self.attempts.is_empty() {
            return Err(invalid("malformed or unsupported publication ticket"));
        }
        let _control = super::lease::read(&self.location, cancel).await?;
        // Resolve the latest native version freshly; retained exact snapshots may serve that version.
        let Some(opened) = super::publish::load(&self.location, &state).await? else {
            return Ok(PublicationSettlement::Unresolved {
                reason: "no durable control witness".into(),
            });
        };
        super::contract::DeclaredCheck::new(&registry, publications::RELATION_ID)?
            .verify(&opened.table)?;
        let snapshot = opened
            .table
            .snapshot()
            .map_err(|e| DataFusionError::External(Box::new(e)))?;
        let log = opened.table.log_store();
        let attempt = snapshot
            .transaction_version(log.as_ref(), format!("pse.attempt:{}", self.attempt_id()))
            .await
            .map_err(|e| DataFusionError::External(Box::new(e)))?;
        let publication = snapshot
            .transaction_version(
                log.as_ref(),
                format!("pse.publication:{}", self.publication_id()),
            )
            .await
            .map_err(|e| DataFusionError::External(Box::new(e)))?;
        let committed = match (attempt, publication) {
            (Some(a), Some(b)) if a == b => Some(a),
            (None, None) => None,
            _ => {
                return Ok(PublicationSettlement::Conflict {
                    reason: "publication or attempt identity belongs to a different native commit"
                        .into(),
                });
            }
        };
        if let Some(version) = committed {
            let root = PublicationRoot {
                location: self.location.clone(),
                version,
            };
            let actual = super::publication::read_record(&root, &registry, state.clone()).await?;
            let mut expected = self.candidate.clone();
            for request in &self.attempts {
                cancel
                    .checkpoint()
                    .map_err(|e| DataFusionError::External(Box::new(e)))?;
                if request.publication_uri != self.location
                    || request.attempt_id != self.attempt_id()
                    || request.publication_id != self.publication_id()
                {
                    return Err(invalid("ticket member belongs to another request"));
                }
                let location = url::Url::parse(&request.member.table_uri)
                    .map_err(|e| DataFusionError::External(Box::new(e)))?;
                let _reader = super::lease::read(&location, cancel).await?;
                let member = super::provider::open_native(
                    location,
                    None,
                    crate::cache_service::snapshot::LoadRequirement::Query,
                    &state,
                )
                .await?;
                let MemberState::Committed(version) =
                    request.inspect(&member.table, &state).await?
                else {
                    return Err(invalid(
                        "published member has no complete matching write witness",
                    ));
                };
                let selected = expected
                    .members
                    .iter_mut()
                    .find(|m| {
                        m.catalog_name == request.member.catalog_name
                            && m.schema_name == request.member.schema_name
                            && m.table_name == request.member.table_name
                    })
                    .ok_or_else(|| invalid("ticket omits written member"))?;
                selected.delta_version = super::provider::signed_version(version)?;
            }
            if actual != expected {
                return Ok(PublicationSettlement::Conflict {
                    reason: "complete published request differs from ticket".into(),
                });
            }
            return Ok(PublicationSettlement::Committed { root });
        }
        // Absence of Txn markers alone is never proof. A different committed head
        // positively defeats the exact-parent/no-rebase condition for this request.
        let version = super::provider::signed_version(
            opened
                .table
                .version()
                .ok_or_else(|| invalid("control version absent"))?,
        )?;
        let current = super::publication::read_optional_record(
            &PublicationRoot {
                location: self.location.clone(),
                version,
            },
            &registry,
            state,
        )
        .await?;
        if current.is_some_and(|r| {
            r.workspace_id != self.candidate.workspace_id
                || Some(r.publication_id) != self.candidate.parent_publication_id
        }) {
            return Ok(PublicationSettlement::Conflict { reason: "fresh control head differs from the exact requested parent; no matching transaction witness is available".into() });
        }
        Ok(PublicationSettlement::Unresolved {
            reason: "no commit witness; the conditional attempt may still be active".into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifact::{ArtifactPlan, PublicationTarget, RelationOutput};
    use datafusion::{
        arrow::{array::RecordBatch, datatypes::DataType},
        common::ResolvedTableReference,
    };
    use pse_ids::SemanticId;
    use pse_schema::{
        RegistryBuilder,
        model::{Authority, FieldContract as F, Namespace, RelationDecl, SnapshotClass},
    };
    use std::collections::BTreeMap;

    #[tokio::test]
    async fn saved_ticket_settles_exact_native_commit_without_rerunning_producers() {
        let mut builder = RegistryBuilder::new();
        pse_schema::catalog::declare_publications(&mut builder);
        builder.declare_relation(
            RelationDecl::new(
                Namespace::Authored,
                "ticket_values",
                1,
                Authority::Authored,
                SnapshotClass::Model,
                "Ticket value",
            )
            .pk(&["id"])
            .columns(vec![F::key("id", F::native(DataType::Int64), "Key")]),
        );
        let registry = Arc::new(builder.build().unwrap());
        let spec = registry.relation("authored.ticket_values").unwrap();
        let schema = Arc::new(pse_schema::arrow::relation_schema(&registry, spec).unwrap());
        let factory = pse_testkit::NativeFixture::new((256 << 20).try_into().unwrap())
            .unwrap()
            .into_factory()
            .with_query_planner(Arc::new(pse_engine::session::planner::UnifiedPlanner::new(
                crate::assembly::planners(),
            )));
        let cancel = CancellationToken::new();
        let session = factory
            .candidate(
                BTreeMap::from([(spec.key, RecordBatch::new_empty(schema))]),
                registry.clone(),
                &cancel,
            )
            .unwrap();
        let source = session.table_reference(&spec.key).unwrap().resolve("", "");
        let name = ResolvedTableReference {
            catalog: "artifact".into(),
            schema: "authored".into(),
            table: "ticket_values".into(),
        };
        let plan = session.relation_plan(&source).unwrap().plan().clone();
        let artifact = ArtifactPlan::new(
            session,
            BTreeMap::from([(
                name.clone(),
                RelationOutput {
                    relation_id: spec.id,
                    plan,
                },
            )]),
            &cancel,
        )
        .unwrap();
        let directory = tempfile::tempdir().unwrap();
        let base = url::Url::from_directory_path(directory.path()).unwrap();
        let header = publications::Row {
            workspace_id: SemanticId::from_bytes([1; 16]),
            publication_id: SemanticId::from_bytes([2; 16]),
            parent_publication_id: None,
            attempt_id: SemanticId::from_bytes([3; 16]),
            kind: pse_relations::generated::enums::PublicationKind::Relations,
            inputs: vec![],
            members: vec![],
        };
        let (command, ticket) = artifact
            .prepare_publication(
                PublicationTarget {
                    reference: ResolvedTableReference {
                        catalog: "artifact".into(),
                        schema: "runtime".into(),
                        table: "publications".into(),
                    },
                    location: base.join("control/").unwrap(),
                },
                header,
                BTreeMap::from([(name.clone(), base.join("members/attempt/values/").unwrap())]),
                vec![],
                &cancel,
            )
            .unwrap();
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 0);
        let ticket: PublicationTicket =
            serde_json::from_slice(&serde_json::to_vec(&ticket).unwrap()).unwrap();
        let state = Arc::new(factory.native_state().clone());
        assert!(matches!(
            ticket
                .settle(registry.clone(), state.clone(), &cancel)
                .await,
            PublicationSettlement::Unresolved { .. }
        ));
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 0);
        let child = |byte: u8| {
            artifact
                .prepare_publication(
                    PublicationTarget {
                        reference: ResolvedTableReference {
                            catalog: "artifact".into(),
                            schema: "runtime".into(),
                            table: "publications".into(),
                        },
                        location: base.join("control/").unwrap(),
                    },
                    publications::Row {
                        workspace_id: SemanticId::from_bytes([1; 16]),
                        publication_id: SemanticId::from_bytes([byte; 16]),
                        parent_publication_id: Some(ticket.publication_id()),
                        attempt_id: SemanticId::from_bytes([byte + 1; 16]),
                        kind: pse_relations::generated::enums::PublicationKind::Relations,
                        inputs: vec![],
                        members: vec![],
                    },
                    BTreeMap::from([(
                        name.clone(),
                        base.join(&format!("members/child-{byte}/values/")).unwrap(),
                    )]),
                    vec![],
                    &cancel,
                )
                .unwrap()
        };
        let (next, next_ticket) = child(4);
        let (competing, competing_ticket) = child(6);
        drop(artifact);
        drop(command.execute(&cancel).await.unwrap());
        let result = ticket
            .settle(registry.clone(), state.clone(), &cancel)
            .await;
        let PublicationSettlement::Committed { root } = result else {
            panic!("{result:?}");
        };
        assert_eq!(root.version, 1);
        drop(next.execute(&cancel).await.unwrap());
        assert!(matches!(
            next_ticket
                .settle(registry.clone(), state.clone(), &cancel)
                .await,
            PublicationSettlement::Committed {
                root: PublicationRoot { version: 2, .. }
            }
        ));
        assert!(competing.execute(&cancel).await.is_err());
        assert!(matches!(
            competing_ticket
                .settle(registry.clone(), state.clone(), &cancel)
                .await,
            PublicationSettlement::Conflict { .. }
        ));
        // A later head and an unpublished competing member cannot alter the exact old result.
        let old = super::super::publication::Publication::open(
            root.clone(),
            registry.clone(),
            &factory,
            &cancel,
        )
        .await
        .unwrap();
        assert_eq!(old.record().publication_id, ticket.publication_id());
        drop(old);
        assert_eq!(
            ticket
                .settle(registry.clone(), state.clone(), &cancel)
                .await,
            PublicationSettlement::Committed { root }
        );
        // Presentation-only registry edits reopen the actual stored contract.
        let mut changed = RegistryBuilder::new();
        pse_schema::catalog::declare_publications(&mut changed);
        changed.declare_relation(
            RelationDecl::new(
                Namespace::Authored,
                "ticket_values",
                1,
                Authority::Authored,
                SnapshotClass::Model,
                "Revised prose only",
            )
            .pk(&["id"])
            .columns(vec![F::key(
                "id",
                F::native(DataType::Int64),
                "Revised field prose",
            )]),
        );
        let changed = Arc::new(changed.build().unwrap());
        let compatible = super::super::publication::Publication::open(
            PublicationRoot {
                location: ticket.location.clone(),
                version: 1,
            },
            changed,
            &factory,
            &cancel,
        )
        .await
        .unwrap();
        assert_eq!(compatible.record().publication_id, ticket.publication_id());
        drop(compatible);
        let mut altered = ticket.clone();
        altered.candidate.members[0].contract_fingerprint = pse_ids::ContentHash::NIL;
        assert!(matches!(
            altered
                .settle(registry.clone(), state.clone(), &cancel)
                .await,
            PublicationSettlement::Conflict { .. }
        ));
        // Missing commit history cannot become a noncommit verdict or a retry instruction.
        std::fs::remove_file(
            directory
                .path()
                .join("members/attempt/values/_delta_log/00000000000000000001.json"),
        )
        .unwrap();
        assert!(matches!(
            ticket.settle(registry, state, &cancel).await,
            PublicationSettlement::Unresolved { .. }
        ));
    }
}
