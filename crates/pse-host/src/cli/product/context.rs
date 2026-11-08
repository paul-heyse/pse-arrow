// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Checkout-owned selection of already configured product targets.
use crate::{
    HostError,
    envelope::{Envelope, OutputOptions, RedactionPolicy},
    environment::Environment,
    investigation::NativePath,
    paths::{CheckoutState, HostPaths, UserDirectory},
};
use clap::Args;
use figment::{
    Figment, Metadata, Profile, Provider,
    providers::{Format, Serialized, Toml},
    value::{Dict, Map},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::OpenOptions,
    io::{self, Read as _},
    num::NonZeroU64,
    os::unix::fs::{MetadataExt as _, OpenOptionsExt as _},
    path::{Path, PathBuf},
};

const MAXIMUM_BYTES: usize = 64 * 1024;
const MAXIMUM_TARGETS: usize = 16;

#[derive(Debug, Default, Args)]
pub(in crate::cli) struct Options {
    /// Exact private CLI settings file, relative to the selected checkout.
    #[arg(long)]
    context_file: Option<PathBuf>,
    /// Named settings profile from the selected file.
    #[arg(long)]
    context_profile: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Configuration {
    defaults: Preferences,
    profiles: BTreeMap<String, Preferences>,
    targets: BTreeMap<String, ConfiguredTarget>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
struct Preferences {
    #[serde(skip_serializing_if = "Option::is_none")]
    target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timeout_ms: Option<NonZeroU64>,
}

struct Layer<'a> {
    values: &'a Preferences,
    name: &'static str,
}
impl Provider for Layer<'_> {
    fn metadata(&self) -> Metadata {
        Metadata::named(self.name)
    }
    fn data(&self) -> Result<Map<Profile, Dict>, figment::Error> {
        Serialized::defaults(self.values).data()
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfiguredTarget {
    state: PathBuf,
    database: String,
    issuing_store: String,
    reader: String,
}

#[derive(Debug)]
pub(super) struct ResolvedSource {
    pub(super) state: PathBuf,
    pub(super) database: Option<String>,
    pub(super) issuing_store: Option<String>,
    pub(super) reader: String,
    pub(super) timeout_ms: Option<NonZeroU64>,
}

pub(super) struct Context {
    checkout: String,
    file: Option<PathBuf>,
    profile: Option<String>,
    preferences: Preferences,
    origins: BTreeMap<&'static str, String>,
    targets: BTreeMap<String, ConfiguredTarget>,
}

fn invalid(message: &str) -> HostError {
    HostError::Ownership(message.into())
}

fn acquire(path: &Path, required: bool) -> Result<Option<String>, HostError> {
    let file = match OpenOptions::new()
        .read(true)
        .custom_flags(
            rustix::fs::OFlags::NOFOLLOW.bits() as i32 | rustix::fs::OFlags::NONBLOCK.bits() as i32,
        )
        .open(path)
    {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound && !required => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let metadata = file.metadata()?;
    if !metadata.is_file()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o077 != 0
        || metadata.len() > MAXIMUM_BYTES as u64
    {
        return Err(invalid(
            "CLI settings require a bounded private regular file owned by the current user",
        ));
    }
    let mut text = String::new();
    file.take((MAXIMUM_BYTES + 1) as u64)
        .read_to_string(&mut text)?;
    if text.len() > MAXIMUM_BYTES {
        return Err(invalid("CLI settings exceed the file bound"));
    }
    Ok(Some(text))
}

fn name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
}

impl Context {
    pub(super) fn load(
        root: &Path,
        environment: &Environment,
        options: &Options,
        target: Option<String>,
        timeout_ms: Option<NonZeroU64>,
    ) -> Result<Self, HostError> {
        let checkout = CheckoutState::resolve(root, environment)?;
        let path = match &options.context_file {
            Some(path) => root.join(path),
            None => HostPaths::new(environment)
                .base(UserDirectory::Config)?
                .join("pse-arrow")
                .join(checkout.checkout_id.as_str())
                .join("cli.toml"),
        };
        if !path.is_absolute() {
            return Err(invalid(
                "CLI settings require an absolute checkout-owned path",
            ));
        }
        let text = acquire(&path, options.context_file.is_some())?;
        let configuration: Configuration = match &text {
            Some(text) => Figment::from(Toml::string(text))
                .extract()
                .map_err(|_| invalid("CLI settings file contains an invalid field or value"))?,
            None => Configuration::default(),
        };
        let Configuration {
            defaults,
            profiles,
            targets,
        } = configuration;
        if targets.len() > MAXIMUM_TARGETS
            || profiles.len() > MAXIMUM_TARGETS
            || targets.iter().any(|(key, target)| {
                !name(key)
                    || !target.state.is_absolute()
                    || target.database.is_empty()
                    || target.database.len() > 128
                    || !target
                        .database
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
                    || target.issuing_store.is_empty()
                    || target.issuing_store.len() > 128
                    || target.reader.len() != 36
                    || uuid::Uuid::parse_str(&target.reader).is_err()
            })
            || profiles.keys().any(|key| !name(key))
        {
            return Err(invalid(
                "CLI settings exceed target bounds or contain an invalid target identity",
            ));
        }
        let profile = options
            .context_profile
            .as_ref()
            .map(|profile| {
                profiles
                    .get(profile)
                    .ok_or_else(|| invalid("selected CLI settings profile is unknown"))
            })
            .transpose()?;
        let mut layers = Figment::from(Serialized::defaults(Preferences::default())).merge(Layer {
            values: &defaults,
            name: "approved settings defaults",
        });
        if let Some(profile) = profile {
            layers = layers.merge(Layer {
                values: profile,
                name: "selected profile",
            });
        }
        layers = layers.merge(Layer {
            values: &Preferences { target, timeout_ms },
            name: "explicit CLI input",
        });
        let preferences: Preferences = layers
            .extract()
            .map_err(|_| invalid("effective CLI settings are invalid"))?;
        if preferences
            .timeout_ms
            .is_some_and(|timeout| timeout.get() > 25_000)
        {
            return Err(invalid(
                "project timeout requires 1 through 25000 milliseconds",
            ));
        }
        if preferences
            .target
            .as_ref()
            .is_some_and(|target| !targets.contains_key(target))
        {
            return Err(invalid(
                "selected product target is unknown in these CLI settings",
            ));
        }
        let origins = ["target", "timeout_ms"]
            .into_iter()
            .filter_map(|field| Some((field, layers.find_metadata(field)?.name.to_string())))
            .collect();
        Ok(Self {
            checkout: checkout.checkout_id.as_str().into(),
            file: text.map(|_| path),
            profile: options.context_profile.clone(),
            preferences,
            origins,
            targets,
        })
    }

    fn selected_name(&self) -> Option<&str> {
        self.preferences.target.as_deref().or_else(|| {
            (self.targets.len() == 1)
                .then(|| self.targets.keys().next().map(String::as_str))
                .flatten()
        })
    }

    pub(super) fn selected(&self) -> Option<ResolvedSource> {
        self.selected_name()
            .and_then(|name| self.targets.get(name))
            .map(|target| self.resolve(target))
    }

    pub(super) fn candidates(&self) -> impl Iterator<Item = (&str, ResolvedSource)> {
        self.targets
            .iter()
            .map(|(name, target)| (name.as_str(), self.resolve(target)))
    }

    fn resolve(&self, target: &ConfiguredTarget) -> ResolvedSource {
        ResolvedSource {
            state: target.state.clone(),
            database: Some(target.database.clone()),
            issuing_store: Some(target.issuing_store.clone()),
            reader: target.reader.clone(),
            timeout_ms: self.preferences.timeout_ms,
        }
    }
}

#[derive(Serialize, JsonSchema)]
pub(in crate::cli) struct View {
    checkout: String,
    settings_file: Option<String>,
    profile: Option<String>,
    configured_targets: Vec<String>,
    selected_target: Option<String>,
    selected_deployment: Option<DeploymentView>,
    timeout_ms: u64,
    origins: BTreeMap<&'static str, String>,
}

#[derive(Serialize, JsonSchema)]
struct DeploymentView {
    state: NativePath,
    database: String,
    issuing_store: String,
    reader: String,
    origin: &'static str,
}

pub(in crate::cli) fn explain(
    root: &Path,
    options: &Options,
    target: Option<String>,
    output: &OutputOptions,
) -> Result<i32, HostError> {
    let environment = std::env::vars_os().collect();
    let context = Context::load(&root.canonicalize()?, &environment, options, target, None)?;
    let policy = RedactionPolicy::from_inputs(&environment);
    let selected_target = context.selected_name().map(|value| policy.redact(value));
    let selected_deployment = context
        .selected_name()
        .and_then(|name| context.targets.get(name))
        .map(|target| DeploymentView {
            state: policy.redact_path(target.state.clone().into()),
            database: policy.redact(&target.database),
            issuing_store: policy.redact(&target.issuing_store),
            reader: policy.redact(&target.reader),
            origin: "approved settings target definition",
        });
    let configured_targets = context
        .targets
        .keys()
        .map(|name| policy.redact(name))
        .collect();
    let mut origins = context.origins;
    if selected_target.is_some() {
        origins
            .entry("target")
            .or_insert_with(|| "sole configured target".into());
    }
    origins
        .entry("timeout_ms")
        .or_insert_with(|| "product owner default".into());
    let mut response = Envelope::response(
        "inspect settings",
        "effective CLI settings; target identity has not been observed",
        View {
            checkout: context.checkout,
            settings_file: context
                .file
                .map(|file| policy.redact(&file.to_string_lossy())),
            profile: context.profile.map(|value| policy.redact(&value)),
            configured_targets,
            selected_target,
            selected_deployment,
            timeout_ms: context
                .preferences
                .timeout_ms
                .map_or(25_000, NonZeroU64::get),
            origins,
        },
    );
    response.invocation_id = output.invocation_id.clone();
    Ok(crate::envelope::write(
        io::stdout().lock(),
        response,
        output.format,
        &output.presentation,
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as _;

    fn checkout() -> &'static Path {
        Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
    }

    #[test]
    fn project_context_resolves_once_and_attributes_explicit_and_profile_values() {
        let state = tempfile::tempdir().unwrap();
        let environment = Environment::from([("XDG_STATE_HOME".into(), state.path().into())]);
        let mut settings = tempfile::NamedTempFile::new().unwrap();
        write!(
            settings,
            r#"
[defaults]
target = "local.a"
timeout_ms = 100
[profiles.operator]
target = "local.b"
timeout_ms = 200
[targets."local.a"]
state = "/fixtures/a"
database = "default"
issuing_store = "issuer-a"
reader = "4c4dd34e-4a26-40f8-8737-aa16df9f9b2f"
[targets."local.b"]
state = "/fixtures/b"
database = "qualified"
issuing_store = "issuer-b"
reader = "4c4dd34e-4a26-40f8-8737-aa16df9f9b2f"
"#
        )
        .unwrap();
        let options = Options {
            context_file: Some(settings.path().to_path_buf()),
            context_profile: Some("operator".into()),
        };
        let context = Context::load(
            checkout(),
            &environment,
            &options,
            Some("local.a".into()),
            None,
        )
        .unwrap();
        let target = context.selected().unwrap();
        assert_eq!(target.state, Path::new("/fixtures/a"));
        assert_eq!(target.issuing_store.as_deref(), Some("issuer-a"));
        assert_eq!(target.timeout_ms.map(NonZeroU64::get), Some(200));
        assert_eq!(context.origins["target"], "explicit CLI input");
        assert_eq!(context.origins["timeout_ms"], "selected profile");
    }

    #[test]
    fn explicit_missing_and_malformed_settings_refuse() {
        let state = tempfile::tempdir().unwrap();
        let environment = Environment::from([("XDG_STATE_HOME".into(), state.path().into())]);
        let options = Options {
            context_file: Some(state.path().join("missing")),
            context_profile: None,
        };
        assert!(Context::load(checkout(), &environment, &options, None, None).is_err());
        let mut settings = tempfile::NamedTempFile::new().unwrap();
        settings
            .write_all(b"[defaults]\nunknown_field = 17\n")
            .unwrap();
        let options = Options {
            context_file: Some(settings.path().to_path_buf()),
            context_profile: None,
        };
        assert!(Context::load(checkout(), &environment, &options, None, None).is_err());
    }
}
