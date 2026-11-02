// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Batch written references to exact Cargo sources, never runtime call accounting.
use super::*;
use crate::semantic::{DependencyReference, DependencyUsageAnswer, NativeLibraryProfile};
use ra_ap_hir::Crate;
use ra_ap_ide::{TryToNav as _, UpmappingResult};
use ra_ap_ide_db::defs::{Definition, IdentClass};
use std::collections::{HashMap, HashSet};

pub(super) fn query(
    snapshot: QuerySnapshot<'_>,
    session: AnalysisContext,
    mut package_ids: Vec<String>,
    workspace_member_ids: &BTreeSet<String>,
    cancellation: &CancellationToken,
) -> Result<DependencyUsageAnswer> {
    package_ids.sort();
    package_ids.dedup();
    if package_ids.is_empty() || package_ids.iter().any(String::is_empty) {
        return Err(Error::Protocol(
            "dependency usage requires exact nonempty package IDs".into(),
        ));
    }
    let profiles = snapshot
        .packages
        .iter()
        .flat_map(|package| {
            std::iter::once(NativeLibraryProfile {
                package_id: package.package_id.clone(),
                manifest: package.manifest.clone(),
                source_root: package.source_root.clone(),
                library_root: package.library_root.clone(),
                features: package.features.clone(),
                target: package.target.clone(),
                role: package.role.clone(),
            })
            .chain(package.dependencies.iter().cloned())
        })
        .collect::<BTreeSet<_>>();
    let unavailable_package_ids = package_ids
        .iter()
        .filter(|id| !profiles.iter().any(|profile| &profile.package_id == *id))
        .cloned()
        .collect();
    // Cargo membership is independent of analyzer source-root classification:
    // local/vendor dependencies may also have non-library source roots.
    let mut consumer_roots: BTreeMap<PathBuf, BTreeSet<String>> = BTreeMap::new();
    for target in snapshot.runnable_targets {
        if let Some(root) = target.manifest.parent() {
            consumer_roots
                .entry(root.to_owned())
                .or_default()
                .insert(target.package_id.clone());
        }
    }
    let mut profiles_by_library: BTreeMap<PathBuf, Vec<NativeLibraryProfile>> = BTreeMap::new();
    for profile in &profiles {
        profiles_by_library
            .entry(profile.library_root.clone())
            .or_default()
            .push(profile.clone());
    }
    let sema = Semantics::new(snapshot.db);
    let mut modules = Crate::all(snapshot.db)
        .into_iter()
        .filter(|krate| {
            snapshot
                .paths
                .get(&krate.root_file(snapshot.db))
                .is_some_and(|path| workspace_consumer(&consumer_roots, workspace_member_ids, path))
        })
        .map(|krate| krate.root_module(snapshot.db))
        .collect::<Vec<_>>();
    let mut visited = HashSet::new();
    let mut resolved: HashMap<Definition<'_>, Option<ResolvedDefinition>> = HashMap::new();
    let mut references = Vec::new();
    let mut unmapped_definitions = 0;
    while let Some(module) = modules.pop() {
        if cancellation.is_cancelled() {
            return Err(Error::Cancelled);
        }
        modules.extend(module.children(snapshot.db));
        let file = module
            .definition_source_file_id(snapshot.db)
            .original_file(snapshot.db)
            .file_id(snapshot.db);
        let source_root_id = snapshot
            .db
            .file_source_root(file)
            .source_root_id(snapshot.db);
        if snapshot
            .db
            .source_root(source_root_id)
            .source_root(snapshot.db)
            .is_library
        {
            continue;
        }
        if !visited.insert(file) {
            continue;
        }
        let Some(path) = snapshot.paths.get(&file) else {
            continue;
        };
        if !workspace_consumer(&consumer_roots, workspace_member_ids, path) {
            continue;
        }
        let Some(digest) = snapshot.digests.get(path) else {
            continue;
        };
        let parsed = sema.parse_guess_edition(file);
        for token in parsed
            .syntax()
            .descendants_with_tokens()
            .filter_map(|element| element.into_token())
        {
            if cancellation.is_cancelled() {
                return Err(Error::Cancelled);
            }
            // Keep upstream's first nonempty macro descent. Shorthand tokens may
            // intentionally resolve to multiple definitions.
            let definitions = sema
                .descend_into_macros_exact(token.clone())
                .into_iter()
                .find_map(|expanded| {
                    IdentClass::classify_token(&sema, &expanded)
                        .map(IdentClass::definitions)
                        .filter(|defs| !defs.is_empty())
                });
            let Some(definitions) = definitions else {
                continue;
            };
            for (definition, _) in definitions {
                let resolved = resolved.entry(definition).or_insert_with(|| {
                    // Filter by HIR crate source before navigation or display work.
                    let krate = definition.krate(snapshot.db)?;
                    let root = snapshot.paths.get(&krate.root_file(snapshot.db))?;
                    let matching = profiles_by_library.get(root)?;
                    if !matching
                        .iter()
                        .any(|profile| package_ids.contains(&profile.package_id))
                    {
                        return None;
                    }
                    let Some(nav) = definition.try_to_nav(&sema).map(UpmappingResult::call_site)
                    else {
                        unmapped_definitions += 1;
                        return None;
                    };
                    let Some(declaration) =
                        location(snapshot.paths, nav.file_id, nav.focus_or_full_range())
                    else {
                        unmapped_definitions += 1;
                        return None;
                    };
                    Some(ResolvedDefinition {
                        definition: declaration,
                        profiles: matching.clone(),
                        name: Some(nav.name.to_string()),
                        symbol_kind: nav
                            .kind
                            .map(|kind| format!("{kind:?}"))
                            .unwrap_or_else(|| "Unknown".into()),
                    })
                });
                let Some(resolved) = resolved else {
                    continue;
                };
                let Some(occurrence) = location(snapshot.paths, file, token.text_range()) else {
                    continue;
                };
                if occurrence.path == resolved.definition.path
                    && occurrence.start == resolved.definition.start
                    && occurrence.end == resolved.definition.end
                {
                    continue;
                }
                let role = token
                    .parent()
                    .map(|node| {
                        if node.ancestors().any(|node| ast::Use::can_cast(node.kind())) {
                            "import"
                        } else if node
                            .ancestors()
                            .any(|node| ast::Type::can_cast(node.kind()))
                        {
                            "type_reference"
                        } else {
                            "reference"
                        }
                    })
                    .unwrap_or("reference");
                references.push(DependencyReference {
                    definition: resolved.definition.clone(),
                    profiles: resolved.profiles.clone(),
                    occurrence,
                    source_sha256: digest.clone(),
                    symbol_kind: resolved.symbol_kind.clone(),
                    name: resolved.name.clone(),
                    role: role.into(),
                });
            }
        }
    }
    let indexed_files = visited.len();
    references.sort_by(|left, right| {
        (
            &left.occurrence.path,
            left.occurrence.start,
            &left.definition.path,
            left.definition.start,
        )
            .cmp(&(
                &right.occurrence.path,
                right.occurrence.start,
                &right.definition.path,
                right.definition.start,
            ))
    });
    let mut limitations = vec![
        "Observed written-token references only; imports and types are not runtime calls or measures of importance.".into(),
        "Coverage is restricted to the selected loaded cfg/features/target/test world; not all implicit operations or indirect dispatch are enumerated.".into(),
        "Macro origins and shared source profiles may be ambiguous; no unique profile or runtime implementation is inferred.".into(),
        "Symbols without original definition mapping cannot be attributed; absence of references is not proof of unused capability.".into(),
        "unmapped_definitions counts selected-package definitions without precise original navigation; unresolved tokens and definitions without captured HIR crate profiles remain unknown.".into(),
        "Only Cargo workspace-member consumer sources are scanned; local/vendor dependencies are providers, not consumer evidence. Symbol kinds are upstream navigation presentation categories.".into(),
    ];
    for package in snapshot.packages {
        limitations.extend(package.dependency_issues.iter().cloned());
    }
    limitations.sort();
    limitations.dedup();
    Ok(DependencyUsageAnswer {
        session,
        generation: SNAPSHOT_GENERATION,
        package_ids,
        unavailable_package_ids,
        references,
        indexed_files,
        unmapped_definitions,
        limitations,
    })
}

