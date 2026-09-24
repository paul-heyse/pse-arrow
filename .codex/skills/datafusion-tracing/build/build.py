"""Build the datafusion-tracing capability repository from `build/acquired/`.

Offline. Everything this reads is in the tree; `acquire.py` is the only stage that is not, and
`build.py` never imports it -- if acquisition could run inside the build, a rebuild could pick
up different bytes from the network and still call itself reproducible.

Stages, in order:

    models       both rustdoc captures -> canonical items, stitched across crates
    visibility   the two captures plus the pinned source -> how a caller reaches each item
    indexes      line-oriented projections for ripgrep
    pages        model/ and api/ per module
    spans        upstream trace snapshots -> the emitted span contract
    macros       macro arms -> the invocation grammar rustdoc keeps and prose does not
    compat       release history -> the version-lock matrix
    corpus       the pinned upstream files, verbatim
    router       questions.tsv -- which artifact answers which question
    pages/2      topics, seams and catalogs
    rules        the generated ast-grep rule
    provenance   pins, counts, corpus commits, per-file sha256

Ordering constraints worth knowing before moving anything: the corpus must land before the
seam and topic pages, which link into it; `visibility` must run before `indexes`, because the
column it produces is the reason most of these tables are worth reading; and provenance is last
because it hashes everything else.
"""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
from pathlib import Path

import capabilities
import catalogs
import compat
import contracts
import emit
import fetch
import macros
import model
import pages
import queries
import router
import spans
import visibility

HERE = Path(__file__).resolve().parent
SKILL_ROOT = HERE.parent
CONTENT = SKILL_ROOT / "content"
QUERIES = SKILL_ROOT / "queries"
ACQUIRED = HERE / "acquired"


def say(message: str) -> None:
    sys.stderr.write(f"{message}\n")
    sys.stderr.flush()


def ast_grep_version() -> str:
    try:
        done = subprocess.run(
            ["ast-grep", "--version"], capture_output=True, text=True, check=False
        )
    except FileNotFoundError:
        return "absent"
    return done.stdout.strip() or "absent"


def load_acquisition() -> dict:
    record = ACQUIRED / "ACQUISITION.json"
    if not record.exists():
        raise SystemExit(
            "build/acquired/ACQUISITION.json is absent. Run:\n"
            "    python3 build/acquire.py\n"
            "which needs a network, cargo and the pinned nightly. Every stage after it is "
            "offline, which is what makes the determinism check mean anything."
        )
    return json.loads(record.read_text())


def document(record: dict, suffix: str = "") -> bytes:
    """Read one acquired rustdoc document, checking it against its recorded digest.

    Locally produced bytes have no upstream form to fall back on, so they are verified rather
    than trusted. The docs.rs half is verified the same way for uniformity: a cache that cannot
    be checked is a cache that can be edited.
    """
    name = f"{record['package']}@{record['version']}{suffix}.json.zst"
    blob = (ACQUIRED / "rustdoc" / name).read_bytes()
    expected = (record["private"] if suffix else record)["compressed_sha256"]
    actual = fetch.digest(blob)
    if actual != expected:
        raise SystemExit(
            f"{name} does not match its recorded digest ({actual[:12]} vs {expected[:12]}). "
            f"Re-run build/acquire.py rather than building from bytes nothing vouches for."
        )
    return fetch.decompress_zstd(blob)


# --------------------------------------------------------------------------- models


