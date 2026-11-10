# Recoverable edited supplier trees

These deltas record the edited trees described by [the native notes](../native-supplier-notes.md). They are source-reconstruction records, not an installation
path, a second production implementation or a producer-eligibility grant. The server keeps its
existing ordered patch-series form in [the server source record](../../scripts/surrealdb-source/source-build.md).

## Pinned published archives

Download each archive from `https://static.crates.io/crates/{package}/{package}-{version}.crate`.
The checksum identifies the actual published source, including normalized manifests, lockfiles,
licenses and packaged VCS metadata. The VCS revision is metadata supplied by that archive; in
particular the SUNDIALS wrapper declares `dirty: true`, so its Git commit alone cannot reconstruct
the archive. The published wrapper bundles SUNDIALS 7.1.1. The maintained wrapper selects the
authenticated SUNDIALS 7.9.0 projection recorded in
[`native-source.json`](../sundials-sys/native-source.json); native and Rust wrapper versions
remain distinct. The wrapper archive below remains the reconstruction baseline; the complete
delta includes its native release replacement and maintained integration.

| Directory | Package | Version | Archive SHA256 | Packaged VCS revision | Packaged VCS path | Origin |
|---|---|---|---|---|---|---|
| `coolprop-sys` | `coolprop-sys` | `8.0.0` | `7d388a1023cf8236d1a9f372db5f3ade4dc752b7588eef68848bba2050b2eb08` | `30801b93f8a27dfa7f93537151bc5d39630d3f24` | `coolprop-sys` | https://github.com/portyanikhin/rfluids |
| `suitesparse-sys` | `suitesparse_sys` | `0.1.4` | `9e4ced97d4de36a6c50e381ae4f9d91d89d056576ceedd2252e4c688dd96b15f` | `590e034a9d708bab072585399e5018d170c87455` | `.` | https://github.com/martinjrobins/suitesparse-sys |
| `sundials-sys` | `sundials-sys` | `0.6.2` | `6fbd1d53481b83fc758406097f07b668bf243945e2b1f7db93a8f997cf6b44c9` | `86aaec38c3cdb4b0a919e6b006a5bc24e72e7e35` | `.` | https://github.com/Chris00/sundials-sys |
| `diffsol` | `diffsol` | `0.16.2` | `4b324c954a5178683516e35e06051f02e3e2471d22b8175d9a01b274bd81269d` | `dc30fee49e692c65577b11a1181bfca62f5ebf42` | `crates/diffsol` | https://github.com/martinjrobins/diffsol |
| `diffsol-la` | `diffsol-la` | `0.1.1` | `0fd836b0b52b6459b1e16c4ebfe267599b39c07a2eb638426a4e72a279a4e384` | `dc30fee49e692c65577b11a1181bfca62f5ebf42` | `crates/diffsol-la` | https://github.com/martinjrobins/diffsol |
| `diffsol-nl` | `diffsol-nl` | `0.1.1` | `6b24ab3f56ff94c563ab85e7897ba57f884617d0dc83a081028d28ec800213a1` | `dc30fee49e692c65577b11a1181bfca62f5ebf42` | `crates/diffsol-nl` | https://github.com/martinjrobins/diffsol |
| `faer` | `faer` | `0.24.4` | `5ab6df3dd147fe8d702a288b95bcd8fcc499ab572fc80da6828f60cd4d524d67` | `0539947ffb757a739d7e703a7d2fa0c792a909c1` | `faer` | https://github.com/sarah-quinones/faer-rs |
| `faer-traits` | `faer-traits` | `0.24.0` | `b87d23ed7ab1f26c0cba0e5b9e061a796fbb7dc170fa8bee6970055a1308bb0f` | `8dfcceee8eb3d81231366acc9d3f157ac5e7057a` | `faer-traits` | https://github.com/sarah-quinones/faer-rs |
| `symbolica` | `symbolica` | `3.0.1` | `0965398bbe5063c00b3a0d078d7d3dc57b61819b97e960b73e9a7e9602662795` | `e9a0d35490a834afb2fa40d97c8c0b1e58ef5a5e` | `.` | https://github.com/symbolica-dev/symbolica |
| `numerica` | `numerica` | `3.0.1` | `836accdb415b7667573bd12676e0b2eb4440e3f60b110038d96a9086f5651c15` | `a8a8fcb8941752e265e3c4fa507c02a3f06bb70e` | `.` | https://github.com/symbolica-dev/numerica |

