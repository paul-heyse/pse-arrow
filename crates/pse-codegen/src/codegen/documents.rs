// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Source-bearing JSON Schemas derived from their actual Rust serde owners.
//! Input and output contracts remain independent files with their original names
//! and references. Native Rust declarations supply constructors and type safety;
//! schema publication neither synthesizes executable types nor loads a runtime.
use std::{collections::{BTreeMap, BTreeSet}, path::{Path, PathBuf}};
use serde_json::Value;
use sha2::{Digest, Sha256};
use super::GeneratedTree;
use crate::{Registry, SchemaError};

/// Published product and authoring JSON Schemas.
pub const SCHEMA_ROOT: &str = "docs/generated/schema";
/// Responsibility-owned private JSON Schemas and native metadata.
pub const PRIVATE_SCHEMA_ROOT: &str = "scripts/generated";

/// Committed trees owned by product document publication.
pub fn roots() -> Vec<PathBuf> { vec![PathBuf::from(SCHEMA_ROOT)] }

/// One Rust-owned document: its published name and its derived JSON Schema.
#[derive(Clone, Debug)]
pub struct Document {
    /// File stem of the published schema, such as `solve-settings`.
    pub name: String,
    /// The schemars-derived root schema, with its `$defs`.
    pub schema: Value,
}

/// One direction-qualified actual compiled boundary schema and its public Rust path.
#[derive(Clone, Debug)]
pub struct CatalogueDocument {
    /// Exported name.
    pub name: String,
    /// Published schema slug.
    pub slug: String,
    /// Public Rust path, excluded from catalogue identity.
    pub rust_type: String,
    /// Actual registration source supplied by the catalogue producer, not the type definition.
    pub registration_source: String,
    /// Whether this is the serialization contract.
    pub output: bool,
    /// Owning standalone version, when this is a versioned root.
    pub version: Option<u32>,
    /// Actual wire-owner selection, passed through from the runtime catalogue.
    pub wire: String,
    /// Complete direction-specific schema.
    pub schema: Value,
}

/// Publish exact compiled direction-specific schemas and their source catalogue.
/// # Errors
/// Invalid schema titles, repeated directional identities or nonlocal schema names.
pub fn generate_catalogue(reg: &Registry, catalogue: &[CatalogueDocument], fingerprint: pse_ids::ContentHash) -> Result<GeneratedTree, SchemaError> {
    validate_catalogue(catalogue)?;
    let documents = catalogue.iter().map(|entry| Document { name: format!("{}.{}",entry.slug,direction(entry)),schema:entry.schema.clone() }).collect::<Vec<_>>();
    let mut tree = generate(reg,&documents)?;
    let mut rows = Vec::new();
    for (entry,document) in catalogue.iter().zip(&documents) {
        let schema_path = format!("{SCHEMA_ROOT}/{}.schema.json",document.name);
        rows.push(catalogue_row(entry,&schema_path,&tree.files[Path::new(&schema_path)]));
        catalogue_relationships(&mut tree,entry,fingerprint,&schema_path,&format!("{SCHEMA_ROOT}/catalogue.json"),"runtime_catalogue","generate_catalogue");
    }
    rows.sort_by(|left,right|(&left["slug"].as_str(),&left["direction"].as_str()).cmp(&(&right["slug"].as_str(),&right["direction"].as_str())));
    let authoring = format!("{SCHEMA_ROOT}/authoring.schema.json");
    let manifest = serde_json::json!({"version":1,"schema_set_fingerprint":fingerprint.to_prefixed(),"authoring":{"schema_path":authoring,"sha256":hex::encode(Sha256::digest(&tree.files[Path::new(&authoring)]))},"documents":rows});
    tree.files.insert(format!("{SCHEMA_ROOT}/catalogue.json").into(),canonical(&manifest)?.into_bytes());
    Ok(tree)
}

/// Publish complete schemas without interpreting or weakening their native constructs.
/// # Errors
/// Nonlocal or duplicate document names, or a schema that is not a JSON object.
pub fn generate(reg: &Registry, documents: &[Document]) -> Result<GeneratedTree, SchemaError> {
    let mut tree = GeneratedTree::empty(roots());
    for document in documents { insert_document(&mut tree,SCHEMA_ROOT,document)?; }
    if tree.files.insert(format!("{SCHEMA_ROOT}/authoring.schema.json").into(),super::jsonschema::generate(reg)?.into_bytes()).is_some() { return Err(error("document name conflicts with the authoring schema owner")); }
    for document in reg.documents().iter().filter(|spec|spec.kind.is_text()) {
        super::provenance::document(&mut tree,reg,document,"crates/pse-codegen/src/codegen/jsonschema.rs","generate",&format!("{SCHEMA_ROOT}/authoring.schema.json"));
    }
    for relation in reg.relations() {
        tree.relationships.push(super::provenance::SourceRelationship::registry(reg,"relation",super::provenance::relation_key(relation),"crates/pse-codegen/src/codegen/jsonschema.rs","generate",format!("{SCHEMA_ROOT}/authoring.schema.json")));
    }
    Ok(tree)
}

