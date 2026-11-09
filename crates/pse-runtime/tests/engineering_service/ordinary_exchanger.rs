// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! A connected client of the exact ordinary exchanger source closure.

use super::{endpoint, pb, selected};
use pse_admission::authoring_driver::document::capture_package;
use pse_ids::SemanticId;
use pse_runtime::{
    engineering_projection as codec,
    host_operations::engineering::EngineeringClient,
    workflow::{ConformanceManifest, OriginalSourceFileRequest},
};
use std::{collections::BTreeSet, path::Path};

const CASE: &str = "ordinary_exchanger";
const SOURCE_PATH: &str = "models/delivery.pse";

fn source(operation: String) -> (pb::EngineeringAdmitSource, SemanticId, Vec<u8>) {
    let reference = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/reference")
        .canonicalize()
        .unwrap();
    let manifest = ConformanceManifest::load(&reference.join("conformance.toml")).unwrap();
    let seed = manifest
        .runs
        .into_iter()
        .find(|run| run.name == "seed")
        .unwrap();
    let registry = pse_schema::shared_registry().unwrap();
    let capture = |name: &str| {
        let directory = reference.join(name);
        assert_eq!(directory.canonicalize().unwrap(), directory);
        capture_package(&directory, &registry, Default::default()).unwrap()
    };
    let delivery = capture("ordinary-delivery");
    let package = delivery.header.package_id.as_id();
    let exact_source = delivery.files[SOURCE_PATH].clone();
    let wrap = |files: std::collections::BTreeMap<String, Vec<u8>>| pb::EngineeringPackage {
        files: files.into_iter().collect(),
        ..Default::default()
    };
    let physical = wrap(capture(&seed.physical).files);
    let mut modeling = vec![wrap(delivery.files)];
    let mut admitted_paths = BTreeSet::new();
    // Select the same package closure as the native reference. The physical
    // package is also the separately admitted physical context.
    for name in seed
        .dependencies
        .into_iter()
        .chain([seed.package, "data/ordinary-bt".into()])
    {
        if admitted_paths.insert(name.clone()) {
            modeling.push(if name == seed.physical {
                physical.clone()
            } else {
                wrap(capture(&name).files)
            });
        }
    }
    assert!(
        modeling.len() > 16,
        "the actual original closure exceeds the former wire shape"
    );
    (
        pb::EngineeringAdmitSource {
            operation,
            physical: vec![physical],
            modeling,
            ..Default::default()
        },
        package,
        exact_source,
    )
}

#[tokio::test(flavor = "multi_thread")]
async fn prepared_engineering_ordinary_exchanger_reopens_original_source() {
    let endpoint = endpoint();
    let mut client = EngineeringClient::connect(endpoint.clone()).await.unwrap();
    let id: SemanticId = pse_operations::mint_id();
    let source_operation = format!("ordinary-exchanger-{}-source", id.to_hex());
    let (request, package, exact_source) = source(source_operation.clone());
    client.generated().admit_source(request).await.unwrap();
    let original = selected(&mut client, &source_operation).await;
    assert_eq!(original.operation, source_operation);
    let mut offset = 0;
    loop {
        let page = client
            .generated()
            .inspect_source(pb::EngineeringInspectSource {
                operation: source_operation.clone(),
                offset,
                limit: 128,
                ..Default::default()
            })
            .await
            .unwrap()
            .into_owned();
        assert_eq!(page.source.as_option(), Some(&original));
        if let Some(row) = page.declarations.iter().find(|row| row.name == CASE) {
            let declaration =
                SemanticId::from_bytes(row.declaration.as_slice().try_into().unwrap());
            assert_ne!(declaration, SemanticId::NIL);
            break;
        }
        offset = page
            .next_offset
            .expect("original exchanger declaration absent");
    }

    drop(client);
    let client = EngineeringClient::connect(endpoint).await.unwrap();
    let file = client
        .read_source_file(pb::EngineeringReadSourceFile {
            source_operation,
            request: codec::encode_original_source_file_request(OriginalSourceFileRequest {
                parent: original.local_revision.clone(),
                package,
                path: SOURCE_PATH.into(),
                maximum_bytes: 1 << 20,
            })
            .unwrap()
            .into(),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_owned();
    let file = codec::decode_original_source_file(file).unwrap();
    assert_eq!(file.local_revision, original.local_revision);
    assert_eq!(file.body, exact_source);
    assert_eq!(
        file.original_hash,
        pse_ids::encoding_checksum(&exact_source).content_hash()
    );
}
