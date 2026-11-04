// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Typed query configuration, independent of host environment interpretation.
use super::{Error, NativePath, Request};
use figment::{
    Figment, Metadata, Profile, Provider,
    providers::{Format, Json, Serialized, Toml},
    value::{Dict, Map as NativeMap},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

/// Repository configuration contains partial defaults and named complete/partial requests.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Configuration {
    defaults: Map<String, Value>,
    queries: BTreeMap<String, Map<String, Value>>,
    scopes: BTreeMap<String, Map<String, Value>>,
}
pse_contract::projection_struct! {
/// The provider that supplied an effective value, before typed extraction loses its tag.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct InputOrigin {
    projection: { name: "InputOrigin", rust_path: crate::investigation::config::InputOrigin, version: 1, direction: pse_contract::BoundaryDirection::Output, reserved_tags: [], reserved_names: [] };
    /// Configuration layer identity, distinct from a selected saved-scope name.
    pub provider: String => { tag: 1, direction: pse_contract::BoundaryDirection::Output, rules: [] };
    /// Original contributing file, preserving native path bytes.
    pub source: Option<NativePath> => { tag: 2, direction: pse_contract::BoundaryDirection::Output, rules: [] };
}
}
/// Explain effective values and the actual contributing configuration sources.
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct Resolved {
    /// The validated effective request used by execution.
    pub request: Request,
    /// Native source provenance for effective top-level keys.
    pub origins: BTreeMap<String, InputOrigin>,
    /// Actual files consumed, without claiming they are immutable evidence.
    pub files: Vec<PathBuf>,
    /// Original safe CLI replay, retained only for execution recovery.
    #[serde(skip)]
    pub explain_action: Option<crate::envelope::NextAction>,
}

/// Resolve only explicitly supplied CLI values over saved input and typed defaults.
pub fn resolve(
    root: &Path,
    config: Option<&Path>,
    name: Option<&str>,
    request: Option<&Path>,
    overrides: Value,
) -> Result<Request, Error> {
    Ok(explain(root, config, name, request, overrides)?.request)
}

