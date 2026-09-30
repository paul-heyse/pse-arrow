# Source manifests, acquisition and the raw store (v0)

This page specifies how a source is declared, acquired and verified. It is the contract for the
`tk acquire` stage. It records intended design; where code exists and disagrees, the code is what
runs and this page is wrong.

## 1. Principles

- **One declaration per source**: `sources/<id>.toml`. Nothing else states a source's URL, pin,
  terms or payload.
- **The declared pin is not the pin.** Acquisition records what actually resolved in
  `sources.lock`, and every later stage refuses a raw store that disagrees with the lock.
- **Untouched.** The raw store holds bytes as delivered. No reformatting, no line-ending
  conversion, no pruning beyond a declared sparse path list. A completed acquisition is read-only.
- **`tk acquire` is the only networked stage.** Every other stage runs offline from the store.
- **Rights are recorded, not used as a filter.** A source is skipped only when its manifest says
  `kind = "none"` with a stated reason.
- **Third-party bytes never enter git.** The store is gitignored; the lock holds hashes only.

**Why acquisition is written here and not taken from a library.** Hashed downloads of single
files and archives are what libraries such as pooch provide, and data-versioning tools import
git repositories. Neither covers the five kinds together: a git commit without history with
sparse paths and submodules, rate-limited and resumable page retrieval under `robots.txt`, a
verified local checkout, and a source recorded as not acquired. Each records rights per source
and resolves to one lock. Using a library for two kinds would give part of the sources a second
store layout and a second place where pins live. The network and archive mechanics themselves
come from `git`, `httpx`, `tarfile`, `zipfile` and `urllib.robotparser`. Revisit if a library
gains polite page retrieval with a lock, or if the kinds shrink to hashed files only.

## 2. Manifest

```toml
id = "coolprop"                       # lowercase snake_case; also the directory and schema suffix
title = "CoolProp"
tier = "A"                            # A | B | held (already pinned by the repository)
homepage = "https://github.com/CoolProp/CoolProp"
waves = [2, 7]                        # ingestion waves that read this source
notes = "Fluid files are one JSON per fluid."

[acquire]
kind = "git"
url = "https://github.com/CoolProp/CoolProp.git"
commit = "<full 40-hex sha>"          # authoritative
tag = "v8.0.0"                        # descriptive; acquisition fails if the tag no longer names the commit
submodules = []                       # paths of submodules to fetch at their recorded commits
sparse = []                           # if non-empty, only these paths (plus licence files) are kept

[payload]
reader = "coolprop"                   # reader module name
environment = "core"                  # environment the reader runs in
include = ["dev/fluids/*.json", "dev/mixtures/**", "dev/cubics/*.json", "dev/pcsaft/*.json"]
exclude = []

[[rights]]
scope = "code"                        # code | data | a path glob within the source
basis = "licence_grant"               # licence_grant | public_domain | permission | terms_of_use | not_stated
spdx = "MIT"                          # omitted when no SPDX identifier applies
statement = "LICENSE file, MIT."      # quoted or closely paraphrased governing statement
url = "https://github.com/CoolProp/CoolProp/blob/v8.0.0/LICENSE"
store = "yes"                         # yes | no | not_stated
redistribute = "yes"                  # yes | no | not_stated | with_conditions
commercial = "yes"
attribution = true
share_alike = false
observed = "2026-09-30"               # when and how this was established
```

A source may have several `[[rights]]` entries: different scopes, or competing statements about
one scope. Competing statements are all recorded; none is chosen.

The `id` equals the file stem. `[payload]`, `homepage`, `spdx` and a right's `url` are optional;
`payload.environment` defaults to `core`. `observed` is a quoted string. Checksums are compared
in lower case.

### Acquisition kinds

