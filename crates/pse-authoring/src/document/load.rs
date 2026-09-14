// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Package loading into generated row contracts without publishing a candidate.

use super::{SpanIndex, allocation as memory, hydrate, read, value};
use crate::{AuthoringError, ParseBudget, SourceSpan};
use pse_ids::{ContentHash, SemanticId};
use pse_relations::generated::authored;
use pse_schema::Registry;
use pse_schema::model::{Cell, DocumentKind, DocumentSpec};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

/// Rows retain the registry relation identity and exact column order.
pub type Rows = BTreeMap<SemanticId, Vec<Vec<Cell>>>;

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
    /// Original row spans in emitted relation row order.
    pub row_spans: BTreeMap<SemanticId, Vec<SourceSpan>>,
    pub(super) declaration: DocumentSpec,
    pub(super) value: value::Value,
}

/// A parsed package. Cross-package closure belongs to P0/P1 before publication.
#[derive(Clone, Debug)]
pub struct DocumentBundle {
    /// Generated package header row.
    pub package: authored::packages::Row,
    /// Original documents in relative-path order.
    pub documents: Vec<Document>,
    /// Strictly decoded generated rows, including package, documents and entities.
    pub rows: Rows,
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
) -> Result<DocumentBundle, AuthoringError> {
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
) -> Result<DocumentBundle, AuthoringError> {
    load_inventory(texts, registry, budget, None)
}

