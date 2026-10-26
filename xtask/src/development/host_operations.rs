// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Supplied-only evidence operation contribution; independent of the scientific extension.

/// Existing tooling-owned operation metadata. The generator resolves its document
/// references through the independently typed evidence catalogue in this owner.
pub fn declaration() -> serde_json::Value {
    serde_json::json!({
        "operation": "evidence_compare",
        "title": "Compare explicit baseline and candidate evidence",
        "receiver": "xtask::development::evidence_query::EvidenceAnalysis::compare",
        "description": "Compare supplied original evidence snapshots through the native finite owner. Select exact baseline/candidate generations and explicit correspondence where owner identities differ. Preserve full denominators, ambiguity, unsupported measurement associations and separately bounded sections. Opens no store and performs no scientific execution. Continue only on the exact returned cut.",
        "domain": "diagnostics",
        "effects": "Bounded supplied-only native evidence analysis; no build or service startup.",
        "envelope": "EvidenceCompare",
        "effect_facts": {"scope": "observation", "read_only": true, "idempotent": true, "destructive": false},
        "extension": null,
        "document_slugs": {"request": "evidence-comparison-request", "snapshots": "evidence-snapshot"},
        "output_slug": "evidence-comparison",
        "output_many": false,
        "worker": "evidence",
        "prerequisites": "evidence_executable",
        "prepare_import": false,
        "invocation_lifetime": "detached",
        "preserve_document": true,
        "read_eligibility": "supplied_tooling",
        "public_binding": true,
        "context_fields": [],
        "continuations": []
    })
}
