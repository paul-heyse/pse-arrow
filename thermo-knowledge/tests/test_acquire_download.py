# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Kinds `archive` and `file`: verified downloads against a loopback HTTP server."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from test_acquire_support import (
    HttpFixture,
    Route,
    independent_tree_hash,
    make_context,
    make_tar,
    make_zip,
    md5,
    run_acquire,
    sha256,
    tree_files,
    write_manifest,
)
from thermo_knowledge.acquire.manifest import ManifestError, parse_manifest
from thermo_knowledge.acquire.store import ACQUISITION_NAME

TAR = make_tar({"top/a.txt": b"alpha\n", "top/sub/b.txt": b"beta\r\n"})
ZIP = make_zip({"z/one.txt": b"one\n", "z/two.bin": bytes(range(256))})
PLAIN = b"name,value\nx,1\n"


@pytest.fixture
def server(monkeypatch: pytest.MonkeyPatch):
    monkeypatch.setenv("NO_PROXY", "127.0.0.1")
    monkeypatch.delenv("HTTP_PROXY", raising=False)
    monkeypatch.delenv("http_proxy", raising=False)
    with HttpFixture() as fixture:
        fixture.routes["/data.tar.gz"] = Route(TAR)
        fixture.routes["/data.zip"] = Route(ZIP)
        fixture.routes["/plain.csv"] = Route(PLAIN)
        fixture.routes["/moved.csv"] = Route(b"", 302, {"Location": "/plain.csv"})
        yield fixture


def setup(tmp_path: Path, server: HttpFixture, source_id: str, body: str):
    sources = tmp_path / "sources"
    write_manifest(sources, source_id, body.replace("{base}", server.url("")))
    return make_context(tmp_path), sources, tmp_path / "sources.lock"


def nothing_stored(ctx, source_id: str, lock: Path) -> bool:
    return not (ctx.raw_dir / source_id).exists() and not lock.exists()


# -- archive ------------------------------------------------------------------------------


def test_matching_sha256_is_stored_with_its_extracted_tree(
    tmp_path: Path, server: HttpFixture
) -> None:
    body = (
        f'kind = "archive"\nurl = "{{base}}/data.tar.gz"\nsha256 = "{sha256(TAR)}"\nextract = true'
    )
    ctx, sources, lock = setup(tmp_path, server, "tarsrc", body)
    [result] = run_acquire(ctx, sources, lock)
    assert result.ok, result.message
    pin_dir = ctx.raw_dir / "tarsrc" / sha256(TAR)[:12]
    tree = pin_dir / "tree"
    assert (tree / "data.tar.gz").read_bytes() == TAR
    assert (tree / "extracted" / "top" / "a.txt").read_bytes() == b"alpha\n"
    assert (tree / "extracted" / "top" / "sub" / "b.txt").read_bytes() == b"beta\r\n"
    record = json.loads((pin_dir / ACQUISITION_NAME).read_text())
    assert record["resolved"] == sha256(TAR)
    assert record["tool_versions"]["httpx"]
    assert sorted(f["path"] for f in record["files"]) == [
        "data.tar.gz",
        "extracted/top/a.txt",
        "extracted/top/sub/b.txt",
    ]
    entry = json.loads(lock.read_text())["sources"]["tarsrc"]
    assert entry["kind"] == "archive"
    assert entry["pin"] == sha256(TAR)[:12]
    assert entry["tree_hash"] == independent_tree_hash(tree)
    assert entry["file_count"] == 3


def test_archive_without_extract_stores_only_the_archive(
    tmp_path: Path, server: HttpFixture
) -> None:
    body = f'kind = "archive"\nurl = "{{base}}/data.tar.gz"\nsha256 = "{sha256(TAR)}"'
    ctx, sources, lock = setup(tmp_path, server, "tarsrc", body)
    run_acquire(ctx, sources, lock)
    tree = ctx.raw_dir / "tarsrc" / sha256(TAR)[:12] / "tree"
    assert set(tree_files(tree)) == {"data.tar.gz"}


