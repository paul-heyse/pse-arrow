// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Public research discovery and frozen selection through original native source owners.
//! Requires an explicitly prepared private canonical service, never configured user data.
#![cfg(feature = "native-study-tests")]
#![recursion_limit = "512"]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stderr,
    reason = "isolated integration fixtures report original assertion and cleanup failures"
)]

#[path = "reference_support/mod.rs"]
mod reference_support;

use futures_util::{FutureExt, future::BoxFuture};
use pse_ids::{Frame, FramedHasher, SemanticId};
use pse_model::generated::identities::DeclarationId;
use pse_operations::{
    canonical::CanonicalStore,
    native_service::{NativeCanonicalService, NativeModuleSource, NativeServiceSpec},
};
use pse_runtime::{
    CancelSource,
    workflow::{
        CanonicalDeployment, FitDeclarations, ModelingPackage, OuterAttestation,
        ResearchDataObject, ResearchDatasetRequest, ResearchObservation, ResearchReadRequest,
        ResearchReadResult, ResearchSelectionRequest, Runtime,
    },
};
use reference_support::{NativeContext, Result};
use std::{
    collections::BTreeMap,
    panic::AssertUnwindSafe,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

fn id(byte: u8) -> SemanticId {
    SemanticId::from_bytes([byte; 16])
}

fn service() -> Result<NativeCanonicalService> {
    let environment = std::env::vars_os().collect();
    let helper = PathBuf::from(
        std::env::var_os("PSE_NATIVE_FIXTURE_SERVICE_HELPER")
            .expect("explicit already-built native-service helper"),
    );
    assert!(helper.is_absolute() && helper.is_file());
    let receipt = PathBuf::from(
        std::env::var_os("PSE_NATIVE_FIXTURE_MODULE_RECEIPT")
            .expect("explicit prepared private module receipt"),
    );
    Ok(NativeCanonicalService::new(NativeServiceSpec {
        server: pse_host::server::select(&environment)?,
        helper,
        module_source: NativeModuleSource::Prepared(pse_host::study_module::select(
            &receipt,
            pse_host::study_module::Variant::Production,
        )?),
        state_parent: std::env::temp_dir(),
    })?)
}

fn attach(context: &mut NativeContext, store: &CanonicalStore) -> Result<()> {
    context.runtime.close();
    context.runtime.clear_program_cache();
    let factory = Arc::new(
        context
            .shared
            .session_factory(pse_engine::session::native_engine_profile())?,
    );
    let mut executable = std::fs::File::open(std::env::current_exe()?)?;
    let mut hasher = FramedHasher::new(Frame::CanonicalPayloadV1);
    hasher.part_reader(executable.metadata()?.len(), &mut executable, &|| Ok(()))?;
    context.runtime = Runtime::from_shared(
        context.shared.clone(),
        factory.bind_model(pse_schema::shared_registry()?)?,
        CanonicalDeployment::new(
            store.clone(),
            OuterAttestation {
                source: None,
                build: hasher.finish_hash(),
            },
            None,
        ),
    )?;
    Ok(())
}

fn measurement(context: &NativeContext) -> BoxFuture<'_, Result<ModelingPackage>> {
    Box::pin(async move {
        let package = context.package(
            "package fitting { def Measurement { param length:Length=2{m};
            annotation check length(length>0{m}); }
            entity kind origin provenance {attribute title:Text;}
            entity origin experiment {title=\"known length\"}
            enum role {measured facets(measured)}
            entity kind reading {attribute value:Length?;attribute sigma:Length?;}
            @id(\"9d9d9d9d9d9d9d9d9d9d9d9d9d9d9d9d\")
            entity reading measured provenance(experiment,role.measured) {value=2{m},sigma=0.1{m}} }"
        ).await?;
        let case = package
            .declarations(&context.runtime.preparation_services())
            .await?
            .iter()
            .find(|row| row.name == "Measurement")
            .unwrap()
            .declaration_id;
        let mut declarations = FitDeclarations::default();
        declarations.fits.push(serde_json::from_value(serde_json::json!({
            "fit_id": id(158),
            "parameters": [{"symbol_id": id(153), "fixed": true, "value": 2., "lower": 0., "upper": 10., "scale": 1.}],
            "experiments": [{"experiment_id": id(159), "case_id": case, "route": "steady", "bindings": [{"parameter_id": id(153), "path": "length"}]}],
            "observations": [{"observation_id": id(157), "value_attribute": "value", "standard_deviation_attribute": "sigma",
                "experiment_id": id(159), "output_path": "length", "time": null, "time_basis": null, "time_unit_id": null,
                "included": true, "importance": 1.}]
        }))?);
        Ok(Box::pin(
            package.with_fit_declarations(&context.runtime.source_services(), declarations),
        )
        .await?)
    })
}