/// Publish private native documents without executable projections or package imports.
/// # Errors
/// Invalid or repeated schema names, or a schema that is not an object.
pub fn generate_private(_reg: &Registry, documents: &[Document]) -> Result<GeneratedTree, SchemaError> {
    let mut tree=GeneratedTree::empty(Vec::new());
    for document in documents { insert_document(&mut tree,PRIVATE_SCHEMA_ROOT,document)?; }
    tree.roots.extend(tree.files.keys().cloned());
    Ok(tree)
}

/// Publish related private native schemas at their existing family-owned paths.
/// # Errors
/// Invalid or repeated family/document names, or a malformed schema object.
pub fn generate_private_family(_reg: &Registry, family: &str, documents: &[Document]) -> Result<GeneratedTree, SchemaError> {
    local_name(family)?;
    let root=format!("{PRIVATE_SCHEMA_ROOT}/{}_schema",family.replace('-',"_"));
    let mut tree=GeneratedTree::empty(vec![PathBuf::from(&root)]);
    for document in documents { insert_document(&mut tree,&root,document)?; }
    Ok(tree)
}

/// Publish responsibility-owned schemas with exact native source/identity attribution.
/// # Errors
/// Invalid catalogue identities, schema objects or canonical identity framing.
pub fn generate_private_catalogue(catalogue: &[CatalogueDocument]) -> Result<GeneratedTree, SchemaError> {
    const ROOT: &str="scripts/generated/evidence_schema";
    validate_catalogue(catalogue)?;
    let identities=catalogue.iter().map(|entry|serde_json::json!({"name":entry.name,"slug":entry.slug,"direction":direction(entry),"version":entry.version,"wire":entry.wire,"schema":entry.schema})).collect::<Vec<_>>();
    let fingerprint=pse_ids::document::of(pse_ids::Frame::BoundarySchemaSetV1,&identities).map_err(|cause|error(cause.to_string()))?;
    let mut tree=GeneratedTree::empty(vec![PathBuf::from(ROOT)]);
    let mut rows=Vec::new();
    for entry in catalogue {
        let document=Document { name:format!("{}.{}",entry.slug,direction(entry)),schema:entry.schema.clone() };
        insert_document(&mut tree,ROOT,&document)?;
        let schema_path=format!("{ROOT}/{}.schema.json",document.name);
        rows.push(catalogue_row(entry,&schema_path,&tree.files[Path::new(&schema_path)]));
        catalogue_relationships(&mut tree,entry,fingerprint,&schema_path,&format!("{ROOT}/catalogue.json"),"private_catalogue","generate_private_catalogue");
    }
    rows.sort_by(|left,right|(&left["slug"].as_str(),&left["direction"].as_str()).cmp(&(&right["slug"].as_str(),&right["direction"].as_str())));
    tree.files.insert(format!("{ROOT}/catalogue.json").into(),canonical(&serde_json::json!({"version":1,"schema_set_fingerprint":fingerprint.to_prefixed(),"documents":rows,"authoring":null}))?.into_bytes());
    Ok(tree)
}

