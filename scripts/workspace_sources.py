# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Selected source facts and producer links; no service or writable authority."""

from __future__ import annotations

import hashlib
import json
import os
import re
import selectors
import subprocess
import time
import tomllib
from functools import lru_cache
from pathlib import Path

from scripts import docs, document_metadata

ROOT = Path(__file__).resolve().parents[1]
SUFFIXES = {".md", ".rs", ".py", ".pyi", ".toml", ".surql", ".json", ".yml", ".yaml", ".lock", ".patch"}
EXCLUDED = {"external", "build", "target", ".git", "book", "node_modules", "__pycache__", ".store", ".cache", "secrets", "credentials"}


def _command(root: Path, argv: list[str], *, seconds: float = 2, max_bytes: int = 4 << 20, input_bytes: bytes | None = None) -> dict:
    """Bound both pipe memory and elapsed time, including pathological rg lines."""
    deadline = time.monotonic() + seconds
    output = bytearray()
    errors = bytearray()
    complete = True
    process = subprocess.Popen(argv, cwd=root, stdin=subprocess.PIPE if input_bytes is not None else subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    pending = memoryview(input_bytes or b"")
    with selectors.DefaultSelector() as selector:
        for stream in (process.stdout, process.stderr):
            os.set_blocking(stream.fileno(), False)
            selector.register(stream, selectors.EVENT_READ)
        if process.stdin:
            if pending:
                os.set_blocking(process.stdin.fileno(), False)
                selector.register(process.stdin, selectors.EVENT_WRITE)
            else:
                process.stdin.close()
        try:
            while selector.get_map():
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    complete = False
                    break
                for key, event in selector.select(min(remaining, 0.1)):
                    stream = key.fileobj
                    if event & selectors.EVENT_WRITE:
                        try:
                            written = os.write(stream.fileno(), pending[:16384])
                            pending = pending[written:]
                        except BrokenPipeError:
                            pending = memoryview(b"")
                        if not pending:
                            selector.unregister(stream)
                            stream.close()
                    else:
                        chunk = os.read(stream.fileno(), 65536)
                        if not chunk:
                            selector.unregister(stream)
                            stream.close()
                        elif stream is process.stdout:
                            room = max_bytes - len(output)
                            output.extend(chunk[:room])
                            if len(chunk) > room:
                                complete = False
                                break
                        elif len(errors) < 2048:
                            errors.extend(chunk[:2048-len(errors)])
                if not complete:
                    break
        finally:
            if complete:
                try:
                    process.wait(timeout=max(0.001, deadline-time.monotonic()))
                except subprocess.TimeoutExpired:
                    complete = False
            if process.poll() is None:
                process.kill()
            process.wait()
            for stream in (process.stdin, process.stdout, process.stderr):
                if stream and not stream.closed:
                    stream.close()
    return {"stdout": bytes(output), "stderr": bytes(errors), "complete": complete, "returncode": process.returncode}


def git(root: Path, *args: str) -> str:
    result = _command(root, ["git", *args], max_bytes=65536)
    return result["stdout"].decode("utf-8").strip() if result["complete"] and result["returncode"] == 0 else ""


def selected_path(root: Path, relative: str, *, allow_missing: bool = False) -> Path:
    candidate = Path(relative)
    if candidate.is_absolute() or ".." in candidate.parts or not candidate.parts:
        raise ValueError("source must be a repository-relative path")
    if any(part in EXCLUDED or part.startswith(".venv") for part in candidate.parts):
        raise ValueError("source belongs to an excluded environment/storage root")
    if candidate.name == "history.txt" or candidate.name.startswith(".env") or candidate.suffix not in SUFFIXES:
        raise ValueError("source is not an admitted code/document input")
    path = root
    for part in candidate.parts:
        path /= part
        if path.is_symlink():
            raise ValueError("source cannot traverse symbolic links")
    if not path.resolve().is_relative_to(root.resolve()) or (not path.is_file() and not (allow_missing and not path.exists())):
        raise ValueError("source is absent or outside selected checkout")
    return path


def _read(root: Path, relative: str, max_bytes: int) -> bytes:
    path = selected_path(root, relative)
    before = path.stat()
    if before.st_size > max_bytes:
        raise ValueError("selected source exceeds byte bound")
    with path.open("rb") as stream:
        raw = stream.read(max_bytes + 1)
    after = path.stat()
    if len(raw) > max_bytes or (before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns, before.st_ctime_ns) != (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns, after.st_ctime_ns):
        raise ValueError("selected source changed during read")
    return raw


def _stamp(path: Path) -> tuple[int, int, int, int, int]:
    info = path.stat()
    return info.st_dev, info.st_ino, info.st_size, info.st_mtime_ns, info.st_ctime_ns


@lru_cache(maxsize=8)
def _document_context(root: Path, policy_stamp: tuple, site_stamp: tuple) -> tuple[dict, dict, list[dict]]:
    """Reuse one native interpretation per unchanged policy cut, not per result row."""
    inputs = []
    for relative, expected in (("docs/lifecycle.toml", policy_stamp), ("docs/site.toml", site_stamp)):
        raw = _read(root, relative, 4 << 20)
        if _stamp(root / relative) != expected:
            raise ValueError("metadata interpretation changed during selection")
        inputs.append({"path": relative, "digest": hashlib.sha256(raw).hexdigest(), "role": "metadata-interpretation"})
    policy, site = document_metadata.load_policy(root), docs.configuration(root)
    if _stamp(root / "docs/lifecycle.toml") != policy_stamp or _stamp(root / "docs/site.toml") != site_stamp:
        raise ValueError("metadata interpretation changed during read")
    return policy, site, inputs


def source(relative: str, *, root: Path = ROOT, revision: str | None = None, max_bytes: int = 4 << 20, expected_digest: str | None = None) -> dict:
    raw = _read(root, relative, max_bytes)
    digest = hashlib.sha256(raw).hexdigest()
    if expected_digest and digest != expected_digest:
        raise ValueError("selected source changed since selection")
    text = raw.decode("utf-8")
    role, scope, owner = "source", "Current", relative
    metadata, metadata_inputs, topics = {}, [], []
    scope_basis = "current source bytes; no document authority inferred"
    if Path(relative).suffix == ".md":
        document = document_metadata.read(raw, relative, identity=relative, root=root)
        metadata = dict(document.metadata)
        if (root / 'docs/lifecycle.toml').is_file() and (root / 'docs/site.toml').is_file():
            policy, site, metadata_inputs = _document_context(root, _stamp(root / 'docs/lifecycle.toml'), _stamp(root / 'docs/site.toml'))
            metadata.update(document_metadata.interpreted(root, root / relative, document, policy, site))
        role = str(metadata.get("doc_role", "reference"))
        owner = str(metadata.get("doc_owner", relative))
        topics = list(metadata.get('doc_topics', []))
        status = metadata.get("status", "")
        if not isinstance(status, str):
            raise ValueError("document status must be a string")
        scope = None
        if relative.startswith("docs/") and (root / "docs/site.toml").is_file():
            scope = docs.document_scope(Path(relative).relative_to("docs"), status, docs.configuration(root))
        scope_basis = "native publication scope" if scope else "unpublished or unclassified document; authority unresolved"
        scope = scope or "Reference"
    return {"id": relative, "path": relative, "digest": digest, "revision": revision, "revision_basis": "HEAD context only; content digest identifies dirty and clean bytes", "role": role, "scope": scope, "scope_basis": scope_basis, "owner": owner, "topics":topics, "native_id":metadata.get('id', relative), "native_status":metadata.get('status'), "metadata":metadata, "metadata_inputs":metadata_inputs, "text": text, "bytes": len(raw), "basis": "exact selected file content"}


def paths(*, root: Path = ROOT, roots: tuple[str, ...] = (), max_files: int = 4096, max_bytes: int = 4 << 20, seconds: float = 3) -> dict:
    if max_files <= 0 or max_bytes <= 0 or seconds <= 0:
        raise ValueError("positive discovery bounds required")
    for prefix in roots:
        if Path(prefix).is_absolute() or ".." in Path(prefix).parts or not prefix or prefix.startswith(":"):
            raise ValueError("selected roots must be literal repository-relative paths")
    deadline = time.monotonic()+seconds
    result = _command(root, ["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard", "--", *(":(literal)"+prefix for prefix in roots)], seconds=seconds, max_bytes=max_bytes)
    if result["returncode"] != 0 and result["complete"]:
        raise ValueError("source discovery requires an explicit Git checkout")
    complete = result["complete"]
    candidates = result["stdout"].split(b"\0")
    if not complete:
        candidates = candidates[:-1]  # an unfinished filename is not a path
    output = set()
    excluded = 0
    for name in candidates:
        if time.monotonic()>=deadline:
            complete=False
            break
        if not name:
            continue
        try:
            item = name.decode("utf-8")
            selected_path(root, item)
        except (ValueError, UnicodeError, OSError):
            excluded += 1
            continue
        output.add(item)
        if len(output) > max_files:
            complete = False
            break
    selected = sorted(output)[:max_files]
    return {"paths": selected, "complete": complete, "omitted": None if not complete else 0, "excluded": excluded, "revision": git(root, "rev-parse", "HEAD") or None, "membership_digest": hashlib.sha256("\0".join(selected).encode()).hexdigest(), "roots": list(roots), "basis": "selected tracked and untracked Git paths, excluding missing/deleted and protected inputs"}


def _excerpts(text: str, pattern: re.Pattern, *, limit: int = 8) -> list[dict]:
    rows = []
    section = None
    anchor = None
    fenced = False
    for number, line in enumerate(text.splitlines(), 1):
        if line.lstrip().startswith(("```", "~~~")):
            fenced = not fenced
        if not fenced:
            heading = re.match(r"^#{1,6}\s+(\d+(?:\.\d+)+)\b", line)
            if heading:
                section = heading[1]
            explicit = re.search(r'<a\s+id=["\']([^"\']+)', line)
            if explicit:
                anchor = explicit[1]
        match = pattern.search(line)
        if match:
            start = max(0, match.start() - 180)
            rows.append({"line": number, "column": start + 1, "excerpt": line[start:start+600], "section_id": section, "anchor": anchor})
            if len(rows) >= limit:
                break
    return rows


def search(query: str, *, root: Path = ROOT, roots: tuple[str, ...] = (), roles: tuple[str, ...] = (), limit: int = 10, offset: int = 0, literal: bool = False, max_files: int = 4096, max_bytes: int = 32 << 20, max_source_bytes: int = 4 << 20, seconds: float = 10) -> dict:
    if not query.strip() or len(query) > 4096 or not 1 <= limit <= 100 or offset < 0 or min(max_files, max_bytes, max_source_bytes) <= 0 or seconds <= 0:
        raise ValueError("bounded query, result and scan limits required")
    deadline = time.monotonic() + seconds
    membership = paths(root=root, roots=roots, max_files=max_files, seconds=min(3, seconds))
    terms = [query] if literal else list(dict.fromkeys(re.findall(r"[\w-]+", query.casefold())))
    if not terms:
        raise ValueError("query requires at least one lexical term")
    expression = "|".join(re.escape(term) for term in terms)
    pattern = re.compile(expression, re.IGNORECASE)
    items = []
    gaps = []
    scanned_bytes = scanned_files = 0
    for name in membership["paths"]:
        if time.monotonic() >= deadline or scanned_bytes >= max_bytes:
            gaps.append("scan_budget_exhausted")
            break
        try:
            fact = source(name, root=root, revision=membership["revision"], max_bytes=min(max_source_bytes, max_bytes-scanned_bytes))
        except (ValueError, OSError, UnicodeError):
            gaps.append("source_unavailable_changed_or_oversized:" + name)
            continue
        scanned_files += 1
        scanned_bytes += fact["bytes"]
        if roles and fact["role"] not in roles:
            continue
        excerpts = _excerpts(fact["text"], pattern)
        if not excerpts:
            continue
        joined = "\n".join(row["excerpt"] for row in excerpts).casefold()
        fact.pop("text")
        items.append({**fact, "excerpts": excerpts, "matched_terms": sum(term.casefold() in joined for term in terms)})
    items.sort(key=lambda item: ({"Current": 0, "Reference": 1, "History": 2}[item["scope"]], -item["matched_terms"], item["path"]))
    complete = membership["complete"] and not gaps
    return {"version": 1, "query": query, "matching": "literal" if literal else "lexical terms", "items": items[offset:offset+limit], "next_offset": offset+limit if len(items)>offset+limit else None, "coverage": "complete selected Git paths" if complete else "partial selected Git paths", "complete": complete, "gaps": gaps, "scanned_files": scanned_files, "scanned_bytes": scanned_bytes, "membership": {key:value for key,value in membership.items() if key!="paths"}, "meaning": "source-only lexical leads; native role/scope precedence is not scientific or semantic proof; no database required"}


def relationships(symbol: str | tuple[str, ...] = (), *, root: Path = ROOT, limit: int = 40, predicates: dict[str, str] | None = None) -> dict:
    queries = (symbol,) if isinstance(symbol, str) else symbol
    allowed = {'declaration_kind','declaration_key','source_path','source_symbol','source_role','source_identity','generator_path','generator_symbol','output_path','direction','basis','producer'}
    if predicates is not None and (not predicates or set(predicates)-allowed or any(not isinstance(value,str) or not value or len(value)>4096 for value in predicates.values())):
        raise ValueError('exact producer predicates require declared field names and bounded literal values')
    if (not queries and not predicates) or len(queries) > 128 or any(not value or len(value)>512 for value in queries) or not 1 <= limit <= 100:
        raise ValueError("bounded literal symbols/paths and limit required")
    records = []
    producers = []
    documents = []
    revision = git(root, "rev-parse", "HEAD") or None
    source_facts = {}
    gaps = []
    deadline = time.monotonic()+5
    for path in (root / "docs/generated/source-relationships").glob("**/relationships.json"):
        if len(documents)>=128 or time.monotonic()>=deadline:
            gaps.append("producer_discovery_bound")
            break
        relative = path.relative_to(root).as_posix()
        try:
            fact = source(relative, root=root, revision=revision, max_bytes=8<<20)
            document = json.loads(fact.pop("text"))
            if document.get("schema_version") != 1:
                raise ValueError("unknown producer relationship version")
            producer = document.get("producer")
            producers.append(producer)
            documents.append({**fact, "producer": producer, "registry_fingerprint": document.get("fingerprint") or document.get("registry_fingerprint"), "basis": "current sidecar bytes; registry fingerprint is producer declaration, not current source validation"})
            for row in document.get("relationships", []):
                if time.monotonic()>=deadline or len(source_facts)>=128:
                    gaps.append("producer_relationship_read_bound")
                    break
                if predicates and not all((producer if key=='producer' else row.get(key)) == value for key,value in predicates.items()):
                    continue
                if queries and not any(query.casefold() in json.dumps(row).casefold() for query in queries):
                    continue
                if len(records)<limit+1:
                    records.append({'producer':producer, **row})
                # Only producer-declared locations can establish a source link.
                for key in ("source_path", "generator_path", "input_path", "path"):
                    declared = row.get(key)
                    if isinstance(declared, str) and declared not in source_facts:
                        try:
                            linked = source(declared,root=root,revision=revision)
                            linked.pop("text")
                            source_facts[declared] = linked
                        except (ValueError, OSError, UnicodeError):
                            source_facts[declared] = {"path": declared, "availability": "unavailable_or_unadmitted"}
        except (ValueError, OSError, UnicodeError):
            gaps.append("unavailable_or_invalid_producer:"+relative)
    return {"symbol": symbol if isinstance(symbol,str) else None, "queries": list(queries), "predicates":predicates, "selection_basis":"exact field conjunction" if predicates and not queries else "literal text leads over producer records", "items": records[:limit], "truncated": len(records)>limit or bool(gaps), "producers": producers, "producer_documents": documents, "current_source_facts": list(source_facts.values()), "revision": revision, "gaps": gaps, "producer_freshness":"unverified; producer declaration identity and current source bytes are distinct", "coverage": "producer-declared paths only; dynamic consumers unresolved; no inferred declaration locations", "availability": "available" if producers else "not generated; run just codegen explicitly"}


def _toml(root: Path, relative: str) -> tuple[dict, dict]:
    fact = source(relative, root=root, max_bytes=16<<20)
    return tomllib.loads(fact.pop("text")), fact


def _normal_name(name: str) -> str:
    return re.sub(r"[-_.]+", "-", name).casefold()


def _python_declarations(project: dict) -> list[tuple[str, str]]:
    project_table = project.get("project", {})
    selected = [("project.dependencies", value) for value in project_table.get("dependencies", [])]
    selected.extend(("build-system.requires", value) for value in project.get("build-system", {}).get("requires", []))
    for group, values in project.get("dependency-groups", {}).items():
        selected.extend(("dependency-groups."+group,value) for value in values if isinstance(value,str))
    for group, values in project_table.get("optional-dependencies", {}).items():
        selected.extend(("project.optional-dependencies."+group,value) for value in values if isinstance(value,str))
    return [(location,value) for location,value in selected if isinstance(value,str)]


def _context_suitability(context: dict, target: str | None, features: tuple[str,...], manifest_digest: str, lock_digest: str) -> list[str]:
    issues = []
    if not target or not context.get("target"):
        issues.append("target_unknown")
    elif context["target"] != target:
        issues.append("target_mismatch")
    if "features" not in context:
        issues.append("features_unknown")
    elif set(context["features"]) != set(features):
        issues.append("feature_mismatch")
    if context.get("manifest_digest") != manifest_digest or context.get("lock_digest") != lock_digest:
        issues.append("source_cut_unknown_or_mismatch")
    return issues


def library_packet(package: str, *, root: Path = ROOT, references: tuple[str,...] = (), features: tuple[str,...] = (), target: str | None = None, cargo_metadata: str | None = None, rustdoc: str | None = None) -> dict:
    """Read exact pins/locks and selected evidence; never resolve, install or build.

    Optional metadata/rustdoc files may wrap the native object under
    ``cargo_metadata``/``rustdoc`` with a ``context`` carrying target, features,
    manifest_digest and lock_digest. Missing provenance stays unknown. Only
    rustdoc format 61 (the repository's retained extraction schema) is recognized.
    """
    if not re.fullmatch(r"[A-Za-z0-9_.-]{1,160}",package) or len(references)>32 or len(features)>128:
        raise ValueError("bounded literal dependency and evidence selection required")
    provenance = []
    cargo = {}
    if (root/"Cargo.toml").is_file():
        cargo, cargo_fact = _toml(root,"Cargo.toml")
        provenance.append(cargo_fact)
    declaration = cargo.get("workspace",{}).get("dependencies",{}).get(package)
    declared_name = package
    declaration_location = "workspace.dependencies."+package
    ecosystem = "cargo"
    patches = {}
    if declaration is not None:
        manifest_fact = cargo_fact
        lock, lock_fact = _toml(root,"Cargo.lock")
        if isinstance(declaration,dict):
            declared_name = declaration.get("package",package)
        for registry, values in cargo.get("patch",{}).items():
            if declared_name in values:
                patches[registry] = values[declared_name]
    else:
        ecosystem = "python"
        project, manifest_fact = _toml(root,"pyproject.toml")
        provenance.append(manifest_fact)
        declarations = []
        for location,value in _python_declarations(project):
            match = re.match(r"^\s*([A-Za-z0-9_.-]+)(?:\[[^]]+\])?\s*",value)
            if match and _normal_name(match[1]) == _normal_name(package):
                declarations.append({"location":location,"requirement":value})
        if not declarations:
            raise ValueError("dependency is not declared at this source cut")
        declaration = declarations[0]["requirement"]
        declaration_location = declarations[0]["location"]
        lock, lock_fact = _toml(root,"uv.lock")
        declared_name = _normal_name(package)
        override = project.get("tool",{}).get("uv",{}).get("sources",{}).get(package)
        if override:
            patches["tool.uv.sources"] = override
    provenance.append(lock_fact)
    resolved = [row for row in lock.get("package",[]) if _normal_name(row["name"]) == _normal_name(declared_name)]
    spec = declaration.get("version","") if isinstance(declaration,dict) else declaration
    if ecosystem == "cargo":
        pin = re.fullmatch(r"=([^=*,\s]+)",spec)
    else:
        pin = re.search(r"==\s*([^;\s*,]+)",spec)
    exact_version = pin[1] if pin and "*" not in pin[1] else None
    unresolved = ["API behavior/lifetimes and migration semantics require exact primary source or a bounded probe"]
    if lock.get("version") not in ({3,4} if ecosystem=="cargo" else {1}):
        unresolved.append("lock_format_unknown")
    if not exact_version:
        unresolved.append("exact_version_unknown_or_source_revision_pin")
    if not resolved or exact_version and any(row.get("version")!=exact_version for row in resolved):
        unresolved.append("locked_version_missing_or_ambiguous")
    local_sources = []
    local_paths = []
    if isinstance(declaration,dict) and declaration.get("path"):
        local_paths.append(declaration["path"])
    for patch in patches.values():
        if isinstance(patch,dict) and patch.get("path"):
            local_paths.append(patch["path"])
    for path in dict.fromkeys(local_paths):
        manifest = str(Path(path)/"Cargo.toml") if ecosystem=="cargo" else str(Path(path)/"pyproject.toml")
        try:
            local, local_fact = _toml(root,manifest)
            local_sources.append({**local_fact,"package_version": local.get("package",local.get("project",{})).get("version"), "basis":"selected local/patch manifest; full tree identity requires selected source references"})
        except (ValueError, OSError):
            local_sources.append({"path":manifest,"availability":"unavailable_or_unadmitted"})
            unresolved.append("local_source_unavailable")
    evidence = []
    for relative in references:
        try:
            fact = source(relative,root=root)
            text = fact.pop("text")
            primary = Path(relative).suffix in {".rs",".py",".pyi",".surql"}
            signatures = _excerpts(text,re.compile(r"\bpub\s+(?:async\s+)?(?:fn|struct|enum|trait)\b|^\s*(?:def|class)\s+"),limit=6) if primary else []
            patch_evidence = Path(relative).suffix == ".patch"
            changes = _excerpts(text,re.compile(r"^[+-](?![+-])"),limit=12) if patch_evidence else []
            evidence.append({**fact,"evidence_kind":"primary_patch" if patch_evidence else "primary_source" if primary else "overview_or_supplied_evidence", "signature_leads":signatures, "patch_change_leads":changes, "adoption_basis":"selected patch bytes; applied status not inferred" if patch_evidence else None})
        except (ValueError, OSError, UnicodeError):
            evidence.append({"path":relative,"availability":"unavailable_or_unadmitted","suitability":"unknown"})
            unresolved.append("reference_unavailable:"+relative)
    observed = {"status":"unknown", "basis":"no supplied Cargo metadata; selected features are intent only"}
    rustdoc_result = {"status":"unknown", "basis":"no supplied rustdoc JSON"}
    for kind, relative in (("cargo_metadata",cargo_metadata),("rustdoc",rustdoc)):
        if relative is None:
            continue
        try:
            fact = source(relative,root=root,max_bytes=8<<20)
            wrapper = json.loads(fact.pop("text"))
            document = wrapper.get(kind,wrapper)
            context = wrapper.get("context",{})
            issues = _context_suitability(context,target,features,manifest_fact["digest"],lock_fact["digest"])
            result = {**fact,"context":context,"issues":issues}
            if kind=="cargo_metadata":
                if document.get("version")!=1:
                    issues.append("unknown_metadata_format")
                packages = [row for row in document.get("packages",[]) if row.get("name")==declared_name and row.get("version")==exact_version]
                ids = {row["id"] for row in packages}
                nodes = [node for node in (document.get("resolve") or {}).get("nodes",[]) if node.get("id") in ids]
                if not nodes:
                    issues.append("resolved_package_or_features_missing")
                result.update({"format_version":document.get("version"),"observed_features":sorted({feature for node in nodes for feature in node.get("features",[])}),"basis":"supplied Cargo resolution observation, not compiled or executed closure"})
                result["status"] = "suitable" if not issues else "unsuitable_or_unknown"
                observed = result
            else:
                if document.get("format_version")!=61:
                    issues.append("unknown_rustdoc_format")
                if document.get("crate_version")!=exact_version:
                    issues.append("crate_version_mismatch")
                if not isinstance(document.get("index"),dict) or not isinstance(document.get("paths"),dict):
                    issues.append("unrecognized_rustdoc_shape")
                crate = document.get("index",{}).get(str(document.get("root")),{})
                if crate.get("name") != declared_name.replace("-","_"):
                    issues.append("crate_identity_unknown_or_mismatch")
                if local_sources:
                    selected_sources = {row["path"]:row["digest"] for row in evidence if row.get("evidence_kind")=="primary_source"}
                    if not selected_sources:
                        issues.append("patched_source_inputs_unknown")
                    elif any(context.get("source_digests",{}).get(path)!=digest for path,digest in selected_sources.items()):
                        issues.append("patched_source_cut_unknown_or_mismatch")
                result.update({"format_version":document.get("format_version"),"crate_version":document.get("crate_version"),"basis":"selected experimental JSON schema and supplied build context; no runtime semantics inferred"})
                result["status"] = "suitable" if not issues else "unsuitable_or_unknown"
                rustdoc_result = result
        except (ValueError,OSError,UnicodeError,TypeError,KeyError,AttributeError):
            result = {"path":relative,"status":"unknown","issues":["evidence_unavailable_or_invalid"]}
            if kind=="cargo_metadata":
                observed=result
            else:
                rustdoc_result=result
    return {"version":1,"package":package,"resolved_package":declared_name,"ecosystem":ecosystem,"declaration":declaration,"declaration_location":declaration_location,"exact_version":exact_version,"resolved":resolved,"selected_features":list(features),"declared_features":declaration.get("features",[]) if isinstance(declaration,dict) else [],"feature_basis":"caller-selected intent and manifest declarations; neither is an observed build closure","target":target,"observed_closure":observed,"rustdoc":rustdoc_result,"patches":patches,"local_sources":local_sources,"references":evidence,"source_cut":provenance,"revision":git(root,"rev-parse","HEAD") or None,"unresolved":unresolved,"effects":"no dependency resolution, fetch, build, install or service startup"}