/// Resolve and explain without starting a provider, analyzer, compiler or environment evaluator.
pub fn explain(
    root: &Path,
    config: Option<&Path>,
    name: Option<&str>,
    request: Option<&Path>,
    overrides: Value,
) -> Result<Resolved, Error> {
    let root = root.canonicalize()?;
    let mut files = Vec::new();
    let config_path = config
        .map(|path| absolute(&root, path))
        .unwrap_or_else(|| root.join(".config/pse-search.toml"));
    let config_exists = config_path.is_file();
    if config.is_some() && !config_exists {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!(
                "query configuration does not exist: {}",
                config_path.display()
            ),
        )));
    }
    let file_config: Configuration = if config_exists {
        files.push(config_path.clone());
        let provider = file_provider(&config_path);
        provider.extract()?
    } else {
        Configuration::default()
    };
    let base = config_path.parent().unwrap_or(&root);
    let mut defaults = Value::Object(file_config.defaults);
    resolve_paths(&mut defaults, base)?;
    let mut named = if let Some(name) = name {
        Value::Object(
            file_config
                .queries
                .get(name)
                .ok_or_else(|| Error::Invalid(format!("unknown saved query {name}")))?
                .clone(),
        )
    } else {
        Value::Object(Map::new())
    };
    resolve_paths(&mut named, base)?;
    let mut request_value = if let Some(path) = request {
        let path = absolute(&root, path);
        files.push(path.clone());
        let mut value: Value = file_provider(&path).extract()?;
        resolve_paths(&mut value, path.parent().unwrap_or(&root))?;
        value
    } else {
        Value::Object(Map::new())
    };
    let mut overrides = overrides;
    if !overrides.is_object() {
        return Err(Error::Invalid(
            "CLI overrides must be an object of explicitly supplied values".into(),
        ));
    }
    resolve_paths(&mut overrides, &root)?;
    let scope = overrides
        .get("scope")
        .or_else(|| request_value.get("scope"))
        .or_else(|| named.get("scope"))
        .or_else(|| defaults.get("scope"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    let mut scope_layer = if let Some(name) = &scope {
        let selector = Value::Object(
            file_config
                .scopes
                .get(name)
                .ok_or_else(|| Error::Invalid(format!("unknown saved source scope {name}")))?
                .clone(),
        );
        let mut layer = serde_json::json!({"sources": selector});
        resolve_paths(&mut layer, base)?;
        layer
    } else {
        Value::Object(Map::new())
    };
    // Native tagged alternatives replace older alternatives; merging their unlike fields
    // would accidentally construct a hybrid predicate or revision source.
    reset_changed_alternatives(&mut [
        &mut defaults,
        &mut scope_layer,
        &mut named,
        &mut request_value,
        &mut overrides,
    ]);
    let figment = Figment::from(Serialized::defaults(Request::default()))
        .merge(Layer::new(
            defaults,
            "repository defaults",
            Some(config_path.clone()),
        ))
        .merge(Layer::new(
            scope_layer,
            "named source scope",
            Some(config_path.clone()),
        ))
        .merge(Layer::new(named, "saved query", Some(config_path)))
        .merge(Layer::new(
            request_value,
            "request file",
            request.map(|path| absolute(&root, path)),
        ))
        .merge(Layer::new(overrides, "explicit CLI values", None));
    let effective: Request = figment.extract()?;
    effective.validate()?;
    let mut origins = BTreeMap::new();
    // Extract provenance from the same composed configuration as execution. New nested query
    // controls must not require a second, manually maintained list of explainable fields.
    let mut keys = Vec::new();
    provenance_keys(&serde_json::to_value(&effective)?, "", &mut keys);
    for key in keys {
        if let Some(metadata) = figment.find_metadata(&key) {
            origins.insert(
                key,
                InputOrigin {
                    provider: metadata.name.to_string(),
                    source: metadata
                        .source
                        .as_ref()
                        .and_then(figment::Source::file_path)
                        .map(|path| NativePath::from(path.to_path_buf())),
                },
            );
        }
    }
    Ok(Resolved {
        request: effective,
        origins,
        files,
        explain_action: None,
    })
}

impl Resolved {
    /// Array entries share their whole replacement value's contributing provider.
    pub fn origin(&self, key: &str) -> Option<&InputOrigin> {
        let mut key = key;
        loop {
            if let Some(origin) = self.origins.get(key) {
                return Some(origin);
            }
            key = key.rsplit_once('.')?.0;
        }
    }
    /// Reject invalid explicit working roots before any provider or semantic seed work.
    pub fn preflight(&self, root: &Path) -> Result<(), Error> {
        let check = |selector: &super::SourceSelector, key: &str| -> Result<(), Error> {
            if selector.kind != super::SourceKind::Working {
                return Ok(());
            }
            for path in &selector.paths {
                let operand = absolute(root, path);
                if let Err(source) = std::fs::metadata(&operand) {
                    let mut failure = super::SourceFailure::new(
                        super::SourceOperation::SelectRoot,
                        operand,
                        source,
                        false,
                    );
                    failure.recovery.origin = self.origin(key).cloned();
                    failure.recovery.next = self.explain_action.clone();
                    return Err(Error::SourcePath(Box::new(failure)));
                }
            }
            Ok(())
        };
        check(&self.request.sources, "sources.paths")?;
        fn predicates(
            value: &super::Predicate,
            key: &str,
            check: &impl Fn(&super::SourceSelector, &str) -> Result<(), Error>,
        ) -> Result<(), Error> {
            match value {
                super::Predicate::Related { traversal, .. } => {
                    check(traversal, &format!("{key}.traversal.paths"))
                }
                super::Predicate::Refine { candidates, .. } => {
                    predicates(candidates, &format!("{key}.candidates"), check)
                }
                super::Predicate::All { clauses, .. } | super::Predicate::Any { clauses, .. } => {
                    for clause in clauses {
                        predicates(clause, &format!("{key}.clauses"), check)?;
                    }
                    Ok(())
                }
                super::Predicate::Not { clause, .. } => {
                    predicates(clause, &format!("{key}.clause"), check)
                }
                _ => Ok(()),
            }
        }
        predicates(&self.request.predicate, "predicate", &check)
    }
}

fn provenance_keys(value: &Value, prefix: &str, keys: &mut Vec<String>) {
    if let Value::Object(fields) = value {
        for (name, child) in fields {
            let path = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{prefix}.{name}")
            };
            provenance_keys(child, &path, keys);
        }
    } else if !prefix.is_empty() {
        // Arrays replace as a whole; their contributing provider is the meaningful source.
        keys.push(prefix.to_owned());
    }
}

