# Invariant fixtures

Each directory contains complete typed input bindings for one registered invariant and
its exact expected violating keys. `valid.yaml` is valid for that invariant; it does not
claim that its isolated candidate is a publishable complete model. Other invariants may
require additional facts. Inputs remain unpublished and advertise no key constraints.

The files use the YAML 1.2 JSON subset and the schema's sole reversible tagged `Cell`
literal codec. Values, integer ranges, ordered tuples and floating-point bits are retained.
Expected keys come from concrete fixture cases, never from evaluating the rule to generate
its own oracle. The test compiles and executes each actual registered rule with force
validation, including explicit empty dependency relations.

Refresh reviewed fixtures with:

```sh
just conformance-fixtures
```

Verify with `just test-package pse-tests-conformance -p pse-relations --test invariant_fixtures`.
Fixture counts and matching hashes are not semantic evidence; actual results must equal
the declared typed keys.
