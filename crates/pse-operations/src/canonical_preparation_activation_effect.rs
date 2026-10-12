// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native offline preparation activation, driven only by the live Python supervisor.
//! No ordinary store handle, string permit or manifest grants write authority.

use crate::{
    canonical::CanonicalError,
    canonical_preparation_activation::{ActivationBinding, HistoricalProduct, PreparationBackfill},
    canonical_staging::{ProductBlob, SOURCE_BLOCK_BYTES},
    generated::surreal as wire,
};
use pse_ids::{Frame, derive_hash};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::{BufRead, Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};
use surrealdb::{
    Surreal,
    engine::remote::ws::{Client, Ws},
    opt::{Config, RequestContext, WebsocketConfig, auth::Root},
    types::{Object, Value},
};

mod history;
#[cfg(feature = "test-support")]
mod history_fixture;
/// Produces a private historical result batch, block and outputs for activation fixtures.
#[cfg(feature = "test-support")]
pub type FixtureProducer<'a> = dyn Fn(
        bool,
        &str,
        u64,
        u64,
    ) -> Result<
        (
            pse_model::generated::runtime::canonical_result_batches::Row,
            pse_model::generated::runtime::canonical_result_blocks::Row,
            Vec<pse_model::generated::runtime::canonical_result_block_outputs::Row>,
        ),
        CanonicalError,
    > + Send
    + Sync
    + 'a;

const SNAPSHOT_LIMIT: usize = 16 * 1024 * 1024;
const ALLOCATION_LIMIT: usize = 128 * 1024 * 1024;
const RPC_LIMIT: Duration = Duration::from_secs(120);

fn invalid(message: impl Into<String>) -> CanonicalError {
    CanonicalError::Configuration(message.into())
}
fn io(error: impl std::fmt::Display) -> CanonicalError {
    invalid(error.to_string())
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct SavedRow {
    table: String,
    key: String,
    row: Object,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    operation: Vec<u8>,
    marker: Object,
    roots: Vec<Object>,
    records: Vec<SavedRow>,
    #[serde(default)]
    history: Option<history::Archive>,
}

#[derive(Debug)]
struct SnapshotOwner {
    _bytes: Vec<u8>,
}

/// A live parent relationship and retained control pipe, never a caller-made permit.
struct Supervisor {
    state: PathBuf,
    owner: serde_json::Value,
    operation: Option<String>,
    step: u64,
    phase: Option<String>,
    snapshot: Option<(PathBuf, String)>,
}
impl Supervisor {
    fn new(state: &Path) -> Result<Self, CanonicalError> {
        let owner = read_json(&state.join("lifecycle-owner.json"))?;
        let selected = Self {
            state: state.into(),
            owner,
            operation: None,
            step: 0,
            phase: None,
            snapshot: None,
        };
        selected.check_parent()?;
        Ok(selected)
    }
    fn check_parent(&self) -> Result<(), CanonicalError> {
        let process = fs::read_to_string("/proc/self/stat").map_err(io)?;
        let parent = process
            .rsplit_once(')')
            .ok_or_else(|| invalid("process identity unavailable"))?
            .1
            .split_whitespace()
            .nth(1)
            .ok_or_else(|| invalid("parent identity unavailable"))?;
        let pid = self.owner["pid"]
            .as_u64()
            .ok_or_else(|| invalid("reserved parent absent"))?;
        let raw = fs::read_to_string(format!("/proc/{pid}/stat")).map_err(io)?;
        let fields = raw
            .rsplit_once(')')
            .ok_or_else(|| invalid("parent identity malformed"))?
            .1
            .split_whitespace()
            .collect::<Vec<_>>();
        let listener = read_json(&self.state.join(".maintenance-listener.json"))?;
        let config = read_json(&self.state.join("config.json"))?;
        if parent != pid.to_string()
            || self.owner["purpose"] != "maintenance"
            || read_json(&self.state.join("lifecycle-owner.json"))? != self.owner
            || listener["owner"] != self.owner
            || fields.get(19).copied() != self.owner["start"].as_str()
            || matches!(fields.first(), Some(&"Z" | &"X" | &"x"))
            || fs::read_to_string("/proc/sys/kernel/random/boot_id")
                .map_err(io)?
                .trim()
                != self.owner["boot"]
                    .as_str()
                    .ok_or_else(|| invalid("owner boot absent"))?
            || config["accepting_writes"] != false
            || config["admission"] != "validation_required"
            || config["instance_id"] != listener["instance_id"]
            || config["server"] != listener["server"]
            || config["resources"] != listener["resources"]
            || config["service_supervisor"] != listener["supervisor"]
        {
            return Err(CanonicalError::Quiesced);
        }
        Ok(())
    }
    fn check_snapshot(&self) -> Result<(), CanonicalError> {
        if let Some((path, digest)) = &self.snapshot
            && derive_hash(
                Frame::CanonicalSchemaV1,
                &[&read_bytes(path, SNAPSHOT_LIMIT)?],
            )
            .to_hex()
                != *digest
        {
            return Err(invalid("retained original native snapshot changed"));
        }
        Ok(())
    }
    fn archive_boundary(&mut self, phase: &str) -> Result<(), CanonicalError> {
        self.check_snapshot()?;
        self.phase = Some(phase.into());
        self.authorize()?;
        self.check_snapshot()
    }
    fn authorize(&mut self) -> Result<(), CanonicalError> {
        self.check_parent()?;
        self.step = self
            .step
            .checked_add(1)
            .ok_or(CanonicalError::PayloadLimit)?;
        let mut output = std::io::stdout().lock();
        serde_json::to_writer(
            &mut output,
            &serde_json::json!({"event":"authorize","step":self.step,"phase":self.phase.take()}),
        )
        .map_err(io)?;
        output.write_all(b"\n").map_err(io)?;
        output.flush().map_err(io)?;
        let mut line = String::new();
        std::io::stdin()
            .lock()
            .take(65537)
            .read_line(&mut line)
            .map_err(io)?;
        if line.len() > 65536 {
            return Err(CanonicalError::PayloadLimit);
        }
        let answer: serde_json::Value = serde_json::from_str(&line).map_err(io)?;
        let operation = answer["operation"]
            .as_str()
            .ok_or_else(|| invalid("authentic supervisor acknowledgment absent"))?;
        if answer["step"].as_u64() != Some(self.step)
            || self
                .operation
                .as_deref()
                .is_some_and(|previous| previous != operation)
        {
            return Err(invalid("supervisor acknowledgment changed"));
        }
        self.operation = Some(operation.into());
        self.check_parent()
    }
}

fn read_bytes(path: &Path, limit: usize) -> Result<Vec<u8>, CanonicalError> {
    let metadata = fs::symlink_metadata(path).map_err(io)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > limit as u64 {
        return Err(CanonicalError::PayloadLimit);
    }
    fs::read(path).map_err(io)
}
fn read_json(path: &Path) -> Result<serde_json::Value, CanonicalError> {
    serde_json::from_slice(&read_bytes(path, 64 * 1024)?).map_err(io)
}
fn freeze(path: &Path, bytes: &[u8]) -> Result<(), CanonicalError> {
    if path.exists() {
        if read_bytes(path, SNAPSHOT_LIMIT)? != bytes {
            return Err(invalid("immutable native activation snapshot differs"));
        }
        return Ok(());
    }
    // A temporary complete file is synced before publication. Unknown interrupted
    // temporary files are retained, never overwritten or inferred to be disposable.
    let pending = path.with_file_name(format!(".native-pending-{}", uuid::Uuid::new_v4()));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&pending).map_err(io)?;
    file.write_all(bytes).map_err(io)?;
    file.sync_all().map_err(io)?;
    fs::hard_link(&pending, path).map_err(io)?;
    fs::File::open(
        path.parent()
            .ok_or_else(|| invalid("snapshot parent absent"))?,
    )
    .map_err(io)?
    .sync_all()
    .map_err(io)?;
    // Both names are retained: this executor has no deletion authority.
    Ok(())
}

