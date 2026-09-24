// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Pure native plan construction; no storage writes or compiler journeys.
use super::*;
use pse_ids::ContentHash;
use pse_relations::generated::{
    authored::entities,
    enums::{ArtifactReconstruction, PublicationKind},
};
#[test]
fn a_required_empty_member_is_selected_but_an_absent_member_refuses() {
    let registry = pse_schema::shared_registry().unwrap();
    let factory = pse_testkit::NativeFixture::new((32 << 20).try_into().unwrap())
        .unwrap()
        .into_factory();
    let cancel = CancellationToken::new();
    let required =
        pse_schema::product::support_closure(&registry, &BTreeSet::from([entities::RELATION_ID]))
            .unwrap();
    let rows = required
        .iter()
        .map(|id| {
            let spec = registry.relation_by_id(*id).unwrap();
            let schema = Arc::new(pse_schema::arrow::relation_schema(&registry, spec).unwrap());
            (
                spec.key,
                datafusion::arrow::array::RecordBatch::new_empty(schema),
            )
        })
        .collect();
    let session = factory.candidate(rows, registry.clone(), &cancel).unwrap();
    let outputs: BTreeMap<_, _> = required
        .iter()
        .map(|id| {
            let spec = registry.relation_by_id(*id).unwrap();
            let source = session.table_reference(&spec.key).unwrap().resolve("", "");
            let plan = session.relation_plan(&source).unwrap();
            (
                source,
                RelationOutput {
                    relation_id: *id,
                    plan: plan.plan().clone(),
                },
            )
        })
        .collect();
    let mut missing_outputs = outputs.clone();
    missing_outputs.retain(|_, output| output.relation_id != entities::RELATION_ID);
    let hash = ContentHash::from_bytes([1; 32]);
    let descriptor = ArtifactDescriptor::create(wire::Row {
        artifact_id: hash,
        descriptor_version: 1,
        profile: PublicationKind::Inspection,
        profile_contract: registry.fingerprint(),
        requested_relations: vec![entities::RELATION_ID],
        release_id: hash,
        release_members: vec![],
        semantic_identity: hash,
        implementation: wire::RuntimeArtifactDescriptorsFieldImplementation {
            source: hash,
            build: hash,
            registry: registry.fingerprint(),
            algorithms: hash,
        },
        target_contract: hash,
        value_assumptions: vec![],
        reconstruction: ArtifactReconstruction::None,
    })
    .unwrap();
    let empty = ArtifactPlan::new(session.clone(), outputs, &cancel)
        .unwrap()
        .with_product(descriptor.clone(), &cancel)
        .unwrap();
    assert_eq!(empty.outputs().len(), required.len() + 1);
    let descriptor_output = empty
        .outputs()
        .values()
        .find(|output| output.relation_id == wire::RELATION_ID)
        .unwrap();
    let declaration =
        pse_schema::arrow::relation_schema(&registry, wire::spec(&registry).unwrap()).unwrap();
    assert_eq!(descriptor_output.plan.schema().as_arrow(), &declaration);
    crate::delta::layout::DurableLayout::new(Arc::new(declaration))
        .unwrap()
        .encode(descriptor_output.plan.clone())
        .unwrap();
    assert!(
        ArtifactPlan::new(session, missing_outputs, &cancel)
            .unwrap()
            .with_product(descriptor, &cancel)
            .is_err()
    );
}
