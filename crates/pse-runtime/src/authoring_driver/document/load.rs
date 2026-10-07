// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Package loading into generated row contracts without publishing a candidate.

use super::{SpanIndex, allocation as memory, hydrate, read, value};
use crate::authoring_driver::{DriverError, ParseBudget, SourceSpan};
use pse_ids::{ContentHash, SemanticId};
use pse_relations::{columnar::FieldCheckedBatch, generated::authored};
use pse_schema::Registry;
use pse_schema::model::{DocumentKind, DocumentSpec};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

/// Generated Arrow values with local field checks retained by their private owner.
pub type Batches = BTreeMap<SemanticId, FieldCheckedBatch>;

/// Immutable parser rows carry their source allocation even when shared separately.
#[derive(Debug)]
pub struct ModelingRows {
    rows: Vec<authored::modeling_declarations::Row>,
    lease: Option<Arc<pse_columnar::AllocationLease>>,
    admission: std::sync::Mutex<Option<pse_relations::columnar::LocalFieldAdmission>>,
}
impl Clone for ModelingRows {
    fn clone(&self) -> Self {
        // A private mutable clone can be edited; it cannot inherit assurance for
        // the original immutable rows. Arc clones preserve the actual owner.
        Self {
            rows: self.rows.clone(),
            lease: self.lease.clone(),
            admission: Default::default(),
        }
    }
}
impl std::ops::Deref for ModelingRows {
    type Target = [authored::modeling_declarations::Row];
    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}
impl pse_model::HeapUsage for ModelingRows {
    fn heap_bytes(&self) -> usize {
        pse_model::HeapUsage::heap_bytes(&self.rows)
    }
}

/// Immutable source payload and its own admitted allocation lifetime.
#[derive(Debug)]
pub struct SourcePayload<T> {
    value: T,
    lease: Option<Arc<pse_columnar::AllocationLease>>,
}
impl<T> SourcePayload<T> {
    fn new(value: T) -> Self {
        Self { value, lease: None }
    }
}
impl<T: Clone> SourcePayload<T> {
    fn attach(target: &mut Arc<Self>, lease: Arc<pse_columnar::AllocationLease>) {
        if let Some(payload) = Arc::get_mut(target) {
            payload.lease = Some(lease);
        } else {
            // An imported unowned parser snapshot has its own private metadata;
            // the completed preflight covers this fresh immutable payload copy.
            *target = Arc::new(Self {
                value: target.value.clone(),
                lease: Some(lease),
            });
        }
    }
}
impl<T> std::ops::Deref for SourcePayload<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}

#[derive(Debug)]
struct SourceBytes {
    bytes: bytes::Bytes,
    _lease: Arc<pse_columnar::AllocationLease>,
}
struct SharedText(Arc<SourcePayload<String>>);
impl AsRef<[u8]> for SharedText {
    fn as_ref(&self) -> &[u8] {
        self.0.value.as_bytes()
    }
}

impl AsRef<[u8]> for SourceBytes {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

/// A package document's exact original content: bytes with the kind its path declares
/// (ADR-0125). Text documents are one kind among them.
#[derive(Clone, Debug)]
pub enum Content {
    /// A text document's original UTF-8 source; never reconstructed for source locations.
    Text(Arc<SourcePayload<String>>),
    /// Exact modeling source and its immutable parser-produced semantic rows.
    Modeling {
        /// The exact original UTF-8 source.
        text: Arc<SourcePayload<String>>,
        /// Rows admitted by the generated native field contract.
        rows: Arc<ModelingRows>,
    },
    /// A data document's original bytes and the row set decoded from them by Arrow type.
    Data {
        /// The exact original bytes.
        bytes: bytes::Bytes,
        /// The decoded document.
        document: Arc<pse_modeling::document::DataDocument>,
        /// The independent original byte/decoded row allocation owner.
        owner: Option<Arc<pse_columnar::AllocationLease>>,
    },
}

/// One immutable original document plus its parser-derived locations.
#[derive(Clone, Debug)]
pub struct Document {
    /// Stable package-relative document identity.
    pub id: SemanticId,
    /// Original package-relative path.
    pub path: String,
    /// The exact original content.
    pub content: Content,
    /// Integrity identity of the exact original bytes.
    pub content_hash: ContentHash,
    /// Exact original syntax node locations.
    pub spans: SpanIndex,
    pub(super) declaration: DocumentSpec,
    interpretation: InterpretationContext,
    pub(super) value: value::Value,
    pub(super) syntax: Arc<SourcePayload<value::Value>>,
    /// Columnar projection of this exact parsed document.
    pub batches: Batches,
}

/// The inputs consumed by identity-bearing parsing and resource admission.
#[derive(Clone, Debug, PartialEq, Eq)]
struct InterpretationContext {
    document: SemanticId,
    identity_policy: pse_relations::generated::enums::IdPolicy,
    budget: ParseBudget,
}

/// A parsed package. Cross-package closure belongs to P0/P1 before publication.
#[derive(Clone, Debug)]
pub struct DocumentBundle(Arc<BundleData>);

/// Immutable package data produced only by the declared source loader.
#[derive(Clone, Debug)]
pub struct BundleData {
    /// Generated package header row.
    pub package: authored::packages::Row,
    /// Original documents in relative-path order.
    pub documents: Vec<Document>,
    /// Generated columns, including package, document and entity context.
    pub batches: Batches,
    lease: Option<Arc<pse_columnar::AllocationLease>>,
}

impl Document {
    /// Retain original immutable source bytes with their existing allocation owner.
    pub(super) fn shared_bytes(&self) -> bytes::Bytes {
        match &self.content {
            Content::Text(text) | Content::Modeling { text, .. } => {
                bytes::Bytes::from_owner(SharedText(Arc::clone(text)))
            }
            Content::Data { bytes, .. } => bytes.clone(),
        }
    }

    /// The source text of a text document; `None` for a data document.
    pub fn text(&self) -> Option<&str> {
        match &self.content {
            Content::Text(text) | Content::Modeling { text, .. } => Some(text),
            Content::Data { .. } => None,
        }
    }
    /// The exact original bytes.
    pub fn bytes(&self) -> &[u8] {
        match &self.content {
            Content::Text(text) | Content::Modeling { text, .. } => text.as_bytes(),
            Content::Data { bytes, .. } => bytes,
        }
    }
    /// The decoded rows of a data document (ADR-0125); `None` for a text document.
    pub fn data(&self) -> Option<&Arc<pse_modeling::document::DataDocument>> {
        match &self.content {
            Content::Text(_) | Content::Modeling { .. } => None,
            Content::Data { document, .. } => Some(document),
        }
    }
    /// Parser-produced modeling rows; no columnar round trip is needed by the compiler.
    pub fn modeling_rows(&self) -> Option<&Arc<ModelingRows>> {
        match &self.content {
            Content::Modeling { rows, .. } => Some(rows),
            Content::Text(_) | Content::Data { .. } => None,
        }
    }