def build_models(manifest: dict, acquisition: dict) -> tuple[dict[str, model.CrateModel], dict]:
    """Parse every acquired crate. Subject crates are parsed from the private capture.

    This is the one place this repository departs from its siblings, and the departure is the
    point. deltalake reads its `--document-private-items` capture as a supplement that may fill
    in impls but never add items, because there the private surface is genuinely private. Here
    the private capture is the PRIMARY document for the two subject crates: sixteen of the
    methods a caller must use appear in nothing else. Publishing them unlabelled would be the
    opposite mistake, so `visibility.py` classifies every item and `visibility` is carried on
    every row that leaves this build.
    """
    supported = manifest["tools"]["rustdoc_format_supported"]
    models: dict[str, model.CrateModel] = {}
    classifications: dict[str, visibility.Classification] = {}

    for package, record in sorted(acquisition["crates"].items()):
        public = document(record)
        if "private" in record:
            private = document(record, ".private")
            source = ACQUIRED / "corpus" / _source_corpus(manifest, package)
            classifications[package] = visibility.classify(package, public, private, source)
            payload, supplement = private, None
        else:
            payload, supplement = public, None
        models[package] = model.build_crate(
            package, record["version"], payload, supported, supplement
        )
        captures = [("hosted", public)]
        if "private" in record:
            captures.append(("private", private))
        for capture, raw in captures:
            extracted = contracts.collect(
                package, record["version"], json.loads(raw), set(models[package].items), capture
            )
            for operation in extracted:
                owner = operation["owner"] or operation["path"]
                while owner not in models[package].items and "::" in owner:
                    owner = owner.rsplit("::", 1)[0]
                classification = classifications.get(package)
                operation["reachability"] = (
                    classification.of_item(owner) if classification else "supported"
                )
                if operation["visibility"] not in ("public", "default", None):
                    operation["reachability"] = "internal"
                operation["reached_via"] = (
                    classification.reached_via.get(owner, "") if classification else ""
                )
                operation["artifact"]["compressed_sha256"] = (
                    record["private"] if capture == "private" else record
                )["compressed_sha256"]
                operation["artifact"]["local_file"] = (
                    f"build/acquired/rustdoc/{package}@{record['version']}"
                    + (".private" if capture == "private" else "")
                    + ".json.zst"
                )
                if capture == "private":
                    operation["artifact"]["origin"] = "../../" + operation["artifact"]["local_file"]
                    operation["artifact"]["producer"] = manifest["tools"]["local_rustdoc"]
                operation["availability"] = (
                    f"Observed in the {capture} capture; "
                    "reachability is separate from raw visibility. "
                    "Consumer dependency/features/target must be checked independently."
                )
            models[package].contracts.extend(extracted)
        say(f"  {package}: {len(models[package].items)} canonical item(s)")
    return models, classifications


def _source_corpus(manifest: dict, package: str) -> str:
    """The corpus directory holding one subject crate's pinned source."""
    for corpus in manifest["corpora"]:
        if corpus["source_prefix"].startswith(f"{package}/src/"):
            return corpus["name"]
    raise SystemExit(f"no source corpus is declared for {package}; visibility cannot be read")


def apply_visibility(
    items: dict[str, model.Item], classifications: dict[str, visibility.Classification]
) -> dict[str, int]:
    """Stamp every item and method with how a caller reaches it."""
    tally: dict[str, int] = {}
    for path, item in items.items():
        classification = classifications.get(item.crate)
        if classification is None:
            # A crate with no private capture is documented in full by docs.rs, so every item
            # in it is reachable by definition. Counted, not skipped: a tally that disagreed
            # with the index it describes is the drift `verify.py --counts` exists to catch.
            tally[item.visibility] = tally.get(item.visibility, 0) + 1
            continue
        item.visibility = classification.of_item(path)
        item.reached_via = classification.reached_via.get(path, "")
        if item.kind == "struct":
            # Modelled from the private capture, so the raw field list includes private
            # fields. Replaced with what a caller can actually touch -- for these two types
            # the difference decides whether struct-literal construction compiles at all.
            item.fields = classification.public_fields.get(path, [])
        tally[item.visibility] = tally.get(item.visibility, 0) + 1
        for method in item.methods:
            method.visibility = classification.of_method(path, method.name)
    return tally


# --------------------------------------------------------------------------- indexes


