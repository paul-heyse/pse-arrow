# Pure macro compiler controls

Run explicitly through the repository environment boundary:

```bash
scripts/pse-env --recipe-default-class compile -- python3 scripts/tests/fixtures/pure-macro-channels/qualify.py
```

This isolated, locked Rust fixture compiles the actual patched owners and existing
Pest generator. It exercises normal token pasting, literal/multiple inline grammar
parsing, Python literal behavior and upstream py_literal unit tests. Six actual
compiler invocations must refuse generated environment selectors across all three
Paste entries, direct/generated Pest file attributes and a nonliteral inline
expression. The environment selector is present, so refusal is the owner guard,
not a missing environment value. The grammar file is deliberately absent; a
file-access panic cannot satisfy the required guard diagnostic.

These controls establish the bounded owner behavior, not whole-source assurance
or current deployment/scientific qualification. `target` belongs to this fixture
and is never shared with the root target.
