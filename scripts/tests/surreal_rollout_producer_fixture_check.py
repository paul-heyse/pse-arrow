# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Exercise actual beta offline publishing without starting a database service."""

from __future__ import annotations

import argparse
import json
import tempfile
from pathlib import Path
from unittest.mock import patch

from scripts import surreal_rollout_producer as producer


def check(binary: Path, scratch: Path) -> None:
    folder = scratch / "chain"
    first_schema = b"DEFINE TABLE producer_fixture SCHEMAFULL;\nDEFINE FIELD old ON producer_fixture TYPE option<string>;\n"
    second_schema = (
        first_schema + b"DEFINE FIELD count ON producer_fixture TYPE option<int>;\n"
    )
    final_schema = second_schema.replace(
        b"DEFINE FIELD old ON producer_fixture TYPE option<string>;\n", b""
    )
    first = producer.publish(binary, folder, first_schema, "fixture.v1", "1" * 64)
    frozen = {
        name: digest
        for name, digest in producer.inventory(folder).items()
        if name.startswith("rollouts/")
    }
    second = producer.publish(binary, folder, second_schema, "fixture.v1", "2" * 64)
    third = producer.publish(binary, folder, final_schema, "fixture.v1", "3" * 64)
    codec = producer.publish(binary, folder, final_schema, "fixture.v2", "4" * 64)
    actual = producer.inventory(folder)
    assert all(actual[name] == digest for name, digest in frozen.items())
    assert second['predecessor'] == first['id']
    assert third['predecessor'] == second['id']
    assert codec['predecessor'] == third['id']
    assert codec['steps'] == []
    assert [
        step["phase"] for step in third["steps"] if step["phase"] != "rollback"
    ] == ["start", "complete"]
    assert len(producer.load(folder)["rollouts"]) == 4
    assert (
        producer.publish(binary, folder, final_schema, "fixture.v2", "4" * 64)["id"]
        == codec["id"]
    )
    assert actual == producer.inventory(folder)
    with patch.object(
        producer, "declaration", return_value=(final_schema, "fixture.v2", "4" * 64)
    ):
        producer.load(folder, current=True)
    with patch.object(
        producer, "declaration", return_value=(second_schema, "fixture.v2", "4" * 64)
    ):
        try:
            producer.load(folder, current=True)
        except producer.ProducerError:
            pass
        else:
            raise AssertionError("Changed declaration was admitted")
    target = folder / first["target_sql"]
    original = target.read_bytes()
    for replacement in (b"-- falsified\n", None):
        if replacement is None:
            target.unlink()
        else:
            target.write_bytes(replacement)
        before = producer.inventory(folder)
        try:
            producer.publish(binary, folder, final_schema, "fixture.v3", "5" * 64)
        except (producer.ProducerError, KeyError):
            pass
        else:
            raise AssertionError(
                "Falsified/missing frozen history was repaired or accepted"
            )
        assert before == producer.inventory(folder)
        target.write_bytes(original)
    extra = folder / "unowned.surql"
    extra.write_text("DELETE producer_fixture;")
    try:
        producer.load(folder)
    except producer.ProducerError:
        pass
    else:
        raise AssertionError("Unregistered input was admitted")
    extra.unlink()
    producer.load(folder)
    # Keep concrete CLI outputs and producer bindings for recovery integration.
    (scratch / "result.json").write_text(
        json.dumps(
            {"rollouts": [first, second, third, codec], "result": "passed"}, indent=2
        )
        + "\n"
    )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--scratch", type=Path)
    args = parser.parse_args()
    if args.scratch:
        args.scratch.mkdir(parents=True, exist_ok=False)
        check(args.binary, args.scratch)
    else:
        with tempfile.TemporaryDirectory(
            prefix="canonical-producer-check-"
        ) as directory:
            check(args.binary, Path(directory))
    print("immutable beta.6 producer fixture: passed")


if __name__ == "__main__":
    main()