## Complete byte deltas

Each `<directory>.patch` is a single full `git diff --binary --full-index --no-renames`
from the entire extracted archive to the repository's tracked supplier files. It retains
whitespace changes, added files, deleted files and executable modes. It does not select only
Rust/C sources or substantive hunks. All tracked supplier files, including SuiteSparse's
`.gitattributes`, are included.
Cargo-created untracked caches and build outputs are outside the edited tracked trees.

Counts below compare archive files with tracked supplier files.
Whitespace-only means no diff under `git diff --ignore-all-space --ignore-blank-lines`; all
remaining changes are conservatively counted as substantive, including line reflow. These
categories describe the diff and do not assert semantic equivalence.

| Directory | Modified substantive | Modified whitespace-only | Added | Deleted | Delta |
|---|---:|---:|---:|---:|---|
| `coolprop-sys` | 5 | 0 | 1 | 0 | [coolprop-sys.patch](coolprop-sys.patch) |
| `suitesparse-sys` | 1 | 0 | 1 | 0 | [suitesparse-sys.patch](suitesparse-sys.patch) |
| `sundials-sys` | 657 | 0 | 138 | 103 | [sundials-sys.patch](sundials-sys.patch) |
| `diffsol` | 2 | 0 | 0 | 0 | [diffsol.patch](diffsol.patch) |
| `diffsol-la` | 1 | 0 | 0 | 0 | [diffsol-la.patch](diffsol-la.patch) |
| `diffsol-nl` | 1 | 0 | 0 | 0 | [diffsol-nl.patch](diffsol-nl.patch) |
| `faer` | 3 | 0 | 0 | 0 | [faer.patch](faer.patch) |
| `faer-traits` | 1 | 0 | 0 | 0 | [faer-traits.patch](faer-traits.patch) |
| `symbolica` | 13 | 0 | 7 | 0 | [symbolica.patch](symbolica.patch) |
| `numerica` | 1 | 0 | 1 | 0 | [numerica.patch](numerica.patch) |

SuiteSparse adds `.gitattributes` to retain its two published CRLF files. The SUNDIALS release
replacement adds 138 and deletes 103 paths relative to the published wrapper, including the
native provenance record. Its projection retains complete released `cmake/`, `external/`,
`include/` and `src/` directories and matching root support files; obsolete wrapper-bundled CI,
SWIG and other removed release paths are deleted. Other listed trees have no deleted archive
files. The comparison below checks the complete path set, including deletions. The patch itself
is the complete hunk inventory.

## Reconstruct and compare

From the repository root, the following one-off command downloads and checksum-verifies each
archive, applies its complete delta in private scratch and compares every path, byte and executable mode with the tracked tree. It uses this page's archive
table directly. It does not build a supplier or consume an installed binary. Retain or remove
the printed scratch directory as desired.

