// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Paged immutable historical archive and bounded native reconstruction.
use super::*;
use crate::canonical_execution::{
    decode_result_descriptors, result_batch_key, result_set_key, validate_result_batch,
};
use pse_ids::FramedHasher;
use pse_model::generated::runtime::canonical_historical_results::Row as Admission;
use sha2::{Digest, Sha256};

const PAGE_BYTES: usize = 4 * 1024 * 1024;
const INDEX_BYTES: usize = 128 * 1024 * 1024;
pub(super) const TABLES: &[&str] = &[
    "canonical_runs",
    "canonical_attempts",
    "canonical_result_sets",
    "canonical_result_batches",
    "canonical_result_manifests",
    "canonical_execution_operations",
    "canonical_result_blocks",
    "canonical_result_block_outputs",
    "canonical_result_cells",
    "canonical_result_seeds",
    "canonical_problem_runs",
    "canonical_run_sources",
    "canonical_studies",
    "canonical_study_points",
    "canonical_study_dependencies",
    "canonical_analyses",
    "canonical_analysis_nodes",
    "canonical_analysis_edges",
    "canonical_analysis_inputs",
    "canonical_roots",
    "canonical_revisions",
    "canonical_guards",
    "canonical_protections",
    "canonical_result_protections",
    "canonical_result_retirements",
    "canonical_study_retirements",
];
const SOURCE_TABLES: &[&str] = &[
    "canonical_memberships",
    "canonical_edges",
    "canonical_versions",
    "canonical_version_manifests",
    "canonical_version_receipts",
    "canonical_payload_blocks",
    "canonical_problems",
    "canonical_stages",
    "canonical_staged_edits",
    "canonical_reclaimed_ranges",
    "canonical_products",
];
pub(super) const CURRENT_TABLES: &[&str] = &[
    "canonical_result_rows",
    "canonical_historical_results",
    "canonical_study_readiness",
    "canonical_study_readiness_cursors",
    "canonical_body_contents",
    "canonical_selection_receipts",
    "canonical_product_associations",
    "canonical_product_descriptors",
];

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Page {
    name: String,
    digest: String,
    sha256: String,
    bytes: usize,
    rows: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Archive {
    pages: Vec<Page>,
    digest: String,
}
fn archive_digest(pages: &[Page]) -> String {
    let mut hash = FramedHasher::new(Frame::CanonicalSchemaV1);
    hash.str("offline-history-pages.v1");
    for page in pages {
        hash.str(&page.name)
            .str(&page.digest)
            .str(&page.sha256)
            .u64(page.bytes as u64)
            .u64(page.rows as u64);
    }
    hash.finish_hash().to_hex()
}
fn checksum(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
fn directory(archive: &Path) -> Result<PathBuf, CanonicalError> {
    let path = archive.join("native-history");
    match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {}
        Ok(_) => return Err(invalid("historical archive is not an owned directory")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fs::create_dir(&path).map_err(io)?
        }
        Err(error) => return Err(io(error)),
    }
    Ok(path)
}
fn read_page(directory: &Path, page: &Page) -> Result<Vec<SavedRow>, CanonicalError> {
    if page.name.len() != 20
        || !(page.name.starts_with("source-") || page.name.starts_with("derive-"))
        || !page.name.ends_with(".json")
        || !page.name[7..15].bytes().all(|byte| byte.is_ascii_digit())
        || page.bytes > PAGE_BYTES
    {
        return Err(invalid("historical archive page name or extent"));
    }
    let bytes = read_bytes(&directory.join(&page.name), PAGE_BYTES)?;
    if bytes.len() != page.bytes
        || checksum(&bytes) != page.sha256
        || derive_hash(Frame::CanonicalSchemaV1, &[&bytes]).to_hex() != page.digest
    {
        return Err(invalid("immutable historical archive page differs"));
    }
    let rows: Vec<SavedRow> = serde_json::from_slice(&bytes).map_err(io)?;
    if rows.len() != page.rows {
        return Err(invalid("historical archive page row coverage differs"));
    }
    Ok(rows)
}
struct Writer {
    directory: PathBuf,
    prefix: &'static str,
    pages: Vec<Page>,
    rows: Vec<SavedRow>,
    extent: usize,
}
impl Writer {
    fn new(directory: PathBuf, prefix: &'static str) -> Self {
        Self {
            directory,
            prefix,
            pages: Vec::new(),
            rows: Vec::new(),
            extent: 2,
        }
    }
    fn push(&mut self, row: SavedRow) -> Result<(), CanonicalError> {
        let size = encoded_size(&row)?
            .checked_add(1)
            .ok_or(CanonicalError::PayloadLimit)?;
        if size > PAGE_BYTES - 2 {
            return Err(CanonicalError::PayloadLimit);
        }
        if self
            .extent
            .checked_add(size)
            .is_none_or(|bytes| bytes > PAGE_BYTES)
        {
            self.flush()?;
        }
        self.extent = self
            .extent
            .checked_add(size)
            .ok_or(CanonicalError::PayloadLimit)?;
        self.rows.push(row);
        Ok(())
    }
    fn flush(&mut self) -> Result<(), CanonicalError> {
        if self.rows.is_empty() {
            return Ok(());
        }
        let bytes = serde_json::to_vec(&self.rows).map_err(io)?;
        if bytes.len() > PAGE_BYTES || self.pages.len() >= INDEX_BYTES / 1024 {
            return Err(CanonicalError::PayloadLimit);
        }
        let page = Page {
            name: format!("{}-{:08}.json", self.prefix, self.pages.len()),
            digest: derive_hash(Frame::CanonicalSchemaV1, &[&bytes]).to_hex(),
            sha256: checksum(&bytes),
            bytes: bytes.len(),
            rows: self.rows.len(),
        };
        freeze(&self.directory.join(&page.name), &bytes)?;
        self.pages.push(page);
        self.rows.clear();
        self.extent = 2;
        Ok(())
    }
    fn finish(mut self) -> Result<Archive, CanonicalError> {
        self.flush()?;
        let digest = archive_digest(&self.pages);
        Ok(Archive {
            pages: self.pages,
            digest,
        })
    }
}

pub(super) async fn capture(
    client: &mut OfflineClient,
    archive: &Path,
) -> Result<Archive, CanonicalError> {
    let mut response = client.query("INFO FOR DB;", Object::new()).await?;
    let info: Value = response.take(0)?;
    let Value::Object(info) = info else {
        return Err(invalid("historical table inventory unavailable"));
    };
    let Some(Value::Object(tables)) = info.get("tables") else {
        return Err(invalid("historical table inventory unavailable"));
    };
    let mut writer = Writer::new(directory(archive)?, "source");
    for table in TABLES.iter().chain(SOURCE_TABLES).chain(CURRENT_TABLES) {
        if !tables.contains_key(*table) {
            continue;
        }
        let mut after = String::new();
        loop {
            // Page only keys; a page of maximum-sized payloads exceeds the RPC envelope.
            let mut response = client
                .query(
                    "SELECT key FROM type::table($table) WHERE key>$after ORDER BY key LIMIT 64;",
                    Object::from(BTreeMap::from([
                        ("table", Value::String((*table).into())),
                        ("after", Value::String(after.clone())),
                    ])),
                )
                .await?;
            let page: Vec<Object> = response.take(0)?;
            if page.is_empty() {
                break;
            }
            for mut row in page {
                let key = crate::canonical_codec::decode_string(crate::canonical_codec::required(
                    &mut row, "key",
                )?)?;
                if key.is_empty() || key.len() > 4096 || key <= after {
                    return Err(invalid("historical key inventory order or extent"));
                }
                let row = client
                    .record(table, &key)
                    .await?
                    .ok_or_else(|| invalid("historical snapshot record disappeared"))?;
                writer.push(SavedRow {
                    table: (*table).into(),
                    key: key.clone(),
                    row,
                })?;
                after = key;
            }
        }
    }
    writer.finish()
}

/// Only compact identities and page locations remain resident. Scientific payloads
/// are decoded one original block at a time under the native operation's prepaid owner.
struct Catalog {
    directory: PathBuf,
    pages: Vec<Page>,
    keys: BTreeMap<(String, String), usize>,
    owners: BTreeMap<(String, String), Vec<String>>,
}
fn text(row: &Object, name: &str) -> Result<String, CanonicalError> {
    crate::canonical_codec::decode_string(
        row.get(name)
            .cloned()
            .ok_or_else(|| invalid(format!("historical field absent: {name}")))?,
    )
    .map_err(Into::into)
}
fn owner_field(table: &str) -> Option<&'static str> {
    match table {
        "canonical_attempts"
        | "canonical_run_sources"
        | "canonical_execution_operations"
        | "canonical_result_seeds" => Some("run"),
        "canonical_result_sets" => Some("attempt"),
        "canonical_result_batches" => Some("result_set"),
        "canonical_result_block_outputs" | "canonical_result_cells" => Some("batch"),
        "canonical_study_points" => Some("study"),
        "canonical_study_dependencies" => Some("dependent"),
        "canonical_analysis_nodes" | "canonical_analysis_edges" | "canonical_analysis_inputs" => {
            Some("analysis")
        }
        "canonical_roots" => Some("owner"),
        _ => None,
    }
}
impl Catalog {
    fn open(root: &Path, archive: &Archive) -> Result<Self, CanonicalError> {
        if archive_digest(&archive.pages) != archive.digest {
            return Err(invalid("historical archive hash chain differs"));
        }
        let mut catalog = Self {
            directory: directory(root)?,
            pages: archive.pages.clone(),
            keys: BTreeMap::new(),
            owners: BTreeMap::new(),
        };
        let mut extent = catalog
            .pages
            .len()
            .checked_mul(1024)
            .ok_or(CanonicalError::PayloadLimit)?;
        for (index, page) in archive.pages.iter().enumerate() {
            for row in read_page(&catalog.directory, page)? {
                if row.key.len() > 4096
                    || row.table.len() > 128
                    || text(&row.row, "key")? != row.key
                {
                    return Err(invalid("historical archive malformed identity"));
                }
                let owner = owner_field(&row.table)
                    .map(|field| text(&row.row, field))
                    .transpose()?;
                if owner.as_ref().is_some_and(|owner| owner.len() > 4096) {
                    return Err(CanonicalError::PayloadLimit);
                }
                // Count the actual retained strings before moving them into a tree.
                // 512 covers amortized BTree nodes and tuple storage per entry;
                // each owner Vec reserves one slot exactly, with allocator slack.
                let mut charge = row
                    .table
                    .capacity()
                    .checked_add(row.key.capacity())
                    .and_then(|bytes| bytes.checked_add(512))
                    .ok_or(CanonicalError::PayloadLimit)?;
                if let Some(owner) = &owner {
                    charge = charge
                        .checked_add(row.table.len())
                        .and_then(|bytes| bytes.checked_add(owner.capacity()))
                        .and_then(|bytes| bytes.checked_add(row.key.len()))
                        .and_then(|bytes| bytes.checked_add(640))
                        .ok_or(CanonicalError::PayloadLimit)?;
                }
                extent = extent
                    .checked_add(charge)
                    .filter(|bytes| *bytes <= INDEX_BYTES)
                    .ok_or(CanonicalError::PayloadLimit)?;
                if catalog
                    .keys
                    .insert((row.table.clone(), row.key.clone()), index)
                    .is_some()
                {
                    return Err(invalid("historical archive repeated identity"));
                }
                if let Some(owner) = owner {
                    let members = catalog.owners.entry((row.table, owner)).or_default();
                    members.reserve_exact(1);
                    members.push(row.key);
                }
            }
        }
        Ok(catalog)
    }
    fn get(&self, table: &str, key: &str) -> Result<Object, CanonicalError> {
        let index = *self
            .keys
            .get(&(table.into(), key.into()))
            .ok_or_else(|| invalid(format!("historical owner missing: {table}/{key}")))?;
        read_page(&self.directory, &self.pages[index])?
            .into_iter()
            .find(|row| row.table == table && row.key == key)
            .map(|row| row.row)
            .ok_or_else(|| invalid("historical page index differs"))
    }
    fn has(&self, table: &str, key: &str) -> bool {
        self.keys.contains_key(&(table.into(), key.into()))
    }
    fn table<'a>(&'a self, table: &'a str) -> impl Iterator<Item = &'a str> + 'a {
        self.keys
            .keys()
            .filter(move |(name, _)| name == table)
            .map(|(_, key)| key.as_str())
    }
    fn owned(&self, table: &str, owner: &str) -> &[String] {
        self.owners
            .get(&(table.into(), owner.into()))
            .map_or(&[], Vec::as_slice)
    }
}

