// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact local Surrealism artifact admission and installed-module attestation.

use crate::canonical::{CanonicalError, CanonicalStore, MetadataResponse, bounded_query};
use crate::canonical_script::UnframedResult;
#[cfg(feature = "canonical-maintenance")]
use crate::canonical_script::{Envelope, GuardedScript};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    io::Read,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StudyModuleReceipt {
    kind: String,
    artifact: PathBuf,
    pub(crate) sha256: String,
    bytes: u64,
    sdk_version: String,
    abi: u32,
    inputs: serde_json::Map<String, serde_json::Value>,
    limits: ModuleLimits,
}
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct ModuleLimits {
    max_memory_bytes: u64,
    max_pool_size: u32,
    max_execution_time_ms: u64,
    max_kv_entries: u64,
    max_kv_value_bytes: u64,
    strict_timeout: bool,
}
struct ModuleDeclaration {
    abi: u32,
    sdk: String,
    limits: ModuleLimits,
}
impl ModuleDeclaration {
    fn read() -> Result<Self, CanonicalError> {
        Self::from_manifest(include_str!("../../pse-study-module/surrealism.toml"))
    }
    fn from_manifest(source: &str) -> Result<Self, CanonicalError> {
        let manifest: toml::Value = toml::from_str(source).map_err(invalid)?;
        let abi = manifest
            .get("abi")
            .and_then(toml::Value::as_integer)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| invalid("declared ABI absent"))?;
        let mut capabilities = manifest
            .get("capabilities")
            .and_then(toml::Value::as_table)
            .cloned()
            .ok_or_else(|| invalid("declared capabilities absent"))?;
        for name in ["allow_scripting", "allow_arbitrary_queries"] {
            if capabilities.remove(name) != Some(toml::Value::Boolean(false)) {
                return Err(invalid("pure module cannot enable host side effects"));
            }
        }
        for name in ["allow_functions", "allow_net"] {
            if capabilities.remove(name) != Some(toml::Value::Array(Vec::new())) {
                return Err(invalid("pure module cannot enable host attachments"));
            }
        }
        let time = capabilities
            .remove("max_execution_time")
            .ok_or_else(|| invalid("declared execution limit absent"))?;
        capabilities.insert("max_execution_time_ms".into(), time);
        let limits: ModuleLimits = toml::Value::Table(capabilities)
            .try_into()
            .map_err(invalid)?;
        let workspace: toml::Value =
            toml::from_str(include_str!("../../../Cargo.toml")).map_err(invalid)?;
        let sdk = workspace
            .get("workspace")
            .and_then(|value| value.get("dependencies"))
            .and_then(|value| value.get("surrealism"))
            .and_then(toml::Value::as_str)
            .and_then(|value| value.strip_prefix('='))
            .filter(|value| !value.is_empty())
            .ok_or_else(|| invalid("exact workspace SDK pin absent"))?
            .to_owned();
        Ok(Self { abi, sdk, limits })
    }
}
fn invalid(message: impl std::fmt::Display) -> CanonicalError {
    CanonicalError::Configuration(format!("study module: {message}"))
}
impl StudyModuleReceipt {
    /// The same selected module limits constrain its server runtime. These are
    /// guest-policy limits, independent of native host workload placement.
    #[cfg(feature = "native-service")]
    pub(crate) fn runtime_environment(&self) -> Vec<(String, String)> {
        [
            (
                "SURREAL_SURREALISM_MAX_MEMORY",
                self.limits.max_memory_bytes.to_string(),
            ),
            (
                "SURREAL_SURREALISM_MAX_EXECUTION_TIME",
                self.limits.max_execution_time_ms.to_string(),
            ),
            (
                "SURREAL_SURREALISM_MAX_KV_ENTRIES",
                self.limits.max_kv_entries.to_string(),
            ),
            (
                "SURREAL_SURREALISM_MAX_KV_VALUE_BYTES",
                self.limits.max_kv_value_bytes.to_string(),
            ),
            ("SURREAL_SURREALISM_MAX_FS_BYTES", "0".into()),
            ("SURREAL_SURREALISM_LOG_LEVEL", "warn".into()),
        ]
        .into_iter()
        .map(|(key, value)| (key.into(), value))
        .collect()
    }
    /// Copy the already admitted bytes, preserving every original build input.
    /// Only the private artifact location changes; this never claims a rebuild.
    #[cfg(feature = "native-service")]
    pub(crate) fn copy_into_private(
        &self,
        state: &Path,
        directory: &Path,
    ) -> Result<Self, CanonicalError> {
        use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
        let owner = std::fs::metadata(state).map_err(invalid)?;
        if !owner.is_dir()
            || owner.mode() & 0o077 != 0
            || owner.uid() != rustix::process::geteuid().as_raw()
        {
            return Err(invalid(
                "module destination state must be private and owned by the current user",
            ));
        }
        let module_root = state.join("modules");
        let nested_private_module = directory.parent() == Some(module_root.as_path())
            && std::fs::canonicalize(&module_root).map_err(invalid)? == module_root;
        if (directory.parent() != Some(state) && !nested_private_module)
            || std::fs::canonicalize(state).map_err(invalid)? != state
        {
            return Err(invalid("module copy must be inside its new private state"));
        }
        self.validate(state)?;
        std::fs::create_dir(directory).map_err(invalid)?;
        std::fs::set_permissions(directory, std::fs::Permissions::from_mode(0o700))
            .map_err(invalid)?;
        let destination = directory.join("study.surli");
        let mut source = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(&self.artifact)
            .map_err(invalid)?;
        let mut output = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(&destination)
            .map_err(invalid)?;
        let copied = std::io::copy(&mut source.by_ref().take(self.bytes + 1), &mut output)
            .map_err(invalid)?;
        if copied != self.bytes {
            return Err(invalid(
                "original module extent changed during private copy",
            ));
        }
        output.sync_all().map_err(invalid)?;
        std::fs::set_permissions(&destination, std::fs::Permissions::from_mode(0o400))
            .map_err(invalid)?;
        std::fs::File::open(directory)
            .and_then(|directory| directory.sync_all())
            .map_err(invalid)?;
        let mut selected = self.clone();
        selected.artifact = destination;
        selected.validate(state)?;
        Ok(selected)
    }
    pub(crate) fn read(state: &Path) -> Result<Self, CanonicalError> {
        #[derive(Deserialize)]
        struct Selection {
            study_module: StudyModuleReceipt,
        }
        let bytes = std::fs::read(state.join("config.json")).map_err(invalid)?;
        let receipt: Selection = serde_json::from_slice(&bytes).map_err(invalid)?;
        receipt.study_module.validate(state)?;
        Ok(receipt.study_module)
    }
    pub(crate) fn read_migration_source(state: &Path, expected: &str) -> Result<Self, CanonicalError> {
        let selected = Self::read(state)?;
        if selected.sha256 == expected { return Ok(selected); }
        #[derive(Deserialize)]
        struct Selection { schema_migration_predecessor: StudyModuleReceipt }
        let bytes = std::fs::read(state.join("config.json")).map_err(invalid)?;
        let selection: Selection = serde_json::from_slice(&bytes).map_err(invalid)?;
        selection.schema_migration_predecessor.validate(state)?;
        if selection.schema_migration_predecessor.sha256 != expected {
            return Err(invalid("original migration module differs"));
        }
        Ok(selection.schema_migration_predecessor)
    }
    pub(crate) fn read_migration_target(state: &Path) -> Result<Option<Self>, CanonicalError> {
        #[derive(Deserialize)]
        struct Selection { schema_migration_target: Option<StudyModuleReceipt> }
        let bytes = std::fs::read(state.join("config.json")).map_err(invalid)?;
        let selection: Selection = serde_json::from_slice(&bytes).map_err(invalid)?;
        if let Some(receipt) = &selection.schema_migration_target { receipt.validate(state)?; }
        Ok(selection.schema_migration_target)
    }
    #[cfg(all(test, feature = "canonical-tests"))]
    pub(crate) fn read_qualification(state: &Path) -> Result<Self, CanonicalError> {
        #[derive(Deserialize)]
        struct Selection {
            study_module_qualification: StudyModuleReceipt,
        }
        let bytes = std::fs::read(state.join("config.json")).map_err(invalid)?;
        let receipt: Selection = serde_json::from_slice(&bytes).map_err(invalid)?;
        let receipt = receipt.study_module_qualification;
        let (kind, declared) = if receipt.inputs.get("features")
            == Some(&serde_json::json!(["module-qualification"]))
        {
            (
                "pse-study-module-qualification-v1",
                ModuleDeclaration::read()?,
            )
        } else if receipt.inputs.get("features")
            == Some(&serde_json::json!(["module-qualification-long-hold"]))
        {
            (
                "pse-study-module-qualification-long-hold-v1",
                ModuleDeclaration::from_manifest(include_str!(
                    "../../pse-study-module/surrealism-qualification-long-hold.toml"
                ))?,
            )
        } else {
            return Err(invalid(
                "exact separate qualification feature receipt required",
            ));
        };
        receipt.validate_declared(state, kind, &declared)?;
        Ok(receipt)
    }
    fn validate(&self, state: &Path) -> Result<(), CanonicalError> {
        self.validate_kind(state, "pse-study-module-v1")
    }
    fn validate_kind(&self, state: &Path, kind: &str) -> Result<(), CanonicalError> {
        self.validate_declared(state, kind, &ModuleDeclaration::read()?)
    }
    fn validate_declared(
        &self,
        state: &Path,
        kind: &str,
        declared: &ModuleDeclaration,
    ) -> Result<(), CanonicalError> {
        if self.kind != kind
            || self.sdk_version != declared.sdk
            || self.abi != declared.abi
            || self.inputs.is_empty()
            || self.sha256.len() != 64
            || !self
                .sha256
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            || self.bytes == 0
            || self.bytes > 64 * 1024 * 1024
        {
            return Err(invalid(
                "pinned artifact identity, ABI or source provenance differs",
            ));
        }
        if self.limits != declared.limits {
            return Err(invalid("finite pure-module limits differ"));
        }
        if !self.artifact.is_absolute()
            || self.artifact.file_name().and_then(|name| name.to_str()) != Some("study.surli")
            || std::fs::canonicalize(&self.artifact).map_err(invalid)? != self.artifact
        {
            return Err(invalid(
                "artifact must be the exact absolute non-symlink study.surli",
            ));
        }
        let parent = self
            .artifact
            .parent()
            .ok_or_else(|| invalid("artifact parent absent"))?;
        let metadata = std::fs::symlink_metadata(&self.artifact).map_err(invalid)?;
        let directory = std::fs::symlink_metadata(parent).map_err(invalid)?;
        if !metadata.is_file()
            || metadata.file_type().is_symlink()
            || metadata.len() != self.bytes
            || !directory.is_dir()
            || directory.file_type().is_symlink()
        {
            return Err(invalid(
                "artifact file, private directory or exact byte extent differs",
            ));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let owner = std::fs::metadata(state).map_err(invalid)?.uid();
            if metadata.mode() & 0o377 != 0
                || metadata.nlink() != 1
                || metadata.uid() != owner
                || directory.mode() & 0o077 != 0
                || directory.uid() != owner
            {
                return Err(invalid(
                    "artifact must be private, immutable and owned by the selected state owner",
                ));
            }
        }
        let mut file = std::fs::File::open(&self.artifact).map_err(invalid)?;
        let opened = file.metadata().map_err(invalid)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if opened.dev() != metadata.dev() || opened.ino() != metadata.ino() {
                return Err(invalid("artifact identity changed while opening"));
            }
        }
        let mut hash = Sha256::new();
        let mut buffer = [0_u8; 64 * 1024];
        let mut length = 0_u64;
        loop {
            let read = file.read(&mut buffer).map_err(invalid)?;
            if read == 0 {
                break;
            }
            length = length
                .checked_add(read as u64)
                .ok_or_else(|| invalid("artifact extent overflow"))?;
            if length > self.bytes {
                return Err(invalid("artifact grew while hashing"));
            }
            hash.update(&buffer[..read]);
        }
        if length != self.bytes
            || hash
                .finalize()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
                != self.sha256
        {
            return Err(invalid("artifact SHA256 or actual extent differs"));
        }
        Ok(())
    }
    pub(crate) fn bucket(&self) -> String {
        format!("pse_study_{}", self.sha256)
    }
    pub(crate) fn backend(&self) -> Result<String, CanonicalError> {
        let parent = self
            .artifact
            .parent()
            .and_then(Path::to_str)
            .ok_or_else(|| invalid("artifact directory must be UTF-8"))?;
        Ok(format!("file:{parent}"))
    }
    pub(crate) fn declarations(&self, overwrite: bool) -> String {
        let kind = if overwrite { " OVERWRITE" } else { "" };
        let bucket = self.bucket();
        format!(
            "DEFINE BUCKET{kind} {bucket} READONLY BACKEND $study_module_backend PERMISSIONS NONE;\nDEFINE MODULE{kind} mod::pse FROM f\"{bucket}:/study.surli\" UNSIGNED PERMISSIONS FULL;\n"
        )
    }
}
/// Exact installed catalogue proof shared by ordinary opening and native migration.
/// Bind `$bucket` and `$backend` from the immutable validated module receipt.
pub(crate) const CATALOG_VALIDATION_BODY: &str = "LET $info=INFO FOR DB STRUCTURE; LET $b=$info.buckets.filter(|$row|$row.name=$bucket); LET $m=$info.modules.filter(|$row|$row.name='pse');";
pub(crate) const CATALOG_VALIDATION_PREDICATE: &str = "array::len($b)=1 AND array::len($m)=1 AND $b[0].readonly=true AND $b[0].backend=$backend AND $m[0].unsigned=true AND $m[0].executable.type='surrealism' AND $m[0].executable.bucket=$bucket AND $m[0].executable.key='/study.surli' AND array::len($m[0].exports)=1 AND $m[0].exports[0].name='study::readiness' AND $m[0].exports[0].writeable=false AND $pse_prepare_probe.accepted=true AND array::len($pse_prepare_probe.decisions)=1 AND $pse_prepare_probe.decisions[0].disposition='prepare' AND $pse_prepare_probe.decisions[0].action=$pse_probe_action AND $pse_claim_probe.accepted=true AND array::len($pse_claim_probe.decisions)=1 AND $pse_claim_probe.decisions[0].disposition='start' AND $pse_claim_probe.decisions[0].action=$pse_probe_action AND $pse_claim_probe.decisions[0].start=$pse_probe_start";

