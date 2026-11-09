// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Version-bound canonical construction and explicit quiescent schema evolution.
//! Catalog maintenance starts from the current schema and exact observed epoch.
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
pub(crate) fn settlement_scope<F, T>(
    fence: &AttemptFence,
    future: F,
) -> impl Future<Output = Result<T, CanonicalError>>
where
    F: Future<Output = Result<T, CanonicalError>>,
{
    // Close/reconcile/seal compose large request futures. Keep their one child
    // outside the async argument/TaskLocalFuture state instead of duplicating it.
    let future = Box::pin(future);
    async move {
        let mut value = Object::new();
        value.insert("run", fence.run().to_owned());
        value.insert("attempt", fence.attempt().to_owned());
        value.insert("generation", codec::encode_uint(fence.generation())?);
        SETTLEMENT.scope(Value::Object(value), future).await
    }
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
        if predecessor.schema_version != VERSION
            || marker.get("admission_open") != Some(&Value::Bool(true))
        {
            return Err(invalid("current canonical schema must be open"));
        }
        let epoch = predecessor.schema_epoch;
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
    schema_version: u64,
    schema_epoch: String,
}
impl SchemaPredecessor {
    fn decode(marker: &Object) -> Result<Self, CanonicalError> {
        Ok(Self {
            interpretation: string(marker, "interpretation")?,
            schema_digest: string(marker, "schema_digest")?,
            study_module_sha256: string(marker, "study_module_sha256")?,
            issuing_store: string(marker, "issuing_store")?,
            payload_incarnation: string(marker, "payload_incarnation")?,
            schema_version: codec::decode_uint(
                marker.get("schema_version").cloned().unwrap_or(Value::None),
            )?,
            schema_epoch: string(marker, "schema_epoch")?,
        })
    }
    fn supported(&self) -> bool {
        self.interpretation == wire::INTERPRETATION
            && self.schema_version == VERSION
            && self.schema_digest == wire::SCHEMA_DIGEST
            && uuid::Uuid::parse_str(&self.schema_epoch).is_ok()
            && uuid::Uuid::parse_str(&self.issuing_store).is_ok()
    }
    fn value(&self) -> Result<Value, CanonicalError> {
        let mut value = Object::new();
        value.insert("interpretation", self.interpretation.clone());
        value.insert("schema_digest", self.schema_digest.clone());
        value.insert("study_module_sha256", self.study_module_sha256.clone());
        value.insert("issuing_store", self.issuing_store.clone());
        value.insert("payload_incarnation", self.payload_incarnation.clone());
        value.insert("schema_version", codec::encode_uint(self.schema_version)?);
        value.insert("schema_epoch", self.schema_epoch.clone());
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
            return Err(invalid("maintenance requires the current catalog"));
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
            || self.predecessor.schema_epoch == self.successor_epoch
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
        bindings.insert("study_module_backend", self.module_backend.clone());
        bindings.insert(
            "migration_module_sha256",
            self.predecessor.study_module_sha256.clone(),
        );
        bindings.insert(
            "migration_predecessor_bytes",
            surrealdb_types::Bytes::from(bytes(&self.predecessor)?),
        );
        Ok(bindings)
    }
}
/// Verified immutable activation of the exact original migration intent.
/// Receipt verification is independent of the currently open epoch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchemaMigrationReceipt {
    request: SchemaMigrationRequest,
}
impl SchemaMigrationReceipt {
    /// Original serializable request, retaining its operation, predecessor and target.
    pub fn request(&self) -> &SchemaMigrationRequest {
        &self.request
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
    async fn read_activation(
        &self,
        request: &SchemaMigrationRequest,
    ) -> Result<Option<Object>, CanonicalError> {
        let mut bindings = Object::new();
        bindings.insert("migration_operation", request.operation.clone());
        self.execute(
            "",
            "SELECT * FROM ONLY type::record('canonical_maintenance_activations',$migration_operation)",
            bindings,
        )
        .await
    }
    async fn snapshot(&self) -> Result<Object, CanonicalError> {
        self.execute_object("", MIGRATION_SNAPSHOT, Object::new())
            .await
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
                "maintenance requires the current catalog; initialize a fresh catalog",
            ));
        }
        if predecessor.payload_incarnation != self.store.files.incarnation {
            return Err(invalid("maintenance catalog has a different payload incarnation"));
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
        let module = super::study_module::StudyModuleReceipt::read(&self.store.state_path)?;
        if module.sha256 != observed.predecessor.study_module_sha256 {
            return Err(CanonicalError::Quiesced);
        }
        let request =
            observed
                .predecessor
                .to_current(operation, module.bucket(), module.backend()?)?;
        request.validate()?;
        Ok(request)
    }
    fn require_original_module(
        &self,
        request: &SchemaMigrationRequest,
    ) -> Result<super::study_module::StudyModuleReceipt, CanonicalError> {
        let module = super::study_module::StudyModuleReceipt::read(&self.store.state_path)?;
        if request.predecessor.payload_incarnation != self.store.files.incarnation
            || module.sha256 != request.predecessor.study_module_sha256
            || module.bucket() != request.module_bucket
            || module.backend()? != request.module_backend
        {
            return Err(CanonicalError::Quiesced);
        }
        Ok(module)
    }
    /// Read the original immutable activation receipt after restart or lost ACK,
    /// without opening a capability. Absence grants no replay permission.
    pub async fn reconcile_receipt(
        &self,
        request: &SchemaMigrationRequest,
    ) -> Result<Option<SchemaMigrationReceipt>, CanonicalError> {
        request.validate()?;
        self.require_original_module(request)?;
        self.read_activation(request)
            .await?
            .map(|receipt| verified_activation(receipt, request))
            .transpose()
    }
    /// Reconcile the original activation and reconstruct readiness at its exact epoch.
    pub async fn reconcile(
        &self,
        request: &SchemaMigrationRequest,
    ) -> Result<Option<SchemaReady>, CanonicalError> {
        let Some(_receipt) = self.reconcile_receipt(request).await? else {
            return Ok(None);
        };
        self.current_ready(request).await.map(Some)
    }
    async fn current_ready(
        &self,
        request: &SchemaMigrationRequest,
    ) -> Result<SchemaReady, CanonicalError> {
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
    /// Close native epoch admission before draining.
    pub async fn begin(
        &self,
        request: SchemaMigrationRequest,
    ) -> Result<PendingSchemaMigration, CanonicalError> {
        request.validate()?;
        self.require_original_module(&request)?;
        let observed = self.inspect().await?;
        if observed.predecessor != request.predecessor {
            return Err(invalid("migration predecessor changed"));
        }
        let row = self.execute_object("", "fn::pse_schema_v1::close($pse_rpc_expires_at,$migration_predecessor,$migration_operation,$migration_generation,$migration_target,$migration_request)", request.bindings()?).await?;
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
        let result: Value = self.maintenance.execute("", "fn::pse_schema_v1::quiescent($pse_rpc_expires_at,$migration_predecessor,$migration_operation,$migration_generation,$migration_request)", self.request.bindings()?).await?;
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
    /// Apply the current catalog activation and reconstruct readiness.
    pub async fn apply(self) -> Result<SchemaReady, CanonicalError> {
        let maintenance = self.maintenance.clone();
        let receipt = self.apply_receipt().await?;
        maintenance.current_ready(receipt.request()).await
    }
    /// Atomically publish the activation and its original receipt. Reconcile an
    /// unknown acknowledgement through that receipt before considering another write.
    pub async fn apply_receipt(self) -> Result<SchemaMigrationReceipt, CanonicalError> {
        self.apply_receipt_inner(
            #[cfg(test)]
            None,
        )
        .await
    }

    #[cfg(all(test, feature = "native-service"))]
    pub(crate) async fn apply_receipt_with_lost_ack_for_test(
        self,
        committed: &mut Option<SchemaMigrationReceipt>,
    ) -> Result<SchemaMigrationReceipt, CanonicalError> {
        self.apply_receipt_inner(Some(committed)).await
    }

    async fn apply_receipt_inner(
        self,
        #[cfg(test)] committed: Option<&mut Option<SchemaMigrationReceipt>>,
    ) -> Result<SchemaMigrationReceipt, CanonicalError> {
        let request = &self.request;
        let module = self.maintenance.require_original_module(request)?;
        if let Some(receipt) = self.maintenance.reconcile_receipt(request).await? {
            return Ok(receipt);
        }
        let mut bindings = request.bindings()?;
        bindings.extend(module.catalog_bindings());
        let body = final_application_body(request)?;
        execute_final_application(
            request,
            self.maintenance
                .execute_object(&body, "$pse_migration_receipt", bindings),
            self.maintenance.read_activation(request),
            #[cfg(test)]
            committed,
        )
        .await
    }
}

async fn execute_final_application(
    request: &SchemaMigrationRequest,
    write: impl Future<Output = Result<Object, CanonicalError>>,
    readback: impl Future<Output = Result<Option<Object>, CanonicalError>>,
    #[cfg(test)] committed: Option<&mut Option<SchemaMigrationReceipt>>,
) -> Result<SchemaMigrationReceipt, CanonicalError> {
    let write = write.await;
    #[cfg(test)]
    let write = match (write, committed) {
        // The transaction has returned its real receipt. Suppress only this
        // acknowledgement and let the original error path recover it.
        (Ok(receipt), Some(committed)) => {
            *committed = Some(verified_activation(receipt, request)?);
            Err(CanonicalError::Timeout)
        }
        (write, _) => write,
    };
    let receipt = match write {
        Ok(receipt) => receipt,
        Err(error) => match readback.await {
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
        },
    };
    verified_activation(receipt, request)
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
fn final_application_body(request: &SchemaMigrationRequest) -> Result<String, CanonicalError> {
    request.validate()?;
    let declarations = overwrite_declarations(wire::SCHEMA);
    let validation_body = super::study_module::capability_validation_body()?;
    let validation_predicate = super::study_module::capability_validation_predicate();
    Ok(format!(
        "{APPLY_PREDECESSOR}\n{declarations}\n{validation_body}\nIF !({validation_predicate}) {{ THROW 'PSE_REFUSAL:schema_migration_original_differs'; }};\n{APPLY_SUCCESSOR}",
    ))
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
const APPLY_PREDECESSOR: &str = r#"
LET $pse_owner=SELECT * FROM ONLY canonical_maintenance_operations:current FOR UPDATE;
LET $pse_marker=SELECT * FROM ONLY canonical_interpretations:current FOR UPDATE;
IF $pse_owner=NONE OR $pse_owner.kind!='schema' OR $pse_owner.operation!=$migration_operation OR $pse_owner.generation!=$migration_generation OR $pse_owner.after!=$migration_request OR $pse_owner.phase!='schema-applying' OR $pse_owner.complete!=false OR $pse_marker=NONE OR $pse_marker.interpretation!=$migration_predecessor.interpretation OR $pse_marker.schema_digest!=$migration_predecessor.schema_digest OR $pse_marker.issuing_store!=$migration_predecessor.issuing_store OR $pse_marker.study_module_sha256!=$migration_predecessor.study_module_sha256 OR $pse_marker.payload_incarnation!=$migration_predecessor.payload_incarnation OR $pse_marker.schema_epoch!=$migration_predecessor.schema_epoch OR $pse_marker.schema_version!=$migration_predecessor.schema_version OR $pse_marker.admission_open!=false { THROW 'PSE_REFUSAL:activation_predecessor_changed'; };
IF (SELECT * FROM ONLY type::record('canonical_maintenance_activations',$migration_operation))!=NONE { THROW 'PSE_REFUSAL:activation_operation_already_exists_settle_exact_receipt'; };
"#;
const APPLY_SUCCESSOR: &str = r#"
UPDATE ONLY canonical_interpretations:current SET schema_digest=$migration_target,schema_version=$migration_version,schema_epoch=$migration_generation,study_module_sha256=$migration_module_sha256,admission_open=true RETURN NONE;
UPDATE ONLY canonical_maintenance_operations:current SET phase='schema-active',count=1dec,complete=true RETURN NONE;
LET $pse_migration_receipt=CREATE ONLY type::record('canonical_maintenance_activations',$migration_operation) SET key=$migration_operation,input_sha256=$migration_request,predecessor=$migration_predecessor_bytes,interpretation=$migration_predecessor.interpretation,schema_digest=$migration_target,study_module_sha256=$migration_module_sha256,issuing_store=$migration_predecessor.issuing_store;
"#;

/// Fixed-row actual-transaction controls, available only to explicitly selected fixtures.
#[cfg(all(test, feature = "native-service"))]
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
#[cfg(all(test, feature = "native-service"))]
pub use schema_fence_control::SchemaFenceWrite;

#[cfg(test)]
mod settlement_scope_unit {
    use super::*;
    use std::{
        pin::Pin,
        sync::Mutex,
        task::{Context, Poll},
    };

    fn fence() -> AttemptFence {
        AttemptFence::from_row(crate::canonical_execution::CanonicalAttempt {
            key: "scope-attempt".into(),
            run: "scope-run".into(),
            generation: 7,
            claim_operation: "scope-claim".into(),
            request: vec![].into(),
            worker: "scope-worker".into(),
            expires_at: 0,
            phase: pse_model::generated::enums::CanonicalAttemptPhase::Ingesting,
            close_generation: None,
            outcome: None,
            closed_manifest: None,
            completion: None,
            seed_manifest: None,
            finalization_study: None,
        })
        .unwrap()
    }

    fn expected() -> Value {
        let mut value = Object::new();
        value.insert("run", "scope-run");
        value.insert("attempt", "scope-attempt");
        value.insert("generation", codec::encode_uint(7).unwrap());
        Value::Object(value)
    }

    struct Probe {
        observations: Arc<Mutex<Vec<Value>>>,
        ready: bool,
    }
    impl Future for Probe {
        type Output = Result<(), CanonicalError>;
        fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
            self.observations.lock().unwrap().push(settlement_value());
            if self.ready {
                Poll::Ready(Ok(()))
            } else {
                Poll::Pending
            }
        }
    }
    impl Drop for Probe {
        fn drop(&mut self) {
            self.observations.lock().unwrap().push(settlement_value());
        }
    }

    #[test]
    fn settlement_scope_keeps_large_child_out_of_composed_caller_state() {
        let fence = fence();
        let payload = [0_u8; 64 * 1024];
        let child = async move {
            std::future::pending::<()>().await;
            Ok(std::hint::black_box(payload))
        };
        let child_bytes = size_of_val(&child);
        let scoped = settlement_scope(&fence, child);
        assert!(size_of_val(&scoped) < child_bytes / 16);
    }

    #[test]
    fn settlement_scope_ready_and_cancel_drop_keep_original_authority_and_restore_parent() {
        for ready in [false, true] {
            let fence = fence();
            let observations = Arc::new(Mutex::new(Vec::new()));
            let mut scoped = Box::pin(settlement_scope(
                &fence,
                Probe {
                    observations: observations.clone(),
                    ready,
                },
            ));
            let parent = Value::String("enclosing-authority".into());
            SETTLEMENT.sync_scope(parent.clone(), || {
                let mut cx = Context::from_waker(std::task::Waker::noop());
                assert_eq!(scoped.as_mut().poll(&mut cx).is_ready(), ready);
                assert_eq!(settlement_value(), parent);
            });
            drop(scoped);
            assert_eq!(*observations.lock().unwrap(), vec![expected(), expected()]);
            assert_eq!(settlement_value(), Value::None);
        }
    }

    #[test]
    fn settlement_scope_unpolled_drop_keeps_ambient_authority() {
        let fence = fence();
        let observations = Arc::new(Mutex::new(Vec::new()));
        let scoped = settlement_scope(
            &fence,
            Probe {
                observations: observations.clone(),
                ready: false,
            },
        );
        assert!(observations.lock().unwrap().is_empty());
        drop(scoped);
        assert_eq!(*observations.lock().unwrap(), vec![Value::None]);
    }
}

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
            request.predecessor.study_module_sha256.clone(),
        );
        receipt.insert("issuing_store", request.predecessor.issuing_store.clone());
        receipt
    }
    #[tokio::test]
    async fn schema_final_apply_settles_exact_readback_without_replaying_write() {
        let request = request(&marker());
        let original = activation(&request);
        let accepted = execute_final_application(
            &request,
            std::future::ready(Ok(original.clone())),
            async { panic!("a successful write needs no readback") },
            None,
        )
        .await
        .unwrap();
        assert_eq!(accepted.request(), &request);

        let mut foreign = original.clone();
        foreign.insert("key", uuid::Uuid::new_v4().to_string());
        let cases = [
            (Ok(Some(original)), "matching"),
            (Ok(None), "absent"),
            (Err(CanonicalError::IncompleteResponse), "unavailable"),
            (Ok(Some(foreign)), "foreign"),
        ];
        for (readback, case) in cases {
            let write_calls = std::cell::Cell::new(0);
            let readback_calls = std::cell::Cell::new(0);
            let result = execute_final_application(
                &request,
                async {
                    write_calls.set(write_calls.get() + 1);
                    Err(CanonicalError::Timeout)
                },
                async {
                    assert_eq!(write_calls.get(), 1);
                    readback_calls.set(readback_calls.get() + 1);
                    readback
                },
                None,
            )
            .await;
            assert_eq!((write_calls.get(), readback_calls.get()), (1, 1), "{case}");
            match (case, result) {
                ("matching", Ok(receipt)) => assert_eq!(receipt.request(), &request),
                ("absent", Err(CanonicalError::Timeout)) => {}
                ("foreign", Err(CanonicalError::OperationReused)) => {}
                (
                    "unavailable",
                    Err(CanonicalError::SettlementUnresolved {
                        owner,
                        operation,
                        request: recorded,
                        source,
                        readback,
                    }),
                ) => {
                    assert_eq!(owner, "schema-migration");
                    assert_eq!(operation, request.operation());
                    assert_eq!(recorded, bytes(&request).unwrap());
                    assert!(matches!(*source, CanonicalError::Timeout));
                    assert!(matches!(
                        readback.as_deref(),
                        Some(CanonicalError::IncompleteResponse)
                    ));
                }
                (_, outcome) => panic!("unexpected final apply outcome for {case}: {outcome:?}"),
            }
        }
    }
    #[test]
    fn schema_migration_receipt_binds_original_issuer_payload_and_operation() {
        let original = request(&marker());
        let recorded = bytes(&original).unwrap();
        let declarations = final_application_body(&original).unwrap();
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
        changed.successor_epoch = changed.predecessor.schema_epoch.clone();
        assert!(changed.validate().is_err());
    }
    #[test]
    fn current_catalog_admission_refuses_foreign_markers_and_migration_requests() {
        let mut unversioned = marker();
        unversioned.remove("schema_version");
        unversioned.remove("schema_epoch");
        assert!(SchemaPredecessor::decode(&unversioned).is_err());
        assert!(SchemaFence::from_marker(&unversioned).is_err());

        let mut foreign = marker();
        foreign.insert("schema_version", codec::encode_uint(VERSION - 1).unwrap());
        let observed = SchemaPredecessor::decode(&foreign).unwrap();
        assert!(
            observed
                .to_current(uuid::Uuid::new_v4(), "bucket".into(), "backend".into())
                .is_err()
        );
        assert!(SchemaFence::from_marker(&foreign).is_err());
        let mut original = request(&marker());
        original.target_version = VERSION - 1;
        assert!(original.validate().is_err());
    }

    #[test]
    #[cfg(feature = "canonical-maintenance")]
    fn schema_migration_transaction_parses_with_pinned_server_grammar() {
        let body = final_application_body(&request(&marker())).unwrap();
        let script = MigrationScript::new(&body, "$pse_migration_receipt", Object::new());
        if let Err(error) = surrealdb_expr::parser::parse(&script.sql) {
            panic!("current declaration and activation: {error}");
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
