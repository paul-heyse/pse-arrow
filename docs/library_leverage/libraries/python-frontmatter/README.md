# python-frontmatter

Realizations considered: not installed. Upgrade route `python-frontmatter` 1.3.0 (PyPI,
released 2026-05-20). For comparison, installed: `ruamel.yaml` 0.19.1 (a direct pin in the
`docs` group) and `pyyaml` 6.0.3 (transitive).
Research date 2026-10-27. Sources:
- PyPI JSON.
- Repository inspection (static) of the document metadata reader:
  - `ruamel.yaml` `YAML(typ="safe", pure=True)` with `allow_duplicate_keys = False`;
  - a node walk rejecting merge keys and non-string keys;
  - byte-exact `---` delimiter detection;
  - a lazy import with a "run bootstrap-docs" message.
- python-frontmatter's model (`frontmatter.loads`/`load`, `Post.metadata`/`.content`,
  pluggable `handlers` for YAML, TOML and JSON, with the default YAML handler on PyYAML
  `SafeLoader`) is documentation knowledge.

## Types

### python-frontmatter/delimited-metadata-split
- **Need it serves:** Split a Markdown document into a metadata mapping and a body, with
  pluggable metadata formats.
- **Recognizable code shapes:**
  - `text.split("---", 2)`.
  - Regexes such as `^---\n.*?\n---\n`.
  - Per-script front-matter readers.
- **Mechanisms and APIs:**
  - `frontmatter.loads(text)` returns a `Post`, with `.metadata` and `.content`.
  - `frontmatter.dumps(post)`.
  - `handler=` selects a handler: `YAMLHandler`, `TOMLHandler` or `JSONHandler`. Custom
    handlers subclass `BaseHandler`.
- **Obligations discharged:** Delimiter detection, splitting, handler dispatch and
  round-trip dumping.
- **Residual caller obligations:**
  - Strictness: duplicate keys, merge keys, key types and date handling.
  - Encoding policy.
  - Bounded reads.
- **Semantic conditions:**
  - The default YAML handler uses PyYAML, which silently accepts duplicate keys (last wins)
    and resolves merge keys. Both are weaker than a strict ruamel reader.
  - A custom handler can wrap a strict parser. The library then contributes only splitting.
- **Version / build conditions:** Pure Python. Depends on PyYAML. Not stdlib.
- **Limits and negative results:** Not usable on stdlib-only cold-start paths.
- **Evidence:** Interface-checked for the version (PyPI). Model and strictness are
  documentation knowledge, and PyYAML's duplicate-key behaviour is general knowledge.
  Repository reader: Interface-checked by static reading.

## Current applicability

### [CBO-051](../../../library_leverage/codebase/developer-tooling.md#cbo-051) — Hand-written wire and front-matter parsers in tooling (front-matter part)
- **Types:** `python-frontmatter/delimited-metadata-split`.
- **Realization:** Upgrade route, a new dependency.
- **Conclusion:** Ruled out here.
  - Its default YAML path weakens the strict duplicate- and merge-key rejection.
  - With a custom strict handler it contributes only the delimiter split, which the
    existing strict reader already does.
  - It cannot serve the stdlib-only hook path.
- **What it would discharge / what remains local:** Nothing material. The consolidation
  available is local.
- **Prerequisites, configuration or extension:** n/a.
- **Semantic gaps against the CBO's preservation constraints:**
  - Duplicate-key rejection: weakened by default.
  - Stdlib-only cold start: violated.
- **Unsettled premises and what would settle them:** None for the library. The local
  question is whether the two remaining stdlib strippers should share one stdlib delimiter
  function. They are the hook's regex `status:` read and the agent-config checker's
  front-matter removal. The strict reader would expose that function and lazily import
  ruamel only for parsing.
- **Evidence:** Interface-checked (PyPI; static repository reading); application Proposed.
