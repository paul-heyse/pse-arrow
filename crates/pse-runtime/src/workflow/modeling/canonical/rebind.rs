// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Receiver admission of retained source, without treating transport as validation.
use super::*;
use pse_relations::generated::authored::{documents, fit_cases, modeling_declarations, packages};

pub(super) fn rebind(
    source: &ModelingPackage,
    runtime: &Runtime,
) -> Result<ModelingPackage, WorkflowError> {
    let context = runtime.admission_context();
    if !Arc::ptr_eq(&source.runtime.shared, &runtime.shared)
        || !source
            .revision
            .admitted
            .validation_context()
            .same_bindings(&context)
        || !Arc::ptr_eq(source.runtime.registry(), runtime.registry())
    {
        return Err(contract(
            "retained replay receiver requires the same actual deployment, registry and native validation bindings",
        ));
    }
    let receiver_extent = source
        .physical
        .metadata_bytes()
        .checked_add(4096 + size_of::<ModelingPackage>() + size_of::<SourceRevision>())
        .ok_or_else(|| contract("retained receiver metadata extent"))?;
    let receiver_metadata = runtime
        .shared
        .math()
        .reserve("modeling:receiver-source-view", receiver_extent)?;
    let cancel = crate::CancelSource::new();
    // Source objects retain immutable parser/reference validity, but not every successful
    // generated row product. Admit those owned rows directly in this receiver scope.
    admit_rows(source, runtime, &context, &cancel)?;
    let physical = source
        .physical
        .readmit_context(runtime, &context, &cancel.token())?;
    let admitted = source
        .revision
        .admitted
        .rebind_for(runtime, context, receiver_metadata)?;
    let revision = SourceRevision::admitted(admitted, source.revision.identity(), None, None)?;
    // Preserve original executable and physical owners; do not borrow mutable engagement.
    Ok(ModelingPackage {
        runtime: runtime.clone(),
        revision,
        held_preparation: None,
        replay_owner: None,
        quantities: source.quantities.clone(),
        physical,
        providers: source.providers.clone(),
        accelerators: source.accelerators.clone(),
    })
}

