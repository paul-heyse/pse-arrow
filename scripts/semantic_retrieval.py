# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit finite local embedding batches over a selected native knowledge corpus.

No install, source discovery, background process or index refresh occurs on reads.
Vectors are derived leads bound to exact excerpt/source/model identities.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import re
import subprocess
import tempfile
import time
import tomllib
from pathlib import Path

from scripts import knowledge_store, workspace_sources

ROOT = Path(__file__).resolve().parents[1]


def specification() -> dict:
    return tomllib.loads((ROOT / ".config/local-retrieval.toml").read_text())


def installation(spec: dict) -> Path:
    return Path(os.environ.get("XDG_DATA_HOME", Path.home() / ".local/share")) / "pse/tools/local-retrieval" / spec["runtime"]["revision"]


def sha256(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def runtime(spec: dict) -> tuple[Path, Path, dict]:
    selected = installation(spec)
    binary = selected / spec["runtime"]["binary"]
    model = selected / spec["model"]["filename"]
    receipt_path = selected / "installation.json"
    if not binary.is_file() or not model.is_file() or not receipt_path.is_file():
        raise ValueError("optional local runtime/model are not explicitly provisioned; lexical source search remains available")
    if any(path.is_symlink() for path in (selected, binary, model, receipt_path)):
        raise ValueError("optional runtime installation must not redirect through symlinks")
    receipt = json.loads(receipt_path.read_bytes())
    if receipt.get("revision") != spec["runtime"]["revision"] or receipt.get("source_sha256") != spec["runtime"]["source_sha256"] or receipt.get("cpu_native") is not False or receipt.get("cuda_architecture") != spec["runtime"]["cuda_architecture"]:
        raise ValueError("runtime receipt differs from selected source/build contract")
    if sha256(binary) != receipt.get("binary_sha256") or model.stat().st_size != spec["model"]["bytes"] or sha256(model) != spec["model"]["sha256"]:
        raise ValueError("optional runtime or model bytes differ from the authenticated installation")
    if not receipt.get("runtime_members"):
        raise ValueError("native runtime library membership is absent")
    for relative, expected in receipt["runtime_members"].items():
        path = selected / relative
        if not path.resolve().is_relative_to(selected.resolve()) or not path.is_file() or sha256(path) != expected:
            raise ValueError("native runtime library membership changed")
    return binary, model, receipt


def vectors(texts: list[str], *, spec: dict | None = None) -> dict:
    spec = specification() if spec is None else spec
    limits = spec["limits"]
    if not texts or len(texts) > limits["max_inputs"] or any(not isinstance(text, str) or not text.strip() or len(text) > limits["max_input_characters"] for text in texts):
        raise ValueError("explicit finite nonempty input population required")
    separator = "<|pse-owned-embedding-boundary|>"
    if any(separator in text for text in texts):
        raise ValueError("input contains the selected batch separator")
    binary, model, receipt = runtime(spec)
    with tempfile.TemporaryDirectory(prefix="pse-local-retrieval-") as directory:
        prompt = Path(directory) / "inputs.txt"
        prompt.write_text(separator.join(texts))
        command = [str(binary), "--model", str(model), "--file", str(prompt), "--pooling", spec["model"]["pooling"], "--embd-normalize", str(spec["model"]["normalization"]), "--embd-output-format", "array", "--embd-separator", separator, "--n-gpu-layers", "99", "--ctx-size", str(limits["context_tokens"]), "--batch-size", str(limits["context_tokens"]), "--ubatch-size", str(limits["context_tokens"]), "--threads", "8", "--log-verbosity", "5"]
        started = time.monotonic()
        result = subprocess.run(command, cwd=directory, capture_output=True, timeout=limits["seconds"], check=False)
        elapsed = time.monotonic() - started
        if result.returncode:
            raise ValueError("local embedding batch failed: " + result.stderr.decode(errors="replace")[-4000:])
        if len(result.stdout) > 8 << 20 or len(result.stderr) > 2 << 20:
            raise ValueError("embedding result exceeds finite output allowance")
        diagnostic = result.stderr.decode(errors="replace")
        offload = re.search(r"offloaded (\d+)/(\d+) layers to GPU", diagnostic)
        device = re.search(r"using device CUDA\d+ \(NVIDIA GeForce RTX 5090\)", diagnostic)
        if not offload or int(offload[1]) == 0 or not device or "CUDA : ARCHS = 1200" not in diagnostic:
            raise ValueError("selected Blackwell GPU execution was not observed; CPU fallback does not qualify this runtime")
        embedded = json.loads(result.stdout)
        if len(embedded) != len(texts):
            raise ValueError("runtime omitted or added input embeddings")
        for vector in embedded:
            if not isinstance(vector, list) or len(vector) != spec["model"]["dimensions"] or any(type(value) not in {float, int} or not math.isfinite(value) for value in vector):
                raise ValueError("runtime returned invalid vector shape or nonfinite components")
            if not math.isclose(sum(value * value for value in vector), 1, rel_tol=1e-3):
                raise ValueError("runtime did not return the selected L2 normalization")
        return {"vectors": embedded, "elapsed_seconds": elapsed, "inputs": len(texts), "runtime": receipt, "model": spec["model"], "gpu_layers": {"offloaded": int(offload[1]), "total": int(offload[2])}, "stderr": diagnostic, "lifetime": "one finite batch process exited and was waited; no shared daemon"}


def validate_corpus(corpus: dict, *, root: Path) -> None:
    if corpus.get("version") != 1 or not corpus.get("sources") or not corpus.get("excerpts"):
        raise ValueError("a selected native knowledge corpus is required")
    observations = knowledge_store.freshness(corpus, root=root)
    if any(item["state"] != "current" for item in observations):
        raise ValueError("selected source cut is stale, missing or not locally readable; explicitly rebuild at its owner")
    sources = {item["id"]: item for item in corpus["sources"]}
    if len(sources) != len(corpus["sources"]) or len({item["id"] for item in corpus["excerpts"]}) != len(corpus["excerpts"]):
        raise ValueError("duplicate source or excerpt identity")
    for excerpt in corpus["excerpts"]:
        source = sources.get(excerpt["source_id"])
        if source is None or not excerpt["text"].strip() or len(excerpt["text"]) > specification()["limits"]["max_input_characters"]:
            raise ValueError("excerpt source or text is invalid")
        # Exact selected excerpts must occur in the named current original source.
        fact = workspace_sources.source(source["locator"][5:], root=root, expected_digest=source["version_digest"])
        if excerpt["text"] not in fact["text"]:
            raise ValueError("excerpt differs from its declared current source")


def compare(corpus: dict, questions: list[dict], *, root: Path = ROOT) -> dict:
    validate_corpus(corpus, root=root)
    if not questions or len(questions) > 32 or any(not item.get("id") or not item.get("query") for item in questions):
        raise ValueError("select 1..32 named concrete engineering questions")
    spec = specification()
    instruction = spec["model"]["query_instruction"]
    texts = [item["text"] for item in corpus["excerpts"]] + [f'Instruct: {instruction}\nQuery:{item["query"]}' for item in questions]
    observed = vectors(texts, spec=spec)
    validate_corpus(corpus, root=root)
    count = len(corpus["excerpts"])
    source_map = {item["id"]: item for item in corpus["sources"]}
    output = []
    roots = tuple(item["locator"][5:] for item in corpus["sources"])
    for ordinal, question in enumerate(questions):
        query_vector = observed["vectors"][count + ordinal]
        rows = []
        for excerpt, vector in zip(corpus["excerpts"], observed["vectors"][:count], strict=True):
            source = source_map[excerpt["source_id"]]
            rows.append({"id": excerpt["id"], "source_id": source["id"], "source_digest": source["version_digest"], "locator": excerpt["locator"], "status": source["status"], "score": sum(a*b for a, b in zip(query_vector, vector, strict=True)), "text": excerpt["text"]})
        raw = sorted(rows, key=lambda item: (-item["score"], item["id"]))
        ranked = sorted(rows, key=lambda item: ({"current": 0, "unknown": 1, "historical": 2}.get(item["status"], 1), -item["score"], item["id"]))
        lexical = workspace_sources.search(question["query"], root=root, roots=roots, limit=10, literal=question.get("literal", False), seconds=20)
        output.append({"question": question, "lexical": lexical, "semantic": ranked[:10], "raw_similarity": raw[:10], "meaning": "semantic similarity supplies evidence leads; native source authority precedes similarity; lexical searches full selected files, embedding covers only explicit corpus excerpts"})
    validate_corpus(corpus, root=root)
    model_identity = hashlib.sha256(json.dumps({"runtime": spec["runtime"], "runtime_members": observed["runtime"]["runtime_members"], "model": spec["model"]}, sort_keys=True).encode()).hexdigest()
    corpus_basis = {key: value for key, value in corpus.items() if key != "expected_active"}
    return {"version": 1, "corpus_sha256": hashlib.sha256(json.dumps(corpus_basis, sort_keys=True).encode()).hexdigest(), "corpus_digest_basis": "Python sorted JSON payload excluding CAS precondition; native generation remains separately owned", "model_identity": model_identity, "sources": corpus["sources"], "excerpt_ids": [item["id"] for item in corpus["excerpts"]], "embedding_observation": {key: value for key, value in observed.items() if key != "vectors"}, "embeddings": [{"excerpt_id": item["id"], "source_id": item["source_id"], "source_digest": source_map[item["source_id"]]["version_digest"], "vector": vector} for item, vector in zip(corpus["excerpts"], observed["vectors"][:count], strict=True)], "questions": output, "adoption": "not selected for automatic retrieval; lexical delivery; explicit experiment only"}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("describe", "compare"))
    parser.add_argument("--corpus", type=Path)
    parser.add_argument("--questions", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--root", type=Path, default=ROOT)
    args = parser.parse_args()
    if args.command == "describe":
        spec = specification()
        print(json.dumps({"specification": spec, "installation": str(installation(spec)), "effects": "read-only description; no provisioning, model load or index refresh"}))
        return 0
    if not args.corpus or not args.questions or not args.output:
        parser.error("compare requires selected --corpus --questions and --output")
    result = compare(json.loads(args.corpus.read_bytes()), json.loads(args.questions.read_bytes()), root=args.root)
    args.output.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({"output": str(args.output), "questions": len(result["questions"]), "adoption": result["adoption"]}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
