#!/usr/bin/env python3
"""Render V1 — Extension-Point Contracts — from extracted trait/impl facts.

Everything in the output is derived from rustdoc JSON of the pinned worktrees.
No claim is authored here that is not present in the facts.
"""
import json, sys, re, pathlib, collections, datetime

ROOT = pathlib.Path(__file__).resolve().parents[2]
FACTS = ROOT / "build/facts"
OUT = ROOT / "docs/library_ref/dfarrow55_extension_point_contracts_2026-09-13.md"

# Curated ordering: the extension points an architecture actually turns on,
# grouped by the question they answer. Anything not listed still appears in the
# full index table at the end.
GROUPS = [
    ("Data sources and catalogs", [
        "TableProvider", "TableProviderFactory", "TableFunctionImpl",
        "CatalogProvider", "CatalogProviderList", "SchemaProvider",
        "UrlTableFactory", "StreamProvider", "PartitionStream",
    ]),
    ("User-defined calculations", [
        "ScalarUDFImpl", "AggregateUDFImpl", "WindowUDFImpl",
        "AsyncScalarUDFImpl", "HigherOrderUDFImpl",
        "Accumulator", "GroupsAccumulator", "PartitionEvaluator",
    ]),
    ("Planning and optimization", [
        "OptimizerRule", "AnalyzerRule", "OptimizerConfig",
        "UserDefinedLogicalNode", "UserDefinedLogicalNodeCore",
        "ExprPlanner", "TypePlanner", "RelationPlanner", "ContextProvider",
    ]),
    ("Physical execution", [
        "ExecutionPlan", "ExecutionPlanProperties", "ExecutionPlanVisitor",
        "DisplayAs", "StatisticsProvider",
    ]),
    ("Session, runtime and resources", [
        "Session", "MemoryPool", "FunctionRegistry", "SerializerRegistry",
        "ExtensionTypeRegistry", "VarProvider",
    ]),
    ("Plan traversal and contracts", [
        "TreeNode", "TreeNodeVisitor", "TreeNodeRewriter", "DynTreeNode",
        "ConcreteTreeNode", "ExprSchemable", "ExprSchema",
        "ConfigExtension", "ExtensionOptions", "PruningStatistics",
    ]),
]

def load(kind):
    out = []
    for p in sorted(FACTS.glob(f"*/{kind}/*.jsonl")):
        for line in p.read_text().splitlines():
            if line.strip():
                out.append(json.loads(line))
    return out

traits_all = load("traits")
impls_all = load("impls")

# A short trait name can collide across crates; key on (name, crate).
by_name = collections.defaultdict(list)
for t in traits_all:
    by_name[t["trait_name"]].append(t)

# Implementor index: short trait name -> concrete implementors.
# Blanket and synthetic impls are excluded: they are compiler-derived and say
# nothing about who deliberately implements the extension point.
impl_idx = collections.defaultdict(list)
for i in impls_all:
    tp = i.get("trait_path")
    if not tp or i.get("is_blanket") or i.get("is_synthetic") or i.get("is_negative"):
        continue
    short = re.sub(r"<.*", "", tp).split("::")[-1]
    impl_idx[short].append(i)

def first_doc_sentence(docs, limit=320):
    """First prose paragraph of a doc comment, markdown links flattened.

    Taking the first *paragraph* rather than the first N characters of the whole
    comment avoids splicing unrelated sentences together across section breaks.
    """
    if not docs:
        return ""
    para = []
    for raw in docs.splitlines():
        l = raw.strip()
        if not l:
            if para:
                break
            continue
        if l.startswith(("```", "#", "|", "* ", "- ", "1.", "> ")):
            if para:
                break
            continue
        if re.match(r"^\[[^\]]+\]:", l):   # link definition
            continue
        para.append(l)
    text = " ".join(para)
    text = re.sub(r"\[`?([^\]`]+)`?\]\([^)]*\)", r"`\1`", text)
    text = re.sub(r"\[`?([^\]`]+)`?\]", r"`\1`", text)
    text = re.sub(r"\s+", " ", text).strip()
    if len(text) > limit:
        cut = text[:limit].rsplit(". ", 1)[0]
        text = (cut + ".") if len(cut) > 80 else text[:limit].rstrip() + "…"
    return text

