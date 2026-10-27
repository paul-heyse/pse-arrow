# Conversion of existing msgspec Struct instances

**Tested — 2026-10-27, source baseline `4f52fd239`, zero failure baseline.** This probe
checks a validation premise of the proposed CBO-050 target in comparative review
`b29fcf802`. It does not run or change the evidence pipeline.

## Question and realization

Does `msgspec.convert(value, Target, strict=True)` validate fields when `value`, or a
nested value, is already a Struct of the target type? The checkout's pinned msgspec
0.21.1 is used; `probe.py` asserts that version before running. The tiny frozen,
unknown-field-forbidding Structs deliberately contain an incorrectly typed field.

Context7's msgspec documentation lookup confirmed that Struct constructors do not
validate arguments. The exact-version implementation's `convert_other` fast path
returns an existing target Struct unchanged:
[msgspec 0.21.1 source](https://github.com/jcrist/msgspec/blob/0.21.1/src/msgspec/_core.c).
The probe checks the consequence rather than inferring validation from `strict=True`.

## Command and outcome

```bash
scripts/pse-env -- .venv/bin/python docs/design_review/evidence/library-leverage-msgspec-existing-struct-2026-10-27/probe.py > docs/design_review/evidence/library-leverage-msgspec-existing-struct-2026-10-27/output.json
```

Exit 0, zero failed assertions. `output.json` retains the observations:

- Converting the invalid `Snapshot` returns that same object, with its invalid field.
- Converting a raw outer mapping containing an existing invalid nested `Fact` retains
  that invalid nested object.
- Converting an outer mapping containing a raw invalid inner mapping raises
  `ValidationError` at `$.facts[0].value`.

Consequently, constructors followed by `convert`, or validating only the outer mapping,
do not establish the proposed Python validation guarantee. Validate raw input before
constructing Structs, including nested values, or cross a serialization/decode boundary
when revalidation is required. Domain and finite-number rules remain separate obligations.

## Limits

This checks only the stated 0.21.1 conversion paths. It does not establish the completeness
of the repository's generated types, field compatibility with existing TypedDicts,
publication behavior, redaction or finite-value enforcement. Those require source review
and later implementation tests. No performance claim or product qualification follows.
