// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The migration-conformance test (ADR-0112 Outcome 10): the registry declares the meaning
//! of the operational relations, and the migrated `pse_ops` schema is their physical
//! representation. Relation `runtime.operational_<t>` is table `pse_ops.<t>`.

use std::collections::{BTreeMap, BTreeSet};

use arrow_schema::DataType;
use pse_schema::model::{FieldContract, Namespace, RelationSpec};
use sqlx::{PgPool, Row};

/// The registry name prefix of an operational relation.
const PREFIX: &str = "operational_";

/// Tables of the publication catalog, declared in the registry with the catalog commit path
/// (Plan 22 O8). Every other `pse_ops` table must be a declared operational relation.
const CATALOG: [&str; 7] = [
    "publication_heads",
    "publication_members",
    "publications",
    "reader_leases",
    "retention_marks",
    "settlements",
    "workspaces",
];

/// The PostgreSQL representation of a registry logical type, as `information_schema`
/// reports it: `data_type`, or `udt_name` for arrays. Identities are `uuid`, content
/// hashes `bytea`, enumerations `text` (with a CHECK), timestamps `timestamptz`. A text
/// field may be stored as `jsonb` when it holds a JSON document.
fn sql_types(field: &FieldContract) -> Result<&'static [&'static str], String> {
    if field.enum_name().is_some() {
        return Ok(&["text"]);
    }
    Ok(match field.data_type() {
        DataType::FixedSizeBinary(16) => &["uuid"],
        DataType::FixedSizeBinary(32) => &["bytea"],
        DataType::Utf8 => &["text", "jsonb"],
        DataType::Boolean => &["boolean"],
        DataType::Int32 => &["integer"],
        DataType::Int64 => &["bigint"],
        DataType::Float64 => &["double precision"],
        DataType::Timestamp(..) => &["timestamp with time zone"],
        DataType::List(item) => match item.data_type() {
            DataType::Float64 => &["_float8"],
            DataType::Int32 => &["_int4"],
            other => return Err(format!("no PostgreSQL array representation for {other}")),
        },
        other => return Err(format!("no PostgreSQL representation for {other}")),
    })
}

/// The quoted literals of a `col = ANY (ARRAY['a'::text, ...])` CHECK definition.
fn in_list(definition: &str) -> Option<BTreeSet<String>> {
    let start = definition.find("= ANY (ARRAY[")?;
    let list = &definition[start..];
    let end = list.find(']')?;
    Some(
        list[..end]
            .split('\'')
            .skip(1)
            .step_by(2)
            .map(str::to_owned)
            .collect(),
    )
}

#[derive(Debug)]
struct Column {
    sql_type: String,
    nullable: bool,
}

async fn columns(pool: &PgPool, table: &str) -> BTreeMap<String, Column> {
    sqlx::query(
        "SELECT column_name::text AS name, \
             CASE WHEN data_type = 'ARRAY' THEN udt_name::text ELSE data_type::text END AS ty, \
             is_nullable = 'YES' AS nullable \
         FROM information_schema.columns WHERE table_schema = 'pse_ops' AND table_name = $1",
    )
    .bind(table)
    .fetch_all(pool)
    .await
    .unwrap()
    .iter()
    .map(|row| {
        (
            row.get("name"),
            Column {
                sql_type: row.get("ty"),
                nullable: row.get("nullable"),
            },
        )
    })
    .collect()
}

async fn primary_key(pool: &PgPool, table: &str) -> Vec<String> {
    sqlx::query_scalar(
        "SELECT a.attname::text FROM pg_index i \
         JOIN pg_attribute a ON a.attrelid = i.indrelid AND a.attnum = ANY (i.indkey) \
         WHERE i.indrelid = ('pse_ops.' || quote_ident($1))::regclass AND i.indisprimary \
         ORDER BY array_position(i.indkey::int2[], a.attnum)",
    )
    .bind(table)
    .fetch_all(pool)
    .await
    .unwrap()
}

/// Single-column CHECK definitions by column.
async fn checks(pool: &PgPool, table: &str) -> BTreeMap<String, Vec<String>> {
    let mut by_column: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for row in sqlx::query(
        "SELECT a.attname::text AS name, pg_get_constraintdef(c.oid) AS def \
         FROM pg_constraint c \
         JOIN pg_attribute a ON a.attrelid = c.conrelid AND a.attnum = c.conkey[1] \
         WHERE c.conrelid = ('pse_ops.' || quote_ident($1))::regclass \
           AND c.contype = 'c' AND cardinality(c.conkey) = 1",
    )
    .bind(table)
    .fetch_all(pool)
    .await
    .unwrap()
    {
        by_column
            .entry(row.get("name"))
            .or_default()
            .push(row.get("def"));
    }
    by_column
}

/// Single-column foreign keys: column -> (table, column).
async fn references(pool: &PgPool, table: &str) -> BTreeSet<(String, String, String)> {
    sqlx::query(
        "SELECT a.attname::text AS name, t.relname::text AS target, f.attname::text AS target_column \
         FROM pg_constraint c \
         JOIN pg_attribute a ON a.attrelid = c.conrelid AND a.attnum = c.conkey[1] \
         JOIN pg_class t ON t.oid = c.confrelid \
         JOIN pg_attribute f ON f.attrelid = c.confrelid AND f.attnum = c.confkey[1] \
         WHERE c.conrelid = ('pse_ops.' || quote_ident($1))::regclass \
           AND c.contype = 'f' AND cardinality(c.conkey) = 1",
    )
    .bind(table)
    .fetch_all(pool)
    .await
    .unwrap()
    .iter()
    .map(|row| (row.get("name"), row.get("target"), row.get("target_column")))
    .collect()
}

