// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Side-effect-free parser discovery over explicitly supplied metadata/evidence owners.
#[path = "../../../scripts/cli-catalog.rs"]
pub mod description;
use crate::{HostError, invocation::InvocationStore, selection};
use clap::{Command, CommandFactory as _};
use description::{
    Availability, Candidate, CandidateProvider, CandidateSet, Exhaustiveness, Freshness, Page,
    Source, Window,
};
use sha2::Digest as _;
/// An explicitly selected retained metadata snapshot, never produced by passive discovery.
#[derive(Debug)]
pub struct MetadataSnapshot {
    /// Cargo's actual decoded source record.
    pub metadata: cargo_metadata::Metadata,
    /// Exact bytes read, independent of caller-supplied filenames.
    pub digest: String,
}
impl MetadataSnapshot {
    /// Read a selected snapshot without starting Cargo or interpreting it as built evidence.
    pub fn read(path: &std::path::Path) -> Result<Self, crate::correctness::CorrectnessError> {
        let bytes = std::fs::read(path)?;
        let metadata = serde_json::from_slice(&bytes)?;
        let digest = sha2::Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        Ok(Self { metadata, digest })
    }
}

/// Native provider IDs attach to actual arguments, independently of parser validation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Domain {
    /// Workspace members from a supplied Cargo metadata snapshot.
    Packages,
    /// Unambiguous package/target intentions accepted by the current selection owner.
    Targets,
    /// Checkout-authenticated invocation IDs; metadata grants no control or workload status.
    Invocations,
    /// Atoms from the native provider's authoritative capability declaration.
    NativeCapabilities,
}
impl Domain {
    /// Stable discovery association, never a replacement validation rule.
    pub fn name(self) -> &'static str {
        match self {
            Self::Packages => "workspace-packages",
            Self::Targets => "workspace-targets",
            Self::Invocations => "invocation-ids",
            Self::NativeCapabilities => "native-capability-atoms",
        }
    }
    /// Attach a lazy descriptive association without altering the value parser.
    pub fn association(self) -> CandidateProvider {
        CandidateProvider {
            name: self.name().into(),
        }
    }
    fn parse(name: &str) -> Option<Self> {
        [
            Self::Packages,
            Self::Targets,
            Self::Invocations,
            Self::NativeCapabilities,
        ]
        .into_iter()
        .find(|domain| domain.name() == name)
    }
}
/// Captured inputs. Construction never starts Cargo, shells, services or fixtures.
#[derive(Debug, Default)]
pub struct Context<'a> {
    /// Caller-selected cached metadata; absence stays explicitly unavailable.
    pub metadata: Option<&'a cargo_metadata::Metadata>,
    /// Actual snapshot identity when supplied by its retaining owner.
    pub metadata_generation: Option<&'a str>,
    /// Established currentness, independently of source availability.
    pub metadata_freshness: Option<Freshness>,
    /// Bound checkout evidence owner; discovery does not create directories.
    pub invocations: Option<&'a InvocationStore>,
}
fn suggestions(
    source: Source,
    exhaustiveness: Exhaustiveness,
    values: Vec<Candidate>,
    limitations: Vec<String>,
) -> CandidateSet {
    CandidateSet {
        source,
        availability: Availability::Available,
        exhaustiveness,
        values,
        limitations,
    }
}
impl Context<'_> {
    /// Evaluate only this selected domain; unavailable input never becomes empty success.
    pub fn provide(&self, name: &str) -> Result<CandidateSet, HostError> {
        let Some(domain) = Domain::parse(name) else {
            return Ok(CandidateSet::unavailable(
                name,
                "no domain provider is associated with this parser owner",
            ));
        };
        match domain {
            Domain::Packages | Domain::Targets => {
                let Some(metadata) = self.metadata else {
                    return Ok(CandidateSet::unavailable(
                        domain.name(),
                        "cached Cargo metadata is unavailable; discovery does not run Cargo or build inventories",
                    ));
                };
                let source = Source {
                    owner: "captured Cargo workspace metadata".into(),
                    generation: self.metadata_generation.map(str::to_owned),
                    freshness: self.metadata_freshness.unwrap_or(Freshness::Unknown),
                };
                let values=metadata.workspace_packages().into_iter().flat_map(|package| {
                    if domain==Domain::Packages {return vec![Candidate{value:package.name.to_string(),aliases:vec![package.id.repr.clone()],help:Some("workspace member in the supplied metadata snapshot".into()),hidden:false}];}
                    package.targets.iter().filter_map(|target| {
                        let request=selection::Request{operation:selection::Operation::Test,scope:selection::Scope::Packages(vec![selection::PackageIntent{package:package.id.repr.clone(),target:Some(selection::TargetIntent{name:target.name.clone(),kind:None}),features:selection::Features::default()}]),platform:None,filters:selection::Filters::default(),ignored:selection::IgnoredPolicy::Default};
                        // Reuse execution's pure target/feature owner. This establishes neither
                        // admitted compiler units nor a runnable test population.
                        if selection::plan(metadata,request).is_err() {return None;}
                        Some(Candidate{value:format!("{}:{}",package.name,target.name),aliases:Vec::new(),help:Some(format!("declared target; kinds={}; required_features={}; execution readiness is not established",target.kind.iter().map(ToString::to_string).collect::<Vec<_>>().join(","),target.required_features.join(","))),hidden:false})
                    }).collect::<Vec<_>>()
                }).collect();
                Ok(suggestions(source,Exhaustiveness::Exhaustive,values,vec!["snapshot declarations are not binaries or execution-readiness evidence; ambiguous target names need owner disambiguation".into()]))
            }
            Domain::Invocations => {
                let Some(store) = self.invocations else {
                    return Ok(CandidateSet::unavailable(
                        domain.name(),
                        "checkout invocation store is not bound",
                    ));
                };
                let records = store.history::<Box<serde_json::value::RawValue>>()?;
                let values=records.into_iter().map(|record|Candidate{value:record.selection.invocation_id.to_string(),aliases:Vec::new(),help:Some(format!("{}; created_unix_ms={}; metadata does not prove running or settled state",record.selection.command,record.created_unix_ms)),hidden:false}).collect();
                Ok(suggestions(Source{owner:"authenticated checkout invocation metadata".into(),generation:None,freshness:Freshness::Unknown},Exhaustiveness::Exhaustive,values,vec!["directory observations are not an atomic snapshot, workload status or control authority".into()]))
            }
            Domain::NativeCapabilities => Ok(suggestions(
                Source {
                    owner: "native provider capability declaration".into(),
                    generation: None,
                    freshness: Freshness::CurrentGeneration,
                },
                Exhaustiveness::Open,
                crate::native::capability_names()
                    .iter()
                    .map(|name| Candidate {
                        value: (*name).into(),
                        aliases: Vec::new(),
                        help: Some(
                            "capability atom; the owner also accepts comma-separated combinations"
                                .into(),
                        ),
                        hidden: false,
                    })
                    .collect(),
                vec![
                    "atoms are not an exhaustive enumeration of accepted combinations and orders"
                        .into(),
                ],
            )),
        }
    }
}
/// Shared parser with descriptive associations only; argument validation is unchanged.
pub fn command_tree() -> Command {
    let tree = crate::cli::Cli::command().mut_subcommand("native", |command| {
        command.mut_subcommand("prepare", |command| {
            command.mut_arg("capabilities", |arg| {
                arg.add(Domain::NativeCapabilities.association())
            })
        })
    });
    fn attach(mut command: Command, path: Vec<String>) -> Command {
        let name = path.join(" ");
        let mut schemas = Vec::new();
        for name in [
            name.clone(),
            format!("{name} candidates"),
            format!("{name} brief"),
            format!("{name} batch"),
        ] {
            if let Ok(schema) =
                crate::cli::response_schema(&name, &crate::envelope::Presentation::default())
                && let Some(identity) = schema.get("$id").and_then(serde_json::Value::as_str)
            {
                schemas.push(identity.to_owned());
            }
        }
        let mut example = path.clone();
        example.push("--help".into());
        let mut route = vec!["pse".into()];
        route.extend(path.clone());
        command = command.add(description::CommandMetadata {
            examples: vec![example],
            output_schemas: schemas,
            invocation_route: route,
            features: Vec::new(),
        });
        let names = command
            .get_subcommands()
            .map(|child| child.get_name().to_owned())
            .collect::<Vec<_>>();
        for name in names {
            let mut child_path = path.clone();
            child_path.push(name.clone());
            command = command.mut_subcommand(name, |child| attach(child, child_path));
        }
        command
    }
    attach(tree, Vec::new())
}
/// Actual command descriptions, without evaluating dynamic providers.
pub fn commands(
    path: &[String],
    recursive: bool,
    window: &Window,
) -> Page<description::Description> {
    description::commands(command_tree(), path, recursive, window)
}
/// Prefix/paged candidate inquiry over explicitly supplied captured inputs.
pub fn candidates(
    path: &[String],
    argument: &str,
    window: &Window,
    context: &Context<'_>,
) -> Result<Page<Candidate>, HostError> {
    description::candidates(command_tree(), path, argument, window, |name| {
        context.provide(name)
    })
}

#[cfg(test)]
mod tests;
