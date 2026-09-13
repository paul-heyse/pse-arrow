//! dfarrow-apiex — extract a source-grounded API corpus from rustdoc JSON.
//!
//! Hard rule: `document.format_version` must equal `rustdoc_types::FORMAT_VERSION`.
//! A mismatched document may still deserialize into *something*, silently and
//! wrongly, so the version is sniffed before the typed parse and we fail closed.

mod render;
mod routes;

use anyhow::{bail, Context, Result};
use rustdoc_types::*;
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Path as FsPath, PathBuf};

const DECL_SCHEMA: &str = "library-guides.declaration.v1";
const TRAIT_SCHEMA: &str = "dfarrow.trait_contract.v1";
const IMPL_SCHEMA: &str = "dfarrow.impl_relation.v1";

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(String::as_str).unwrap_or("help");
    match cmd {
        "gate" => {
            let p = req(&args, 2, "gate <rustdoc.json>")?;
            let (fv, _) = sniff(FsPath::new(&p))?;
            println!("{p}: format_version={fv} expected={FORMAT_VERSION}");
            if fv != FORMAT_VERSION {
                bail!("format_version mismatch: {fv} != {FORMAT_VERSION}");
            }
            Ok(())
        }
        "normalize" => {
            let input = req(&args, 2, "normalize <rustdoc.json> <profile> <out-dir>")?;
            let profile = req(&args, 3, "normalize <rustdoc.json> <profile> <out-dir>")?;
            let outdir = req(&args, 4, "normalize <rustdoc.json> <profile> <out-dir>")?;
            normalize(FsPath::new(&input), &profile, FsPath::new(&outdir))
        }
        _ => {
            eprintln!(
                "dfarrow-apiex <command>\n\
                 \n\
                   gate <rustdoc.json>                          verify format_version\n\
                   normalize <rustdoc.json> <profile> <outdir>  emit api/traits/impls records\n"
            );
            Ok(())
        }
    }
}

fn req(a: &[String], i: usize, usage: &str) -> Result<String> {
    a.get(i).cloned().with_context(|| format!("usage: dfarrow-apiex {usage}"))
}

/// Read `format_version` without committing to the typed model.
fn sniff(p: &FsPath) -> Result<(u32, String)> {
    let raw = fs::read_to_string(p).with_context(|| format!("reading {}", p.display()))?;
    let v: Value = serde_json::from_str(&raw).with_context(|| format!("parsing {}", p.display()))?;
    let fv = v
        .get("format_version")
        .and_then(Value::as_u64)
        .context("document has no format_version")? as u32;
    Ok((fv, raw))
}

fn digest(s: &str) -> String {
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    format!("{:x}", h.finalize())
}

#[derive(Serialize)]
struct Decl<'a> {
    schema: &'static str,
    profile: &'a str,
    id: String,
    artifact: String,
    local_id: String,
    symbol: String,
    kind: &'static str,
    owner: Option<String>,
    owner_contract: Option<Value>,
    public_routes: Vec<String>,
    route_state: &'static str,
    signature: Option<String>,
    type_occurrences: Vec<Value>,
    deprecated: Option<Value>,
    availability_conditions: Vec<String>,
    docs: Option<&'a str>,
    span: Option<Value>,
    semantic_digest: String,
}

fn kind_of(inner: &ItemEnum, in_assoc: bool) -> &'static str {
    match inner {
        ItemEnum::Module(_) => "module",
        ItemEnum::ExternCrate { .. } => "extern_crate",
        ItemEnum::Use(_) => "use",
        ItemEnum::Union(_) => "union",
        ItemEnum::Struct(_) => "struct",
        ItemEnum::StructField(_) => "struct_field",
        ItemEnum::Enum(_) => "enum",
        ItemEnum::Variant(_) => "variant",
        ItemEnum::Function(_) => if in_assoc { "method" } else { "function" },
        ItemEnum::Trait(_) => "trait",
        ItemEnum::TraitAlias(_) => "trait_alias",
        ItemEnum::Impl(_) => "impl",
        ItemEnum::TypeAlias(_) => "type_alias",
        ItemEnum::Constant { .. } => "constant",
        ItemEnum::Static(_) => "static",
        ItemEnum::ExternType => "extern_type",
        ItemEnum::Macro(_) => "macro",
        ItemEnum::ProcMacro(_) => "proc_macro",
        ItemEnum::Primitive(_) => "primitive",
        ItemEnum::AssocConst { .. } => "assoc_const",
        ItemEnum::AssocType { .. } => "assoc_type",
    }
}

