// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Complete standard-catalog products, emitted from the independently built source owner.
use super::{emit, error};
use crate::{Registry, SchemaError, model::*};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::{BTreeMap, BTreeSet};

fn id(value: pse_ids::SemanticId) -> TokenStream {
    let bytes = proc_macro2::Literal::byte_string(value.as_bytes());
    quote!(pse_ids::SemanticId::from_bytes(*#bytes))
}
fn hash(value: pse_ids::ContentHash) -> TokenStream {
    let bytes = proc_macro2::Literal::byte_string(value.as_bytes());
    quote!(pse_ids::ContentHash::from_bytes(*#bytes))
}
fn unit(name: &str, value: impl std::fmt::Debug) -> Result<TokenStream, SchemaError> {
    let ty = syn::parse_str::<syn::Path>(name).map_err(|e| error(e.to_string()))?;
    let variant =
        syn::parse_str::<syn::Ident>(&format!("{value:?}")).map_err(|e| error(e.to_string()))?;
    Ok(quote!(#ty::#variant))
}
fn option<T>(value: Option<T>, f: impl FnOnce(T) -> TokenStream) -> TokenStream {
    value.map_or_else(
        || quote!(None),
        |v| {
            let v = f(v);
            quote!(Some(#v))
        },
    )
}
fn optstr(value: Option<&str>) -> TokenStream {
    option(value, |v| quote!(#v))
}
fn strings(values: &[String]) -> TokenStream {
    quote!(vec![#(#values.into()),*])
}
fn texts(values: &[&str]) -> TokenStream {
    quote!(vec![#(#values),*])
}
fn ids(values: impl IntoIterator<Item = pse_ids::SemanticId>) -> Vec<TokenStream> {
    values.into_iter().map(id).collect()
}
// A Vec tracks its initialized prefix at runtime. In contrast, a large array
// literal containing fallible constructors gives each partial prefix its own
// cleanup graph before the owned collection is even constructed.
fn vector(values: impl IntoIterator<Item = TokenStream>) -> TokenStream {
    let values = values.into_iter().collect::<Vec<_>>();
    let count = values.len();
    if count == 0 {
        return quote!(Vec::new());
    }
    quote!({let mut values=Vec::with_capacity(#count); #(values.push(#values);)* values})
}
fn collection(values: impl IntoIterator<Item = TokenStream>) -> TokenStream {
    let values = values.into_iter().collect::<Vec<_>>();
    if values.is_empty() {
        return quote!(Default::default());
    }
    let values = vector(values);
    quote!((#values).into_iter().collect())
}
fn map(values: &BTreeMap<String, String>) -> TokenStream {
    if values.is_empty() {
        return quote!(std::collections::BTreeMap::new());
    }
    collection(values.iter().map(|(k, v)| quote!((#k.into(),#v.into()))))
}
#[derive(Default)]
struct NativeJsonPool {
    functions: std::cell::RefCell<Vec<TokenStream>>,
    text: std::cell::RefCell<String>,
    offsets: std::cell::RefCell<BTreeMap<String, (usize, usize)>>,
}
fn outlined(pool: &NativeJsonPool, ty: TokenStream, value: TokenStream) -> TokenStream {
    let mut functions = pool.functions.borrow_mut();
    let name = format_ident!("resolved_product_{}", functions.len());
    functions.push(quote! {
        #[inline(never)]
        fn #name()->Result<#ty,crate::SchemaError>{Ok(#value)}
    });
    quote!(#name()?)
}
fn pooled(pool: &NativeJsonPool, text: String) -> (usize, usize) {
    let existing = pool.offsets.borrow().get(&text).copied();
    let (start, end) = if let Some(range) = existing {
        range
    } else {
        let mut bytes = pool.text.borrow_mut();
        let start = bytes.len();
        bytes.push_str(&text);
        let end = bytes.len();
        bytes.push('\n');
        pool.offsets.borrow_mut().insert(text, (start, end));
        (start, end)
    };
    (start, end)
}
fn json(pool: &NativeJsonPool, value: &impl serde::Serialize) -> Result<TokenStream, SchemaError> {
    let text =
        pse_columnar::native_field::canonical_json(value).map_err(|e| error(e.to_string()))?;
    let (start, end) = pooled(pool, text);
    Ok(quote!(native(#start,#end)?))
}
fn field(pool: &NativeJsonPool, value: &FieldContract) -> Result<TokenStream, SchemaError> {
    let native = json(pool, value.field())?;
    Ok(quote!(FieldContract::from_field(#native)))
}
fn fields(pool: &NativeJsonPool, values: &[FieldContract]) -> Result<TokenStream, SchemaError> {
    let fields = values
        .iter()
        .map(|v| field(pool, v))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(vector(fields))
}
fn key(value: RelationKey) -> Result<TokenStream, SchemaError> {
    let ns = unit("Namespace", value.namespace)?;
    let name = value.name;
    let version = value.version;
    Ok(quote!(RelationKey::new(#ns,#name,#version)))
}
fn relation(pool: &NativeJsonPool, value: &RelationSpec) -> Result<TokenStream, SchemaError> {
    let id = id(value.id);
    let key = key(value.key)?;
    let authority = unit("Authority", value.authority)?;
    let snapshot_class = unit("SnapshotClass", value.snapshot_class)?;
    let stability = unit("Stability", value.stability)?;
    let granularity = value
        .derivation_granularity
        .map(|v| unit("DerivationGranularity", v))
        .transpose()?;
    let granularity = option(granularity, |v| v);
    let primary_key = texts(&value.primary_key);
    let columns = fields(pool, &value.columns)?;
    let checks = map(&value.checks);
    let unique = value.unique_keys.iter().map(|v| {
        let name = v.name;
        let columns = texts(&v.columns);
        quote!(UniqueKey {name:#name,columns:#columns})
    });
    let foreign = value.foreign_keys.iter().map(|v| {
        let name = v.name;
        let columns = texts(&v.columns);
        let target = v.target;
        let targets = texts(&v.target_columns);
        quote!(ForeignKeyDecl {name:#name,columns:#columns,target:#target,target_columns:#targets})
    });
    let doc = value.doc;
    let fingerprint = hash(value.fingerprint);
    Ok(outlined(
        pool,
        quote!(RelationSpec),
        quote!(RelationSpec {id:#id,key:#key,authority:#authority,snapshot_class:#snapshot_class,
        derivation_granularity:#granularity,stability:#stability,primary_key:#primary_key,columns:#columns,
        checks:#checks,unique_keys:vec![#(#unique),*],foreign_keys:vec![#(#foreign),*],doc:#doc,fingerprint:#fingerprint}),
    ))
}
fn enumeration(pool: &NativeJsonPool, v: &EnumSpec) -> TokenStream {
    let id = id(v.id);
    let name = v.name;
    let source = optstr(v.source);
    let idaes = optstr(v.idaes_source);
    let default = optstr(v.default_member);
    let members = v.members.iter().map(|m| {
        let name = m.name;
        let idaes = optstr(m.idaes_name);
        let deprecated = m.deprecated;
        let doc = m.doc;
        quote!(EnumMember{name:#name,idaes_name:#idaes,deprecated:#deprecated,doc:#doc})
    });
    outlined(
        pool,
        quote!(EnumSpec),
        quote!(EnumSpec{id:#id,name:#name,idaes_source:#idaes,source:#source,default_member:#default,members:vec![#(#members),*]}),
    )
}
fn invariant(pool: &NativeJsonPool, v: &InvariantSpec) -> Result<TokenStream, SchemaError> {
    let origin = match v.origin() {
        InvariantOrigin::AuthoredQuery => quote!(InvariantOrigin::AuthoredQuery),
        InvariantOrigin::GeneratedIntegrity(binding) => {
            let relation = key(binding.relation)?;
            let derivation = match &binding.derivation {
                IntegrityDerivation::PrimaryKey => quote!(IntegrityDerivation::PrimaryKey),
                IntegrityDerivation::UniqueKey(v) => {
                    quote!(IntegrityDerivation::UniqueKey(#v.into()))
                }
                IntegrityDerivation::TableReference(v) => {
                    quote!(IntegrityDerivation::TableReference(#v.into()))
                }
                IntegrityDerivation::ReferenceOccurrence(v) => {
                    let v = strings(v);
                    quote!(IntegrityDerivation::ReferenceOccurrence(#v))
                }
                IntegrityDerivation::OrdinalOccurrence(v) => {
                    let v = strings(v);
                    quote!(IntegrityDerivation::OrdinalOccurrence(#v))
                }
            };
            quote!(InvariantOrigin::GeneratedIntegrity(IntegrityBinding {relation:#relation,derivation:#derivation}))
        }
    };
    let id = id(v.id);
    let name = &v.name;
    let relation = &v.relation;
    let kind = unit("InvariantKind", v.kind)?;
    let query = &v.query;
    let inputs = strings(&v.inputs);
    let keys = texts(&v.key_columns);
    let severity = unit("Severity", v.severity)?;
    let doc = v.doc;
    Ok(outlined(
        pool,
        quote!(InvariantSpec),
        quote!(InvariantSpec {origin:#origin,id:#id,name:#name.into(),relation:#relation.into(),kind:#kind,
        query:#query.into(),inputs:#inputs,key_columns:#keys,severity:#severity,doc:#doc}),
    ))
}
fn literal(pool: &NativeJsonPool, v: &crate::NativeLiteral) -> Result<TokenStream, SchemaError> {
    let field = json(pool, v.field().as_ref())?;
    let text = v.as_json();
    Ok(quote!(crate::NativeLiteral::from_json(std::sync::Arc::new(#field),#text)?))
}
fn migration_mapping(
    pool: &NativeJsonPool,
    v: &MigrationValues,
) -> Result<TokenStream, SchemaError> {
    let nulls = unit("MigrationNulls", v.nulls)?;
    let policy = match &v.policy {
        MappingPolicy::Identity => quote!(MappingPolicy::Identity),
        MappingPolicy::Finite(rows) => {
            let rows = rows
                .iter()
                .map(|row| {
                    let source = row
                        .source
                        .iter()
                        .map(|v| literal(pool, v))
                        .collect::<Result<Vec<_>, _>>()?;
                    let target = row
                        .target
                        .iter()
                        .map(|v| literal(pool, v))
                        .collect::<Result<Vec<_>, _>>()?;
                    let source = vector(source);
                    let target = vector(target);
                    Ok(quote!(MigrationValuePair{source:#source,target:#target}))
                })
                .collect::<Result<Vec<_>, SchemaError>>()?;
            let rows = vector(rows);
            quote!(MappingPolicy::Finite(#rows))
        }
    };
    Ok(quote!(MigrationValues{policy:#policy,nulls:#nulls}))
}
fn migration(pool: &NativeJsonPool, v: &MigrationSpec) -> Result<TokenStream, SchemaError> {
    let steps = v
        .steps
        .iter()
        .map(|step| {
            Ok(match step {
                MigrationStep::AddColumn { name, default } => {
                    let default = literal(pool, default)?;
                    quote!(MigrationStep::AddColumn{name:#name,default:#default})
                }
                MigrationStep::DropColumn(name) => quote!(MigrationStep::DropColumn(#name)),
                MigrationStep::RenameColumn { from, to } => {
                    quote!(MigrationStep::RenameColumn{from:#from,to:#to})
                }
                MigrationStep::ChangeNullable { name, nullable } => {
                    quote!(MigrationStep::ChangeNullable{name:#name,nullable:#nullable})
                }
                MigrationStep::RecodeDomain { name, mapping } => {
                    let mapping = migration_mapping(pool, mapping)?;
                    quote!(MigrationStep::RecodeDomain{name:#name,mapping:#mapping})
                }
                MigrationStep::MapReferenceKey { columns, mapping } => {
                    let columns = texts(columns);
                    let mapping = migration_mapping(pool, mapping)?;
                    quote!(MigrationStep::MapReferenceKey{columns:#columns,mapping:#mapping})
                }
            })
        })
        .collect::<Result<Vec<_>, SchemaError>>()?;
    let relation = v.relation;
    let from = v.from_version;
    let to = v.to_version;
    let doc = v.doc;
    let steps = vector(steps);
    Ok(outlined(
        pool,
        quote!(MigrationSpec),
        quote!(MigrationSpec{relation:#relation,from_version:#from,to_version:#to,steps:#steps,doc:#doc}),
    ))
}
fn algorithm(pool: &NativeJsonPool, v: &AlgorithmSpec) -> Result<TokenStream, SchemaError> {
    let id = id(v.id);
    let name = v.name;
    let version = v.version;
    let inputs=v.inputs.iter().map(|i|{let port=&i.port;let relation=&i.relation;let required=i.required;
        let consumption=match &i.consumption {algorithm::InputConsumption::Whole=>quote!(algorithm::InputConsumption::Whole),
            algorithm::InputConsumption::Columns(v)=>{let v=collection(v.iter().map(|v|quote!(#v.into())));quote!(algorithm::InputConsumption::Columns(#v))}};
        quote!(ArgumentSpec{port:#port.into(),relation:#relation.into(),required:#required,consumption:#consumption})});
    let outputs = v.outputs.iter().map(|v| {
        let port = &v.port;
        let relation = &v.relation;
        quote!(ResultSpec{port:#port.into(),relation:#relation.into()})
    });
    let pre = strings(&v.preconditions);
    let post = strings(&v.postconditions);
    let determinism = unit("Determinism", v.determinism)?;
    let diagnostics = texts(&v.diagnostics);
    let effects = v
        .effects
        .iter()
        .map(|e| unit("provider::OperationEffect", e))
        .collect::<Result<Vec<_>, _>>()?;
    let effects = collection(effects);
    Ok(outlined(
        pool,
        quote!(AlgorithmSpec),
        quote!(AlgorithmSpec{id:#id,name:#name,version:#version,inputs:vec![#(#inputs),*],outputs:vec![#(#outputs),*],
        preconditions:#pre,postconditions:#post,determinism:#determinism,diagnostics:#diagnostics,effects:#effects}),
    ))
}
fn document(pool: &NativeJsonPool, v: &DocumentSpec) -> Result<TokenStream, SchemaError> {
    let name = v.name;
    let kind = unit("DocumentKind", v.kind)?;
    let path = v.path_glob;
    let doc = v.doc;
    let sections=v.sections.iter().map(|s|{let key=s.key;let relation=s.relation;let repeated=s.repeated;let id=optstr(s.identity_column);
        let entity=optstr(s.entity_kind);let name=optstr(s.name_column);let scope=optstr(s.naming_scope_column);let doc=s.doc;
        quote!(DocumentSection{key:#key,relation:#relation,repeated:#repeated,identity_column:#id,entity_kind:#entity,name_column:#name,naming_scope_column:#scope,doc:#doc})});
    Ok(outlined(
        pool,
        quote!(DocumentSpec),
        quote!(DocumentSpec{name:#name,kind:#kind,path_glob:#path,sections:vec![#(#sections),*],doc:#doc}),
    ))
}
fn obligations(
    pool: &NativeJsonPool,
    v: &crate::obligations::RelationObligations,
) -> Result<TokenStream, SchemaError> {
    let value = &v.value_column;
    let pk = texts(&v.primary_key);
    let checks = map(&v.checks);
    let refs=v.references.iter().map(|r|{let path=strings(&r.path);let reference=json(pool,&r.reference)?;let input=&r.input;
        Ok(quote!(crate::obligations::ReferenceObligation{path:#path,reference:#reference,input:#input.into()}))}).collect::<Result<Vec<_>,SchemaError>>()?;
    let ordinals=v.ordinals.iter().map(|r|{let path=strings(&r.path);let target=&r.target;let input=&r.input;
        quote!(crate::obligations::OrdinalObligation{path:#path,target:#target.into(),input:#input.into()})});
    let refs = vector(refs);
    Ok(outlined(
        pool,
        quote!(crate::obligations::RelationObligations),
        quote!(crate::obligations::RelationObligations{value_column:#value.into(),primary_key:#pk,references:#refs,ordinals:vec![#(#ordinals),*],checks:#checks}),
    ))
}
fn expected(v: crate::resolved_contract::ExpectedContract) -> TokenStream {
    let relation = id(v.relation);
    let version = v.version;
    let semantic = v.semantic_version;
    let semantics = hash(v.semantics);
    let encoding = hash(v.encoding);
    quote!(crate::resolved_contract::ExpectedContract{relation:#relation,version:#version,semantic_version:#semantic,semantics:#semantics,encoding:#encoding})
}
fn set(v: &BTreeSet<pse_ids::SemanticId>) -> TokenStream {
    let v = ids(v.iter().copied());
    quote!(semantic_set(&[#(#v),*]))
}

pub(super) fn generate(
    reg: &Registry,
    tree: &mut super::super::GeneratedTree,
) -> Result<(), SchemaError> {
    let p = reg.resolved_products()?;
    // These are the two actual products assembled below; their declarations are
    // semantic keys because the admitted registry does not retain source paths.
    for output in [
        "crates/pse-schema/src/generated/standard_registry.rs",
        "crates/pse-schema/src/generated/standard-data.jsonl",
    ] {
        for relation in reg.relations() {
            tree.relationships
                .push(crate::codegen::provenance::SourceRelationship::registry(
                    reg,
                    "relation",
                    crate::codegen::provenance::relation_key(relation),
                    "crates/pse-codegen/src/codegen/rust/registry.rs",
                    "generate",
                    output,
                ));
        }
        // DocumentSpec values are outlined Rust constructors, not JSON-pool entries.
        for document in reg.documents().iter().filter(|_| output.ends_with("standard_registry.rs")) {
            crate::codegen::provenance::document(
                tree,
                reg,
                document,
                "crates/pse-codegen/src/codegen/rust/registry.rs",
                "generate",
                output,
            );
        }
    }
    let pool = NativeJsonPool::default();
    let relations = p
        .relations
        .iter()
        .map(|v| relation(&pool, v))
        .collect::<Result<Vec<_>, _>>()?;
    let enums = p
        .enums
        .iter()
        .map(|v| enumeration(&pool, v))
        .collect::<Vec<_>>();
    let identities=p.identities.iter().map(|v|{let identity=id(v.id);let name=v.name;let doc=v.doc;let base=unit("IdentityBase",v.base)?;
        let owner=option(v.owner.as_ref(),|o|{let id=id(o.relation_id);let relation=&o.relation;let column=&o.column;
            quote!(IdentityOwner{relation_id:#id,relation:#relation.into(),column:#column.into()})});
        Ok(quote!(IdentitySpec{id:#identity,name:#name,doc:#doc,base:#base,owner:#owner}))}).collect::<Result<Vec<_>,SchemaError>>()?;
    let structures = p
        .structures
        .iter()
        .map(|(k, v)| {
            let v = field(&pool, v)?;
            Ok(quote!((#k.into(),#v)))
        })
        .collect::<Result<Vec<_>, SchemaError>>()?;
    let logical=p.logical_types.iter().map(|v|{let id=id(v.id);let name=&v.name;let (start,end)=pooled(&pool,v.arrow_storage.clone());
        let extension=option(v.extension_name.as_ref(),|v|quote!(#v.into()));let metadata=option(v.metadata_schema.as_ref(),|v|quote!(#v.into()));
        outlined(&pool,quote!(FieldTypeRow),quote!(FieldTypeRow{id:#id,name:#name.into(),arrow_storage:native_text(#start,#end)?.to_owned(),extension_name:#extension,metadata_schema:#metadata}))}).collect::<Vec<_>>();
    let storage = p
        .logical_storage
        .iter()
        .map(|v| json(&pool, v))
        .collect::<Result<Vec<_>, _>>()?;
    let invariants = p
        .invariants
        .iter()
        .map(|v| invariant(&pool, v))
        .collect::<Result<Vec<_>, _>>()?;
    let migrations = p
        .migrations
        .iter()
        .map(|v| migration(&pool, v))
        .collect::<Result<Vec<_>, _>>()?;
    let algorithms = p
        .algorithms
        .iter()
        .map(|v| algorithm(&pool, v))
        .collect::<Result<Vec<_>, _>>()?;
    let documents = p
        .documents
        .iter()
        .map(|v| document(&pool, v))
        .collect::<Result<Vec<_>, _>>()?;
    let profiles = p.artifact_profiles.iter().map(|(k, v)| {
        let v = set(v);
        quote!((#k.into(),#v))
    });
    let obligations = p
        .obligations
        .iter()
        .map(|(k, v)| {
            let k = key(*k)?;
            let v = obligations(&pool, v)?;
            Ok(quote!((#k,#v)))
        })
        .collect::<Result<Vec<_>, SchemaError>>()?;
    let graph=p.contracts.relations.iter().map(|v|{let declaration=relation(&pool,&v.declaration)?;let fields=fields(&pool,&v.fields)?;
        let references=ids(v.references.iter().copied());let enums=ids(v.enums.iter().copied());let extensions=strings(&v.extensions);
        Ok(outlined(&pool,quote!(crate::resolved_contract::ResolvedRelationContract),quote!(crate::resolved_contract::ResolvedRelationContract{declaration:#declaration,fields:#fields,references:vec![#(#references),*],enums:vec![#(#enums),*],extensions:#extensions})))
    }).collect::<Result<Vec<_>,SchemaError>>()?;
    let graph_enums = p
        .contracts
        .enums
        .iter()
        .map(|v| enumeration(&pool, v))
        .collect::<Vec<_>>();
    let extensions=p.contracts.extensions.iter().map(|v|{let name=&v.name;let storage=field(&pool,&v.storage)?;let metadata=&v.metadata_kind;let version=v.version;
        Ok(quote!(crate::resolved_contract::ResolvedExtensionContract{name:#name.into(),storage:#storage,metadata_kind:#metadata.into(),version:#version}))}).collect::<Result<Vec<_>,SchemaError>>()?;
    let schemas = p
        .contracts
        .schemas
        .iter()
        .map(|(k, v)| {
            let id = id(*k);
            let schema = json(&pool, v.as_ref())?;
            Ok(quote!((#id,std::sync::Arc::new(#schema))))
        })
        .collect::<Result<Vec<_>, SchemaError>>()?;
    let expectations = p.contracts.expectations.iter().map(|(k, v)| {
        let k = id(*k);
        let v = expected(*v);
        quote!((#k,#v))
    });
    let descriptions = p
        .semantics
        .descriptions
        .iter()
        .map(|(k, v)| {
            let k = id(*k);
            let v = json(&pool, v)?;
            Ok(quote!((#k,#v)))
        })
        .collect::<Result<Vec<_>, SchemaError>>()?;
    let closures = p
        .semantics
        .closures
        .iter()
        .map(|(roots, closure, identity)| {
            let roots = set(roots);
            let closure = set(closure);
            let identity = hash(*identity);
            outlined(
                &pool,
                quote!((
                    std::collections::BTreeSet<pse_ids::SemanticId>,
                    std::collections::BTreeSet<pse_ids::SemanticId>,
                    pse_ids::ContentHash
                )),
                quote!((#roots,#closure,#identity)),
            )
        })
        .collect::<Vec<_>>();
    let invariant_index = p
        .semantics
        .invariants
        .iter()
        .map(|(k, v)| quote!((#k.into(),vec![#(#v),*])));
    let package = id(p.package_id);
    let fingerprint = hash(p.fingerprint);
    let mut batches = Vec::new();
    for (index, (k, batch)) in reg.schema_batches().iter().enumerate() {
        let mut bytes = Vec::new();
        {
            let mut writer =
                arrow_ipc::writer::StreamWriter::try_new(&mut bytes, batch.schema().as_ref())
                    .map_err(|e| error(e.to_string()))?;
            writer.write(batch).map_err(|e| error(e.to_string()))?;
            writer.finish().map_err(|e| error(e.to_string()))?;
        }
        let filename = format!("standard-{index}.arrow");
        tree.files.insert(
            format!("crates/pse-schema/src/generated/{filename}").into(),
            bytes,
        );
        let k = key(*k)?;
        batches.push(quote!((#k,batch(include_bytes!(#filename))?)));
    }
    // Each declaration/family has an independently checked concrete return type.
    // The standard constructor itself never owns a model-sized expression tree.
    let relations = outlined(&pool, quote!(Vec<RelationSpec>), vector(relations));
    let obligations = outlined(
        &pool,
        quote!(std::collections::BTreeMap<RelationKey,crate::obligations::RelationObligations>),
        collection(obligations),
    );
    let enums = outlined(&pool, quote!(Vec<EnumSpec>), vector(enums));
    let identities = outlined(&pool, quote!(Vec<IdentitySpec>), vector(identities));
    let structures = outlined(
        &pool,
        quote!(std::collections::BTreeMap<String,FieldContract>),
        collection(structures),
    );
    let logical = outlined(&pool, quote!(Vec<FieldTypeRow>), vector(logical));
    let storage = outlined(&pool, quote!(Vec<arrow_schema::DataType>), vector(storage));
    let invariants = outlined(&pool, quote!(Vec<InvariantSpec>), vector(invariants));
    let migrations = outlined(&pool, quote!(Vec<MigrationSpec>), vector(migrations));
    let algorithms = outlined(&pool, quote!(Vec<AlgorithmSpec>), vector(algorithms));
    let documents = outlined(&pool, quote!(Vec<DocumentSpec>), vector(documents));
    let profiles = outlined(
        &pool,
        quote!(std::collections::BTreeMap<String,std::collections::BTreeSet<pse_ids::SemanticId>>),
        collection(profiles),
    );
    let graph = outlined(
        &pool,
        quote!(Vec<crate::resolved_contract::ResolvedRelationContract>),
        vector(graph),
    );
    let graph_enums = outlined(&pool, quote!(Vec<EnumSpec>), vector(graph_enums));
    let extensions = outlined(
        &pool,
        quote!(Vec<crate::resolved_contract::ResolvedExtensionContract>),
        vector(extensions),
    );
    let schemas = outlined(
        &pool,
        quote!(std::collections::BTreeMap<pse_ids::SemanticId,arrow_schema::SchemaRef>),
        collection(schemas),
    );
    let expectations = outlined(
        &pool,
        quote!(std::collections::BTreeMap<pse_ids::SemanticId,crate::resolved_contract::ExpectedContract>),
        collection(expectations),
    );
    let contracts = outlined(
        &pool,
        quote!(crate::resolved_contract::ResolvedContractProducts),
        quote!(crate::resolved_contract::ResolvedContractProducts{
        relations:#graph,enums:#graph_enums,extensions:#extensions,
        schemas:#schemas,expectations:#expectations}),
    );
    let descriptions = outlined(
        &pool,
        quote!(std::collections::BTreeMap<pse_ids::SemanticId,serde_json::Value>),
        collection(descriptions),
    );
    let closures = outlined(
        &pool,
        quote!(Vec<ResolvedSemanticClosure>),
        vector(closures),
    );
    let invariant_index = outlined(
        &pool,
        quote!(std::collections::BTreeMap<String,Vec<usize>>),
        collection(invariant_index),
    );
    let semantics = outlined(
        &pool,
        quote!(crate::fingerprint::ResolvedSemanticProducts),
        quote!(crate::fingerprint::ResolvedSemanticProducts{
        descriptions:#descriptions,closures:#closures,invariants:#invariant_index}),
    );
    let batches = outlined(
        &pool,
        quote!(Vec<(RelationKey, arrow_array::RecordBatch)>),
        vector(batches),
    );
    let functions = pool.functions.into_inner();
    tree.files.insert(
        "crates/pse-schema/src/generated/standard-data.jsonl".into(),
        pool.text.into_inner().into_bytes(),
    );
    emit(
        tree,
        "crates/pse-schema/src/generated/mod.rs",
        quote!(
            pub(crate) mod standard_registry;
        ),
    )?;
    emit(
        tree,
        "crates/pse-schema/src/generated/standard_registry.rs",
        quote! {
            #![allow(
                clippy::vec_init_then_push,
                reason = "sequential generated hydration bounds fallible-constructor cleanup graphs; array-backed vec literals exhausted the compilation memory cap"
            )]
            use crate::model::*;
            use crate::builder::products::{batch,decode};
            /// Declared roots, complete support closure and original framed identity.
            type ResolvedSemanticClosure = (
                std::collections::BTreeSet<pse_ids::SemanticId>,
                std::collections::BTreeSet<pse_ids::SemanticId>,
                pse_ids::ContentHash
            );
            const DATA:&str=include_str!("standard-data.jsonl");
            fn native_text(start:usize,end:usize)->Result<&'static str,crate::SchemaError>{
                DATA.get(start..end).ok_or_else(||crate::checks::invalid("standard registry products","native JSON slice outside generated data"))
            }
            fn native<T:serde::de::DeserializeOwned>(start:usize,end:usize)->Result<T,crate::SchemaError>{
                decode(native_text(start,end)?)
            }
            // Every declared closure uses the same iterator implementation rather
            // than a separately optimized array iterator for each const length.
            #[inline(never)]
            fn semantic_set(values:&[pse_ids::SemanticId])->std::collections::BTreeSet<pse_ids::SemanticId>{
                values.iter().copied().collect()
            }
            #(#functions)*
            pub(crate) fn hydrate()->Result<crate::Registry,crate::SchemaError>{
                let products=crate::builder::ResolvedRegistryProducts{
                    package_id:#package,fingerprint:#fingerprint,relations:#relations,
                    obligations:#obligations,enums:#enums,identities:#identities,
                    structures:#structures,logical_types:#logical,logical_storage:#storage,
                    invariants:#invariants,migrations:#migrations,algorithms:#algorithms,documents:#documents,
                    artifact_profiles:#profiles,contracts:#contracts,semantics:#semantics,
                };
                products.hydrate(#batches)
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_json_pool_is_canonical_deduplicated_and_preserves_utf8_boundaries() {
        let pool = NativeJsonPool::default();
        let mut left = std::collections::HashMap::new();
        left.insert("z".into(), "β".into());
        left.insert("a".into(), "first".into());
        let mut right = std::collections::HashMap::new();
        right.insert("a".into(), "first".into());
        right.insert("z".into(), "β".into());
        let left =
            arrow_schema::Field::new("é", arrow_schema::DataType::Utf8, true).with_metadata(left);
        let right =
            arrow_schema::Field::new("é", arrow_schema::DataType::Utf8, true).with_metadata(right);
        assert_eq!(
            json(&pool, &left).unwrap().to_string(),
            json(&pool, &right).unwrap().to_string()
        );
        assert_eq!(pool.offsets.borrow().len(), 1);
        let text = pool.text.borrow();
        for (expected, (start, end)) in pool.offsets.borrow().iter() {
            assert_eq!(text.get(*start..*end).unwrap(), expected);
            let field: arrow_schema::Field =
                serde_json::from_str(text.get(*start..*end).unwrap()).unwrap();
            assert_eq!(field, left);
        }
    }
    #[test]
    fn independently_assembled_standard_products_generate_identical_native_data() {
        let first = crate::catalog::assemble().unwrap();
        let second = crate::catalog::assemble().unwrap();
        let mut a = super::super::super::GeneratedTree::default();
        let mut b = super::super::super::GeneratedTree::default();
        generate(&first, &mut a).unwrap();
        generate(&second, &mut b).unwrap();
        assert_eq!(
            a.files, b.files,
            "all native metadata and IPC data must be independent of HashMap seed"
        );
        let rust =
            &a.files[std::path::Path::new("crates/pse-schema/src/generated/standard_registry.rs")];
        let data =
            &a.files[std::path::Path::new("crates/pse-schema/src/generated/standard-data.jsonl")];
        assert!(std::str::from_utf8(rust).unwrap().contains("include_str!"));
        assert!(!data.is_empty());
    }
}