def write_indexes(
    content: Path,
    items: dict[str, model.Item],
    aliases: dict[str, list[str]],
    unresolved: list[str],
    crate_facts: dict[str, dict],
    classifications: dict[str, visibility.Classification],
    acquisition: dict,
) -> dict[str, int]:
    index = content / "index"
    index.mkdir(parents=True, exist_ok=True)

    def write(name: str, rows: list[str]) -> int:
        (index / name).write_text("".join(f"{row}\n" for row in sorted(set(rows))))
        return len(set(rows))

    subject_of = {r["package"]: r["subject"] for r in acquisition["crates"].values()}

    symbols = [
        "\t".join(
            (
                item.path,
                item.kind,
                item.crate,
                subject_of.get(item.crate, "-"),
                item.visibility,
                item.reached_via or "-",
                emit.api_file(item.module),
                str(len(item.aliases)),
                str(len(item.methods)),
                item.summary or "-",
            )
        )
        for item in items.values()
    ]
    # Derived Clone/Default/Debug methods are filtered here exactly as `emit._model_record`
    # filters them. Leaving them in put `InstrumentationOptionsBuilder::default` beside
    # `add_custom_field` under `reachable-undocumented`, which turned the one count this
    # repository is read for -- sixteen methods no published artifact carries -- into eighteen.
    methods = [
        "\t".join(
            (
                item.path,
                method.name,
                method.via_trait or "-",
                method.visibility,
                method.signature,
                method.summary or "-",
            )
        )
        for item in items.values()
        for method in item.methods
        if not (method.via_trait and emit.is_ubiquitous(method.via_trait))
    ]
    impls = [
        "\t".join((trait, item.path, item.crate))
        for item in items.values()
        for trait in item.implements
        if not emit.is_ubiquitous(trait)
    ]
    alias_rows = [
        "\t".join((access, canonical, items[canonical].kind))
        for canonical, accesses in aliases.items()
        for access in accesses
        if canonical in items
    ]
    features = [
        "\t".join(
            (
                crate,
                name,
                ",".join(enables) or "-",
                "default" if name in facts.get("default_features", []) else "-",
            )
        )
        for crate, facts in crate_facts.items()
        for name, enables in (facts.get("features") or {}).items()
    ]

    # Every name the crate root advertises that rustdoc emits nowhere, in either capture.
    # Recorded because silence in an index is not evidence, and here the index would otherwise
    # contradict the crate's own lib.rs.
    unreachable = [
        "\t".join(
            (
                name,
                package,
                "advertised by lib.rs, absent from both rustdoc captures",
                "read content/corpus/source/lib.rs; the macros are the supported route",
            )
        )
        for package, classification in classifications.items()
        for name in classification.absent_from_rustdoc
    ]

    counts = {
        "symbols": write("symbols.tsv", symbols),
        "methods": write("methods.tsv", methods),
        "impls": write("impls.tsv", impls),
        "aliases": write("aliases.tsv", alias_rows),
        "features": write("features.tsv", features),
        "unreachable": write("unreachable.tsv", unreachable),
        "unresolved": write(
            "unresolved.tsv",
            ["\t".join((path, "not in an indexed crate")) for path in unresolved],
        ),
    }
    return counts


# --------------------------------------------------------------------------- probes


def read_behaviours(content: Path) -> dict[str, int]:
    """Read what the probes established, if they have run.

    `build.py` never runs them -- they compile DataFusion, and a build that executed a compiler
    could not claim to reproduce byte for byte. `behaviors.tsv` is an input to the build exactly
    as the acquired rustdoc documents are.
    """
    path = content / "index" / "behaviors.tsv"
    if not path.exists():
        say("  behaviors.tsv absent: every span row stays `recorded`. Run build/probes.py.")
        return {"probes": 0}
    tally: dict[str, int] = {}
    for line in path.read_text().splitlines():
        if not line:
            continue
        columns = line.split("\t")
        verdict = columns[2]
        tally[verdict] = tally.get(verdict, 0) + 1
    return {
        "probes": sum(tally.values()),
        **{f"probe_{name}": count for name, count in tally.items()},
    }


# --------------------------------------------------------------------------- corpus


def write_corpus(content: Path, manifest: dict, acquisition: dict) -> int:
    """Copy each acquired corpus to its published destination, verbatim."""
    written = 0
    for corpus in manifest["corpora"]:
        record = acquisition["corpora"][corpus["name"]]
        source = ACQUIRED / "corpus" / corpus["name"]
        destination = content / corpus["dest"].removeprefix("content/")
        for relative, expected in sorted(record["files"].items()):
            payload = (source / relative).read_bytes()
            if fetch.digest(payload) != expected:
                raise SystemExit(
                    f"{corpus['name']}/{relative} does not match its acquisition digest. "
                    f"Re-run build/acquire.py."
                )
            target = destination / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(payload)
            written += 1
    return written


