# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Exact development concepts adapted from their existing semantic owners.

Files carry source versions. Metadata, stable section owners, native ScopeItems,
producer triples and explicit original evidence adapters own concept meaning.
"""

from __future__ import annotations

import hashlib
import json
import time
from pathlib import Path

from scripts import (
    adr,
    document_lifecycle,
    document_metadata,
    evidence_projection,
    evidence_sources,
    workspace_domain_contracts,
    workspace_sources,
)

ROOT = Path(__file__).resolve().parents[1]
CONCEPTS = workspace_domain_contracts.CONCEPTS


def _identity(kind: str, native: str) -> str:
    return kind + ":" + native


def _subject(
    kind: str,
    native: str,
    owner: str,
    label: str,
    *,
    basis: str = "native",
    version: str | None = None,
) -> dict:
    return {
        "id": _identity(kind, native),
        "kind": kind,
        "owner": owner,
        "native_id": native,
        "label": label,
        "basis": basis,
        "native_state": None,
        "state_basis": None,
        "publication_scope": None,
        "document_role": None,
        "native_version": version,
        "evidence": None,
        "sources": [],
        "topics": [],
    }


class _Projection:
    def __init__(self, root: Path, limit: int) -> None:
        self.root = root
        self.limit = limit
        self.subjects: dict[str, dict] = {}
        self.facts: dict[str, dict] = {}
        self.edges: dict[str, dict] = {}
        self.productions: dict[str, dict] = {}
        self.ranges: list[dict] = []
        self.unknowns: list[str] = []
        self.omissions: list[str] = []
        self.native_facts: list[dict] = []
        self.read_bytes = 0
        self.deadline = time.monotonic() + 15

    def fact(self, path: str) -> dict | None:
        if path in self.facts:
            return self.facts[path]
        if (
            self.read_bytes >= 32 << 20
            or len(self.facts) >= 1024
            or time.monotonic() >= self.deadline
        ):
            self.omissions.append("owner source discovery/read bound exhausted")
            return None
        try:
            fact = workspace_sources.source(path, root=self.root)
            self.read_bytes += fact["bytes"]
            self.facts[path] = fact
            for item in fact.get("metadata_inputs", []):
                if item["path"] not in self.facts:
                    policy = workspace_sources.source(
                        item["path"], root=self.root, expected_digest=item["digest"]
                    )
                    self.facts[item["path"]] = policy
        except (OSError, UnicodeError, ValueError) as error:
            self.omissions.append(f"source unavailable: {path}: {error}")
            return None
        else:
            return fact

    def contribute(self, subject: dict, fact: dict, role: str) -> None:
        contributor = {"source_id": fact["path"], "role": role}
        if contributor not in subject["sources"]:
            subject["sources"].append(contributor)
        for item in fact.get("metadata_inputs", []):
            contributor = {"source_id": item["path"], "role": "interpretation_policy"}
            if contributor not in subject["sources"]:
                subject["sources"].append(contributor)
        if len(subject["sources"]) > 8:
            self.omissions.append("native contributor bound exceeded: " + subject["id"])
            subject["sources"] = subject["sources"][:8]

    def add(self, subject: dict) -> dict | None:
        if any(
            len(subject[field]) > 4096
            for field in ("id", "owner", "native_id", "label")
        ):
            self.omissions.append("native subject string bound exceeded")
            return None
        old = self.subjects.get(subject["id"])
        if old is not None:
            for name in (
                "kind",
                "owner",
                "native_id",
                "native_version",
                "native_state",
            ):
                if old[name] != subject[name]:
                    self.omissions.append(
                        "conflicting owner-native field " + name + ": " + subject["id"]
                    )
            for contributor in subject["sources"]:
                if contributor not in old["sources"]:
                    old["sources"].append(contributor)
            if len(old["sources"]) > 8:
                self.omissions.append("native contributor bound exceeded: " + old["id"])
                old["sources"] = old["sources"][:8]
            return old
        if len(self.subjects) >= self.limit:
            self.omissions.append("native subject bound exceeded")
            return None
        self.subjects[subject["id"]] = subject
        return subject

    def edge(self, source: str, target: str, relation: str) -> None:
        if source == target:
            return
        if source not in self.subjects or target not in self.subjects:
            self.unknowns.append(
                f"declared {relation} endpoint unavailable: {source} -> {target}"
            )
            return
        identity = json.dumps([relation, source, target], separators=(",", ":"))
        self.edges[identity] = {
            "id": identity,
            "from": source,
            "to": target,
            "relation": relation,
            "basis": "declared",
        }

    def document(self, fact: dict) -> dict | None:
        if Path(fact["path"]).suffix != ".md":
            return None
        if fact["role"] == "decision":
            return None
        existing = self.subjects.get(_identity("document", fact["path"]))
        if existing is not None:
            return existing
        subject = _subject(
            "document",
            fact["path"],
            fact["owner"],
            str(fact["metadata"].get("title", fact["path"])),
            version=fact["digest"],
        )
        subject.update(
            native_state=fact["native_status"],
            state_basis="authored document metadata"
            if fact["native_status"] is not None
            else None,
            publication_scope=fact["scope"],
            document_role=fact["role"],
        )
        subject["topics"] = [
            {"topic": topic, "source_id": fact["path"]} for topic in fact["topics"][:16]
        ]
        if len(fact["topics"]) > 16:
            self.omissions.append("native topic bound exceeded: " + subject["id"])
        self.contribute(subject, fact, "metadata")
        kept = self.add(subject)
        owner_path, fragment, _ = fact["owner"].partition("#")
        if owner_path != fact["path"] and not owner_path.startswith(("https:", "git:")):
            owner_fact = self.fact(owner_path)
            if owner_fact:
                self.document(owner_fact)
                if fragment:
                    self.unknowns.append(
                        "document owner fragment retained; typed fragment relationship undeclared: "
                        + fact["owner"]
                    )
                else:
                    self.edge(_identity("document", owner_path), subject["id"], "owns")
        return kept

    def native_document_subject(
        self, fact: dict, kind: str, native: str, label: str
    ) -> dict | None:
        subject = _subject(kind, native, fact["owner"], label, version=fact["digest"])
        subject.update(
            native_state=fact["native_status"],
            state_basis="authored document metadata"
            if fact["native_status"] is not None
            else None,
            publication_scope=fact["scope"],
            document_role=fact["role"],
        )
        subject["topics"] = [
            {"topic": topic, "source_id": fact["path"]} for topic in fact["topics"][:16]
        ]
        self.contribute(subject, fact, "metadata")
        owner_path, fragment, _ = fact["owner"].partition("#")
        owner_fact = None
        if owner_path != fact["path"] and not owner_path.startswith(("https:", "git:")):
            owner_fact = self.fact(owner_path)
            if owner_fact:
                self.contribute(subject, owner_fact, "metadata")
                self.document(owner_fact)
        selected = self.add(subject)
        if selected and kind != "decision":
            self.edge(_identity("document", fact["path"]), selected["id"], "contains")
        if selected and owner_fact:
            if fragment:
                self.unknowns.append(
                    "document owner fragment retained; typed fragment relationship undeclared: "
                    + fact["owner"]
                )
            else:
                self.edge(_identity("document", owner_path), selected["id"], "owns")
        return selected

    def result(self) -> dict:
        # Recheck all source and interpretation bytes before returning one projection.
        for path, fact in self.facts.items():
            if fact.get("caller_input") or fact.get("original_evidence"):
                continue
            try:
                observed = workspace_sources.source(
                    path, root=self.root, expected_digest=fact["digest"]
                )
                if observed.get("metadata_inputs", []) != fact.get(
                    "metadata_inputs", []
                ):
                    self.omissions.append(
                        "metadata policy changed during projection: " + path
                    )
            except (OSError, ValueError, UnicodeError):
                self.omissions.append("source changed during projection: " + path)
        return {
            "schema": "workspace-domains-v1",
            "subjects": sorted(self.subjects.values(), key=lambda item: item["id"]),
            "edges": sorted(self.edges.values(), key=lambda item: item["id"]),
            "productions": sorted(
                self.productions.values(), key=lambda item: item["id"]
            ),
            "source_facts": [fact for _, fact in sorted(self.facts.items())],
            "section_ranges": self.ranges,
            "native_facts": self.native_facts,
            "complete": not self.omissions,
            "omissions": list(dict.fromkeys(self.omissions)),
            "unknowns": list(dict.fromkeys(self.unknowns)),
            "coverage": "exact selected owner-native declarations; undeclared links remain unknown; no database or language server used",
        }


def _sections(
    projection: _Projection, selected: set[str], identity: str | None = None
) -> None:
    blueprint = projection.root / "docs/authoritative_design/blueprint.md"
    if not blueprint.is_file():
        projection.unknowns.append("stable contract-section owner index unavailable")
        return
    try:
        owners = adr.section_owners(blueprint)
    except (OSError, ValueError) as error:
        projection.omissions.append(
            "contract-section owner index unavailable: " + str(error)
        )
        return
    index_fact = projection.fact(blueprint.relative_to(projection.root).as_posix())
    for section, (path, heading) in owners.items():
        relative = path.relative_to(projection.root).as_posix()
        if (identity is not None and section != identity) or (
            identity is None and relative not in selected
        ):
            continue
        fact = projection.fact(relative)
        if fact is None:
            continue
        if fact["role"] != "contract":
            projection.unknowns.append(
                "section owner is not classified as a contract: " + relative
            )
            continue
        projection.document(fact)
        subject = projection.native_document_subject(
            fact, "contract", section, heading.lstrip("# ")
        )
        if subject is None:
            continue
        if index_fact:
            projection.contribute(subject, index_fact, "interpretation_policy")
        # The same authoritative heading reader supplies exact numbered-section ranges.
        numbered = [
            (position, line)
            for position, line in adr.markdown_headings(fact["text"])
            if adr.HEADING_RE.match(line)
        ]
        starts = [index for index, (_, line) in enumerate(numbered) if line == heading]
        if len(starts) != 1:
            projection.omissions.append(
                "section heading source cut differs from owner index: " + section
            )
            continue
        index = starts[0]
        start = numbered[index][0] + 1
        end = (
            numbered[index + 1][0]
            if index + 1 < len(numbered)
            else len(fact["text"].splitlines())
        )
        projection.ranges.append(
            {
                "subject_id": subject["id"],
                "source_id": relative,
                "section_id": section,
                "start_line": start,
                "end_line": end,
            }
        )


def _decisions(projection: _Projection, facts: list[dict]) -> None:
    decisions = []
    for fact in facts:
        metadata = fact.get("metadata", {})
        if fact["role"] != "decision":
            continue
        native = metadata.get("id")
        if not isinstance(native, str) or not native:
            projection.unknowns.append(
                "decision has no authored native ID: " + fact["path"]
            )
            continue
        subject = projection.native_document_subject(
            fact, "decision", native, str(metadata.get("title", native))
        )
        if subject:
            decisions.append((subject, metadata))
    for subject, metadata in decisions:
        supersedes = metadata.get("supersedes", [])
        if not isinstance(supersedes, list) or any(
            not isinstance(value, str) for value in supersedes
        ):
            projection.omissions.append(
                "invalid authored decision supersedes list: " + subject["native_id"]
            )
            continue
        for native in supersedes:
            projection.edge(
                _identity("decision", native), subject["id"], "superseded_by"
            )
        successor = metadata.get("superseded-by")
        if isinstance(successor, str) and successor:
            projection.edge(
                subject["id"], _identity("decision", successor), "superseded_by"
            )


def _packages(
    projection: _Projection, documents: set[str], *, identity: str | None = None
) -> None:
    if not documents:
        return

    def read_source(path: str) -> bytes:
        fact = projection.fact(path)
        if fact is None:
            raise ValueError("native scope source unavailable: " + path)
        return fact["text"].encode("utf-8")

    try:
        policy = document_metadata.load_policy(projection.root, read_source=read_source)
        rows = document_lifecycle.aggregate(
            projection.root, policy, documents=documents, read_source=read_source
        )
    except (OSError, ValueError) as error:
        projection.omissions.append("native package scope unavailable: " + str(error))
        return
    packages = {
        row["id"]: row for row in rows if row.get("kind") in {"package", "packet"}
    }
    all_rows = {row["id"]: row for row in rows}
    selected = {
        key
        for key, row in packages.items()
        if row["document"] in documents
        and (identity is None or key == identity or row.get("native_id") == identity)
    }
    if identity is not None and not selected:
        projection.unknowns.append(
            "named native package not found; no state inferred from headings or prose"
        )
    # Traverse only actual authored relationships, retaining canonical status owners.
    while True:
        expanded = selected | {
            edge["target"]
            for key in selected
            for edge in packages[key].get("relationships", [])
            if edge["target"] in packages
        }
        if expanded == selected:
            break
        if len(expanded) > projection.limit:
            projection.omissions.append(
                "native package relationship closure exceeds subject bound"
            )
            break
        selected = expanded
    for key in sorted(selected):
        row = packages[key]
        fact = projection.fact(row["document"])
        if fact is None:
            continue
        projection.document(fact)
        subject = _subject(
            "plan_package",
            key,
            row.get("owner", row["document"]),
            row.get("label") or row.get("native_id") or key,
            version=fact["digest"],
        )
        subject.update(
            native_state=row["state"],
            state_basis=row["status_basis"],
            publication_scope=fact["scope"],
            document_role=fact["role"],
        )
        subject["topics"] = [
            {"topic": topic, "source_id": fact["path"]} for topic in fact["topics"][:16]
        ]
        projection.contribute(subject, fact, "native_scope")
        projection.contribute(subject, fact, "metadata")
        status_chain = []
        status_key = key
        while status_key in all_rows and status_key not in {
            item["id"] for item in status_chain
        }:
            status_row = all_rows[status_key]
            status_chain.append(status_row)
            status_fact = projection.fact(status_row["document"])
            if status_fact:
                projection.contribute(subject, status_fact, "native_scope")
            targets = [
                edge["target"]
                for edge in status_row.get("relationships", [])
                if edge["kind"] == "status-owned-by"
            ]
            if not targets:
                break
            status_key = targets[0]
        if not status_chain or status_chain[-1]["id"] != row.get("status_owner", key):
            projection.omissions.append(
                "canonical native package status owner chain unavailable: " + key
            )
        if projection.add(subject):
            projection.edge(
                _identity("document", row["document"]), subject["id"], "contains"
            )
            projection.native_facts.append(
                {
                    "subject_id": subject["id"],
                    "scope_item": row,
                    "status_owner_chain": status_chain,
                }
            )
            if isinstance(row.get("source_line"), int):
                projection.ranges.append(
                    {
                        "subject_id": subject["id"],
                        "source_id": row["document"],
                        "native_scope_id": key,
                        "start_line": row["source_line"],
                        "end_line": row["source_line"],
                    }
                )
    relations = {
        "depends-on": "depends_on",
        "status-owned-by": "status_owned_by",
        "superseded-by": "superseded_by",
    }
    for key in sorted(selected):
        for edge in packages[key].get("relationships", []):
            if edge["kind"] in relations:
                if edge["target"] not in packages:
                    projection.omissions.append(
                        "native package relationship endpoint kind has no typed adapter: "
                        + edge["target"]
                    )
                projection.edge(
                    _identity("plan_package", key),
                    _identity("plan_package", edge["target"]),
                    relations[edge["kind"]],
                )


def _productions(projection: _Projection, predicates: dict[str, str]) -> None:
    try:
        selected = workspace_sources.relationships(
            root=projection.root, predicates=predicates, limit=100
        )
    except (OSError, ValueError) as error:
        projection.omissions.append("producer declarations unavailable: " + str(error))
        return
    if selected.get("truncated"):
        projection.omissions.append(
            "exact producer selection exceeds its finite declaration/read bound"
        )
    projection.unknowns.append(
        "producer declaration fingerprint freshness against current registry is unverified"
    )
    producer_documents = {
        fact["producer"]: fact for fact in selected.get("producer_documents", [])
    }
    for row in selected["items"]:
        carrier = producer_documents.get(row["producer"])
        if carrier is None:
            projection.omissions.append(
                "producer row lacks its authenticated source carrier"
            )
            continue
        declaration_fact = projection.fact(carrier["path"])
        if declaration_fact is None:
            continue
        if declaration_fact["digest"] != carrier["digest"]:
            projection.omissions.append(
                "producer declaration changed during projection: " + carrier["path"]
            )
            continue
        declaration_native = row["declaration_kind"] + ":" + row["declaration_key"]
        declaration = _subject(
            "declaration",
            declaration_native,
            row.get("source_path") or carrier["path"],
            row["declaration_key"],
            basis="declared",
            version=row.get("source_identity"),
        )
        generator_native = row["generator_path"] + "#" + row["generator_symbol"]
        generator_fact = projection.fact(row["generator_path"])
        generator = _subject(
            "generator",
            generator_native,
            row["generator_path"],
            generator_native,
            basis="declared",
            version=generator_fact["digest"] if generator_fact else None,
        )
        output_fact = projection.fact(row["output_path"])
        artifact = _subject(
            "generated_artifact",
            row["output_path"],
            "generated-artifact",
            row["output_path"],
            basis="declared",
            version=output_fact["digest"] if output_fact else None,
        )
        for subject in (declaration, generator, artifact):
            projection.contribute(subject, declaration_fact, "producer_declaration")
        if generator_fact:
            projection.contribute(generator, generator_fact, "definition")
        if output_fact:
            projection.contribute(artifact, output_fact, "definition")
        if row.get("source_path"):
            source_fact = projection.fact(row["source_path"])
            if source_fact:
                role = (
                    "catalogue_registration"
                    if row["source_role"] == "catalogue_registration"
                    else "definition"
                    if row["source_role"] == "type_definition"
                    else "producer_declaration"
                )
                projection.contribute(declaration, source_fact, role)
        elif row["source_role"] == "semantic_declaration_without_source_location":
            projection.unknowns.append(
                "semantic declaration source location unavailable: "
                + declaration_native
            )
        kept = [
            projection.add(subject) for subject in (declaration, generator, artifact)
        ]
        if not all(kept):
            continue
        triple = {
            "declaration_id": declaration["id"],
            "generator_id": generator["id"],
            "artifact_id": artifact["id"],
            "direction": row.get("direction"),
            "basis": "declared",
        }
        identity = hashlib.sha256(
            json.dumps([row["producer"], triple], sort_keys=True).encode()
        ).hexdigest()
        projection.productions[identity] = {"id": identity, **triple}
        sections = row.get("contract_sections", [])
        if (
            not isinstance(sections, list)
            or len(sections) > 16
            or any(not isinstance(section, str) for section in sections)
        ):
            raise ValueError(
                "producer contract sections require finite stable identities"
            )
        for section in sections:
            _sections(projection, set(), section)
            projection.edge(_identity("contract", section), artifact["id"], "governs")
        projection.native_facts.append(
            {
                "production_id": identity,
                "producer_row": row,
                "producer_freshness": selected.get("producer_freshness", "unverified"),
            }
        )


def project(paths: tuple[str, ...], *, root: Path = ROOT, limit: int = 256) -> dict:
    """Project only exact declarations over explicitly selected original source paths."""
    if not paths or len(paths) > 64 or not 1 <= limit <= 256:
        raise ValueError("select 1..64 sources and a native subject bound in 1..256")
    projection = _Projection(root, limit)
    facts = [
        fact
        for path in dict.fromkeys(paths)
        if (fact := projection.fact(path)) is not None
    ]
    for fact in facts:
        projection.document(fact)
    _decisions(projection, facts)
    if any(fact["role"] == "contract" for fact in facts):
        _sections(projection, set(paths))
    plans = {fact["path"] for fact in facts if fact["role"] == "plan"}
    plans.update(
        fact["owner"].partition("#")[0] for fact in facts if fact["role"] == "plan"
    )
    _packages(projection, plans)
    for fact in facts:
        if Path(fact["path"]).suffix != ".md":
            for field in ("source_path", "generator_path", "output_path"):
                _productions(projection, {field: fact["path"]})
    if not projection.subjects:
        projection.unknowns.append(
            "no owner-native development concept declared in the selected source population"
        )
    return projection.result()


def _discovery(root: Path, prefixes: tuple[str, ...]) -> tuple[list[str], list[str]]:
    selected = workspace_sources.paths(
        root=root, roots=prefixes, max_files=1024, max_bytes=1 << 20, seconds=5
    )
    return [
        path for path in selected["paths"] if Path(path).suffix == ".md"
    ], [] if selected["complete"] else ["native document discovery bound exhausted"]


def _selected_reference(projection: _Projection, kind: str, selection: dict) -> None:
    adapter = selection["adapter"]
    if kind == "experiment":
        projection.unknowns.append(
            "experiment concept adapter not implemented; generic evidence is not a new experiment lifecycle"
        )
        return
    path = selection.get("path")
    if not isinstance(path, str) or not path or len(path) > 4096:
        raise ValueError("explicit bounded original evidence path/handle required")
    selected_path = Path(path)
    if not selected_path.is_absolute():
        selected_path = projection.root / selected_path
    if adapter == "diagnostic":
        snapshot = evidence_sources.diagnostic_snapshot(selected_path)
    elif adapter == "assessment":
        snapshot = evidence_sources.assessment_snapshot(
            selected_path, root=projection.root
        )
    else:
        snapshot = evidence_projection.invocation_snapshot(path)
    if selection.get("codec") not in {None, snapshot["codec"]}:
        projection.unknowns.append(
            "selected codec differs from the original evidence owner's codec"
        )
        return
    if snapshot["coverage"] != "complete":
        projection.omissions.append("original evidence snapshot coverage is partial")
    member = selection.get("member")
    if kind == "diagnostic":
        if member not in {None, "action"}:
            projection.unknowns.append(
                "diagnostic identity belongs to the original action; other members are evidence references"
            )
            return
        member = "action"
    facts = [
        fact for fact in snapshot["facts"] if member is None or fact["member"] == member
    ]
    if member is not None and not facts:
        projection.unknowns.append("selected original evidence member is not declared")
        return
    loaded_artifacts: set[str] = set()
    for fact in facts[:32]:
        native = json.dumps(
            [
                snapshot["authority"],
                snapshot["codec"],
                snapshot["source_id"],
                snapshot["snapshot_digest"],
                fact["member"],
            ],
            separators=(",", ":"),
        )
        subject = _subject(
            kind,
            native,
            snapshot["authority"],
            fact["member"],
            version=snapshot["snapshot_digest"],
        )
        if kind == "evidence_reference":
            subject["evidence"] = {
                "authority": snapshot["authority"],
                "codec": snapshot["codec"],
                "source_id": snapshot["source_id"],
                "version": snapshot["snapshot_digest"],
                "member": fact["member"],
            }
        # This is a compact owner-decoded source reference, not a generic file record.
        source_id = (
            "evidence-source:"
            + hashlib.sha256(
                json.dumps(
                    [
                        snapshot["authority"],
                        snapshot["codec"],
                        snapshot["source_id"],
                        snapshot["snapshot_digest"],
                    ]
                ).encode()
            ).hexdigest()
        )
        projection.facts[source_id] = {
            "id": source_id,
            "path": source_id,
            "kind": "owner-evidence-reference",
            "locator": snapshot["source_id"],
            "digest": snapshot["snapshot_digest"],
            "role": "evidence",
            "scope": "Reference",
            "owner": snapshot["authority"],
            "native_status": None,
            "metadata_inputs": [],
            "metadata": {},
            "topics": [],
            "basis": "owner-decoded explicit original evidence snapshot",
            "original_evidence": {
                key: value for key, value in snapshot.items() if key != "facts"
            },
        }
        subject["sources"] = [{"source_id": source_id, "role": "owner_evidence"}]
        projection.add(subject)
        projection.native_facts.append(
            {"subject_id": subject["id"], "original_fact": fact, "adapter": adapter}
        )
        if adapter == "assessment" and fact.get("link_relation") in {
            "selected_contract",
            "executed_contract",
        }:
            section = fact.get("link_target")
            artifact = fact.get("detail_ref")
            if isinstance(section, str) and isinstance(artifact, str):
                _sections(projection, set(), section)
                target = _identity("contract", section)
                # Failed execution remains failed evidence; supports means an
                # observed contract exercise, never an assertion of correctness.
                if fact["link_relation"] == "executed_contract":
                    projection.edge(subject["id"], target, "supports")
                else:
                    projection.edge(target, subject["id"], "references")
                if artifact not in loaded_artifacts:
                    _productions(projection, {"output_path": artifact})
                    loaded_artifacts.add(artifact)
    if len(facts) > 32:
        projection.omissions.append("selected evidence member bound exceeded")


def _select_query_context(result: dict, items: list[dict]) -> None:
    """Retain selected subjects and declared context, never unrelated page payloads.

    Semantic relationship closure carries native status/dependencies and decision
    supersession. Container/owner links add their direct context without expanding
    into unrelated sibling sections. Producer triples remain indivisible.
    """
    selected = {item["id"] for item in items}
    semantic_relations = {
        "depends_on",
        "status_owned_by",
        "superseded_by",
        "governs",
        "references",
        "supports",
    }
    while True:
        related = selected | {
            endpoint
            for edge in result["edges"]
            if edge["relation"] in semantic_relations
            and (edge["from"] in selected or edge["to"] in selected)
            for endpoint in (edge["from"], edge["to"])
        }
        related.update(
            row[field]
            for row in result["productions"]
            if related.intersection(
                (row["declaration_id"], row["generator_id"], row["artifact_id"])
            )
            for field in ("declaration_id", "generator_id", "artifact_id")
        )
        if related == selected:
            break
        selected = related
    productions = [
        row
        for row in result["productions"]
        if selected.intersection(
            (row["declaration_id"], row["generator_id"], row["artifact_id"])
        )
    ]
    selected.update(
        row[field]
        for row in productions
        for field in ("declaration_id", "generator_id", "artifact_id")
    )
    edges = [
        edge
        for edge in result["edges"]
        if edge["from"] in selected or edge["to"] in selected
    ]
    selected.update(
        endpoint for edge in edges for endpoint in (edge["from"], edge["to"])
    )
    result["subjects"] = [row for row in result["subjects"] if row["id"] in selected]
    result["edges"] = edges
    result["productions"] = productions
    result["section_ranges"] = [
        row for row in result["section_ranges"] if row["subject_id"] in selected
    ]
    production_ids = {row["id"] for row in productions}
    result["native_facts"] = [
        row
        for row in result["native_facts"]
        if row.get("subject_id") in selected
        or row.get("production_id") in production_ids
    ]


def query(
    request: workspace_domain_contracts.ConceptQuery,
    *,
    root: Path = ROOT,
) -> dict:
    """Exact conjunction over an owner-validated request; no ranking fallback."""
    request = workspace_domain_contracts.parse_query(request)
    concept = request.concept
    identity = getattr(request, "identity", None)
    topics = getattr(request, "topics", [])
    role = getattr(request, "role", None)
    owner = getattr(request, "owner", None)
    document = getattr(request, "document", None)
    publication_scope = getattr(request, "publication_scope", None)
    native_status = getattr(request, "native_status", None)
    limit, offset = request.limit, request.offset
    projection = _Projection(root, 256)
    kind = CONCEPTS[concept]
    if isinstance(request, workspace_domain_contracts.ResearchQuestionQuery):
        identity = request.identity
        origin = request.origin
        question = request.question
        subject = _subject(kind, identity, "caller", question, basis=origin)
        source_id = (
            "caller-input:"
            + hashlib.sha256(json.dumps([identity, origin]).encode()).hexdigest()
        )
        projection.facts[source_id] = {
            "id": source_id,
            "path": source_id,
            "kind": "question-input",
            "locator": "caller:" + identity,
            "digest": hashlib.sha256(
                json.dumps([identity, origin, question]).encode()
            ).hexdigest(),
            "text": question,
            "role": "question-input",
            "scope": "Reference",
            "owner": "caller",
            "native_status": None,
            "metadata_inputs": [],
            "metadata": {},
            "topics": [],
            "basis": origin,
            "caller_input": True,
        }
        subject["sources"] = [
            {
                "source_id": source_id,
                "role": "question_input"
                if origin == "user-question"
                else "interpretation_input",
            }
        ]
        projection.add(subject)
        projection.unknowns.append(
            "question has no newly invented lifecycle; answers/support links require explicit owner declarations"
        )
    elif isinstance(
        request,
        (
            workspace_domain_contracts.ExperimentQuery,
            workspace_domain_contracts.DiagnosticQuery,
            workspace_domain_contracts.EvidenceReferenceQuery,
        ),
    ):
        _selected_reference(
            projection, kind, request.selected_reference.model_dump(exclude_none=True)
        )
    elif concept in {"declaration", "generator", "generated-output"}:
        if identity is None:
            raise ValueError("producer concept requires an exact native identity")
        if concept == "generated-output":
            predicates = {"output_path": identity}
        elif concept == "generator":
            path, separator, symbol = identity.partition("#")
            predicates = {
                "generator_path": path,
                **({"generator_symbol": symbol} if separator else {}),
            }
        else:
            declaration_kind, separator, key = identity.partition(":")
            predicates = (
                {"declaration_key": key, "declaration_kind": declaration_kind}
                if separator
                else {"declaration_key": identity}
            )
        _productions(projection, predicates)
    elif concept == "contract-section":
        if identity is None:
            raise ValueError("contract-section requires an exact stable section ID")
        _sections(projection, set(), identity)
    elif concept == "plan-package":
        if identity is None:
            raise ValueError(
                "plan-package requires an exact native ScopeItem ID or document#package"
            )
        if "::" in identity:
            plan = identity.split("::", 1)[0]
            native = identity
        else:
            plan, marker, native = identity.partition("#")
            if not marker:
                raise ValueError(
                    "short package identity requires selected document#native-package"
                )
        fact = projection.fact(plan)
        if fact:
            _packages(
                projection, {plan, fact["owner"].partition("#")[0]}, identity=native
            )
    else:
        if concept == "document" and identity:
            paths = [identity]
        else:
            paths, omissions = _discovery(
                root,
                ("docs/adr",)
                if concept == "decision"
                else ((document,) if document else ("docs",)),
            )
            projection.omissions.extend(omissions)
        facts = [fact for path in paths if (fact := projection.fact(path)) is not None]
        if concept == "document":
            facts = [
                fact
                for fact in facts
                if (owner is None or fact["owner"] == owner)
                and (role is None or fact["role"] == role)
                and set(topics) <= set(fact["topics"])
                and (publication_scope is None or fact["scope"] == publication_scope)
                and (native_status is None or fact["native_status"] == native_status)
            ]
        if concept == "decision" and identity is not None:
            by_id = {
                fact["metadata"].get("id"): fact
                for fact in facts
                if fact["role"] == "decision"
            }
            chosen = [
                fact
                for fact in facts
                if fact["role"] == "decision" and fact["metadata"].get("id") == identity
            ]
            linked_ids = {
                value
                for fact in chosen
                for value in fact["metadata"].get("supersedes", [])
            }
            linked_ids.update(
                fact["metadata"]["superseded-by"]
                for fact in chosen
                if isinstance(fact["metadata"].get("superseded-by"), str)
            )
            facts = chosen + [
                by_id[native]
                for native in linked_ids
                if native in by_id and by_id[native] not in chosen
            ]
        for fact in facts:
            projection.document(fact)
        if concept == "decision":
            _decisions(projection, facts)
    result = projection.result()
    candidates = [item for item in result["subjects"] if item["kind"] == kind]
    if identity is not None and concept not in {
        "plan-package",
        "generator",
        "declaration",
    }:
        candidates = [item for item in candidates if item["native_id"] == identity]
    if concept == "plan-package" and identity:
        candidates = [
            item
            for item in candidates
            if item["native_id"] == identity
            or (
                item["native_id"].endswith("::" + identity.partition("#")[2])
                and "#" in identity
            )
        ]
    if owner is not None:
        candidates = [item for item in candidates if item["owner"] == owner]
    if role is not None:
        candidates = [item for item in candidates if item["document_role"] == role]
    if topics:
        candidates = [
            item
            for item in candidates
            if set(topics) <= {topic["topic"] for topic in item["topics"]}
        ]
    if publication_scope is not None:
        candidates = [
            item
            for item in candidates
            if item["publication_scope"] == publication_scope
        ]
    if native_status is not None:
        candidates = [
            item for item in candidates if item["native_state"] == native_status
        ]
    if document is not None:
        candidates = [
            item
            for item in candidates
            if any(source["source_id"] == document for source in item["sources"])
        ]
    result.update(
        query={
            "concept": concept,
            "identity": identity,
            "owner": owner,
            "role": role,
            "topics": list(topics),
            "document": document,
            "publication_scope": publication_scope,
            "native_status": native_status,
        },
        items=candidates[offset : offset + limit],
        next_offset=offset + limit if len(candidates) > offset + limit else None,
    )
    result["population"] = {
        "matched_count": len(candidates),
        "returned_count": len(result["items"]),
        "offset": offset,
        "limit": limit,
        "complete": result["complete"],
    }
    _select_query_context(result, result["items"])
    result["adapter_status"] = (
        "not_implemented"
        if concept == "experiment"
        or (concept in {"evidence-reference", "diagnostic"} and not candidates)
        else "available"
        if candidates
        else "no_declared_match"
    )
    if not candidates:
        result["unknowns"].append(
            "no exact concept in inspected owner population; no text or semantic match substituted"
        )
    result["unknowns"].extend(
        [
            "governing-contract links not explicitly declared by inspected owners remain unknown",
            "qualification-evidence links not explicitly declared by inspected owners remain unknown",
        ]
    )
    used = {
        source["source_id"]
        for subject in result["subjects"]
        for source in subject["sources"]
    }
    result["source_facts"] = [
        {key: value for key, value in fact.items() if key != "text"}
        for fact in result["source_facts"]
        if fact["path"] in used
    ]
    return result