pub(super) fn load_inventory(
    texts: BTreeMap<String, String>,
    registry: &Registry,
    budget: ParseBudget,
    mut allocation: Option<&mut super::allocation::Allocation<'_>>,
) -> Result<DocumentBundle, AuthoringError> {
    check_size(&texts, budget.max_bytes)?;
    let header_text = texts
        .get("package.toml")
        .ok_or_else(|| contract(None, "missing package.toml"))?;
    if let Some(funds) = allocation.as_deref_mut() {
        let checksum_extent = texts.iter().try_fold(0, |sum, (path, text)| {
            memory::add(sum, memory::add(16, memory::add(path.len(), text.len())?)?)
        })?;
        funds.grow(memory::add(
            memory::mul(checksum_extent, 4)?,
            memory::mul(texts.len(), 4 * size_of::<Document>())?,
        )?)?;
        funds.grow(memory::parser_extent(header_text, &budget, false)?)?;
        for path in texts.keys() {
            let declaration = select(registry, path)?;
            funds.grow(memory::mul(
                declaration.sections.len(),
                size_of::<pse_schema::model::DocumentSection>(),
            )?)?;
        }
    }
    let checksum = package_checksum(&texts);
    let (package, header_value, header_spans) = header(header_text, checksum, &budget)?;
    let mut header_parts = Some((header_value, header_spans));
    let mut documents = Vec::with_capacity(texts.len());
    for (path, text) in texts {
        let declaration = select(registry, &path)?.clone();
        let id = pse_ids::named_id(package.package_id, &path);
        let (value, spans) = if declaration.kind == DocumentKind::PackageHeader {
            header_parts
                .take()
                .ok_or_else(|| contract(None, "duplicate package header shape"))?
        } else {
            let (value, spans) =
                value::parse_yaml_accounted(&text, id, &budget, allocation.as_deref_mut())?;
            (value.value, spans)
        };
        documents.push(Document {
            id,
            path,
            content_hash: pse_ids::encoding_checksum(text.as_bytes()).content_hash(),
            text,
            spans,
            row_spans: BTreeMap::new(),
            declaration,
            value,
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
    let mut rows = Rows::new();
    for document in &documents {
        if let Some(funds) = allocation.as_deref_mut() {
            funds.grow(memory::row_extent(document, registry)?)?;
        }
        super::binding::validate_document(document, registry)?;
        let decoded = crate::generated::documents::rows_from_document(
            document.declaration.name,
            document.value.clone(),
        )
        .map_err(|error| contract(Some(SourceSpan::head(document.id)), &error.to_string()))?;
        for (id, values) in decoded {
            rows.entry(id).or_default().extend(values);
        }
    }
    append_context_rows(
        registry,
        &mut rows,
        entities,
        &documents,
        package.package_id,
        allocation.as_deref_mut(),
    )?;
    validate_rows(registry, &rows, allocation)?;
    Ok(DocumentBundle {
        package,
        documents,
        rows,
    })
}

fn append_context_rows(
    registry: &Registry,
    rows: &mut Rows,
    entities: Vec<authored::entities::Row>,
    documents: &[Document],
    package_id: SemanticId,
    allocation: Option<&mut memory::Allocation<'_>>,
) -> Result<(), AuthoringError> {
    if let Some(funds) = allocation {
        let mut bytes = memory::mul(
            entities.len(),
            8 * size_of::<Vec<Cell>>() + 64 * size_of::<Cell>(),
        )?;
        for entity in &entities {
            bytes = memory::add(
                bytes,
                memory::mul(
                    memory::add(entity.name.len(), entity.qualified_name.len())?,
                    4,
                )?,
            )?;
        }
        for document in documents {
            bytes = memory::add(
                bytes,
                memory::add(32 * size_of::<Cell>(), memory::mul(document.path.len(), 4)?)?,
            )?;
        }
        funds.grow(bytes)?;
    }
    insert(
        registry,
        rows,
        "authored.entities",
        entities
            .into_iter()
            .map(authored::entities::Row::into_cells)
            .collect(),
    )?;
    insert(
        registry,
        rows,
        "authored.documents",
        documents
            .iter()
            .map(|document| {
                authored::documents::Row {
                    document_id: document.id,
                    package_id,
                    path: document.path.clone(),
                    content_hash: document.content_hash,
                }
                .into_cells()
            })
            .collect(),
    )?;
    Ok(())
}

pub(super) fn header(
    text: &str,
    checksum: ContentHash,
    budget: &ParseBudget,
) -> Result<(authored::packages::Row, value::Value, SpanIndex), AuthoringError> {
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
    let id = crate::ids::parse_id(id, SourceSpan::head(SemanticId::NIL))?;
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
            let dependency_id = crate::ids::parse_id(dependency_id, SourceSpan::head(id))?;
            dependency
                .value
                .set("package_id", value::Value::Text(dependency_id.to_string()));
        }
    }
    let package = authored::packages::Row::deserialize(row.clone())
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
) -> Result<&'a DocumentSpec, AuthoringError> {
    if path.split('/').any(|part| matches!(part, "" | "." | "..")) || path.contains('\\') {
        return Err(contract(None, "invalid package-relative path"));
    }
    let mut matches = registry.documents().iter().filter(|document| {
        document.path_glob == path
            || document
                .path_glob
                .strip_suffix("*.yaml")
                .is_some_and(|prefix| {
                    path.strip_prefix(prefix).is_some_and(|name| {
                        Path::new(name).extension() == Some(std::ffi::OsStr::new("yaml"))
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

fn check_size(texts: &BTreeMap<String, String>, allowed: u64) -> Result<(), AuthoringError> {
    let needed = texts
        .values()
        .try_fold(0_u64, |sum, text| {
            sum.checked_add(u64::try_from(text.len()).unwrap_or(u64::MAX))
        })
        .unwrap_or(u64::MAX);
    if needed > allowed {
        Err(AuthoringError::Budget {
            limit: "package bytes",
            allowed,
            needed,
        })
    } else {
        Ok(())
    }
}

fn package_checksum(texts: &BTreeMap<String, String>) -> ContentHash {
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

pub(super) fn contract(at: Option<SourceSpan>, reason: &str) -> AuthoringError {
    AuthoringError::Contract {
        at,
        reason: reason.to_owned(),
    }
}

fn insert(
    registry: &Registry,
    rows: &mut Rows,
    relation: &str,
    values: Vec<Vec<Cell>>,
) -> Result<(), AuthoringError> {
    let spec = registry
        .relation(relation)
        .ok_or_else(|| contract(None, &format!("missing {relation} declaration")))?;
    rows.entry(spec.id).or_default().extend(values);
    Ok(())
}

fn validate_rows(
    registry: &Registry,
    rows: &Rows,
    mut allocation: Option<&mut memory::Allocation<'_>>,
) -> Result<(), AuthoringError> {
    for (id, rows) in rows {
        let spec = registry
            .relations()
            .iter()
            .find(|spec| spec.id == *id)
            .ok_or_else(|| contract(None, "unregistered output relation"))?;
        if let Some(funds) = allocation.as_deref_mut() {
            funds.grow(memory::key_extent(rows)?)?;
            pse_relations::cells::batch_from_cells_owned(
                registry,
                spec,
                rows,
                funds.reserver,
                funds.cancel,
            )?;
        } else {
            pse_relations::cells::batch_from_cells(registry, spec, rows)
                .map_err(|error| contract(None, &error.to_string()))?;
        }
        let indices = spec
            .primary_key
            .iter()
            .map(|name| {
                spec.columns
                    .iter()
                    .position(|column| column.name == *name)
                    .ok_or_else(|| contract(None, "unregistered primary key"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut keys = std::collections::BTreeSet::new();
        for row in rows {
            if !keys.insert(
                indices
                    .iter()
                    .map(|index| row[*index].literal_spec())
                    .collect::<Vec<_>>(),
            ) {
                return Err(contract(
                    None,
                    &format!("duplicate primary key in {}", spec.key),
                ));
            }
        }
    }
    Ok(())
}

fn check_header_depth(value: &toml::Value, allowed: u32) -> Result<(), AuthoringError> {
    let mut pending = vec![(value, 0_u32)];
    while let Some((value, depth)) = pending.pop() {
        if depth > allowed {
            return Err(AuthoringError::Budget {
                limit: "header depth",
                allowed: u64::from(allowed),
                needed: u64::from(depth),
            });
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