pub(super) fn derive(
    snapshot: &Snapshot,
    binding: &ActivationBinding,
    root: &Path,
    rebuilder: &dyn crate::canonical_preparation_activation::HistoricalResultRebuilder,
) -> Result<Option<Archive>, CanonicalError> {
    let Some(archive) = &snapshot.history else {
        return Ok(None);
    };
    let catalog = Catalog::open(root, archive)?;
    let mut output = Writer::new(directory(root)?, "derive");
    validate_sources(&catalog)?;
    validate_owners(&catalog)?;
    for attempt_key in catalog.table("canonical_attempts") {
        let attempt =
            wire::decode_canonical_attempts(catalog.get("canonical_attempts", attempt_key)?)?;
        let run = wire::decode_canonical_runs(catalog.get("canonical_runs", &attempt.run)?)?;
        if catalog.has("canonical_result_retirements", &run.key) {
            continue;
        }
        if !attempt.terminal
            || !attempt.closed
            || attempt.ingestion_open
            || !matches!(
                attempt.outcome.as_deref(),
                Some("succeeded" | "failed" | "partial" | "cancelled")
            )
        {
            return Err(invalid(
                "open or unknown historical attempt refuses activation",
            ));
        }
        let manifest = attempt
            .closed_manifest
            .as_ref()
            .map(|key| catalog.get("canonical_result_manifests", key))
            .transpose()?
            .map(wire::decode_canonical_result_manifests)
            .transpose()?;
        // Frozen failed/cancelled prefixes are preserved-incomplete; no read or
        // seed admission is inferred from their terminal class or existing bytes.
        if manifest.is_none() && !matches!(attempt.outcome.as_deref(), Some("failed" | "cancelled"))
        {
            return Err(invalid(
                "successful/partial historical attempt lacks complete manifest",
            ));
        }
        let descriptors = manifest
            .as_ref()
            .map(decode_result_descriptors)
            .transpose()?;
        if let Some(manifest) = &manifest
            && (manifest.key != attempt.key
                || manifest.attempt != attempt.key
                || manifest.generation != attempt.generation
                || descriptors.as_ref().is_none_or(|items| {
                    items.len() != catalog.owned("canonical_result_sets", &attempt.key).len()
                }))
        {
            return Err(invalid(
                "historical manifest attempt/set population differs",
            ));
        }
        for set_key in catalog.owned("canonical_result_sets", &attempt.key) {
            let set =
                wire::decode_canonical_result_sets(catalog.get("canonical_result_sets", set_key)?)?;
            if set.interpretation != run.interpretation
                || set.key != result_set_key(&attempt.key, &set.name)
            {
                return Err(invalid("historical result-set meaning differs"));
            }
            let mut digest = FramedHasher::new(Frame::CanonicalPayloadV1);
            digest.str("pse.execution.batch-selection.v1");
            let mut rows = 0_u64;
            let mut count = 0_u64;
            // Exact ordinal keys are derived from the retained set, so neither
            // an all-payload vector nor a sorted trajectory is materialized.
            if catalog.owned("canonical_result_batches", &set.key).len() as u64 != set.next_ordinal
            {
                return Err(invalid("historical batch population differs"));
            }
            for ordinal in 0..set.next_ordinal {
                let batch = wire::decode_canonical_result_batches(catalog.get(
                    "canonical_result_batches",
                    &result_batch_key(&attempt.key, &set.key, ordinal),
                )?)?;
                validate_result_batch(&batch)?;
                if batch.attempt != attempt.key
                    || batch.ordinal != ordinal
                    || batch.result_set != set.key
                {
                    return Err(invalid("historical original batch coordinates differ"));
                }
                rows = rows
                    .checked_add(batch.row_count)
                    .ok_or(CanonicalError::PayloadLimit)?;
                count += 1;
                digest
                    .str(&batch.key)
                    .str(&batch.digest)
                    .u64(batch.row_count);
                if catalog.has("canonical_result_blocks", &batch.key) {
                    let block = wire::decode_canonical_result_blocks(
                        catalog.get("canonical_result_blocks", &batch.key)?,
                    )?;
                    let mut current = block.clone();
                    current.interpretation = wire::INTERPRETATION.into();
                    crate::canonical_results::validate_result_block(&current, &batch)?;
                    if block.interpretation != run.interpretation {
                        return Err(invalid("historical block interpretation differs"));
                    }
                    if batch.row_count > 32 * 1024 {
                        return Err(CanonicalError::PayloadLimit);
                    }
                    let projection = rebuilder.rebuild(&set.name, &batch, &block)?;
                    let originals = catalog
                        .owned("canonical_result_block_outputs", &batch.key)
                        .iter()
                        .map(|key| {
                            wire::decode_canonical_result_block_outputs(
                                catalog.get("canonical_result_block_outputs", key)?,
                            )
                            .map_err(CanonicalError::from)
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    if originals.len() != projection.outputs.len()
                        || projection.outputs.iter().any(|expected| {
                            !originals.iter().any(|original| {
                                original.batch == expected.batch
                                    && original.result_set == expected.result_set
                                    && original.output == expected.output
                                    && original.partition == expected.partition
                                    && original.start == expected.start
                                    && original.end == expected.end
                                    && original.coordinate_min == expected.coordinate_min
                                    && original.coordinate_max == expected.coordinate_max
                                    && original.interpretation == run.interpretation
                            })
                        })
                    {
                        return Err(invalid(
                            "historical dense selectors differ from original scientific columns",
                        ));
                    }
                    if manifest.is_some() && run.interpretation == "pse.substrate.v3" {
                        for locator in projection.rows {
                            crate::canonical_results::validate_result_row(&locator)?;
                            if locator.batch != batch.key
                                || locator.attempt != attempt.key
                                || locator.result_set != set.key
                                || locator.row < block.start
                                || locator.row >= block.end
                            {
                                return Err(invalid(
                                    "historical rebuilder returned foreign typed row",
                                ));
                            }
                            output.push(SavedRow {
                                table: "canonical_result_rows".into(),
                                key: locator.key.clone(),
                                row: wire::encode_canonical_result_rows(&locator)?,
                            })?;
                        }
                    }
                } else if !set.name.starts_with("__") {
                    return Err(invalid(
                        "historical scientific payload lacks its original IPC block",
                    ));
                }
            }
            if rows != set.row_count {
                return Err(invalid("historical stored row coverage differs"));
            }
            if let Some(descriptors) = &descriptors {
                let descriptor = descriptors
                    .iter()
                    .find(|item| item.key == set.key)
                    .ok_or_else(|| invalid("historical manifest omitted set"))?;
                if descriptor.name != set.name
                    || descriptor.batch_count != count
                    || descriptor.row_count != rows
                    || descriptor.batches_digest != digest.finish_hash().to_hex()
                {
                    return Err(invalid("historical manifest frozen coverage differs"));
                }
            }
        }
        if let Some(manifest) = manifest.filter(|_| run.interpretation == "pse.substrate.v3") {
            let admission = Admission {
                key: attempt.key.clone(),
                attempt: attempt.key.clone(),
                run: run.key,
                revision: run.revision,
                sequence: run.source_sequence,
                generation: attempt.generation,
                manifest: manifest.key,
                manifest_digest: manifest.digest,
                outcome: attempt
                    .outcome
                    .ok_or_else(|| invalid("historical terminal class absent"))?,
                interpretation: run.interpretation,
                issuing_store: binding.issuing_store.clone(),
                migration: binding.migration.clone(),
                snapshot: binding.snapshot.clone(),
                source_digest: archive.digest.clone(),
            };
            output.push(SavedRow {
                table: "canonical_historical_results".into(),
                key: admission.key.clone(),
                row: wire::encode_canonical_historical_results(&admission)?,
            })?;
        }
    }
    let derived = output.finish()?;
    freeze(
        &root.join("native-history-derived.json"),
        &serde_json::to_vec(&derived).map_err(io)?,
    )?;
    Ok(Some(derived))
}

pub(super) async fn apply(
    client: &mut OfflineClient,
    root: &Path,
    archive: Option<&Archive>,
) -> Result<(), CanonicalError> {
    let Some(archive) = archive else {
        return Ok(());
    };
    if archive_digest(&archive.pages) != archive.digest {
        return Err(invalid("derived historical archive hash chain differs"));
    }
    let directory = directory(root)?;
    for page in &archive.pages {
        let rows = read_page(&directory, page)?;
        for group in rows.chunks(128) {
            let values = group
                .iter()
                .map(|row| {
                    Value::Object(Object::from(BTreeMap::from([
                        ("table", Value::String(row.table.clone())),
                        ("key", Value::String(row.key.clone())),
                        ("row", Value::Object(row.row.clone())),
                    ])))
                })
                .collect::<Vec<_>>();
            let bindings = Object::from(BTreeMap::from([("rows", Value::Array(values.into()))]));
            client.query("BEGIN; FOR $item IN $rows { LET $saved=SELECT * OMIT id FROM ONLY type::record($item.table,$item.key); IF $saved=NONE { CREATE type::record($item.table,$item.key) CONTENT $item.row RETURN NONE; } ELSE IF $saved!=$item.row { THROW 'offline immutable collision'; }; }; COMMIT;",bindings.clone()).await?;
            // Independent read-back uses a second RPC and compares every exact
            // staged object; no transport ACK substitutes for stored equality.
            client.query("FOR $item IN $rows { LET $saved=SELECT * OMIT id FROM ONLY type::record($item.table,$item.key); IF $saved!=$item.row { THROW 'offline created row differs'; }; };",bindings).await?;
        }
    }
    Ok(())
}
fn expected_key(
    catalog: &mut Catalog,
    extent: &mut usize,
    table: &str,
    key: &str,
) -> Result<(), CanonicalError> {
    if !catalog.has(table, key) {
        let charge = table
            .len()
            .checked_add(key.len())
            .and_then(|bytes| bytes.checked_add(512))
            .ok_or(CanonicalError::PayloadLimit)?;
        *extent = extent
            .checked_add(charge)
            .filter(|bytes| *bytes <= INDEX_BYTES)
            .ok_or(CanonicalError::PayloadLimit)?;
        catalog.keys.insert((table.into(), key.into()), usize::MAX);
    }
    Ok(())
}
#[allow(
    clippy::too_many_arguments,
    reason = "history comparison requires the exact catalog, extent, table, key and completeness boundary"
)]
async fn expected_derived(
    client: &mut OfflineClient,
    catalog: &mut Catalog,
    extent: &mut usize,
    tables: &std::collections::BTreeSet<String>,
    table: &str,
    key: &str,
    row: Object,
    complete: bool,
) -> Result<(), CanonicalError> {
    expected_key(catalog, extent, table, key)?;
    if !tables.contains(table) {
        if !complete && CURRENT_TABLES.contains(&table) {
            return Ok(());
        }
        return Err(invalid(format!(
            "expected historical table missing: {table}"
        )));
    }
    match client.record(table, key).await? {
        Some(mut saved) => {
            saved.remove("id");
            if saved != row {
                return Err(invalid(format!(
                    "expected derived historical owner changed: {table}/{key}"
                )));
            }
        }
        None if complete => {
            return Err(invalid(format!(
                "expected derived historical owner missing: {table}/{key}"
            )));
        }
        None => {}
    }
    Ok(())
}
pub(super) async fn verify(
    client: &mut OfflineClient,
    root: &Path,
    archive: Option<&Archive>,
    backfill: &PreparationBackfill,
    historical: Option<&Archive>,
    complete: bool,
) -> Result<(), CanonicalError> {
    let Some(archive) = archive else {
        return Ok(());
    };
    if archive_digest(&archive.pages) != archive.digest {
        return Err(invalid("original historical archive hash chain differs"));
    }
    let mut response = client.query("INFO FOR DB;", Object::new()).await?;
    let info: Value = response.take(0)?;
    let Value::Object(info) = info else {
        return Err(invalid("historical table inventory unavailable"));
    };
    let Some(Value::Object(tables)) = info.get("tables") else {
        return Err(invalid("historical table inventory unavailable"));
    };
    let tables = tables
        .keys()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    let directory = directory(root)?;
    for page in &archive.pages {
        for row in read_page(&directory, page)? {
            if client.record(&row.table, &row.key).await?.as_ref() != Some(&row.row) {
                return Err(invalid(format!(
                    "original historical owner changed: {}/{}",
                    row.table, row.key
                )));
            }
        }
    }
    // One compact inventory covers original rows and the exact frozen additions.
    // Source tables also receive derived stages/transport; treating them as
    // unrestricted would admit a new open stage on interrupted-operation resume.
    let mut catalog = Catalog::open(root, archive)?;
    catalog.owners.clear();
    let mut extent = catalog
        .pages
        .len()
        .checked_mul(1024)
        .ok_or(CanonicalError::PayloadLimit)?;
    for (table, key) in catalog.keys.keys() {
        extent = extent
            .checked_add(table.capacity())
            .and_then(|bytes| bytes.checked_add(key.capacity()))
            .and_then(|bytes| bytes.checked_add(512))
            .filter(|bytes| *bytes <= INDEX_BYTES)
            .ok_or(CanonicalError::PayloadLimit)?;
    }
    macro_rules! expected {
        ($table:literal,$rows:expr,$encode:path) => {
            for row in $rows {
                expected_derived(
                    client,
                    &mut catalog,
                    &mut extent,
                    &tables,
                    $table,
                    &row.key,
                    $encode(row)?,
                    complete,
                )
                .await?;
            }
        };
    }
    expected!(
        "canonical_body_contents",
        &backfill.contents,
        wire::encode_canonical_body_contents
    );
    expected!(
        "canonical_selection_receipts",
        &backfill.receipts,
        wire::encode_canonical_selection_receipts
    );
    expected!(
        "canonical_product_associations",
        &backfill.associations,
        wire::encode_canonical_product_associations
    );
    expected!(
        "canonical_product_descriptors",
        &backfill.descriptors,
        wire::encode_canonical_product_descriptors
    );
    expected!(
        "canonical_stages",
        &backfill.stages,
        wire::encode_canonical_stages
    );
    expected!(
        "canonical_staged_edits",
        &backfill.staged_edits,
        wire::encode_canonical_staged_edits
    );
    expected!(
        "canonical_version_manifests",
        &backfill.manifests,
        wire::encode_canonical_version_manifests
    );
    expected!(
        "canonical_version_receipts",
        &backfill.version_receipts,
        wire::encode_canonical_version_receipts
    );
    expected!(
        "canonical_payload_blocks",
        &backfill.blocks,
        wire::encode_canonical_payload_blocks
    );
    if let Some(historical) = historical {
        if archive_digest(&historical.pages) != historical.digest {
            return Err(invalid("derived historical archive hash chain differs"));
        }
        for page in &historical.pages {
            let rows = read_page(&directory, page)?;
            for group in rows.chunks(128) {
                let mut values = Vec::with_capacity(group.len());
                for row in group {
                    expected_key(&mut catalog, &mut extent, &row.table, &row.key)?;
                    if !tables.contains(&row.table) {
                        if !complete && CURRENT_TABLES.contains(&row.table.as_str()) {
                            continue;
                        }
                        return Err(invalid(format!(
                            "expected historical table missing: {}",
                            row.table
                        )));
                    }
                    values.push(Value::Object(Object::from(BTreeMap::from([
                        ("table", Value::String(row.table.clone())),
                        ("key", Value::String(row.key.clone())),
                        ("row", Value::Object(row.row.clone())),
                    ]))));
                }
                if values.is_empty() {
                    continue;
                }
                client.query("FOR $item IN $rows { LET $saved=SELECT * OMIT id FROM ONLY type::record($item.table,$item.key); IF $saved=NONE { IF $complete { THROW 'expected derived historical owner missing'; }; } ELSE IF $saved!=$item.row { THROW 'expected derived historical owner changed'; }; };",Object::from(BTreeMap::from([
                    ("rows",Value::Array(values.into())),("complete",Value::Bool(complete))
                ]))).await?;
            }
        }
    }
    for table in TABLES.iter().chain(SOURCE_TABLES).chain(CURRENT_TABLES) {
        if !tables.contains(*table) {
            if !complete && CURRENT_TABLES.contains(table) {
                continue;
            }
            return Err(invalid(format!(
                "expected historical table missing: {table}"
            )));
        }
        let mut after = String::new();
        loop {
            let mut response = client
                .query(
                    "SELECT key FROM type::table($table) WHERE key>$after ORDER BY key LIMIT 64;",
                    Object::from(BTreeMap::from([
                        ("table", Value::String((*table).into())),
                        ("after", Value::String(after.clone())),
                    ])),
                )
                .await?;
            let page: Vec<Object> = response.take(0)?;
            if page.is_empty() {
                break;
            }
            for mut row in page {
                let key = crate::canonical_codec::decode_string(crate::canonical_codec::required(
                    &mut row, "key",
                )?)?;
                if key <= after || !catalog.has(table, &key) {
                    return Err(invalid(format!(
                        "original historical population changed: {table}/{key}"
                    )));
                }
                after = key;
            }
        }
    }
    Ok(())
}

