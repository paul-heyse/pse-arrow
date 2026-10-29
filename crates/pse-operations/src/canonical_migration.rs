// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Version-bound canonical construction and explicit quiescent schema evolution.
//!
//! The frozen pre-epoch and schema-v2 declarations are supported historical predecessors,
//! not a current schema authority. Legacy clients cannot be retroactively fenced:
//! that transition requires the deployment owner's exclusive maintenance boundary.
use super::{CanonicalError, CanonicalOptions, CanonicalStore, bounded_query, wire};
use crate::{
    canonical_codec as codec, canonical_execution::AttemptFence, canonical_script::ResultSlot,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    future::Future,
    sync::{Arc, OnceLock},
};
use surrealdb_types::{Decimal, Number, Object, SurrealValue, Value};

/// Preserved declaration needed to construct a real pre-epoch disposable database.
pub const PRE_EPOCH_DECLARATION: &str = include_str!("migrations/substrate-v10-pre-epoch.surql");
/// Immutable native admission predecessor, kept outside generated publication.
pub const SCHEMA_V2_DECLARATION: &str = include_str!("migrations/substrate-v10-schema-v2.surql");
const VERSION: u64 = pse_schema::catalog::substrate::SCHEMA_VERSION;
const V2_VERSION: u64 = pse_schema::catalog::substrate::SCHEMA_V2_VERSION;
const V2_DIGEST: &str = pse_schema::catalog::substrate::SCHEMA_V2_DIGEST;
pub(crate) const TRANSACTION_ADMISSION: &str = "\nfn::pse_schema_v1::admit($pse_rpc_expires_at,$pse_schema_expected,$pse_schema_settlement,$pse_schema_access);\n";

#[derive(Clone, Copy, Debug)]
pub(crate) enum SchemaAdmission {
    Read,
    Mutation,
    Cleanup,
}
impl SchemaAdmission {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Mutation => "mutation",
            Self::Cleanup => "cleanup",
        }
    }
}
tokio::task_local! { static SETTLEMENT: Value; }
pub(crate) fn settlement_value() -> Value {
    SETTLEMENT.try_with(Clone::clone).unwrap_or(Value::None)
}
pub(crate) fn settling() -> bool {
    SETTLEMENT
        .try_with(|value| !matches!(value, Value::None))
        .unwrap_or(false)
}
pub(crate) async fn settlement_scope<F, T>(
    fence: &AttemptFence,
    future: F,
) -> Result<T, CanonicalError>
where
    F: Future<Output = Result<T, CanonicalError>>,
{
    let mut value = Object::new();
    value.insert("run", fence.run().to_owned());
    value.insert("attempt", fence.attempt().to_owned());
    value.insert("generation", codec::encode_uint(fence.generation())?);
    SETTLEMENT.scope(Value::Object(value), future).await
}
fn invalid(message: impl Into<String>) -> CanonicalError {
    CanonicalError::Configuration(message.into())
}
fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
fn bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, CanonicalError> {
    serde_json::to_vec(value).map_err(|error| invalid(error.to_string()))
}
fn string(object: &Object, name: &'static str) -> Result<String, CanonicalError> {
    codec::decode_string(
        object
            .get(name)
            .cloned()
            .ok_or(CanonicalError::IncompleteResponse)?,
    )
    .map_err(Into::into)
}
fn optional_uint(object: &Object, name: &'static str) -> Result<Option<u64>, CanonicalError> {
    match object.get(name) {
        None | Some(Value::None) => Ok(None),
        Some(value) => codec::decode_uint(value.clone())
            .map(Some)
            .map_err(Into::into),
    }
}
fn optional_string(object: &Object, name: &'static str) -> Result<Option<String>, CanonicalError> {
    match object.get(name) {
        None | Some(Value::None) => Ok(None),
        Some(value) => codec::decode_string(value.clone())
            .map(Some)
            .map_err(Into::into),
    }
}
/// Exact immutable opening premise. Dynamic permission is checked in each transaction.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaFence {
    schema_version: u64,
    schema_epoch: String,
    schema_digest: String,
    issuing_store: String,
}
impl SchemaFence {
    /// Supported schema version established once at opening.
    pub fn version(&self) -> u64 {
        self.schema_version
    }
    /// Original epoch; a successor capability must be reconstructed.
    pub fn epoch(&self) -> &str {
        &self.schema_epoch
    }
    /// Issuer bound to this handle.
    pub fn issuer(&self) -> &str {
        &self.issuing_store
    }
    pub(crate) fn value(&self) -> Value {
        let mut value = Object::new();
        value.insert(
            "schema_version",
            Value::Number(Number::Decimal(Decimal::from(self.schema_version))),
        );
        value.insert("schema_epoch", self.schema_epoch.clone());
        value.insert("schema_digest", self.schema_digest.clone());
        value.insert("issuing_store", self.issuing_store.clone());
        Value::Object(value)
    }
    pub(crate) fn from_marker(marker: &Object) -> Result<Self, CanonicalError> {
        let predecessor = SchemaPredecessor::decode(marker)?;
        if predecessor.interpretation != wire::INTERPRETATION {
            return Err(CanonicalError::Interpretation {
                expected: wire::INTERPRETATION,
                observed: predecessor.interpretation,
            });
        }
        if predecessor.schema_digest != wire::SCHEMA_DIGEST {
            return Err(CanonicalError::Interpretation {
                expected: wire::SCHEMA_DIGEST,
                observed: predecessor.schema_digest,
            });
        }
        if predecessor.schema_version != Some(VERSION)
            || marker.get("admission_open") != Some(&Value::Bool(true))
        {
            return Err(invalid(
                "schema is not current and open; use explicit native maintenance for a supported predecessor",
            ));
        }
        let epoch = predecessor
            .schema_epoch
            .ok_or_else(|| invalid("schema epoch is absent"))?;
        uuid::Uuid::parse_str(&epoch).map_err(|_| invalid("schema epoch is invalid"))?;
        uuid::Uuid::parse_str(&predecessor.issuing_store)
            .map_err(|_| invalid("schema issuer is invalid"))?;
        Ok(Self {
            schema_version: VERSION,
            schema_epoch: epoch,
            schema_digest: predecessor.schema_digest,
            issuing_store: predecessor.issuing_store,
        })
    }
}
/// An opened, supported native control capability; no mathematical runtime is created.
#[derive(Clone, Debug)]
pub struct SchemaReady {
    store: CanonicalStore,
    fence: SchemaFence,
}
impl SchemaReady {
    /// Existing canonical operations, with mandatory transaction fences.
    pub fn store(&self) -> &CanonicalStore {
        &self.store
    }
    /// Established opening premise, never a perpetual grant to write.
    pub fn fence(&self) -> &SchemaFence {
        &self.fence
    }
    /// Transfer the same opened connection owner.
    pub fn into_store(self) -> CanonicalStore {
        self.store
    }
}
impl CanonicalStore {
    /// The native fixture owner calls this only for its newly created private
    /// deployment. Actual catalog absence is required; no existing canonical
    /// schema or records can be replaced and no caller supplies SQL.
    #[cfg(feature = "native-service")]
    pub(crate) async fn install_pre_epoch_fixture(
        options: &CanonicalOptions,
    ) -> Result<(), CanonicalError> {
        Self::install_frozen_fixture(options, false).await
    }
    /// Install the exact frozen v2 only for the native service's newly owned fixture.
    /// An existing catalog is refused before any schema or data effects.
    #[cfg(feature = "native-service")]
    pub(crate) async fn install_schema_v2_fixture(
        options: &CanonicalOptions,
    ) -> Result<(), CanonicalError> {
        Self::install_frozen_fixture(options, true).await
    }
    #[cfg(feature = "native-service")]
    async fn install_frozen_fixture(
        options: &CanonicalOptions,
        schema_v2: bool,
    ) -> Result<(), CanonicalError> {
        let store = Self::connect(options).await?;
        let installed = async {
            if !store.initialization_required().await? {
                return Err(invalid("predecessor installation requires a fresh owned canonical catalog"));
            }
            store.provision_context().await?;
            verify_frozen_declarations()?;
            let (declaration, digest, epoch_fields) = if schema_v2 {
                (SCHEMA_V2_DECLARATION, V2_DIGEST, ",schema_version=2dec,schema_epoch=<string>rand::uuid::v4(),admission_open=true")
            } else {
                (PRE_EPOCH_DECLARATION, pse_schema::catalog::substrate::PRE_EPOCH_SCHEMA_DIGEST, "")
            };
            let module = super::study_module::StudyModuleReceipt::read(&store.state_path)?;
            let body = format!("{}{}\nCREATE canonical_interpretations:current SET key='current',interpretation=$interpretation,schema_digest=$schema_digest,study_module_sha256=$study_module_sha256,issuing_store=<string>rand::uuid::v4(),payload_incarnation=$payload_incarnation{epoch_fields};\nCREATE canonical_file_admission:current SET key='current',storage_incarnation=$payload_incarnation,frozen=false,gc_frozen=false;", module.declarations(false), declaration);
            let mut bindings = Object::new();
            bindings.insert("interpretation", wire::INTERPRETATION);
            bindings.insert("schema_digest", digest);
            bindings.insert("study_module_backend", module.backend()?);
            bindings.insert("study_module_sha256", module.sha256.clone());
            bindings.insert("payload_incarnation", store.files.incarnation.clone());
            let maintenance = SchemaMaintenance { store: store.clone() };
            if maintenance.execute::<Option<bool>>(&body, "true", bindings).await? != Some(true) {
                return Err(CanonicalError::IncompleteResponse);
            }
            let actual = maintenance.inspect().await?;
            let exact_schema = if schema_v2 { actual.predecessor.versioned(V2_VERSION, V2_DIGEST) } else { actual.predecessor.legacy() };
            if !exact_schema || actual.predecessor.study_module_sha256 != module.sha256
                || actual.predecessor.payload_incarnation != store.files.incarnation {
                return Err(CanonicalError::IncompleteResponse);
            }
            Ok(())
        }.await;
        let drained = store.drain_owned_connection().await;
        match (installed, drained) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
            (Err(error), Err(cleanup)) => Err(CanonicalError::MigrationDrain {
                failures: vec![error, cleanup],
            }),
        }
    }
    /// Connect and establish current schema, issuer and module premises without science services.
    pub async fn connect_ready(options: &CanonicalOptions) -> Result<SchemaReady, CanonicalError> {
        let store = Self::connect(options).await?;
        if let Err(error) = store.open().await {
            return match store.drain_owned_connection().await {
                Ok(()) => Err(error),
                Err(cleanup) => Err(CanonicalError::MigrationDrain {
                    failures: vec![error, cleanup],
                }),
            };
        }
        let fence = store
            .db
            .schema
            .get()
            .cloned()
            .ok_or(CanonicalError::IncompleteResponse)?;
        Ok(SchemaReady { store, fence })
    }
    /// Give the fixed module validation probe an unpublished token. Other aliases
    /// cannot write through this token before complete construction succeeds.
    pub(crate) fn with_unpublished_schema(&self, fence: &SchemaFence) -> Self {
        let schema = Arc::new(OnceLock::from(fence.clone()));
        let mut db = (*self.db).clone();
        db.schema = schema.clone();
        let mut activation = (*self.activation_db).clone();
        activation.schema = schema;
        Self {
            db: Arc::new(db),
            activation_db: Arc::new(activation),
            ..self.clone()
        }
    }
}