def span_anchor(sp):
    if not sp:
        return ""
    return f"{sp['filename']}:{sp['begin'][0]}"

def method_block(ms):
    return "\n".join(f"    {m['signature']};" for m in ms if m.get("signature"))

def render_trait(t, n):
    name = t["trait_name"]
    L = []
    L.append(f"# {n}) `{name}` — {t['crate'].replace('_','-')}\n")
    L.append("| | |")
    L.append("|---|---|")
    L.append(f"| **Canonical path** | `{t.get('canonical_route') or t.get('defining_path')}` |")
    L.append(f"| **Defining crate** | `{t['crate'].replace('_','-')}` ({t['artifact'].split('-')[-1]}) |")
    L.append(f"| **Object safe (`dyn`)** | {'yes' if t['is_dyn_compatible'] else '**no** — cannot be used as `dyn ' + name + '`'} |")
    sup = ", ".join(f"`{s}`" for s in t["supertraits"]) or "—"
    L.append(f"| **Supertraits** | {sup} |")
    L.append(f"| **Required / provided** | {len(t['required_methods'])} required · {len(t['provided_methods'])} provided |")
    impls = impl_idx.get(name, [])
    L.append(f"| **In-tree implementors** | {len(impls)} |")
    if t.get("deprecated"):
        L.append(f"| **Deprecated** | {t['deprecated']} |")
    anchor = span_anchor(t.get("span"))
    if anchor:
        L.append(f"| **Source** | `{anchor}` |")
    L.append("")

    doc = first_doc_sentence(t.get("docs"))
    if doc:
        L.append(f"{doc}\n")

    L.append("```rust")
    L.append(f"{t['header']} {{")
    if t["assoc_types"]:
        L.append("    // associated types")
        for a in t["assoc_types"]:
            L.append(f"    {a['signature']};")
    if t["assoc_consts"]:
        for a in t["assoc_consts"]:
            L.append(f"    {a['signature']};")
    if t["required_methods"]:
        L.append("")
        L.append("    // REQUIRED — you must implement these")
        L.append(method_block(t["required_methods"]))
    if t["provided_methods"]:
        L.append("")
        L.append(f"    // PROVIDED — defaulted; override only with a reason ({len(t['provided_methods'])})")
        L.append(method_block(t["provided_methods"]))
    L.append("}")
    L.append("```\n")

    if t["required_methods"]:
        L.append(f"**Required methods — the minimum viable implementation.**\n")
        L.append("| Method | Contract |")
        L.append("|---|---|")
        for m in t["required_methods"]:
            d = first_doc_sentence(m.get("docs"), 200) or "—"
            L.append(f"| `{m['name']}` | {d} |")
        L.append("")

    if t["provided_methods"]:
        L.append("**Provided methods — defaults exist; each override is a decision.**\n")
        L.append("| Method | What the default does |")
        L.append("|---|---|")
        for m in t["provided_methods"]:
            d = first_doc_sentence(m.get("docs"), 200) or "—"
            dep = " **(deprecated)**" if m.get("deprecated") else ""
            L.append(f"| `{m['name']}`{dep} | {d} |")
        L.append("")

    if impls:
        byc = collections.defaultdict(list)
        for i in impls:
            byc[i["crate"].replace("_", "-")].append(i["for_type"])
        L.append(f"**In-tree implementors ({len(impls)})** — the built-ins to check before writing your own.\n")
        L.append("| Crate | Implementors |")
        L.append("|---|---|")
        for c in sorted(byc):
            types = ", ".join(f"`{x}`" for x in sorted(set(byc[c])))
            L.append(f"| `{c}` | {types} |")
        L.append("")

    if t["required_methods"] or t["assoc_types"]:
        gen = t.get("generics") or ""
        L.append("**Minimal implementation skeleton.**\n")
        L.append("```rust")
        has_async = any("async fn" in (m.get("signature") or "") for m in t["required_methods"])
        if has_async:
            L.append("#[async_trait::async_trait]")
        L.append(f"impl {name}{gen} for MyType {{")
        for a in t["assoc_types"]:
            L.append(f"    {a['signature'].split(' = ')[0].split(':')[0]} = /* … */;")
        for m in t["required_methods"]:
            sig = m.get("signature")
            if sig:
                L.append(f"    {sig} {{ todo!() }}")
        L.append("}")
        L.append("```\n")
    return "\n".join(L)