/// Prove both new progression modes at the exact immutable-module admission
/// boundary. These synthetic policy calls are pure and create no canonical work.
pub(crate) fn capability_validation_body() -> Result<String, CanonicalError> {
    use pse_model::generated::enums::StudyPointState;
    use pse_model::study::{ActionKind, EffectState, OccurrenceKey, PointAction, PointFacts, PointPolicy, ScientificFacts, SeedNeed, StartPolicy, StartProvenance};
    use surrealdb_types::{Bytes, Object, ToSql, Value};
    fn encode<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, CanonicalError> {
        serde_json::to_vec(value).map_err(|error| CanonicalError::Configuration(error.to_string()))
    }
    let policy = PointPolicy { key: OccurrenceKey(0), dependencies: Vec::new(), seed_need: SeedNeed::NotNeeded, start: StartPolicy::Fresh, attempt_limit: 1 };
    let facts = PointFacts { key: policy.key, revision: 0, lifecycle: StudyPointState::Pending, admitted_seed_need: None, native_started: false, scientific: ScientificFacts::default(), seed: None, attempt_count: 0, retry_failure: None, effect: EffectState::Absent };
    let action = PointAction { occurrence: policy.key, expected_revision: 0, kind: ActionKind::Start(StartProvenance::NotNeeded) };
    let mut row = Object::new();
    row.insert("key", "module-capability-probe");
    row.insert("occurrence", crate::canonical_codec::encode_uint(0)?);
    row.insert("revision", crate::canonical_codec::encode_uint(0)?);
    row.insert("policy", Bytes::from(encode(&policy)?));
    row.insert("facts", Bytes::from(encode(&facts)?));
    row.insert("assigned", false); row.insert("settled", false); row.insert("outcome", Value::None);
    let mut item = Object::new(); item.insert("point", "module-capability-probe"); item.insert("predecessors", Value::Array(Vec::<Value>::new().into()));
    let mut page = Object::new();
    page.insert("generation", crate::canonical_codec::encode_uint(0)?);
    page.insert("cancelled", false); page.insert("mode", "prepare"); page.insert("seed", Value::None);
    page.insert("rows", Value::Array(vec![Value::Object(row)].into()));
    page.insert("items", Value::Array(vec![Value::Object(item)].into()));
    let prepare = page.to_sql();
    page.insert("mode", "claim"); page.insert("seed", Bytes::from(b"null".to_vec()));
    Ok(format!("{CATALOG_VALIDATION_BODY} LET $pse_probe_action={}; LET $pse_probe_start={}; LET $pse_prepare_probe=mod::pse::study::readiness({prepare}); LET $pse_claim_probe=mod::pse::study::readiness({});", Bytes::from(encode(&action)?).to_sql(), Bytes::from(encode(&StartProvenance::NotNeeded)?).to_sql(), page.to_sql()))
}

