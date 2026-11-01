// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Compare selected owned observations from independently authenticated native worlds.
use super::*;
use crate::semantic::{self as native, FactLocation};
use std::collections::BTreeSet;

#[derive(Debug, Serialize, JsonSchema)]
pub(super) struct Side {
    pub reference: ResultReference,
    pub checkout: PathBuf,
    pub configuration_identity: String,
    pub declaration_path: Option<Vec<String>>,
    pub declaration_candidates: usize,
    pub original: native::Answer,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, JsonSchema)]
pub(super) struct Address {
    pub relation: String,
    pub path: query::NativePath,
    pub start: usize,
    pub end: usize,
    pub source_sha256: String,
}
#[derive(Debug, Serialize, JsonSchema)]
pub(in crate::cli) struct Comparison {
    before: Observation<Side>,
    after: Observation<Side>,
    pub correspondence: String,
    pub context_differences: Vec<String>,
    before_targets: Vec<Address>,
    after_targets: Vec<Address>,
    pub resolved_observation_changed: Option<bool>,
    unknown: Vec<String>,
    limitations: Vec<String>,
}
impl Comparison {
    pub(super) fn partial(&self) -> bool {
        self.before.value().is_none()
            || self.after.value().is_none()
            || !self.unknown.is_empty()
            || self.correspondence != "selected_declaration_candidate"
    }
}
pub(super) fn record(
    root: &Path,
    reference: &ResultReference,
) -> Result<native::SessionRecord, HostError> {
    let Locator::Semantic {
        project,
        session_invocation: Some(invocation),
        generation: Some(_),
        source_sha256: Some(_),
        ..
    } = &reference.locator
    else {
        return Err(invalid(
            "native comparison requires original guarded semantic addresses",
        ));
    };
    let environment = std::env::vars_os().collect();
    let record = native::load(root, &environment)
        .map_err(query::Error::from)?
        .ok_or_else(|| {
            query::Error::Unavailable(
                "selected original world is unprepared; prepare it explicitly".into(),
            )
        })?;
    if record.invocation.as_str() != invocation || record.configuration.project != *project {
        return Err(query::Error::Stale(
            "original native session or project differs from selected world".into(),
        )
        .into());
    }
    if record.configuration.backend != native::Backend::NativeSnapshot {
        return Err(invalid(
            "two-world comparison requires explicitly prepared native snapshots",
        ));
    }
    Ok(record)
}
pub(super) async fn read(
    root: &Path,
    reference: &ResultReference,
    token: &tokio_util::sync::CancellationToken,
) -> Result<Side, HostError> {
    if token.is_cancelled() {
        return Err(query::Error::Cancelled.into());
    }
    let record = record(root, reference)?;
    let configuration_identity = record
        .configuration
        .identity()
        .map_err(query::Error::from)?;
    let original = query::rust::semantic_operation_cancellable(
        root,
        &reference.locator,
        native::Kind::Explain,
        token,
    )
    .await?;
    if original
        .session
        .configuration
        .identity()
        .map_err(query::Error::from)?
        != configuration_identity
    {
        return Err(query::Error::Stale(
            "authenticated native answer has different configuration premises".into(),
        )
        .into());
    }
    // The native owner checked source bytes and generation; use the same bytes to nominate
    // lexical correspondence, never the native ID from another database.
    let text = crate::source_read::SourceRequest::Working {
        path: original.source.path().to_owned(),
    }
    .acquire()
    .map_err(|e| query::Error::Io(std::io::Error::other(e)))?;
    if text.identity().source_sha256() != original.source.source_sha256() {
        return Err(query::Error::Stale(
            "selected native source changed before correspondence inspection".into(),
        )
        .into());
    }
    let outline = crate::source_read::rust(text.text(), crate::source_read::RustEdition::E2024);
    let mut enclosing = outline
        .entries
        .iter()
        .filter(|entry| {
            entry.range.byte_start <= original.source_byte_offset
                && original.source_byte_offset < entry.range.byte_end
        })
        .collect::<Vec<_>>();
    enclosing.sort_by_key(|entry| entry.range.byte_end - entry.range.byte_start);
    let declaration_path = enclosing.first().map(|entry| entry.path.clone());
    let declaration_candidates = declaration_path.as_ref().map_or(0, |path| {
        outline
            .entries
            .iter()
            .filter(|entry| entry.path == *path)
            .count()
    });
    Ok(Side {
        reference: reference.clone(),
        checkout: root.to_owned(),
        configuration_identity,
        declaration_path,
        declaration_candidates,
        original,
    })
}
pub(super) async fn package_contexts(
    root: &Path,
    reference: &ResultReference,
    packages: &BTreeSet<String>,
    token: &tokio_util::sync::CancellationToken,
) -> Result<Vec<native::NativePackageContext>, HostError> {
    // Authenticate source/generation too: a world locator is not merely a configuration name.
    let guard = read(root, reference, token).await?;
    let record = record(root, reference)?;
    let environment = std::env::vars_os().collect();
    let mut profiles = Vec::new();
    for package_id in packages {
        let answer = native::client_cancellable(
            root,
            &environment,
            &record,
            &native::Request::PackageContext {
                package_id: package_id.clone(),
            },
            token,
        )
        .await
        .map_err(query::Error::from)?
        .into_package_context()
        .map_err(query::Error::from)?;
        if answer.session.configuration != record.configuration
            || answer.generation != guard.original.generation
        {
            return Err(query::Error::Stale(
                "native package context changed selected configuration or generation".into(),
            )
            .into());
        }
        profiles.extend(answer.packages);
    }
    Ok(profiles)
}
fn addresses(side: &Side) -> (Vec<Address>, Vec<String>) {
    let mut values = BTreeSet::new();
    let mut unknown = Vec::new();
    let address = |relation: String, path: &Path, start: usize, end: usize| {
        let digest = if path == side.original.source.path() {
            Some(side.original.source.source_sha256().to_owned())
        } else {
            side.original
                .location_sources
                .iter()
                .find(|source| source.path == path)
                .map(|source| source.source_sha256.clone())
        };
        digest.map(|source_sha256| Address {
            relation,
            path: path
                .strip_prefix(&side.original.session.configuration.project)
                .unwrap_or(path)
                .to_path_buf()
                .into(),
            start,
            end,
            source_sha256,
        })
    };
    for relation in &side.original.observed.relationships {
        match &relation.location {
            FactLocation::Bytes { path, start, end } => {
                if let Some(value) = address(
                    format!("{:?}/{:?}", relation.relation, relation.direction),
                    path,
                    *start,
                    *end,
                ) {
                    values.insert(value);
                } else {
                    unknown.push("resolved observation lacks original mapped-source bytes".into());
                }
            }
            FactLocation::Protocol { .. } => {
                unknown.push("protocol-only observation has no native byte correspondence".into())
            }
        }
    }
    if let Some(target) = side
        .original
        .native_types
        .as_ref()
        .and_then(|facts| facts.target.as_ref())
    {
        if let Some(location) = &target.location {
            if let Some(value) = address(
                target.relationship.clone(),
                &location.path,
                location.start,
                location.end,
            ) {
                values.insert(value);
            } else {
                unknown.push("native resolved target lacks original mapped-source bytes".into());
            }
        } else {
            unknown.push("native resolved target has no unique original source mapping".into());
        }
    }
    (values.into_iter().collect(), unknown)
}
fn compare(before: Observation<Side>, after: Observation<Side>) -> Comparison {
    let mut context_differences = Vec::new();
    let mut unknown = Vec::new();
    let mut before_targets = Vec::new();
    let mut after_targets = Vec::new();
    let mut changed = None;
    let mut correspondence = "unmatched".to_owned();
    if let (Some(a), Some(b)) = (before.value(), after.value()) {
        if a.declaration_candidates == 1
            && b.declaration_candidates == 1
            && a.declaration_path == b.declaration_path
        {
            correspondence = "selected_declaration_candidate".into();
        } else {
            correspondence = "ambiguous_selected_declarations".into();
            unknown.push(
                "names and ranges do not establish unique cross-world declaration correspondence"
                    .into(),
            );
        }
        let a_config = &a.original.session.configuration;
        let b_config = &b.original.session.configuration;
        if a_config.features != b_config.features {
            context_differences.push("requested_features".into());
        }
        if a_config.no_default_features != b_config.no_default_features {
            context_differences.push("default_features_policy".into());
        }
        if a_config.target != b_config.target {
            context_differences.push("requested_target".into());
        }
        if a_config.test_context != b_config.test_context {
            context_differences.push("test_context".into());
        }
        if a_config.build_macros != b_config.build_macros {
            context_differences.push("macro_preparation".into());
        }
        match (
            a.original
                .native_types
                .as_ref()
                .and_then(|f| f.context.as_ref()),
            b.original
                .native_types
                .as_ref()
                .and_then(|f| f.context.as_ref()),
        ) {
            (Some(a), Some(b)) => {
                if a.features != b.features {
                    context_differences.push("actual_features".into());
                }
                if a.cfg != b.cfg {
                    context_differences.push("actual_cfg".into());
                }
                if a.target != b.target {
                    context_differences.push("actual_target".into());
                }
                if a.edition != b.edition {
                    context_differences.push("edition".into());
                }
                if a.compiler_identity != b.compiler_identity {
                    context_differences.push("compiler_identity".into());
                }
            }
            _ => unknown.push(
                "one or both selected observations lack unique actual native type context".into(),
            ),
        }
        let (a_targets, a_unknown) = addresses(a);
        let (b_targets, b_unknown) = addresses(b);
        before_targets = a_targets;
        after_targets = b_targets;
        unknown.extend(a_unknown);
        unknown.extend(b_unknown);
        if before_targets.is_empty() || after_targets.is_empty() {
            unknown.push("one or both worlds supplied no uniquely mapped selected resolved target observations".into());
        }
        if a.original
            .omitted_relationships
            .values()
            .any(|count| *count > 0)
            || b.original
                .omitted_relationships
                .values()
                .any(|count| *count > 0)
        {
            unknown.push("selected native relationship observations were bounded; omitted records were not compared".into());
        }
        if !a_config.build_macros || !b_config.build_macros {
            unknown.push("macro/build-script preparation is incomplete; unexpanded observations remain unknown".into());
        }
        if correspondence == "selected_declaration_candidate"
            && !before_targets.is_empty()
            && !after_targets.is_empty()
        {
            if before_targets != after_targets {
                changed = Some(true);
            } else if unknown.is_empty() {
                changed = Some(false);
            }
        }
    } else {
        unknown.push("two separately available qualified native worlds are required; no single-world negative comparison".into());
    }
    Comparison{before,after,correspondence,context_differences,before_targets,after_targets,resolved_observation_changed:changed,unknown,limitations:vec!["Only selected Explain-operation resolved target observations were compared. Other declarations, cfg configurations, compiler units, macro expansions and worlds were not examined.".into(),"Matching normalized original target addresses are correspondence candidates, not cross-database HIR identity or semantic dependency equivalence. Type display strings were not compared as identities.".into()]}
}
pub(super) async fn inspect(
    request: &Request,
    before_root: &Path,
    after_root: &Path,
    token: &tokio_util::sync::CancellationToken,
) -> Result<Comparison, HostError> {
    let before = Observation::from_result(read(before_root, &request.before, token).await);
    let after = Observation::from_result(read(after_root, &request.after, token).await);
    if token.is_cancelled() {
        return Err(query::Error::Cancelled.into());
    }
    Ok(compare(before, after))
}
#[cfg(test)]
mod tests {
    use super::*;
    fn side(root: &str, target_start: usize, omitted: usize) -> Side {
        let project = PathBuf::from(root);
        let path = project.join("src/lib.rs");
        let source = crate::source_read::SourceText::new(
            path.clone(),
            "pub fn caller() { target(); }".into(),
        )
        .unwrap();
        let configuration = native::Configuration {
            backend: native::Backend::NativeSnapshot,
            project: project.clone(),
            features: Vec::new(),
            no_default_features: true,
            target: Some("target".into()),
            build_macros: true,
            test_context: false,
        };
        let mut observed = native::ObservedFacts::default();
        observed.relationships.push(native::RelationshipFact {
            relation: native::FactRelation::Definition,
            direction: native::FactDirection::Outgoing,
            location: FactLocation::Bytes {
                path: path.clone(),
                start: target_start,
                end: target_start + 1,
            },
            name: Some("target".into()),
            read: None,
            write: None,
            import: None,
            test: None,
        });
        let original = native::Answer {
            session: native::AnalysisContext {
                backend: "native".into(),
                configuration: configuration.clone(),
                position_encoding: "utf-8".into(),
                backend_status: None,
                preparation: "explicit".into(),
            },
            source: source.identity().clone(),
            source_uri: format!("file://{}", path.display()),
            source_byte_offset: 18,
            availability: native::Availability::Available,
            generation: 1,
            position: native::ProtocolPosition {
                line: 0,
                character: 18,
            },
            observed,
            omitted_relationships: [(native::FactRelation::Definition, omitted)].into(),
            native_types: Some(native::TypedFacts {
                context: Some(native::TypeContext {
                    crate_root: path.clone(),
                    crate_source_sha256: Some(source.identity().source_sha256().into()),
                    features: BTreeSet::new(),
                    cfg: Vec::new(),
                    edition: "2024".into(),
                    target: Some("target".into()),
                    compiler_identity: "same compiler".into(),
                }),
                expression: None,
                original: None,
                adjusted: None,
                original_equals_adjusted: None,
                original_coerces_to_adjusted: None,
                target: None,
                bounds: Vec::new(),
                callsite: None,
                adjustments: Vec::new(),
                captures: Vec::new(),
                capture_owner: None,
                callable_kind: None,
                layout: None,
                callable_candidates: Vec::new(),
            }),
            sections: Vec::new(),
            location_sources: Vec::new(),
            proposal: None,
            limitations: Vec::new(),
        };
        let reference = ResultReference::new(Locator::Semantic {
            project: project.clone(),
            path: path.into(),
            line: 1,
            column: 19,
            generation: Some("1".into()),
            source_sha256: Some(source.identity().source_sha256().into()),
            candidate_fingerprint: None,
            session_invocation: Some(crate::envelope::CliInvocationId::new().to_string()),
        });
        Side {
            reference,
            checkout: project,
            configuration_identity: configuration.identity().unwrap(),
            declaration_path: Some(vec!["caller".into()]),
            declaration_candidates: 1,
            original,
        }
    }
    #[test]
    fn comparison_same_visible_prefix_with_omissions_never_claims_no_change() {
        let comparison = compare(
            Observation::Available {
                value: side("/before", 1, 1),
            },
            Observation::Available {
                value: side("/after", 1, 0),
            },
        );
        assert_eq!(comparison.resolved_observation_changed, None);
        assert!(
            comparison
                .unknown
                .iter()
                .any(|reason| reason.contains("omitted"))
        );
        let changed = compare(
            Observation::Available {
                value: side("/before", 1, 1),
            },
            Observation::Available {
                value: side("/after", 2, 0),
            },
        );
        assert_eq!(changed.resolved_observation_changed, Some(true));
    }
    #[test]
    fn comparison_cfg_only_selected_observations_keep_spelling_and_context_distinct() {
        let before = side("/before", 1, 0);
        let mut after = side("/after", 2, 0);
        after.original.session.configuration.features = vec!["alternate".into()];
        after
            .original
            .native_types
            .as_mut()
            .unwrap()
            .context
            .as_mut()
            .unwrap()
            .cfg = vec!["feature=alternate".into()];
        let comparison = compare(
            Observation::Available { value: before },
            Observation::Available { value: after },
        );
        assert_eq!(comparison.resolved_observation_changed, Some(true));
        assert!(
            comparison
                .context_differences
                .contains(&"actual_cfg".into())
        );
    }
    #[test]
    fn comparison_missing_world_never_establishes_two_world_equality() {
        let missing = || {
            Observation::from_result(Err::<Side, _>(
                query::Error::Unavailable("unprepared".into()).into(),
            ))
        };
        let result = compare(missing(), missing());
        assert_eq!(result.resolved_observation_changed, None);
        assert!(result.partial());
    }
}