    /// Whether immutable parser rows require admission in this actual native owner.
    pub(super) fn modeling_validation_required(
        &self,
        registry: &Registry,
        validation: &pse_relations::validate::ValidationContext,
    ) -> Result<bool, DriverError> {
        let Some(rows) = self.modeling_rows() else {
            return Ok(false);
        };
        let spec = registry
            .relation_by_id(authored::modeling_declarations::RELATION_ID)
            .ok_or_else(|| contract(None, "modeling row declaration absent"))?;
        let admission = rows
            .admission
            .lock()
            .map_err(|_| contract(None, "modeling native owner lock poisoned"))?;
        Ok(!admission
            .as_ref()
            .map(|owner| owner.matches(registry, spec, validation))
            .transpose()?
            .unwrap_or(false))
    }

    /// Materialize generated columns only at an explicit native admission/export boundary.
    /// # Errors
    /// Actual field/native refusal, a foreign declaration or cancellation.
    pub fn modeling_batch(
        &self,
        registry: &Registry,
        validation: &pse_relations::validate::ValidationContext,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<Option<FieldCheckedBatch>, DriverError> {
        let Some(rows) = self.modeling_rows() else {
            return Ok(None);
        };
        cancel.checkpoint()?;
        let mut builder = authored::modeling_declarations::Builder::with_registry(
            registry,
            rows.len(),
            validation,
        )?;
        for row in rows.iter() {
            cancel.checkpoint()?;
            builder.push(row.clone())?;
        }
        let batch = builder.finish()?;
        cancel.checkpoint()?;
        *rows
            .admission
            .lock()
            .map_err(|_| contract(None, "modeling native owner lock poisoned"))? =
            batch.local_admission().cloned();
        Ok(Some(batch))
    }
}

impl Document {
    pub(super) fn fresh_payload_extent(&self) -> Result<usize, DriverError> {
        use pse_model::HeapUsage;
        // Explicit wrapper/owner allowance accompanies each new independently
        // published allocation; it is an accounting policy rather than RSS.
        const OWNER: usize = 256;
        let mut extent = 0;
        match &self.content {
            Content::Text(text) | Content::Modeling { text, .. } => {
                if text.lease.is_none() {
                    extent = memory::add(text.capacity(), OWNER)?;
                }
            }
            Content::Data {
                bytes,
                document,
                owner,
            } => {
                if owner.is_none() {
                    extent = memory::add(bytes.len(), OWNER)?;
                }
                if !document.is_owned() {
                    extent = memory::add(extent, memory::add(document.retained_bytes(), OWNER)?)?;
                }
            }
        }
        if let Content::Modeling { rows, .. } = &self.content
            && rows.lease.is_none()
        {
            extent = memory::add(
                extent,
                memory::add(rows.heap_bytes(), size_of::<ModelingRows>() + OWNER)?,
            )?;
        }
        if self.syntax.lease.is_none() {
            extent = memory::add(
                extent,
                memory::add(memory::value_retained(&self.syntax)?, OWNER)?,
            )?;
        }
        Ok(extent)
    }
    fn attach_payload_lease(&mut self, lease: Arc<pse_columnar::AllocationLease>) {
        match &mut self.content {
            Content::Text(text) | Content::Modeling { text, .. } => {
                if text.lease.is_none() {
                    SourcePayload::attach(text, Arc::clone(&lease));
                }
            }
            Content::Data {
                bytes,
                document,
                owner,
            } => {
                if owner.is_none() {
                    *bytes = bytes::Bytes::from_owner(SourceBytes {
                        bytes: bytes.clone(),
                        _lease: Arc::clone(&lease),
                    });
                    *owner = Some(Arc::clone(&lease));
                }
                if !document.is_owned() {
                    Arc::make_mut(document).attach_owner(lease.clone());
                }
            }
        }
        if let Content::Modeling { rows, .. } = &mut self.content
            && rows.lease.is_none()
        {
            Arc::make_mut(rows).lease = Some(Arc::clone(&lease));
        }
        if self.syntax.lease.is_none() {
            SourcePayload::attach(&mut self.syntax, lease);
        }
    }
}

impl std::ops::Deref for DocumentBundle {
    type Target = BundleData;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DocumentBundle {
    pub(super) fn retain_columns(
        &mut self,
        pool: &Arc<dyn pse_columnar::MemoryPool>,
        cancel: &pse_columnar::CancellationToken,
    ) -> Result<(), DriverError> {
        let data = Arc::make_mut(&mut self.0);
        for document in &mut data.documents {
            for batch in document.batches.values_mut() {
                *batch = batch.retained(pool, cancel)?;
            }
        }
        for batch in data.batches.values_mut() {
            *batch = batch.retained(pool, cancel)?;
        }
        Ok(())
    }
    /// Freeze fresh payloads independently; reused owners remain attached to their
    /// original allocations. Only copied bundle/parser metadata retains the remainder.
    pub(super) fn attach_lease(
        &mut self,
        lease: Arc<pse_columnar::AllocationLease>,
    ) -> Result<Arc<pse_columnar::AllocationLease>, DriverError> {
        let data = Arc::make_mut(&mut self.0);
        let extents = data
            .documents
            .iter()
            .map(Document::fresh_payload_extent)
            .collect::<Result<Vec<_>, _>>()?;
        let total = extents
            .iter()
            .try_fold(0, |sum, extent| memory::add(sum, *extent))?;
        let remainder = lease
            .size()
            .checked_sub(total)
            .ok_or_else(|| contract(None, "source payload partition exceeds admission"))?;
        let mut sizes = Vec::with_capacity(extents.len() + 1);
        sizes.push(remainder);
        sizes.extend(extents);
        let owners = lease.partition(&sizes).map_err(|_| {
            contract(
                None,
                "source admission lease must be unique before publication",
            )
        })?;
        let metadata = Arc::clone(&owners[0]);
        for (document, owner) in data.documents.iter_mut().zip(owners.into_iter().skip(1)) {
            document.attach_payload_lease(owner);
        }
        data.lease = Some(Arc::clone(&metadata));
        Ok(metadata)
    }
}

/// Read a package using declared document paths and a total byte budget.
///
/// # Errors
/// Refuses missing headers, symlinks, malformed/unknown fields, invalid identities,
/// ambiguous names, missing owners and exceeded byte/parser limits.
pub fn load_package(
    root: &Path,
    registry: &Registry,
    budget: ParseBudget,
    validation: &pse_relations::validate::ValidationContext,
) -> Result<DocumentBundle, DriverError> {
    load_package_documents(
        read::package_files(root, registry, &budget)?,
        registry,
        budget,
        validation,
    )
}

/// Load an explicit immutable inventory of package documents by path, each the exact bytes
/// of the kind its path declares (ADR-0125); also used by document editors and fixtures.
///
/// # Errors
/// The same syntax and semantic authoring failures as [`load_package`].
pub fn load_package_documents(
    sources: BTreeMap<String, Vec<u8>>,
    registry: &Registry,
    budget: ParseBudget,
    validation: &pse_relations::validate::ValidationContext,
) -> Result<DocumentBundle, DriverError> {
    load_inventory(sources, registry, budget, None, validation)
}

pub(super) fn load_inventory(
    sources: BTreeMap<String, Vec<u8>>,
    registry: &Registry,
    budget: ParseBudget,
    allocation: Option<&mut super::allocation::Allocation<'_>>,
    validation: &pse_relations::validate::ValidationContext,
) -> Result<DocumentBundle, DriverError> {
    load_reusing(sources, None, registry, budget, allocation, validation)
}

/// The UTF-8 text of a text document.
fn utf8(path: &str, bytes: bytes::Bytes) -> Result<String, DriverError> {
    std::str::from_utf8(&bytes)
        .map(str::to_owned)
        .map_err(|_| contract(None, &format!("text document {path} must be UTF-8")))
}

pub(super) fn load_reusing<T: AsRef<[u8]> + Into<bytes::Bytes>>(
    sources: BTreeMap<String, T>,
    previous: Option<&DocumentBundle>,
    registry: &Registry,
    budget: ParseBudget,
    mut allocation: Option<&mut super::allocation::Allocation<'_>>,
    validation: &pse_relations::validate::ValidationContext,
) -> Result<DocumentBundle, DriverError> {
    let sources = sources
        .into_iter()
        .map(|(path, bytes)| (path, bytes.into()))
        .collect::<BTreeMap<_, bytes::Bytes>>();
    check_size(&sources, budget.max_bytes)?;
    let checksum = package_checksum(&sources);
    let header_text = std::str::from_utf8(
        sources
            .get("package.toml")
            .ok_or_else(|| contract(None, "missing package.toml"))?,
    )
    .map_err(|_| contract(None, "text document package.toml must be UTF-8"))?
    .to_owned();
    if let Some(funds) = allocation.as_deref_mut() {
        reserve_inventory(&sources, &header_text, registry, &budget, funds)?;
    }
    let (package, header_value, header_spans) =
        reuse_header(previous, &header_text, checksum, &budget)?;
    let mut header_parts = Some((header_value, header_spans));
    let mut documents = Vec::with_capacity(sources.len());
    for (path, bytes) in sources {
        let declaration = select(registry, &path)?.clone();
        let id = pse_ids::named_id(package.package_id.as_id(), &path);
        let content_hash = pse_ids::encoding_checksum(&bytes).content_hash();
        let interpretation = InterpretationContext {
            document: id,
            identity_policy: package.id_policy,
            budget,
        };
        let prior = previous.and_then(|bundle| {
            bundle.documents.iter().find(|document| {
                document.path == path
                    && document.bytes() == bytes.as_ref()
                    && document.declaration == declaration
                    && document.interpretation == interpretation
            })
        });
        if declaration.kind == DocumentKind::Data {
            // ADR-0125: bytes decoded by Arrow type; an unchanged document keeps its rows.
            let content = if let Some(prior) = prior {
                prior.content.clone()
            } else {
                let rows = super::data::decode(&path, &bytes, budget, allocation.as_deref_mut())?;
                Content::Data {
                    bytes,
                    document: Arc::new(pse_modeling::document::DataDocument {
                        allocation_owner: None,
                        id,
                        path: path.clone(),
                        content_hash,
                        rows,
                    }),
                    owner: None,
                }
            };
            let value = Arc::new(SourcePayload::new(value::Value::Map(Vec::new())));
            documents.push(Document {
                id,
                path,
                content,
                content_hash,
                spans: SpanIndex::default(),
                declaration,
                interpretation,
                value: value::Value::Map(Vec::new()),
                syntax: value,
                batches: Batches::new(),
            });
            continue;
        }
        let text = if prior.is_some() {
            String::new()
        } else {
            utf8(&path, bytes)?
        };
        let mut modeling_rows = None;
        let (value, spans) = if declaration.kind == DocumentKind::PackageHeader {
            header_parts
                .take()
                .ok_or_else(|| contract(None, "duplicate package header shape"))?
        } else if let Some(prior) = prior {
            if declaration.kind == DocumentKind::Modeling {
                modeling_rows = prior.modeling_rows().cloned();
            }
            ((**prior.syntax).clone(), prior.spans.clone())
        } else if declaration.kind == DocumentKind::Modeling {
            if let Some(funds) = allocation.as_deref_mut() {
                funds.grow(memory::modeling_parser_extent(&text, &budget)?)?;
            }
            let policy = if package.id_policy == pse_relations::generated::enums::IdPolicy::Named {
                pse_authoring::language::IdentityPolicy::Named
            } else {
                pse_authoring::language::IdentityPolicy::Explicit
            };
            let (rows, fields) =
                pse_authoring::language::parse_with_spans(&text, id, policy, budget)?;
            let mut spans = SpanIndex::default();
            for (declaration, locations) in fields {
                for (field, span) in locations {
                    spans.insert(format!("/modeling-fields/{declaration}/{field}"), span);
                }
            }
            for (ordinal, row) in rows.iter().enumerate() {
                spans.insert(
                    format!("/modeling_declarations/{ordinal}"),
                    SourceSpan::new(id, row.source_start as u32, row.source_end as u32),
                );
            }
            modeling_rows = Some(Arc::new(ModelingRows {
                rows,
                lease: None,
                admission: Default::default(),
            }));
            (value::Value::Map(Vec::new()), spans)
        } else {
            let (value, spans) =
                value::parse_yaml_accounted(&text, id, &budget, allocation.as_deref_mut())?;
            (value.value, spans)
        };
        let syntax = prior.map_or_else(
            || Arc::new(SourcePayload::new(value.clone())),
            |prior| Arc::clone(&prior.syntax),
        );
        let text = prior
            .and_then(|prior| match &prior.content {
                Content::Text(text) | Content::Modeling { text, .. } => Some(Arc::clone(text)),
                Content::Data { .. } => None,
            })
            .unwrap_or_else(|| Arc::new(SourcePayload::new(text)));
        documents.push(Document {
            id,
            path,
            content_hash,
            content: match modeling_rows {
                Some(rows) => Content::Modeling { text, rows },
                None => Content::Text(text),
            },
            spans,
            declaration,
            interpretation,
            value,
            syntax,
            batches: Batches::new(),
        });
    }
    if let Some(funds) = allocation.as_deref_mut() {
        funds.grow(memory::hydration_extent(
            &documents,
            &package.name,
            registry,
        )?)?;
    }
    let entities = hydrate::documents(&mut documents, &package, registry)?;
    let mut parts = project_documents(
        &mut documents,
        registry,
        allocation.as_deref_mut(),
        validation,
    )?;
    if let Some(funds) = allocation {
        funds.grow(memory::mul(entities.len() + documents.len(), 1024)?)?;
    }
    append_source_inventory(
        &mut parts,
        entities,
        &documents,
        package.package_id,
        registry,
        validation,
    )?;
    let batches = concatenate_parts(parts, registry)?;
    crate::authoring_driver::p1::admit_references(&batches, registry)?;
    Ok(DocumentBundle(Arc::new(BundleData {
        package,
        documents,
        batches,
        lease: None,
    })))
}

fn concatenate_parts(
    parts: BTreeMap<SemanticId, Vec<FieldCheckedBatch>>,
    registry: &Registry,
) -> Result<Batches, DriverError> {
    parts
        .into_iter()
        .map(|(id, values)| {
            let spec = registry
                .relations()
                .iter()
                .find(|spec| spec.id == id)
                .ok_or_else(|| contract(None, "unregistered source output"))?;
            Ok((id, FieldCheckedBatch::concat(registry, spec, &values)?))
        })
        .collect()
}

pub(super) fn header(
    text: &str,
    checksum: ContentHash,
    budget: &ParseBudget,
) -> Result<(authored::packages::Row, value::Value, SpanIndex), DriverError> {
    let parsed: toml::Spanned<toml::Value> = toml::from_str(text)
        .map_err(|error: toml::de::Error| contract(None, &error.to_string()))?;
    check_header_depth(
        parsed.get_ref(),
        budget.max_depth.min(ParseBudget::DEFAULT_MAX_DEPTH),
    )?;
    let range = parsed.span();
    let mut tree = value::from_toml(parsed.into_inner());
    let row = tree
        .get_mut("package")
        .ok_or_else(|| contract(None, "package.toml requires [package]"))?;
    if row.get("id").is_some() && row.get("package_id").is_some() {
        return Err(contract(None, "id and package_id both supplied"));
    }
    if let Some(id) = row.remove("id") {
        row.set("package_id", id);
    }
    let id = row
        .get("package_id")
        .and_then(value::Value::text)
        .ok_or_else(|| contract(None, "package identity is required"))?;
    let id = crate::authoring_driver::ids::parse_id(id, SourceSpan::head(SemanticId::NIL))?;
    row.set("package_id", value::Value::Text(id.to_string()));
    if row.get("content_hash").is_some() {
        return Err(contract(
            None,
            "package content_hash is computed from the complete input inventory",
        ));
    }
    row.set("content_hash", value::Value::Text(checksum.to_prefixed()));
    if let Some(value::Value::List(dependencies)) = row.get_mut("dependencies") {
        for dependency in dependencies {
            let dependency_id = dependency
                .value
                .get("package_id")
                .and_then(value::Value::text)
                .ok_or_else(|| contract(None, "dependency package_id is required"))?;
            let dependency_id =
                crate::authoring_driver::ids::parse_id(dependency_id, SourceSpan::head(id))?;
            dependency
                .value
                .set("package_id", value::Value::Text(dependency_id.to_string()));
        }
    }
    let package = authored::packages::Row::deserialize(&*row)
        .map_err(|error| contract(None, &error.to_string()))?;
    let document = pse_ids::named_id(id, "package.toml");
    let mut spans = SpanIndex::default();
    spans.insert(
        "/package".to_owned(),
        SourceSpan::new(
            document,
            u32::try_from(range.start).unwrap_or(u32::MAX),
            u32::try_from(range.end).unwrap_or(u32::MAX),
        ),
    );
    Ok((package, tree, spans))
}

pub(super) fn select<'a>(
    registry: &'a Registry,
    path: &str,
) -> Result<&'a DocumentSpec, DriverError> {
    if path.split('/').any(|part| matches!(part, "" | "." | "..")) || path.contains('\\') {
        return Err(contract(None, "invalid package-relative path"));
    }
    let mut matches = registry.documents().iter().filter(|document| {
        document.path_glob == path
            || document
                .path_glob
                .rsplit_once("*.")
                .is_some_and(|(prefix, extension)| {
                    path.strip_prefix(prefix).is_some_and(|name| {
                        Path::new(name).extension() == Some(std::ffi::OsStr::new(extension))
                            && !name.contains('/')
                    })
                })
    });
    match (matches.next(), matches.next()) {
        (Some(document), None) => Ok(document),
        _ => Err(contract(
            None,
            &format!("path `{path}` requires exactly one declared document shape"),
        )),
    }
}

fn check_size<T: AsRef<[u8]>>(
    sources: &BTreeMap<String, T>,
    allowed: u64,
) -> Result<(), DriverError> {
    let needed = sources
        .values()
        .try_fold(0_u64, |sum, bytes| {
            sum.checked_add(u64::try_from(bytes.as_ref().len()).unwrap_or(u64::MAX))
        })
        .unwrap_or(u64::MAX);
    if needed > allowed {
        Err(DriverError::Authoring(
            pse_authoring::AuthoringError::Budget {
                limit: "package bytes",
                allowed,
                needed,
            },
        ))
    } else {
        Ok(())
    }
}

/// The package content hash (blueprint §6.1) over sorted path/content pairs: the identity
/// of an authored package, and of a source bundle stored for job execution (ADR-0112). Every
/// document enters with its exact bytes, a data document's included (ADR-0125); a text
/// document's bytes are its UTF-8 source.
pub fn package_checksum<T: AsRef<[u8]>>(sources: &BTreeMap<String, T>) -> ContentHash {
    // An integrity encoding: sorted path/content pairs, each prefixed by its byte length.
    let mut hasher = pse_ids::EncodingHasher::new();
    for (path, content) in sources {
        for part in [path.as_bytes(), content.as_ref()] {
            hasher.update(&u64::try_from(part.len()).unwrap_or(u64::MAX).to_le_bytes());
            hasher.update(part);
        }
    }
    hasher.finish().content_hash()
}

pub(super) fn contract(at: Option<SourceSpan>, reason: &str) -> DriverError {
    DriverError::Authoring(pse_authoring::AuthoringError::Contract {
        at,
        reason: reason.to_owned(),
    })
}

fn check_header_depth(value: &toml::Value, allowed: u32) -> Result<(), DriverError> {
    let mut pending = vec![(value, 0_u32)];
    while let Some((value, depth)) = pending.pop() {
        if depth > allowed {
            return Err(DriverError::Authoring(
                pse_authoring::AuthoringError::Budget {
                    limit: "header depth",
                    allowed: u64::from(allowed),
                    needed: u64::from(depth),
                },
            ));
        }
        match value {
            toml::Value::Array(values) => {
                pending.extend(values.iter().map(|value| (value, depth + 1)));
            }
            toml::Value::Table(values) => {
                pending.extend(values.values().map(|value| (value, depth + 1)));
            }
            _ => {}
        }
    }
    Ok(())
}

fn reserve_inventory<T: AsRef<[u8]>>(
    sources: &BTreeMap<String, T>,
    header_text: &str,
    registry: &Registry,
    budget: &ParseBudget,
    funds: &mut super::allocation::Allocation<'_>,
) -> Result<(), DriverError> {
    let checksum_extent = sources.iter().try_fold(0, |sum, (path, bytes)| {
        memory::add(
            sum,
            memory::add(16, memory::add(path.len(), bytes.as_ref().len())?)?,
        )
    })?;
    funds.grow(memory::add(
        memory::mul(checksum_extent, 4)?,
        memory::mul(sources.len(), 4 * size_of::<Document>())?,
    )?)?;
    funds.grow(memory::parser_extent(header_text, budget, false)?)?;
    for path in sources.keys() {
        let declaration = select(registry, path)?;
        funds.grow(memory::mul(
            declaration.sections.len(),
            size_of::<pse_schema::model::DocumentSection>(),
        )?)?;
    }
    Ok(())
}

fn project_documents(
    documents: &mut [Document],
    registry: &Registry,
    mut allocation: Option<&mut super::allocation::Allocation<'_>>,
    validation: &pse_relations::validate::ValidationContext,
) -> Result<BTreeMap<SemanticId, Vec<FieldCheckedBatch>>, DriverError> {
    let mut parts = BTreeMap::<SemanticId, Vec<FieldCheckedBatch>>::new();
    for document in documents {
        if document.declaration.kind == DocumentKind::Data {
            continue;
        }
        if document.declaration.kind == DocumentKind::Modeling {
            let cancel = allocation
                .as_ref()
                .map_or_else(pse_columnar::CancellationToken::new, |funds| {
                    funds.cancel.clone()
                });
            cancel.checkpoint()?;
            if !document.modeling_validation_required(registry, validation)? {
                continue;
            }
            if let Some(funds) = allocation.as_deref_mut() {
                funds.grow(memory::modeling_batch_extent(document)?)?;
            }
            // Generated native validation is columnar today. Discard its narrow
            // projection immediately; the admitted parser rows remain authoritative.
            document.modeling_batch(registry, validation, &cancel)?;
            continue;
        }
        if let Some(funds) = allocation.as_deref_mut() {
            funds.grow(memory::row_extent(document, registry)?)?;
        }
        let decoded = crate::generated::document_adapters::batches_from_document(
            document.declaration.name,
            &document.value,
            registry,
            validation,
        )?;
        for (id, values) in decoded {
            let spec = registry
                .relations()
                .iter()
                .find(|spec| spec.id == id)
                .ok_or_else(|| contract(None, "unregistered document projection"))?;
            let batch = FieldCheckedBatch::concat(registry, spec, &values)?;
            document.batches.insert(id, batch.clone());
            parts.entry(id).or_default().push(batch);
        }
    }
    Ok(parts)
}

fn reuse_header(
    previous: Option<&DocumentBundle>,
    header_text: &str,
    checksum: ContentHash,
    budget: &ParseBudget,
) -> Result<(authored::packages::Row, value::Value, SpanIndex), DriverError> {
    let prior_header = previous.and_then(|bundle| {
        bundle.documents.iter().find(|document| {
            document.path == "package.toml"
                && document.text() == Some(header_text)
                && document.interpretation.budget == *budget
        })
    });
    let (package, header_value, header_spans) = if let Some(prior) = prior_header {
        let mut tree = (**prior.syntax).clone();
        let row = tree
            .get_mut("package")
            .ok_or_else(|| contract(None, "missing package header"))?;
        row.set("content_hash", value::Value::Text(checksum.to_prefixed()));
        let package = authored::packages::Row::deserialize(&*row)
            .map_err(|error| contract(None, &error.to_string()))?;
        (package, tree, prior.spans.clone())
    } else {
        header(header_text, checksum, budget)?
    };
    Ok((package, header_value, header_spans))
}

fn append_source_inventory(
    parts: &mut BTreeMap<SemanticId, Vec<FieldCheckedBatch>>,
    entities: Vec<authored::entities::Row>,
    documents: &[Document],
    package_id: pse_relations::generated::identities::PackageId,
    registry: &Registry,
    validation: &pse_relations::validate::ValidationContext,
) -> Result<(), DriverError> {
    let mut entity_builder =
        authored::entities::Builder::with_registry(registry, entities.len(), validation)?;
    for entity in entities {
        entity_builder.push(entity)?;
    }
    parts
        .entry(authored::entities::RELATION_ID)
        .or_default()
        .push(entity_builder.finish()?);
    let mut document_builder =
        authored::documents::Builder::with_registry(registry, documents.len(), validation)?;
    for document in documents {
        document_builder.push(authored::documents::Row {
            document_id: document.id,
            package_id,
            path: document.path.clone(),
            source_text: document.text().map(str::to_owned),
            content: match &document.content {
                Content::Text(_) | Content::Modeling { .. } => None,
                Content::Data { bytes, .. } => Some(pse_model::Bytes::new(bytes.to_vec())),
            },
        })?;
    }
    parts
        .entry(authored::documents::RELATION_ID)
        .or_default()
        .push(document_builder.finish()?);
    Ok(())
}

#[cfg(test)]
mod kernel_document_tests {
    use super::*;