struct Counted {
    bytes: usize,
}
impl Write for Counted {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.bytes = self
            .bytes
            .checked_add(bytes.len())
            .filter(|n| *n <= SNAPSHOT_LIMIT)
            .ok_or_else(|| std::io::Error::other("snapshot extent exhausted"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn encoded_size(value: &impl Serialize) -> Result<usize, CanonicalError> {
    let mut count = Counted { bytes: 0 };
    serde_json::to_writer(&mut count, value).map_err(io)?;
    Ok(count.bytes)
}
fn bounded_encode(value: &impl Serialize) -> Result<Vec<u8>, CanonicalError> {
    let extent = encoded_size(value)?;
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(extent).map_err(io)?;
    serde_json::to_writer(&mut bytes, value).map_err(io)?;
    Ok(bytes)
}

struct OfflineClient {
    db: Surreal<Client>,
    supervisor: Supervisor,
}
impl OfflineClient {
    async fn connect(state: &Path) -> Result<Self, CanonicalError> {
        let mut supervisor = Supervisor::new(state)?;
        supervisor.authorize()?;
        let config = read_json(&state.join("config.json"))?;
        let credentials = read_json(&state.join("credentials.json"))?;
        let field = |key: &str| -> Result<String, CanonicalError> {
            config[key]
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| invalid("deployment field absent"))
        };
        let endpoint = field("endpoint")?;
        if !endpoint.starts_with("ws://127.0.0.1:") {
            return Err(invalid("offline endpoint is not owned loopback"));
        }
        let options = Config::new()
            .bounded_requests()
            .websocket(WebsocketConfig::new().max_message_size(wire::RESULT_MESSAGE_BYTES))?;
        let db = tokio::time::timeout(
            RPC_LIMIT,
            Surreal::new::<Ws>((endpoint.trim_start_matches("ws://"), options)),
        )
        .await
        .map_err(|_| CanonicalError::Timeout)??;
        supervisor.authorize()?;
        tokio::time::timeout(
            RPC_LIMIT,
            db.signin(Root {
                username: credentials["username"]
                    .as_str()
                    .ok_or_else(|| invalid("maintenance account absent"))?
                    .into(),
                password: credentials["password"]
                    .as_str()
                    .ok_or_else(|| invalid("maintenance secret absent"))?
                    .into(),
            })
            .request_context(RequestContext::control(Instant::now() + RPC_LIMIT)),
        )
        .await
        .map_err(|_| CanonicalError::Timeout)??;
        supervisor.authorize()?;
        tokio::time::timeout(
            RPC_LIMIT,
            db.use_ns(field("namespace")?)
                .use_db(field("database")?)
                .request_context(RequestContext::control(Instant::now() + RPC_LIMIT)),
        )
        .await
        .map_err(|_| CanonicalError::Timeout)??;
        Ok(Self { db, supervisor })
    }
    async fn query(
        &mut self,
        sql: &str,
        bindings: Object,
    ) -> Result<surrealdb::IndexedResults, CanonicalError> {
        self.supervisor.authorize()?;
        let mut response = tokio::time::timeout(
            RPC_LIMIT,
            self.db
                .query(sql)
                .bind(bindings)
                .request_context(RequestContext::control(Instant::now() + RPC_LIMIT)),
        )
        .await
        .map_err(|_| CanonicalError::Timeout)??;
        response = crate::canonical::checked(response)?;
        self.supervisor.check_parent()?;
        Ok(response)
    }
    async fn record(&mut self, table: &str, key: &str) -> Result<Option<Object>, CanonicalError> {
        let mut response = self
            .query(
                "SELECT * FROM ONLY type::record($table,$key);",
                Object::from(BTreeMap::from([
                    ("table", Value::String(table.into())),
                    ("key", Value::String(key.into())),
                ])),
            )
            .await?;
        Ok(response.take(0)?)
    }
    async fn roots(&mut self) -> Result<Vec<Object>, CanonicalError> {
        let mut roots = Vec::new();
        let mut after = Value::None;
        let mut extent = 0usize;
        loop {
            let mut response = self.query("SELECT * FROM canonical_roots WHERE owner_kind='product' AND ($after=NONE OR key>$after) ORDER BY key LIMIT 64;", Object::from(BTreeMap::from([("after",after.clone())]))).await?;
            let page: Vec<Object> = response.take(0)?;
            if page.is_empty() {
                return Ok(roots);
            }
            for row in page {
                let decoded = wire::decode_canonical_roots(row.clone())?;
                if decoded.key.is_empty()
                    || [
                        &decoded.key,
                        &decoded.problem,
                        &decoded.revision,
                        &decoded.owner_kind,
                        &decoded.owner,
                    ]
                    .iter()
                    .any(|value| value.len() > 4096)
                    || matches!(&after,Value::String(previous) if decoded.key<=*previous)
                {
                    return Err(invalid("root snapshot order or shape changed"));
                }
                extent = extent
                    .checked_add(encoded_size(&row)?)
                    .filter(|n| *n <= SNAPSHOT_LIMIT / 4)
                    .ok_or(CanonicalError::PayloadLimit)?;
                if roots.len() >= 4096 {
                    return Err(CanonicalError::PayloadLimit);
                }
                after = Value::String(decoded.key);
                roots.push(row);
            }
        }
    }
    async fn compare_create(
        &mut self,
        table: &str,
        key: &str,
        row: Object,
    ) -> Result<(), CanonicalError> {
        self.compare_create_statement(
            table,
            key,
            row,
            "CREATE type::record($table,$key) CONTENT $row RETURN NONE;",
        )
        .await
    }
    #[cfg(feature = "test-support")]
    async fn compare_create_relation(
        &mut self,
        table: &str,
        key: &str,
        row: Object,
    ) -> Result<(), CanonicalError> {
        if !matches!(
            table,
            "canonical_run_sources" | "canonical_problem_runs" | "canonical_analysis_inputs"
        ) {
            return Err(invalid("unsupported fixture relation owner"));
        }
        self.compare_create_statement(
            table,
            key,
            row,
            "INSERT RELATION object::extend($row,{id:type::record($table,$key)}) RETURN NONE;",
        )
        .await
    }
    async fn compare_create_statement(
        &mut self,
        table: &str,
        key: &str,
        row: Object,
        create: &str,
    ) -> Result<(), CanonicalError> {
        let sql = format!(
            "BEGIN; LET $saved=SELECT * OMIT id FROM ONLY type::record($table,$key); IF $saved=NONE {{ {create} }} ELSE IF $saved!=$row {{ THROW 'offline immutable collision'; }}; COMMIT;"
        );
        self.query(
            &sql,
            Object::from(BTreeMap::from([
                ("table", Value::String(table.into())),
                ("key", Value::String(key.into())),
                ("row", Value::Object(row.clone())),
            ])),
        )
        .await
        .map_err(|error| invalid(format!("offline row {table}/{key}: {error}")))?;
        self.supervisor.phase = Some(format!("readback:{table}"));
        let mut saved = self
            .record(table, key)
            .await?
            .ok_or_else(|| invalid("created offline row absent"))?;
        saved.remove("id");
        if saved != row {
            return Err(invalid("offline created row differs"));
        }
        Ok(())
    }
}

async fn saved_record(
    client: &mut OfflineClient,
    records: &mut BTreeMap<(String, String), Object>,
    extent: &mut usize,
    table: &str,
    key: &str,
) -> Result<Object, CanonicalError> {
    let identity = (table.to_owned(), key.to_owned());
    if let Some(row) = records.get(&identity) {
        return Ok(row.clone());
    }
    let row = client
        .record(table, key)
        .await?
        .ok_or_else(|| invalid(format!("historical material missing: {table}/{key}")))?;
    *extent = extent
        .checked_add(encoded_size(&row)?)
        .filter(|n| *n <= SNAPSHOT_LIMIT / 2)
        .ok_or(CanonicalError::PayloadLimit)?;
    records.insert(identity, row.clone());
    Ok(row)
}

fn hydrate(
    records: &BTreeMap<(String, String), Object>,
    version: &str,
) -> Result<pse_model::generated::runtime::canonical_versions::Row, CanonicalError> {
    let get = |table: &str, key: &str| {
        records
            .get(&(table.into(), key.into()))
            .cloned()
            .ok_or_else(|| invalid(format!("historical snapshot lacks {table}/{key}")))
    };
    let header =
        wire::decode_canonical_version_manifests(get("canonical_version_manifests", version)?)?;
    let length = usize::try_from(header.payload_len).map_err(io)?;
    if !header.closed
        || header.key != version
        || !crate::canonical::historical_product_interpretation_supported(&header.interpretation)
        || header.reference_count != 0
        || length > SNAPSHOT_LIMIT
        || header.block_count != length.div_ceil(SOURCE_BLOCK_BYTES) as u64
    {
        return Err(invalid(format!(
            "unknown historical blob manifest: {version}"
        )));
    }
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(length).map_err(io)?;
    for ordinal in 0..header.block_count {
        let key = format!("{version}:{ordinal}");
        let block = wire::decode_canonical_payload_blocks(get("canonical_payload_blocks", &key)?)?;
        if block.key != key
            || block.version != version
            || block.ordinal != ordinal
            || block.payload.len() != (length - bytes.len()).min(SOURCE_BLOCK_BYTES)
            || block.digest
                != derive_hash(Frame::CanonicalPayloadV1, &[block.payload.as_slice()]).to_hex()
        {
            return Err(invalid(format!("historical block differs: {key}")));
        }
        bytes.extend_from_slice(block.payload.as_slice());
    }
    if bytes.len() != length
        || derive_hash(Frame::CanonicalPayloadV1, &[&bytes]).to_hex() != header.payload_digest
    {
        return Err(invalid(format!("historical payload differs: {version}")));
    }
    Ok(pse_model::generated::runtime::canonical_versions::Row {
        key: version.into(),
        logical: header.logical,
        kind: header.kind,
        interpretation: header.interpretation,
        payload: bytes.into(),
    })
}

const LEGACY_SCHEMA: &str = "fbd5217949165ea2e4a4c18ccb28d12a2fc2ff255819f18c6bf486676b0cc916";
fn source_marker(
    raw: &Object,
    operation: &[u8],
) -> Result<pse_model::generated::runtime::canonical_interpretations::Row, CanonicalError> {
    if raw.get("interpretation") == Some(&Value::String("pse.substrate.v3".into())) {
        let mut original = raw.clone();
        if let Some(id) = original.remove("id") {
            crate::canonical_codec::check_record_identity(
                id,
                "canonical_interpretations",
                "current",
            )?;
        }
        let expected = Object::from(BTreeMap::from([
            ("key", Value::String("current".into())),
            ("interpretation", Value::String("pse.substrate.v3".into())),
            ("schema_digest", Value::String(LEGACY_SCHEMA.into())),
        ]));
        if original != expected {
            return Err(invalid("unsupported v3 marker shape or schema digest"));
        }
        let operation: serde_json::Value = serde_json::from_slice(operation).map_err(io)?;
        let issuing_store = operation["legacy_issuing_store"]
            .as_str()
            .ok_or_else(|| invalid("original legacy issuer assignment absent"))?;
        uuid::Uuid::parse_str(issuing_store).map_err(io)?;
        return Ok(
            pse_model::generated::runtime::canonical_interpretations::Row {
                key: "current".into(),
                interpretation: "pse.substrate.v3".into(),
                schema_digest: LEGACY_SCHEMA.into(),
                issuing_store: issuing_store.into(),
            },
        );
    }
    let marker = wire::decode_canonical_interpretations(raw.clone())?;
    if marker.key != "current"
        || marker.interpretation != wire::INTERPRETATION
        || marker.schema_digest != wire::SCHEMA_DIGEST
        || uuid::Uuid::parse_str(&marker.issuing_store).is_err()
    {
        return Err(invalid("unsupported original marker or issuing UUID"));
    }
    Ok(marker)
}

async fn supported_histories(
    client: &mut OfflineClient,
    allow_derived: bool,
) -> Result<(), CanonicalError> {
    let current = client
        .record("canonical_interpretations", "current")
        .await?
        .is_some_and(|row| {
            row.get("interpretation") == Some(&Value::String(wire::INTERPRETATION.into()))
        });
    let mut markers=client.query("SELECT key FROM canonical_interpretations WHERE id!=canonical_interpretations:current LIMIT 1;",Object::new()).await?;
    if !markers.take::<Vec<Object>>(0)?.is_empty() {
        return Err(invalid(
            "unsupported additional canonical interpretation marker",
        ));
    }
    let mut response = client.query("INFO FOR DB;", Object::new()).await?;
    let info: Value = response.take(0)?;
    let Value::Object(info) = info else {
        return Err(invalid("historical database inventory is not an object"));
    };
    let Some(Value::Object(tables)) = info.get("tables") else {
        return Err(invalid("historical table inventory unavailable"));
    };
    let allowed = [
        "canonical_edges",
        "canonical_guards",
        "canonical_interpretations",
        "canonical_memberships",
        "canonical_payload_blocks",
        "canonical_problems",
        "canonical_products",
        "canonical_protections",
        "canonical_reclaimed_ranges",
        "canonical_revisions",
        "canonical_roots",
        "canonical_staged_edits",
        "canonical_stages",
        "canonical_version_manifests",
        "canonical_version_receipts",
        "canonical_versions",
    ];
    for table in tables.keys() {
        if allowed.contains(&table.as_str())
            || history::TABLES.contains(&table.as_str())
            || ((current || allow_derived) && history::CURRENT_TABLES.contains(&table.as_str()))
        {
            continue;
        }
        let mut result = client
            .query(
                "SELECT key FROM type::table($table) LIMIT 1;",
                Object::from(BTreeMap::from([("table", Value::String(table.clone()))])),
            )
            .await?;
        let rows: Vec<Object> = result.take(0)?;
        if !rows.is_empty() {
            return Err(invalid(format!(
                "unsupported non-preparation history requires its own activation adapter: {table}"
            )));
        }
    }
    let mut response=client.query("SELECT key,owner_kind FROM canonical_roots WHERE owner_kind NOT IN ['history','product','run','analysis'] LIMIT 1;",Object::new()).await?;
    let roots: Vec<Object> = response.take(0)?;
    if !roots.is_empty() {
        return Err(invalid(format!(
            "unsupported non-preparation root history: {roots:?}"
        )));
    }
    Ok(())
}

async fn capture(
    client: &mut OfflineClient,
    operation: Vec<u8>,
    archive: &Path,
) -> Result<Snapshot, CanonicalError> {
    let marker = client
        .record("canonical_interpretations", "current")
        .await?
        .ok_or_else(|| {
            invalid("original schema marker absent; store UUID must not be fabricated")
        })?;
    source_marker(&marker, &operation)?;
    supported_histories(client, false).await?;
    let roots = client.roots().await?;
    let mut records = BTreeMap::new();
    let mut size = operation
        .len()
        .checked_add(encoded_size(&roots)?)
        .and_then(|n| n.checked_add(encoded_size(&marker).ok()?))
        .ok_or(CanonicalError::PayloadLimit)?;
    for root in &roots {
        let root = wire::decode_canonical_roots(root.clone())?;
        let product = wire::decode_canonical_products(
            saved_record(
                client,
                &mut records,
                &mut size,
                "canonical_products",
                &root.key,
            )
            .await?,
        )?;
        let descriptor = ProductBlob::decode(product.payload.as_slice())?;
        if descriptor.receipt.is_some() {
            return Err(invalid(format!(
                "historical product already uses an unrecognized split route: {}",
                product.key
            )));
        }
        let header = wire::decode_canonical_version_manifests(
            saved_record(
                client,
                &mut records,
                &mut size,
                "canonical_version_manifests",
                &descriptor.version,
            )
            .await?,
        )?;
        let length = usize::try_from(header.payload_len).map_err(io)?;
        if length > SNAPSHOT_LIMIT
            || header.block_count > (SNAPSHOT_LIMIT / SOURCE_BLOCK_BYTES) as u64
        {
            return Err(CanonicalError::PayloadLimit);
        }
        saved_record(
            client,
            &mut records,
            &mut size,
            "canonical_version_receipts",
            &descriptor.version,
        )
        .await?;
        for ordinal in 0..header.block_count {
            saved_record(
                client,
                &mut records,
                &mut size,
                "canonical_payload_blocks",
                &format!("{}:{ordinal}", descriptor.version),
            )
            .await?;
        }
    }
    let history = Some(history::capture(client, archive).await?);
    Ok(Snapshot {
        operation,
        marker,
        roots,
        records: records
            .into_iter()
            .map(|((table, key), row)| SavedRow { table, key, row })
            .collect(),
        history,
    })
}

fn derive(
    snapshot: &Snapshot,
    bytes: Vec<u8>,
    owner: Arc<dyn std::fmt::Debug + Send + Sync>,
) -> Result<PreparationBackfill, CanonicalError> {
    let operation: serde_json::Value = serde_json::from_slice(&snapshot.operation).map_err(io)?;
    let text = |key: &str| {
        operation[key]
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| invalid("immutable operation field absent"))
    };
    let marker = source_marker(&snapshot.marker, &snapshot.operation)?;
    let records = snapshot
        .records
        .iter()
        .map(|row| ((row.table.clone(), row.key.clone()), row.row.clone()))
        .collect::<BTreeMap<_, _>>();
    if records.len() != snapshot.records.len() {
        return Err(invalid("duplicate historical snapshot records"));
    }
    let products = snapshot
        .roots
        .iter()
        .map(|root| {
            let root = wire::decode_canonical_roots(root.clone())?;
            let product = wire::decode_canonical_products(
                records
                    .get(&("canonical_products".into(), root.key.clone()))
                    .cloned()
                    .ok_or_else(|| invalid("historical publication absent"))?,
            )?;
            let descriptor = ProductBlob::decode(product.payload.as_slice())?;
            Ok(HistoricalProduct {
                blob: hydrate(&records, &descriptor.version)?,
                product,
                root,
            })
        })
        .collect::<Result<Vec<_>, CanonicalError>>()?;
    if PreparationBackfill::allocation_bytes(&products)? > ALLOCATION_LIMIT {
        return Err(CanonicalError::PayloadLimit);
    }
    PreparationBackfill::derive(
        ActivationBinding {
            migration: text("migration")?,
            issuing_store: marker.issuing_store,
            old_schema_digest: marker.schema_digest,
            snapshot: derive_hash(Frame::CanonicalSchemaV1, &[&bytes]).to_hex(),
            borrowers: text("borrowers_digest")?,
        },
        products,
        Arc::new((SnapshotOwner { _bytes: bytes }, owner)),
    )
}

fn schema_install() -> Result<String, CanonicalError> {
    // Only generated declarations are installed. The initializer's UUID/marker
    // transaction is deliberately not reused. OVERWRITE changes definition
    // replacement behavior, never the declared contracts or existing row data.
    if !wire::SCHEMA.starts_with("-- @generated")
        || wire::SCHEMA.contains("CREATE canonical_interpretations")
    {
        return Err(invalid("generated schema includes an activation effect"));
    }
    Ok(wire::SCHEMA
        .lines()
        .map(|line| {
            for kind in ["TABLE", "FIELD", "INDEX", "FUNCTION"] {
                let prefix = format!("DEFINE {kind} ");
                if line.starts_with(&prefix) && !line.starts_with(&format!("{prefix}OVERWRITE ")) {
                    return line.replacen(&prefix, &format!("{prefix}OVERWRITE "), 1);
                }
            }
            line.to_owned()
        })
        .collect::<Vec<_>>()
        .join("\n"))
}

async fn verify_originals(
    client: &mut OfflineClient,
    snapshot: &Snapshot,
    archive: &Path,
    backfill: &PreparationBackfill,
    historical: Option<&history::Archive>,
    complete: bool,
) -> Result<(), CanonicalError> {
    supported_histories(client, true).await?;
    if client.roots().await? != snapshot.roots {
        return Err(invalid("original rooted publication inventory changed"));
    }
    for record in &snapshot.records {
        if client.record(&record.table, &record.key).await?.as_ref() != Some(&record.row) {
            return Err(invalid(format!(
                "original historical material changed: {}/{}",
                record.table, record.key
            )));
        }
    }
    history::verify(
        client,
        archive,
        snapshot.history.as_ref(),
        backfill,
        historical,
        complete,
    )
    .await?;
    Ok(())
}

/// Execute only beneath the real reserved supervisor, using its bounded native
/// maintenance listener. The immutable archive and all historical data survive.
pub async fn run(
    state: &Path,
    archive: &Path,
    owner: Arc<dyn std::fmt::Debug + Send + Sync>,
    rebuilder: &dyn crate::canonical_preparation_activation::HistoricalResultRebuilder,
) -> Result<(), CanonicalError> {
    let operation = read_bytes(&archive.join("operation.json"), 64 * 1024)?;
    let mut client = OfflineClient::connect(state).await?;
    let path = archive.join("native-snapshot.json");
    let bytes = if path.exists() {
        read_bytes(&path, SNAPSHOT_LIMIT)?
    } else {
        let snapshot = capture(&mut client, operation.clone(), archive).await?;
        let bytes = bounded_encode(&snapshot)?;
        freeze(&path, &bytes)?;
        bytes
    };
    let snapshot: Snapshot = serde_json::from_slice(&bytes).map_err(io)?;
    if snapshot.operation != operation {
        return Err(invalid("resume changed the original activation binding"));
    }
    let digest = derive_hash(Frame::CanonicalSchemaV1, &[&bytes]).to_hex();
    freeze(
        &archive.join("native-snapshot.identity.json"),
        &serde_json::to_vec(&serde_json::json!({"bytes":bytes.len(),"digest":digest}))
            .map_err(io)?,
    )?;
    client.supervisor.snapshot = Some((path, digest));
    client.supervisor.archive_boundary("archive:source")?;
    let backfill = derive(&snapshot, bytes, owner)?;
    let historical = history::derive(&snapshot, &backfill.binding, archive, rebuilder)?;
    client.supervisor.archive_boundary("archive:derived")?;
    let target = wire::encode_canonical_interpretations(&backfill.marker)?;
    let mut target_with_id = target.clone();
    target_with_id.insert(
        "id",
        crate::canonical_codec::record_link("canonical_interpretations", "current"),
    );
    let current = client
        .record("canonical_interpretations", "current")
        .await?
        .ok_or_else(|| invalid("schema marker disappeared"))?;
    if current != snapshot.marker && current != target_with_id {
        return Err(invalid("resume encountered an unknown schema marker"));
    }
    verify_originals(
        &mut client,
        &snapshot,
        archive,
        &backfill,
        historical.as_ref(),
        false,
    )
    .await?;
    if current == snapshot.marker {
        // Definitions may precede the final marker only while authentic offline
        // admission remains closed. Every failure retains that closed state.
        client
            .query(
                &format!("BEGIN; {}\nCOMMIT;", schema_install()?),
                Object::new(),
            )
            .await?;
    }
    macro_rules! apply {
        ($table:literal,$rows:expr,$encode:path) => {
            for row in $rows {
                client
                    .compare_create($table, &row.key, $encode(row)?)
                    .await?;
            }
        };
    }
    apply!(
        "canonical_body_contents",
        &backfill.contents,
        wire::encode_canonical_body_contents
    );
    apply!(
        "canonical_selection_receipts",
        &backfill.receipts,
        wire::encode_canonical_selection_receipts
    );
    apply!(
        "canonical_product_associations",
        &backfill.associations,
        wire::encode_canonical_product_associations
    );
    apply!(
        "canonical_product_descriptors",
        &backfill.descriptors,
        wire::encode_canonical_product_descriptors
    );
    apply!(
        "canonical_stages",
        &backfill.stages,
        wire::encode_canonical_stages
    );
    apply!(
        "canonical_staged_edits",
        &backfill.staged_edits,
        wire::encode_canonical_staged_edits
    );
    apply!(
        "canonical_version_manifests",
        &backfill.manifests,
        wire::encode_canonical_version_manifests
    );
    apply!(
        "canonical_version_receipts",
        &backfill.version_receipts,
        wire::encode_canonical_version_receipts
    );
    apply!(
        "canonical_payload_blocks",
        &backfill.blocks,
        wire::encode_canonical_payload_blocks
    );
    history::apply(&mut client, archive, historical.as_ref()).await?;
    verify_originals(
        &mut client,
        &snapshot,
        archive,
        &backfill,
        historical.as_ref(),
        true,
    )
    .await?;
    client.supervisor.archive_boundary("archive:final")?;
    // Existing rows have now been read back and compared independently of the
    // create result. Pure derivation checked every original rooted ACK oracle.
    if current == snapshot.marker {
        let mut old = snapshot.marker.clone();
        old.remove("id");
        client.query("BEGIN; LET $old=SELECT * OMIT id FROM ONLY canonical_interpretations:current; IF $old!=$original { THROW 'original activation fence changed'; }; UPDATE ONLY canonical_interpretations:current CONTENT $target RETURN NONE; COMMIT;",Object::from(BTreeMap::from([("original",Value::Object(old)),("target",Value::Object(target.clone()))]))).await?;
    }
    let mut installed = client
        .record("canonical_interpretations", "current")
        .await?
        .ok_or_else(|| invalid("final activation marker absent"))?;
    installed.remove("id");
    if installed != target {
        return Err(invalid("final activation marker differs"));
    }
    client.supervisor.authorize()?;
    tokio::time::timeout(RPC_LIMIT, client.db.disconnect())
        .await
        .map_err(|_| CanonicalError::Timeout)??;
    let mut output = std::io::stdout().lock();
    serde_json::to_writer(&mut output,&serde_json::json!({"event":"complete","migration":backfill.binding.migration,"issuing_store":backfill.binding.issuing_store,"schema_digest":backfill.marker.schema_digest,"source_digest":backfill.source_digest})).map_err(io)?;
    output.write_all(b"\n").map_err(io)?;
    output.flush().map_err(io)?;
    Ok(())
}

/// Native historical fixture material, available only in explicit test-support
/// builds. The caller must create a fresh private supervisor state; this helper
/// still requires the authentic offline parent and authorizes every real RPC.
#[cfg(feature = "test-support")]
pub async fn seed_private_fixture(
    state: &Path,
    _allocation: Arc<dyn std::fmt::Debug + Send + Sync>,
    nonpreparation: bool,
    history: Option<&FixtureProducer<'_>>,
) -> Result<(), CanonicalError> {
    use pse_model::generated::runtime::{
        canonical_payload_blocks::Row as Block, canonical_problems::Row as Problem,
        canonical_products::Row as Product, canonical_revisions::Row as Revision,
        canonical_roots::Row as RootRow, canonical_staged_edits::Row as Edit,
        canonical_stages::Row as Stage, canonical_version_manifests::Row as Manifest,
        canonical_version_receipts::Row as Receipt,
    };
    let mut client = OfflineClient::connect(state).await?;
    let raw = client
        .record("canonical_interpretations", "current")
        .await?
        .ok_or_else(|| invalid("fixture original marker absent"))?;
    let marker = source_marker(
        &raw,
        b"{\"legacy_issuing_store\":\"00000000-0000-4000-8000-000000000001\"}",
    )?;
    if marker.interpretation != "pse.substrate.v3" {
        return Err(invalid("fixture requires exact empty v3 schema"));
    }
    for table in [
        "canonical_products",
        "canonical_roots",
        "canonical_problems",
        "canonical_version_manifests",
    ] {
        let mut result = client
            .query(
                "SELECT key FROM type::table($table) LIMIT 1;",
                Object::from(BTreeMap::from([("table", Value::String(table.into()))])),
            )
            .await?;
        if !result.take::<Vec<Object>>(0)?.is_empty() {
            return Err(invalid("fixture refuses existing canonical material"));
        }
    }
    supported_histories(&mut client, false).await?;
    let selected =
        serde_json::to_vec(&serde_json::json!({"interpretation":"pse.substrate.v3","premises":[]}))
            .map_err(io)?;
    let (descriptor, edit) =
        crate::canonical_staging::product_blob(b"exact-original-historical-body", &selected)?;
    let mut blob = edit
        .version
        .ok_or_else(|| invalid("fixture native combined blob absent"))?;
    blob.interpretation = "pse.substrate.v3".into();
    let digest = derive_hash(Frame::CanonicalPayloadV1, &[blob.payload.as_slice()]).to_hex();
    let len = blob.payload.len() as u64;
    let manifest = Manifest {
        key: blob.key.clone(),
        logical: blob.logical.clone(),
        kind: blob.kind.clone(),
        interpretation: blob.interpretation.clone(),
        payload_digest: digest.clone(),
        payload_len: len,
        block_count: blob.payload.len().div_ceil(SOURCE_BLOCK_BYTES) as u64,
        reference_count: 0,
        creator_stage: "a-original".into(),
        closed: true,
    };
    let receipt = Receipt {
        key: blob.key.clone(),
        content_digest: derive_hash(
            Frame::CanonicalVersionV1,
            &[
                blob.logical.as_bytes(),
                blob.kind.as_bytes(),
                blob.interpretation.as_bytes(),
                &len.to_le_bytes(),
                digest.as_bytes(),
                b"[]",
            ],
        )
        .to_hex(),
    };
    client
        .compare_create(
            "canonical_version_manifests",
            &manifest.key,
            wire::encode_canonical_version_manifests(&manifest)?,
        )
        .await?;
    client
        .compare_create(
            "canonical_version_receipts",
            &receipt.key,
            wire::encode_canonical_version_receipts(&receipt)?,
        )
        .await?;
    for (ordinal, part) in blob
        .payload
        .as_slice()
        .chunks(SOURCE_BLOCK_BYTES)
        .enumerate()
    {
        let block = Block {
            key: format!("{}:{ordinal}", blob.key),
            version: blob.key.clone(),
            ordinal: ordinal as u64,
            payload: part.to_vec().into(),
            digest: derive_hash(Frame::CanonicalPayloadV1, &[part]).to_hex(),
        };
        client
            .compare_create(
                "canonical_payload_blocks",
                &block.key,
                wire::encode_canonical_payload_blocks(&block)?,
            )
            .await?;
    }
    let problem = Problem {
        key: "fixture-problem".into(),
        head: "original-revision".into(),
        sequence: 1,
    };
    client
        .compare_create(
            "canonical_problems",
            &problem.key,
            wire::encode_canonical_problems(&problem)?,
        )
        .await?;
    let revision = Revision {
        key: problem.head.clone(),
        problem: problem.key.clone(),
        sequence: 1,
        parent: None,
        operation: "original-source".into(),
        request: b"original-source-request".to_vec().into(),
        interpretation: blob.interpretation.clone(),
    };
    client
        .compare_create(
            "canonical_revisions",
            &revision.key,
            wire::encode_canonical_revisions(&revision)?,
        )
        .await?;
    let source_root = RootRow {
        key: "history:original-revision".into(),
        problem: problem.key.clone(),
        revision: revision.key.clone(),
        sequence: 1,
        owner_kind: "history".into(),
        owner: revision.key.clone(),
    };
    client
        .compare_create(
            "canonical_roots",
            &source_root.key,
            wire::encode_canonical_roots(&source_root)?,
        )
        .await?;
    for key in ["a-original", "z-original"] {
        let stage = Stage {
            key: key.into(),
            problem: problem.key.clone(),
            expected_head: None,
            request_digest: key.into(),
            expires_at: 0,
            closed: true,
            activated: true,
            edit_count: 1,
            generation: 1,
            abandoned: false,
            cleanup_complete: false,
        };
        let child = Edit {
            key: format!("{key}:0"),
            stage: key.into(),
            ordinal: 0,
            logical: blob.logical.clone(),
            scope: blob.kind.clone(),
            name: blob.kind.clone(),
            version: Some(blob.key.clone()),
        };
        let product = Product {
            key: key.into(),
            problem: problem.key.clone(),
            revision: revision.key.clone(),
            request: b"exact-original-request".to_vec().into(),
            payload: serde_json::to_vec(&descriptor).map_err(io)?.into(),
            dependencies: Vec::new().into(),
            producer: "original-producer".into(),
            interpretation: blob.interpretation.clone(),
        };
        let root = RootRow {
            key: key.into(),
            problem: problem.key.clone(),
            revision: revision.key.clone(),
            sequence: 1,
            owner_kind: "product".into(),
            owner: key.into(),
        };
        client
            .compare_create(
                "canonical_stages",
                key,
                wire::encode_canonical_stages(&stage)?,
            )
            .await?;
        client
            .compare_create(
                "canonical_staged_edits",
                &child.key,
                wire::encode_canonical_staged_edits(&child)?,
            )
            .await?;
        client
            .compare_create(
                "canonical_products",
                key,
                wire::encode_canonical_products(&product)?,
            )
            .await?;
        client
            .compare_create("canonical_roots", key, wire::encode_canonical_roots(&root)?)
            .await?;
    }
    if nonpreparation {
        let operation = pse_model::generated::runtime::canonical_execution_operations::Row {
            issuing_store: "00000000-0000-4000-8000-000000000000".into(), study: None, point: None, minimal:false, disposition:"settled".into(),
            key: "unsupported-execution-history".into(),
            run: "original-run".into(),
            attempt: None,
            kind: "original-operation".into(),
            request: b"preserved-request".to_vec().into(),
            result: b"preserved-result".to_vec().into(),
        };
        client
            .compare_create(
                "canonical_execution_operations",
                &operation.key,
                wire::encode_canonical_execution_operations(&operation)?,
            )
            .await?;
    }
    if let Some(producer) = history {
        history_fixture::seed(&mut client, producer).await?;
    }
    client.supervisor.authorize()?;
    tokio::time::timeout(RPC_LIMIT, client.db.disconnect())
        .await
        .map_err(|_| CanonicalError::Timeout)??;
    let mut output = std::io::stdout().lock();
    output
        .write_all(b"{\"event\":\"complete\",\"fixture\":\"seeded-native-v3-originals\"}\n")
        .map_err(io)?;
    output.flush().map_err(io)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn offline_schema_installer_has_no_initializing_marker_or_uuid() -> Result<(), CanonicalError> {
        let schema = schema_install()?;
        assert!(schema.contains("DEFINE TABLE OVERWRITE canonical_body_contents"));
        assert!(!schema.contains("CREATE canonical_interpretations"));
        assert!(!schema.contains("UPDATE ONLY canonical_interpretations"));
        Ok(())
    }
    #[test]
    fn native_snapshot_preserves_binary_decimal_and_record_values() -> Result<(), CanonicalError> {
        let mut row = Object::new();
        row.insert(
            "binary",
            crate::canonical_codec::encode_bytes(vec![0, 255, 128].into())?,
        );
        row.insert("integer", crate::canonical_codec::encode_uint(u64::MAX)?);
        row.insert(
            "id",
            crate::canonical_codec::record_link("canonical_products", "original:key"),
        );
        let bytes = serde_json::to_vec(&row).map_err(io)?;
        let restored: Object = serde_json::from_slice(&bytes).map_err(io)?;
        assert_eq!(row, restored);
        Ok(())
    }
    #[test]
    fn standalone_bridge_has_no_offline_authority() {
        assert!(Supervisor::new(Path::new("/not-an-owned-state")).is_err());
    }
    #[test]
    fn legacy_issuer_assignment_is_frozen_without_rewriting_original_marker()
    -> Result<(), CanonicalError> {
        let original = Object::from(BTreeMap::from([
            ("key", Value::String("current".into())),
            ("interpretation", Value::String("pse.substrate.v3".into())),
            ("schema_digest", Value::String(LEGACY_SCHEMA.into())),
        ]));
        let operation = serde_json::to_vec(
            &serde_json::json!({"legacy_issuing_store":"00000000-0000-4000-8000-000000000001"}),
        )
        .map_err(io)?;
        let marker = source_marker(&original, &operation)?;
        assert_eq!(marker.issuing_store, "00000000-0000-4000-8000-000000000001");
        assert_eq!(marker.interpretation, "pse.substrate.v3");
        assert!(!original.contains_key("issuing_store"));
        let mut unknown = original.clone();
        unknown.insert("schema_digest", Value::String("unknown".into()));
        assert!(source_marker(&unknown, &operation).is_err());
        assert!(source_marker(&original, b"{}").is_err());
        Ok(())
    }
    #[test]
    fn an_existing_issuing_uuid_is_preserved_instead_of_legacy_assignment()
    -> Result<(), CanonicalError> {
        let original = pse_model::generated::runtime::canonical_interpretations::Row {
            key: "current".into(),
            interpretation: wire::INTERPRETATION.into(),
            schema_digest: wire::SCHEMA_DIGEST.into(),
            issuing_store: "00000000-0000-4000-8000-000000000002".into(),
        };
        let marker = source_marker(
            &wire::encode_canonical_interpretations(&original)?,
            b"{\"legacy_issuing_store\":\"00000000-0000-4000-8000-000000000001\"}",
        )?;
        assert_eq!(marker.issuing_store, original.issuing_store);
        Ok(())
    }
}