/// Exact observed predecessor; callers cannot select arbitrary schema or SQL as a migration.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaPredecessor {
    interpretation: String,
    schema_digest: String,
    study_module_sha256: String,
    issuing_store: String,
    payload_incarnation: String,
    schema_version: Option<u64>,
    schema_epoch: Option<String>,
}
impl SchemaPredecessor {
    fn decode(marker: &Object) -> Result<Self, CanonicalError> {
        Ok(Self {
            interpretation: string(marker, "interpretation")?,
            schema_digest: string(marker, "schema_digest")?,
            study_module_sha256: string(marker, "study_module_sha256")?,
            issuing_store: string(marker, "issuing_store")?,
            payload_incarnation: string(marker, "payload_incarnation")?,
            schema_version: optional_uint(marker, "schema_version")?,
            schema_epoch: optional_string(marker, "schema_epoch")?,
        })
    }
    fn legacy(&self) -> bool {
        self.schema_version.is_none()
            && self.schema_epoch.is_none()
            && self.schema_digest == pse_schema::catalog::substrate::PRE_EPOCH_SCHEMA_DIGEST
    }
    fn versioned(&self, version: u64, digest: &str) -> bool {
        self.schema_version == Some(version)
            && self.schema_digest == digest
            && self
                .schema_epoch
                .as_ref()
                .is_some_and(|epoch| uuid::Uuid::parse_str(epoch).is_ok())
    }
    fn supported(&self) -> bool {
        self.interpretation == wire::INTERPRETATION
            && uuid::Uuid::parse_str(&self.issuing_store).is_ok()
            && (self.legacy()
                || self.versioned(V2_VERSION, V2_DIGEST)
                || self.versioned(VERSION, wire::SCHEMA_DIGEST))
    }
    fn value(&self) -> Result<Value, CanonicalError> {
        let mut value = Object::new();
        value.insert("interpretation", self.interpretation.clone());
        value.insert("schema_digest", self.schema_digest.clone());
        value.insert("study_module_sha256", self.study_module_sha256.clone());
        value.insert("issuing_store", self.issuing_store.clone());
        value.insert("payload_incarnation", self.payload_incarnation.clone());
        value.insert(
            "schema_version",
            self.schema_version
                .map(codec::encode_uint)
                .transpose()?
                .unwrap_or(Value::None),
        );
        value.insert(
            "schema_epoch",
            self.schema_epoch
                .clone()
                .map(Value::String)
                .unwrap_or(Value::None),
        );
        Ok(Value::Object(value))
    }
    /// Construct an original request for the one current target. Preserve this
    /// serializable request for exact restart/reconciliation; never mint a retry id.
    fn to_current(
        &self,
        operation: uuid::Uuid,
        module_bucket: String,
        module_backend: String,
    ) -> Result<SchemaMigrationRequest, CanonicalError> {
        if !self.supported() {
            return Err(invalid("unsupported schema predecessor"));
        }
        Ok(SchemaMigrationRequest {
            operation: operation.to_string(),
            successor_epoch: uuid::Uuid::new_v4().to_string(),
            predecessor: self.clone(),
            target_version: VERSION,
            target_digest: wire::SCHEMA_DIGEST.into(),
            module_bucket,
            module_backend,
            target_module: None,
        })
    }
}
/// Validated immutable target selected by the existing module receipt owner.
/// Its artifact, source provenance and backend are part of the original intent.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreparedSchemaModule {
    pub(crate) receipt: super::study_module::StudyModuleReceipt,
    bucket: String,
    backend: String,
}
impl PreparedSchemaModule {
    fn from_receipt(
        receipt: super::study_module::StudyModuleReceipt,
    ) -> Result<Self, CanonicalError> {
        Ok(Self {
            bucket: receipt.bucket(),
            backend: receipt.backend()?,
            receipt,
        })
    }
    /// Exact immutable target artifact hash.
    pub fn sha256(&self) -> &str {
        &self.receipt.sha256
    }
}
/// Original migration intent, safe to retain without credentials or native pointers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaMigrationRequest {
    operation: String,
    successor_epoch: String,
    predecessor: SchemaPredecessor,
    target_version: u64,
    target_digest: String,
    module_bucket: String,
    module_backend: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    target_module: Option<PreparedSchemaModule>,
}
impl SchemaMigrationRequest {
    /// Original migration identity used by its immutable effect receipt.
    pub fn operation(&self) -> &str {
        &self.operation
    }
    fn validate(&self) -> Result<(), CanonicalError> {
        let current_target =
            self.target_version == VERSION && self.target_digest == wire::SCHEMA_DIGEST;
        // An already-issued v2 request retains its exact fingerprint and receipt.
        // It may settle at v2, but never grants a current SchemaReady capability.
        let original_v2_target = self.target_version == V2_VERSION
            && self.target_digest == V2_DIGEST
            && (self.predecessor.legacy() || self.predecessor.versioned(V2_VERSION, V2_DIGEST));
        if !self.predecessor.supported()
            || !(current_target || original_v2_target)
            || (self.target_module.is_some() && !current_target)
            || self.target_module.as_ref().is_some_and(|target| {
                target.bucket != target.receipt.bucket()
                    || target.receipt.backend().ok().as_deref() != Some(target.backend.as_str())
            })
            || uuid::Uuid::parse_str(&self.operation).is_err()
            || uuid::Uuid::parse_str(&self.successor_epoch).is_err()
            || self.predecessor.schema_epoch.as_deref() == Some(self.successor_epoch.as_str())
        {
            return Err(invalid(
                "migration request differs from its supported original inputs",
            ));
        }
        Ok(())
    }
    fn target_module_hash(&self) -> &str {
        self.target_module
            .as_ref()
            .map_or(&self.predecessor.study_module_sha256, |target| {
                target.sha256()
            })
    }
    fn fingerprint(&self) -> Result<String, CanonicalError> {
        Ok(hash(&bytes(self)?))
    }
    fn needs_v2_checkpoint(&self) -> bool {
        self.predecessor.legacy() && self.target_version == VERSION
    }
    fn checkpoint_predecessor(&self) -> SchemaPredecessor {
        SchemaPredecessor {
            schema_digest: V2_DIGEST.into(),
            schema_version: Some(V2_VERSION),
            schema_epoch: Some(self.successor_epoch.clone()),
            ..self.predecessor.clone()
        }
    }
    fn checkpoint_fingerprint(&self) -> Result<String, CanonicalError> {
        Ok(hash(&bytes(&(
            self.fingerprint()?,
            self.checkpoint_predecessor(),
        ))?))
    }
    fn bindings(&self) -> Result<Object, CanonicalError> {
        let mut bindings = Object::new();
        bindings.insert("migration_operation", self.operation.clone());
        bindings.insert("migration_generation", self.successor_epoch.clone());
        bindings.insert("migration_request", self.fingerprint()?);
        bindings.insert("migration_predecessor", self.predecessor.value()?);
        bindings.insert("migration_target", self.target_digest.clone());
        bindings.insert("migration_v2_digest", V2_DIGEST);
        bindings.insert("migration_checkpoint", self.checkpoint_fingerprint()?);
        bindings.insert(
            "migration_version",
            codec::encode_uint(self.target_version)?,
        );
        bindings.insert(
            "bucket",
            self.target_module
                .as_ref()
                .map_or(&self.module_bucket, |target| &target.bucket)
                .clone(),
        );
        bindings.insert(
            "backend",
            self.target_module
                .as_ref()
                .map_or(&self.module_backend, |target| &target.backend)
                .clone(),
        );
        bindings.insert(
            "study_module_backend",
            self.target_module
                .as_ref()
                .map_or(&self.module_backend, |target| &target.backend)
                .clone(),
        );
        bindings.insert(
            "migration_module_sha256",
            self.target_module_hash().to_owned(),
        );
        bindings.insert(
            "migration_predecessor_bytes",
            surrealdb_types::Bytes::from(bytes(&self.predecessor)?),
        );
        Ok(bindings)
    }
}
/// Verified immutable activation of the exact original migration intent.
/// Historical success is a receipt, independent of current-schema readiness.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchemaMigrationReceipt {
    request: SchemaMigrationRequest,
}
impl SchemaMigrationReceipt {
    /// Original serializable request, retaining its operation, predecessor and target.
    pub fn request(&self) -> &SchemaMigrationRequest {
        &self.request
    }
    /// Explicit prepared module change verified by this original activation.
    pub fn target_module(&self) -> Option<&PreparedSchemaModule> {
        self.request.target_module.as_ref()
    }
    /// Installed target version recorded by this original activation.
    pub fn version(&self) -> u64 {
        self.request.target_version
    }
    /// Exact installed target digest.
    pub fn digest(&self) -> &str {
        &self.request.target_digest
    }
}
/// Exact observed state; a pending migration never claims that native writers drained.
#[derive(Clone, Debug)]
pub struct SchemaMigrationState {
    /// Observed supported predecessor, retaining scientific and payload authority.
    pub predecessor: SchemaPredecessor,
    /// Exact persisted pending owner when present.
    pub pending_operation: Option<String>,
    /// Persisted phase; not physical completion evidence.
    pub pending_phase: Option<String>,
}
/// Explicit privileged native migration connection, distinct from ordinary schema readiness.
#[derive(Clone, Debug)]
pub struct SchemaMaintenance {
    store: CanonicalStore,
}
impl SchemaMaintenance {
    /// Select existing private administrative configuration; never create a database or schema.
    pub async fn connect(options: &CanonicalOptions) -> Result<Self, CanonicalError> {
        Ok(Self {
            store: CanonicalStore::connect(options).await?,
        })
    }
    async fn execute<T: SurrealValue>(
        &self,
        body: &str,
        result: &str,
        bindings: Object,
    ) -> Result<T, CanonicalError>
    where
        usize: crate::transport::client::QueryResult<T>,
    {
        let script = MigrationScript::new(body, result, bindings);
        let mut response = bounded_query(self.store.activation_db.migration_script(script)).await?;
        ResultSlot::returned().take(&mut response)
    }
    async fn execute_object(
        &self,
        body: &str,
        result: &str,
        bindings: Object,
    ) -> Result<Object, CanonicalError> {
        self.execute::<Option<Object>>(body, result, bindings)
            .await?
            .ok_or(CanonicalError::IncompleteResponse)
    }
    async fn snapshot(&self) -> Result<Object, CanonicalError> {
        self.execute_object("", MIGRATION_SNAPSHOT, Object::new())
            .await
    }
    async fn checkpoint(&self, request: &SchemaMigrationRequest) -> Result<bool, CanonicalError> {
        verify_v2_checkpoint(&self.snapshot().await?, request)
    }
    /// Observe actual marker and pending native owner, without interpreting a config flag as readiness.
    pub async fn inspect(&self) -> Result<SchemaMigrationState, CanonicalError> {
        let value = self.snapshot().await?;
        let Some(Value::Object(marker)) = value.get("marker") else {
            return Err(invalid("existing schema marker is required for migration"));
        };
        let predecessor = SchemaPredecessor::decode(marker)?;
        if !predecessor.supported() {
            return Err(invalid(
                "unsupported schema predecessor; no schema or data was changed",
            ));
        }
        let pending = match value.get("pending") {
            Some(Value::Object(row)) if row.get("complete") == Some(&Value::Bool(false)) => {
                Some(row)
            }
            _ => None,
        };
        Ok(SchemaMigrationState {
            predecessor,
            pending_operation: pending.map(|row| string(row, "operation")).transpose()?,
            pending_phase: pending.map(|row| string(row, "phase")).transpose()?,
        })
    }
    /// Bind the original predecessor and immutable supported policy artifact once.
    pub async fn request_current(
        &self,
        operation: uuid::Uuid,
    ) -> Result<SchemaMigrationRequest, CanonicalError> {
        let observed = self.inspect().await?;
        if observed.pending_operation.is_some() {
            return Err(invalid(
                "pending schema migration requires its original request",
            ));
        }
        let module = super::study_module::StudyModuleReceipt::read_migration_source(
            &self.store.state_path,
            &observed.predecessor.study_module_sha256,
        )?;
        if module.sha256 != observed.predecessor.study_module_sha256 {
            return Err(CanonicalError::Quiesced);
        }
        let mut request =
            observed
                .predecessor
                .to_current(operation, module.bucket(), module.backend()?)?;
        if let Some(target) =
            super::study_module::StudyModuleReceipt::read_migration_target(&self.store.state_path)?
            && target.sha256 != module.sha256
        {
            request.target_module = Some(PreparedSchemaModule::from_receipt(target)?);
        }
        request.validate()?;
        Ok(request)
    }
    fn require_original_module(
        &self,
        request: &SchemaMigrationRequest,
    ) -> Result<(), CanonicalError> {
        let module = super::study_module::StudyModuleReceipt::read_migration_source(
            &self.store.state_path,
            &request.predecessor.study_module_sha256,
        )?;
        if module.sha256 != request.predecessor.study_module_sha256
            || module.bucket() != request.module_bucket
            || module.backend()? != request.module_backend
        {
            return Err(CanonicalError::Quiesced);
        }
        if let Some(target) = &request.target_module {
            let selected = super::study_module::StudyModuleReceipt::read_migration_target(
                &self.store.state_path,
            )?
            .ok_or(CanonicalError::Quiesced)?;
            if selected != target.receipt
                || selected.bucket() != target.bucket
                || selected.backend()? != target.backend
            {
                return Err(CanonicalError::Quiesced);
            }
        }
        Ok(())
    }
    /// Read the original immutable activation receipt after restart or lost ACK,
    /// including a historical target. Absence grants no replay permission; an
    /// unreadable receipt remains an error. This never opens a current capability.
    pub async fn reconcile_receipt(
        &self,
        request: &SchemaMigrationRequest,
    ) -> Result<Option<SchemaMigrationReceipt>, CanonicalError> {
        request.validate()?;
        self.require_original_module(request)?;
        let receipt: Option<Object> = self.execute("", "SELECT * FROM ONLY type::record('canonical_maintenance_activations',$migration_operation)", request.bindings()?).await?;
        receipt
            .map(|receipt| verified_activation(receipt, request))
            .transpose()
    }
    /// Reconcile a current-target activation and reconstruct current readiness.
    /// Use `reconcile_receipt` to settle an original historical activation.
    pub async fn reconcile(
        &self,
        request: &SchemaMigrationRequest,
    ) -> Result<Option<SchemaReady>, CanonicalError> {
        require_current_target(request)?;
        let Some(_receipt) = self.reconcile_receipt(request).await? else {
            return Ok(None);
        };
        self.current_ready(request).await.map(Some)
    }
    async fn current_ready(
        &self,
        request: &SchemaMigrationRequest,
    ) -> Result<SchemaReady, CanonicalError> {
        require_current_target(request)?;
        let ready = CanonicalStore::connect_ready(&self.store.connection_options).await?;
        if ready.fence.schema_epoch != request.successor_epoch
            || ready.fence.issuing_store != request.predecessor.issuing_store
        {
            return match ready.store.drain_owned_connection().await {
                Ok(()) => Err(CanonicalError::Quiesced),
                Err(cleanup) => Err(CanonicalError::MigrationDrain {
                    failures: vec![CanonicalError::Quiesced, cleanup],
                }),
            };
        }
        Ok(ready)
    }
    /// Close native epoch admission before draining. A legacy pending phase explicitly
    /// requires B's exclusive maintenance boundary; it cannot fence historical binaries.
    pub async fn begin(
        &self,
        request: SchemaMigrationRequest,
    ) -> Result<PendingSchemaMigration, CanonicalError> {
        request.validate()?;
        self.require_original_module(&request)?;
        verify_frozen_declarations()?;
        if self.checkpoint(&request).await? {
            return Ok(PendingSchemaMigration {
                maintenance: self.clone(),
                request,
            });
        }
        let observed = self.inspect().await?;
        if observed.predecessor != request.predecessor {
            return Err(invalid("migration predecessor changed"));
        }
        let row: Object = if request.predecessor.legacy() {
            self.execute_object(LEGACY_BEGIN, "$pse_migration_saved", request.bindings()?)
                .await?
        } else {
            self.execute_object("", "fn::pse_schema_v1::close($pse_rpc_expires_at,$migration_predecessor,$migration_operation,$migration_generation,$migration_target,$migration_request)", request.bindings()?).await?
        };
        verify_owner(&row, &request)?;
        Ok(PendingSchemaMigration {
            maintenance: self.clone(),
            request,
        })
    }
    /// Explicitly join this controller's native transport; it never stops the server.
    pub async fn disconnect(&self) -> Result<(), CanonicalError> {
        self.store.drain_owned_connection().await
    }
}
/// Admission remains pending if this owner is dropped or any real join fails.
#[derive(Debug)]
pub struct PendingSchemaMigration {
    maintenance: SchemaMaintenance,
    request: SchemaMigrationRequest,
}
impl PendingSchemaMigration {
    /// Original intent survives cancellation and must be reused on restart.
    pub fn request(&self) -> &SchemaMigrationRequest {
        &self.request
    }
    /// Await B/C's actual ingress close and native stop/join, then all supplied
    /// canonical connection drains. The future must implement those physical effects;
    /// a configuration flag, database count or completed health request is insufficient.
    pub async fn quiesce<F>(
        self,
        native_stop_and_join: F,
        connections: &[CanonicalStore],
    ) -> Result<QuiescentSchemaMigration, CanonicalError>
    where
        F: Future<Output = Result<(), CanonicalError>>,
    {
        let mut failures = Vec::new();
        if let Err(error) = native_stop_and_join.await {
            failures.push(error);
        }
        let results = futures_util::future::join_all(
            connections
                .iter()
                .map(|connection| connection.drain_owned_connection()),
        )
        .await;
        failures.extend(results.into_iter().filter_map(Result::err));
        if !failures.is_empty() {
            return Err(CanonicalError::MigrationDrain { failures });
        }
        let result: Value = if self.maintenance.checkpoint(&self.request).await? {
            self.maintenance
                .execute(VERIFY_V2_CHECKPOINT, "true", self.request.bindings()?)
                .await?
        } else if self.request.predecessor.legacy() {
            self.maintenance
                .execute(LEGACY_QUIESCENT, "true", self.request.bindings()?)
                .await?
        } else {
            self.maintenance.execute("", "fn::pse_schema_v1::quiescent($pse_rpc_expires_at,$migration_operation,$migration_generation,$migration_request)", self.request.bindings()?).await?
        };
        if result != Value::Bool(true) {
            return Err(CanonicalError::IncompleteResponse);
        }
        Ok(QuiescentSchemaMigration {
            maintenance: self.maintenance,
            request: self.request,
        })
    }
}
/// Private construction follows actual awaited physical joins and the canonical drain check.
#[derive(Debug)]
pub struct QuiescentSchemaMigration {
    maintenance: SchemaMaintenance,
    request: SchemaMigrationRequest,
}
impl QuiescentSchemaMigration {
    /// Apply a current-target transformation and reconstruct current readiness.
    /// Historical requests must use `apply_receipt`; the target is checked before effects.
    pub async fn apply(self) -> Result<SchemaReady, CanonicalError> {
        require_current_target(&self.request)?;
        if self.request.target_module.is_some() {
            return Err(invalid(
                "module-changing activation requires apply_receipt and explicit prepared local selection",
            ));
        }
        let maintenance = self.maintenance.clone();
        let receipt = self.apply_receipt().await?;
        maintenance.current_ready(receipt.request()).await
    }
    async fn advance_v2_checkpoint(
        &self,
        discard_acknowledgement: bool,
    ) -> Result<(), CanonicalError> {
        let request = &self.request;
        if !request.needs_v2_checkpoint() {
            return Err(invalid("this original request has no intermediate v2 edge"));
        }
        self.maintenance.require_original_module(request)?;
        if self.maintenance.checkpoint(request).await? {
            return Ok(());
        }
        let body = v2_checkpoint_body(request)?;
        let write = self
            .maintenance
            .execute_object(&body, MIGRATION_SNAPSHOT, request.bindings()?)
            .await;
        let write = if discard_acknowledgement && write.is_ok() {
            Err(CanonicalError::Timeout)
        } else {
            write
        };
        let snapshot = match write {
            Ok(snapshot) => snapshot,
            Err(error) => match self.maintenance.snapshot().await {
                Ok(snapshot) => {
                    if !verify_v2_checkpoint(&snapshot, request)? {
                        return match self.maintenance.reconcile_receipt(request).await {
                            Ok(Some(_)) => Ok(()),
                            Ok(None) => Err(error),
                            Err(readback) => Err(CanonicalError::SettlementUnresolved {
                                owner: "schema-migration-v2-checkpoint",
                                operation: request.operation.clone(),
                                request: bytes(request)?,
                                source: Box::new(error),
                                readback: Some(Box::new(readback)),
                            }),
                        };
                    }
                    snapshot
                }
                Err(readback) => {
                    return Err(CanonicalError::SettlementUnresolved {
                        owner: "schema-migration-v2-checkpoint",
                        operation: request.operation.clone(),
                        request: bytes(request)?,
                        source: Box::new(error),
                        readback: Some(Box::new(readback)),
                    });
                }
            },
        };
        if !verify_v2_checkpoint(&snapshot, request)? {
            return Err(CanonicalError::IncompleteResponse);
        }
        Ok(())
    }
    /// Publish only the closed v2 checkpoint and deliberately discard its ACK.
    /// This fixture control exercises actual original-request readback before interruption.
    #[cfg(feature = "test-support")]
    pub async fn checkpoint_v2_lost_ack_for_test(&self) -> Result<(), CanonicalError> {
        self.advance_v2_checkpoint(true).await
    }
    /// Apply each ordered edge, then atomically publish the final activation receipt.
    /// A failed/unknown acknowledgement is reconciled through the original activation
    /// receipt. Historical target-v2 success returns its typed receipt unchanged.
    /// Never replay a transformation because readback itself failed.
    pub async fn apply_receipt(self) -> Result<SchemaMigrationReceipt, CanonicalError> {
        let request = &self.request;
        self.maintenance.require_original_module(request)?;
        if let Some(receipt) = self.maintenance.reconcile_receipt(request).await? {
            return Ok(receipt);
        }
        if request.needs_v2_checkpoint() {
            self.advance_v2_checkpoint(false).await?;
        }
        let bindings = request.bindings()?;
        let body = final_application_body(request)?;
        let write: Result<Object, CanonicalError> = self
            .maintenance
            .execute_object(&body, "$pse_migration_receipt", bindings)
            .await;
        let receipt = match write {
            Ok(receipt) => receipt,
            Err(error) => {
                let readback: Result<Option<Object>, CanonicalError> = self.maintenance.execute("", "SELECT * FROM ONLY type::record('canonical_maintenance_activations',$migration_operation)", request.bindings()?).await;
                match readback {
                    Ok(Some(receipt)) => receipt,
                    Ok(None) => return Err(error),
                    Err(readback) => {
                        return Err(CanonicalError::SettlementUnresolved {
                            owner: "schema-migration",
                            operation: request.operation.clone(),
                            request: bytes(request)?,
                            source: Box::new(error),
                            readback: Some(Box::new(readback)),
                        });
                    }
                }
            }
        };
        verified_activation(receipt, request)
    }
}
fn verified_activation(
    receipt: Object,
    request: &SchemaMigrationRequest,
) -> Result<SchemaMigrationReceipt, CanonicalError> {
    verify_activation(receipt, request)?;
    Ok(SchemaMigrationReceipt {
        request: request.clone(),
    })
}
fn verify_activation(
    receipt: Object,
    request: &SchemaMigrationRequest,
) -> Result<(), CanonicalError> {
    let receipt = wire::decode_canonical_maintenance_activations(receipt)?;
    if receipt.key != request.operation
        || receipt.input_sha256 != request.fingerprint()?
        || receipt.schema_digest != request.target_digest
        || receipt.issuing_store != request.predecessor.issuing_store
        || receipt.interpretation != request.predecessor.interpretation
        || receipt.study_module_sha256 != request.target_module_hash()
        || receipt.predecessor.as_slice() != bytes(&request.predecessor)?.as_slice()
    {
        return Err(CanonicalError::OperationReused);
    }
    Ok(())
}
fn verify_owner(row: &Object, request: &SchemaMigrationRequest) -> Result<(), CanonicalError> {
    if string(row, "operation")? != request.operation
        || string(row, "generation")? != request.successor_epoch
        || string(row, "after")? != request.fingerprint()?
        || string(row, "target")? != request.target_digest
    {
        return Err(CanonicalError::OperationReused);
    }
    Ok(())
}
fn require_current_target(request: &SchemaMigrationRequest) -> Result<(), CanonicalError> {
    if request.target_version != VERSION || request.target_digest != wire::SCHEMA_DIGEST {
        return Err(invalid(
            "current schema readiness requires a current-target migration; settle historical requests through their original receipt",
        ));
    }
    Ok(())
}
fn verify_frozen_declarations() -> Result<(), CanonicalError> {
    for (declaration, expected) in [
        (
            PRE_EPOCH_DECLARATION,
            pse_schema::catalog::substrate::PRE_EPOCH_DECLARATION_SHA256,
        ),
        (
            SCHEMA_V2_DECLARATION,
            pse_schema::catalog::substrate::SCHEMA_V2_DECLARATION_SHA256,
        ),
    ] {
        if hash(declaration.as_bytes()) != expected {
            return Err(invalid(
                "frozen predecessor declaration differs from its recorded bytes",
            ));
        }
    }
    Ok(())
}
fn migration_declarations(request: &SchemaMigrationRequest) -> Result<String, CanonicalError> {
    request.validate()?;
    verify_frozen_declarations()?;
    Ok(overwrite_declarations(
        if request.target_version == V2_VERSION {
            SCHEMA_V2_DECLARATION
        } else {
            wire::SCHEMA
        },
    ))
}
fn v2_checkpoint_body(request: &SchemaMigrationRequest) -> Result<String, CanonicalError> {
    request.validate()?;
    verify_frozen_declarations()?;
    if !request.needs_v2_checkpoint() {
        return Err(invalid("no intermediate v2 edge"));
    }
    Ok(format!(
        "{APPLY_PREDECESSOR}\n{}\n{PUBLISH_V2_CHECKPOINT}",
        overwrite_declarations(SCHEMA_V2_DECLARATION)
    ))
}
fn final_application_body(request: &SchemaMigrationRequest) -> Result<String, CanonicalError> {
    let predecessor = if request.needs_v2_checkpoint() {
        VERIFY_V2_CHECKPOINT
    } else {
        APPLY_PREDECESSOR
    };
    // Historical intent attests its original module contract. Settling v2 must
    // neither demand v3 progression exports nor mint current SchemaReady authority.
    let (validation_body, validation_predicate) = if request.target_version == V2_VERSION {
        (
            super::study_module::CATALOG_VALIDATION_BODY.to_owned(),
            super::study_module::CATALOG_VALIDATION_PREDICATE.to_owned(),
        )
    } else {
        (
            super::study_module::capability_validation_body()?,
            super::study_module::capability_validation_predicate(),
        )
    };
    let module = request
        .target_module
        .as_ref()
        .map(|target| target.receipt.declarations(true))
        .unwrap_or_default();
    Ok(format!(
        "{predecessor}\n{module}\n{}\n{validation_body}\nIF !({validation_predicate}) {{ THROW 'PSE_REFUSAL:schema_migration_original_differs'; }};\n{APPLY_SUCCESSOR}",
        migration_declarations(request)?
    ))
}
fn verify_v2_checkpoint(
    snapshot: &Object,
    request: &SchemaMigrationRequest,
) -> Result<bool, CanonicalError> {
    let Some(Value::Object(owner)) = snapshot.get("pending") else {
        return Ok(false);
    };
    if owner.get("phase") != Some(&Value::String("schema-v2-checkpoint".into())) {
        return Ok(false);
    }
    if !request.needs_v2_checkpoint() {
        return Err(CanonicalError::OperationReused);
    }
    verify_owner(owner, request)?;
    let Some(Value::Object(marker)) = snapshot.get("marker") else {
        return Err(CanonicalError::IncompleteResponse);
    };
    if string(owner, "kind")? != "schema"
        || owner.get("complete") != Some(&Value::Bool(false))
        || string(owner, "source")? != request.predecessor.schema_digest
        || string(owner, "next")? != request.checkpoint_fingerprint()?
        || optional_uint(owner, "count")? != Some(1)
        || SchemaPredecessor::decode(marker)? != request.checkpoint_predecessor()
        || marker.get("admission_open") != Some(&Value::Bool(false))
    {
        return Err(CanonicalError::OperationReused);
    }
    Ok(true)
}
fn overwrite_declarations(schema: &str) -> String {
    // Only declaration lines in the generated catalogue are rewritten. Existing
    // OVERWRITE declarations keep their spelling; function bodies are untouched.
    schema
        .split_inclusive('\n')
        .map(|line| {
            for kind in [
                "TABLE", "FIELD", "INDEX", "EVENT", "FUNCTION", "API", "ACCESS",
            ] {
                let prefix = format!("DEFINE {kind} ");
                if let Some(tail) = line.strip_prefix(&prefix)
                    && !tail.starts_with("OVERWRITE ")
                {
                    return format!("{prefix}OVERWRITE {tail}");
                }
            }
            line.to_owned()
        })
        .collect()
}
/// Only this owner constructs raw privileged transaction text. Ordinary clients
/// have no general unguarded query method.
pub(crate) struct MigrationScript {
    pub(super) sql: String,
    pub(super) bindings: Object,
}
impl MigrationScript {
    fn new(body: &str, expression: &str, bindings: Object) -> Self {
        Self {
            sql: format!(
                "BEGIN;\nIF time::micros()>=$pse_rpc_expires_at {{ THROW 'PSE_REFUSAL:original_schema_installation_deadline_expired'; }};\n{body}\nLET $pse_migration_result=({expression});\nIF time::micros()>=$pse_rpc_expires_at {{ THROW 'PSE_REFUSAL:original_schema_installation_deadline_expired'; }};\nRETURN $pse_migration_result;\nCOMMIT;"
            ),
            bindings,
        }
    }
}
const MIGRATION_SNAPSHOT: &str = "{marker:(SELECT * FROM ONLY canonical_interpretations:current),pending:(SELECT * FROM ONLY canonical_maintenance_operations:current)}";
const PUBLISH_V2_CHECKPOINT: &str = r#"
UPDATE ONLY canonical_interpretations:current SET schema_digest=$migration_v2_digest,schema_version=2dec,schema_epoch=$migration_generation,admission_open=false RETURN NONE;
UPDATE ONLY canonical_maintenance_operations:current SET phase='schema-v2-checkpoint',next=$migration_checkpoint,count=1dec RETURN NONE;
"#;
const VERIFY_V2_CHECKPOINT: &str = r#"
LET $pse_owner=SELECT * FROM ONLY canonical_maintenance_operations:current FOR UPDATE;
LET $pse_marker=SELECT * FROM ONLY canonical_interpretations:current FOR UPDATE;
IF $pse_owner=NONE OR $pse_owner.kind!='schema' OR $pse_owner.operation!=$migration_operation OR $pse_owner.generation!=$migration_generation OR $pse_owner.after!=$migration_request OR $pse_owner.target!=$migration_target OR $pse_owner.source!=$migration_predecessor.schema_digest OR $pse_owner.phase!='schema-v2-checkpoint' OR $pse_owner.next!=$migration_checkpoint OR $pse_owner.count!=1dec OR $pse_owner.complete!=false OR $pse_marker=NONE OR $pse_marker.interpretation!=$migration_predecessor.interpretation OR $pse_marker.schema_digest!=$migration_v2_digest OR $pse_marker.schema_version!=2dec OR $pse_marker.schema_epoch!=$migration_generation OR $pse_marker.issuing_store!=$migration_predecessor.issuing_store OR $pse_marker.payload_incarnation!=$migration_predecessor.payload_incarnation OR $pse_marker.study_module_sha256!=$migration_predecessor.study_module_sha256 OR $pse_marker.admission_open!=false { THROW 'PSE_REFUSAL:schema_migration_original_differs'; };
IF (SELECT * FROM ONLY type::record('canonical_maintenance_activations',$migration_operation))!=NONE { THROW 'PSE_REFUSAL:activation_operation_already_exists_settle_exact_receipt'; };
LET $pse_writers=SELECT key FROM canonical_attempts WHERE phase NOT IN ['terminal','terminal_compacted'] LIMIT 1;
LET $pse_owners=SELECT key FROM canonical_file_owners WHERE !drained LIMIT 1;
LET $pse_pins=SELECT key FROM canonical_protections WHERE !released AND expires_at>time::micros() LIMIT 1;
IF array::len($pse_writers)!=0 OR array::len($pse_owners)!=0 OR array::len($pse_pins)!=0 { THROW 'PSE_REFUSAL:maintenance_protected_source'; };
"#;
const LEGACY_BEGIN: &str = r#"
LET $pse_marker=SELECT * FROM ONLY canonical_interpretations:current FOR UPDATE;
IF $pse_marker=NONE OR $pse_marker.interpretation!=$migration_predecessor.interpretation OR $pse_marker.schema_digest!=$migration_predecessor.schema_digest OR $pse_marker.issuing_store!=$migration_predecessor.issuing_store OR $pse_marker.study_module_sha256!=$migration_predecessor.study_module_sha256 OR $pse_marker.payload_incarnation!=$migration_predecessor.payload_incarnation OR $pse_marker.schema_epoch!=NONE OR $pse_marker.schema_version!=NONE { THROW 'PSE_REFUSAL:activation_predecessor_changed'; };
LET $pse_previous=SELECT * FROM ONLY canonical_maintenance_operations:current FOR UPDATE;
IF $pse_previous!=NONE AND !$pse_previous.complete AND ($pse_previous.operation!=$migration_operation OR $pse_previous.generation!=$migration_generation OR $pse_previous.after!=$migration_request OR $pse_previous.kind!='schema') { THROW 'PSE_REFUSAL:maintenance_generation_changed'; };
LET $pse_migration_saved=UPSERT ONLY canonical_maintenance_operations:current SET key='current',operation=$migration_operation,generation=$migration_generation,phase='schema-legacy-pending',source=$migration_predecessor.schema_digest,target=$migration_target,kind='schema',after=$migration_request,next='',count=0dec,complete=false;
"#;
const LEGACY_QUIESCENT: &str = r#"
LET $pse_owner=SELECT * FROM ONLY canonical_maintenance_operations:current FOR UPDATE;
LET $pse_marker=SELECT * FROM ONLY canonical_interpretations:current FOR UPDATE;
IF $pse_owner=NONE OR $pse_owner.kind!='schema' OR $pse_owner.operation!=$migration_operation OR $pse_owner.generation!=$migration_generation OR $pse_owner.after!=$migration_request OR $pse_owner.target!=$migration_target OR $pse_owner.phase NOT IN ['schema-legacy-pending','schema-applying'] OR $pse_owner.complete OR $pse_marker=NONE OR $pse_marker.schema_epoch!=NONE OR $pse_marker.schema_version!=NONE OR $pse_marker.schema_digest!=$migration_predecessor.schema_digest OR $pse_marker.issuing_store!=$migration_predecessor.issuing_store OR $pse_marker.payload_incarnation!=$migration_predecessor.payload_incarnation OR $pse_marker.study_module_sha256!=$migration_predecessor.study_module_sha256 { THROW 'PSE_REFUSAL:maintenance_generation_changed'; };
LET $pse_writers=SELECT key FROM canonical_attempts WHERE phase NOT IN ['terminal','terminal_compacted'] LIMIT 1;
LET $pse_owners=SELECT key FROM canonical_file_owners WHERE !drained LIMIT 1;
LET $pse_pins=SELECT key FROM canonical_protections WHERE !released AND expires_at>time::micros() LIMIT 1;
IF array::len($pse_writers)!=0 OR array::len($pse_owners)!=0 OR array::len($pse_pins)!=0 { THROW 'PSE_REFUSAL:maintenance_protected_source'; };
UPDATE ONLY canonical_maintenance_operations:current SET phase='schema-applying' RETURN NONE;
"#;
const APPLY_PREDECESSOR: &str = r#"
LET $pse_owner=SELECT * FROM ONLY canonical_maintenance_operations:current FOR UPDATE;
LET $pse_marker=SELECT * FROM ONLY canonical_interpretations:current FOR UPDATE;
IF $pse_owner=NONE OR $pse_owner.kind!='schema' OR $pse_owner.operation!=$migration_operation OR $pse_owner.generation!=$migration_generation OR $pse_owner.after!=$migration_request OR $pse_owner.phase!='schema-applying' OR $pse_owner.complete!=false OR $pse_marker=NONE OR $pse_marker.interpretation!=$migration_predecessor.interpretation OR $pse_marker.schema_digest!=$migration_predecessor.schema_digest OR $pse_marker.issuing_store!=$migration_predecessor.issuing_store OR $pse_marker.study_module_sha256!=$migration_predecessor.study_module_sha256 OR $pse_marker.payload_incarnation!=$migration_predecessor.payload_incarnation OR $pse_marker.schema_epoch!=$migration_predecessor.schema_epoch OR $pse_marker.schema_version!=$migration_predecessor.schema_version OR ($pse_marker.admission_open ?? false) { THROW 'PSE_REFUSAL:activation_predecessor_changed'; };
IF (SELECT * FROM ONLY type::record('canonical_maintenance_activations',$migration_operation))!=NONE { THROW 'PSE_REFUSAL:activation_operation_already_exists_settle_exact_receipt'; };
"#;
const APPLY_SUCCESSOR: &str = r#"
UPDATE ONLY canonical_interpretations:current SET schema_digest=$migration_target,schema_version=$migration_version,schema_epoch=$migration_generation,study_module_sha256=$migration_module_sha256,admission_open=true RETURN NONE;
UPDATE ONLY canonical_maintenance_operations:current SET phase='schema-active',count=1dec,complete=true RETURN NONE;
LET $pse_migration_receipt=CREATE ONLY type::record('canonical_maintenance_activations',$migration_operation) SET key=$migration_operation,input_sha256=$migration_request,predecessor=$migration_predecessor_bytes,interpretation=$migration_predecessor.interpretation,schema_digest=$migration_target,study_module_sha256=$migration_module_sha256,issuing_store=$migration_predecessor.issuing_store;
"#;