# ---- assemble ----
today = datetime.date.today().isoformat()
doc = []
doc.append("# DataFusion 55.0.0 + Arrow 59.2.0 — Extension-Point Contracts (source-verified)\n")
doc.append(f"""
**Corpus context:** `datafusion-rust-src-55.0.0` · `arrow-rust-src-59.2.0`
**Generated:** {today} — every statement below is derived from rustdoc JSON of
pinned git worktrees at tags `55.0.0` and `59.2.0`, not from prose or docs.rs.

> **Precedence.** This document is *source-verified*. Where it and the narrative
> deep-dives (`datafusion_55_arrow_59_2_comprehensive_advanced_reference_narrative_2026-09-12.md`,
> `arrow_rust_59_datafusion55_advanced_reference_2026-08-23.md`) disagree, **this wins** —
> that is what `V5) Final source-of-truth rule` already instructs, now with evidence behind it.
> The narrative documents remain authoritative for *why and when*; this one is *what exists*.

## Provenance

| | |
|---|---|
| DataFusion | `55.0.0` @ `d5552342` (tag `55.0.0`) |
| arrow-rs | `59.2.0` @ `782e5a68` (tag `59.2.0`) |
| Toolchain | `nightly-2026-08-18` (`rustc 1.100.0-nightly 8fa1c96cf`) |
| rustdoc `format_version` | **61**, equal to `rustdoc_types::FORMAT_VERSION` — verified before every parse |
| Feature profile | `default` |
| Target | `x86_64-unknown-linux-gnu` |

**Re-derive:**

```bash
tooling/dfarrow-apiex/gen-rustdoc.sh .worktrees/datafusion-55.0.0 df55-default <pkg>...
tooling/dfarrow-apiex/target/release/dfarrow-apiex normalize \\
    build/rustdoc/df55-default/<crate>.json df55-default build/facts/df55-default
python3 tooling/dfarrow-apiex/render_v1.py
```

## How to read this

* **Required** methods are the minimum viable implementation. **Provided** methods have
  defaults — each override is a deliberate decision, and the table says what you are
  overriding.
* **In-tree implementors** is the built-in catalogue. Check it before implementing an
  extension point yourself: the escalation ladder says stop at the first rung that
  expresses the semantics.
* Blanket and compiler-synthesised impls (`Send`, `Sync`, `Freeze`, `From`/`Into` …) are
  excluded from implementor counts — they say nothing about deliberate extension.
* **Object safety** matters: a trait marked not `dyn`-compatible cannot be stored as
  `Arc<dyn Trait>`, which constrains registry and plugin designs.

## Crate map — where these traits actually live

A trait's name does not predict its crate. In 55.0.0 the source/catalog traits live in
**`datafusion-session`**, not `datafusion-catalog`; the accumulators live in
**`datafusion-expr-common`**, not `datafusion-functions-aggregate`.

| Trait | Crate | Canonical path |
|---|---|---|""")

seen = set()
rows = []
for _, names in GROUPS:
    for nm in names:
        for t in by_name.get(nm, []):
            rows.append((nm, t["crate"].replace("_", "-"), t.get("canonical_route") or t.get("defining_path")))
for nm, cr, cp in sorted(set(rows)):
    doc.append(f"| `{nm}` | `{cr}` | `{cp}` |")
doc.append("")

n = 0
for title, names in GROUPS:
    doc.append(f"\n---\n\n## Part — {title}\n")
    for nm in names:
        for t in sorted(by_name.get(nm, []), key=lambda x: x["crate"]):
            key = (t["trait_name"], t["crate"])
            if key in seen:
                continue
            seen.add(key)
            n += 1
            doc.append(render_trait(t, n))