async fn conform(
    pool: &PgPool,
    registry: &pse_schema::Registry,
    spec: &RelationSpec,
    table: &str,
    problems: &mut Vec<String>,
) {
    let mut report = |message: String| problems.push(format!("pse_ops.{table}: {message}"));
    let physical = columns(pool, table).await;
    let declared: BTreeSet<&str> = spec.columns.iter().map(FieldContract::name).collect();
    for name in physical.keys() {
        if !declared.contains(name.as_str()) {
            report(format!("column `{name}` is not declared by the registry"));
        }
    }
    let checks = checks(pool, table).await;
    for field in &spec.columns {
        let name = field.name();
        let Some(column) = physical.get(name) else {
            report(format!("declared column `{name}` is missing"));
            continue;
        };
        match sql_types(field) {
            Ok(types) if types.contains(&column.sql_type.as_str()) => {}
            Ok(types) => report(format!(
                "column `{name}` is {}, the registry type needs one of {types:?}",
                column.sql_type
            )),
            Err(error) => report(format!("column `{name}`: {error}")),
        }
        if column.nullable != field.nullable() {
            report(format!(
                "column `{name}` nullable {} but the registry says {}",
                column.nullable,
                field.nullable()
            ));
        }
        let lists: Vec<BTreeSet<String>> = checks
            .get(name)
            .into_iter()
            .flatten()
            .filter_map(|definition| in_list(definition))
            .collect();
        match field.enum_name() {
            Some(enumeration) => {
                let members: BTreeSet<String> = registry
                    .enum_spec(enumeration)
                    .map(|e| e.members.iter().map(|m| m.name.to_owned()).collect())
                    .unwrap_or_default();
                if lists != [members.clone()] {
                    report(format!(
                        "column `{name}` must CHECK exactly the {enumeration} spellings {members:?}; found {lists:?}"
                    ));
                }
            }
            None if !lists.is_empty() => report(format!(
                "column `{name}` has a value-domain CHECK {lists:?} but no registry enumeration"
            )),
            None => {}
        }
    }
    let key = primary_key(pool, table).await;
    if key != spec.primary_key {
        report(format!(
            "primary key {key:?}; the registry declares {:?}",
            spec.primary_key
        ));
    }
    let physical_references = references(pool, table).await;
    for field in &spec.columns {
        if let Some(fk) = field.fk() {
            let target = fk
                .relation
                .strip_prefix("runtime.")
                .and_then(|name| name.strip_prefix(PREFIX));
            let Some(target) = target else {
                report(format!(
                    "column `{}` references {} outside the operational relations",
                    field.name(),
                    fk.relation
                ));
                continue;
            };
            let expected = (
                field.name().to_owned(),
                target.to_owned(),
                fk.column.to_owned(),
            );
            if !physical_references.contains(&expected) {
                report(format!(
                    "column `{}` must reference pse_ops.{target}({})",
                    field.name(),
                    fk.column
                ));
            }
        }
    }
}

#[sqlx::test(migrator = "crate::MIGRATOR")]
async fn schema_matches_registry_relations(pool: PgPool) {
    let registry = pse_schema::shared_registry().unwrap();
    let declared: Vec<(&RelationSpec, &str)> = registry
        .relations()
        .iter()
        .filter(|spec| spec.key.namespace == Namespace::Runtime)
        .filter_map(|spec| {
            spec.key
                .name
                .strip_prefix(PREFIX)
                .map(|table| (spec, table))
        })
        .collect();
    assert!(
        declared.len() >= 11,
        "{} operational relations",
        declared.len()
    );
    let tables: BTreeSet<String> = sqlx::query_scalar(
        "SELECT table_name::text FROM information_schema.tables \
         WHERE table_schema = 'pse_ops' AND table_type = 'BASE TABLE'",
    )
    .fetch_all(&pool)
    .await
    .unwrap()
    .into_iter()
    .collect();
    let mut problems = Vec::new();
    for table in &tables {
        let is_declared = declared.iter().any(|(_, t)| t == table);
        if !is_declared && !CATALOG.contains(&table.as_str()) {
            problems.push(format!(
                "pse_ops.{table} is not declared as runtime.{PREFIX}{table}"
            ));
        }
    }
    for (spec, table) in &declared {
        if !tables.contains(*table) {
            problems.push(format!(
                "runtime.{PREFIX}{table} has no table pse_ops.{table}"
            ));
            continue;
        }
        conform(&pool, &registry, spec, table, &mut problems).await;
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));

    // Negative control: a drifted spelling set and a drifted type are both reported.
    sqlx::query(
        "ALTER TABLE pse_ops.jobs DROP CONSTRAINT jobs_state_check, \
         ADD CONSTRAINT jobs_state_check CHECK (state IN ('queued', 'running'))",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("ALTER TABLE pse_ops.incumbents ALTER COLUMN gap TYPE real")
        .execute(&pool)
        .await
        .unwrap();
    let mut drift = Vec::new();
    for (spec, table) in declared
        .iter()
        .filter(|(_, table)| ["jobs", "incumbents"].contains(table))
    {
        conform(&pool, &registry, spec, table, &mut drift).await;
    }
    assert_eq!(drift.len(), 2, "{drift:?}");
    assert!(drift.iter().any(|p| p.contains("JobState")), "{drift:?}");
    assert!(
        drift.iter().any(|p| p.contains("`gap` is real")),
        "{drift:?}"
    );
}

#[test]
fn value_domain_checks_are_parsed_from_their_definition() {
    assert_eq!(
        in_list("CHECK ((state = ANY (ARRAY['planned'::text, 'queued'::text])))"),
        Some(BTreeSet::from(["planned".to_owned(), "queued".to_owned()]))
    );
    assert_eq!(in_list("CHECK ((kind <> ''::text))"), None);
}
