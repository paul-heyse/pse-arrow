# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Collect independently authored G1-G8 and PS-G1-PS-G3 decisions; never generate an approval."""

from __future__ import annotations

import hashlib
import json
import os
import sys
import time
from pathlib import Path

from scripts import implementation_phase as phase
from scripts import validation
from scripts.plan14_measure import source_digest

ROOT = Path(__file__).resolve().parents[1]


def main() -> int:
    phase.guard(ROOT, ["plan14-reviews"])
    output = Path(sys.argv[1]).resolve()
    output.mkdir(parents=True, exist_ok=True)
    # This is an input supplied by the independent M22 reviewers, not a status toggle.
    directory = Path(
        os.environ.get("PSE_PLAN14_REVIEW_INPUT", "build/plan14/independent-reviews")
    ).resolve()
    rows = []
    current = source_digest(ROOT)
    for gate in phase.manifest(ROOT)["review_gates"]:
        path = directory / f"{gate}.json"
        record = json.loads(path.read_text())
        if (
            record.get("gate") != gate
            or record.get("source_digest") != current
            or record.get("verdict") not in {"Accept", "Accept-scoped"}
        ):
            raise ValueError(f"{gate}: missing current independent acceptance")
        if (
            not record.get("reviewer")
            or record["reviewer"] in record.get("implementation_authors", [])
            or not record.get("implementation_authors")
        ):
            raise ValueError(f"{gate}: independent reviewer identity required")
        if (
            not record.get("scope")
            or not record.get("evidence")
            or record.get("open_must_findings") != 0
        ):
            raise ValueError(f"{gate}: incomplete independent review")
        evidence = []
        retained = output / "independent-reviews" / gate
        retained.mkdir(parents=True, exist_ok=False)
        review = retained / "review.json"
        review.write_bytes(path.read_bytes())
        artifacts = {
            str(review.relative_to(output)): hashlib.sha256(
                review.read_bytes()
            ).hexdigest()
        }
        for index, name in enumerate(record["evidence"]):
            source = phase.relative(ROOT, name)
            if not source.is_file():
                raise ValueError(f"missing independent evidence: {name}")
            copy = retained / f"evidence-{index}{source.suffix}"
            copy.write_bytes(source.read_bytes())
            name_in_report = str(copy.relative_to(output))
            content_digest = hashlib.sha256(copy.read_bytes()).hexdigest()
            artifacts[name_in_report] = content_digest
            evidence.append(
                {
                    "path": name_in_report,
                    "source": name,
                    "digest": content_digest,
                }
            )
        rows.append(
            {
                "id": gate,
                **record,
                "evidence": evidence,
                "review_digest": hashlib.sha256(path.read_bytes()).hexdigest(),
                "review_path": str(review.relative_to(output)),
                "artifacts": artifacts,
            }
        )
    validation.write_json(
        output / "plan14-reviews.json",
        {
            "schema": "architecture-review-v1",
            "profile": "performance-native",
            "started": time.time(),
            "source_digest": current,
            "cases": rows,
        },
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