impl CanonicalStore {
    /// Install only under the exact selected maintenance owner. Publication of
    /// the schema/module pair remains a separate validated activation.
    #[cfg(feature = "canonical-maintenance")]
    pub async fn install_study_module(&self) -> Result<(), CanonicalError> {
        let activity = self.admit_activity()?;
        activity.scope(async {
            crate::canonical::within_clock(crate::canonical::original_deadline(crate::canonical::ACTIVATION_REQUEST_TIMEOUT),async {
                let before=self.maintenance_config()?;
                let receipt=StudyModuleReceipt::read(&self.state_path)?;
                let response=bounded_query(self.db.script(GuardedScript::new(Envelope::DeadlineOnly).returning::<surrealdb_types::Value>(&format!("LET $marker=SELECT study_module_sha256 FROM ONLY canonical_interpretations:current FOR UPDATE; IF $marker!=NONE AND $marker.study_module_sha256!=$study_module_sha256 {{THROW 'PSE_REFUSAL:activation_predecessor_changed';}}; {}",receipt.declarations(true)),"NONE")?)
                    .bind(("study_module_backend",receipt.backend()?)).bind(("study_module_sha256",receipt.sha256.clone()))).await?;
                crate::canonical::complete_response(response)?;
                if self.maintenance_config()?!=before {return Err(CanonicalError::Quiesced);}
                self.validate_study_module(&receipt).await
            }).await
        }).await
    }
    /// Explain selected artifact and its installed readonly policy capability.
    pub async fn inspect_study_module(&self) -> Result<serde_json::Value, CanonicalError> {
        let receipt = StudyModuleReceipt::read(&self.state_path)?;
        self.validate_study_module(&receipt).await?;
        let mut response = bounded_query(self.db.metadata_query(MetadataResponse::DiagnosticPlan,
            "LET $info=INFO FOR DB STRUCTURE; RETURN {buckets:$info.buckets.filter(|$row|$row.name=$bucket),modules:$info.modules.filter(|$row|$row.name='pse')};")
            .bind(("bucket", receipt.bucket()))).await?;
        let installed = UnframedResult::<surrealdb_types::Value>::returned().take(&mut response)?;
        Ok(
            serde_json::json!({"selected": receipt, "installed": installed,
            "attestation": "sole readonly study::readiness export, exact immutable bucket and pure preparation/claim capability probes passed",
            "scope": "catalogue and pure synthetic policy invocation; no scientific execution or installation; local deadline/memory/controller-pool limits remain in force"}),
        )
    }

