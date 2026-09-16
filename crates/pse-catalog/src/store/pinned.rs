// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Reopen exact immutable physical bindings, then admit every actual dependency.

mod binding;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use pse_ids::{CancellationToken, ContentHash, Reservation};

use super::control::OwnedControl;
use super::membership::AdmissionContext;
use super::open::Catalog;
use super::refs::RefState;
use super::verify::admission;
use crate::snapshot::ManifestRef;
use crate::{CatalogError, RefName, Snapshot};

use binding::ContextWire;

type Key = (ContentHash, ContentHash);
enum Step {
    Enter(ManifestRef),
    Admit(
        ManifestRef,
        crate::snapshot::OwnedManifest,
        OwnedControl<ContextWire>,
    ),
}
struct Graph {
    pending: Vec<Step>,
    active: BTreeSet<Key>,
    admitted: BTreeMap<Key, Arc<Snapshot>>,
    reservation: Box<dyn Reservation>,
    traversal: Arc<super::traversal::AdmissionTraversal>,
}
fn key(reference: ManifestRef) -> Key {
    (reference.snapshot_id.0, reference.manifest_checksum.0)
}
impl Graph {
    fn push(&mut self, step: Step) -> Result<(), CatalogError> {
        // Covers stack growth/reallocation plus active/admitted tree entries. Payload
        // controls and actual snapshots retain their own independently measured leases.
        let maps = super::control::add(
            super::stage_owned::map_extent::<Key, Arc<Snapshot>>(1)?,
            super::stage_owned::map_extent::<Key, ()>(1)?,
        )?;
        let stack = super::control::mul(size_of::<Step>(), 4)?;
        self.reservation
            .try_grow(super::control::add(maps, stack)?)?;
        self.pending.push(step);
        Ok(())
    }

    fn enter(&mut self, reference: ManifestRef) -> Result<bool, CatalogError> {
        let key = key(reference);
        if self.admitted.contains_key(&key) {
            return Ok(false);
        }
        if !self.active.insert(key) {
            return Err(admission(
                "admission binding",
                "exact dependency bindings contain a cycle",
            ));
        }
        Ok(true)
    }

    fn context(
        &mut self,
        wire: &ContextWire,
        catalog: &Catalog,
    ) -> Result<AdmissionContext, CatalogError> {
        let slots = super::stage_owned::map_extent::<String, Arc<Snapshot>>(wire.parents.len())?;
        let bytes = wire.parents.keys().try_fold(slots, |bytes, role| {
            super::control::add(bytes, role.capacity())
        })?;
        self.reservation.try_grow(bytes)?;
        let parents = wire
            .parents
            .iter()
            .map(|(role, reference)| {
                self.admitted
                    .get(&key(*reference))
                    .map(|snapshot| (role.clone(), Arc::clone(snapshot)))
                    .ok_or_else(|| admission("admission binding", "parent is not admitted"))
            })
            .collect::<Result<_, _>>()?;
        let invocation = wire
            .invocation
            .as_ref()
            .map(|invocation| {
                let document_source = invocation
                    .document_source
                    .map(|reference| {
                        self.admitted.get(&key(reference)).ok_or_else(|| {
                            admission("admission binding", "document source is not admitted")
                        })
                    })
                    .transpose()?;
                for policy in invocation.policies.values() {
                    if !self.admitted.contains_key(&key(policy.snapshot)) {
                        return Err(admission(
                            "admission binding",
                            "policy owner is not admitted",
                        ));
                    }
                }
                super::invocation::InvocationContext::capture(
                    invocation.policies.iter().map(|(role, policy)| {
                        (
                            role.as_str(),
                            policy.policy_id,
                            &self.admitted[&key(policy.snapshot)],
                            policy.port.as_str(),
                        )
                    }),
                    document_source,
                    invocation.engine.as_ref(),
                    catalog.reserver.as_ref(),
                )
            })
            .transpose()?;
        Ok(AdmissionContext {
            parents,
            stage_pass: wire.producer,
            invocation,
            traversal: Arc::clone(&self.traversal),
        })
    }
}

impl Catalog {
    /// Prepare a captured alias-resolution command without reading the backend.
    /// # Errors
    /// Scoped policy, native planning, cancellation or resource refusal.
    pub fn prepare_pinned_ref(
        &self,
        name: RefName,
        cancel: &CancellationToken,
    ) -> Result<
        super::operation::PreparedStoreOperation<Option<super::refs::PinnedRef>>,
        CatalogError,
    > {
        use pse_schema::model::provider::{OperationPurpose, ProviderScope};
        self.prepare_store_operation(
            "store.resolve_ref",
            ProviderScope::Table("store".into(), "refs".into(), name.as_str().into()),
            OperationPurpose::Resolve,
            vec![datafusion::logical_expr::lit(name.as_str())],
            Box::new(move |catalog, _, cancel| {
                Box::pin(async move {
                    let result = catalog.read_pinned_ref_inner(&name, &cancel).await?;
                    let count = u64::from(result.is_some());
                    Ok((result, count))
                })
            }),
            cancel,
        )
    }