fn absolute(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_owned()
    } else {
        root.join(path)
    }
}
fn reset_changed_alternatives(layers: &mut [&mut Value]) {
    let mut paths = vec![("/sources/kind".into(), "/sources".into(), "kind".into())];
    for layer in layers.iter() {
        if let Some(predicate) = layer.get("predicate") {
            predicate_tag_paths(predicate, "/predicate", &mut paths);
        }
    }
    paths.sort_by(|left, right| {
        left.0
            .len()
            .cmp(&right.0.len())
            .then_with(|| left.cmp(right))
    });
    paths.dedup();
    for (value_path, parent_path, field) in paths {
        let selected = layers
            .iter()
            .rev()
            .find_map(|layer| {
                layer
                    .pointer(&value_path)
                    .and_then(|value| value.get("kind"))
                    .and_then(Value::as_str)
            })
            .map(str::to_owned);
        if let Some(selected) = selected {
            for layer in layers.iter_mut() {
                let differs = layer
                    .pointer(&value_path)
                    .and_then(|value| value.get("kind"))
                    .and_then(Value::as_str)
                    .is_some_and(|kind| kind != selected);
                if differs
                    && let Some(parent) = layer
                        .pointer_mut(&parent_path)
                        .and_then(Value::as_object_mut)
                {
                    parent.remove(&field);
                }
            }
        }
    }
}
fn predicate_tag_paths(value: &Value, path: &str, paths: &mut Vec<(String, String, String)>) {
    if let Some((parent, field)) = path.rsplit_once('/') {
        paths.push((path.into(), parent.into(), field.into()));
    }
    paths.push((
        format!("{path}/traversal/kind"),
        format!("{path}/traversal"),
        "kind".into(),
    ));
    paths.push((format!("{path}/filter"), path.into(), "filter".into()));
    for field in ["candidates", "clause"] {
        if let Some(child) = value.get(field) {
            predicate_tag_paths(child, &format!("{path}/{field}"), paths);
        }
    }
    // Clause/seed arrays replace as whole values in Figment; their entries never merge
    // across layers, so walking array indexes would reset unrelated replacement entries.
}
fn file_provider(path: &Path) -> Figment {
    if path
        .extension()
        .is_some_and(|extension| extension == "json")
    {
        Figment::from(Json::file(path))
    } else {
        Figment::from(Toml::file(path))
    }
}
struct Layer {
    value: Value,
    name: &'static str,
    path: Option<PathBuf>,
}
impl Layer {
    fn new(value: Value, name: &'static str, path: Option<PathBuf>) -> Self {
        Self { value, name, path }
    }
}
impl Provider for Layer {
    fn metadata(&self) -> Metadata {
        let metadata = Metadata::named(self.name);
        self.path
            .as_ref()
            .map_or(metadata.clone(), |path| metadata.source(path.as_path()))
    }
    fn data(&self) -> Result<NativeMap<Profile, Dict>, figment::Error> {
        Serialized::defaults(&self.value).data()
    }
}
fn resolve_paths(layer: &mut Value, root: &Path) -> Result<(), Error> {
    if let Some(paths) = layer
        .pointer_mut("/sources/paths")
        .and_then(Value::as_array_mut)
    {
        for value in paths {
            let path: NativePath = serde_json::from_value(value.clone())?;
            *value = serde_json::to_value(NativePath::from(absolute(root, &path.to_path_buf())))?;
        }
    }
    if let Some(value) = layer.pointer_mut("/predicate/artifact")
        && let Some(path) = value.as_str()
    {
        *value = path_value(root, path)?;
    }
    if let Some(predicate) = layer.get_mut("predicate") {
        resolve_predicate_paths(predicate, root)?;
    }
    Ok(())
}
fn path_value(root: &Path, path: &str) -> Result<Value, Error> {
    Ok(Value::String(
        absolute(root, Path::new(path))
            .into_os_string()
            .into_string()
            .map_err(|_| {
                Error::Invalid("JSON path has a non-UTF8 base; use an exact native locator".into())
            })?,
    ))
}
fn resolve_predicate_paths(predicate: &mut Value, root: &Path) -> Result<(), Error> {
    if let Some(paths) = predicate
        .pointer_mut("/traversal/paths")
        .and_then(Value::as_array_mut)
    {
        for value in paths {
            let path: NativePath = serde_json::from_value(value.clone())?;
            *value = serde_json::to_value(NativePath::from(absolute(root, &path.to_path_buf())))?;
        }
    }
    if let Some(value) = predicate.get_mut("descriptor") {
        let path: NativePath = serde_json::from_value(value.clone())?;
        *value = serde_json::to_value(NativePath::from(absolute(root, &path.to_path_buf())))?;
    }
    if let Some(value) = predicate.get_mut("artifact")
        && let Some(path) = value.as_str()
    {
        *value = path_value(root, path)?;
    }
    if let Some(clauses) = predicate.get_mut("clauses").and_then(Value::as_array_mut) {
        for clause in clauses {
            resolve_predicate_paths(clause, root)?;
        }
    }
    if let Some(clause) = predicate.get_mut("clause") {
        resolve_predicate_paths(clause, root)?;
    }
    if let Some(candidates) = predicate.get_mut("candidates") {
        resolve_predicate_paths(candidates, root)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_symlink_preflight_checks_the_effective_target() {
        use std::os::unix::fs::symlink;
        let root = tempfile::tempdir().unwrap();
        let link = root.path().join("selected.rs");
        let target = root.path().join("target.rs");
        symlink("target.rs", &link).unwrap();
        // Explanation resolves input without observing or acquiring the selected target.
        let resolved = explain(
            root.path(),
            None,
            None,
            None,
            serde_json::json!({"sources":{"paths":["selected.rs"]}}),
        )
        .unwrap();
        let Error::SourcePath(failure) = resolved.preflight(root.path()).unwrap_err() else {
            panic!("lost target selection cause")
        };
        assert_eq!(
            failure.recovery.cause,
            crate::envelope::RecoveryCause::MissingRoot
        );
        assert_eq!(failure.recovery.operand.unwrap().to_path_buf(), link);
        std::fs::write(&target, "fn target() {}\n").unwrap();
        resolved.preflight(root.path()).unwrap();
        let source = crate::source_read::SourceRequest::Working { path: link.clone() };
        let acquired = super::super::providers::acquire(root.path(), &source).unwrap();
        assert_eq!(acquired.bytes, b"fn target() {}\n");
        std::fs::remove_file(&target).unwrap();
        assert!(std::fs::symlink_metadata(&link).is_ok());
        let error = super::super::providers::acquire(root.path(), &source).unwrap_err();
        assert_eq!(error.disposition(), super::super::FailureDisposition::Stale);
        let recovery = error.query_error_record().unwrap().recovery.unwrap();
        assert_eq!(
            recovery.cause,
            crate::envelope::RecoveryCause::SourceDisappeared
        );
        assert_eq!(recovery.operand.unwrap().to_path_buf(), link);
    }
    #[test]
    fn missing_request_root_retains_origin_and_passive_resolution() {
        let root = tempfile::tempdir().unwrap();
        let nested = root.path().join("input");
        std::fs::create_dir(&nested).unwrap();
        let request = nested.join("request.json");
        std::fs::write(&request, r#"{"sources":{"paths":["missing"]}}"#).unwrap();
        // Resolution/explanation succeeds; preflight is a separate execution-only step.
        let mut resolved = explain(
            root.path(),
            None,
            None,
            Some(&request),
            serde_json::json!({}),
        )
        .unwrap();
        resolved.explain_action = Some(crate::envelope::NextAction::Command {
            argv: vec![
                "pse".into(),
                "search".into(),
                "--request".into(),
                request.to_str().unwrap().into(),
                "--explain".into(),
            ],
            reason: "inspect inputs".into(),
        });
        let Error::SourcePath(error) = resolved.preflight(root.path()).unwrap_err() else {
            panic!("lost selection cause")
        };
        assert_eq!(
            error.recovery.cause,
            crate::envelope::RecoveryCause::MissingRoot
        );
        assert_eq!(
            error.recovery.operand.unwrap().to_path_buf(),
            nested.join("missing")
        );
        let origin = error.recovery.origin.unwrap();
        assert_eq!(origin.provider, "request file");
        assert_eq!(origin.source.unwrap().to_path_buf(), request);
        assert!(error.recovery.next.is_some());
        let absolute = root.path().join("absolute-missing");
        let resolved = explain(
            root.path(),
            None,
            None,
            None,
            serde_json::json!({"sources":{"paths":[absolute]}}),
        )
        .unwrap();
        assert_eq!(resolved.request.sources.paths, vec![absolute]);
        assert_eq!(
            resolved.origins["sources.paths"].provider,
            "explicit CLI values"
        );
    }
    #[test]
    fn named_scope_identity_and_kind_reset_are_independent_of_origin() {
        let root = tempfile::tempdir().unwrap();
        let config = root.path().join("queries.json");
        std::fs::write(&config, r#"{"scopes":{"old":{"kind":{"kind":"revision","revision":"obsolete"},"paths":["old"]}},"defaults":{"scope":"old"}}"#).unwrap();
        let resolved = explain(root.path(), Some(&config), None, None, serde_json::json!({"sources":{"kind":{"kind":"working"},"paths":["new"],"include_hidden":false}})).unwrap();
        assert_eq!(resolved.request.scope.as_deref(), Some("old"));
        assert_eq!(resolved.origins["scope"].provider, "repository defaults");
        assert_eq!(
            resolved.request.sources.kind,
            super::super::SourceKind::Working
        );
        assert_eq!(
            resolved.request.sources.paths,
            vec![root.path().join("new")]
        );
        assert_eq!(
            resolved.origins["sources.paths"].provider,
            "explicit CLI values"
        );
        assert!(!resolved.request.sources.include_hidden);
        assert_eq!(
            resolved.origins["sources.include_hidden"].provider,
            "explicit CLI values"
        );
    }
    #[test]
    fn schema_descriptors_resolve_at_request_config_and_override_origins() {
        let root = tempfile::tempdir().unwrap();
        let nested = root.path().join("nested");
        std::fs::create_dir(&nested).unwrap();
        let path = serde_json::to_value(NativePath::from(PathBuf::from("schema.bin"))).unwrap();
        let predicate = serde_json::json!({"kind":"schema","descriptor":path,"query":"Payload"});
        let request = nested.join("request.json");
        let configured = nested.join("saved.json");
        std::fs::write(
            &request,
            serde_json::to_vec(&serde_json::json!({"predicate":predicate})).unwrap(),
        )
        .unwrap();
        std::fs::write(
            &configured,
            serde_json::to_vec(&serde_json::json!({"queries":{"saved":{"predicate":predicate}}}))
                .unwrap(),
        )
        .unwrap();
        for (config, name, input) in [
            (None, None, Some(request.as_path())),
            (Some(configured.as_path()), Some("saved"), None),
        ] {
            let resolved =
                resolve(root.path(), config, name, input, serde_json::json!({})).unwrap();
            let super::super::Predicate::Schema(query) = resolved.predicate else {
                panic!("lost schema predicate");
            };
            assert_eq!(query.descriptor.to_path_buf(), nested.join("schema.bin"));
            let resolved = resolve(
                root.path(),
                config,
                name,
                input,
                serde_json::json!({"predicate":predicate}),
            )
            .unwrap();
            let super::super::Predicate::Schema(query) = resolved.predicate else {
                panic!("lost schema override");
            };
            assert_eq!(
                query.descriptor.to_path_buf(),
                root.path().join("schema.bin")
            );
        }
        let mut grouped = serde_json::json!({"kind":"any","clauses":[predicate]});
        resolve_predicate_paths(&mut grouped, &nested).unwrap();
        let path: NativePath =
            serde_json::from_value(grouped["clauses"][0]["descriptor"].clone()).unwrap();
        assert_eq!(path.to_path_buf(), nested.join("schema.bin"));
    }
    #[test]
    fn nested_refinement_candidates_keep_their_request_and_configuration_directory() {
        let root = tempfile::tempdir().unwrap();
        let nested = root.path().join("nested");
        std::fs::create_dir(&nested).unwrap();
        let predicate = serde_json::json!({"kind":"refine","candidates":{"kind":"api","query":"increment","artifact":"api/document.json"},"filter":{"kind":"native_predicate","text":"Copy"}});
        let request = nested.join("request.json");
        std::fs::write(
            &request,
            serde_json::to_vec(&serde_json::json!({"predicate":predicate})).unwrap(),
        )
        .unwrap();
        let configured = nested.join("saved.json");
        std::fs::write(
            &configured,
            serde_json::to_vec(&serde_json::json!({"queries":{"saved":{"predicate":predicate}}}))
                .unwrap(),
        )
        .unwrap();
        for resolved in [
            resolve(
                root.path(),
                None,
                None,
                Some(&request),
                serde_json::json!({}),
            )
            .unwrap(),
            resolve(
                root.path(),
                Some(&configured),
                Some("saved"),
                None,
                serde_json::json!({}),
            )
            .unwrap(),
        ] {
            let super::super::Predicate::Refine { candidates, .. } = resolved.predicate else {
                panic!("lost refined request");
            };
            let super::super::Predicate::Api(query) = *candidates else {
                panic!("lost native candidate provider");
            };
            assert_eq!(query.artifact, Some(nested.join("api/document.json")));
        }
    }
    #[test]
    fn related_seed_arrays_replace_and_traversal_selectors_merge_with_provenance() {
        use super::super::{Locator, Predicate, ResultReference, SourceKind};
        let root = tempfile::tempdir().unwrap();
        let reference = |name: &str| {
            ResultReference::new(Locator::Source {
                path: PathBuf::from(name).into(),
                source: SourceKind::Working,
                range: None,
                guard: None,
            })
        };
        let first = reference("first.rs");
        let second = reference("second.rs");
        let config = root.path().join("query.json");
        std::fs::write(
            &config,
            serde_json::to_vec(&serde_json::json!({"defaults": {
                "predicate":{"kind":"related","seeds":[first],"relations":["uses"],
                    "traversal":{"paths":["src"],"types":["rust"]}}
            }}))
            .unwrap(),
        )
        .unwrap();
        let resolved = explain(
            root.path(),
            Some(&config),
            None,
            None,
            serde_json::json!({"predicate":{"seeds":[second],"traversal":{"include_hidden":true}}}),
        )
        .unwrap();
        let Predicate::Related {
            seeds, traversal, ..
        } = resolved.request.predicate
        else {
            panic!("lost relationship");
        };
        assert_eq!(seeds.len(), 1);
        assert_eq!(
            serde_json::to_value(&seeds[0]).unwrap(),
            serde_json::to_value(&second).unwrap()
        );
        assert_eq!(traversal.paths, vec![root.path().join("src")]);
        assert_eq!(traversal.types, vec!["rust"]);
        assert!(traversal.include_hidden);
        assert_eq!(
            resolved.origins["predicate.seeds"].provider,
            "explicit CLI values"
        );
        assert_eq!(
            resolved.origins["predicate.traversal.paths"].provider,
            "repository defaults"
        );
    }
    #[test]
    fn nested_relationship_domain_tag_switch_replaces_the_original_alternative() {
        use super::super::{Locator, ResultReference, SourceKind};
        let root = tempfile::tempdir().unwrap();
        let seed = ResultReference::new(Locator::Source {
            path: PathBuf::from("src/lib.rs").into(),
            source: SourceKind::Working,
            range: None,
            guard: None,
        });
        let config = root.path().join("query.json");
        std::fs::write(&config, serde_json::to_vec(&serde_json::json!({"defaults": {
            "predicate":{"kind":"refine","filter":{"kind":"native_predicate","text":"Copy"},
                "candidates":{"kind":"related","seeds":[seed],"relations":["uses"],
                    "traversal":{"kind":{"kind":"revision","revision":"obsolete"},"types":["rust"]}}}
        }})).unwrap()).unwrap();
        let resolved = resolve(root.path(), Some(&config), None, None,
            serde_json::json!({"predicate":{"candidates":{"traversal":{"kind":{"kind":"working"}}}}})).unwrap();
        let value = serde_json::to_value(resolved).unwrap();
        assert_eq!(
            value["predicate"]["candidates"]["traversal"]["kind"],
            serde_json::json!({"kind":"working"})
        );
        assert_eq!(
            value["predicate"]["candidates"]["traversal"]["types"],
            serde_json::json!(["rust"])
        );
    }
    #[test]
    fn explicit_overrides_replace_arrays_without_implicit_cli_defaults() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join(".config")).unwrap();
        std::fs::write(root.path().join(".config/pse-search.toml"), "[defaults.answer]\nlimit = 7\n[defaults.sources]\nglobs = ['*.rs', '*.toml']\n[queries.timeout.predicate]\nkind = 'text'\npatterns = ['timeout']\nliteral = true\n").unwrap();
        let request = resolve(
            root.path(),
            None,
            Some("timeout"),
            None,
            serde_json::json!({"sources":{"globs":["*.rs"]}}),
        )
        .unwrap();
        assert_eq!(request.answer.limit, Some(7));
        assert_eq!(request.sources.globs, vec!["*.rs"]);
        assert!(matches!(
            request.predicate,
            super::super::Predicate::Text(_)
        ));
    }
    #[test]
    fn nested_provenance_preserves_explicit_false_and_independent_detail_controls() {
        let root = tempfile::tempdir().unwrap();
        let config = root.path().join("query.json");
        std::fs::write(&config, r#"{"defaults":{"sources":{"include_hidden":true},"predicate":{"kind":"text","patterns":["x"],"literal":true},"presentation":{"context":3}}}"#).unwrap();
        let resolved = explain(
            root.path(),
            Some(&config),
            None,
            None,
            serde_json::json!({"sources":{"include_hidden":false},"predicate":{"literal":false}}),
        )
        .unwrap();
        assert!(!resolved.request.sources.include_hidden);
        assert_eq!(
            resolved.origins["sources.include_hidden"].provider,
            "explicit CLI values"
        );
        assert_eq!(
            resolved.origins["predicate.literal"].provider,
            "explicit CLI values"
        );
        assert_eq!(
            resolved.origins["predicate.patterns"].provider,
            "repository defaults"
        );
        assert_eq!(
            resolved.origins["presentation.context"].provider,
            "repository defaults"
        );
    }
    #[test]
    fn request_relative_paths_and_scopes_resolve_at_their_own_source() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("queries");
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(
            dir.join("request.json"),
            r#"{"sources":{"paths":["local.rs"]}}"#,
        )
        .unwrap();
        let request = resolve(
            root.path(),
            None,
            None,
            Some(Path::new("queries/request.json")),
            serde_json::json!({}),
        )
        .unwrap();
        assert_eq!(request.sources.paths, vec![dir.join("local.rs")]);
        assert!(
            resolve(
                root.path(),
                None,
                Some("missing"),
                None,
                serde_json::json!({})
            )
            .is_err()
        );
    }
    #[test]
    fn unsupported_keys_and_arrival_cursor_are_not_silently_ignored() {
        let root = tempfile::tempdir().unwrap();
        assert!(resolve(root.path(), None, None, None, serde_json::json!({"typo":1})).is_err());
        assert!(
            resolve(
                root.path(),
                None,
                None,
                None,
                serde_json::json!({"answer":{"cursor":"old"}})
            )
            .is_err()
        );
    }
    #[test]
    fn native_alternatives_do_not_inherit_incompatible_older_fields() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(
            root.path().join("config.json"),
            r#"{"defaults":{"predicate":{"kind":"text","patterns":["old"]}}}"#,
        )
        .unwrap();
        let request = resolve(
            root.path(),
            Some(Path::new("config.json")),
            None,
            None,
            serde_json::json!({"predicate":{"kind":"symbol","name":"Host"}}),
        )
        .unwrap();
        assert!(matches!(
            request.predicate,
            super::super::Predicate::Symbol(_)
        ));
    }
}
