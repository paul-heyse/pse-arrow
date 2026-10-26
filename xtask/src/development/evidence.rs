// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Disposable evidence projections on the selected service. Never scientific authority.
use pse_operations::{canonical::{self, CanonicalError}, transport::{self, lifetime::ConnectionAccess, original_deadline, within_clock}};
use crate::generated::store::evidence as wire;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, path::Path, sync::Arc};
use surrealdb::{
    Surreal,
    engine::remote::grpc::{Client, Grpc},
    opt::{Config, GrpcConfig, RequestContext},
    types::{Object, Value},
};
/// Tooling lifecycle failures, retaining original native uncertainty.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[diagnostic(code(runtime::infrastructure))]
pub enum EvidenceError {
    /// Invalid local configuration or typed snapshot.
    #[error("evidence configuration: {0}")]
    Configuration(String),
    /// Existing index is absent or incompatible; opening never installs it.
    #[error("evidence index is unavailable or incompatible: {0}")]
    Unavailable(String),
    /// Cursor was issued by another authority, query or active generation.
    #[error("evidence cursor context differs or its generation is stale")]
    Cursor,
    /// Original immutable source identity cannot mean different projection bytes.
    #[error("evidence snapshot identity was reused with different content")]
    Reused,
    /// Mutation may have committed. Repeat only the same original snapshot.
    #[error("evidence generation {generation} did not settle: {source}")]
    Unsettled {
        /// Exact content-addressed generation.
        generation: String,
        /// Native failure; absence is not inferred.
        #[source]
        source: Box<CanonicalError>,
    },
    /// A submitted lifecycle request failed, retaining its exact intended authority.
    /// The native source determines effect knowledge; this does not imply rollback or replay.
    #[error(
        "evidence {operation} for {namespace}/{database} (issuing authority {issuing_store}) failed: {source}"
    )]
    LifecycleFailure {
        /// Original explicit lifecycle operation.
        operation: &'static str,
        /// Selected tooling namespace.
        namespace: String,
        /// Selected tooling database.
        database: String,
        /// Requested initialization authority or observed disposal authority.
        issuing_store: String,
        /// Original native failure, including definite refusals and uncertain completion.
        #[source]
        source: Box<CanonicalError>,
    },
    /// Underlying selected service transport or lifecycle failure.
    #[error(transparent)]
    Transport(#[from] CanonicalError),
}
impl From<pse_operations::canonical_codec::CodecError> for EvidenceError {
    fn from(e: pse_operations::canonical_codec::CodecError) -> Self {
        Self::Transport(e.into())
    }
}
impl From<surrealdb::Error> for EvidenceError {
    fn from(e: surrealdb::Error) -> Self {
        Self::Transport(e.into())
    }
}
impl pse_diagnostics::TypedDiagnostic for EvidenceError {
    fn diagnostic_code(&self) -> Option<pse_diagnostics::DiagnosticCode> {
        Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure)
    }
}
fn config(message: impl Into<String>) -> EvidenceError {
    EvidenceError::Configuration(message.into())
}
fn rpc_expiry(deadline: tokio::time::Instant) -> Result<i64, EvidenceError> {
    let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
    let expiry = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .saturating_add(remaining)
        .as_micros();
    i64::try_from(expiry).map_err(|_| config("deadline overflow"))
}
/// Private tooling deployment. Debug never prints credentials.
#[derive(Clone)]
pub struct ToolingOptions {
    /// Selected loopback service endpoint.
    pub endpoint: String,
    /// Tooling namespace, distinct from scientific namespace.
    pub namespace: String,
    /// Independently disposable database.
    pub database: String,
    /// Host service issuing identity.
    pub service_instance_id: String,
    credentials: Credentials,
}
impl std::fmt::Debug for ToolingOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolingOptions")
            .field("namespace", &self.namespace)
            .field("database", &self.database)
            .finish_non_exhaustive()
    }
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Credentials {
    reader_username: String,
    reader_password: String,
    importer_username: String,
    importer_password: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Deployment {
    version: u32,
    service_instance_id: String,
    endpoint: String,
    namespace: String,
    database: String,
    credentials_file: String,
}
fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|v| v.is_ascii_alphanumeric() || v == b'_')
}
fn private_read(path: &Path, max: u64) -> Result<Vec<u8>, EvidenceError> {
    let metadata = std::fs::symlink_metadata(path).map_err(|e| config(e.to_string()))?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > max {
        return Err(config("expected bounded real private file"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let uid = std::fs::metadata("/proc/self")
            .map_err(|e| config(e.to_string()))?
            .uid();
        if metadata.mode() & 0o077 != 0 || metadata.uid() != uid {
            return Err(config(
                "tooling configuration must be private and owned by current user",
            ));
        }
    }
    std::fs::read(path).map_err(|e| config(e.to_string()))
}
impl ToolingOptions {
    /// Read the tooling contract only. Scientific schema/modules/receivers are irrelevant.
    pub fn from_state(state: &Path) -> Result<Self, EvidenceError> {
        let directory = state.join(".tooling/evidence");
        for path in [state.to_owned(), state.join(".tooling"), directory.clone()] {
            let meta = std::fs::symlink_metadata(&path).map_err(|e| config(e.to_string()))?;
            if !meta.is_dir() || meta.file_type().is_symlink() {
                return Err(config("tooling state must be real directories"));
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                let uid = std::fs::metadata("/proc/self")
                    .map_err(|e| config(e.to_string()))?
                    .uid();
                if meta.mode() & 0o077 != 0 || meta.uid() != uid {
                    return Err(config(
                        "tooling directories must be private and owned by current user",
                    ));
                }
            }
        }
        let d: Deployment =
            serde_json::from_slice(&private_read(&directory.join("config.json"), 64 * 1024)?)
                .map_err(|e| config(e.to_string()))?;
        if d.version != 1
            || d.service_instance_id.is_empty()
            || !identifier(&d.namespace)
            || !identifier(&d.database)
            || d.credentials_file != "credentials.json"
        {
            return Err(config(
                "tooling deployment version, identity or identifier differs",
            ));
        }
        let c: Credentials = serde_json::from_slice(&private_read(
            &directory.join(&d.credentials_file),
            64 * 1024,
        )?)
        .map_err(|e| config(e.to_string()))?;
        if !identifier(&c.reader_username)
            || !identifier(&c.importer_username)
            || c.reader_username == c.importer_username
            || c.reader_password.is_empty()
            || c.importer_password.is_empty()
        {
            return Err(config(
                "distinct nonempty reader/importer credentials required",
            ));
        }
        let options = Self {
            endpoint: d.endpoint,
            namespace: d.namespace,
            database: d.database,
            service_instance_id: d.service_instance_id,
            credentials: c,
        };
        options.validate()?;
        Ok(options)
    }
    fn validate(&self) -> Result<(), EvidenceError> {
        if !(self.namespace == "pse_tooling" || self.namespace.starts_with("pse_tooling_"))
            || !identifier(&self.namespace)
            || !identifier(&self.database)
            || self.service_instance_id.is_empty()
            || self.service_instance_id.len() > 4096
            || !identifier(&self.credentials.reader_username)
            || !identifier(&self.credentials.importer_username)
            || self.credentials.reader_username == self.credentials.importer_username
            || self.credentials.reader_password.is_empty()
            || self.credentials.importer_password.is_empty()
        {
            return Err(config(
                "dedicated tooling namespace and distinct scoped credentials required",
            ));
        }
        self.address()?;
        Ok(())
    }
    fn address(&self) -> Result<std::net::SocketAddr, EvidenceError> {
        let url = url::Url::parse(&self.endpoint).map_err(|e| config(e.to_string()))?;
        if url.scheme() != "grpc"
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
            || !matches!(url.path(), "" | "/")
        {
            return Err(config("unadorned loopback grpc endpoint required"));
        }
        let host = url
            .host_str()
            .ok_or_else(|| config("missing endpoint host"))?;
        let ip: std::net::IpAddr = host
            .parse()
            .map_err(|_| config("literal loopback endpoint required"))?;
        if !ip.is_loopback() {
            return Err(config("loopback endpoint required"));
        }
        Ok(std::net::SocketAddr::new(
            ip,
            url.port().ok_or_else(|| config("endpoint port required"))?,
        ))
    }
}
/// One compact authentic owner fact, without raw bytes or a lifetime grant.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceFact {
    /// Original stable member identity.
    pub member: String,
    /// Owner-declared category.
    pub kind: String,
    /// Original outcome, including unreported/command failures.
    pub status: String,
    /// Declared owning crate, if available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crate_name: Option<String>,
    /// Exact Rust binary/test identity or Python node ID, if available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_id: Option<String>,
    /// Descriptive owner reference, never an automatic path or grant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail_ref: Option<String>,
    /// Compact owner-declared binary id; absent means unknown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binary_id: Option<String>,
    /// Compact owner-declared context id; absent means unknown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_id: Option<String>,
    /// Compact owner-declared context json; absent means unknown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_json: Option<String>,
    /// Compact owner-declared link relation; absent means unknown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link_relation: Option<String>,
    /// Compact owner-declared link target; absent means unknown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link_target: Option<String>,
    /// Compact owner-declared population id; absent means unknown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub population_id: Option<String>,
    /// Compact owner-declared sample id; absent means unknown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample_id: Option<String>,
    /// Compact owner-declared value ns; absent means unknown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_ns: Option<f64>,
}
impl EvidenceFact {
    pub(crate) fn to_row(
        &self,
        generation: &str,
    ) -> Result<crate::generated::model::runtime::evidence_facts::Row, EvidenceError> {
        Ok(crate::generated::model::runtime::evidence_facts::Row {
            key: hex(serde_json::to_vec(&(generation, &self.member))
                .map_err(|e| config(e.to_string()))?),
            generation: generation.to_owned(),
            member: self.member.clone(),
            kind: self.kind.clone(),
            status: self.status.clone(),
            crate_name: self.crate_name.clone(),
            test_id: self.test_id.clone(),
            detail_ref: self.detail_ref.clone(),
            binary_id: self.binary_id.clone(),
            context_id: self.context_id.clone(),
            context_json: self.context_json.clone(),
            link_relation: self.link_relation.clone(),
            link_target: self.link_target.clone(),
            population_id: self.population_id.clone(),
            sample_id: self.sample_id.clone(),
            value_ns: self.value_ns,
        })
    }
}
/// Frozen owner adapter output. Partial source coverage is not a complete test population.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceSnapshot {
    /// Boundary codec version.
    pub version: u32,
    /// Original authority identity.
    pub authority: String,
    /// Original owner codec.
    pub codec: String,
    /// Original invocation/snapshot source identity.
    pub source_id: String,
    /// SHA256 of original authenticated source snapshot bytes.
    pub snapshot_digest: String,
    /// Original population completeness: complete or partial.
    pub coverage: String,
    /// Exact selected member identities, including unreported members.
    pub expected_members: Vec<String>,
    /// Compact facts emitted by original readers.
    pub facts: Vec<EvidenceFact>,
}
fn hex(bytes: impl AsRef<[u8]>) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
impl EvidenceSnapshot {
    /// Validate exact declared membership, preserving UInt64 and original outcomes.
    pub fn validate(&self) -> Result<(), EvidenceError> {
        if self.version != 2
            || !matches!(self.coverage.as_str(), "complete" | "partial")
            || self.authority.is_empty()
            || self.codec.is_empty()
            || self.source_id.is_empty()
            || [&self.authority, &self.codec, &self.source_id]
                .iter()
                .any(|v| v.len() > 4096)
            || self.snapshot_digest.len() != 64
            || !self
                .snapshot_digest
                .bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
        {
            return Err(config(
                "invalid evidence identity, version, digest or coverage",
            ));
        }
        if !matches!(
            self.codec.as_str(),
            "exact-resource-files-invocation-v2"
                | "exact-resource-files-invocation-v3"
                | "assessment-v5"
                | "assessment-v6"
                | "active-diagnostic-action-v1"
                | "process-cost-v5"
                | "host-admission-ledger-v1"
                | "case-smoke-v1"
                | "storage-health-v1"
        ) {
            return Err(config("unsupported original evidence source codec"));
        }
        let expected = self.expected_members.iter().collect::<BTreeSet<_>>();
        let actual = self
            .facts
            .iter()
            .map(|f| &f.member)
            .collect::<BTreeSet<_>>();
        if expected.len() != self.expected_members.len()
            || actual.len() != self.facts.len()
            || expected != actual
        {
            return Err(config(
                "duplicate or incomplete selected snapshot membership",
            ));
        }
        let mut contexts = std::collections::BTreeMap::new();
        let mut samples = BTreeSet::new();
        for f in &self.facts {
            if let Some(value) = &f.context_json {
                if f.kind != "context" {
                    return Err(config("context declaration has wrong category"));
                }
                let value: std::collections::BTreeMap<String, serde_json::Value> =
                    serde_json::from_str(value).map_err(|e| config(e.to_string()))?;
                let fields = [
                    "workload",
                    "input",
                    "revision",
                    "build",
                    "profile",
                    "features",
                    "provider",
                    "isa",
                    "capture",
                    "host",
                    "environment",
                    "source",
                    "mode",
                    "process",
                ];
                if value.keys().any(|key| !fields.contains(&key.as_str()))
                    || value.len() != fields.len()
                {
                    return Err(config("undeclared or missing compact context field"));
                }
                let id = f
                    .context_id
                    .as_ref()
                    .ok_or_else(|| config("context identity required"))?;
                if contexts.insert(id.clone(), value).is_some() {
                    return Err(config("duplicate context declaration"));
                }
            }
            if f.kind == "declared_link" && (f.link_relation.is_none() || f.link_target.is_none()) {
                return Err(config("declared link lacks exact relation or target"));
            }
            if f.kind == "criterion_sample" {
                if f.sample_id.is_none() || f.population_id.is_none() || f.value_ns.is_none() {
                    return Err(config(
                        "sample lacks actual population, membership or value",
                    ));
                }
                if !samples.insert((&f.population_id, &f.sample_id)) {
                    return Err(config("duplicate actual sample membership"));
                }
            }

            if f.member.is_empty() || f.kind.is_empty() || f.status.is_empty() {
                return Err(config("fact identity/kind/status required"));
            }
            if f.value_ns.is_some_and(|v| !v.is_finite() || v < 0.0) {
                return Err(config("measurement must be finite nonnegative nanoseconds"));
            }
            for value in [
                Some(&f.member),
                Some(&f.kind),
                Some(&f.status),
                f.crate_name.as_ref(),
                f.test_id.as_ref(),
                f.detail_ref.as_ref(),
                f.binary_id.as_ref(),
                f.context_id.as_ref(),
                f.context_json.as_ref(),
                f.link_relation.as_ref(),
                f.link_target.as_ref(),
                f.population_id.as_ref(),
                f.sample_id.as_ref(),
            ]
            .into_iter()
            .flatten()
            {
                if value.len() > 4096 {
                    return Err(config("fact text exceeds compact metadata envelope"));
                }
            }
        }
        for fact in &self.facts {
            if fact
                .context_id
                .as_ref()
                .is_some_and(|id| !contexts.contains_key(id))
            {
                return Err(config("fact refers to undeclared context"));
            }
        }
        let bytes = serde_json::to_vec(self).map_err(|e| config(e.to_string()))?;
        if bytes.len() > canonical::PAYLOAD_BYTES {
            return Err(config(
                "snapshot exceeds bounded import envelope; select a smaller explicit source cut",
            ));
        }
        Ok(())
    }
    pub(crate) fn identity(&self) -> Result<(String, String), EvidenceError> {
        self.validate()?;
        let mut canonical = self.clone();
        canonical.expected_members.sort();
        canonical.facts.sort_by(|a, b| a.member.cmp(&b.member));
        let content = hex(serde_json::to_vec(&canonical).map_err(|e| config(e.to_string()))?);
        let key = hex(serde_json::to_vec(&(
            &self.authority,
            &self.codec,
            &self.source_id,
            &self.snapshot_digest,
        ))
        .map_err(|e| config(e.to_string()))?);
        Ok((key, content))
    }
}
/// Exact issuing and selected source cut; metadata does not protect original files.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EvidenceCursor {
    /// Index issuing identity.
    pub issuing_store: String,
    /// Exact active generation.
    pub generation: String,
    /// Query filter identity.
    pub query_digest: String,
    /// Last delivered original member.
    pub after: String,
}
/// Bounded native indexed query.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceQuery {
    /// Explicit prior activated cut; absent follows the current active pointer.
    pub generation: Option<String>,
    /// Optional exact category.
    pub kind: Option<String>,
    /// Optional exact status.
    pub status: Option<String>,
    /// Delivery page extent, 1..=64; default 64.
    pub limit: Option<usize>,
    /// Continuation tied to this query and issuing cut.
    pub cursor: Option<EvidenceCursor>,
}
/// One complete delivered page and its honest source-cut description.
#[derive(Clone, Debug, Serialize)]
pub struct EvidencePage {
    /// Issuing tooling authority.
    pub issuing_store: String,
    /// Exact selected activated import.
    pub generation: String,
    /// Original source authority.
    pub authority: String,
    /// Original source identity.
    pub source_id: String,
    /// Original source digest.
    pub snapshot_digest: String,
    /// Original source population coverage.
    pub coverage: String,
    /// Index freshness is relative only to imported cut, never live owners.
    pub freshness: &'static str,
    /// Compact selected facts.
    pub facts: Vec<EvidenceFact>,
    /// More rows under the same cut.
    pub next: Option<EvidenceCursor>,
}
/// Complete immutable import acknowledgement.
#[derive(Clone, Debug, Serialize)]
pub struct EvidenceReceipt {
    /// Original content-addressed generation, reused on replay.
    pub generation: String,
    /// True after exact source membership was reconciled.
    pub activated: bool,
}
/// Independently scoped role. Importer is not a canonical writer.
#[derive(Clone, Copy, Debug)]
pub enum EvidenceAccess {
    /// Database-scoped read-only role.
    Reader,
    /// Database-scoped editor; cannot manage users/access or reach scientific namespace.
    Importer,
}
/// Carry one original caller allowance through connection, retries, effects and response processing.
/// An enclosing operation can only shorten this deadline.
pub async fn within_allowance<T, F>(
    deadline: std::time::Instant,
    future: F,
) -> Result<T, EvidenceError>
where
    F: Future<Output = Result<T, EvidenceError>>,
{
    let deadline =
        tokio::time::Instant::from_std(deadline).min(original_deadline(canonical::REQUEST_TIMEOUT));
    tokio::time::timeout_at(
        deadline,
        transport::operation_scope(deadline.into_std(), future),
    )
    .await
    .map_err(|_| EvidenceError::Transport(CanonicalError::Timeout))?
}
/// A tooling connection with shared admission/transport semantics, no scientific Runtime.
#[derive(Clone)]
pub struct EvidenceStore {
    db: Arc<Surreal<Client>>,
    access: ConnectionAccess,
    options: ToolingOptions,
    pub(crate) issuing_store: String,
    pub(crate) role: EvidenceAccess,
    #[cfg(test)]
    pub(crate) ack_fault: Arc<std::sync::atomic::AtomicU8>,
}
impl std::fmt::Debug for EvidenceStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EvidenceStore")
            .field("options", &self.options)
            .field("issuing_store", &self.issuing_store)
            .finish_non_exhaustive()
    }
}
impl EvidenceStore {
    async fn client(options: &ToolingOptions) -> Result<Surreal<Client>, EvidenceError> {
        options.validate()?;
        let cfg = Config::new()
            .bounded_requests()
            .grpc(GrpcConfig::new().max_message_size(canonical::MESSAGE_BYTES))?;
        let deadline = original_deadline(canonical::REQUEST_TIMEOUT);
        Ok(within_clock(deadline, async {
            Surreal::new::<Grpc>((
                options
                    .address()
                    .map_err(|e| CanonicalError::Configuration(e.to_string()))?,
                cfg,
            ))
            .await
            .map_err(Into::into)
        })
        .await?)
    }
    /// Open and attest an existing index. No schema, scientific module or payload setup occurs.
    pub async fn open(
        options: &ToolingOptions,
        role: EvidenceAccess,
    ) -> Result<Self, EvidenceError> {
        within_allowance(
            original_deadline(canonical::REQUEST_TIMEOUT).into_std(),
            Self::open_selected(options, role, true),
        )
        .await
    }
    async fn open_selected(
        options: &ToolingOptions,
        role: EvidenceAccess,
        compatible: bool,
    ) -> Result<Self, EvidenceError> {
        let db = Self::client(options).await?;
        let deadline = original_deadline(canonical::REQUEST_TIMEOUT);
        let (username, password) = match role {
            EvidenceAccess::Reader => (
                &options.credentials.reader_username,
                &options.credentials.reader_password,
            ),
            EvidenceAccess::Importer => (
                &options.credentials.importer_username,
                &options.credentials.importer_password,
            ),
        };
        let connect = within_clock(deadline, async {
            db.signin(surrealdb::opt::auth::Database {
                namespace: options.namespace.clone(),
                database: options.database.clone(),
                username: username.clone(),
                password: password.clone(),
            })
            .request_context(RequestContext::control(deadline.into_std()))
            .await?;
            db.use_ns(&options.namespace)
                .use_db(&options.database)
                .request_context(RequestContext::control(deadline.into_std()))
                .await?;
            Ok(())
        })
        .await;
        if let Err(e) = connect {
            let _ = db.disconnect().await;
            return Err(e.into());
        }
        let mut store = Self {
            db: Arc::new(db),
            access: ConnectionAccess::default(),
            options: options.clone(),
            issuing_store: String::new(),
            role,
            #[cfg(test)]
            ack_fault: Arc::default(),
        };
        let marker = store.marker_checked(compatible).await;
        match marker {
            Ok(marker) => {
                store.issuing_store = marker.issuing_store;
                Ok(store)
            }
            Err(e) => {
                let _ = store.disconnect().await;
                Err(e)
            }
        }
    }
    pub(crate) fn query<'a>(
        &'a self,
        sql: impl Into<std::borrow::Cow<'a, str>>,
    ) -> Result<surrealdb::method::Query<'a, Client>, EvidenceError> {
        self.query_with_limits(sql, canonical::MetadataResponse::Page.limits())
    }
    pub(crate) fn query_with_limits<'a>(
        &'a self,
        sql: impl Into<std::borrow::Cow<'a, str>>,
        limits: surrealdb::opt::ResponseLimits,
    ) -> Result<surrealdb::method::Query<'a, Client>, EvidenceError> {
        let activity = self.access.admit()?;
        let deadline = original_deadline(canonical::REQUEST_TIMEOUT);
        let expiry = rpc_expiry(deadline)?;
        Ok(self
            .db
            .query(sql)
            .request_context(
                RequestContext::new(deadline.into_std())
                    .with_lifetime_owner(Arc::new(activity))
                    .with_response_limits(limits),
            )
            .bind(("expiry", expiry)))
    }
    pub(crate) async fn marker(
        &self,
    ) -> Result<crate::generated::model::runtime::evidence_interpretations::Row, EvidenceError> {
        self.marker_checked(true).await
    }
    async fn marker_checked(
        &self,
        compatible: bool,
    ) -> Result<crate::generated::model::runtime::evidence_interpretations::Row, EvidenceError> {
        let mut response = transport::bounded_query(
            self.query("SELECT * FROM ONLY evidence_interpretations:current;")?,
        )
        .await?;
        let marker: Option<Object> = response.take(0)?;
        let marker = marker
            .ok_or_else(|| EvidenceError::Unavailable("initialization marker absent".into()))?;
        self.decode_marker(marker, compatible)
    }
    pub(crate) fn decode_marker(
        &self,
        object: Object,
        compatible: bool,
    ) -> Result<crate::generated::model::runtime::evidence_interpretations::Row, EvidenceError> {
        let marker = wire::decode_evidence_interpretations(object)?;
        if marker.interpretation != wire::INTERPRETATION
            || (compatible && marker.schema_digest != wire::SCHEMA_DIGEST)
            || marker.service_instance_id != self.options.service_instance_id
            || marker.issuing_store.is_empty()
            || (!self.issuing_store.is_empty() && marker.issuing_store != self.issuing_store)
        {
            return Err(EvidenceError::Unavailable(
                "interpretation/schema/service issuing identity differs; explicit rebuild required"
                    .into(),
            ));
        }
        Ok(marker)
    }
    /// Validate current index identity and return its immutable issuing/source-cut metadata.
    pub async fn status(&self) -> Result<serde_json::Value, EvidenceError> {
        let marker = self.marker().await?;
        Ok(
            serde_json::json!({"interpretation":marker.interpretation,"schema_digest":marker.schema_digest,"issuing_store":marker.issuing_store,"service_instance_id":marker.service_instance_id,"namespace":self.options.namespace,"database":self.options.database,"active_generation":marker.active_generation,"freshness":"imported-cut-only","payload_lifetime_granted":false}),
        )
    }
    /// Close logical admission without granting cancellation of already admitted transport.
    pub fn close(&self) {
        self.access.close_caller();
    }
    /// Drain the actual SDK connection, reusing existing physical lifetime semantics.
    pub async fn disconnect(&self) -> Result<(), EvidenceError> {
        let db = self.db.clone();
        self.access.disconnect(move || async move { db.disconnect().await.map_err(Into::into) })
            .await?;
        Ok(())
    }
    /// Stage and reconcile one exact authentic snapshot, then activate it atomically.
    /// Repeating the same snapshot settles its original activation without duplicate rows
    /// or replacing a newer current generation.
    pub async fn import(
        &self,
        snapshot: &EvidenceSnapshot,
    ) -> Result<EvidenceReceipt, EvidenceError> {
        let deadline = original_deadline(canonical::REQUEST_TIMEOUT).into_std();
        within_allowance(deadline, self.import_snapshot(snapshot)).await
    }
    async fn import_snapshot(
        &self,
        snapshot: &EvidenceSnapshot,
    ) -> Result<EvidenceReceipt, EvidenceError> {
        if !matches!(self.role, EvidenceAccess::Importer) {
            return Err(config("reader cannot import"));
        }
        let (generation, content_digest) = snapshot.identity()?;
        let activity = self.access.admit()?;
        activity.scope(async {
            let input=crate::generated::model::runtime::evidence_generations::Row{key:generation.clone(),authority:snapshot.authority.clone(),codec:snapshot.codec.clone(),source_id:snapshot.source_id.clone(),snapshot_digest:snapshot.snapshot_digest.clone(),content_digest:content_digest.clone(),coverage:snapshot.coverage.parse().map_err(|_|config("invalid coverage vocabulary"))?,expected_members:snapshot.facts.len().try_into().map_err(|_|config("membership count overflow"))?,state:crate::generated::model::enums::EvidenceGenerationState::Staged};
            let input=wire::encode_evidence_generations(&input)?;
            let mut facts=Vec::with_capacity(snapshot.facts.len());
            for fact in &snapshot.facts {
                facts.push(Value::Object(wire::encode_evidence_facts(&fact.to_row(&generation)?)?));
            }
            let stage=r#"BEGIN;
IF time::micros()>=$expiry { THROW 'evidence original deadline expired'; };
LET $marker=SELECT * FROM ONLY evidence_interpretations:current;
IF $marker.issuing_store!=$issuer { THROW 'evidence authority changed'; };
LET $old=SELECT * FROM ONLY type::record('evidence_generations',$generation);
IF $old!=NONE AND $old.content_digest!=$input.content_digest { THROW 'evidence snapshot identity reused'; };
IF $old=NONE {
 CREATE type::record('evidence_generations',$generation) CONTENT $input;
 FOR $fact IN $facts { CREATE type::record('evidence_facts',$fact.key) CONTENT $fact; };
};
IF time::micros()>=$expiry { THROW 'evidence original deadline expired'; };
COMMIT;"#;
            let staged=transport::protected_query("evidence-stage",||self.query(stage).map(|q|q.bind(("issuer",self.issuing_store.clone())).bind(("generation",generation.clone())).bind(("input",input.clone())).bind(("facts",facts.clone()))).map_err(|e|CanonicalError::Configuration(e.to_string()))).await;
            #[cfg(test)]
            let staged=if self.ack_fault.compare_exchange(1,0,std::sync::atomic::Ordering::SeqCst,std::sync::atomic::Ordering::SeqCst).is_ok(){staged.and_then(|_|Err(CanonicalError::IncompleteResponse))}else{staged};
            if let Err(source)=staged {
                // A successful receipt read may prove this exact stage after lost acknowledgement.
                match self.generation(&generation).await {Ok(row) if row.content_digest==content_digest=>{},Ok(_)=>return Err(EvidenceError::Reused),Err(_)=>return Err(EvidenceError::Unsettled{generation:generation.clone(),source:Box::new(source)})}
            }
            let staged=self.generation(&generation).await?;if staged.content_digest!=content_digest{return Err(EvidenceError::Reused);}
            let activate=r#"BEGIN;
IF time::micros()>=$expiry { THROW 'evidence original deadline expired'; };
LET $marker=SELECT * FROM ONLY evidence_interpretations:current;
IF $marker.issuing_store!=$issuer { THROW 'evidence authority changed'; };
LET $generation_row=SELECT * FROM ONLY type::record('evidence_generations',$generation);
IF $generation_row=NONE OR $generation_row.content_digest!=$content { THROW 'evidence generation identity differs'; };
IF $generation_row.state!='active' {
    LET $members=SELECT count() AS n FROM evidence_facts WHERE generation=$generation GROUP ALL;
    IF $generation_row.expected_members!=0dec AND ($members[0].n=NONE OR <decimal>$members[0].n!=$generation_row.expected_members) { THROW 'evidence incomplete generation'; };
    UPDATE type::record('evidence_generations',$generation) SET state='active';
    UPDATE evidence_interpretations:current SET active_generation=$generation;
};
IF time::micros()>=$expiry { THROW 'evidence original deadline expired'; };
COMMIT;"#;
            let activated=transport::protected_query("evidence-activate",||self.query(activate).map(|q|q.bind(("issuer",self.issuing_store.clone())).bind(("generation",generation.clone())).bind(("content",content_digest.clone()))).map_err(|e|CanonicalError::Configuration(e.to_string()))).await;
            #[cfg(test)]
            let activated=if self.ack_fault.compare_exchange(2,0,std::sync::atomic::Ordering::SeqCst,std::sync::atomic::Ordering::SeqCst).is_ok(){activated.and_then(|_|Err(CanonicalError::IncompleteResponse))}else{activated};
            if let Err(source)=activated {let receipt=self.generation(&generation).await;if !matches!(receipt,Ok(ref row) if row.state==crate::generated::model::enums::EvidenceGenerationState::Active&&row.content_digest==content_digest){return Err(EvidenceError::Unsettled{generation:generation.clone(),source:Box::new(source)});}}
            Ok(EvidenceReceipt{generation:generation.clone(),activated:true})
        }).await
    }
    async fn generation(
        &self,
        generation: &str,
    ) -> Result<crate::generated::model::runtime::evidence_generations::Row, EvidenceError> {
        let mut r=transport::bounded_query(self.query("BEGIN; SELECT * FROM ONLY evidence_interpretations:current; SELECT * FROM ONLY type::record('evidence_generations',$generation); COMMIT;")?.bind(("generation",generation.to_owned()))).await?;
        let marker: Option<Object> = r.take(1)?;
        self.decode_marker(
            marker.ok_or_else(|| EvidenceError::Unavailable("marker absent".into()))?,
            true,
        )?;
        let row: Option<Object> = r.take(2)?;
        wire::decode_evidence_generations(row.ok_or_else(|| {
            EvidenceError::Unavailable("generation receipt absent; effect remains unproved".into())
        })?)
        .map_err(Into::into)
    }
    /// Query only an activated complete import. Cursor context is checked in one read snapshot.
    pub async fn select(&self, request: &EvidenceQuery) -> Result<EvidencePage, EvidenceError> {
        let limit = request.limit.unwrap_or(64);
        if !(1..=64).contains(&limit) {
            return Err(config("evidence query page must be 1..=64"));
        }
        if request
            .kind
            .iter()
            .chain(request.status.iter())
            .any(|v| v.len() > 4096)
            || request
                .cursor
                .as_ref()
                .is_some_and(|v| v.after.len() > 4096)
        {
            return Err(config(
                "query metadata exceeds compact response identity envelope",
            ));
        }
        if request.generation.as_ref().is_some_and(|v| {
            v.len() != 64
                || !v
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        }) {
            return Err(config(
                "explicit generation must be a lower-case SHA256 identity",
            ));
        }

        let query_digest =
            hex(
                serde_json::to_vec(&(&request.generation, &request.kind, &request.status, limit))
                    .map_err(|e| config(e.to_string()))?,
            );
        if let Some(cursor) = &request.cursor
            && (cursor.issuing_store != self.issuing_store || cursor.query_digest != query_digest)
        {
            return Err(EvidenceError::Cursor);
        }
        let cursor_generation = request.cursor.as_ref().map(|c| c.generation.clone());
        let after = request
            .cursor
            .as_ref()
            .map(|c| c.after.clone())
            .unwrap_or_default();
        let sql = r#"BEGIN;
LET $marker=SELECT * FROM ONLY evidence_interpretations:current;
LET $selected=IF $generation=NONE { $marker.active_generation } ELSE { $generation };
SELECT * FROM ONLY evidence_interpretations:current;
SELECT * FROM evidence_generations WHERE key=$selected LIMIT 1;
SELECT * FROM evidence_facts WHERE generation=$selected AND member>$after AND ($kind=NONE OR kind=$kind) AND ($status=NONE OR status=$status) ORDER BY member LIMIT $limit;
COMMIT;"#;
        let mut r = transport::bounded_query(
            self.query(sql)?
                .bind(("after", after))
                .bind(("generation", request.generation.clone()))
                .bind(("kind", request.kind.clone()))
                .bind(("status", request.status.clone()))
                .bind(("limit", (limit + 1) as i64)),
        )
        .await?;
        let marker: Option<Object> = r.take(3)?;
        let generation: Option<Object> = r.take::<Vec<Object>>(4)?.into_iter().next();
        let rows: Vec<Object> = r.take(5)?;
        let marker = self.decode_marker(
            marker.ok_or_else(|| EvidenceError::Unavailable("marker absent".into()))?,
            true,
        )?;
        if marker.issuing_store != self.issuing_store
            || marker.schema_digest != wire::SCHEMA_DIGEST
            || marker.service_instance_id != self.options.service_instance_id
        {
            return Err(EvidenceError::Cursor);
        }
        let generation = wire::decode_evidence_generations(
            generation.ok_or_else(|| EvidenceError::Unavailable("no active generation".into()))?,
        )?;
        if generation.state != crate::generated::model::enums::EvidenceGenerationState::Active
            || request
                .generation
                .as_deref()
                .or(marker.active_generation.as_deref())
                != Some(generation.key.as_str())
            || cursor_generation.is_some_and(|key| key != generation.key)
        {
            return Err(EvidenceError::Cursor);
        }
        let mut facts = Vec::with_capacity(rows.len());
        for row in rows {
            let row = wire::decode_evidence_facts(row)?;
            if row.generation != generation.key {
                return Err(EvidenceError::Cursor);
            }
            facts.push(EvidenceFact {
                member: row.member,
                kind: row.kind,
                status: row.status,
                crate_name: row.crate_name,
                test_id: row.test_id,
                detail_ref: row.detail_ref,
                binary_id: row.binary_id,
                context_id: row.context_id,
                context_json: row.context_json,
                link_relation: row.link_relation,
                link_target: row.link_target,
                population_id: row.population_id,
                sample_id: row.sample_id,
                value_ns: row.value_ns,
            });
        }
        let more = facts.len() > limit;
        facts.truncate(limit);
        let next = if more {
            facts.last().map(|f| EvidenceCursor {
                issuing_store: self.issuing_store.clone(),
                generation: generation.key.clone(),
                query_digest,
                after: f.member.clone(),
            })
        } else {
            None
        };
        Ok(EvidencePage {
            issuing_store: self.issuing_store.clone(),
            generation: generation.key,
            authority: generation.authority,
            source_id: generation.source_id,
            snapshot_digest: generation.snapshot_digest,
            coverage: generation.coverage.as_str().to_owned(),
            freshness: "imported-cut-only",
            facts,
            next,
        })
    }
    /// Materialize only caller-selected immutable activated generations. No latest-pointer
    /// inference can turn one imported invocation into a cross-run population.
    pub async fn source_cut(
        &self,
        generations: &[String],
        history: bool,
    ) -> Result<Vec<EvidenceSnapshot>, EvidenceError> {
        if generations.is_empty() || generations.len() > 64 {
            return Err(config("select 1..=64 explicit activated generation IDs"));
        }
        let mut unique = BTreeSet::new();
        let mut sources = BTreeSet::new();
        let mut snapshots = Vec::new();
        for id in generations {
            if id.len() != 64
                || !id
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                return Err(config("exact lower-case generation identity required"));
            }
            if !unique.insert(id) {
                continue;
            }
            let row = self.generation(id).await?;
            if row.state != crate::generated::model::enums::EvidenceGenerationState::Active {
                return Err(EvidenceError::Unavailable(format!(
                    "generation {id} is staged, not activated"
                )));
            }
            let identity = (
                row.authority.clone(),
                row.codec.clone(),
                row.source_id.clone(),
            );
            if !history && !sources.insert(identity) {
                return Err(config(
                    "ambiguous source versions; select one or explicitly request history",
                ));
            }
            let mut facts = Vec::new();
            let mut cursor = None;
            loop {
                let page = self
                    .select(&EvidenceQuery {
                        generation: Some(id.clone()),
                        cursor,
                        ..EvidenceQuery::default()
                    })
                    .await?;
                facts.extend(page.facts);
                cursor = page.next;
                if cursor.is_none() {
                    break;
                }
            }
            if u64::try_from(facts.len()).ok() != Some(row.expected_members) {
                return Err(EvidenceError::Unavailable(
                    "activated membership differs".into(),
                ));
            }
            let snapshot = EvidenceSnapshot {
                version: 2,
                authority: row.authority,
                codec: row.codec,
                source_id: row.source_id,
                snapshot_digest: row.snapshot_digest,
                coverage: row.coverage.as_str().to_owned(),
                expected_members: facts.iter().map(|f| f.member.clone()).collect(),
                facts,
            };
            if snapshot.identity()?.1 != row.content_digest {
                return Err(EvidenceError::Reused);
            }
            snapshots.push(snapshot);
        }
        Ok(snapshots)
    }
    /// Discover bounded prior source cuts, including interrupted staged imports.
    pub async fn generations(
        &self,
        after: Option<&str>,
        limit: usize,
    ) -> Result<serde_json::Value, EvidenceError> {
        if !(1..=64).contains(&limit) {
            return Err(config("generation page must be 1..=64"));
        }
        if after.is_some_and(|v| {
            v.len() != 64
                || !v
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        }) {
            return Err(config(
                "generation continuation must be an exact lower-case SHA256 identity",
            ));
        }
        let mut r=transport::bounded_query(self.query("BEGIN; SELECT * FROM ONLY evidence_interpretations:current; SELECT * FROM evidence_generations WHERE key>$after ORDER BY key LIMIT $limit; COMMIT;")?.bind(("after",after.unwrap_or("").to_owned())).bind(("limit",(limit+1) as i64))).await?;
        let marker: Option<Object> = r.take(1)?;
        self.decode_marker(
            marker.ok_or_else(|| EvidenceError::Unavailable("marker absent".into()))?,
            true,
        )?;
        let rows: Vec<Object> = r.take(2)?;
        let more = rows.len() > limit;
        let mut output = Vec::new();
        let mut last = None;
        for row in rows.into_iter().take(limit) {
            let row = wire::decode_evidence_generations(row)?;
            last = Some(row.key.clone());
            output.push(serde_json::json!({"generation":row.key,"authority":row.authority,"codec":row.codec,"source_id":row.source_id,"snapshot_digest":row.snapshot_digest,"coverage":row.coverage,"expected_members":row.expected_members,"state":row.state}));
        }
        Ok(
            serde_json::json!({"issuing_store":self.issuing_store,"generations":output,"next_after":if more{last}else{None},"freshness":"imported-cut-only"}),
        )
    }
    /// Explicit privileged initialization only. The root account is never retained by an open store.
    pub async fn initialize(
        options: &ToolingOptions,
        username: &str,
        password: &str,
    ) -> Result<(), EvidenceError> {
        within_allowance(
            original_deadline(canonical::REQUEST_TIMEOUT).into_std(),
            Self::initialize_selected(options, username, password),
        )
        .await
    }
    async fn initialize_selected(
        options: &ToolingOptions,
        username: &str,
        password: &str,
    ) -> Result<(), EvidenceError> {
        let db = Self::client(options).await?;
        let deadline = original_deadline(canonical::REQUEST_TIMEOUT);
        let result=async{
            transport::request(db.signin(surrealdb::opt::auth::Root{username:username.into(),password:password.into()}).request_context(RequestContext::control(deadline.into_std()))).await?;
            transport::bounded_query(db.query(format!("DEFINE NAMESPACE IF NOT EXISTS `{}`;",options.namespace)).request_context(RequestContext::control(deadline.into_std()))).await?;
            transport::request(db.use_ns(&options.namespace).request_context(RequestContext::control(deadline.into_std()))).await?;
            transport::bounded_query(db.query(format!("DEFINE DATABASE IF NOT EXISTS `{}`;",options.database)).request_context(RequestContext::control(deadline.into_std()))).await?;
            transport::request(db.use_ns(&options.namespace).use_db(&options.database).request_context(RequestContext::control(deadline.into_std()))).await?;
            let marker=crate::generated::model::runtime::evidence_interpretations::Row{key:"current".into(),interpretation:wire::INTERPRETATION.into(),schema_digest:wire::SCHEMA_DIGEST.into(),issuing_store:uuid::Uuid::new_v4().to_string(),service_instance_id:options.service_instance_id.clone(),active_generation:None};
            let input=wire::encode_evidence_interpretations(&marker)?;
            let reader=serde_json::to_string(&options.credentials.reader_password).map_err(|e|config(e.to_string()))?;let importer=serde_json::to_string(&options.credentials.importer_password).map_err(|e|config(e.to_string()))?;
            let sql=format!("BEGIN; IF time::micros()>=$expiry {{ THROW 'evidence original initialization deadline expired'; }}; {} {} CREATE knowledge_state:current SET key='current'; CREATE evidence_interpretations:current CONTENT $marker; DEFINE USER `{}` ON DATABASE PASSWORD {reader} ROLES VIEWER; DEFINE USER `{}` ON DATABASE PASSWORD {importer} ROLES EDITOR; IF time::micros()>=$expiry {{ THROW 'evidence original initialization deadline expired'; }}; COMMIT;",wire::SCHEMA,crate::generated::store::knowledge::SCHEMA,options.credentials.reader_username,options.credentials.importer_username);
            transport::bounded_query(db.query(sql).bind(("marker",input)).bind(("expiry",rpc_expiry(deadline)?)).request_context(RequestContext::control(deadline.into_std()))).await.map_err(|source|EvidenceError::LifecycleFailure{operation:"initialize",namespace:options.namespace.clone(),database:options.database.clone(),issuing_store:marker.issuing_store.clone(),source:Box::new(source)})?;Ok::<_,EvidenceError>(())
        }.await;
        let drained = db.disconnect().await;
        result?;
        drained?;
        let store = Self::open(options, EvidenceAccess::Reader).await?;
        store.disconnect().await?;
        Ok(())
    }
    /// Explicit privileged disposal of this tooling database only. Original owners are untouched.
    /// Issuing-authority validation and removal share one native SSI transaction.
    pub async fn drop_database(
        options: &ToolingOptions,
        username: &str,
        password: &str,
    ) -> Result<(), EvidenceError> {
        within_allowance(
            original_deadline(canonical::REQUEST_TIMEOUT).into_std(),
            Self::drop_selected(options, username, password),
        )
        .await
    }
    async fn drop_selected(
        options: &ToolingOptions,
        username: &str,
        password: &str,
    ) -> Result<(), EvidenceError> {
        // Refuse to drop an absent/wrong interpretation or a foreign service's database.
        let store = Self::open_selected(options, EvidenceAccess::Reader, false).await?;
        let expected_issuer = store.issuing_store.clone();
        store.disconnect().await?;
        #[cfg(test)]
        if let Some((entered, resume)) = DROP_PAUSE.with(|pause| pause.borrow_mut().take()) {
            entered.notify_one();
            resume.notified().await;
        }
        let db = Self::client(options).await?;
        let deadline = original_deadline(canonical::REQUEST_TIMEOUT);
        let result=async{
            transport::request(db.signin(surrealdb::opt::auth::Root{username:username.into(),password:password.into()}).request_context(RequestContext::control(deadline.into_std()))).await?;
            transport::request(db.use_ns(&options.namespace).use_db(&options.database).request_context(RequestContext::control(deadline.into_std()))).await?;
            let sql=format!("BEGIN; IF time::micros()>=$expiry {{ THROW 'evidence original disposal deadline expired'; }}; LET $marker=SELECT * FROM ONLY evidence_interpretations:current; IF $marker=NONE OR $marker.issuing_store!=$issuer OR $marker.interpretation!=$interpretation OR $marker.service_instance_id!=$service {{ THROW 'evidence disposal authority changed'; }}; REMOVE DATABASE `{}`; COMMIT;",options.database);
            transport::protected_query("evidence-dispose",||Ok(db.query(sql.clone()).bind(("expiry",rpc_expiry(deadline).map_err(|e|CanonicalError::Configuration(e.to_string()))?)).bind(("issuer",expected_issuer.clone())).bind(("interpretation",wire::INTERPRETATION)).bind(("service",options.service_instance_id.clone())).request_context(RequestContext::control(deadline.into_std())))).await.map_err(|source|EvidenceError::LifecycleFailure{operation:"drop",namespace:options.namespace.clone(),database:options.database.clone(),issuing_store:expected_issuer.clone(),source:Box::new(source)})?;
            Ok::<_,EvidenceError>(())
        }.await;
        let drained = db.disconnect().await;
        result?;
        drained?;
        Ok(())
    }
}
#[cfg(test)]
thread_local! {static DROP_PAUSE:std::cell::RefCell<Option<(Arc<tokio::sync::Notify>,Arc<tokio::sync::Notify>)>>=const{std::cell::RefCell::new(None)};}
#[cfg(test)]
#[path = "evidence_tests.rs"]
mod tests;