fn signature_of(item: &Item, in_assoc: bool) -> Option<String> {
    let name = item.name.as_deref().unwrap_or("_");
    match &item.inner {
        ItemEnum::Function(f) => Some(render::desugar_async_trait(&render::function(name, f))),
        ItemEnum::Trait(t) => Some(render::trait_header(name, t)),
        ItemEnum::Impl(i) => Some(render::impl_header(i)),
        ItemEnum::StructField(t) => Some(format!("{name}: {}", render::ty(t))),
        ItemEnum::AssocType { generics, bounds, type_, .. } => {
            let mut s = format!("type {name}{}", render::generic_params(generics));
            if !bounds.is_empty() {
                s.push_str(&format!(": {}", render::bounds(bounds, " + ")));
            }
            if let Some(d) = type_ {
                s.push_str(&format!(" = {}", render::ty(d)));
            }
            s.push_str(&render::where_clause(generics));
            Some(s)
        }
        ItemEnum::AssocConst { type_, value, .. } => {
            let mut s = format!("const {name}: {}", render::ty(type_));
            if let Some(v) = value {
                s.push_str(&format!(" = {v}"));
            }
            Some(s)
        }
        ItemEnum::TypeAlias(a) => Some(format!(
            "type {name}{} = {}{}",
            render::generic_params(&a.generics),
            render::ty(&a.type_),
            render::where_clause(&a.generics)
        )),
        ItemEnum::Constant { type_, const_, .. } => {
            Some(format!("const {name}: {} = {}", render::ty(type_), const_.expr))
        }
        ItemEnum::Static(s) => Some(format!(
            "static {}{name}: {}",
            if s.is_mutable { "mut " } else { "" },
            render::ty(&s.type_)
        )),
        ItemEnum::Struct(s) => Some(format!(
            "struct {name}{}{}",
            render::generic_params(&s.generics),
            render::where_clause(&s.generics)
        )),
        ItemEnum::Enum(e) => Some(format!(
            "enum {name}{}{}",
            render::generic_params(&e.generics),
            render::where_clause(&e.generics)
        )),
        ItemEnum::Union(u) => Some(format!(
            "union {name}{}{}",
            render::generic_params(&u.generics),
            render::where_clause(&u.generics)
        )),
        _ => {
            let _ = in_assoc;
            None
        }
    }
}

/// Types referenced by a declaration, with the role they play. Only resolved
/// paths are recorded — a `Generic` or `Primitive` is not a cross-item edge.
fn occurrences(item: &Item) -> Vec<Value> {
    let mut out = Vec::new();
    let mut push = |role: &str, pos: usize, sel: String, t: &Type| {
        collect_paths(t, &mut |p: &Path| {
            out.push(json!({
                "role": role,
                "position": pos,
                "selector": sel,
                "referenced_id": p.id,
                "path": p.path,
                "direct": true,
            }));
        });
    };
    match &item.inner {
        ItemEnum::Function(f) => {
            for (i, (n, t)) in f.sig.inputs.iter().enumerate() {
                push("parameter", i, n.clone(), t);
            }
            if let Some(o) = &f.sig.output {
                push("return", 0, "->".into(), o);
            }
        }
        ItemEnum::StructField(t) => push("field", 0, "field".into(), t),
        ItemEnum::AssocType { type_: Some(t), .. } => push("assoc_type", 0, "=".into(), t),
        ItemEnum::AssocConst { type_, .. } => push("assoc_const", 0, ":".into(), type_),
        ItemEnum::TypeAlias(a) => push("alias_target", 0, "=".into(), &a.type_),
        ItemEnum::Impl(i) => {
            push("impl_self", 0, "for".into(), &i.for_);
            if let Some(t) = &i.trait_ {
                out.push(json!({
                    "role": "impl_trait", "position": 0, "selector": "impl",
                    "referenced_id": t.id, "path": t.path, "direct": true,
                }));
            }
        }
        _ => {}
    }
    out
}

