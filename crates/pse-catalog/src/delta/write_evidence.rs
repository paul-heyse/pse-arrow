// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Volatile proof from this invocation's successfully completed native writes.
//! No receipt, table property or caller-supplied fingerprint constructs a completion.
use datafusion::{common::Result, execution::session_state::SessionState};
use pse_engine::session::execution::NativeExecutionContext;
use pse_model::HeapUsage;
use pse_relations::generated::runtime::publications;
use std::sync::{Arc, Mutex, Weak};

type EvidenceKey = (
    pse_ids::SemanticId,
    pse_ids::SemanticId,
    pse_ids::SemanticId,
    u64,
);
#[derive(Debug, Default)]
struct WriteEvidenceSet(Mutex<std::collections::BTreeMap<EvidenceKey, Vec<MemberWriteCompletion>>>);
#[derive(Debug)]
struct MemberWriteCompletion {
    workspace_id: pse_ids::SemanticId,
    publication_id: pse_ids::SemanticId,
    parent_publication_id: Option<pse_ids::SemanticId>,
    attempt_id: pse_ids::SemanticId,
    member: publications::RuntimePublicationsFieldMembersItem,
    inputs: Vec<publications::RuntimePublicationsFieldInputsItem>,
    registry: Weak<pse_schema::Registry>,
    contract: super::contract::DeclaredCheck,
    version: u64,
    complete_contents: bool,
    _owner: pse_columnar::MemoryReservation,
}
/// Reserved before native effects. Only `finish` after native success establishes proof.
#[derive(Debug)]
pub(super) struct Pending {
    set: Arc<WriteEvidenceSet>,
    completion: MemberWriteCompletion,
}
// Completion checking only consumes these typed fields. Native dependencies belong
// to the durable attempt receipt, not to this invocation-local content proof.
fn completion_extent(attempt: &super::attempt::MemberAttempt) -> usize {
    size_of::<MemberWriteCompletion>()
        .saturating_add(attempt.member.heap_bytes())
        .saturating_add(attempt.inputs.heap_bytes())
        // Map/Vec nodes, capacity growth and allocator bookkeeping.
        .saturating_add(4096)
}
impl Pending {
    pub(super) fn reserve(
        state: &SessionState,
        attempt: &super::attempt::MemberAttempt,
        contract: &super::contract::DeclaredCheck,
        complete_contents: bool,
    ) -> Result<Option<Self>> {
        let services = NativeExecutionContext::from_session(state)?;
        let declared =
            super::contract::DeclaredCheck::new(services.registry(), attempt.member.relation_id)?;
        if !contract.same_declaration(&declared) {
            return Ok(None);
        }
        let owner = pse_columnar::MemoryConsumer::new("pse.delta.write_completion")
            .register(services.pool());
        // Refusal loses an optimization, not an otherwise valid native write.
        if owner.try_grow(completion_extent(attempt)).is_err() {
            return Ok(None);
        }
        let set = match services.invocation_resource(|| Ok(WriteEvidenceSet::default())) {
            Ok(set) => set,
            Err(datafusion::common::DataFusionError::ResourcesExhausted(_)) => return Ok(None),
            Err(error) => return Err(error),
        };
        Ok(Some(Self {
            set,
            completion: MemberWriteCompletion {
                workspace_id: attempt.workspace_id,
                publication_id: attempt.publication_id,
                parent_publication_id: attempt.parent_publication_id,
                attempt_id: attempt.attempt_id,
                member: attempt.member.clone(),
                inputs: attempt.inputs.clone(),
                registry: Arc::downgrade(services.registry()),
                contract: contract.clone(),
                version: 0,
                complete_contents,
                _owner: owner,
            },
        }))
    }
    pub(super) fn finish(mut self, version: u64) {
        self.completion.version = version;
        self.set
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .entry((
                self.completion.publication_id,
                self.completion.attempt_id,
                self.completion.member.relation_id,
                version,
            ))
            .or_default()
            .push(self.completion);
    }
}