    fn fixture_validation(registry: &Registry) -> Arc<pse_relations::validate::ValidationContext> {
        // This source-only fixture deliberately captures its fixed native session state.
        Arc::new(pse_relations::validate::ValidationContext::new(
            registry,
            pse_engine::validation::NativeValidation(
                datafusion::prelude::SessionContext::new().state(),
            ),
        ))
    }

    fn named_sources(model: &[u8]) -> BTreeMap<String, Vec<u8>> {
        BTreeMap::from([
            (
                "package.toml".into(),
                include_bytes!(
                    "../../../../../tests/fixtures/packages/minimal_explicit/package.toml"
                )
                .to_vec(),
            ),
            ("models/kernel.pse".into(), model.to_vec()),
        ])
        .into_iter()
        .map(|(path, bytes)| {
            if path == "package.toml" {
                (
                    path,
                    String::from_utf8(bytes)
                        .unwrap()
                        .replace("id_policy = \"explicit\"", "id_policy = \"named\"")
                        .into_bytes(),
                )
            } else {
                (path, bytes)
            }
        })
        .collect()
    }
    fn modeling_rows(bundle: &DocumentBundle) -> Vec<authored::modeling_declarations::Row> {
        bundle
            .documents
            .iter()
            .filter_map(Document::modeling_rows)
            .flat_map(|rows| rows.iter().cloned())
            .collect()
    }
    #[test]
    fn typed_modeling_rows_export_only_on_request_and_reuse_runs_native_admission() {
        let registry = pse_schema::shared_registry().unwrap();
        let validation = fixture_validation(&registry);
        let sources = named_sources(b"package q { def D { var x: Scalar; } }");
        let mut bundle = load_package_documents(
            sources.clone(),
            &registry,
            ParseBudget::default(),
            &validation,
        )
        .unwrap();
        let rows = modeling_rows(&bundle);
        assert!(
            !bundle
                .batches
                .contains_key(&authored::modeling_declarations::RELATION_ID)
        );
        let document = bundle
            .documents
            .iter()
            .find(|document| document.modeling_rows().is_some())
            .unwrap();
        let projected = document
            .modeling_batch(
                &registry,
                &validation,
                &pse_columnar::CancellationToken::new(),
            )
            .unwrap()
            .unwrap();
        use pse_relations::columnar::RelationRow;
        assert_eq!(
            rows,
            authored::modeling_declarations::Row::rows(&projected).unwrap()
        );
        assert!(
            !bundle
                .batches
                .contains_key(&authored::modeling_declarations::RELATION_ID)
        );
        let document = Arc::make_mut(&mut bundle.0)
            .documents
            .iter_mut()
            .find(|document| document.modeling_rows().is_some())
            .unwrap();
        let Content::Modeling { rows, .. } = &mut document.content else {
            panic!("expected retained typed modeling rows for the selected source document")
        };
        let original_rows = rows.clone();
        Arc::make_mut(rows).rows[0].source_start = -1;
        assert!(!Arc::ptr_eq(rows, &original_rows));
        assert!(original_rows.rows[0].source_start >= 0);
        assert!(
            load_reusing(
                sources.clone(),
                Some(&bundle),
                &registry,
                ParseBudget::default(),
                None,
                &validation
            )
            .is_err()
        );
        assert!(
            load_package_documents(sources, &registry, ParseBudget::default(), &validation).is_ok()
        );
    }
    #[tokio::test]
    async fn source_and_physical_boundaries_refuse_the_same_missing_registration() {
        let registry = pse_schema::shared_registry().unwrap();
        let pool: Arc<dyn pse_columnar::MemoryPool> =
            Arc::new(pse_columnar::GreedyMemoryPool::new(64 << 20));
        let thread = std::num::NonZeroUsize::new(1).unwrap();
        let sessions = pse_testkit::factory(
            pool.clone(),
            pse_engine::session::ExecutionSettings::default(),
            pse_engine::session::ThreadBudget {
                pool_threads: thread,
                target_partitions: thread,
            },
        )
        .unwrap();
        let validation = sessions.validation_context(&registry).unwrap();
        let mut sources = BTreeMap::from([
            ("package.toml".into(), include_bytes!("../../../../../tests/fixtures/packages/minimal_explicit/package.toml").to_vec()),
            ("materials/quantity-kinds.yaml".into(), include_bytes!("../../../../../tests/fixtures/packages/minimal_explicit/materials/quantity-kinds.yaml").to_vec()),
        ]);
        let mut bundle = load_package_documents(
            std::mem::take(&mut sources),
            &registry,
            ParseBudget::default(),
            &validation,
        )
        .unwrap();
        // Deliberately violate a private retained projection while preserving its native
        // field evidence; both downstream boundaries must inspect actual reference facts.
        Arc::make_mut(&mut bundle.0)
            .batches
            .remove(&authored::entities::RELATION_ID);
        let cancel = pse_columnar::CancellationToken::new();
        let part =
            super::super::owned::retain_bundle(&bundle, &registry, &pool, &cancel, &validation)
                .unwrap();
        let documents =
            super::super::OwnedDocumentSet::try_from_bundles(vec![part], &pool, &cancel).unwrap();
        let headers = documents
            .bundles()
            .iter()
            .map(|bundle| bundle.package.clone())
            .collect::<Vec<_>>();
        let limits = pse_authoring::p0::GraphLimits {
            nodes: headers.len(),
            edges: crate::authoring_driver::work::sources(documents.bundles()).unwrap() / 128,
        };
        let source = crate::authoring_driver::p1::source_context_batches(
            documents.bundles(),
            &registry,
            &headers,
            limits,
        )
        .unwrap_err();
        let physical = crate::workflow::PhysicalContext::from_documents(
            &documents, registry, &sessions, &cancel,
        )
        .await
        .unwrap_err();
        let DriverError::Rules(pse_rules::RuleError::References { violations: source }) = source
        else {
            panic!("typed source registration refusal");
        };
        let crate::workflow::WorkflowError::Authoring(DriverError::Rules(
            pse_rules::RuleError::References {
                violations: physical,
            },
        )) = physical
        else {
            panic!("typed physical registration refusal");
        };
        assert!(!source.is_empty());
        assert_eq!(source, physical);
    }