def test_matching_md5_and_a_zip_are_stored(tmp_path: Path, server: HttpFixture) -> None:
    body = f'kind = "archive"\nurl = "{{base}}/data.zip"\nmd5 = "{md5(ZIP)}"\nextract = true'
    ctx, sources, lock = setup(tmp_path, server, "zipsrc", body)
    [result] = run_acquire(ctx, sources, lock)
    assert result.ok, result.message
    tree = ctx.raw_dir / "zipsrc" / md5(ZIP)[:12] / "tree"
    assert (tree / "extracted" / "z" / "two.bin").read_bytes() == bytes(range(256))
    assert (tree / "data.zip").read_bytes() == ZIP


def test_redirects_are_followed(tmp_path: Path, server: HttpFixture) -> None:
    body = f'kind = "file"\nfiles = [{{ url = "{{base}}/moved.csv", name = "x.csv", sha256 = "{sha256(PLAIN)}" }}]'
    ctx, sources, lock = setup(tmp_path, server, "filesrc", body)
    [result] = run_acquire(ctx, sources, lock)
    assert result.ok, result.message
    assert server.requested() == ["/moved.csv", "/plain.csv"]


@pytest.mark.parametrize("algorithm", ["sha256", "md5"])
def test_a_mismatching_checksum_is_refused_and_nothing_is_stored(
    tmp_path: Path, server: HttpFixture, algorithm: str
) -> None:
    wrong = "0" * (64 if algorithm == "sha256" else 32)
    body = (
        f'kind = "archive"\nurl = "{{base}}/data.tar.gz"\n{algorithm} = "{wrong}"\nextract = true'
    )
    ctx, sources, lock = setup(tmp_path, server, "tarsrc", body)
    [result] = run_acquire(ctx, sources, lock)
    assert not result.ok
    assert result.status == "refused"
    actual = sha256(TAR) if algorithm == "sha256" else md5(TAR)
    assert wrong in result.message
    assert actual in result.message
    assert "/data.tar.gz" in result.message
    assert nothing_stored(ctx, "tarsrc", lock)


@pytest.mark.parametrize(
    "members",
    [{"../evil.txt": b"x"}, {"ok.txt": b"y", "a/../../evil.txt": b"x"}, {"/abs/evil.txt": b"x"}],
)
def test_a_tar_member_that_escapes_is_refused(
    tmp_path: Path, server: HttpFixture, members: dict[str, bytes]
) -> None:
    data = make_tar(members)
    server.routes["/bad.tar.gz"] = Route(data)
    body = (
        f'kind = "archive"\nurl = "{{base}}/bad.tar.gz"\nsha256 = "{sha256(data)}"\nextract = true'
    )
    ctx, sources, lock = setup(tmp_path, server, "badtar", body)
    [result] = run_acquire(ctx, sources, lock)
    assert not result.ok
    assert "escape" in result.message
    assert "evil.txt" in result.message
    assert nothing_stored(ctx, "badtar", lock)
    assert not (tmp_path / "evil.txt").exists()
    assert not (tmp_path / "raw" / "evil.txt").exists()


def test_a_zip_member_that_escapes_is_refused(tmp_path: Path, server: HttpFixture) -> None:
    data = make_zip({"fine.txt": b"y", "../evil.txt": b"x"})
    server.routes["/bad.zip"] = Route(data)
    body = f'kind = "archive"\nurl = "{{base}}/bad.zip"\nsha256 = "{sha256(data)}"\nextract = true'
    ctx, sources, lock = setup(tmp_path, server, "badzip", body)
    [result] = run_acquire(ctx, sources, lock)
    assert not result.ok
    assert "escape" in result.message
    assert nothing_stored(ctx, "badzip", lock)
    assert not (tmp_path / "evil.txt").exists()


