// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Selected dependency units reuse the same C07 intent and admission owner as execution.
use super::{HostError, OutputOptions};
use clap::Subcommand;
use schemars::JsonSchema;
use serde::Serialize;
use std::{ffi::OsString, path::Path};

#[derive(Debug, Subcommand)]
pub(super) enum Action {
    /// Explain dependency instances in actual admitted check contexts without compiling.
    Units {
        /// Exact locked dependency, distinct from the following selected workspace roots.
        #[arg(id = "dependency")]
        dependency: String,
        #[command(flatten)]
        selection: super::selection::SelectionArgs,
        /// Also inspect the already prepared native world; never prepares an analyzer.
        #[arg(long)]
        native_world: bool,
        /// Include original Cargo metadata JSON, preserving fields absent from the typed model.
        #[arg(long)]
        metadata_original: bool,
    },
}
#[derive(Debug, Serialize, JsonSchema)]
pub(super) struct UnitContext {
    /// Original selected/admitted compiler arguments, including feature absence policy outcomes.
    arguments: Vec<OsString>,
    /// Exact roots sharing this compatible context; incompatible roots remain separate.
    roots: Vec<crate::correctness::CompilerUnit>,
    /// Only source-qualified instances of the selected dependency; absence is not test coverage.
    dependency_units: Vec<crate::correctness::CompilerUnit>,
    /// Every unit examined in this original admitted context.
    examined_units: usize,
    /// Package/profile associations, retaining ambiguity and unproven compiler equivalence.
    native_joins: Vec<WorldJoin>,
}
#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(in crate::cli) enum JoinState {
    UniqueCandidate,
    Ambiguous,
    NoMatchingProfile,
    NativeWorldUnavailable,
}
#[derive(Debug, Serialize, JsonSchema)]
pub(in crate::cli) struct WorldJoin {
    /// Index in this context's dependency_units, not a cross-context unit identity.
    pub(in crate::cli) unit: usize,
    pub(in crate::cli) state: JoinState,
    /// Indexes in native_world.packages with matching observed package/profile dimensions.
    pub(in crate::cli) profiles: Vec<usize>,
    /// A loaded analyzer profile never establishes exact admitted rustc-unit equivalence.
    pub(in crate::cli) compiler_equivalence: &'static str,
}
#[derive(Debug, Serialize, JsonSchema)]
#[serde(tag = "availability", rename_all = "snake_case")]
enum NativeWorld {
    Available {
        observation: Box<crate::semantic::PackageContextAnswer>,
    },
    Unavailable {
        reason: String,
    },
    NotRequested,
}
fn join_units(units: &[crate::correctness::CompilerUnit], world: &NativeWorld) -> Vec<WorldJoin> {
    profile_joins(
        units,
        match world {
            NativeWorld::Available { observation } => Some(&observation.packages),
            _ => None,
        },
    )
}
/// Pure projection shared by live inspection and original retained-context comparison.
pub(in crate::cli) fn profile_joins(
    units: &[crate::correctness::CompilerUnit],
    profiles: Option<&[crate::semantic::NativePackageContext]>,
) -> Vec<WorldJoin> {
    units.iter().enumerate().map(|(unit, selected)| {
        let matching = match profiles {
            Some(packages) => packages.iter().enumerate().filter_map(|(index, native)| {
                let role = if selected.mode == "test" && selected.target_kinds.iter().any(|kind| kind == "lib") { "lib-test" }
                    else if selected.target_kinds.iter().any(|kind| kind == "proc-macro") { "proc-macro" }
                    else if selected.target_kinds.iter().any(|kind| kind == "lib") { "lib" } else { return None };
                (native.package_id == selected.package_id && native.library_root == Path::new(&selected.source_path)
                    && native.features == selected.features && native.target == selected.platform && native.role == role).then_some(index)
            }).collect::<Vec<_>>(), _ => Vec::new(),
        };
        let state = if profiles.is_none() { JoinState::NativeWorldUnavailable }
            else { match matching.len() { 0 => JoinState::NoMatchingProfile, 1 => JoinState::UniqueCandidate, _ => JoinState::Ambiguous } };
        WorldJoin { unit, state, profiles:matching, compiler_equivalence: "not_established; package/library/features/target/role association only" }
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn compiler_unit_world_join_preserves_host_test_and_profile_ambiguity() {
        let profile = json!({"package_id":"exact-source","manifest":"/workspace/Cargo.toml","source_root":"/workspace","workspace_root":"/workspace","library_root":"/workspace/src/lib.rs","features":["selected"],"target":"target-triple","role":"lib","edition":"2024","cfg":["feature=selected"],"source_hashes":{},"dependencies":[],"dependency_issues":[],"dependency_exclusions":[]});
        let world = |profiles| {
            NativeWorld::Available { observation: serde_json::from_value(json!({"session":{"backend":"native-snapshot","configuration":{"backend":"native_snapshot","project":"/workspace","features":[],"no_default_features":false,"target":"target-triple","build_macros":false},"position_encoding":"utf-8","backend_status":null,"preparation":"restricted"},"generation":1,"packages":profiles})).unwrap() }
        };
        let mut unit = crate::correctness::CompilerUnit {
            package_id: "exact-source".into(),
            package_name: "dep".into(),
            target_name: "dep".into(),
            target_kinds: vec!["lib".into()],
            source_path: "/workspace/src/lib.rs".into(),
            features: ["selected".into()].into(),
            platform: Some("target-triple".into()),
            mode: "check".into(),
            root: false,
        };
        let one = world(vec![profile.clone()]);
        assert!(matches!(
            join_units(&[unit.clone()], &one)[0].state,
            JoinState::UniqueCandidate
        ));
        let ambiguous = world(vec![profile.clone(), profile]);
        assert!(matches!(
            join_units(&[unit.clone()], &ambiguous)[0].state,
            JoinState::Ambiguous
        ));
        unit.mode = "test".into();
        assert!(matches!(
            join_units(&[unit.clone()], &one)[0].state,
            JoinState::NoMatchingProfile
        ));
        unit.mode = "build".into();
        unit.platform = None;
        assert!(matches!(
            join_units(&[unit.clone()], &one)[0].state,
            JoinState::NoMatchingProfile
        ));
        assert!(matches!(
            join_units(&[unit], &NativeWorld::NotRequested)[0].state,
            JoinState::NativeWorldUnavailable
        ));
    }
}
#[derive(Debug, Serialize, JsonSchema)]
pub(super) struct Units {
    /// Original exact metadata dependency facts.
    dependency: crate::dependency::Report,
    /// Original normalized C07 selection, preserving default/negative feature policies.
    selection: Option<crate::selection::Plan>,
    /// Original native compiler-unit contexts; none has executed.
    contexts: Option<Vec<UnitContext>>,
    /// Native source/cfg world, independently observed from Cargo's planned actions.
    native_world: NativeWorld,
    /// Original native response; decoding never reconstructs compatibility defaults here.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<serde_json::Value>")]
    original_metadata: Option<Box<serde_json::value::RawValue>>,
}
pub(super) fn dispatch(
    action: Action,
    selected: Option<&Path>,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    output.validate_for::<Units>("inspect dependency-units")?;
    let Action::Units {
        dependency,
        selection,
        native_world,
        metadata_original,
    } = action;
    let root = super::checkout_root(selected)?;
    let environment = std::env::vars_os().collect();
    let (snapshot, original) = crate::dependency::capture_original(&root, &environment)?;
    let original_metadata = metadata_original
        .then(|| serde_json::from_slice(&original))
        .transpose()
        .map_err(std::io::Error::other)?;
    let facts = crate::dependency::inspect(&snapshot, &dependency, "locked_offline_metadata");
    let package = match &facts.payload.selected {
        crate::dependency::Selection::Exact { facts } => facts.package.id.clone(),
        _ => {
            let mut response = facts.map_payload(|dependency| Units {
                dependency,
                selection: None,
                contexts: None,
                native_world: NativeWorld::NotRequested,
                original_metadata,
            });
            response.command = "inspect dependency-units".into();
            return Ok(output.emit_observation(response)?);
        }
    };
    let world = if native_world {
        match crate::semantic::load(&root, &environment) {
            Ok(Some(record)) => match super::runtime()?
                .block_on(crate::semantic::client(
                    &root,
                    &environment,
                    &record,
                    &crate::semantic::Request::PackageContext {
                        package_id: package.clone(),
                    },
                ))
                .and_then(crate::semantic::Response::into_package_context)
            {
                Ok(observation) => NativeWorld::Available {
                    observation: Box::new(observation),
                },
                Err(error) => NativeWorld::Unavailable {
                    reason: error.to_string(),
                },
            },
            Ok(None) => NativeWorld::Unavailable {
                reason: "native world is unprepared; explicitly prepare a session".into(),
            },
            Err(error) => NativeWorld::Unavailable {
                reason: error.to_string(),
            },
        }
    } else {
        NativeWorld::NotRequested
    };
    let admitted = crate::test_workflow::CheckIntent::new(
        snapshot.metadata,
        selection.request(crate::selection::Operation::Check)?,
        environment,
    )
    .and_then(crate::test_workflow::CheckIntent::admit)
    .map_err(|error| HostError::Workflow(Box::new(error)))?;
    let contexts = admitted
        .resolved()
        .admitted()
        .iter()
        .map(|context| {
            let dependency_units: Vec<_> = context
                .units
                .iter()
                .filter(|unit| unit.package_id == package)
                .cloned()
                .collect();
            let native_joins = join_units(&dependency_units, &world);
            UnitContext {
                arguments: context.arguments.clone(),
                roots: context
                    .units
                    .iter()
                    .filter(|unit| unit.root)
                    .cloned()
                    .collect(),
                dependency_units,
                examined_units: context.units.len(),
                native_joins,
            }
        })
        .collect();
    let mut response = crate::envelope::Envelope::response(
        "inspect dependency-units",
        "selected dependency compiler contexts; no build or test listing performed",
        Units {
            dependency: facts.payload,
            selection: Some(admitted.resolved().plan().clone()),
            contexts: Some(contexts),
            native_world: world,
            original_metadata,
        },
    );
    response.observation.scope = "original_shared_check_intent_and_correctness_admission".into();
    response.observation.limitations = vec![
        "Cargo unit graphs describe planned compiler actions, not captured artifacts, actual compilation, runnable tests or behavioral impact".into(),
        "only selected check-context dependency instances are returned; original normalized roots/defaults/negative features and host/target identities remain separate".into(),
        "native provider readiness, fixture preparation and original physical settlement are not granted by this read".into(),
    ];
    Ok(output.emit_observation(response)?)
}