struct ResolvedDefinition {
    definition: NativeLocation,
    profiles: Vec<NativeLibraryProfile>,
    name: Option<String>,
    symbol_kind: String,
}

fn workspace_consumer(
    roots: &BTreeMap<PathBuf, BTreeSet<String>>,
    members: &BTreeSet<String>,
    path: &Path,
) -> bool {
    roots
        .iter()
        .filter(|(root, _)| path.starts_with(root))
        .max_by_key(|(root, _)| root.components().count())
        .is_some_and(|(_, owners)| owners.iter().all(|id| members.contains(id)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dependency_usage_consumer_membership_excludes_nested_nonmember_sources() {
        let roots = BTreeMap::from([
            (
                PathBuf::from("/project"),
                BTreeSet::from(["consumer".to_owned()]),
            ),
            (
                PathBuf::from("/project/vendor/dep"),
                BTreeSet::from(["supplier".to_owned()]),
            ),
        ]);
        let members = BTreeSet::from(["consumer".to_owned()]);
        assert!(workspace_consumer(
            &roots,
            &members,
            Path::new("/project/src/lib.rs")
        ));
        assert!(!workspace_consumer(
            &roots,
            &members,
            Path::new("/project/vendor/dep/src/lib.rs")
        ));
        assert!(!workspace_consumer(
            &roots,
            &members,
            Path::new("/unknown/lib.rs")
        ));
    }

    #[tokio::test]
    async fn dependency_usage_resolves_renamed_import_type_and_method_references() {
        let (root, configuration, _) = super::super::tests::fixture();
        std::fs::create_dir_all(root.path().join("supplier/src")).unwrap();
        std::fs::write(
            root.path().join("supplier/Cargo.toml"),
            "[package]\nname='supplier'\nversion='0.0.0'\nedition='2024'\n",
        )
        .unwrap();
        std::fs::write(
            root.path().join("supplier/src/lib.rs"),
            "pub struct Value { pub field: u32 } impl Value { pub const COUNT: u32 = 3; pub fn work(&self) {} } pub trait Act { fn act(&self); } impl Act for Value { fn act(&self) {} } impl core::ops::Add for Value { type Output=Self; fn add(self, _: Self)->Self { self } } pub use Value as Facade; #[macro_export] macro_rules! through { ($v:expr) => { $v.work() }; } pub fn internal(value: Value) { value.work(); let _ = value.field; }\n",
        )
        .unwrap();
        std::fs::write(root.path().join("Cargo.toml"), "[package]\nname='native-analysis-control'\nversion='0.0.0'\nedition='2024'\n[dependencies]\nrenamed={package='supplier',path='supplier'}\n[workspace]\nexclude=['supplier']\n").unwrap();
        std::fs::write(root.path().join("Cargo.lock"), "version=4\n[[package]]\nname='native-analysis-control'\nversion='0.0.0'\ndependencies=['supplier']\n[[package]]\nname='supplier'\nversion='0.0.0'\n").unwrap();
        std::fs::write(
            root.path().join("src/lib.rs"),
            "use renamed::{Facade as Local, Act}; pub fn run(value: Local) { value.work(); value.act(); let _ = value.field; let _ = Local::COUNT; let field = 3; let other = Local { field }; renamed::through!(value); let _ = value + other; }\n",
        )
        .unwrap();
        let engine = Engine::start(root.path(), configuration).unwrap();
        let id = engine
            .packages
            .iter()
            .flat_map(|package| package.dependencies.iter())
            .find(|profile| profile.source_root == root.path().join("supplier"))
            .unwrap()
            .package_id
            .clone();
        let answer = engine
            .start_dependency_usage(vec![id.clone(), "missing".into()], CancellationToken::new())
            .settle()
            .await
            .unwrap()
            .unwrap();
        assert_eq!(answer.unavailable_package_ids, vec!["missing"]);
        assert!(!answer.references.is_empty());
        assert!(
            answer
                .references
                .iter()
                .all(|reference| reference.occurrence.path == root.path().join("src/lib.rs"))
        );
        assert!(
            answer
                .references
                .iter()
                .any(|reference| reference.name.as_deref() == Some("field"))
        );
        assert!(
            answer
                .references
                .iter()
                .any(|reference| reference.name.as_deref() == Some("act"))
        );
        assert!(
            answer
                .references
                .iter()
                .any(|reference| reference.name.as_deref() == Some("COUNT"))
        );
        // Compare the slim projection to the upstream resolver on the exact same
        // small fixture, retaining its macro/shorthand/operator coverage limits.
        let (db, _) = engine.world.lock().unwrap().db.request_snapshot();
        let analysis = ra_ap_ide::AnalysisHost::with_database(db).analysis();
        let index = ra_ap_ide::StaticIndex::compute(
            &analysis,
            ra_ap_ide::VendoredLibrariesConfig::Excluded,
            false,
        );
        let paths = engine.world.lock().unwrap().paths.clone();
        let mut upstream = BTreeSet::new();
        for (_, symbol) in index.tokens.iter() {
            let Some(definition) = symbol
                .definition
                .and_then(|span| location(&paths, span.file_id, span.range))
            else {
                continue;
            };
            if !definition.path.starts_with(root.path().join("supplier")) {
                continue;
            }
            for reference in symbol
                .references
                .into_iter()
                .filter(|reference| !reference.is_definition)
            {
                let Some(occurrence) =
                    location(&paths, reference.range.file_id, reference.range.range)
                else {
                    continue;
                };
                if occurrence.path == root.path().join("src/lib.rs") {
                    upstream.insert((definition.path.clone(), definition.start, occurrence.start));
                }
            }
        }
        let observed = answer
            .references
            .iter()
            .map(|reference| {
                (
                    reference.definition.path.clone(),
                    reference.definition.start,
                    reference.occurrence.start,
                )
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(observed, upstream);
        assert!(answer.references.iter().all(|reference| {
            reference
                .profiles
                .iter()
                .any(|profile| profile.package_id == id)
        }));
        assert!(
            answer
                .references
                .iter()
                .any(|reference| reference.role == "import")
        );
        assert!(
            answer
                .references
                .iter()
                .any(|reference| reference.role == "type_reference")
        );
        assert!(
            answer
                .references
                .iter()
                .any(|reference| reference.name.as_deref() == Some("work"))
        );
        let encoded = crate::semantic::wire::response_event(
            crate::semantic::Response::DependencyUsage(answer),
        )
        .unwrap();
        use crate::code_intelligence_contracts::pb::pse::code_intelligence::v1::native_event;
        let Some(native_event::Item::DependencyUsage(answer)) = encoded.item else {
            panic!("wrong response kind");
        };
        let decoded = crate::semantic::wire::usage_answer_from_pb(*answer).unwrap();
        assert_eq!(decoded.unavailable_package_ids, vec!["missing"]);
        assert!(
            decoded
                .references
                .iter()
                .any(|reference| reference.role == "import")
        );
    }
}