```sh
scripts/pse-env -- .venv/bin/python - <<'PY'
from pathlib import Path
import hashlib
import json
import stat
import subprocess
import tarfile
import tempfile
import urllib.request

repo = Path.cwd()
scratch = Path(tempfile.mkdtemp(prefix="pse-supplier-reconstruction-"))
print(f"scratch: {scratch}")
for line in (repo / "vendor/deltas/readme.md").read_text().splitlines():
    cells = [cell.strip().strip("`") for cell in line.strip("|").split("|")]
    if len(cells) != 7 or len(cells[3]) != 64:
        continue
    directory, package, version, expected, revision, vcs_path, origin = cells
    archive = scratch / f"{package}-{version}.crate"
    url = f"https://static.crates.io/crates/{package}/{archive.name}"
    with urllib.request.urlopen(url, timeout=60) as response:
        archive.write_bytes(response.read())
    assert hashlib.sha256(archive.read_bytes()).hexdigest() == expected, package
    with tarfile.open(archive) as bundle:
        bundle.extractall(scratch, filter="data")
    tree = scratch / f"{package}-{version}"
    vcs = json.loads((tree / ".cargo_vcs_info.json").read_text())
    assert vcs["git"]["sha1"] == revision, package
    assert (vcs["path_in_vcs"] or ".") == vcs_path, package
    subprocess.run(["git", "-C", str(tree), "apply", "--whitespace=nowarn",
                    str(repo / f"vendor/deltas/{directory}.patch")], check=True)
    prefix = f"vendor/{directory}/"
    entries = subprocess.check_output(
        ["git", "ls-files", "--stage", "-z", "--", prefix], cwd=repo
    ).decode().split("\0")
    wanted = {}
    for entry in filter(None, entries):
        metadata, path = entry.split("\t", 1)
        relative = path[len(prefix):]
        source = repo / path
        mode = metadata.split()[0]
        wanted[relative] = (source.read_bytes(), mode)
    actual = {
        str(path.relative_to(tree)): (
            path.read_bytes(), "100755" if path.stat().st_mode & stat.S_IXUSR else "100644"
        )
        for path in tree.rglob("*") if path.is_file()
    }
    assert actual == wanted, directory
    print(f"{directory}: exact paths, bytes and executable modes ({len(actual)} files)")
PY
```

To refresh a delta after an intentional supplier edit, extract its pinned archive into an
isolated Git work directory and commit that unmodified archive there. Replace its file tree
with only `git ls-files --stage -- vendor/<directory>/` from the repository, retaining modes;
remove files absent from that tracked set. Stage that isolated
tree and write `git diff --cached --binary --full-index --no-renames HEAD` to the existing delta.
Run the full comparison above. Never regenerate from a working directory walk that could
capture `target/`, caches or other untracked build artifacts. This maintenance step affects
only the provenance delta; any intentional implementation change requires its own functional
qualification.

The checked binary64 evaluator transfer refresh follows this complete tracked-tree
procedure for Symbolica only. Its checksum-verified 3.0.1 archive plus delta was
independently reapplied and compared across all 159 files, bytes and executable modes.
The [supplier contract and focused controls](../symbolica-f64-transfer.md) retain its
separate execution-profile and qualification limits.

For the build-authenticated native suppliers, refresh the complete delta in the private
Git work directory described above after an intentional source edit. Independently reapply
that delta to a second extraction of the checksum-verified archive and compare every tracked
path, byte and executable mode. Derive the existing `<directory>.source.json` receipt only
after that comparison passes: retain the pinned package, version and archive checksum;
set `delta_sha256` from the complete delta bytes; and derive each `files` entry's `sha256`
and `100644`/`100755` mode from the compared tracked tree. Publish the delta and derived
receipt together. The complete tracked file population includes intentional new paths,
so stage those paths before collecting `git ls-files --stage`; build caches remain excluded.
When `native-source.json` selects a replacement native release, the same owner additionally
downloads/checks that release, verifies its complete projection and allows changes only in the
explicit maintained integration files before publishing the checked receipt.
The receipt is derived evidence; this page remains the published baseline owner and the delta
the complete maintained change. The native build helper checks the selected source tree and
delta against that receipt and embeds its digest. Same-version source edits with stale receipts
refuse at build time; loaded artifacts do not scan source during service. Cargo watches the
selected source directory, delta and receipt. External SUNDIALS selection instead requires its
selected provider receipt; an unused bundled delta cannot stand in for the external provider.

Tested 2026-10-26: the reconstruction command above matched all seven trees exactly, including
all additions, whitespace and file modes, against the pinned archive digests. The failure
baseline was zero; there were zero reconstruction failures. This does not establish numerical,
transport, producer-context or assembled product qualification.