    #[tokio::test]
    async fn source_and_physical_boundaries_refuse_the_same_package_closure_and_cycle() {
        let registry = pse_schema::shared_registry().unwrap();
        let pool: Arc<dyn pse_columnar::MemoryPool> =
            Arc::new(pse_columnar::GreedyMemoryPool::new(64 << 20));
        let thread = std::num::NonZeroUsize::new(1).unwrap();
        let sessions = pse_testkit::factory(
            pool.clone(),
            pse_engine::session::ExecutionSettings::default(),
            pse_engine::session::ThreadBudget {
                pool_threads: thread,
                target_partitions: thread,
            },
        )
        .unwrap();
        let validation = sessions.validation_context(&registry).unwrap();
        let original = load_package_documents(
            named_sources(b"package q { def D {} }"),
            &registry,
            ParseBudget::default(),
            &validation,
        )
        .unwrap();
        for cycle in [false, true] {
            let mut bundle = original.clone();
            let header = &mut Arc::make_mut(&mut bundle.0).package;
            let target = if cycle {
                header.package_id
            } else {
                pse_relations::generated::identities::PackageId::from_bytes([99; 16])
            };
            header
                .dependencies
                .push(authored::packages::AuthoredPackagesFieldDependenciesItem {
                    package_id: target,
                    version_req: pse_authoring::language::exact_requirement(&header.version)
                        .unwrap(),
                });
            let cancel = pse_columnar::CancellationToken::new();
            let part =
                super::super::owned::retain_bundle(&bundle, &registry, &pool, &cancel, &validation)
                    .unwrap();
            let documents =
                super::super::OwnedDocumentSet::try_from_bundles(vec![part], &pool, &cancel)
                    .unwrap();
            let headers = documents
                .bundles()
                .iter()
                .map(|bundle| bundle.package.clone())
                .collect::<Vec<_>>();
            let limits = pse_authoring::p0::GraphLimits {
                nodes: headers.len(),
                edges: crate::authoring_driver::work::sources(documents.bundles()).unwrap() / 128,
            };
            let source = crate::authoring_driver::p1::source_context_batches(
                documents.bundles(),
                &registry,
                &headers,
                limits,
            )
            .unwrap_err();
            let physical = crate::workflow::PhysicalContext::from_documents(
                &documents,
                registry.clone(),
                &sessions,
                &cancel,
            )
            .await
            .unwrap_err();
            let DriverError::Authoring(pse_authoring::AuthoringError::Contract {
                reason: source,
                ..
            }) = source
            else {
                panic!("source package closure refusal")
            };
            let crate::workflow::WorkflowError::Authoring(DriverError::Authoring(
                pse_authoring::AuthoringError::Contract {
                    reason: physical, ..
                },
            )) = physical
            else {
                panic!("physical package closure refusal")
            };
            assert_eq!(source, physical);
            assert!(source.contains(if cycle {
                "package dependency cycle at"
            } else {
                "package dependency target absent"
            }));
            if cycle {
                assert!(source.contains(&target.to_string()));
            }
        }
    }