fn direction(entry: &CatalogueDocument) -> &'static str { if entry.output {"output"} else {"input"} }
fn error(reason:impl Into<String>) -> SchemaError { SchemaError::Codegen { language:"documents",reason:reason.into() } }
fn local_name(name:&str) -> Result<(),SchemaError> {
    if name.is_empty() || !name.bytes().all(|byte|byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte,b'_'|b'-'|b'.')) || name=="." || name==".." { return Err(error("schema publication requires a local document name")); }
    Ok(())
}
fn insert_document(tree:&mut GeneratedTree,root:&str,document:&Document) -> Result<(),SchemaError> {
    local_name(&document.name)?;
    let path=PathBuf::from(format!("{root}/{}.schema.json",document.name));
    if tree.files.insert(path,canonical(&document.schema)?.into_bytes()).is_some() { return Err(error("duplicate schema document name")); }
    Ok(())
}
fn validate_catalogue(catalogue:&[CatalogueDocument]) -> Result<(),SchemaError> {
    let mut names=BTreeSet::new();
    let mut owners=BTreeMap::new();
    for entry in catalogue {
        local_name(&entry.slug)?;
        if entry.schema.get("title").and_then(Value::as_str)!=Some(entry.name.as_str()) { return Err(error(format!("catalogue root {} differs from its native owner title",entry.slug))); }
        if !names.insert((&entry.slug,entry.output)) { return Err(error("duplicate direction-qualified catalogue identity")); }
        let owner=(&entry.rust_type,entry.version,&entry.wire);
        if owners.insert(&entry.name,owner).is_some_and(|previous|previous!=owner) { return Err(error("conflicting native catalogue owner")); }
    }
    Ok(())
}
fn catalogue_row(entry:&CatalogueDocument,path:&str,bytes:&[u8]) -> Value {
    serde_json::json!({"name":entry.name,"exported_name":entry.name,"slug":entry.slug,"direction":direction(entry),"wire":entry.wire,"version":entry.version,"rust_type":entry.rust_type,"registration_source":entry.registration_source,"schema_path":path,"schema_sha256":hex::encode(Sha256::digest(bytes))})
}
fn catalogue_relationships(tree:&mut GeneratedTree,entry:&CatalogueDocument,fingerprint:pse_ids::ContentHash,path:&str,manifest:&str,kind:&str,symbol:&str) {
    for output_path in [path,manifest] {
        tree.relationships.push(super::provenance::SourceRelationship { declaration_key:entry.name.clone(),declaration_kind:kind.into(),contract_sections:Vec::new(),source_path:Some(entry.registration_source.clone()),source_symbol:Some(entry.rust_type.clone()),source_role:"catalogue_registration".into(),source_identity:Some(fingerprint.to_prefixed()),generator_path:"crates/pse-codegen/src/codegen/documents.rs".into(),generator_symbol:symbol.into(),output_path:output_path.into(),direction:Some(direction(entry).into()),basis:"declared".into() });
    }
}
fn canonical(schema:&Value) -> Result<String,SchemaError> {
    let mut value=schema.clone();
    value.as_object_mut().ok_or_else(||error("a document schema is not an object"))?.insert("$comment".into(),Value::String("@generated by pse-schema; do not edit".into()));
    value.sort_all_objects();
    let mut text=serde_json::to_string_pretty(&value).map_err(|cause|error(cause.to_string()))?;
    text.push('\n');
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_schema_publication_retains_original_directional_names_references_and_constraints() {
        let registry=crate::RegistryBuilder::new().build().unwrap();
        let schema=serde_json::json!({"title":"Request","type":"object","additionalProperties":false,"properties":{"mode":{"$ref":"#/$defs/Mode"}},"required":["mode"],"$defs":{"Mode":{"type":"string","enum":["ordinary","reference"]}}});
        let entries=[false,true].map(|output|CatalogueDocument {name:"Request".into(),slug:"request".into(),rust_type:"owner::Request".into(),registration_source:"owner/catalogue.rs".into(),output,version:Some(1),wire:"Native".into(),schema:if output { let mut value=schema.clone();value["$defs"]["Mode"]["enum"]=serde_json::json!(["completed"]);value } else {schema.clone()} });
        let tree=generate_catalogue(&registry,&entries,registry.fingerprint()).unwrap();
        assert_eq!(tree.roots,[PathBuf::from(SCHEMA_ROOT)]);
        for entry in &entries {
            let path=PathBuf::from(format!("{SCHEMA_ROOT}/request.{}.schema.json",direction(entry)));
            let mut published:Value=serde_json::from_slice(&tree.files[&path]).unwrap();
            published.as_object_mut().unwrap().remove("$comment");
            assert_eq!(published,entry.schema);
            assert_eq!(published["title"],"Request");
            assert_eq!(published["properties"]["mode"]["$ref"],"#/$defs/Mode");
        }
        assert!(tree.files.keys().all(|path|path.extension().is_some_and(|extension|extension=="json")));
        assert!(tree.relationships.iter().all(|record|tree.files.contains_key(Path::new(&record.output_path))));
        assert_eq!(tree,generate_catalogue(&registry,&entries,registry.fingerprint()).unwrap());
        assert!(generate_catalogue(&registry,&[entries[0].clone(),entries[0].clone()],registry.fingerprint()).is_err());
    }
    #[test]
    fn private_schema_families_publish_native_json_without_runtime_or_executable_types() {
        let registry=crate::RegistryBuilder::new().build().unwrap();
        let document=Document {name:"request".into(),schema:serde_json::json!({"title":"Request","type":"object","properties":{"values":{"type":"array","items":{"type":"integer","minimum":0}}},"required":["values"]})};
        let tree=generate_private_family(&registry,"deployment",std::slice::from_ref(&document)).unwrap();
        assert_eq!(tree.files.len(),1);
        let path=PathBuf::from("scripts/generated/deployment_schema/request.schema.json");
        let mut published:Value=serde_json::from_slice(&tree.files[&path]).unwrap();
        published.as_object_mut().unwrap().remove("$comment");
        assert_eq!(published,document.schema);
        assert!(generate_private_family(&registry,"../escape",&[document.clone()]).is_err());
        assert!(generate_private(&registry,&[document.clone(),document]).is_err());
    }
}