fn collect_paths(t: &Type, f: &mut impl FnMut(&Path)) {
    match t {
        Type::ResolvedPath(p) => {
            f(p);
            if let Some(a) = &p.args {
                collect_args(a, f);
            }
        }
        Type::DynTrait(d) => {
            for pt in &d.traits {
                f(&pt.trait_);
                if let Some(a) = &pt.trait_.args {
                    collect_args(a, f);
                }
            }
        }
        Type::Tuple(ts) => ts.iter().for_each(|x| collect_paths(x, f)),
        Type::Slice(x) | Type::Array { type_: x, .. } | Type::RawPointer { type_: x, .. }
        | Type::BorrowedRef { type_: x, .. } | Type::Pat { type_: x, .. } => collect_paths(x, f),
        Type::ImplTrait(bs) => {
            for b in bs {
                if let GenericBound::TraitBound { trait_, .. } = b {
                    f(trait_);
                }
            }
        }
        Type::QualifiedPath { self_type, trait_, .. } => {
            collect_paths(self_type, f);
            if let Some(tr) = trait_ {
                f(tr);
            }
        }
        Type::FunctionPointer(fp) => {
            for (_, x) in &fp.sig.inputs {
                collect_paths(x, f);
            }
            if let Some(o) = &fp.sig.output {
                collect_paths(o, f);
            }
        }
        _ => {}
    }
}

fn collect_args(a: &GenericArgs, f: &mut impl FnMut(&Path)) {
    if let GenericArgs::AngleBracketed { args, .. } = a {
        for g in args {
            if let GenericArg::Type(t) = g {
                collect_paths(t, f);
            }
        }
    }
}

/// Strip machine-specific prefixes so a record is portable and diffable.
fn sanitize_paths(s: &str) -> String {
    let mut out = s.to_string();
    for marker in ["/.worktrees/datafusion-55.0.0/", "/.worktrees/arrow-rs-59.2.0/"] {
        while let Some(i) = out.find(marker) {
            let start = out[..i].rfind(|c: char| c.is_whitespace()).map_or(0, |x| x + 1);
            out.replace_range(start..i + marker.len(), "");
        }
    }
    out
}

/// Conditions under which the item exists at all: `cfg`/`cfg_attr` gates, plus
/// the structured attributes that change how it may be used.
///
/// rustdoc has no dedicated `cfg` variant — these arrive as `Attribute::Other`
/// with the raw attribute text, which embeds absolute source spans, so the text
/// is sanitised before it is recorded.
fn availability(item: &Item) -> Vec<String> {
    item.attrs
        .iter()
        .filter_map(|a| match a {
            Attribute::Other(s) if s.contains("cfg") => Some(sanitize_paths(s)),
            Attribute::NonExhaustive => Some("#[non_exhaustive]".to_string()),
            Attribute::MustUse { reason } => Some(match reason {
                Some(r) => format!("#[must_use = {r:?}]"),
                None => "#[must_use]".to_string(),
            }),
            Attribute::TargetFeature { enable } => {
                Some(format!("#[target_feature(enable = \"{}\")]", enable.join(",")))
            }
            _ => None,
        })
        .collect()
}

/// Spans are recorded repo-relative: an absolute path leaks the extraction
/// worktree and is not a durable anchor.
fn rel_span(p: &std::path::Path) -> String {
    let s = p.display().to_string();
    for marker in ["/.worktrees/datafusion-55.0.0/", "/.worktrees/arrow-rs-59.2.0/"] {
        if let Some(i) = s.find(marker) {
            return s[i + marker.len()..].to_string();
        }
    }
    // Items pulled in from external crates live under the cargo registry, whose
    // path embeds the user's home and a registry hash. Reduce it to the durable
    // part: `registry/<crate>-<version>/<path>`.
    if let Some(i) = s.find("/registry/src/") {
        if let Some(rest) = s[i + "/registry/src/".len()..].split_once('/') {
            return format!("registry/{}", rest.1);
        }
    }
    s
}

