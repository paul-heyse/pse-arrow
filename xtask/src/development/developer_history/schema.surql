-- SPDX-License-Identifier: MIT OR Apache-2.0
-- Copyright (c) 2026 Paul Heyse
-- Only for an explicitly provisioned developer history database.

DEFINE TABLE developer_history_deployment SCHEMAFULL;
DEFINE FIELD issuer ON developer_history_deployment TYPE string READONLY;
DEFINE FIELD schema_sha256 ON developer_history_deployment TYPE string READONLY;

DEFINE TABLE developer_test_history SCHEMAFULL;
DEFINE FIELD content_sha256 ON developer_test_history TYPE string READONLY;
DEFINE FIELD expires_unix_ms ON developer_test_history TYPE int READONLY;
DEFINE FIELD action ON developer_test_history TYPE string VALUE 'test' READONLY;
DEFINE FIELD fact ON developer_test_history TYPE object READONLY;
DEFINE FIELD fact.invocation ON developer_test_history TYPE string;
DEFINE FIELD fact.repository_id ON developer_test_history TYPE string;
DEFINE FIELD fact.checkout_id ON developer_test_history TYPE string;
DEFINE FIELD fact.control_manifest_sha256 ON developer_test_history TYPE string;
DEFINE FIELD fact.revision ON developer_test_history TYPE option<string>;
DEFINE FIELD fact.created_unix_ms ON developer_test_history TYPE int;
DEFINE FIELD fact.interpretation ON developer_test_history TYPE int;
DEFINE FIELD fact.outcome ON developer_test_history TYPE string
    ASSERT $value IN ['tests_succeeded', 'failed', 'zero_match', 'pending', 'incomplete'];
DEFINE FIELD fact.report_sha256 ON developer_test_history TYPE string;
DEFINE FIELD fact.inventory_sha256 ON developer_test_history TYPE string;
DEFINE FIELD fact.settlement_sha256 ON developer_test_history TYPE string;
DEFINE FIELD fact.selected_count ON developer_test_history TYPE int;
DEFINE FIELD fact.omitted_evidence ON developer_test_history TYPE option<int>;
DEFINE FIELD fact.failure_codes ON developer_test_history TYPE array<string>
    ASSERT array::len($value) <= __FAILURES__;
DEFINE FIELD fact.failed ON developer_test_history TYPE array<object>
    ASSERT array::len($value) <= __FAILURES__;
DEFINE FIELD fact.failed.*.package ON developer_test_history TYPE string;
DEFINE FIELD fact.failed.*.target_kind ON developer_test_history TYPE string;
DEFINE FIELD fact.failed.*.target_name ON developer_test_history TYPE string;
DEFINE FIELD fact.failed.*.enabled_features ON developer_test_history TYPE array<string>
    ASSERT array::len($value) <= __FEATURES__;
DEFINE FIELD fact.failed.*.absent_features ON developer_test_history TYPE array<string>
    ASSERT array::len($value) <= __FEATURES__;
DEFINE FIELD fact.failed.*.binary_id ON developer_test_history TYPE string;
DEFINE FIELD fact.failed.*.test_name ON developer_test_history TYPE string;
DEFINE FIELD fact.profile ON developer_test_history TYPE string;
DEFINE FIELD fact.platform ON developer_test_history TYPE option<string>;
DEFINE FIELD fact.filters ON developer_test_history TYPE object;
DEFINE FIELD fact.filters.ignore_default_filter ON developer_test_history TYPE bool;
DEFINE FIELD fact.filters.words ON developer_test_history TYPE array<string>
    ASSERT array::len($value) <= __FILTERS__;
DEFINE FIELD fact.filters.expressions ON developer_test_history TYPE array<string>
    ASSERT array::len($value) <= __FILTERS__;
DEFINE FIELD fact.ignored_policy ON developer_test_history TYPE string
    ASSERT $value IN ['default', 'only', 'all'];

DEFINE INDEX developer_test_history_expiry ON developer_test_history FIELDS expires_unix_ms;