def test_extract_of_something_that_is_not_an_archive_is_refused(
    tmp_path: Path, server: HttpFixture
) -> None:
    body = (
        f'kind = "archive"\nurl = "{{base}}/plain.csv"\nsha256 = "{sha256(PLAIN)}"\nextract = true'
    )
    ctx, sources, lock = setup(tmp_path, server, "notarchive", body)
    [result] = run_acquire(ctx, sources, lock)
    assert not result.ok
    assert "neither a tar nor a zip" in result.message
    assert nothing_stored(ctx, "notarchive", lock)


def test_a_failing_status_is_refused_with_the_url(tmp_path: Path, server: HttpFixture) -> None:
    body = f'kind = "archive"\nurl = "{{base}}/missing.tgz"\nsha256 = "{"a" * 64}"'
    ctx, sources, lock = setup(tmp_path, server, "gone", body)
    [result] = run_acquire(ctx, sources, lock)
    assert not result.ok
    assert "/missing.tgz" in result.message
    assert "404" in result.message
    assert nothing_stored(ctx, "gone", lock)


def test_an_unreachable_server_is_reported(tmp_path: Path, server: HttpFixture) -> None:
    body = f'kind = "archive"\nurl = "http://127.0.0.1:1/x.tgz"\nsha256 = "{"a" * 64}"'
    ctx, sources, lock = setup(tmp_path, server, "down", body)
    [result] = run_acquire(ctx, sources, lock)
    assert not result.ok
    assert "download failed" in result.message
    assert nothing_stored(ctx, "down", lock)


def test_rerunning_fetches_nothing(tmp_path: Path, server: HttpFixture) -> None:
    body = f'kind = "archive"\nurl = "{{base}}/data.tar.gz"\nsha256 = "{sha256(TAR)}"'
    ctx, sources, lock = setup(tmp_path, server, "tarsrc", body)
    run_acquire(ctx, sources, lock)
    requests = len(server.log)
    before = lock.read_bytes()
    [result] = run_acquire(ctx, sources, lock)
    assert result.status == "unchanged"
    assert len(server.log) == requests
    assert lock.read_bytes() == before


def test_an_existing_pin_directory_is_never_written_into(
    tmp_path: Path, server: HttpFixture
) -> None:
    body = f'kind = "archive"\nurl = "{{base}}/data.tar.gz"\nsha256 = "{sha256(TAR)}"'
    ctx, sources, lock = setup(tmp_path, server, "tarsrc", body)
    occupied = ctx.raw_dir / "tarsrc" / sha256(TAR)[:12]
    occupied.mkdir(parents=True)
    (occupied / "junk.txt").write_text("not an acquisition")
    [result] = run_acquire(ctx, sources, lock)
    assert not result.ok
    assert "ACQUISITION.json" in result.message
    assert server.log == []
    assert [p.name for p in occupied.iterdir()] == ["junk.txt"]
    assert not lock.exists()


# -- file ---------------------------------------------------------------------------------


def file_body(*items: str) -> str:
    return 'kind = "file"\nfiles = [\n' + ",\n".join(f"  {{ {item} }}" for item in items) + "\n]"


def test_files_are_stored_as_delivered_and_the_lock_records_them(
    tmp_path: Path, server: HttpFixture
) -> None:
    body = file_body(
        f'url = "{{base}}/plain.csv", name = "a.csv", sha256 = "{sha256(PLAIN)}"',
        f'url = "{{base}}/plain.csv", name = "b.csv", md5 = "{md5(PLAIN)}"',
        'url = "{base}/data.zip", name = "c.zip"',
    )
    ctx, sources, lock = setup(tmp_path, server, "filesrc", body)
    [result] = run_acquire(ctx, sources, lock)
    assert result.ok, result.message
    entry = json.loads(lock.read_text())["sources"]["filesrc"]
    pin_dir = ctx.raw_dir / "filesrc" / entry["pin"]
    assert tree_files(pin_dir / "tree") == {"a.csv": PLAIN, "b.csv": PLAIN, "c.zip": ZIP}
    assert entry["tree_hash"] == independent_tree_hash(pin_dir / "tree")
    record = json.loads((pin_dir / ACQUISITION_NAME).read_text())
    by_path = {f["path"]: f["sha256"] for f in record["files"]}
    assert by_path == {"a.csv": sha256(PLAIN), "b.csv": sha256(PLAIN), "c.zip": sha256(ZIP)}