fn validate_sources(catalog: &Catalog) -> Result<(), CanonicalError> {
    for key in catalog.table("canonical_version_manifests") {
        let manifest = wire::decode_canonical_version_manifests(
            catalog.get("canonical_version_manifests", key)?,
        )?;
        if !manifest.closed
            || !crate::canonical::historical_product_interpretation_supported(
                &manifest.interpretation,
            )
        {
            return Err(invalid("open or unknown historical source transport"));
        }
        let mut digest = FramedHasher::new(Frame::CanonicalPayloadV1);
        digest
            .part_writer(manifest.payload_len, &|| Ok(()), |writer| {
                for ordinal in 0..manifest.block_count {
                    let block = wire::decode_canonical_payload_blocks(
                        catalog
                            .get(
                                "canonical_payload_blocks",
                                &format!("{}:{ordinal}", manifest.key),
                            )
                            .map_err(std::io::Error::other)?,
                    )
                    .map_err(std::io::Error::other)?;
                    if block.version != manifest.key
                        || block.ordinal != ordinal
                        || derive_hash(Frame::CanonicalPayloadV1, &[block.payload.as_slice()])
                            .to_hex()
                            != block.digest
                    {
                        return Err(std::io::Error::other("historical source block differs"));
                    }
                    writer.write_all(block.payload.as_slice())?;
                }
                Ok(())
            })
            .map_err(io)?;
        if digest.finish_hash().to_hex() != manifest.payload_digest {
            return Err(invalid("historical source transport digest differs"));
        }
    }
    for key in catalog.table("canonical_stages") {
        let stage = wire::decode_canonical_stages(catalog.get("canonical_stages", key)?)?;
        if !stage.closed && !stage.abandoned {
            return Err(invalid("open historical source stage refuses activation"));
        }
    }
    Ok(())
}
fn validate_owners(catalog: &Catalog) -> Result<(), CanonicalError> {
    for key in catalog.table("canonical_result_retirements") {
        let retirement = wire::decode_canonical_result_retirements(
            catalog.get("canonical_result_retirements", key)?,
        )?;
        if retirement.key != retirement.run
            || !retirement.complete
            || !catalog.has("canonical_runs", &retirement.run)
            || !catalog
                .owned("canonical_result_seeds", &retirement.run)
                .is_empty()
            || !catalog.owned("canonical_roots", &retirement.run).is_empty()
        {
            return Err(invalid("incomplete or conflicting historical retirement"));
        }
        for attempt in catalog.owned("canonical_attempts", &retirement.run) {
            let attempt =
                wire::decode_canonical_attempts(catalog.get("canonical_attempts", attempt)?)?;
            if !attempt.terminal
                || !attempt.closed
                || attempt.ingestion_open
                || !matches!(
                    attempt.outcome.as_deref(),
                    Some("succeeded" | "failed" | "partial" | "cancelled")
                )
                || attempt.generation > retirement.after_generation
                || attempt.completion.is_some()
                || !catalog
                    .owned("canonical_result_sets", &attempt.key)
                    .is_empty()
                || catalog.has("canonical_result_manifests", &attempt.key)
            {
                return Err(invalid(
                    "completed retirement retains mutable or scientific payload",
                ));
            }
        }
        for operation in catalog.owned("canonical_execution_operations", &retirement.run) {
            let operation = wire::decode_canonical_execution_operations(
                catalog.get("canonical_execution_operations", operation)?,
            )?;
            if operation.kind == "append"
                || (operation.kind == "seal" && !operation.result.as_slice().is_empty())
            {
                return Err(invalid(
                    "completed retirement retained scientific effect payload",
                ));
            }
        }
    }
    for key in catalog.table("canonical_study_retirements") {
        let retirement = wire::decode_canonical_study_retirements(
            catalog.get("canonical_study_retirements", key)?,
        )?;
        let study =
            wire::decode_canonical_studies(catalog.get("canonical_studies", &retirement.study)?)?;
        if retirement.key != study.key || !study.terminal {
            return Err(invalid(
                "historical study retirement differs from terminal owner",
            ));
        }
    }
    for key in catalog.table("canonical_runs") {
        let run = wire::decode_canonical_runs(catalog.get("canonical_runs", key)?)?;
        if !crate::canonical::historical_product_interpretation_supported(&run.interpretation) {
            return Err(invalid("unknown historical run interpretation"));
        }
        if catalog.has("canonical_result_retirements", &run.key) {
            continue;
        }
        let revision =
            wire::decode_canonical_revisions(catalog.get("canonical_revisions", &run.revision)?)?;
        let root = wire::decode_canonical_roots(
            catalog.get("canonical_roots", &format!("run:{}", run.key))?,
        )?;
        if revision.problem != run.problem
            || revision.sequence != run.source_sequence
            || revision.interpretation != run.interpretation
            || root.owner_kind != "run"
            || root.owner != run.key
            || root.problem != run.problem
            || root.revision != run.revision
            || root.sequence != run.source_sequence
        {
            return Err(invalid("historical execution source/root differs"));
        }
        let current = run
            .current_attempt
            .as_deref()
            .ok_or_else(|| invalid("unclaimed historical run refuses activation"))?;
        let attempt = wire::decode_canonical_attempts(catalog.get("canonical_attempts", current)?)?;
        if attempt.run != run.key
            || !attempt.terminal
            || !attempt.closed
            || attempt.ingestion_open
            || attempt.generation != run.current_generation
            || run.terminal_attempt.as_deref() != Some(current)
            || run.terminal_class != attempt.outcome
        {
            return Err(invalid(
                "live or conflicting historical run refuses activation",
            ));
        }
    }
    for (table, parent, field) in [
        ("canonical_result_sets", "canonical_attempts", "attempt"),
        (
            "canonical_result_batches",
            "canonical_result_sets",
            "result_set",
        ),
        (
            "canonical_result_blocks",
            "canonical_result_batches",
            "batch",
        ),
        (
            "canonical_result_cells",
            "canonical_result_batches",
            "batch",
        ),
        (
            "canonical_result_block_outputs",
            "canonical_result_batches",
            "batch",
        ),
        ("canonical_execution_operations", "canonical_runs", "run"),
        ("canonical_study_points", "canonical_studies", "study"),
        ("canonical_analysis_nodes", "canonical_analyses", "analysis"),
        ("canonical_analysis_edges", "canonical_analyses", "analysis"),
        (
            "canonical_analysis_inputs",
            "canonical_analyses",
            "analysis",
        ),
    ] {
        for key in catalog.table(table) {
            let row = catalog.get(table, key)?;
            if !catalog.has(parent, &text(&row, field)?) {
                return Err(invalid(format!("{table} historical owner absent")));
            }
        }
    }
    for key in catalog.table("canonical_run_sources") {
        let source =
            wire::decode_canonical_run_sources(catalog.get("canonical_run_sources", key)?)?;
        catalog.get("canonical_runs", &source.run)?;
        if catalog.has("canonical_result_retirements", &source.run) {
            continue;
        }
        let revision = wire::decode_canonical_revisions(
            catalog.get("canonical_revisions", &source.revision)?,
        )?;
        let mut found = false;
        for key in catalog.owned("canonical_roots", &source.run) {
            let root = wire::decode_canonical_roots(catalog.get("canonical_roots", key)?)?;
            found |= root.owner_kind == "run"
                && root.revision == revision.key
                && root.problem == revision.problem
                && root.sequence == revision.sequence;
        }
        if !found {
            return Err(invalid("historical selected source root absent"));
        }
    }
    for key in catalog.table("canonical_studies") {
        let study = wire::decode_canonical_studies(catalog.get("canonical_studies", key)?)?;
        if !crate::canonical::historical_product_interpretation_supported(&study.interpretation) {
            return Err(invalid("unknown historical study interpretation"));
        }
        let run = wire::decode_canonical_runs(catalog.get("canonical_runs", &study.run)?)?;
        if study.interpretation != run.interpretation
            || study.problem != run.problem
            || study.revision != run.revision
            || !study.active
            || !study.terminal
            || study.next_ordinal != study.point_count
            || catalog.owned("canonical_study_points", key).len() as u64 != study.point_count
        {
            return Err(invalid(
                "mutable or incomplete historical study owner refuses activation",
            ));
        }
        let mut ordinals = std::collections::BTreeSet::new();
        for point in catalog.owned("canonical_study_points", key) {
            let point =
                wire::decode_canonical_study_points(catalog.get("canonical_study_points", point)?)?;
            if !point.settled
                || point.ordinal >= study.point_count
                || !ordinals.insert(point.ordinal)
                || !catalog.has("canonical_runs", &point.run)
            {
                return Err(invalid("historical study settled membership differs"));
            }
            if let Some(attempt) = &point.attempt {
                let attempt =
                    wire::decode_canonical_attempts(catalog.get("canonical_attempts", attempt)?)?;
                if attempt.run != point.run {
                    return Err(invalid("historical study point attempt owner differs"));
                }
            }
        }
    }
    for key in catalog.table("canonical_study_dependencies") {
        let edge = wire::decode_canonical_study_dependencies(
            catalog.get("canonical_study_dependencies", key)?,
        )?;
        if !catalog.has("canonical_study_points", &edge.predecessor)
            || !catalog.has("canonical_study_points", &edge.dependent)
        {
            return Err(invalid("historical study dependency endpoint absent"));
        }
    }
    for key in catalog.table("canonical_analyses") {
        let analysis = wire::decode_canonical_analyses(catalog.get("canonical_analyses", key)?)?;
        if !crate::canonical::historical_product_interpretation_supported(&analysis.interpretation)
        {
            return Err(invalid("unknown historical analysis interpretation"));
        }
        if !analysis.active
            || analysis.retiring
            || catalog.owned("canonical_analysis_nodes", key).len() as u64 != analysis.node_count
            || catalog.owned("canonical_analysis_edges", key).len() as u64 != analysis.edge_count
        {
            return Err(invalid(
                "mutable or incomplete historical analysis refuses activation",
            ));
        }
        let revision = wire::decode_canonical_revisions(
            catalog.get("canonical_revisions", &analysis.revision)?,
        )?;
        if analysis.primary_problem != revision.problem
            || analysis.interpretation != revision.interpretation
        {
            return Err(invalid("historical analysis source owner differs"));
        }
        let mut found = false;
        for root in catalog.owned("canonical_roots", key) {
            let root = wire::decode_canonical_roots(catalog.get("canonical_roots", root)?)?;
            found |= root.owner_kind == "analysis"
                && root.revision == revision.key
                && root.problem == revision.problem
                && root.sequence == revision.sequence;
        }
        if !found {
            return Err(invalid("historical analysis source root absent"));
        }
    }
    for key in catalog.table("canonical_analysis_edges") {
        let edge =
            wire::decode_canonical_analysis_edges(catalog.get("canonical_analysis_edges", key)?)?;
        for endpoint in [&edge.source, &edge.target] {
            let node = wire::decode_canonical_analysis_nodes(
                catalog.get("canonical_analysis_nodes", endpoint)?,
            )?;
            if node.analysis != edge.analysis {
                return Err(invalid("historical analysis edge endpoint differs"));
            }
        }
    }
    for key in catalog.table("canonical_analysis_inputs") {
        let input =
            wire::decode_canonical_analysis_inputs(catalog.get("canonical_analysis_inputs", key)?)?;
        wire::decode_canonical_artifacts(catalog.get("canonical_artifacts", &input.artifact)?)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn check(rows: Vec<SavedRow>) -> Result<(), CanonicalError> {
        let root = std::env::temp_dir().join(format!(
            "pse-history-owner-control-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir(&root).map_err(io)?;
        let result = (|| {
            let mut writer = Writer::new(directory(&root)?, "source");
            for row in rows {
                writer.push(row)?;
            }
            let archive = writer.finish()?;
            let catalog = Catalog::open(&root, &archive)?;
            validate_owners(&catalog)
        })();
        fs::remove_dir_all(root).map_err(io)?;
        result
    }
    #[test]
    fn historical_study_unknown_interpretation_refuses_before_missing_owner_can_be_assumed() {
        let study = pse_model::generated::runtime::canonical_studies::Row {
            key: "study".into(),
            run: "run".into(),
            problem: "problem".into(),
            revision: "revision".into(),
            metadata: Vec::new().into(),
            interpretation: "unknown".into(),
            point_count: 0,
            next_ordinal: 0,
            active: true,
            cancelled: false,
            generation: 1,
            terminal: true,
        };
        let error = check(vec![SavedRow {
            table: "canonical_studies".into(),
            key: study.key.clone(),
            row: wire::encode_canonical_studies(&study).unwrap(),
        }])
        .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("unknown historical study interpretation")
        );
    }
    #[test]
    fn historical_analysis_unknown_interpretation_and_wrong_primary_problem_refuse() {
        let mut analysis = pse_model::generated::runtime::canonical_analyses::Row {
            key: "analysis".into(),
            revision: "revision".into(),
            method: "method".into(),
            configuration: Vec::new().into(),
            input_digest: "digest".into(),
            primary_problem: "foreign-problem".into(),
            primary_authority: "authority".into(),
            creation_nonce: "nonce".into(),
            creation_request_digest: "request".into(),
            creation_expires_at: 0,
            interpretation: "unknown".into(),
            node_count: 0,
            edge_count: 0,
            active: true,
            retiring: false,
        };
        let selected =
            |analysis: &pse_model::generated::runtime::canonical_analyses::Row| SavedRow {
                table: "canonical_analyses".into(),
                key: analysis.key.clone(),
                row: wire::encode_canonical_analyses(analysis).unwrap(),
            };
        assert!(
            check(vec![selected(&analysis)])
                .unwrap_err()
                .to_string()
                .contains("unknown historical analysis interpretation")
        );
        analysis.interpretation = "pse.substrate.v3".into();
        let revision = pse_model::generated::runtime::canonical_revisions::Row {
            key: "revision".into(),
            problem: "actual-problem".into(),
            sequence: 1,
            parent: None,
            operation: "original".into(),
            request: Vec::new().into(),
            interpretation: "pse.substrate.v3".into(),
        };
        let error = check(vec![
            selected(&analysis),
            SavedRow {
                table: "canonical_revisions".into(),
                key: revision.key.clone(),
                row: wire::encode_canonical_revisions(&revision).unwrap(),
            },
        ])
        .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("historical analysis source owner differs")
        );
    }
}
