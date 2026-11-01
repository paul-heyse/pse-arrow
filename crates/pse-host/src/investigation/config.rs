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
/// Explain effective values and the actual contributing configuration sources.
#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct Resolved {
    /// The validated effective request used by execution.
    pub request: Request,
    /// Native source provenance for effective top-level keys.
    pub origins: BTreeMap<String, String>,
    /// Actual files consumed, without claiming they are immutable evidence.
    pub files: Vec<PathBuf>,
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
    let scope_layer = if let Some(name) = &scope {
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
                format!(
                    "{}{}",
                    metadata.name,
                    metadata
                        .source
                        .as_ref()
                        .map(|source| format!(" ({source})"))
                        .unwrap_or_default()
                ),
            );
        }
    }
    if let Some(scope) = scope {
        origins.insert("scope".into(), scope);
    }
    Ok(Resolved {
        request: effective,
        origins,
        files,
    })
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
    for (value_path, parent_path, field) in [
        ("/predicate", "", "predicate"),
        ("/sources/kind", "/sources", "kind"),
    ] {
        let selected = layers
            .iter()
            .rev()
            .find_map(|layer| {
                layer
                    .pointer(value_path)
                    .and_then(|value| value.get("kind"))
                    .and_then(Value::as_str)
            })
            .map(str::to_owned);
        if let Some(selected) = selected {
            for layer in layers.iter_mut() {
                let differs = layer
                    .pointer(value_path)
                    .and_then(|value| value.get("kind"))
                    .and_then(Value::as_str)
                    .is_some_and(|kind| kind != selected);
                if differs
                    && let Some(parent) = layer
                        .pointer_mut(parent_path)
                        .and_then(Value::as_object_mut)
                {
                    parent.remove(field);
                }
            }
        }
    }
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
        assert!(resolved.origins["sources.include_hidden"].contains("explicit CLI values"));
        assert!(resolved.origins["predicate.literal"].contains("explicit CLI values"));
        assert!(resolved.origins["predicate.patterns"].contains("repository defaults"));
        assert!(resolved.origins["presentation.context"].contains("repository defaults"));
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