def test_a_single_file_pin_is_the_prefix_of_its_checksum(
    tmp_path: Path, server: HttpFixture
) -> None:
    ctx, sources, lock = setup(
        tmp_path,
        server,
        "one",
        file_body(f'url = "{{base}}/plain.csv", name = "a.csv", sha256 = "{sha256(PLAIN)}"'),
    )
    run_acquire(ctx, sources, lock)
    assert (ctx.raw_dir / "one" / sha256(PLAIN)[:12] / "tree" / "a.csv").read_bytes() == PLAIN
    md5_ctx, md5_sources, md5_lock = setup(
        tmp_path / "m",
        server,
        "two",
        file_body(f'url = "{{base}}/plain.csv", name = "a.csv", md5 = "{md5(PLAIN)}"'),
    )
    run_acquire(md5_ctx, md5_sources, md5_lock)
    assert (md5_ctx.raw_dir / "two" / md5(PLAIN)[:12] / "tree" / "a.csv").exists()


def test_a_file_without_a_checksum_is_stored_and_recorded_by_its_own_sha256(
    tmp_path: Path, server: HttpFixture
) -> None:
    ctx, sources, lock = setup(
        tmp_path, server, "bare", file_body('url = "{base}/plain.csv", name = "a.csv"')
    )
    [result] = run_acquire(ctx, sources, lock)
    assert result.ok, result.message
    entry = json.loads(lock.read_text())["sources"]["bare"]
    assert entry["pin"] == sha256(PLAIN)[:12]
    assert entry["resolved"] == sha256(PLAIN)
    requests = len(server.log)
    [again] = run_acquire(ctx, sources, lock)
    assert again.status == "unchanged"
    assert len(server.log) == requests


def test_a_matching_md5_is_stored(tmp_path: Path, server: HttpFixture) -> None:
    ctx, sources, lock = setup(
        tmp_path,
        server,
        "m",
        file_body(f'url = "{{base}}/plain.csv", name = "a.csv", md5 = "{md5(PLAIN)}"'),
    )
    [result] = run_acquire(ctx, sources, lock)
    assert result.ok, result.message
    assert tree_files(ctx.raw_dir / "m" / md5(PLAIN)[:12] / "tree") == {"a.csv": PLAIN}


def test_a_mismatching_md5_is_refused_and_nothing_is_stored(
    tmp_path: Path, server: HttpFixture
) -> None:
    ctx, sources, lock = setup(
        tmp_path,
        server,
        "m",
        file_body(
            f'url = "{{base}}/plain.csv", name = "good.csv", sha256 = "{sha256(PLAIN)}"',
            f'url = "{{base}}/data.zip", name = "bad.zip", md5 = "{"0" * 32}"',
        ),
    )
    [result] = run_acquire(ctx, sources, lock)
    assert not result.ok
    assert "0" * 32 in result.message
    assert md5(ZIP) in result.message
    assert "/data.zip" in result.message
    assert nothing_stored(ctx, "m", lock)


def test_a_mismatching_sha256_of_a_file_is_refused(tmp_path: Path, server: HttpFixture) -> None:
    ctx, sources, lock = setup(
        tmp_path,
        server,
        "s",
        file_body(f'url = "{{base}}/plain.csv", name = "a.csv", sha256 = "{"1" * 64}"'),
    )
    [result] = run_acquire(ctx, sources, lock)
    assert not result.ok
    assert nothing_stored(ctx, "s", lock)


def test_both_checksums_on_one_file_are_refused_by_the_manifest() -> None:
    from test_acquire_support import manifest_text

    text = manifest_text(
        "both",
        file_body(f'url = "u", name = "a", sha256 = "{"a" * 64}", md5 = "{"b" * 32}"'),
    )
    with pytest.raises(ManifestError, match="at most one"):
        parse_manifest(text, file="both.toml", stem="both")
