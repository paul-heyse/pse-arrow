// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! File ownership of one source-derived engineering descriptor graph.

use anyhow::{Context, Result, ensure};
use buffa_codegen::generated::descriptor::{
    DescriptorProto, FileDescriptorProto, FileDescriptorSet, ServiceDescriptorProto,
};
use pse_runtime::host_operations::engineering_contract::{
    EXPANDED_STUDY_SERVICE, EXPLICIT_STUDY_SERVICE, STUDY_CONTROL_SERVICE,
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) const MAIN: &str = "engineering.proto";
pub(super) const SHARED: &str = "engineering_shared.proto";
pub(super) const EXPLICIT: &str = "engineering_explicit_study.proto";
pub(super) const CONTROL: &str = "engineering_study_control.proto";
pub(super) const EXPANDED: &str = "engineering_expanded_study.proto";
pub(super) const INSPECTION: &str = "engineering_inspection.proto";
pub(super) const PACKAGE: &str = "pse.engineering.v1";

fn top_level(name: &str) -> Option<&str> {
    name.strip_prefix(".pse.engineering.v1.")?.split('.').next()
}

fn message_dependencies<'a>(message: &'a DescriptorProto, references: &mut Vec<&'a str>) {
    references.extend(
        message
            .field
            .iter()
            .filter_map(|field| field.type_name.as_deref().and_then(top_level)),
    );
    for nested in &message.nested_type {
        message_dependencies(nested, references);
    }
}

fn service_dependencies(service: &ServiceDescriptorProto) -> impl Iterator<Item = &str> {
    service.method.iter().flat_map(|method| {
        [method.input_type.as_deref(), method.output_type.as_deref()]
            .into_iter()
            .flatten()
            .filter_map(top_level)
    })
}

fn reachable<'a>(
    messages: &BTreeMap<&'a str, &'a DescriptorProto>,
    roots: impl IntoIterator<Item = &'a str>,
) -> BTreeSet<&'a str> {
    let mut seen = BTreeSet::new();
    let mut pending = roots.into_iter().collect::<Vec<_>>();
    while let Some(name) = pending.pop() {
        if let Some(message) = messages.get(name)
            && seen.insert(name)
        {
            message_dependencies(message, &mut pending);
        }
    }
    seen
}

/// Move whole declarations without regenerating names, options or nested definitions.
pub(super) fn partition(mut source: FileDescriptorSet) -> Result<FileDescriptorSet> {
    let index = source
        .file
        .iter()
        .position(|file| file.name.as_deref() == Some(MAIN))
        .context("source engineering descriptor missing")?;
    let original = source.file.remove(index);
    let messages = original
        .message_type
        .iter()
        .filter_map(|message| Some((message.name.as_deref()?, message)))
        .collect::<BTreeMap<_, _>>();
    let services = original
        .service
        .iter()
        .filter_map(|service| Some((service.name.as_deref()?, service)))
        .collect::<BTreeMap<_, _>>();
    let explicit = *services
        .get(EXPLICIT_STUDY_SERVICE)
        .context("explicit Study service missing")?;
    let control = *services
        .get(STUDY_CONTROL_SERVICE)
        .context("Study control service missing")?;
    ensure!(
        services.contains_key(EXPANDED_STUDY_SERVICE),
        "expanded Study service missing"
    );
    let ordinary = reachable(
        &messages,
        service_dependencies(explicit).chain(service_dependencies(control)),
    );
    ensure!(
        !ordinary.contains("SourceDelta"),
        "explicit Study services depend on inline authoring"
    );
    let inspection = reachable(&messages, ["ModelingInspection"]);
    ensure!(
        inspection.contains("ModelingInspection"),
        "inspection observation missing"
    );
    ensure!(
        !ordinary.contains("ModelingInspection"),
        "explicit Study services depend on model inspection"
    );
    let shared = ordinary
        .intersection(&inspection)
        .copied()
        .collect::<BTreeSet<_>>();
    ensure!(
        !shared.is_empty(),
        "engineering selection lacks shared source owners"
    );

    let mut owner = BTreeMap::new();
    let new_file = |name: &str| FileDescriptorProto {
        name: Some(name.into()),
        package: original.package.clone(),
        syntax: original.syntax.clone(),
        dependency: original.dependency.clone(),
        ..Default::default()
    };
    let mut files = BTreeMap::from([
        (SHARED, new_file(SHARED)),
        (EXPLICIT, new_file(EXPLICIT)),
        (CONTROL, new_file(CONTROL)),
        (EXPANDED, new_file(EXPANDED)),
        (INSPECTION, new_file(INSPECTION)),
        (MAIN, new_file(MAIN)),
    ]);
    for (name, message) in messages {
        let file = if shared.contains(name) {
            SHARED
        } else if ordinary.contains(name) {
            EXPLICIT
        } else if inspection.contains(name) {
            INSPECTION
        } else {
            MAIN
        };
        owner.insert(name, file);
        files
            .get_mut(file)
            .context("selected message file")?
            .message_type
            .push(message.clone());
    }
    for (name, service) in services {
        let file = match name {
            EXPLICIT_STUDY_SERVICE => EXPLICIT,
            STUDY_CONTROL_SERVICE => CONTROL,
            EXPANDED_STUDY_SERVICE => EXPANDED,
            _ => MAIN,
        };
        files
            .get_mut(file)
            .context("selected service file")?
            .service
            .push(service.clone());
    }
    for (name, file) in &mut files {
        let mut dependencies = Vec::new();
        for message in &file.message_type {
            message_dependencies(message, &mut dependencies);
        }
        dependencies.extend(file.service.iter().flat_map(service_dependencies));
        file.dependency.extend(
            dependencies
                .into_iter()
                .filter_map(|reference| owner.get(reference).copied())
                .filter(|dependency| dependency != name)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .map(str::to_owned),
        );
    }
    // The full world also publishes the independent recorded-result identity.
    files
        .get_mut(MAIN)
        .context("main engineering file")?
        .dependency
        .push(super::super::recorded_result::FILE.into());
    for name in [SHARED, EXPLICIT, CONTROL, INSPECTION, MAIN, EXPANDED] {
        let file = files
            .remove(name)
            .context("partitioned engineering file missing")?;
        for dependency in &file.dependency {
            ensure!(
                source
                    .file
                    .iter()
                    .any(|file| file.name.as_ref() == Some(dependency)),
                "{name} has an unavailable or later file dependency {dependency}"
            );
        }
        source.file.push(file);
    }
    Ok(source)
}

/// Retain only files actually reached by an independent reflection world.
pub(super) fn world(source: &FileDescriptorSet, targets: &[&str]) -> Result<FileDescriptorSet> {
    let files = source
        .file
        .iter()
        .filter_map(|file| Some((file.name.as_deref()?, file)))
        .collect::<BTreeMap<_, _>>();
    let mut pending = targets.to_vec();
    let mut seen = BTreeSet::new();
    while let Some(name) = pending.pop() {
        if seen.insert(name) {
            let file = files
                .get(name)
                .with_context(|| format!("missing descriptor import {name}"))?;
            pending.extend(file.dependency.iter().map(String::as_str));
        }
    }
    Ok(FileDescriptorSet {
        file: source
            .file
            .iter()
            .filter(|file| file.name.as_deref().is_some_and(|name| seen.contains(name)))
            .cloned()
            .collect(),
        ..Default::default()
    })
}