    /// Catalog structure, not rendered SQL text, attests the exact immutable
    /// artifact path and only supported read-only policy export.
    pub(crate) async fn validate_study_module(
        &self,
        receipt: &StudyModuleReceipt,
    ) -> Result<(), CanonicalError> {
        let mut response = bounded_query(
            self.db
                .metadata_read_query(
                    MetadataResponse::Header,
                    format!("{} RETURN {CATALOG_VALIDATION_PREDICATE};", capability_validation_body()?),
                )
                .bind(("bucket", receipt.bucket()))
                .bind(("backend", receipt.backend()?)),
        )
        .await?;
        if UnframedResult::<Option<bool>>::returned().take(&mut response)? != Some(true) {
            return Err(invalid(
                "installed module, readonly bucket or pure export differs",
            ));
        }
        Ok(())
    }
}

#[cfg(all(test, unix))]
#[allow(
    clippy::unwrap_used,
    reason = "isolated fixture construction must fail its test immediately"
)]
pub(super) mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    pub(crate) fn receipt_in(state: &Path) -> StudyModuleReceipt {
        let parent = state.join("module");
        std::fs::create_dir(&parent).unwrap();
        std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o700)).unwrap();
        let artifact = parent.join("study.surli");
        let payload = b"exact immutable artifact";
        std::fs::write(&artifact, payload).unwrap();
        std::fs::set_permissions(&artifact, std::fs::Permissions::from_mode(0o400)).unwrap();
        let sha256 = Sha256::digest(payload)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        let declared = ModuleDeclaration::read().unwrap();
        StudyModuleReceipt {
            kind: "pse-study-module-v1".into(),
            artifact,
            sha256,
            bytes: payload.len() as u64,
            sdk_version: declared.sdk,
            abi: declared.abi,
            inputs: serde_json::Map::from_iter([(
                "source".into(),
                serde_json::Value::String("fixture".into()),
            )]),
            limits: declared.limits,
        }
    }
    struct Fixture {
        state: PathBuf,
        receipt: StudyModuleReceipt,
    }
    impl Fixture {
        fn new() -> Self {
            let state =
                std::env::temp_dir().join(format!("pse-study-module-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir(&state).unwrap();
            let receipt = receipt_in(&state);
            Self { state, receipt }
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.state).unwrap();
        }
    }
    #[cfg(feature = "native-service")]
    #[test]
    fn study_module_private_copy_preserves_original_provenance_and_refuses_replacement() {
        let fixture = Fixture::new();
        std::fs::set_permissions(&fixture.state, std::fs::Permissions::from_mode(0o700)).unwrap();
        let original = serde_json::to_value(&fixture.receipt).unwrap();
        let destination = fixture.state.join("private-copy");
        let copied = fixture
            .receipt
            .copy_into_private(&fixture.state, &destination)
            .unwrap();
        let mut actual = serde_json::to_value(&copied).unwrap();
        actual["artifact"] = original["artifact"].clone();
        assert_eq!(
            actual, original,
            "only the admitted artifact path may change"
        );
        assert_ne!(copied.artifact, fixture.receipt.artifact);
        copied.validate(&fixture.state).unwrap();
        assert!(
            fixture
                .receipt
                .copy_into_private(&fixture.state, &destination)
                .is_err()
        );
        std::fs::set_permissions(&copied.artifact, std::fs::Permissions::from_mode(0o600)).unwrap();
        std::fs::write(&copied.artifact, b"changed").unwrap();
        assert!(copied.validate(&fixture.state).is_err());
        fixture.receipt.validate(&fixture.state).unwrap();
    }
    #[test]
    fn study_module_receipt_binds_hash_private_extent_and_pinned_limits() {
        let fixture = Fixture::new();
        fixture.receipt.validate(&fixture.state).unwrap();
        let mut wrong = fixture.receipt.clone();
        wrong.sha256 = "0".repeat(64);
        assert!(wrong.validate(&fixture.state).is_err());
        let mut wrong = fixture.receipt.clone();
        wrong.bytes += 1;
        assert!(wrong.validate(&fixture.state).is_err());
        let mut wrong = fixture.receipt.clone();
        wrong.limits.max_pool_size += 1;
        assert!(wrong.validate(&fixture.state).is_err());
        let mut wrong = fixture.receipt.clone();
        wrong.abi = 1;
        assert!(wrong.validate(&fixture.state).is_err());
        assert!(
            fixture
                .receipt
                .declarations(false)
                .contains(&fixture.receipt.bucket())
        );
        assert!(
            fixture
                .receipt
                .declarations(true)
                .contains("DEFINE MODULE OVERWRITE mod::pse")
        );
        std::fs::set_permissions(
            &fixture.receipt.artifact,
            std::fs::Permissions::from_mode(0o600),
        )
        .unwrap();
        assert!(fixture.receipt.validate(&fixture.state).is_err());
    }
    #[cfg(feature = "canonical-tests")]
    #[test]
    fn study_module_receipt_qualification_variants_bind_exact_features_and_declared_limits() {
        let fixture = Fixture::new();
        for (kind, features, declared) in [
            (
                "pse-study-module-qualification-v1",
                serde_json::json!(["module-qualification"]),
                ModuleDeclaration::read().unwrap(),
            ),
            (
                "pse-study-module-qualification-long-hold-v1",
                serde_json::json!(["module-qualification-long-hold"]),
                ModuleDeclaration::from_manifest(include_str!(
                    "../../pse-study-module/surrealism-qualification-long-hold.toml"
                ))
                .unwrap(),
            ),
        ] {
            let mut receipt = fixture.receipt.clone();
            receipt.kind = kind.into();
            receipt.limits = declared.limits;
            receipt.inputs.insert("features".into(), features);
            let write = |receipt: &StudyModuleReceipt| {
                std::fs::write(
                    fixture.state.join("config.json"),
                    serde_json::to_vec(&serde_json::json!({"study_module_qualification":receipt}))
                        .unwrap(),
                )
                .unwrap()
            };
            write(&receipt);
            StudyModuleReceipt::read_qualification(&fixture.state).unwrap();
            let mut wrong = receipt.clone();
            wrong.limits.max_execution_time_ms += 1;
            write(&wrong);
            assert!(StudyModuleReceipt::read_qualification(&fixture.state).is_err());
            let mut wrong = receipt.clone();
            wrong.kind = "pse-study-module-v1".into();
            write(&wrong);
            assert!(StudyModuleReceipt::read_qualification(&fixture.state).is_err());
            let mut wrong = receipt.clone();
            wrong.inputs.insert(
                "features".into(),
                serde_json::json!(["module-qualification", "module-qualification-long-hold"]),
            );
            write(&wrong);
            assert!(StudyModuleReceipt::read_qualification(&fixture.state).is_err());
            assert!(
                receipt.validate(&fixture.state).is_err(),
                "qualification artifact never admits as production"
            );
        }
    }
    #[test]
    fn study_module_receipt_rejects_symlink_hardlink_and_shared_directory() {
        let fixture = Fixture::new();
        let alias = fixture.state.join("alias");
        std::fs::hard_link(&fixture.receipt.artifact, &alias).unwrap();
        assert!(fixture.receipt.validate(&fixture.state).is_err());
        std::fs::remove_file(&alias).unwrap();
        let parent = fixture.receipt.artifact.parent().unwrap();
        std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(fixture.receipt.validate(&fixture.state).is_err());
        std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700)).unwrap();
        std::os::unix::fs::symlink(parent, &alias).unwrap();
        let mut wrong = fixture.receipt.clone();
        wrong.artifact = alias.join("study.surli");
        assert!(wrong.validate(&fixture.state).is_err());
    }
}