fn span_json(s: &Option<Span>) -> Option<Value> {
    s.as_ref().map(|s| {
        json!({ "filename": rel_span(&s.filename),
                "begin": [s.begin.0, s.begin.1], "end": [s.end.0, s.end.1] })
    })
}

struct Ctx<'a> {
    krate: &'a Crate,
    idx: &'a routes::RouteIndex,
    profile: &'a str,
    artifact: String,
    crate_name: &'a str,
}

impl<'a> Ctx<'a> {
    /// Absolute path as rustdoc knows it, independent of re-exports.
    fn defining_path(&self, id: &Id) -> Option<String> {
        self.krate.paths.get(id).map(|s| s.path.join("::"))
    }

    fn decl(&self, id: &Id, item: &'a Item, owner: Option<&str>, in_assoc: bool) -> Decl<'a> {
        let routes_v = self.idx.for_id(id);
        let symbol = self
            .idx
            .canonical(id)
            .or_else(|| self.defining_path(id))
            .or_else(|| {
                owner.zip(item.name.as_deref()).map(|(o, n)| format!("{o}::{n}"))
            })
            .unwrap_or_else(|| {
                // Anonymous items (impl blocks) have no path. `Id` is opaque and
                // document-local, so it must never become a durable key; derive a
                // stable name from the rendered header instead.
                match signature_of(item, in_assoc) {
                    Some(h) => format!("{}::{{impl {}}}", self.crate_name, &digest(&h)[..16]),
                    None => format!("{}::{{anon {}}}", self.crate_name, &digest(&format!("{:?}", item.inner))[..16]),
                }
            });
        let sig = signature_of(item, in_assoc);
        let kind = kind_of(&item.inner, in_assoc);
        let sd = digest(&format!(
            "{}|{}|{}",
            kind,
            symbol,
            sig.as_deref().unwrap_or("")
        ));
        Decl {
            schema: DECL_SCHEMA,
            profile: self.profile,
            id: format!("{}:{}:{}", self.profile, self.artifact, symbol),
            artifact: self.artifact.clone(),
            local_id: id.0.to_string(),
            symbol,
            kind,
            owner: owner.map(str::to_string),
            owner_contract: None,
            public_routes: routes_v,
            route_state: self.idx.state(),
            signature: sig,
            type_occurrences: occurrences(item),
            deprecated: item.deprecation.as_ref().map(|d| json!({
                "since": d.since, "note": d.note
            })),
            availability_conditions: availability(item),
            docs: item.docs.as_deref(),
            span: span_json(&item.span),
            semantic_digest: sd,
        }
    }
}

fn normalize(input: &FsPath, profile: &str, outdir: &FsPath) -> Result<()> {
    let (fv, raw) = sniff(input)?;
    if fv != FORMAT_VERSION {
        bail!(
            "{}: format_version {fv} != rustdoc-types FORMAT_VERSION {FORMAT_VERSION}; \
             refusing to parse (a mismatched document deserializes silently and wrongly)",
            input.display()
        );
    }
    let krate: Crate = serde_json::from_str(&raw)
        .with_context(|| format!("typed parse of {}", input.display()))?;

    let crate_name = krate
        .index
        .get(&krate.root)
        .and_then(|i| i.name.clone())
        .or_else(|| input.file_stem().map(|s| s.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "unknown".into());
    let version = krate.crate_version.clone().unwrap_or_else(|| "0.0.0".into());
    let artifact = format!("{crate_name}-{version}");

    let idx = routes::build(&krate, &crate_name);
    let ctx = Ctx { krate: &krate, idx: &idx, profile, artifact: artifact.clone(), crate_name: &crate_name };

    fs::create_dir_all(outdir)?;
    let mut api = writer(outdir, "api", &artifact)?;
    let mut traits_w = writer(outdir, "traits", &artifact)?;
    let mut impls_w = writer(outdir, "impls", &artifact)?;

    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    let mut assoc_owner: BTreeMap<Id, String> = BTreeMap::new();

    // Pass 1: name the owner of every associated item, so methods get a real
    // qualified symbol instead of floating free.
    //
    // One associated item can be claimed by several owners: a blanket impl such
    // as `impl<T> Borrow<T> for U` reuses the *same* rustdoc Id across every U it
    // covers. Collect all candidates and pick deterministically, preferring a
    // real impl over a blanket or compiler-synthesised one. Last-writer-wins over
    // a HashMap would make the output vary between runs on identical input.
    let mut owner_candidates: BTreeMap<Id, Vec<(u8, String)>> = BTreeMap::new();
    for (id, item) in &krate.index {
        match &item.inner {
            ItemEnum::Trait(t) => {
                let owner = idx
                    .canonical(id)
                    .or_else(|| ctx.defining_path(id))
                    .unwrap_or_else(|| item.name.clone().unwrap_or_default());
                for m in &t.items {
                    owner_candidates.entry(*m).or_default().push((0, owner.clone()));
                }
            }
            ItemEnum::Impl(i) => {
                let owner = match &i.trait_ {
                    Some(tr) => format!("<{} as {}>", render::ty(&i.for_), render::path(tr)),
                    None => render::ty(&i.for_),
                };
                let rank = if i.blanket_impl.is_some() || i.is_synthetic { 2 } else { 1 };
                for m in &i.items {
                    owner_candidates.entry(*m).or_default().push((rank, owner.clone()));
                }
            }
            _ => {}
        }
    }
    for (id, mut cands) in owner_candidates {
        cands.sort();
        if let Some((_, owner)) = cands.into_iter().next() {
            assoc_owner.insert(id, owner);
        }
    }

    // Pass 2: emit one declaration per item, associated items included.
    //
    // `krate.index` is a HashMap, so its iteration order varies between runs.
    // Emit in a stable order (symbol, then kind) so a re-extraction of unchanged
    // input is byte-identical and diffs show only real API change.
    let mut decls: Vec<Decl> = Vec::with_capacity(krate.index.len());
    for (id, item) in &krate.index {
        let owner = assoc_owner.get(id).map(String::as_str);
        decls.push(ctx.decl(id, item, owner, owner.is_some()));
    }
    decls.sort_by(|a, b| {
        (&a.symbol, a.kind, &a.local_id).cmp(&(&b.symbol, b.kind, &b.local_id))
    });
    for d in &decls {
        *counts.entry(d.kind).or_default() += 1;
        writeln!(api, "{}", serde_json::to_string(d)?)?;
    }

    let mut trait_rows: Vec<(String, Value)> = Vec::new();
    let mut impl_rows: Vec<(String, Value)> = Vec::new();
    for (id, item) in &krate.index {
        match &item.inner {
            ItemEnum::Trait(t) => {
                let v = trait_contract(&ctx, id, item, t);
                trait_rows.push((item.name.clone().unwrap_or_default(), v));
            }
            ItemEnum::Impl(i) => {
                let v = impl_relation(&ctx, item, i);
                impl_rows.push((render::impl_header(i), v));
            }
            _ => {}
        }
    }
    let mut trait_lines: Vec<String> = trait_rows
        .iter()
        .map(|(_, v)| serde_json::to_string(v))
        .collect::<std::result::Result<_, _>>()?;
    let mut impl_lines: Vec<String> = impl_rows
        .iter()
        .map(|(_, v)| serde_json::to_string(v))
        .collect::<std::result::Result<_, _>>()?;
    trait_lines.sort();
    impl_lines.sort();
    for l in &trait_lines {
        writeln!(traits_w, "{l}")?;
    }
    for l in &impl_lines {
        writeln!(impls_w, "{l}")?;
    }

    api.flush()?;
    traits_w.flush()?;
    impls_w.flush()?;

    let receipt = json!({
        "schema": "dfarrow.receipt.v1",
        "profile": profile,
        "artifact": artifact,
        "crate": crate_name,
        "crate_version": version,
        "format_version": fv,
        "rustdoc_types_format_version": FORMAT_VERSION,
        "target": krate.target.triple,
        "includes_private": krate.includes_private,
        "index_items": krate.index.len(),
        "paths_entries": krate.paths.len(),
        "external_crates": krate.external_crates.len(),
        "route_state": idx.state(),
        "routed_items": idx.routes.len(),
        "unresolved_uses": idx.unresolved.iter().collect::<Vec<_>>(),
        "unresolved_globs": idx.unresolved_globs.iter().collect::<Vec<_>>(),
        "kind_counts": counts,
        "input_digest": digest(&raw),
    });
    fs::write(
        outdir.join(format!("receipt.{artifact}.json")),
        serde_json::to_string_pretty(&receipt)? + "\n",
    )?;
    println!("{}", serde_json::to_string(&receipt)?);
    Ok(())
}

fn writer(dir: &FsPath, kind: &str, artifact: &str) -> Result<BufWriter<fs::File>> {
    let d: PathBuf = dir.join(kind);
    fs::create_dir_all(&d)?;
    Ok(BufWriter::new(fs::File::create(d.join(format!("{artifact}.jsonl")))?))
}

fn trait_contract(ctx: &Ctx, id: &Id, item: &Item, t: &Trait) -> Value {
    let name = item.name.clone().unwrap_or_default();
    let mut required = Vec::new();
    let mut provided = Vec::new();
    let mut assoc_types = Vec::new();
    let mut assoc_consts = Vec::new();

    for mid in &t.items {
        let Some(m) = ctx.krate.index.get(mid) else { continue };
        let mname = m.name.clone().unwrap_or_default();
        let entry = json!({
            "name": mname,
            "signature": signature_of(m, true),
            "docs": m.docs,
            "deprecated": m.deprecation.as_ref().map(|d| json!({"since": d.since, "note": d.note})),
            "span": span_json(&m.span),
        });
        match &m.inner {
            // `has_body` inside a trait declaration == the trait supplies a default.
            ItemEnum::Function(f) if f.has_body => provided.push(entry),
            ItemEnum::Function(_) => required.push(entry),
            ItemEnum::AssocType { type_, .. } => {
                let mut e = entry;
                e["has_default"] = json!(type_.is_some());
                assoc_types.push(e);
            }
            ItemEnum::AssocConst { value, .. } => {
                let mut e = entry;
                e["has_default"] = json!(value.is_some());
                assoc_consts.push(e);
            }
            _ => {}
        }
    }

    json!({
        "schema": TRAIT_SCHEMA,
        "profile": ctx.profile,
        "artifact": ctx.artifact,
        "crate": ctx.crate_name,
        "trait_name": name,
        "canonical_route": ctx.idx.canonical(id),
        "public_routes": ctx.idx.for_id(id),
        "defining_path": ctx.defining_path(id),
        "header": render::trait_header(&name, t),
        "is_unsafe": t.is_unsafe,
        "is_auto": t.is_auto,
        "is_dyn_compatible": t.is_dyn_compatible,
        "supertraits": t.bounds.iter().map(render::bound).collect::<Vec<_>>(),
        "generics": render::generic_params(&t.generics),
        "where_clause": render::where_clause(&t.generics),
        "required_methods": required,
        "provided_methods": provided,
        "assoc_types": assoc_types,
        "assoc_consts": assoc_consts,
        "implementation_ids": t.implementations.len(),
        "deprecated": item.deprecation.as_ref().map(|d| json!({"since": d.since, "note": d.note})),
        "docs": item.docs,
        "span": span_json(&item.span),
    })
}

fn impl_relation(ctx: &Ctx, item: &Item, i: &Impl) -> Value {
    json!({
        "schema": IMPL_SCHEMA,
        "profile": ctx.profile,
        "artifact": ctx.artifact,
        "crate": ctx.crate_name,
        "header": render::impl_header(i),
        "trait_path": i.trait_.as_ref().map(render::path),
        "trait_id": i.trait_.as_ref().map(|t| t.id.0),
        "for_type": render::ty(&i.for_),
        "is_blanket": i.blanket_impl.is_some(),
        "is_synthetic": i.is_synthetic,
        "is_negative": i.is_negative,
        "is_unsafe": i.is_unsafe,
        "generics": render::generic_params(&i.generics),
        "where_clause": render::where_clause(&i.generics),
        "provided_trait_methods": i.provided_trait_methods,
        "item_count": i.items.len(),
        "span": span_json(&item.span),
        "docs": item.docs,
    })
}