    /// Prepare exact current-format admission as an inspectable native operation.
    /// # Errors
    /// Scoped policy, native planning, cancellation or resource refusal.
    pub fn prepare_pinned_manifest(
        &self,
        reference: ManifestRef,
        cancel: &CancellationToken,
    ) -> Result<super::operation::PreparedStoreOperation<Arc<Snapshot>>, CatalogError> {
        use pse_schema::model::provider::{OperationPurpose, ProviderScope};
        self.prepare_store_operation(
            "store.admit_manifest",
            ProviderScope::Schema("store".into(), "manifests".into()),
            OperationPurpose::Resolve,
            vec![
                datafusion::logical_expr::lit(reference.snapshot_id.0.to_prefixed()),
                datafusion::logical_expr::lit(reference.manifest_checksum.0.to_prefixed()),
            ],
            Box::new(move |catalog, _, cancel| {
                Box::pin(async move {
                    Ok((
                        catalog
                            .read_pinned_manifest_inner(reference, &cancel)
                            .await?,
                        1,
                    ))
                })
            }),
            cancel,
        )
    }
    /// Pin a mutable ref once, retaining its exact observed revision and admitting
    /// its complete immutable parent graph. Later ref movement cannot change this pair.
    ///
    /// # Errors
    /// Missing context, malformed bindings, actual row/producer admission, cancellation,
    /// resource exhaustion or backend errors. An absent alias returns `None`.
    pub async fn read_pinned_ref(
        &self,
        name: &RefName,
        cancel: &CancellationToken,
    ) -> Result<Option<(RefState, Arc<Snapshot>)>, CatalogError> {
        Ok(self
            .prepare_pinned_ref(name.clone(), cancel)?
            .execute(cancel)
            .await?
            .into_value())
    }

    async fn read_pinned_ref_inner(
        &self,
        name: &RefName,
        cancel: &CancellationToken,
    ) -> Result<Option<(RefState, Arc<Snapshot>)>, CatalogError> {
        let Some(state) = self.read_ref(name, cancel).await? else {
            return Ok(None);
        };
        let snapshot = self
            .read_pinned_manifest_inner(state.manifest_ref(), cancel)
            .await?;
        if let Some(reference) = state.revision_ref() {
            let artifact = self.read_sidecar(&reference.artifact, cancel).await?;
            let revision = self.revision_receipt(&artifact, reference.revision_id, &snapshot)?;
            if let Some(reference) = &reference.change_set {
                let changes = self.read_change_set(reference, cancel).await?;
                self.check_reopened_change_context(&changes, &revision, cancel)
                    .await?;
            }
        }
        Ok(Some((state, snapshot)))
    }

    /// Admit an exact manifest and every dependency selected by its immutable binding.
    /// No latest-ref lookup, snapshot-ID search, or stage-key match establishes validity.
    /// Traversal is iterative and charged to the common pool, without a fixed depth cap.
    ///
    /// # Errors
    /// Missing context-dependent bindings, cycles, wrong bindings or producers, actual
    /// content/semantic admission errors, cancellation, budgets and backend failures.
    pub async fn read_pinned_manifest(
        &self,
        reference: ManifestRef,
        cancel: &CancellationToken,
    ) -> Result<Arc<Snapshot>, CatalogError> {
        Ok(self
            .prepare_pinned_manifest(reference, cancel)?
            .execute(cancel)
            .await?
            .into_value())
    }

    async fn read_pinned_manifest_inner(
        &self,
        reference: ManifestRef,
        cancel: &CancellationToken,
    ) -> Result<Arc<Snapshot>, CatalogError> {
        let mut graph = Graph {
            pending: Vec::new(),
            active: BTreeSet::new(),
            admitted: BTreeMap::new(),
            reservation: self.reserver.open("store:pinned-parent-graph"),
            traversal: Arc::default(),
        };
        graph.push(Step::Enter(reference))?;
        while let Some(step) = graph.pending.pop() {
            cancel.checkpoint()?;
            match step {
                Step::Enter(reference) => {
                    if !graph.enter(reference)? {
                        continue;
                    }
                    let manifest = self.read_manifest_envelope(reference, cancel).await?;
                    let binding = self.read_admission_binding(&manifest, cancel).await?;
                    graph.push(Step::Admit(reference, manifest, binding.clone()))?;
                    if let Some(invocation) = &binding.invocation {
                        if let Some(source) = invocation.document_source {
                            graph.push(Step::Enter(source))?;
                        }
                        for policy in invocation.policies.values().rev() {
                            graph.push(Step::Enter(policy.snapshot))?;
                        }
                    }
                    for parent in binding.parents.values().rev() {
                        graph.push(Step::Enter(*parent))?;
                    }
                }
                Step::Admit(reference, manifest, binding) => {
                    let context = graph.context(&binding, self)?;
                    let snapshot = self
                        .admit_manifest(reference, manifest, &context, cancel)
                        .await?;
                    graph.active.remove(&key(reference));
                    graph.admitted.insert(key(reference), snapshot);
                }
            }
        }
        graph
            .admitted
            .remove(&key(reference))
            .ok_or_else(|| admission("admission binding", "target was not admitted"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_ids::{EncodingChecksum, FixedBudget, MemoryReserver, SnapshotId};

    #[test]
    fn repeated_active_physical_reference_is_a_cycle_and_releases_graph_budget() {
        let budget = FixedBudget::new(4096);
        let mut graph = Graph {
            pending: Vec::new(),
            active: BTreeSet::new(),
            admitted: BTreeMap::new(),
            reservation: budget.open("fixture:parent-graph"),
            traversal: Arc::default(),
        };
        let reference = ManifestRef {
            snapshot_id: SnapshotId(ContentHash::NIL),
            manifest_checksum: EncodingChecksum(ContentHash::NIL),
        };
        graph.push(Step::Enter(reference)).expect("bounded stack");
        assert!(graph.enter(reference).expect("first entry"));
        assert!(
            matches!(graph.enter(reference), Err(CatalogError::Admission { reason, .. }) if reason.contains("cycle"))
        );
        drop(graph);
        assert_eq!(budget.reserved(), 0);
    }
}