| `kind` | Required keys | What is stored |
|---|---|---|
| `git` | `url`, `commit`; optional `tag`, `submodules`, `sparse` | a checkout of exactly that commit, without history |
| `archive` | `url`; one of `sha256`, `md5` (the publisher's checksum); optional `doi`, `extract` | the archive as delivered and, when `extract = true`, its extracted tree |
| `file` | `files = [{ url, name, sha256?, md5? }]` (at most one publisher's checksum per file) | each file as delivered |
| `pages` | `enumerator`, `min_interval_seconds`, optional `limit` | each response body as delivered, plus a sidecar with URL, status, headers and retrieval time |
| `local` | `path` (relative to the repository root), `commit` or `tag` | nothing is copied; the lock records the verified revision of the existing checkout |
| `none` | `reason` | nothing; the source is recorded as not acquired |

`pages` is for sources with no bulk download. Its `enumerator` names a small module that yields
the URLs to fetch (for example every table linked from an index page, or the pages for a scoped
list of species). The fetcher:

- reads `robots.txt` once per host and does not fetch a disallowed path;
- waits at least `min_interval_seconds` between requests to one host, and at least any
  `Crawl-delay` the host declares;
- sends a descriptive `User-Agent` naming the project and saying it is a rate-limited personal
  research retrieval; it carries no personal contact details;
- is resumable: a page already in the store with a recorded hash is not fetched again;
- stops after three consecutive server errors, rate-limit responses or transport failures
  instead of retrying indefinitely; an isolated error fails the run at the end;
- checks every enumerated URL against `robots.txt` before requesting any page, and follows
  redirects hop by hop under the same rules;
- stores a 4xx response with its status, so an absent page is evidence and not a gap.

An enumerator receives the manifest's `[acquire]` table and a function that reads a page through
the same polite fetcher, so an index page is fetched under the same rules as the pages it lists.
The generic enumerator `url_list` takes `urls = [...]` from `[acquire]`. The source-specific
enumerators take their one index page from `urls` and discover the rest: `janaf` reads the formula
index and the site's `dat/janaf.json` code list (the index itself carries no links) and yields
both pages and every `tables/<code>.txt`; `iapws` reads the release index and yields that page and
every document's `<document page>.download`, stored as `<slug>.pdf`. An index page that does not
have the expected structure is an error naming the page, never an empty enumeration. A completed `pages`
acquisition is never refreshed in place: a new retrieval is a new pin.

## 3. Store layout

```
.store/raw/<id>/<resolved-pin>/
  tree/                 the acquired files, untouched
  ACQUISITION.json      resolved pin, retrieval time, tool versions, and path, size and sha256 of every file
```

`<resolved-pin>` is the first twelve hex digits of the commit for `git`; of the declared checksum
for `archive` and for a single `file`; of the tree hash for several files; and the UTC date the
run completed for `pages`. `local` and `none` have no pin directory. A new pin is a new
directory; an existing directory is never modified in place. Work happens in a `.partial`
sibling that is renamed into place only when the acquisition completed, so an interrupted run
leaves nothing that looks complete.

When the acquisition completes, the stage makes the pin directory read-only: it, every directory
under it and every file lose their write permission, so nothing can rewrite third-party bytes in
place (links are left alone). A `.partial` directory stays writable, since it is where the
acquisition is built. A pin directory that must go (a tampered store, a pin to acquire again) is
made writable first, `chmod -R u+w .store/raw/<id>/<pin>`, then removed together with its entry in
`sources.lock`; `tk acquire <id>` then acquires it again.

An archive is stored as `tree/<archive name>` and, with `extract = true`, also as
`tree/extracted/`. A `git` tree holds the checked-out files only; line endings and filters are
disabled so the bytes are exactly the repository's blobs.

## 4. Lock

`sources.lock` is committed. It is JSON with sorted keys and a trailing newline, so it diffs
cleanly. Per source it records: acquisition kind, resolved pin, retrieval
time (UTC), file count, total bytes, and the **tree hash**: SHA-256 over the lines
`<sha256>  <relative path>\n` for every file under `tree/`, sorted by path. Per-file hashes stay
in `ACQUISITION.json` in the store. The lock carries a top-level `version`, and per source also
the full resolved commit or checksum, the path of a `local` source and the reason of a `none`
source. An entry keeps its retrieval time when re-verification resolves to the same thing.

Concurrent `tk acquire` runs are safe (for example `tk acquire janaf` and `tk acquire thermoml`
at the same time). A run keeps no copy of the lock. Recording one source is a read-modify-write
under an exclusive `flock` on the sidecar file `sources.lock.flock`, next to the lock and
gitignored by the tree: the run takes the lock, re-reads the current `sources.lock`, sets only
its own source's entry, writes a temporary file in the same directory and renames it over the
lock, then releases. Entries other runs recorded meanwhile are kept, the keep-or-record decision
compares with the entry just read, and the lock is never observable half-written. The file stays
byte-identical to the sorted JSON above. `flock` is Linux (POSIX) advisory locking: it binds
processes that use it, which every writer of the lock does, and the operating system releases it
when a run dies.

## 5. Commands

| Command | Effect |
|---|---|
| `tk acquire <id>...` | acquire the named sources (all when none is named) that are not already in the store at the declared pin; update the lock |
| `tk acquire --check [<id>...]` | offline: recompute each tree hash from the store and compare with the lock; report missing, extra and changed files by path |
| `tk acquire --list` | one line per source: tier, kind, declared pin, store state (absent, present, mismatched, not acquired) |

`--check` exits non-zero on any difference. A source with `kind = "none"` is reported as not
acquired and is not a failure. A declared source that was never acquired fails `--check` as
absent. `--sources PATH` and `--lock PATH` select other locations than the tree's.

Exit codes: 0 for success; 1 for a failed or refused acquisition, a manifest error or any
`--check` difference; 2 for an unknown source id or contradictory options. One failing source
does not stop the others, and the lock is written after each source.

`--list` states: `absent` (declared, not in the store), `present`, `mismatched` (the lock, the
store and the declared pin disagree) and `not acquired` (`kind = "none"`). A store or lock that
disagrees is refused, never repaired; a missing lock entry is restored from a store that
verifies.

## 6. Refusals

- a manifest with an unknown key, an unknown `kind`, or without at least one `[[rights]]` entry
- a `git` tag that does not name the declared commit
- a published checksum that does not match the delivered archive or file
- a `pages` URL disallowed by `robots.txt`
- writing into an existing `<resolved-pin>` directory
- a `local` checkout whose revision differs from the declared one, or which has local changes