# --------------------------------------------------------------------------- entry point


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", default=str(HERE / "manifests" / "datafusion-tracing.json"))
    parser.add_argument("--content", default=str(CONTENT))
    parser.add_argument(
        "--stage",
        choices=("model", "all"),
        default="all",
        help="`model` stops after the indexes, for iterating on the item table alone.",
    )
    arguments = parser.parse_args()

    manifest = json.loads(Path(arguments.manifest).read_text())
    content = Path(arguments.content)
    acquisition = load_acquisition()

    say("models:")
    models, classifications = build_models(manifest, acquisition)
    items, aliases, unresolved, _hidden_impls = model.stitch(models)

    say("visibility:")
    tally = apply_visibility(items, classifications)
    say(f"  {tally}")

    crate_facts = {
        package: fetch.parse_manifest_facts(
            (ACQUIRED / "manifests" / f"{package}.Cargo.toml").read_text()
        )
        for package in acquisition["crates"]
    }

    say("indexes:")
    counts = write_indexes(
        content, items, aliases, unresolved, crate_facts, classifications, acquisition
    )
    say(f"  {counts}")
    if arguments.stage == "model":
        sys.stdout.write(
            json.dumps({"canonical_items": len(items), "index": counts}, indent=2) + "\n"
        )
        return 0

    say("pages:")
    grouped = emit.group_by_module(items)
    emit.write_model(grouped, content)
    emit.write_api(grouped, content)
    say(f"  {len(grouped)} module(s)")

    say("corpus:")
    corpus_files = write_corpus(content, manifest, acquisition)
    say(f"  {corpus_files} file(s)")

    say("observed spans:")
    probe_tally = read_behaviours(content)
    span_report = spans.write_all(content, ACQUIRED / "corpus")
    say(f"  {span_report['spans']} span(s), {span_report['fields']} field row(s)")

    say("macro grammar:")
    macro_sources: dict[str, str] = {}
    for record in acquisition["crates"].values():
        if "private" in record:
            macro_sources.update(macros.sources_from(json.loads(document(record, ".private"))))
    macro_count = macros.write_all(content, items, macro_sources)
    say(f"  {macro_count} arm(s)")

    contract_counts = contracts.write(
        models, items, content, SKILL_ROOT / "authoring/contract-notes.json"
    )
    say(f"  contracts: {contract_counts}")

    say("compatibility:")
    compat_report = compat.write_all(content, acquisition["compatibility"], manifest)
    say(f"  {compat_report['releases']} release(s)")

    say("catalogs and pages:")
    catalog_counts = catalogs.write_all(content, items, classifications, acquisition)
    page_counts = pages.write_all(content, items, HERE / "topics.json")
    say(f"  {catalog_counts} | {page_counts}")

    say("router:")
    routed = router.write_questions(content, HERE / "router.json")
    say(f"  {routed} question(s)")

    say("rules:")
    generated = queries.generate(content / "model", QUERIES, items, classifications)
    say(f"  {generated}")

    shutil.copytree(ACQUIRED / "licenses", content / "licenses", dirs_exist_ok=True)
    capability_counts = capabilities.write(SKILL_ROOT, content)

    counts.update(
        {
            **probe_tally,
            **contract_counts,
            **capability_counts,
            "canonical_items": len(items),
            "modules": len(grouped),
            "corpus_files": corpus_files,
            "questions": routed,
            "macro_arms": macro_count,
            **{f"visibility_{k.replace('-', '_')}": v for k, v in tally.items()},
            **span_report,
            **compat_report,
            **catalog_counts,
            **page_counts,
        }
    )
    emit.write_provenance(
        content,
        manifest,
        crate_facts={
            package: {**record, **crate_facts.get(package, {})}
            for package, record in acquisition["crates"].items()
        },
        counts=counts,
        tool_versions={
            "ast_grep": ast_grep_version(),
            "rustdoc_format_supported": manifest["tools"]["rustdoc_format_supported"],
            "local_rustdoc": manifest["tools"]["local_rustdoc"],
        },
        corpora=acquisition["corpora"],
    )
    sys.stdout.write(json.dumps(counts, indent=2, sort_keys=True) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