    #[test]
    fn complete_source_package_context_retains_admitted_external_headers() {
        let registry = pse_schema::shared_registry().unwrap();
        let mut bundle = load_package_documents(
            named_sources(b"package q { def D {} }"),
            &registry,
            ParseBudget::default(),
            &fixture_validation(&registry),
        )
        .unwrap();
        let mut external = bundle.package.clone();
        external.package_id = pse_relations::generated::identities::PackageId::from_bytes([97; 16]);
        external.name = "external".into();
        Arc::make_mut(&mut bundle.0).package.dependencies.push(
            authored::packages::AuthoredPackagesFieldDependenciesItem {
                package_id: external.package_id,
                version_req: pse_authoring::language::exact_requirement(&external.version).unwrap(),
            },
        );
        let limits = pse_authoring::p0::GraphLimits { nodes: 2, edges: 1 };
        let mut headers = vec![bundle.package.clone(), external];
        assert!(
            crate::authoring_driver::p1::source_context_batches(
                std::slice::from_ref(&bundle),
                &registry,
                &headers,
                limits,
            )
            .is_ok()
        );
        headers[0].version = "2.0.0".into();
        let refused = crate::authoring_driver::p1::source_context_batches(
            std::slice::from_ref(&bundle),
            &registry,
            &headers,
            limits,
        )
        .unwrap_err();
        assert!(
            matches!(refused, DriverError::Authoring(pse_authoring::AuthoringError::Contract { reason, .. }) if reason == "complete package context differs from submitted source header")
        );
    }