pub(super) fn establishes(
    state: &SessionState,
    registry: &Arc<pse_schema::Registry>,
    record: &publications::Row,
    member: &publications::RuntimePublicationsFieldMembersItem,
) -> Result<bool> {
    let Some(services) = state.config().get_extension::<NativeExecutionContext>() else {
        return Ok(false);
    };
    let Some(set) = services.find_invocation_resource::<WriteEvidenceSet>()? else {
        return Ok(false);
    };
    let Ok(version) = u64::try_from(member.delta_version) else {
        return Ok(false);
    };
    let contract = super::contract::DeclaredCheck::new(registry, member.relation_id)?;
    let values = set
        .0
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    Ok(values
        .get(&(
            record.publication_id,
            record.attempt_id,
            member.relation_id,
            version,
        ))
        .into_iter()
        .flatten()
        .any(|value| value.matches(registry, record, member, &contract)))
}
impl MemberWriteCompletion {
    fn matches(
        &self,
        registry: &Arc<pse_schema::Registry>,
        record: &publications::Row,
        member: &publications::RuntimePublicationsFieldMembersItem,
        contract: &super::contract::DeclaredCheck,
    ) -> bool {
        let mut expected = self.member.clone();
        let Ok(version) = i64::try_from(self.version) else {
            return false;
        };
        expected.delta_version = version;
        self.complete_contents
            && expected == *member
            && self
                .registry
                .upgrade()
                .is_some_and(|owner| Arc::ptr_eq(&owner, registry))
            && self.contract.same_declaration(contract)
            && self.workspace_id == record.workspace_id
            && self.publication_id == record.publication_id
            && self.parent_publication_id == record.parent_publication_id
            && self.attempt_id == record.attempt_id
            && self.inputs == record.inputs
    }
}

#[cfg(test)]
mod completion_unit {
    use super::*;
    #[test]
    fn coverage_and_exact_member_owner_bound_local_evidence() {
        let registry = pse_schema::shared_registry().unwrap();
        let spec = registry.relation("authored.documents").unwrap();
        let contract = super::super::contract::DeclaredCheck::new(&registry, spec.id).unwrap();
        let member = publications::RuntimePublicationsFieldMembersItem {
            catalog_name: "artifact".into(),
            schema_name: "authored".into(),
            table_name: "documents".into(),
            relation_id: spec.id,
            relation_version: i64::from(spec.key.version),
            contract_fingerprint: spec.fingerprint,
            table_uri: "memory:///documents/".into(),
            delta_version: 7,
            selection: publications::RuntimePublicationsFieldMembersItemSelection::from_full(),
        };
        let record = publications::Row {
            workspace_id: pse_ids::SemanticId::from_bytes([1; 16]),
            publication_id: pse_ids::SemanticId::from_bytes([2; 16]),
            parent_publication_id: None,
            attempt_id: pse_ids::SemanticId::from_bytes([3; 16]),
            kind: pse_relations::generated::enums::PublicationKind::Relations,
            inputs: vec![],
            members: vec![member.clone()],
        };
        let pool: Arc<dyn pse_columnar::MemoryPool> =
            Arc::new(pse_columnar::GreedyMemoryPool::new(4096));
        let mut completion = MemberWriteCompletion {
            workspace_id: record.workspace_id,
            publication_id: record.publication_id,
            parent_publication_id: record.parent_publication_id,
            attempt_id: record.attempt_id,
            member: member.clone(),
            inputs: vec![],
            registry: Arc::downgrade(&registry),
            contract: contract.clone(),
            version: 7,
            complete_contents: true,
            _owner: pse_columnar::MemoryConsumer::new("unit:completion").register(&pool),
        };
        assert!(completion.matches(&registry, &record, &member, &contract));
        completion.complete_contents = false;
        assert!(
            !completion.matches(&registry, &record, &member, &contract),
            "append does not certify untouched rows"
        );
        completion.complete_contents = true;
        let mut changed = member.clone();
        changed.delta_version = 8;
        assert!(!completion.matches(&registry, &record, &changed, &contract));
        changed = member.clone();
        changed.table_uri = "memory:///other/".into();
        assert!(!completion.matches(&registry, &record, &changed, &contract));
        let mut next = record.clone();
        next.attempt_id = pse_ids::SemanticId::from_bytes([9; 16]);
        assert!(!completion.matches(&registry, &next, &member, &contract));
        completion.registry = Weak::new();
        assert!(
            !completion.matches(&registry, &record, &member, &contract),
            "properties alone are not a local check witness"
        );
    }
}