fn finish<T>(outcome: std::thread::Result<Result<T>>, cleanup: Result<()>) -> Result<T> {
    match (outcome, cleanup) {
        (Err(panic), cleanup) => {
            if let Err(error) = cleanup {
                eprintln!(
                    "native research cleanup failed while preserving original panic: {error}"
                );
            }
            std::panic::resume_unwind(panic)
        }
        (Ok(Ok(value)), Ok(())) => Ok(value),
        (Ok(Err(error)), Ok(())) | (Ok(Ok(_)), Err(error)) => Err(error),
        (Ok(Err(error)), Err(cleanup)) => Err(std::io::Error::other(format!(
            "native research failed: {error}; independent cleanup failed: {cleanup}"
        ))
        .into()),
    }
}

fn journey<'a>(context: &'a NativeContext, store: &'a CanonicalStore) -> BoxFuture<'a, Result<()>> {
    Box::pin(async move {
        let runtime = &context.runtime;
        let cancel = CancelSource::new();
        let initial = measurement(context).await?;
        store
            .create_project("create-research", "research", "Original typed measurements")
            .await?;
        let project = store.project("research").await?.unwrap();
        let revision = Box::pin(runtime.project_fork(
            &project,
            "research-main",
            "research-source",
            "fork-research",
            &initial,
        ))
        .await?;
        let revision_key = revision.key.clone();
        let source = Box::pin(runtime.modeling_revision(
            revision,
            initial.physical_context().clone(),
            BTreeMap::new(),
        ))
        .await?;
        let project = store.project("research").await?.unwrap();
        Box::pin(runtime.project_record_context(
            "research-context",
            &project,
            "measurements",
            &source,
            0,
            None,
            None,
            false,
        ))
        .await?;
        let cut = store.project_context_cut("research").await?;
        let selected = store
            .select_project_context(&cut, "measurements", 1)
            .await?
            .unwrap();
        let case = source
            .declarations(&runtime.preparation_services())
            .await?
            .iter()
            .find(|row| row.name == "Measurement")
            .unwrap()
            .declaration_id;
        let quantity = |name| {
            source
                .physical_context()
                .quantities()
                .quantity_types()
                .find(|quantity| quantity.name.as_deref() == Some(name))
                .unwrap()
                .id
                .as_id()
        };
        let request = ResearchDatasetRequest {
            key: "measurements-v1".into(),
            project: project.clone(),
            context: selected.clone(),
            parent: None,
            role: "raw".into(),
            members: vec![ResearchDataObject {
                root: case,
                record: DeclarationId::from(id(157)),
                attribute: "value".into(),
                standard_deviation_attribute: Some("sigma".into()),
                quantity: quantity("Length"),
                acquisition_clock: "independently authored known-length fixture".into(),
                parent_member: None,
                calibration: vec![],
            }],
        };
        let created = Box::pin(runtime.research_dataset_create(&request, &cancel)).await?;
        assert_eq!(
            Box::pin(runtime.research_dataset_create(&request, &cancel)).await?,
            created
        );
        let ResearchReadResult::Datasets { rows } = runtime
            .research_read(&ResearchReadRequest::Datasets {
                project: "research".into(),
                after: String::new(),
                limit: 1,
            })
            .await?
        else {
            panic!("bounded native dataset discovery")
        };
        assert_eq!(rows.as_slice(), std::slice::from_ref(&created));
        let opened =
            Box::pin(runtime.research_dataset_open(&created.key, &selected, &cancel)).await?;
        assert_eq!(opened.canonical_revision().unwrap().key, revision_key);
        // Numeric payloads remain with the protected admitted source rather than
        // being copied into a dataset catalog or inferred from a string label.
        let mut wrong = request.clone();
        wrong.key = "wrong-quantity".into();
        wrong.members[0].quantity = quantity("Time");
        assert!(
            Box::pin(runtime.research_dataset_create(&wrong, &cancel))
                .await
                .is_err()
        );

        let selection = ResearchSelectionRequest {
            key: "observations-v1".into(),
            project: project.clone(),
            context: selected.clone(),
            dataset: created.key.clone(),
            fit: id(158).into(),
            observations: vec![ResearchObservation {
                observation: id(157).into(),
                member: 0,
                reason: "included known length and authored standard uncertainty".into(),
            }],
        };
        let frozen = Box::pin(runtime.research_selection_create(&selection, &cancel)).await?;
        assert_eq!(frozen.dataset, created.key);
        let (opened, fit) = Box::pin(runtime.research_selection_open(&frozen.key, &cancel)).await?;
        assert_eq!(fit, selection.fit);
        assert_eq!(opened.canonical_revision().unwrap().key, revision_key);
        let ResearchReadResult::Selection {
            row,
            selection: exact,
        } = runtime
            .research_read(&ResearchReadRequest::Selection {
                key: frozen.key.clone(),
            })
            .await?
        else {
            panic!("exact native frozen-selection detail")
        };
        assert_eq!(row.as_ref(), Some(&frozen));
        let exact = exact.expect("original complete observation selection");
        assert_eq!(
            serde_json::to_value(&*exact)?,
            serde_json::to_value(&selection)?
        );
        let ResearchReadResult::Dataset { row, members } = runtime
            .research_read(&ResearchReadRequest::Dataset {
                key: created.key.clone(),
            })
            .await?
        else {
            panic!("exact native dataset detail")
        };
        assert_eq!(row.as_ref(), Some(&created));
        assert_eq!(members.as_deref(), Some(request.members.as_slice()));

        Box::pin(runtime.project_record_context(
            "later-context",
            &project,
            "measurements",
            &source,
            0,
            None,
            Some("research-context"),
            false,
        ))
        .await?;
        let later_cut = store.project_context_cut("research").await?;
        let later = store
            .select_project_context(&later_cut, "measurements", 1)
            .await?
            .unwrap();
        assert_eq!(later.context.key, "later-context");
        assert_eq!(selected.context.key, "research-context");
        assert!(later_cut.sequence > cut.sequence);
        let (opened, fit) = Box::pin(runtime.research_selection_open(&frozen.key, &cancel)).await?;
        assert_eq!(fit, selection.fit);
        assert_eq!(opened.canonical_revision().unwrap().key, revision_key);
        assert_eq!(opened.project_context(), Some(&selected));
        Ok(())
    })
}

#[tokio::test]
async fn native_research_discovers_and_reopens_exact_selection_after_context_correction()
-> Result<()> {
    let mut service = service()?;
    let outcome = AssertUnwindSafe(service.run(
        Instant::now() + Duration::from_secs(300),
        |service| {
            Box::pin(async move {
                service
                    .launch(Instant::now() + Duration::from_secs(90))
                    .await?;
                let store = service.ready()?.store().clone();
                let mut context =
                    NativeContext::new().expect("explicit local scientific resources");
                let outcome = AssertUnwindSafe(async {
                    attach(&mut context, &store)?;
                    journey(&context, &store).await
                })
                .catch_unwind()
                .await;
                Ok(finish(outcome, context.close().await))
            })
        },
    ))
    .catch_unwind()
    .await;
    let disposal = service.dispose().map_err(Into::into);
    let outcome = outcome.map(|service_result| {
        service_result
            .map_err(Into::into)
            .and_then(|scientific| scientific)
    });
    finish(outcome, disposal)
}