/// Fixed-row actual-transaction controls, available only to explicitly selected fixtures.
#[cfg(all(feature = "test-support", feature = "native-service"))]
mod schema_fence_control {
    use super::*;
    use crate::transport::client::{Client, HeldTransaction, RequestContext, Root};
    use std::time::{Instant, SystemTime, UNIX_EPOCH};
    struct State {
        client: Option<Client>,
        held: Option<HeldTransaction>,
    }
    /// One original RPC transaction with an already-admitted fixed guard write.
    /// The fixture retains a clone and explicitly cancels/drains abandoned work.
    #[derive(Clone)]
    pub struct SchemaFenceWrite {
        state: Arc<tokio::sync::Mutex<State>>,
        store: CanonicalStore,
        fence: SchemaFence,
        key: String,
        deadline: Instant,
    }
    impl std::fmt::Debug for SchemaFenceWrite {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("SchemaFenceWrite")
                .field("key", &self.key)
                .finish()
        }
    }
    impl SchemaFenceWrite {
        pub(crate) fn new(ready: &SchemaReady, deadline: Instant) -> Self {
            Self {
                state: Arc::new(tokio::sync::Mutex::new(State {
                    client: None,
                    held: None,
                })),
                store: ready.store().clone(),
                fence: ready.fence().clone(),
                key: format!("schema-fence-probe:{}", uuid::Uuid::new_v4()),
                deadline,
            }
        }
        pub(crate) async fn stage(&self) -> Result<(), CanonicalError> {
            let mut state = self.state.lock().await;
            let options = &self.store.connection_options;
            let address = url::Url::parse(&options.endpoint)
                .map_err(|error| invalid(error.to_string()))?
                .socket_addrs(|| None)
                .map_err(|error| invalid(error.to_string()))?
                .into_iter()
                .next()
                .ok_or_else(|| invalid("fixture address is absent"))?;
            let client = Client::connect(address, super::super::MESSAGE_BYTES).await?;
            // Retain the physical driver before any authentication/transaction dispatch.
            state.client = Some(client.clone());
            client
                .signin(Root {
                    username: options.username.clone(),
                    password: options.password.clone(),
                })
                .request_context(RequestContext::control(self.deadline))
                .await?;
            client
                .use_ns(&options.namespace)
                .use_db(&options.database)
                .request_context(RequestContext::control(self.deadline))
                .await?;
            state.held = Some(client.test_begin(self.deadline).await?);
            let held = state
                .held
                .as_ref()
                .ok_or(CanonicalError::IncompleteResponse)?;
            let remaining = self.deadline.saturating_duration_since(Instant::now());
            let utc = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default();
            let expiry =
                i64::try_from(utc.saturating_add(remaining).as_micros()).unwrap_or(i64::MAX);
            let mut bindings = Object::new();
            bindings.insert("expiry", expiry);
            bindings.insert("expected", self.fence.value());
            bindings.insert("probe", self.key.clone());
            let response = held.query("fn::pse_schema_v1::admit($expiry,$expected,NONE,'mutation'); UPSERT type::record('canonical_guards',$probe) SET key=$probe,generation=1dec; RETURN true;")
                .bind_guarded(bindings).await?;
            crate::transport::complete_response(response).map(|_| ())
        }
        /// Commit only the original held transaction and physically settle its driver.
        /// A conflict remains the original server error; nothing is retried.
        pub async fn commit(&self) -> Result<(), CanonicalError> {
            let mut state = self.state.lock().await;
            let held = state
                .held
                .take()
                .ok_or_else(|| invalid("original held transaction is absent or already settled"))?;
            let result = held.commit().await.map_err(CanonicalError::from);
            let drain = match &state.client {
                Some(client) => client.disconnect().await.map_err(CanonicalError::from),
                None => Ok(()),
            };
            if drain.is_ok() {
                state.client = None;
            }
            match (result, drain) {
                (Ok(()), Ok(())) => Ok(()),
                (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
                (Err(error), Err(cleanup)) => Err(CanonicalError::MigrationDrain {
                    failures: vec![error, cleanup],
                }),
            }
        }
        pub(crate) async fn cancel(&self) -> Result<(), CanonicalError> {
            let mut state = self.state.lock().await;
            let result = match state.held.take() {
                Some(held) => held.cancel().await.map_err(CanonicalError::from),
                None => Ok(()),
            };
            let drain = match &state.client {
                Some(client) => client.disconnect().await.map_err(CanonicalError::from),
                None => Ok(()),
            };
            if drain.is_ok() {
                state.client = None;
            }
            match (result, drain) {
                (Ok(()), Ok(())) => Ok(()),
                (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
                (Err(error), Err(cleanup)) => Err(CanonicalError::MigrationDrain {
                    failures: vec![error, cleanup],
                }),
            }
        }
        /// Read only this control's exact fixed-row effect using an actual observer fence.
        pub async fn persisted(&self, observer: &SchemaReady) -> Result<bool, CanonicalError> {
            if observer.store().deployment_state() != self.store.deployment_state() {
                return Err(invalid("schema fence observer belongs to another fixture"));
            }
            let mut response = bounded_query(observer.store().db.query("RETURN (SELECT * FROM ONLY type::record('canonical_guards',$probe))!=NONE;")
                .bind(("probe", self.key.clone()))).await?;
            response
                .take::<Option<bool>>(0)?
                .ok_or(CanonicalError::IncompleteResponse)
        }
    }
}
#[cfg(all(feature = "test-support", feature = "native-service"))]
pub use schema_fence_control::SchemaFenceWrite;

#[cfg(test)]
mod tests {
    use super::*;
    fn marker() -> Object {
        let mut marker = Object::new();
        marker.insert("interpretation", wire::INTERPRETATION);
        marker.insert("schema_digest", wire::SCHEMA_DIGEST);
        marker.insert("study_module_sha256", "a".repeat(64));
        marker.insert("issuing_store", uuid::Uuid::new_v4().to_string());
        marker.insert("payload_incarnation", uuid::Uuid::new_v4().to_string());
        marker.insert("schema_version", codec::encode_uint(VERSION).unwrap());
        marker.insert("schema_epoch", uuid::Uuid::new_v4().to_string());
        marker.insert("admission_open", true);
        marker
    }
    fn v2_marker() -> Object {
        let mut source = marker();
        source.insert("schema_version", codec::encode_uint(V2_VERSION).unwrap());
        source.insert("schema_digest", V2_DIGEST);
        source
    }
    fn legacy_marker() -> Object {
        let mut source = marker();
        source.remove("schema_version");
        source.remove("schema_epoch");
        source.remove("admission_open");
        source.insert(
            "schema_digest",
            pse_schema::catalog::substrate::PRE_EPOCH_SCHEMA_DIGEST,
        );
        source
    }
    fn request(source: &Object) -> SchemaMigrationRequest {
        SchemaPredecessor::decode(source)
            .unwrap()
            .to_current(
                uuid::Uuid::new_v4(),
                "fixture-bucket".into(),
                "fixture-backend".into(),
            )
            .unwrap()
    }
    fn activation(request: &SchemaMigrationRequest) -> Object {
        let mut receipt = Object::new();
        receipt.insert("key", request.operation.clone());
        receipt.insert("input_sha256", request.fingerprint().unwrap());
        receipt.insert(
            "predecessor",
            surrealdb_types::Bytes::from(bytes(&request.predecessor).unwrap()),
        );
        receipt.insert("interpretation", request.predecessor.interpretation.clone());
        receipt.insert("schema_digest", request.target_digest.clone());
        receipt.insert(
            "study_module_sha256",
            request.target_module_hash().to_owned(),
        );
        receipt.insert("issuing_store", request.predecessor.issuing_store.clone());
        receipt
    }
    #[test]
    fn schema_v2_predecessor_requires_exact_version_digest_epoch_and_interpretation() {
        let source = v2_marker();
        let expected = SchemaPredecessor::decode(&source).unwrap();
        assert!(expected.supported());
        assert!(!expected.legacy());
        assert!(
            !SchemaPredecessor::decode(&marker())
                .unwrap()
                .versioned(V2_VERSION, V2_DIGEST)
        );
        assert!(SchemaPredecessor::decode(&marker()).unwrap().supported());
        assert!(SchemaFence::from_marker(&source).is_err());
        let migration = request(&source);
        assert_eq!(migration.predecessor, expected);
        assert_eq!(migration.target_version, 3);
        assert_eq!(migration.predecessor.interpretation, "pse.substrate.v10");
        for (key, value) in [
            ("schema_version", codec::encode_uint(1).unwrap()),
            ("schema_version", codec::encode_uint(VERSION + 1).unwrap()),
            ("schema_digest", Value::String("unknown".into())),
            ("schema_epoch", Value::None),
            ("schema_epoch", Value::String("invalid".into())),
            ("interpretation", Value::String("pse.substrate.v11".into())),
            ("issuing_store", Value::String("invalid".into())),
        ] {
            let mut changed = source.clone();
            changed.insert(key, value);
            assert!(
                !SchemaPredecessor::decode(&changed).unwrap().supported(),
                "{key}"
            );
        }
    }
    #[test]
    fn schema_migration_chain_retains_exact_frozen_v2_before_current() {
        verify_frozen_declarations().unwrap();
        assert_eq!(
            V2_DIGEST,
            "04c2b46a1ee5d153244089e816e1fc2586c1208a8fce5e48998f22a091c55bcc"
        );
        assert_eq!(SCHEMA_V2_DECLARATION.len(), 413340);
        let migration = request(&legacy_marker());
        let first = v2_checkpoint_body(&migration).unwrap();
        assert!(first.starts_with(APPLY_PREDECESSOR));
        assert!(first.contains(&overwrite_declarations(SCHEMA_V2_DECLARATION)));
        assert!(first.ends_with(PUBLISH_V2_CHECKPOINT));
        assert!(PUBLISH_V2_CHECKPOINT.contains("admission_open=false"));
        assert!(!PUBLISH_V2_CHECKPOINT.contains("canonical_maintenance_activations"));
        let final_edge = final_application_body(&migration).unwrap();
        assert!(final_edge.starts_with(VERIFY_V2_CHECKPOINT));
        assert!(final_edge.contains(&overwrite_declarations(wire::SCHEMA)));
        assert!(!final_edge.contains(PUBLISH_V2_CHECKPOINT));
        assert!(final_edge.ends_with(APPLY_SUCCESSOR));
        assert_eq!(
            migration_declarations(&request(&v2_marker())).unwrap(),
            overwrite_declarations(wire::SCHEMA)
        );
    }
    fn checkpoint_snapshot(request: &SchemaMigrationRequest) -> Object {
        let Value::Object(mut marker) = request.checkpoint_predecessor().value().unwrap() else {
            unreachable!()
        };
        marker.insert("admission_open", false);
        let mut owner = Object::new();
        for (key, value) in [
            ("operation", request.operation.clone()),
            ("generation", request.successor_epoch.clone()),
            ("after", request.fingerprint().unwrap()),
            ("target", request.target_digest.clone()),
            ("source", request.predecessor.schema_digest.clone()),
            ("next", request.checkpoint_fingerprint().unwrap()),
            ("kind", "schema".into()),
            ("phase", "schema-v2-checkpoint".into()),
        ] {
            owner.insert(key, value);
        }
        owner.insert("count", codec::encode_uint(1).unwrap());
        owner.insert("complete", false);
        let mut snapshot = Object::new();
        snapshot.insert("marker", marker);
        snapshot.insert("pending", owner);
        snapshot
    }
    #[test]
    fn schema_v2_checkpoint_binds_original_request_and_closed_intermediate_identity() {
        let original = request(&legacy_marker());
        let snapshot = checkpoint_snapshot(&original);
        assert!(verify_v2_checkpoint(&snapshot, &original).unwrap());
        for field in [
            "operation",
            "generation",
            "after",
            "target",
            "source",
            "next",
            "kind",
        ] {
            let mut changed = snapshot.clone();
            let Some(Value::Object(owner)) = changed.get_mut("pending") else {
                unreachable!()
            };
            owner.insert(field, "different");
            assert!(
                verify_v2_checkpoint(&changed, &original).is_err(),
                "{field}"
            );
        }
        for (field, value) in [
            ("complete", Value::Bool(true)),
            ("count", codec::encode_uint(2).unwrap()),
        ] {
            let mut changed = snapshot.clone();
            let Some(Value::Object(owner)) = changed.get_mut("pending") else {
                unreachable!()
            };
            owner.insert(field, value);
            assert!(
                verify_v2_checkpoint(&changed, &original).is_err(),
                "{field}"
            );
        }
        for (field, value) in [
            ("schema_version", codec::encode_uint(VERSION).unwrap()),
            ("schema_digest", Value::String("unknown".into())),
            (
                "schema_epoch",
                Value::String(uuid::Uuid::new_v4().to_string()),
            ),
            (
                "issuing_store",
                Value::String(uuid::Uuid::new_v4().to_string()),
            ),
            (
                "payload_incarnation",
                Value::String(uuid::Uuid::new_v4().to_string()),
            ),
            ("study_module_sha256", Value::String("other-module".into())),
            ("admission_open", Value::Bool(true)),
        ] {
            let mut changed = snapshot.clone();
            let Some(Value::Object(marker)) = changed.get_mut("marker") else {
                unreachable!()
            };
            marker.insert(field, value);
            assert!(
                verify_v2_checkpoint(&changed, &original).is_err(),
                "{field}"
            );
        }
        let foreign = request(&legacy_marker());
        assert!(verify_v2_checkpoint(&snapshot, &foreign).is_err());
        let new_from_v2 = request(&v2_marker());
        assert!(verify_v2_checkpoint(&snapshot, &new_from_v2).is_err());
        let mut historical = original.clone();
        historical.target_version = V2_VERSION;
        historical.target_digest = V2_DIGEST.into();
        assert!(verify_v2_checkpoint(&snapshot, &historical).is_err());
        // A lost ACK's exact readback grants only the remaining edge, never final receipt/readiness.
        assert_ne!(original.predecessor, original.checkpoint_predecessor());
        assert!(!snapshot.contains_key("receipt"));
        assert!(
            SchemaFence::from_marker(match snapshot.get("marker").unwrap() {
                Value::Object(marker) => marker,
                _ => unreachable!(),
            })
            .is_err()
        );
    }
    #[test]
    fn schema_migration_v2_original_request_and_receipt_are_not_retargeted() {
        let mut original = request(&legacy_marker());
        original.target_version = V2_VERSION;
        original.target_digest = V2_DIGEST.into();
        original.validate().unwrap();
        let recorded = bytes(&original).unwrap();
        assert!(
            serde_json::from_slice::<serde_json::Value>(&recorded)
                .unwrap()
                .get("target_module")
                .is_none()
        );
        let fingerprint = original.fingerprint().unwrap();
        let decoded: SchemaMigrationRequest = serde_json::from_slice(&recorded).unwrap();
        assert_eq!(bytes(&decoded).unwrap(), recorded);
        assert_eq!(decoded.fingerprint().unwrap(), fingerprint);
        let settled = verified_activation(activation(&original), &decoded).unwrap();
        assert_eq!(settled.version(), V2_VERSION);
        assert_eq!(settled.digest(), V2_DIGEST);
        assert_eq!(bytes(settled.request()).unwrap(), recorded);
        assert!(require_current_target(&decoded).is_err());
        let declarations = migration_declarations(&decoded).unwrap();
        assert!(declarations.starts_with(&overwrite_declarations(SCHEMA_V2_DECLARATION)));
        assert_eq!(declarations, overwrite_declarations(SCHEMA_V2_DECLARATION));
        let mut retargeted = decoded.clone();
        retargeted.target_version = VERSION;
        retargeted.target_digest = wire::SCHEMA_DIGEST.into();
        assert!(verify_activation(activation(&original), &retargeted).is_err());
        assert_ne!(retargeted.fingerprint().unwrap(), fingerprint);
        let current = request(&marker());
        let mut downgrade = current.clone();
        downgrade.target_version = V2_VERSION;
        downgrade.target_digest = V2_DIGEST.into();
        assert!(downgrade.validate().is_err());
    }
    #[test]
    fn schema_migration_receipt_binds_original_issuer_payload_and_operation() {
        let original = request(&v2_marker());
        let recorded = bytes(&original).unwrap();
        let declarations = migration_declarations(&original).unwrap();
        assert_eq!(bytes(&original).unwrap(), recorded);
        assert!(
            !declarations
                .contains("UPDATE ONLY canonical_interpretations:current SET issuing_store=")
        );
        verify_activation(activation(&original), &original).unwrap();
        for key in [
            "key",
            "input_sha256",
            "issuing_store",
            "schema_digest",
            "study_module_sha256",
        ] {
            let mut changed = activation(&original);
            changed.insert(key, "changed");
            assert!(verify_activation(changed, &original).is_err(), "{key}");
        }
        let mut changed = original.clone();
        changed.predecessor.payload_incarnation = uuid::Uuid::new_v4().to_string();
        assert!(verify_activation(activation(&original), &changed).is_err());
    }
    #[test]
    fn schema_construction_requires_actual_supported_open_marker() {
        let source = marker();
        let opened = SchemaFence::from_marker(&source).unwrap();
        assert_eq!(opened.version(), VERSION);
        for (key, value) in [
            ("admission_open", Value::Bool(false)),
            ("schema_epoch", Value::String("invalid".into())),
            ("schema_digest", Value::String("unrecognized".into())),
            ("issuing_store", Value::String("invalid".into())),
        ] {
            let mut changed = source.clone();
            changed.insert(key, value);
            assert!(SchemaFence::from_marker(&changed).is_err(), "{key}");
        }
        for (key, expected) in [
            ("interpretation", wire::INTERPRETATION),
            ("schema_digest", wire::SCHEMA_DIGEST),
        ] {
            let mut changed = source.clone();
            changed.insert(key, "unknown-reader");
            assert!(
                matches!(SchemaFence::from_marker(&changed), Err(CanonicalError::Interpretation { expected: actual, observed }) if actual == expected && observed == "unknown-reader")
            );
        }
    }
    #[test]
    fn schema_migration_drain_classification_retains_nested_effect_uncertainty() {
        let definite = CanonicalError::MigrationDrain {
            failures: vec![invalid("original native owner was not joined")],
        };
        assert!(definite.is_infrastructure());
        assert!(!definite.effect_is_unknown());
        assert!(!definite.retryable());
        let uncertain = CanonicalError::MigrationDrain {
            failures: vec![
                definite,
                CanonicalError::MigrationDrain {
                    failures: vec![CanonicalError::Timeout],
                },
            ],
        };
        assert!(uncertain.is_infrastructure());
        assert!(uncertain.effect_is_unknown());
        assert!(!uncertain.retryable());
    }
    #[test]
    fn schema_migration_requests_bind_exact_predecessor_operation_and_target() {
        let source = SchemaPredecessor::decode(&marker()).unwrap();
        let request = source
            .to_current(
                uuid::Uuid::new_v4(),
                "fixture-bucket".into(),
                "fixture-backend".into(),
            )
            .unwrap();
        let recorded = bytes(&request).unwrap();
        let decoded: SchemaMigrationRequest = serde_json::from_slice(&recorded).unwrap();
        assert_eq!(request, decoded);
        assert_eq!(
            request.fingerprint().unwrap(),
            decoded.fingerprint().unwrap()
        );
        let mut changed = request.clone();
        changed.target_digest = "unknown".into();
        assert!(changed.validate().is_err());
        let mut changed = request.clone();
        changed.successor_epoch = changed.predecessor.schema_epoch.clone().unwrap();
        assert!(changed.validate().is_err());
    }
    #[test]
    fn schema_legacy_predecessor_is_exact_and_cannot_mint_ready_capability() {
        assert_eq!(
            hash(PRE_EPOCH_DECLARATION.as_bytes()),
            pse_schema::catalog::substrate::PRE_EPOCH_DECLARATION_SHA256
        );
        let mut legacy = marker();
        legacy.remove("schema_version");
        legacy.remove("schema_epoch");
        legacy.remove("admission_open");
        legacy.insert(
            "schema_digest",
            pse_schema::catalog::substrate::PRE_EPOCH_SCHEMA_DIGEST,
        );
        let source = SchemaPredecessor::decode(&legacy).unwrap();
        assert!(source.legacy());
        assert!(
            source
                .to_current(
                    uuid::Uuid::new_v4(),
                    "fixture-bucket".into(),
                    "fixture-backend".into()
                )
                .is_ok()
        );
        assert!(SchemaFence::from_marker(&legacy).is_err());
        legacy.insert("schema_digest", "another-historical-layout");
        assert!(
            SchemaPredecessor::decode(&legacy)
                .unwrap()
                .to_current(
                    uuid::Uuid::new_v4(),
                    "fixture-bucket".into(),
                    "fixture-backend".into()
                )
                .is_err()
        );
    }
    #[test]
    #[cfg(feature = "canonical-maintenance")]
    fn schema_migration_transactions_parse_with_pinned_server_grammar() {
        let applications = [legacy_marker(), v2_marker(), marker()]
            .map(|source| final_application_body(&request(&source)).unwrap());
        let first = v2_checkpoint_body(&request(&legacy_marker())).unwrap();
        for (name, body, expression) in [
            ("closed v2 publication", first.as_str(), MIGRATION_SNAPSHOT),
            ("checkpoint quiescent resume", VERIFY_V2_CHECKPOINT, "true"),
        ] {
            let script = MigrationScript::new(body, expression, Object::new());
            if let Err(error) = surrealdb_syn::parse(&script.sql) {
                panic!("{name}: {error}");
            }
        }
        for application in &applications {
            let script = MigrationScript::new(application, "$pse_migration_receipt", Object::new());
            if let Err(error) = surrealdb_syn::parse(&script.sql) {
                panic!("ordered final edge: {error}");
            }
        }
        for (name, body, result) in [
            ("legacy pending", LEGACY_BEGIN, "$pse_migration_saved"),
            ("legacy quiescence", LEGACY_QUIESCENT, "true"),
            (
                "current declaration and activation",
                applications[2].as_str(),
                "$pse_migration_receipt",
            ),
        ] {
            let script = MigrationScript::new(body, result, Object::new());
            if let Err(error) = surrealdb_syn::parse(&script.sql) {
                panic!("{name}: {error}");
            }
        }
    }
    #[test]
    fn schema_overwrite_rewrites_declaration_only_once() {
        let input = "DEFINE TABLE canonical_test SCHEMAFULL;\nDEFINE FIELD OVERWRITE key ON canonical_test TYPE string;\n    RETURN 'DEFINE TABLE quoted';\n";
        let output = overwrite_declarations(input);
        assert!(output.starts_with("DEFINE TABLE OVERWRITE canonical_test"));
        assert!(output.contains("DEFINE FIELD OVERWRITE key"));
        assert!(!output.contains("OVERWRITE OVERWRITE"));
        assert!(output.contains("RETURN 'DEFINE TABLE quoted'"));
    }
}