pub(super) fn admit_rows(
    source: &ModelingPackage,
    runtime: &Runtime,
    context: &pse_relations::validate::ValidationContext,
    cancel: &crate::CancelSource,
) -> Result<(), WorkflowError> {
    // Opaque data bytes are not copied or decoded. Their selected typed admission still
    // belongs to preparation. This envelope covers only typed row/check conversion.
    let extent = source
        .revision
        .admitted
        .objects
        .values()
        .try_fold(8192usize, |n, object| {
            let bytes = if let Some((row, _)) = &object.declaration {
                row.owned_bytes()
            } else if matches!(object.kind.as_str(), "package" | "document" | "fit") {
                object.payload.len()
            } else {
                0
            };
            n.checked_add(bytes.checked_mul(32)?.checked_add(1024)?)
        })
        .ok_or_else(|| contract("retained replay row admission extent overflow"))?;
    let _scratch = runtime
        .shared
        .math()
        .reserve("replay:receiver-source-rows", extent)?;
    let mut declarations =
        modeling_declarations::Builder::with_registry(runtime.registry(), 0, context)
            .map_err(super::super::super::relation)?;
    let mut package_rows = packages::Builder::with_registry(runtime.registry(), 0, context)
        .map_err(super::super::super::relation)?;
    let mut document_rows = documents::Builder::with_registry(runtime.registry(), 0, context)
        .map_err(super::super::super::relation)?;
    let mut fit_rows = fit_cases::Builder::with_registry(runtime.registry(), 0, context)
        .map_err(super::super::super::relation)?;
    for object in source.revision.admitted.objects.values() {
        checkpoint(cancel)?;
        if let Some((row, _)) = &object.declaration {
            declarations
                .push(row.clone())
                .map_err(super::super::super::relation)?;
            continue;
        }
        match object.kind.as_str() {
            "package" => package_rows
                .push(
                    object
                        .typed
                        .as_ref()
                        .and_then(packages::Row::carried)
                        .cloned()
                        .map(Ok)
                        .unwrap_or_else(|| decode(&object.payload))?,
                )
                .map_err(super::super::super::relation)?,
            "document" => document_rows
                .push(
                    object
                        .typed
                        .as_ref()
                        .and_then(documents::Row::carried)
                        .cloned()
                        .map(Ok)
                        .unwrap_or_else(|| decode(&object.payload))?,
                )
                .map_err(super::super::super::relation)?,
            "fit" => fit_rows
                .push(
                    object
                        .typed
                        .as_ref()
                        .and_then(fit_cases::Row::carried)
                        .cloned()
                        .map(Ok)
                        .unwrap_or_else(|| decode(&object.payload))?,
                )
                .map_err(super::super::super::relation)?,
            CONTEXT
            | "physical_scope"
            | "physical_scope_document"
            | "ingress_frame"
            | "record_supplier"
            | "data" => {}
            _ => {
                return Err(contract(
                    "retained replay source has unsupported validation coverage",
                ));
            }
        }
    }
    declarations
        .finish()
        .map_err(super::super::super::relation)?;
    package_rows
        .finish()
        .map_err(super::super::super::relation)?;
    document_rows
        .finish()
        .map_err(super::super::super::relation)?;
    fit_rows.finish().map_err(super::super::super::relation)?;
    checkpoint(cancel)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows() -> Vec<Declaration> {
        pse_authoring::language::parse(
            "package p {param a:Scalar=1;def Root {var x:Scalar;eq e:x==a;}}",
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap()
    }
    fn bind(source: &ModelingPackage, runtime: &Runtime) -> Result<ModelingPackage, WorkflowError> {
        let activity = runtime.admit_activity()?;
        activity.sync_scope(|| rebind(source, runtime))
    }
    #[tokio::test]
    async fn retained_receiver_shares_exact_source_and_parser_but_has_an_independent_head() {
        let runtime = crate::workflow::tests::runtime();
        let source = runtime
            .modeling_package(rows(), crate::workflow::tests::physical())
            .await
            .unwrap();
        let receiver =
            Runtime::from_shared_local(runtime.shared.clone(), runtime.model.clone()).unwrap();
        runtime.close();
        let rebound = bind(&source, &receiver).unwrap();
        assert_ne!(source.revision_id(), rebound.revision_id());
        assert_ne!(source.revision.lineage_id(), rebound.revision.lineage_id());
        assert!(rebound.canonical_revision().is_none());
        assert!(rebound.held_preparation.is_none());
        assert!(Arc::ptr_eq(&source.providers, &rebound.providers));
        assert!(source.physical.same_sources(&rebound.physical));
        for (logical, original) in source.revision.admitted.objects.iter() {
            let current = &rebound.revision.admitted.objects[logical];
            assert!(Arc::ptr_eq(original, current));
            if let Some(parsed) = &original.parsed {
                assert!(Arc::ptr_eq(parsed, current.parsed.as_ref().unwrap()));
            }
        }
        let original_head = source.revision.head.lock().unwrap().revision.clone();
        let changed = pse_authoring::language::parse(
            "package p {param a:Scalar=2;def Root {var x:Scalar;eq e:x==a;}}",
            SemanticId::NIL,
            pse_authoring::language::IdentityPolicy::Named,
            Default::default(),
        )
        .unwrap();
        let next = rebound.with_declarations(changed).await.unwrap();
        assert_eq!(source.revision.head.lock().unwrap().revision, original_head);
        assert_ne!(next.revision.identity(), source.revision.identity());
        assert_eq!(next.revision.lineage_id(), rebound.revision.lineage_id());
        receiver.close();
        assert!(matches!(
            bind(&source, &receiver),
            Err(WorkflowError::CallerClosed)
        ));
    }
    #[tokio::test]
    async fn retained_receiver_readmits_physical_scope_checks_in_the_actual_receiving_invocation() {
        use datafusion::arrow::{
            array::{Int64Array, RecordBatch},
            datatypes::DataType,
        };
        use pse_schema::model::{Authority, FieldContract, Namespace, RelationDecl, SnapshotClass};
        let original = crate::workflow::tests::runtime();
        let mut builder = pse_schema::RegistryBuilder::new();
        pse_schema::catalog::declare(&mut builder);
        builder.declare_relation(
            RelationDecl::new(
                Namespace::Reference,
                "rebind_clock",
                1,
                Authority::Reference,
                SnapshotClass::Model,
                "receiver scope control",
            )
            .pk(&["n"])
            .columns(vec![FieldContract::payload(
                "n",
                FieldContract::native(DataType::Int64),
                "time",
            )])
            .checks(BTreeMap::from([(
                "fresh".into(),
                "to_timestamp_seconds(n) <= now()".into(),
            )])),
        );
        let registry = Arc::new(builder.build().unwrap());
        let model = original
            .model
            .factory()
            .bind_model(registry.clone())
            .unwrap();
        let runtime = Runtime::from_shared_local(original.shared.clone(), model).unwrap();
        let spec = registry.relation("reference.rebind_clock").unwrap();
        let actual = runtime.admission_context();
        let now = actual
            .admission_scope()
            .execution_props()
            .query_execution_start_time
            .unwrap();
        let mut future_properties = actual.admission_scope().execution_props().clone();
        future_properties.query_execution_start_time =
            Some(now + std::time::Duration::from_secs(60));
        let future = actual.for_scope(
            pse_relations::validate::AdmissionScope::from_execution_props(&future_properties),
        );
        let make_physical = |timestamp| {
            let raw = RecordBatch::try_new(
                pse_schema::arrow::relation_schema_ref(&registry, spec).unwrap(),
                vec![Arc::new(Int64Array::from(vec![timestamp]))],
            )
            .unwrap();
            let checked = pse_relations::columnar::FieldCheckedBatch::admit(
                &registry,
                spec,
                raw,
                &future,
                &crate::CancelSource::new().token(),
            )
            .unwrap();
            let mut physical = crate::workflow::tests::physical();
            physical.sources.insert(spec.key, checked);
            physical
        };
        let past_source = runtime
            .modeling_package(rows(), make_physical(0))
            .await
            .unwrap();
        let activity = runtime.admit_activity().unwrap();
        activity.sync_scope(|| {
            let receiving_context = runtime.admission_context();
            let rebound = rebind(&past_source, &runtime).unwrap();
            assert!(
                rebound.physical.sources[&spec.key]
                    .local_admission()
                    .unwrap()
                    .matches(&registry, spec, &receiving_context)
                    .unwrap()
            );
            assert!(
                past_source.physical.sources[&spec.key]
                    .same_source(&rebound.physical.sources[&spec.key])
            );
        });
        let future_source = runtime
            .modeling_package(
                rows(),
                make_physical((now + std::time::Duration::from_secs(30)).timestamp()),
            )
            .await
            .unwrap();
        let root = rows()
            .iter()
            .find(|row| row.name == "Root")
            .unwrap()
            .declaration_id;
        runtime
            .scope_activity(past_source.checked_selection(&[root], &crate::CancelSource::new()))
            .await
            .unwrap();
        assert!(
            runtime
                .shared
                .math()
                .modeling_cache
                .selected(&future_source.selection_request(&[root]))
                .is_some(),
            "the negative control reaches a populated warm candidate"
        );
        assert!(
            runtime
                .scope_activity(
                    future_source.checked_selection(&[root], &crate::CancelSource::new())
                )
                .await
                .is_err(),
            "warm selection must renew physical scope checks before carrying eligibility"
        );
        assert!(
            bind(&future_source, &runtime).is_err(),
            "a successful future-clock source admission cannot authorize the receiving current scope"
        );
        assert!(
            future_source
                .replay_source()
                .unwrap()
                .reconstruct(&runtime, future_source.physical.clone(), BTreeMap::new())
                .is_err(),
            "supplied transport also renews physical evidence under its actual receiving scope"
        );
    }
    #[tokio::test]
    async fn retained_receiver_refuses_foreign_bindings_and_unknown_source_coverage() {
        let runtime = crate::workflow::tests::runtime();
        let source = runtime
            .modeling_package(rows(), crate::workflow::tests::physical())
            .await
            .unwrap();
        let foreign = crate::workflow::tests::runtime();
        assert!(bind(&source, &foreign).is_err());
        let unrecognized = edit(
            "opaque:foreign".into(),
            "opaque".into(),
            "foreign".into(),
            "unknown".into(),
            b"{}".to_vec(),
            Vec::new(),
        );
        let admitted = AdmittedModelRevision::apply(
            &runtime,
            Some(&source.revision.admitted),
            &[unrecognized],
            None,
        )
        .unwrap();
        let revision =
            SourceRevision::admitted(admitted, source.revision.identity(), None, None).unwrap();
        let unsupported = ModelingPackage {
            revision,
            ..source.clone()
        };
        assert!(
            bind(&unsupported, &runtime)
                .unwrap_err()
                .to_string()
                .contains("unsupported validation coverage")
        );
        assert!(
            !source
                .revision
                .admitted
                .objects
                .contains_key("opaque:foreign")
        );
    }
}