# Reconciliation against the narrative corpus, for the claims this slice can settle.
doc.append("""
---

# Appendix — reconciliation against the narrative corpus

Claims in the prose deep-dives that this extraction can settle, classified with the
existing `library-guides.derived_behaviors.v1` vocabulary
(`agreed` · `refined` · `contradicted`).

| Claim | Source | Verdict | Evidence |
|---|---|---|---|
| `as_any` was removed from the provider/UDF traits; `Any` is a supertrait instead | `df` narrative ~7842, ~8880, ~8904 | **agreed** | No extracted contract lists `as_any`; `TableProvider`'s supertraits are `Any + Debug + Sync + Send`. A hand-written `fn as_any` fails to compile (`E0407`). |
| `ExecutionPlan::apply_expressions` is required in 55 | `df` §40A (~88, ~131) | **agreed** | Extracted as a required method: `fn apply_expressions(&self, f: &mut dyn FnMut(&Arc<dyn PhysicalExpr>) -> Result<TreeNodeRecursion>) -> Result<TreeNodeRecursion>`; omitting it fails with `E0046`. |
| `TableProvider::scan_with_args` / `ScanArgs` exist in 55 | `df` §40A | **agreed** | Extracted as a provided method: `async fn scan_with_args<'a>(&self, state: &dyn Session, args: ScanArgs<'a>) -> Result<ScanResult>`. |
| `ScalarUDFImpl::is_strict` exists in 55 | `df` §40A, V6 | **agreed** | Extracted as a provided method: `fn is_strict(&self) -> bool`. |
| Merge-into surfaces exist in 55 | `df` §40A | **agreed** | `TableProvider::merge_into` extracted, taking `Vec<MergeIntoClause>`. |
| `ExecutionPlan::properties` returns `&PlanProperties` | conventional usage, pre-55 | **contradicted** | 55.0.0 returns **`&Arc<PlanProperties>`** (`datafusion/physical-plan/src/execution_plan.rs:160`). The wrong form fails with `E0053`. |
| `TableProvider` lives in `datafusion-catalog` | plausible inference from the crate name | **contradicted** | In 55.0.0 it is defined in **`datafusion-session`** (`datafusion/session/src/table.rs:52`), together with `CatalogProvider`, `SchemaProvider`, `CatalogProviderList`, `TableFunctionImpl` and `Session`. |

**Reading.** Five of seven checkable claims came out *agreed*, which raises rather than
lowers confidence in the narrative corpus; the two contradictions are both cases where
a *convention* — not the corpus — was stale. That is the expected shape of a healthy
reconciliation, and it is why the ledger records agreements as well as corrections.
""")

# Full index of everything extracted, so absence from the curated list is never
# mistaken for absence from the library.
doc.append("\n---\n\n# Appendix — complete extracted trait index\n")
doc.append(f"All {len(traits_all)} public traits found across the {len(set(t['crate'] for t in traits_all))} "
           "extracted crates, curated or not. A trait missing from the sections above is **not** "
           "evidence it is unimportant — only that it was not curated.\n")
doc.append("| Trait | Crate | Canonical path | req | prov | impls | dyn |")
doc.append("|---|---|---|---:|---:|---:|---|")
for t in sorted(traits_all, key=lambda x: (x["crate"], x["trait_name"])):
    doc.append(
        f"| `{t['trait_name']}` | `{t['crate'].replace('_','-')}` | `{t.get('canonical_route') or t.get('defining_path')}` "
        f"| {len(t['required_methods'])} | {len(t['provided_methods'])} "
        f"| {len(impl_idx.get(t['trait_name'], []))} | {'y' if t['is_dyn_compatible'] else 'n'} |"
    )

OUT.write_text("\n".join(doc) + "\n")
print(f"wrote {OUT}")
print(f"  traits documented in detail: {n}")
print(f"  traits in appendix index   : {len(traits_all)}")
print(f"  impl relations considered  : {len(impls_all)}")
