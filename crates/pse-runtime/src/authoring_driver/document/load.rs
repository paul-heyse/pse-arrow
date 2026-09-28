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

/// One immutable original document plus its parser-derived locations.
#[derive(Clone, Debug)]
pub struct Document {
    /// Stable package-relative document identity.
    pub id: SemanticId,
    /// Original package-relative path.
    pub path: String,
    /// Original UTF-8 bytes; never reconstructed for source locations.
    pub text: String,
    /// Integrity identity of the exact original bytes.
    pub content_hash: ContentHash,
    /// Exact original syntax node locations.
    pub spans: SpanIndex,
    pub(super) declaration: DocumentSpec,
    pub(super) value: value::Value,
    pub(super) syntax: Arc<value::Value>,
    /// Columnar projection of this exact parsed document.
    pub batches: Batches,
    lease: Option<Arc<pse_columnar::AllocationLease>>,
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
    pub(super) fn attach_lease(&mut self, lease: Arc<pse_columnar::AllocationLease>) {
        let data = Arc::make_mut(&mut self.0);
        for document in &mut data.documents {
            document.lease = Some(Arc::clone(&lease));
        }
        data.lease = Some(lease);
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
) -> Result<DocumentBundle, DriverError> {
    load_package_texts(
        read::package_files(root, registry, &budget)?,
        registry,
        budget,
    )
}

/// Load an explicit immutable inventory, also used by document editors and fixtures.
///
/// # Errors
/// The same syntax and semantic authoring failures as [`load_package`].
pub fn load_package_texts(
    texts: BTreeMap<String, String>,
    registry: &Registry,
    budget: ParseBudget,
) -> Result<DocumentBundle, DriverError> {
    load_inventory(texts, registry, budget, None)
}

pub(super) fn load_inventory(
    texts: BTreeMap<String, String>,
    registry: &Registry,
    budget: ParseBudget,
    allocation: Option<&mut super::allocation::Allocation<'_>>,
) -> Result<DocumentBundle, DriverError> {
    load_reusing(texts, None, registry, budget, allocation)
}

pub(super) fn load_reusing(
    texts: BTreeMap<String, String>,
    previous: Option<&DocumentBundle>,
    registry: &Registry,
    budget: ParseBudget,
    mut allocation: Option<&mut super::allocation::Allocation<'_>>,
) -> Result<DocumentBundle, DriverError> {
    check_size(&texts, budget.max_bytes)?;
    // Generated row admission may contain SQL checks. Their built-in function
    // owner belongs to this native authoring boundary, never the pure registry.
    pse_engine::validation::bind_defaults(registry)?;
    let header_text = texts
        .get("package.toml")
        .ok_or_else(|| contract(None, "missing package.toml"))?;
    if let Some(funds) = allocation.as_deref_mut() {
        reserve_inventory(&texts, header_text, registry, &budget, funds)?;
    }
    let checksum = package_checksum(&texts);
    let (package, header_value, header_spans) =
        reuse_header(previous, header_text, checksum, &budget)?;
    let mut header_parts = Some((header_value, header_spans));
    let mut documents = Vec::with_capacity(texts.len());
    for (path, text) in texts {
        let declaration = select(registry, &path)?.clone();
        let id = pse_ids::named_id(package.package_id.as_id(), &path);
        let mut source_batches=Batches::new();
        let (value, spans) = if declaration.kind == DocumentKind::PackageHeader {
            header_parts
                .take()
                .ok_or_else(|| contract(None, "duplicate package header shape"))?
        } else if let Some(prior) = previous.and_then(|bundle| {
            bundle.documents.iter().find(|document| {
                document.path == path
                    && document.text == text
                    && document.declaration == declaration
            })
        }) {
            if declaration.kind==DocumentKind::Modeling{source_batches=prior.batches.clone();}
            ((*prior.syntax).clone(), prior.spans.clone())
        } else if declaration.kind==DocumentKind::Modeling {
            if let Some(funds)=allocation.as_deref_mut(){funds.grow(memory::parser_extent(&text,&budget,false)?)?;}
            let policy=if package.id_policy==pse_relations::generated::enums::IdPolicy::Named{pse_authoring::language::IdentityPolicy::Named}else{pse_authoring::language::IdentityPolicy::Explicit};
            let rows=pse_authoring::language::parse(&text,id,policy,budget)?;
            let mut spans=SpanIndex::default();
            for (ordinal,row) in rows.iter().enumerate(){spans.insert(format!("/modeling_declarations/{ordinal}"),SourceSpan::new(id,row.source_start as u32,row.source_end as u32));}
            let mut builder=authored::modeling_declarations::Builder::with_registry(registry,rows.len())?;
            for row in rows{builder.push(row)?;}
            source_batches.insert(authored::modeling_declarations::RELATION_ID,builder.finish()?);
            (value::Value::Map(Vec::new()),spans)
        } else {
            let (value, spans) =
                value::parse_yaml_accounted(&text, id, &budget, allocation.as_deref_mut())?;
            (value.value, spans)
        };
        let syntax = previous
            .and_then(|bundle| {
                bundle.documents.iter().find(|document| {
                    document.path == path
                        && document.text == text
                        && document.declaration == declaration
                })
            })
            .map_or_else(
                || Arc::new(value.clone()),
                |prior| Arc::clone(&prior.syntax),
            );
        documents.push(Document {
            id,
            path,
            content_hash: pse_ids::encoding_checksum(text.as_bytes()).content_hash(),
            text,
            spans,
            declaration,
            value,
            syntax,
            batches: source_batches,
            lease: None,
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
    let mut parts = project_documents(&mut documents, registry, allocation.as_deref_mut())?;
    if let Some(funds) = allocation {
        funds.grow(memory::mul(entities.len() + documents.len(), 1024)?)?;
    }
    append_source_inventory(
        &mut parts,
        entities,
        &documents,
        package.package_id,
        registry,
    )?;
    let batches = concatenate_parts(parts, registry)?;
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
            || document.path_glob.rsplit_once("*.").is_some_and(|(prefix,extension)|{
                path.strip_prefix(prefix).is_some_and(|name|Path::new(name).extension()==Some(std::ffi::OsStr::new(extension))&&!name.contains('/'))
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

fn check_size(texts: &BTreeMap<String, String>, allowed: u64) -> Result<(), DriverError> {
    let needed = texts
        .values()
        .try_fold(0_u64, |sum, text| {
            sum.checked_add(u64::try_from(text.len()).unwrap_or(u64::MAX))
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

/// The package content hash (blueprint §6.1) over sorted path/text pairs: the identity of an
/// authored package, and of a source bundle stored for job execution (ADR-0112).
pub fn package_checksum(texts: &BTreeMap<String, String>) -> ContentHash {
    // An integrity encoding: sorted path/text pairs, each prefixed by its byte length.
    let mut bytes = Vec::new();
    for (path, text) in texts {
        for part in [path.as_bytes(), text.as_bytes()] {
            bytes.extend_from_slice(&u64::try_from(part.len()).unwrap_or(u64::MAX).to_le_bytes());
            bytes.extend_from_slice(part);
        }
    }
    pse_ids::encoding_checksum(&bytes).content_hash()
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

fn reserve_inventory(
    texts: &BTreeMap<String, String>,
    header_text: &str,
    registry: &Registry,
    budget: &ParseBudget,
    funds: &mut super::allocation::Allocation<'_>,
) -> Result<(), DriverError> {
    let checksum_extent = texts.iter().try_fold(0, |sum, (path, text)| {
        memory::add(sum, memory::add(16, memory::add(path.len(), text.len())?)?)
    })?;
    funds.grow(memory::add(
        memory::mul(checksum_extent, 4)?,
        memory::mul(texts.len(), 4 * size_of::<Document>())?,
    )?)?;
    funds.grow(memory::parser_extent(header_text, budget, false)?)?;
    for path in texts.keys() {
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
) -> Result<BTreeMap<SemanticId, Vec<FieldCheckedBatch>>, DriverError> {
    let mut parts = BTreeMap::<SemanticId, Vec<FieldCheckedBatch>>::new();
    for document in documents {
        if document.declaration.kind==DocumentKind::Modeling {
            for (id,batch) in &document.batches {parts.entry(*id).or_default().push(batch.clone());}
            continue;
        }
        if let Some(funds) = allocation.as_deref_mut() {
            funds.grow(memory::row_extent(document, registry)?)?;
        }
        let decoded = crate::generated::document_adapters::batches_from_document(
            document.declaration.name,
            &document.value,
            registry,
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
        bundle
            .documents
            .iter()
            .find(|document| document.path == "package.toml" && document.text == header_text)
    });
    let (package, header_value, header_spans) = if let Some(prior) = prior_header {
        let mut tree = (*prior.syntax).clone();
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
) -> Result<(), DriverError> {
    let mut entity_builder = authored::entities::Builder::with_registry(registry, entities.len())?;
    for entity in entities {
        entity_builder.push(entity)?;
    }
    parts
        .entry(authored::entities::RELATION_ID)
        .or_default()
        .push(entity_builder.finish()?);
    let mut document_builder =
        authored::documents::Builder::with_registry(registry, documents.len())?;
    for document in documents {
        document_builder.push(authored::documents::Row {
            document_id: document.id,
            package_id,
            path: document.path.clone(),
            source_text: document.text.clone(),
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
    #[test]
    fn pse_source_uses_generated_rows_and_original_ranges(){
        let registry=pse_schema::shared_registry().unwrap();
        let mut texts=BTreeMap::from([
            ("package.toml".into(),include_str!("../../../../../tests/fixtures/packages/minimal_explicit/package.toml").into()),
            ("models/kernel.pse".into(),"package synthetic { def Root { var x: Scalar; eq e: x == 1; } }".into()),
        ]);
        let mut serial=10u8;
        let edits=super::super::assign_ids(&texts,&registry,ParseBudget::default(),&mut ||{serial+=1;SemanticId::from_bytes([serial;16])}).unwrap();
        super::super::apply_edits(&mut texts,&edits).unwrap();
        let bundle=load_package_texts(texts.clone(),&registry,ParseBudget::default()).unwrap();
        let batch=&bundle.batches[&authored::modeling_declarations::RELATION_ID];
        let rows=authored::modeling_declarations::View::from_checked(batch).unwrap().rows().unwrap();
        assert_eq!(rows.len(),4);
        let document=bundle.documents.iter().find(|d|d.path=="models/kernel.pse").unwrap();
        for (ordinal,row) in rows.iter().enumerate(){
            let span=document.spans.span(&format!("/modeling_declarations/{ordinal}")).unwrap();
            assert_eq!(u64::from(span.start),row.source_start as u64);
            assert!(document.text[span.start as usize..span.end as usize].starts_with("@id"));
        }
        let reparsed=load_package_texts(texts,&registry,ParseBudget::default()).unwrap();
        let again=authored::modeling_declarations::View::from_checked(&reparsed.batches[&authored::modeling_declarations::RELATION_ID]).unwrap().rows().unwrap();
        assert_eq!(rows,again);
    }
}
