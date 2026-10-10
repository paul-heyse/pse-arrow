---
id: ADR-0125
title: Admit bulk package data as typed data documents
status: accepted
date: 2026-09-29
deciders: [paul-heyse]
level: decision
principles: [DP-01, DP-02, DP-20, PS-02]
blueprint: [§3.2, §5.3, §20.3, §22.1]
review: not-required: a data-document kind and admission path within ADR-0099, ADR-0116 and ADR-0123; reviewed with ADR-0123 at the maintainer's discretion
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A data bank exceeds the admission limits after caching, a consumer needs cross-package queries over admitted knowledge (register row for the relational projection), or a bank's upstream format cannot be represented by the declared Arrow types.
verification: Plan 23 KR9 tests parquet_dataset_admits_through_declared_relation, parquet_column_type_mismatch_refused, parquet_metadata_unit_disagreement_is_refused, parquet_reference_by_identifier_scheme_resolves, data_bytes_enter_package_checksum_and_source_revision, unchanged_dataset_reuses_admitted_table (incremental equals clean after one document edit), durable_job_round_trips_a_package_with_a_data_document and large_table_admits_without_cell_evaluation; the 10^5-row admission measurement; scenario DM1.
standard: core-3.1/process-simulator-1.1
scenarios: []
---

# ADR-0125: Admit bulk package data as typed data documents

## Context

Modeling tables are fed only by inline `.pse` datasets, whose cells are re-evaluated as
expressions. The data banks that are about to be ported (hundreds of species, many
correlations) cannot be authored or admitted that way. Plan 21 left the storage format of
large datasets open until the first dataset larger than the seed.

## Scope

This record decides that storage format. It adds a package document kind and an admission
path. It leaves open the relational projection of admitted knowledge. That projection has a
reserved shape and a register row.

## Drivers

- **Scale:** Plan 23 scenario DM1 and measurement M1.
- **Architecture §3.2:** the semantic crates stay free of Arrow and Parquet.
- **Identity:** package identity must cover data bytes.

## Options

| Option | Effect |
|---|---|
| Inline only | Unworkable at bank scale. Rejected |
| CSV | Untyped; every cell would be parsed from text. Rejected (DP-02) |
| Arrow IPC files | Typed, but has no compression or statistics, and external tooling is weaker. Rejected for authored files |
| Parquet data documents decoded by Arrow type in `pse-runtime`, admitted through the typed relation path | Selected |

## Outcome

**Document kind.** A package may carry `data/*.parquet` documents. Package documents
become bytes with a declared kind; text documents are one kind of document.

**Referencing.** A dataset declaration names its document by document-relative path.

**Decoding.** `pse-runtime` decodes each document by Arrow type into an Arrow-free row set:
- floating-point values become magnitudes in the column's declared storage unit;
- 16-byte binary values become declaration identities;
- text becomes an identifier or text;
- integers and booleans are read as themselves;
- dictionary values become enumeration members.

References are carried as identities or identifier-scheme values, never names. Column names
must equal the declared names.

**Units.** Storage units are declared only by the dataset's relation declaration. Parquet
metadata that disagrees with the declared unit is refused; it is never a second authority.

**Admission.** The ADR-0123 relation path admits the row set. The document's content hash
enters both the package checksum and the source revision (ADR-0123 Outcome 8). An admitted
table is a Salsa-tracked query over its table declaration and its document inputs; the
compiler workspace stays the single owner of reuse (§22.2).

**Inline cells** remain for seed-size data.

**Reserved projection.** The relational projection of admitted knowledge is deferred. It is
reserved as narrow, registry-declared relations over entities, attribute values, relation
rows and values.

### Consequences

- Everything that carries package documents moves from text to bytes with a declared kind:
  - the document registry;
  - the package loaders (`load_package_texts`);
  - `package_checksum`;
  - the operational store's source documents, which gain a binary content column (regenerated store, ADR-0114);
  - worker bundle verification.
- The compiler workspace tracks admitted tables.
- Rows are charged to the workspace limits.

### Compensating controls

- Refusals name the column and the declared type.
- The admission bench measures 10⁵ rows.
- Register row R-49 covers the projection.

### Confirmation

The KR9 targeted tests and scenario DM1 establish the behavior. Plan 23 owns
implementation status.

## Pros and cons

| For | Against |
|---|---|
| Typed, compressed, tool-friendly bank files with one admission path | Parquet authoring needs external tooling; the reference scripts generate it |

## More information

- [Plan 23](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/23-thermodynamic-domain-and-campaign.md).
- ADR-0116 (typed boundary documents) and ADR-0123.
- The [design review](../design_review/reviews/design_review_typed-domain-model_2026-09-29.md): its findings F01, F09 and F10 are incorporated in this record.

## Status history

- 2026-09-29: proposed; amended the same day with the design-review resolutions.
- 2026-09-29: accepted on the maintainer's Plan 23 authorization; design-review verdict Accept (§12–§13).
