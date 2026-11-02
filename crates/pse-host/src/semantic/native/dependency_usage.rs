// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Batch written references to exact Cargo sources, never runtime call accounting.
use super::*;
use crate::semantic::{DependencyReference, DependencyUsageAnswer, NativeLibraryProfile};

pub(super) fn query(
    snapshot: QuerySnapshot<'_>,
    session: AnalysisContext,
    mut package_ids: Vec<String>,
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
    let analysis = ra_ap_ide::AnalysisHost::with_database(snapshot.db.clone()).analysis();
    let index = ra_ap_ide::StaticIndex::compute(
        &analysis,
        ra_ap_ide::VendoredLibrariesConfig::Excluded,
        false,
    );
    let indexed_files = index.files.len();
    let mut references = Vec::new();
    let mut unmapped_definitions = 0;
    for (_, symbol) in index.tokens.iter() {
        if cancellation.is_cancelled() {
            return Err(Error::Cancelled);
        }
        let Some(definition) = symbol
            .definition
            .and_then(|range| location(snapshot.paths, range.file_id, range.range))
        else {
            unmapped_definitions += 1;
            continue;
        };
        let matching = profiles_for_path(&profiles, &definition.path);
        if !matching
            .iter()
            .any(|profile| package_ids.contains(&profile.package_id))
        {
            continue;
        }
        for reference in symbol
            .references
            .into_iter()
            .filter(|reference| !reference.is_definition)
        {
            let Some(occurrence) = location(
                snapshot.paths,
                reference.range.file_id,
                reference.range.range,
            ) else {
                continue;
            };
            let Some(digest) = snapshot.digests.get(&occurrence.path) else {
                continue;
            };
            let parsed = Semantics::new(snapshot.db).parse_guess_edition(reference.range.file_id);
            let role = parsed
                .syntax()
                .token_at_offset(reference.range.range.start())
                .right_biased()
                .and_then(|token| token.parent())
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
                definition: definition.clone(),
                profiles: matching.clone(),
                occurrence,
                source_sha256: digest.clone(),
                symbol_kind: format!("{:?}", symbol.kind),
                name: symbol.display_name.clone(),
                role: role.into(),
            });
        }
    }
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
        "Coverage is restricted to the selected loaded cfg/features/target/test world; implicit operations and indirect dispatch are not enumerated.".into(),
        "Macro origins and shared source profiles may be ambiguous; no unique profile or runtime implementation is inferred.".into(),
        "Symbols without original definition mapping cannot be attributed; absence of references is not proof of unused capability.".into(),
        "Generated definitions outside captured package source roots may remain unattributed; unmapped_definitions counts the whole indexed world, not only requested packages.".into(),
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

fn profiles_for_path(
    profiles: &BTreeSet<NativeLibraryProfile>,
    path: &Path,
) -> Vec<NativeLibraryProfile> {
    let depth = profiles
        .iter()
        .filter(|profile| path.starts_with(&profile.source_root))
        .map(|profile| profile.source_root.components().count())
        .max();
    profiles
        .iter()
        .filter(|profile| {
            path.starts_with(&profile.source_root)
                && Some(profile.source_root.components().count()) == depth
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dependency_usage_profiles_keep_ambiguity_and_prefer_specific_source_root() {
        let profile = |id: &str, root: &str, role: &str| NativeLibraryProfile {
            package_id: id.into(),
            manifest: Path::new(root).join("Cargo.toml"),
            source_root: root.into(),
            library_root: Path::new(root).join("src/lib.rs"),
            features: BTreeSet::new(),
            target: None,
            role: role.into(),
        };
        let profiles = BTreeSet::from([
            profile("outer", "/project", "lib"),
            profile("inner", "/project/dep", "lib"),
            profile("inner", "/project/dep", "test"),
        ]);
        let matched = profiles_for_path(&profiles, Path::new("/project/dep/src/lib.rs"));
        assert_eq!(matched.len(), 2);
        assert!(matched.iter().all(|profile| profile.package_id == "inner"));
        assert!(profiles_for_path(&profiles, Path::new("/unknown/lib.rs")).is_empty());
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
            "pub struct Value; impl Value { pub fn work(&self) {} } pub use Value as Facade;\n",
        )
        .unwrap();
        std::fs::write(root.path().join("Cargo.toml"), "[package]\nname='native-analysis-control'\nversion='0.0.0'\nedition='2024'\n[dependencies]\nrenamed={package='supplier',path='supplier'}\n[workspace]\n").unwrap();
        std::fs::write(root.path().join("Cargo.lock"), "version=4\n[[package]]\nname='native-analysis-control'\nversion='0.0.0'\ndependencies=['supplier']\n[[package]]\nname='supplier'\nversion='0.0.0'\n").unwrap();
        std::fs::write(
            root.path().join("src/lib.rs"),
            "use renamed::Facade as Local; pub fn run(value: Local) { value.work(); }\n",
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
