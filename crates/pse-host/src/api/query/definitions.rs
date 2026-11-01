// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Request-owned original facade/definition qualification, without acquisition or Cargo queries.
use super::{ApiError, Artifact, DefinitionEvidence};
use crate::api::{
    DocumentationClosure, DocumentationLibraryProfile, PackageSourceEvidence, invalid,
};
use rustdoc_types::{Id, ItemEnum};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    sync::Arc,
};

/// Immutable graph/source premises and per-definition routes reused throughout one request.
#[derive(Debug)]
pub struct QualificationContext {
    context: crate::api::Context,
    original: DocumentationClosure,
    bindings: BTreeMap<String, BTreeSet<DocumentationLibraryProfile>>,
    surfaces: BTreeMap<Id, (String, Vec<String>)>,
    inputs: BTreeMap<PathBuf, String>,
    definitions: BTreeMap<(PathBuf, String), PreparedDefinition>,
    graphs: BTreeMap<PathBuf, String>,
}
#[derive(Debug)]
struct PreparedDefinition {
    package_id: String,
    context_key: String,
    profile: DocumentationLibraryProfile,
    canonical: BTreeMap<Vec<String>, (Id, String)>,
}
impl QualificationContext {
    /// Decode the original hashed dependency graph once, retaining its exact native edges.
    pub fn new(facade: &Artifact) -> Result<Self, ApiError> {
        let graph = crate::api::retained_documentation_graph(facade)?;
        let original = crate::api::documentation_closure(&graph, &facade.context)?;
        let root = crate::api::documentation_root(&graph, &facade.context)?;
        let mut bindings: BTreeMap<String, BTreeSet<DocumentationLibraryProfile>> = BTreeMap::new();
        let mut original_crate_bindings: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for edge in graph.units[root]
            .dependencies
            .as_ref()
            .ok_or_else(|| invalid("original dependency edges absent"))?
        {
            let Some(binding) = &edge.extern_crate_name else {
                continue;
            };
            let unit = graph
                .units
                .get(edge.index)
                .ok_or_else(|| invalid("original dependency edge invalid"))?;
            original_crate_bindings
                .entry(unit.target.name.replace('-', "_"))
                .or_default()
                .insert(binding.clone());
            bindings
                .entry(binding.clone())
                .or_default()
                .insert(crate::api::documentation_profile(
                    unit,
                    &facade.context.source_root,
                )?);
        }
        let mut surfaces: BTreeMap<Id, (String, Vec<String>)> = facade
            .document
            .index
            .iter()
            .filter_map(|(id, item)| {
                let ItemEnum::Use(import) = &item.inner else {
                    return None;
                };
                let canonical = import.id.and_then(|id| facade.document.paths.get(&id))?;
                let binding = import.source.trim_start_matches("::").split("::").next()?;
                Some((*id, (binding.to_owned(), canonical.path.clone())))
            })
            .collect();
        // Original summary IDs also address links, bounds and impl trait owners. Resolve
        // through the exact original external-crate binding, never another artifact's ID.
        for (id, summary) in &facade.document.paths {
            if facade.document.index.contains_key(id) {
                continue;
            }
            if let Some(external) = facade.document.external_crates.get(&summary.crate_id) {
                if let Some(names) = original_crate_bindings.get(&external.name) {
                    if names.len() == 1 {
                        surfaces.entry(*id).or_insert_with(|| {
                            (names.first().unwrap().clone(), summary.path.clone())
                        });
                    }
                }
            }
        }
        let inputs = facade
            .context
            .source
            .inputs()
            .map(|(path, hash)| (facade.context.source_root.join(path), hash.to_owned()))
            .collect();
        let graphs = [(
            graph_path(facade)?,
            facade.context.unit_graph_sha256.clone(),
        )]
        .into();
        Ok(Self {
            context: facade.context.clone(),
            original,
            bindings,
            surfaces,
            inputs,
            definitions: BTreeMap::new(),
            graphs,
        })
    }
    pub(crate) fn closure(&self) -> &DocumentationClosure {
        &self.original
    }
    fn expected(
        &self,
        surface: Id,
    ) -> Result<(DocumentationLibraryProfile, Vec<String>), ApiError> {
        let (binding, canonical) = self
            .surfaces
            .get(&surface)
            .ok_or_else(|| invalid("original external reexport target absent"))?;
        let profiles = self
            .bindings
            .get(binding)
            .ok_or_else(|| invalid("external binding has no retained exact Cargo edge"))?;
        let mut profiles = profiles.iter();
        let profile = profiles
            .next()
            .ok_or_else(|| invalid("external Cargo profile absent"))?;
        if profiles.next().is_some() {
            return Err(invalid(
                "external binding has ambiguous original Cargo profiles",
            ));
        }
        Ok((profile.clone(), canonical.clone()))
    }
    fn context_matches(
        &self,
        expected: &DocumentationLibraryProfile,
        candidate: &crate::api::Context,
    ) -> bool {
        if candidate.package_id != expected.package_id
            || candidate.features != expected.features
            || candidate.target != expected.target
            || candidate.rustdoc_identity != self.context.rustdoc_identity
            || candidate.rustdoc_format != self.context.rustdoc_format
            || candidate.extraction_flags != self.context.extraction_flags
            || candidate.environment_sha256 != self.context.environment_sha256
            || !candidate
                .source
                .recorded_build_inputs_match(&self.context.source)
        {
            return false;
        }
        // Registry/git package bytes may be outside SourceFingerprint's checkout universe.
        // Only the dependency's original recorded package-byte digest can prove equivalence.
        let Some(PackageSourceEvidence::Recorded { sha256 }) =
            self.context.dependency_sources.get(&candidate.package_id)
        else {
            return false;
        };
        if sha256 != &candidate.package_source_sha256 {
            return false;
        }
        if !candidate.source.inputs().all(|(path, hash)| {
            self.inputs
                .get(&candidate.source_root.join(path))
                .is_some_and(|original| original == hash)
        }) {
            return false;
        }
        candidate.dependency_sources.iter().all(|(package,evidence)| {
            matches!((self.context.dependency_sources.get(package),evidence),
                (Some(PackageSourceEvidence::Recorded{sha256:a}),PackageSourceEvidence::Recorded{sha256:b}) if a==b)
        })
    }
    fn prepare_definition(&mut self, artifact: &Artifact) -> Result<&PreparedDefinition, ApiError> {
        let key = (artifact.path.clone(), artifact.sha256.clone());
        if !self.definitions.contains_key(&key) {
            let graph = crate::api::retained_documentation_graph(artifact)?;
            let closure = crate::api::documentation_closure(&graph, &artifact.context)?;
            if !self.context_matches(&closure.root, &artifact.context)
                || !closure.dependencies.is_subset(&self.original.dependencies)
            {
                return Err(invalid(
                    "defining source/build/profile inputs differ from the original facade dependency world",
                ));
            }
            let addresses = super::addresses(&artifact.document);
            let mut by_path: BTreeMap<String, BTreeSet<Id>> = BTreeMap::new();
            let mut candidates: BTreeMap<Vec<String>, BTreeSet<String>> = BTreeMap::new();
            for address in addresses {
                by_path
                    .entry(address.path.clone())
                    .or_default()
                    .insert(address.id);
                if let Some(summary) = artifact.document.paths.get(&address.id) {
                    candidates
                        .entry(summary.path.clone())
                        .or_default()
                        .insert(address.path);
                }
            }
            let mut canonical = BTreeMap::new();
            for (route, paths) in candidates {
                let Some(path) = paths.into_iter().min_by_key(String::len) else {
                    continue;
                };
                let Some(ids) = by_path.get(&path) else {
                    continue;
                };
                if ids.len() == 1
                    && let Some(id) = ids.first()
                {
                    canonical.insert(route, (*id, path));
                }
            }
            self.graphs.insert(
                graph_path(artifact)?,
                artifact.context.unit_graph_sha256.clone(),
            );
            self.definitions.insert(
                key.clone(),
                PreparedDefinition {
                    package_id: artifact.context.package_id.clone(),
                    context_key: artifact.context.key()?,
                    profile: closure.root,
                    canonical,
                },
            );
        }
        self.definitions
            .get(&key)
            .ok_or_else(|| invalid("prepared defining routes absent"))
    }
    fn address(&mut self, surface: Id, definition: &Artifact) -> Result<(Id, String), ApiError> {
        let (expected, canonical) = self.expected(surface)?;
        let prepared = self.prepare_definition(definition)?;
        if prepared.profile != expected {
            return Err(invalid(
                "defining profile is not the original reexport Cargo edge",
            ));
        }
        prepared.canonical.get(&canonical).cloned().ok_or_else(|| {
            invalid("original external target has no unambiguous defining declaration")
        })
    }
    /// Check a supplied original item pair using cached graph, byte and native path facts.
    pub fn validate(
        &mut self,
        surface_id: &str,
        evidence: &DefinitionEvidence,
        definition: &Artifact,
    ) -> Result<(), ApiError> {
        let id = Id(surface_id
            .parse()
            .map_err(|_| invalid("surface item is not an artifact-local ID"))?);
        if evidence.artifact != definition.path || evidence.artifact_sha256 != definition.sha256 {
            return Err(invalid(
                "defining evidence disagrees with its retained artifact",
            ));
        }
        let (target, path) = self.address(id, definition)?;
        let prepared = self.prepare_definition(definition)?;
        if evidence.package_id != prepared.package_id
            || evidence.context_key != prepared.context_key
            || evidence.item_id != target.0
            || evidence.public_path != path
        {
            return Err(invalid(
                "defining item/public path is not the original reexport's qualified target",
            ));
        }
        Ok(())
    }
    /// Rehash original graph sidecars at the final barrier, without decoding or rewalking them.
    pub fn verify_unchanged(&self) -> Result<(), ApiError> {
        for (path, expected) in &self.graphs {
            let bytes = crate::test_inventory::read_private(path)
                .map_err(|error| invalid(error.to_string()))?;
            if crate::probe::digest(&bytes) != *expected {
                return Err(invalid("original qualification dependency graph changed"));
            }
        }
        Ok(())
    }
}
fn graph_path(artifact: &Artifact) -> Result<PathBuf, ApiError> {
    Ok(artifact
        .path
        .parent()
        .ok_or_else(|| invalid("API artifact directory absent"))?
        .join("unit-graph.json"))
}
#[derive(Default)]
pub(super) struct Definitions {
    qualification: Option<Result<QualificationContext, String>>,
    packages: BTreeMap<DocumentationLibraryProfile, Result<Arc<Artifact>, String>>,
}
impl Definitions {
    pub(super) fn resolve(
        &mut self,
        facade: &Artifact,
        id: Id,
    ) -> Result<(Arc<Artifact>, Id, DefinitionEvidence), ApiError> {
        let qualification = self.qualification.get_or_insert_with(|| {
            QualificationContext::new(facade).map_err(|error| error.to_string())
        });
        let qualification = qualification
            .as_mut()
            .map_err(|error| invalid(error.clone()))?;
        let (profile, _) = qualification.expected(id)?;
        let cached = self.packages.entry(profile.clone()).or_insert_with(|| {
            acquire_existing(facade, &profile, qualification)
                .map(Arc::new)
                .map_err(|error| error.to_string())
        });
        let definition = cached
            .as_ref()
            .map_err(|reason| invalid(reason.clone()))?
            .clone();
        let (id, public_path) = qualification.address(id, &definition)?;
        let prepared = qualification.prepare_definition(&definition)?;
        let proof = DefinitionEvidence {
            package_id: prepared.package_id.clone(),
            context_key: prepared.context_key.clone(),
            artifact: definition.path.clone(),
            artifact_sha256: definition.sha256.clone(),
            item_id: id.0,
            public_path,
        };
        Ok((definition, id, proof))
    }
    pub(super) fn verify_unchanged(&self) -> Result<(), ApiError> {
        if let Some(Ok(qualification)) = &self.qualification {
            qualification.verify_unchanged()?;
        }
        Ok(())
    }
}
pub(super) fn validate_definition(
    facade: &Artifact,
    surface_id: &str,
    evidence: &DefinitionEvidence,
    definition: &Artifact,
) -> Result<(), ApiError> {
    let mut qualification = QualificationContext::new(facade)?;
    qualification.validate(surface_id, evidence, definition)?;
    qualification.verify_unchanged()
}
fn acquire_existing(
    facade: &Artifact,
    expected: &DocumentationLibraryProfile,
    qualification: &mut QualificationContext,
) -> Result<Artifact, ApiError> {
    let api_directory = facade
        .path
        .parent()
        .and_then(|path| path.parent())
        .and_then(|path| path.parent())
        .ok_or_else(|| invalid("retained API store absent"))?;
    let mut contexts = std::fs::read_dir(api_directory)?.collect::<Result<Vec<_>, _>>()?;
    contexts.sort_by_key(std::fs::DirEntry::file_name);
    for context in contexts {
        if !context.file_type()?.is_dir() {
            continue;
        }
        let mut generations = std::fs::read_dir(context.path())?.collect::<Result<Vec<_>, _>>()?;
        generations.sort_by_key(std::fs::DirEntry::file_name);
        for generation in generations.into_iter().rev() {
            let directory = generation.path();
            if !directory.join("artifact.json").is_file() {
                continue;
            }
            let bytes = crate::test_inventory::read_private(&directory.join("context.json"))
                .map_err(|error| invalid(error.to_string()))?;
            let candidate: crate::api::Context = serde_json::from_slice(&bytes)?;
            if !qualification.context_matches(expected, &candidate) {
                continue;
            }
            let artifact = crate::api::retained(&directory.join("document.json"))?;
            if qualification.prepare_definition(&artifact).is_err() {
                continue;
            }
            return Ok(artifact);
        }
    }
    Err(invalid(
        "no acquired defining artifact proves the facade's original dependency profile and package bytes",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, process::Command};

    #[test]
    fn acquired_registry_definition_requires_original_nonlocal_package_bytes() {
        let sandbox = tempfile::tempdir().unwrap();
        let root = sandbox.path().join("project");
        let state = sandbox.path().join("state");
        let registry = sandbox.path().join("registry");
        let package = registry.join("registry-definition-0.1.0");
        for path in [root.join("src"), root.join(".cargo"), package.join("src")] {
            fs::create_dir_all(path).unwrap();
        }
        fs::write(root.join("Cargo.toml"),"[package]\nname='registry-facade'\nversion='0.1.0'\nedition='2024'\n[dependencies]\nregistry-definition='=0.1.0'\n").unwrap();
        fs::write(
            root.join("src/lib.rs"),
            "#[doc(inline)] pub use registry_definition::increment as renamed;\n",
        )
        .unwrap();
        fs::write(
            root.join("rust-toolchain.toml"),
            include_str!("../../../../../rust-toolchain.toml"),
        )
        .unwrap();
        fs::write(root.join(".cargo/config.toml"),format!("[source.crates-io]\nreplace-with='fixture-source'\n[source.fixture-source]\ndirectory={registry:?}\n")).unwrap();
        fs::write(
            package.join("Cargo.toml"),
            "[package]\nname='registry-definition'\nversion='0.1.0'\nedition='2024'\n",
        )
        .unwrap();
        let set_source = |text: &str| {
            fs::write(package.join("src/lib.rs"), text).unwrap();
            let files = ["Cargo.toml", "src/lib.rs"]
                .into_iter()
                .map(|path| {
                    (
                        path,
                        crate::probe::digest(&fs::read(package.join(path)).unwrap()),
                    )
                })
                .collect::<BTreeMap<_, _>>();
            fs::write(
                package.join(".cargo-checksum.json"),
                serde_json::to_vec(&serde_json::json!({"package":null,"files":files})).unwrap(),
            )
            .unwrap();
        };
        let original = "pub fn increment(value:u32)->u32 {value+1}\n";
        set_source(original);
        let environment: crate::environment::Environment = std::env::vars_os().collect();
        let cargo = crate::correctness::real_cargo_in(&environment).unwrap();
        let lock = Command::new(cargo)
            .args(["generate-lockfile", "--offline"])
            .current_dir(&root)
            .env_clear()
            .envs(&environment)
            .output()
            .unwrap();
        assert!(
            lock.status.success(),
            "{}",
            String::from_utf8_lossy(&lock.stderr)
        );
        let snapshot = crate::dependency::capture(&root, &environment).unwrap();
        let dependency = snapshot
            .metadata
            .packages
            .iter()
            .find(|package| package.name == "registry-definition")
            .unwrap();
        assert!(
            dependency.source.is_some(),
            "this fixture must exercise Cargo registry provenance, not a path dependency"
        );
        let acquire = |name: &str| {
            let acquisition =
                crate::api::prepare(&root, &state, &snapshot, name, None, &environment).unwrap();
            let output = Command::new(&acquisition.command[0])
                .args(&acquisition.command[1..])
                .current_dir(&root)
                .env_clear()
                .envs(&acquisition.environment)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            crate::api::finish(&acquisition, &snapshot).unwrap()
        };
        let facade = acquire("registry-facade");
        assert!(
            !facade.context.source.inputs().any(|(path, _)| facade
                .context
                .source_root
                .join(path)
                .starts_with(&package)),
            "nonlocal registry source must lie outside the checkout source fingerprint"
        );
        assert!(matches!(
            facade
                .context
                .dependency_sources
                .get(&dependency.id.to_string()),
            Some(PackageSourceEvidence::Recorded { .. })
        ));
        // Same source-qualified package/version/profile, changed bytes outside the
        // checkout fingerprint: this later artifact cannot prove the original alias.
        set_source("pub fn increment(value:bool)->bool {value}\n");
        let changed = acquire("registry-definition");
        let query = super::super::Query {
            search: Some("renamed".into()),
            input_shape: Some(super::super::TypePattern::parse("bool").unwrap()),
            ..Default::default()
        };
        let unknown = super::super::inspect(&facade, &query, 20, None).unwrap();
        assert_eq!(unknown.matched_candidates, 0);
        assert_eq!(unknown.unknown_candidates, 1);
        assert_ne!(
            facade.context.dependency_sources[&dependency.id.to_string()],
            PackageSourceEvidence::Recorded {
                sha256: changed.context.package_source_sha256
            }
        );
        set_source(original);
        let definition = acquire("registry-definition");
        let positive = super::super::inspect(
            &facade,
            &super::super::Query {
                input_shape: Some(super::super::TypePattern::parse("u32").unwrap()),
                ..query
            },
            20,
            None,
        )
        .unwrap();
        assert_eq!(positive.matched_candidates, 1, "{positive:?}");
        let candidate = &positive.candidates[0];
        assert_eq!(
            candidate.definition.as_ref().unwrap().artifact_sha256,
            definition.sha256
        );
        super::super::validate_definition(
            &facade,
            &candidate.item_id.to_string(),
            candidate.definition.as_ref().unwrap(),
            &definition,
        )
        .unwrap();
    }

    #[test]
    fn actual_external_aliases_require_acquired_original_definition_shape() {
        let sandbox = tempfile::tempdir().unwrap();
        let root = sandbox.path().join("project");
        let state = sandbox.path().join("state");
        for path in ["src", ".cargo", "dependency/src"] {
            fs::create_dir_all(root.join(path)).unwrap();
        }
        fs::write(root.join("Cargo.toml"), "[package]\nname='alias-facade'\nversion='0.1.0'\nedition='2024'\n[workspace]\nexclude=['dependency']\n[dependencies]\noriginal_binding={package='alias-definition',path='dependency'}\n").unwrap();
        fs::write(
            root.join("dependency/Cargo.toml"),
            "[package]\nname='alias-definition'\nversion='0.1.0'\nedition='2024'\n",
        )
        .unwrap();
        fs::write(root.join("src/lib.rs"), "#[doc(inline)] pub use original_binding::increment as renamed;\n#[doc(inline)] pub use original_binding::increment as secondary;\npub fn local(value:bool)->bool {value}\n").unwrap();
        fs::write(root.join("dependency/src/lib.rs"), "pub fn increment(value:u32)->u32 {value+1}\npub fn unrelated(value:u32)->u32 {value}\n").unwrap();
        fs::write(root.join("Cargo.lock"), "version=4\n[[package]]\nname=\"alias-facade\"\nversion=\"0.1.0\"\ndependencies=[\"alias-definition\"]\n[[package]]\nname=\"alias-definition\"\nversion=\"0.1.0\"\n").unwrap();
        fs::write(root.join(".cargo/config.toml"), "").unwrap();
        fs::write(
            root.join("rust-toolchain.toml"),
            include_str!("../../../../../rust-toolchain.toml"),
        )
        .unwrap();
        let environment = std::env::vars_os().collect();
        let snapshot = crate::dependency::capture(&root, &environment).unwrap();
        let acquire_with = |name: &str, environment: &crate::environment::Environment| {
            let acquisition =
                crate::api::prepare(&root, &state, &snapshot, name, None, environment).unwrap();
            let output = Command::new(&acquisition.command[0])
                .args(&acquisition.command[1..])
                .current_dir(&acquisition.cwd)
                .env_clear()
                .envs(&acquisition.environment)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            crate::api::finish(&acquisition, &snapshot).unwrap()
        };
        let acquire = |name: &str| acquire_with(name, &environment);
        let facade = acquire("alias-facade");
        let query = super::super::Query {
            search: Some("renamed".into()),
            input_shape: Some(super::super::TypePattern::parse("u32").unwrap()),
            output_shape: Some(super::super::TypePattern::parse("u32").unwrap()),
            ..Default::default()
        };
        let missing = super::super::inspect(&facade, &query, 20, None).unwrap();
        assert_eq!(missing.matched_candidates, 0);
        assert_eq!(missing.unknown_candidates, 1);
        assert_eq!(missing.candidates.len(), 1);
        assert!(missing.candidates[0].qualification_unknown.is_some());
        let missing_graph = super::super::adjacency::inspect(
            &facade,
            Id(missing.candidates[0].item_id),
            &[super::super::adjacency::Relation::Reexport],
            16,
        )
        .unwrap();
        assert!(!missing_graph.complete);
        assert!(
            missing_graph
                .nodes
                .iter()
                .any(|node| !node.declaration_available)
        );
        let unrelated = super::super::inspect(
            &facade,
            &super::super::Query {
                search: Some("local".into()),
                input_shape: Some(super::super::TypePattern::parse("bool").unwrap()),
                ..Default::default()
            },
            20,
            None,
        )
        .unwrap();
        assert_eq!(unrelated.matched_candidates, 1);
        assert_eq!(unrelated.unknown_candidates, 0);
        let definition = acquire("alias-definition");
        let page = super::super::inspect(&facade, &query, 20, None).unwrap();
        assert_eq!(page.matched_candidates, 1, "{page:?}");
        assert_eq!(page.unknown_candidates, 0);
        let row = &page.candidates[0];
        assert!(row.public_path.ends_with("::renamed"));
        assert!(matches!(
            facade.document.index[&Id(row.item_id)].inner,
            ItemEnum::Use(_)
        ));
        assert_eq!(
            row.definition.as_ref().unwrap().artifact_sha256,
            definition.sha256
        );
        assert!(row.signature.contains("u32"));
        let proof = row.definition.as_ref().unwrap();
        let graph = super::super::adjacency::inspect(
            &facade,
            Id(row.item_id),
            &[super::super::adjacency::Relation::Reexport],
            16,
        )
        .unwrap();
        assert!(graph.complete, "{graph:?}");
        assert_eq!(graph.edges.len(), 1);
        let edge = &graph.edges[0];
        assert_ne!(edge.source, edge.target);
        let crate::investigation::Locator::Api { artifact: from, .. } =
            &graph.nodes[edge.source].reference.locator
        else {
            panic!("qualified facade expected")
        };
        let crate::investigation::Locator::Api {
            artifact: to,
            definition: node_proof,
            ..
        } = &graph.nodes[edge.target].reference.locator
        else {
            panic!("qualified definition expected")
        };
        assert_eq!(from, &facade.path);
        assert_eq!(to, &definition.path);
        assert!(node_proof.is_none());
        assert!(matches!(
            &graph.witnesses[edge.witness].reference.locator,
            crate::investigation::Locator::Api {
                definition: Some(_),
                ..
            }
        ));
        super::super::validate_definition(&facade, &row.item_id.to_string(), proof, &definition)
            .unwrap();
        let unrelated = definition
            .document
            .index
            .values()
            .find(|item| item.name.as_deref() == Some("unrelated"))
            .unwrap();
        let mut forged = proof.clone();
        forged.item_id = unrelated.id.0;
        forged.public_path = "alias_definition::unrelated".into();
        assert!(
            super::super::validate_definition(
                &facade,
                &row.item_id.to_string(),
                &forged,
                &definition
            )
            .is_err()
        );
        let newer = acquire("alias-definition");
        assert_ne!(newer.path, definition.path);
        super::super::validate_definition(&facade, &row.item_id.to_string(), proof, &definition)
            .unwrap();
        let absent = super::super::inspect(
            &facade,
            &super::super::Query {
                input_shape: Some(super::super::TypePattern::parse("bool").unwrap()),
                ..query.clone()
            },
            20,
            None,
        )
        .unwrap();
        assert_eq!(absent.matched_candidates, 0);
        assert_eq!(absent.unknown_candidates, 0);
        let aliases = super::super::inspect(
            &facade,
            &super::super::Query {
                kind: Some(super::super::Kind::Function),
                input_shape: Some(super::super::TypePattern::parse("u32").unwrap()),
                ..Default::default()
            },
            20,
            None,
        )
        .unwrap();
        assert_eq!(aliases.matched_candidates, 2);
        assert_ne!(aliases.candidates[0].item_id, aliases.candidates[1].item_id);
        assert_eq!(
            aliases.candidates[0].definition.as_ref().unwrap().item_id,
            aliases.candidates[1].definition.as_ref().unwrap().item_id
        );
        let mut qualification = QualificationContext::new(&facade).unwrap();
        let first = &aliases.candidates[0];
        let alias_definition =
            crate::api::retained(&first.definition.as_ref().unwrap().artifact).unwrap();
        qualification
            .validate(
                &first.item_id.to_string(),
                first.definition.as_ref().unwrap(),
                &alias_definition,
            )
            .unwrap();
        let facade_graph = graph_path(&facade).unwrap();
        let defining_graph = graph_path(&alias_definition).unwrap();
        let held_facade = facade_graph.with_extension("held");
        let held_definition = defining_graph.with_extension("held");
        fs::rename(&facade_graph, &held_facade).unwrap();
        fs::rename(&defining_graph, &held_definition).unwrap();
        let second = &aliases.candidates[1];
        // A second alias checks cached original premises; the final barrier still
        // detects any graph disappearance/mutation before accepting the batch.
        qualification
            .validate(
                &second.item_id.to_string(),
                second.definition.as_ref().unwrap(),
                &alias_definition,
            )
            .unwrap();
        assert!(qualification.verify_unchanged().is_err());
        fs::rename(held_facade, facade_graph).unwrap();
        fs::rename(held_definition, defining_graph).unwrap();
        qualification.verify_unchanged().unwrap();
        // A newer acquired artifact with the same package spelling cannot substitute
        // different original source bytes for the facade's captured dependency.
        fs::remove_file(definition.path.parent().unwrap().join("artifact.json")).unwrap();
        fs::remove_file(newer.path.parent().unwrap().join("artifact.json")).unwrap();
        let mut other_environment = environment.clone();
        other_environment.insert("RUSTFLAGS".into(), "--cfg alias_other_context".into());
        let changed_configuration = acquire_with("alias-definition", &other_environment);
        assert_ne!(
            changed_configuration.context.environment_sha256,
            definition.context.environment_sha256
        );
        let other_inputs = super::super::inspect(&facade, &query, 20, None).unwrap();
        assert_eq!(other_inputs.matched_candidates, 0);
        assert_eq!(other_inputs.unknown_candidates, 1);
        fs::write(
            root.join("dependency/src/lib.rs"),
            "pub fn increment(value:bool)->bool {value}\n",
        )
        .unwrap();
        let changed = acquire("alias-definition");
        assert_ne!(
            changed.context.package_source_sha256,
            definition.context.package_source_sha256
        );
        let mismatch = super::super::inspect(&facade, &query, 20, None).unwrap();
        assert_eq!(mismatch.matched_candidates, 0);
        assert_eq!(mismatch.unknown_candidates, 1);
        assert!(mismatch.candidates[0].definition.is_none());
        fs::write(
            root.join("dependency/src/lib.rs"),
            "pub mod nested { pub fn child(value:u32)->u32 {value} }\n",
        )
        .unwrap();
        fs::write(
            root.join("src/lib.rs"),
            "pub use original_binding::nested as boundary; pub fn local()->bool {true}\n",
        )
        .unwrap();
        let module_facade = acquire("alias-facade");
        let children = super::super::inspect(
            &module_facade,
            &super::super::Query {
                search: Some("child".into()),
                input_shape: Some(super::super::TypePattern::parse("u32").unwrap()),
                ..Default::default()
            },
            20,
            None,
        )
        .unwrap();
        assert_eq!(children.matched_candidates, 0);
        assert_eq!(
            children.unexamined_external_routes,
            vec!["alias_facade::boundary"]
        );
        let unrelated_path = super::super::inspect(
            &module_facade,
            &super::super::Query {
                path: Some("local".into()),
                ..Default::default()
            },
            20,
            None,
        )
        .unwrap();
        assert_eq!(unrelated_path.matched_candidates, 1);
        assert_eq!(
            unrelated_path.unexamined_external_routes,
            vec!["alias_facade::boundary"]
        );
        for path in ["child", "deep::child"] {
            let suffix = super::super::inspect(
                &module_facade,
                &super::super::Query {
                    path: Some(path.into()),
                    ..Default::default()
                },
                20,
                None,
            )
            .unwrap();
            assert_eq!(suffix.matched_candidates, 0);
            assert_eq!(
                suffix.unexamined_external_routes,
                vec!["alias_facade::boundary"]
            );
        }
        fs::write(
            root.join("src/lib.rs"),
            "pub use original_binding::nested::*;\n",
        )
        .unwrap();
        let glob_facade = acquire("alias-facade");
        let glob = super::super::inspect(
            &glob_facade,
            &super::super::Query {
                path: Some("child".into()),
                ..Default::default()
            },
            20,
            None,
        )
        .unwrap();
        assert_eq!(glob.unexamined_external_routes, vec!["alias_facade::*"]);
    }
}
