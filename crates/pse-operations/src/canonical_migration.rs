// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Version-bound canonical construction and explicit quiescent schema evolution.
//!
//! The frozen pre-epoch declaration is the sole supported historical predecessor,
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

/// Preserved generated predecessor declaration for the one adjacent native migration.
/// Its 407 KB are needed to construct a real prior-version disposable database.
pub const PRE_EPOCH_DECLARATION: &str = include_str!("migrations/substrate-v10-pre-epoch.surql");
const VERSION: u64 = pse_schema::catalog::substrate::SCHEMA_VERSION;
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
        let store = Self::connect(options).await?;
        let installed = async {
            if !store.initialization_required().await? {
                return Err(invalid("predecessor installation requires a fresh owned canonical catalog"));
            }
            store.provision_context().await?;
            if hash(PRE_EPOCH_DECLARATION.as_bytes()) != pse_schema::catalog::substrate::PRE_EPOCH_DECLARATION_SHA256 {
                return Err(invalid("frozen predecessor declaration differs from its recorded bytes"));
            }
            let module = super::study_module::StudyModuleReceipt::read(&store.state_path)?;
            let body = format!("{}{}\nCREATE canonical_interpretations:current SET key='current',interpretation=$interpretation,schema_digest=$schema_digest,study_module_sha256=$study_module_sha256,issuing_store=<string>rand::uuid::v4(),payload_incarnation=$payload_incarnation;\nCREATE canonical_file_admission:current SET key='current',storage_incarnation=$payload_incarnation,frozen=false,gc_frozen=false;", module.declarations(false), PRE_EPOCH_DECLARATION);
            let mut bindings = Object::new();
            bindings.insert("interpretation", wire::INTERPRETATION);
            bindings.insert("schema_digest", pse_schema::catalog::substrate::PRE_EPOCH_SCHEMA_DIGEST);
            bindings.insert("study_module_backend", module.backend()?);
            bindings.insert("study_module_sha256", module.sha256.clone());
            bindings.insert("payload_incarnation", store.files.incarnation.clone());
            let maintenance = SchemaMaintenance { store: store.clone() };
            if maintenance.execute::<Option<bool>>(&body, "true", bindings).await? != Some(true) {
                return Err(CanonicalError::IncompleteResponse);
            }
            let actual = maintenance.inspect().await?;
            if !actual.predecessor.legacy() || actual.predecessor.study_module_sha256 != module.sha256
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
    fn supported(&self) -> bool {
        self.interpretation == wire::INTERPRETATION
            && uuid::Uuid::parse_str(&self.issuing_store).is_ok()
            && (self.legacy()
                || (self.schema_version == Some(VERSION)
                    && self.schema_digest == wire::SCHEMA_DIGEST
                    && self
                        .schema_epoch
                        .as_ref()
                        .is_some_and(|epoch| uuid::Uuid::parse_str(epoch).is_ok())))
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
        })
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
}
impl SchemaMigrationRequest {
    /// Original migration identity used by its immutable effect receipt.
    pub fn operation(&self) -> &str {
        &self.operation
    }
    fn validate(&self) -> Result<(), CanonicalError> {
        if !self.predecessor.supported()
            || self.target_version != VERSION
            || self.target_digest != wire::SCHEMA_DIGEST
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
    fn fingerprint(&self) -> Result<String, CanonicalError> {
        Ok(hash(&bytes(self)?))
    }
    fn bindings(&self) -> Result<Object, CanonicalError> {
        let mut bindings = Object::new();
        bindings.insert("migration_operation", self.operation.clone());
        bindings.insert("migration_generation", self.successor_epoch.clone());
        bindings.insert("migration_request", self.fingerprint()?);
        bindings.insert("migration_predecessor", self.predecessor.value()?);
        bindings.insert("migration_target", self.target_digest.clone());
        bindings.insert(
            "migration_version",
            codec::encode_uint(self.target_version)?,
        );
        bindings.insert("bucket", self.module_bucket.clone());
        bindings.insert("backend", self.module_backend.clone());
        bindings.insert(
            "migration_predecessor_bytes",
            surrealdb_types::Bytes::from(bytes(&self.predecessor)?),
        );
        Ok(bindings)
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
    /// Observe actual marker and pending native owner, without interpreting a config flag as readiness.
    pub async fn inspect(&self) -> Result<SchemaMigrationState, CanonicalError> {
        let value: Object = self.execute_object("", "{marker:(SELECT * FROM ONLY canonical_interpretations:current),pending:(SELECT * FROM ONLY canonical_maintenance_operations:current)}", Object::new()).await?;
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
        let module = super::study_module::StudyModuleReceipt::read(&self.store.state_path)?;
        if module.sha256 != observed.predecessor.study_module_sha256 {
            return Err(CanonicalError::Quiesced);
        }
        observed
            .predecessor
            .to_current(operation, module.bucket(), module.backend()?)
    }
    fn require_original_module(
        &self,
        request: &SchemaMigrationRequest,
    ) -> Result<(), CanonicalError> {
        let module = super::study_module::StudyModuleReceipt::read(&self.store.state_path)?;
        if module.sha256 != request.predecessor.study_module_sha256
            || module.bucket() != request.module_bucket
            || module.backend()? != request.module_backend
        {
            return Err(CanonicalError::Quiesced);
        }
        Ok(())
    }
    /// Read the original immutable activation receipt after restart or lost ACK.
    /// Absence grants no replay permission; an unreadable receipt remains an error.
    pub async fn reconcile(
        &self,
        request: &SchemaMigrationRequest,
    ) -> Result<Option<SchemaReady>, CanonicalError> {
        request.validate()?;
        self.require_original_module(request)?;
        let receipt: Option<Object> = self.execute("", "SELECT * FROM ONLY type::record('canonical_maintenance_activations',$migration_operation)", request.bindings()?).await?;
        let Some(receipt) = receipt else {
            return Ok(None);
        };
        verify_activation(receipt, request)?;
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
        Ok(Some(ready))
    }
    /// Close native epoch admission before draining. A legacy pending phase explicitly
    /// requires B's exclusive maintenance boundary; it cannot fence historical binaries.
    pub async fn begin(
        &self,
        request: SchemaMigrationRequest,
    ) -> Result<PendingSchemaMigration, CanonicalError> {
        request.validate()?;
        self.require_original_module(&request)?;
        if hash(PRE_EPOCH_DECLARATION.as_bytes())
            != pse_schema::catalog::substrate::PRE_EPOCH_DECLARATION_SHA256
        {
            return Err(invalid(
                "frozen predecessor declaration differs from its recorded bytes",
            ));
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
        let result: Value = if self.request.predecessor.legacy() {
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
    /// Apply the ordered current declaration and marker transformation atomically.
    /// A failed/unknown acknowledgement is reconciled through the original activation
    /// receipt. Never replay a transformation because readback itself failed.
    pub async fn apply(self) -> Result<SchemaReady, CanonicalError> {
        let request = &self.request;
        self.maintenance.require_original_module(request)?;
        let bindings = request.bindings()?;
        let validation_body = super::study_module::CATALOG_VALIDATION_BODY;
        let validation_predicate = super::study_module::CATALOG_VALIDATION_PREDICATE;
        let body = format!(
            "{APPLY_PREDECESSOR}\n{}\n{validation_body}\nIF !({validation_predicate}) {{ THROW 'PSE_REFUSAL:schema_migration_original_differs'; }};\n{APPLY_SUCCESSOR}",
            overwrite_declarations(wire::SCHEMA)
        );
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
        verify_activation(receipt, request)?;
        let ready =
            CanonicalStore::connect_ready(&self.maintenance.store.connection_options).await?;
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
        || receipt.study_module_sha256 != request.predecessor.study_module_sha256
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
const LEGACY_BEGIN: &str = r#"
LET $pse_marker=SELECT * FROM ONLY canonical_interpretations:current FOR UPDATE;
IF $pse_marker=NONE OR $pse_marker.interpretation!=$migration_predecessor.interpretation OR $pse_marker.schema_digest!=$migration_predecessor.schema_digest OR $pse_marker.issuing_store!=$migration_predecessor.issuing_store OR $pse_marker.study_module_sha256!=$migration_predecessor.study_module_sha256 OR $pse_marker.payload_incarnation!=$migration_predecessor.payload_incarnation OR $pse_marker.schema_epoch!=NONE OR $pse_marker.schema_version!=NONE { THROW 'PSE_REFUSAL:activation_predecessor_changed'; };
LET $pse_previous=SELECT * FROM ONLY canonical_maintenance_operations:current FOR UPDATE;
IF $pse_previous!=NONE AND !$pse_previous.complete AND ($pse_previous.operation!=$migration_operation OR $pse_previous.generation!=$migration_generation OR $pse_previous.after!=$migration_request OR $pse_previous.kind!='schema') { THROW 'PSE_REFUSAL:maintenance_generation_changed'; };
LET $pse_migration_saved=UPSERT ONLY canonical_maintenance_operations:current SET key='current',operation=$migration_operation,generation=$migration_generation,phase='schema-legacy-pending',source=$migration_predecessor.schema_digest,target=$migration_target,kind='schema',after=$migration_request,next='',count=0dec,complete=false;
"#;
const LEGACY_QUIESCENT: &str = r#"
LET $pse_owner=SELECT * FROM ONLY canonical_maintenance_operations:current FOR UPDATE;
IF $pse_owner=NONE OR $pse_owner.kind!='schema' OR $pse_owner.operation!=$migration_operation OR $pse_owner.generation!=$migration_generation OR $pse_owner.after!=$migration_request OR $pse_owner.complete { THROW 'PSE_REFUSAL:maintenance_generation_changed'; };
LET $pse_writers=SELECT key FROM canonical_attempts WHERE phase NOT IN ['terminal','terminal_compacted'] LIMIT 1;
LET $pse_owners=SELECT key FROM canonical_file_owners WHERE !drained LIMIT 1;
LET $pse_pins=SELECT key FROM canonical_protections WHERE !released AND expires_at>time::micros() LIMIT 1;
IF array::len($pse_writers)!=0 OR array::len($pse_owners)!=0 OR array::len($pse_pins)!=0 { THROW 'PSE_REFUSAL:maintenance_protected_source'; };
UPDATE ONLY canonical_maintenance_operations:current SET phase='schema-applying' RETURN NONE;
"#;
const APPLY_PREDECESSOR: &str = r#"
LET $pse_owner=SELECT * FROM ONLY canonical_maintenance_operations:current FOR UPDATE;
LET $pse_marker=SELECT * FROM ONLY canonical_interpretations:current FOR UPDATE;
IF $pse_owner=NONE OR $pse_owner.kind!='schema' OR $pse_owner.operation!=$migration_operation OR $pse_owner.generation!=$migration_generation OR $pse_owner.after!=$migration_request OR $pse_owner.phase!='schema-applying' OR $pse_owner.complete OR $pse_marker=NONE OR $pse_marker.interpretation!=$migration_predecessor.interpretation OR $pse_marker.schema_digest!=$migration_predecessor.schema_digest OR $pse_marker.issuing_store!=$migration_predecessor.issuing_store OR $pse_marker.study_module_sha256!=$migration_predecessor.study_module_sha256 OR $pse_marker.payload_incarnation!=$migration_predecessor.payload_incarnation OR $pse_marker.schema_epoch!=$migration_predecessor.schema_epoch OR $pse_marker.schema_version!=$migration_predecessor.schema_version OR ($pse_marker.admission_open ?? false) { THROW 'PSE_REFUSAL:activation_predecessor_changed'; };
IF (SELECT * FROM ONLY type::record('canonical_maintenance_activations',$migration_operation))!=NONE { THROW 'PSE_REFUSAL:activation_operation_already_exists_settle_exact_receipt'; };
"#;
const APPLY_SUCCESSOR: &str = r#"
UPDATE ONLY canonical_interpretations:current SET schema_digest=$migration_target,schema_version=$migration_version,schema_epoch=$migration_generation,admission_open=true RETURN NONE;
UPDATE ONLY canonical_maintenance_operations:current SET phase='schema-active',count=1dec,complete=true RETURN NONE;
LET $pse_migration_receipt=CREATE ONLY type::record('canonical_maintenance_activations',$migration_operation) SET key=$migration_operation,input_sha256=$migration_request,predecessor=$migration_predecessor_bytes,interpretation=$migration_predecessor.interpretation,schema_digest=$migration_target,study_module_sha256=$migration_predecessor.study_module_sha256,issuing_store=$migration_predecessor.issuing_store;
"#;

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
        let validation_body = super::super::study_module::CATALOG_VALIDATION_BODY;
        let validation_predicate = super::super::study_module::CATALOG_VALIDATION_PREDICATE;
        let application = format!(
            "{APPLY_PREDECESSOR}\n{}\n{validation_body}\nIF !({validation_predicate}) {{ THROW 'PSE_REFUSAL:schema_migration_original_differs'; }};\n{APPLY_SUCCESSOR}",
            overwrite_declarations(wire::SCHEMA)
        );
        for (name, body, result) in [
            ("legacy pending", LEGACY_BEGIN, "$pse_migration_saved"),
            ("legacy quiescence", LEGACY_QUIESCENT, "true"),
            (
                "current declaration and activation",
                application.as_str(),
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