    #[test]
    fn document_reuse_policy_changes_match_clean_admission() {
        let registry = pse_schema::shared_registry().unwrap();
        let budget = ParseBudget::default();
        let mut sources = named_sources(b"package q { def D { var x: Scalar; } }");
        let old = load_package_documents(
            sources.clone(),
            &registry,
            budget,
            &fixture_validation(&registry),
        )
        .unwrap();
        let header = String::from_utf8(sources["package.toml"].clone())
            .unwrap()
            .replace("id_policy = \"named\"", "id_policy = \"explicit\"");
        sources.insert("package.toml".into(), header.into_bytes());
        let incremental = load_reusing(
            sources.clone(),
            Some(&old),
            &registry,
            budget,
            None,
            &fixture_validation(&registry),
        )
        .unwrap_err();
        let clean =
            load_package_documents(sources, &registry, budget, &fixture_validation(&registry))
                .unwrap_err();
        assert_eq!(incremental.to_string(), clean.to_string());
        let mut explicit = named_sources(b"@id(\"00000000000000000000000000000031\") package q { @id(\"00000000000000000000000000000032\") def D {} }");
        let header = String::from_utf8(explicit["package.toml"].clone())
            .unwrap()
            .replace("id_policy = \"named\"", "id_policy = \"explicit\"");
        explicit.insert("package.toml".into(), header.into_bytes());
        let old = load_package_documents(
            explicit.clone(),
            &registry,
            budget,
            &fixture_validation(&registry),
        )
        .unwrap();
        let header = String::from_utf8(explicit["package.toml"].clone())
            .unwrap()
            .replace("id_policy = \"explicit\"", "id_policy = \"named\"");
        explicit.insert("package.toml".into(), header.into_bytes());
        let incremental = load_reusing(
            explicit.clone(),
            Some(&old),
            &registry,
            budget,
            None,
            &fixture_validation(&registry),
        )
        .unwrap();
        let clean =
            load_package_documents(explicit, &registry, budget, &fixture_validation(&registry))
                .unwrap();
        assert_eq!(modeling_rows(&incremental), modeling_rows(&clean));
        let original = old
            .documents
            .iter()
            .find(|d| d.path == "models/kernel.pse")
            .unwrap();
        let next = incremental
            .documents
            .iter()
            .find(|d| d.path == "models/kernel.pse")
            .unwrap();
        assert!(!Arc::ptr_eq(&original.syntax, &next.syntax));
    }
    #[test]
    fn document_reuse_unrelated_manifest_edits_keep_parser_owner() {
        let registry = pse_schema::shared_registry().unwrap();
        let budget = ParseBudget::default();
        let mut sources = named_sources(b"package q { def D {} }");
        let old = load_package_documents(
            sources.clone(),
            &registry,
            budget,
            &fixture_validation(&registry),
        )
        .unwrap();
        let header = String::from_utf8(sources["package.toml"].clone())
            .unwrap()
            .replace("Minimal explicit identity fixture.", "Updated description.");
        sources.insert("package.toml".into(), header.into_bytes());
        let next = load_reusing(
            sources.clone(),
            Some(&old),
            &registry,
            budget,
            None,
            &fixture_validation(&registry),
        )
        .unwrap();
        let clean =
            load_package_documents(sources, &registry, budget, &fixture_validation(&registry))
                .unwrap();
        let original = old
            .documents
            .iter()
            .find(|d| d.path == "models/kernel.pse")
            .unwrap();
        let reused = next
            .documents
            .iter()
            .find(|d| d.path == "models/kernel.pse")
            .unwrap();
        assert!(Arc::ptr_eq(&original.syntax, &reused.syntax));
        assert_eq!(modeling_rows(&next), modeling_rows(&clean));
    }
    #[test]
    fn document_reuse_identity_changes_rebind_rows_and_binary_wrappers() {
        use datafusion::arrow::{
            array::{Int64Array, RecordBatch},
            datatypes::{DataType, Field, Schema},
        };
        let registry = pse_schema::shared_registry().unwrap();
        let budget = ParseBudget::default();
        let mut sources = named_sources(b"package q { def D { var x: Scalar; } }");
        let schema = Arc::new(Schema::new(vec![Field::new(
            "value",
            DataType::Int64,
            false,
        )]));
        let batch =
            RecordBatch::try_new(schema.clone(), vec![Arc::new(Int64Array::from(vec![1, 2]))])
                .unwrap();
        let mut bytes = Vec::new();
        let mut writer = parquet::arrow::ArrowWriter::try_new(&mut bytes, schema, None).unwrap();
        writer.write(&batch).unwrap();
        writer.close().unwrap();
        sources.insert("data/bank.parquet".into(), bytes);
        let old = load_package_documents(
            sources.clone(),
            &registry,
            budget,
            &fixture_validation(&registry),
        )
        .unwrap();
        let header = String::from_utf8(sources["package.toml"].clone())
            .unwrap()
            .replace(
                "01991d6a-13a0-7000-8000-000000000001",
                "01991d6a-13a0-7000-8000-000000000002",
            );
        sources.insert("package.toml".into(), header.into_bytes());
        let incremental = load_reusing(
            sources.clone(),
            Some(&old),
            &registry,
            budget,
            None,
            &fixture_validation(&registry),
        )
        .unwrap();
        let clean =
            load_package_documents(sources, &registry, budget, &fixture_validation(&registry))
                .unwrap();
        assert_eq!(modeling_rows(&incremental), modeling_rows(&clean));
        assert_ne!(
            modeling_rows(&old)[0].declaration_id,
            modeling_rows(&incremental)[0].declaration_id
        );
        let binary = incremental
            .documents
            .iter()
            .find(|d| d.path == "data/bank.parquet")
            .unwrap();
        assert_eq!(binary.id, binary.data().unwrap().id);
        let clean_binary = clean
            .documents
            .iter()
            .find(|d| d.path == "data/bank.parquet")
            .unwrap();
        assert_eq!(binary.data(), clean_binary.data());
        let tighter = ParseBudget {
            max_data_rows: 1,
            ..budget
        };
        let sources = incremental
            .documents
            .iter()
            .map(|d| (d.path.clone(), d.bytes().to_vec()))
            .collect();
        assert!(
            load_reusing(
                sources,
                Some(&incremental),
                &registry,
                tighter,
                None,
                &fixture_validation(&registry)
            )
            .is_err()
        );
    }
    #[test]
    fn pse_source_uses_generated_rows_and_original_ranges() {
        let registry = pse_schema::shared_registry().unwrap();
        let mut texts = BTreeMap::from([
            (
                "package.toml".to_owned(),
                include_bytes!(
                    "../../../../../tests/fixtures/packages/minimal_explicit/package.toml"
                )
                .to_vec(),
            ),
            (
                "models/kernel.pse".to_owned(),
                b"package synthetic { def Root { var x: Scalar; eq e: x == 1; } }".to_vec(),
            ),
        ]);
        let mut serial = 10u8;
        let edits = super::super::assign_ids(
            &texts,
            &registry,
            ParseBudget::default(),
            &mut || {
                serial += 1;
                SemanticId::from_bytes([serial; 16])
            },
            &fixture_validation(&registry),
        )
        .unwrap();
        super::super::apply_edits(&mut texts, &edits).unwrap();
        let bundle = load_package_documents(
            texts.clone(),
            &registry,
            ParseBudget::default(),
            &fixture_validation(&registry),
        )
        .unwrap();
        let rows = modeling_rows(&bundle);
        assert!(
            !bundle
                .batches
                .contains_key(&authored::modeling_declarations::RELATION_ID)
        );
        assert_eq!(rows.len(), 4);
        let document = bundle
            .documents
            .iter()
            .find(|d| d.path == "models/kernel.pse")
            .unwrap();
        for (ordinal, row) in rows.iter().enumerate() {
            let span = document
                .spans
                .span(&format!("/modeling_declarations/{ordinal}"))
                .unwrap();
            assert_eq!(u64::from(span.start), row.source_start as u64);
            assert!(
                document.text().unwrap()[span.start as usize..span.end as usize].starts_with("@id")
            );
        }
        let reparsed = load_package_documents(
            texts,
            &registry,
            ParseBudget::default(),
            &fixture_validation(&registry),
        )
        .unwrap();
        let again = modeling_rows(&reparsed);
        assert_eq!(rows, again);
    }
}
