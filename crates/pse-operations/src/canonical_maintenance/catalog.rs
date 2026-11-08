// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Connection-only catalog comparison for an already selected canonical issuer.

use super::{
    Definitions, ManagedCatalog, Marker, invalid, managed_table_names, parse_definitions, sha256,
};
use crate::canonical::{CanonicalError, CanonicalOptions, MESSAGE_BYTES, PAYLOAD_BYTES, wire};
use crate::transport::{
    REQUEST_TIMEOUT, bounded_query,
    client::{Client, IndexedResults, RequestContext, ResponseLimits, Root},
    operation_scope, original_deadline, within_clock,
};
use serde::Serialize;
use std::{net::SocketAddr, path::Path};
use surrealdb_types::{Object, Value};
use tokio::time::Instant;
use uuid::Uuid;

/// Exact preselected coordinates and required installed-store authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct CatalogTarget {
    /// Existing configured loopback gRPC endpoint.
    pub endpoint: String,
    /// Existing configured namespace.
    pub namespace: String,
    /// Existing configured or explicitly selected database.
    pub database: String,
    /// Persistent issuing-store identity required on the live connection.
    #[schemars(with = "String")]
    pub issuing_store: Uuid,
}

/// Identifies an installed declaration without returning SQL or credential material.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct CatalogDefinition {
    /// Managed definition family.
    pub kind: String,
    /// Owning table for a field, index or event.
    pub table: Option<String>,
    /// Declaration name or API path.
    pub name: String,
}

/// Exact normalized comparison to the current generated SQL definitions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct CatalogComparison {
    /// SHA-256 of the generated SQL input bytes.
    pub generated_declaration_sha256: String,
    /// Managed identities read from the installed catalog, in definition order.
    pub installed: Vec<CatalogDefinition>,
    /// Expected identities absent from the installed catalog.
    pub missing: Vec<CatalogDefinition>,
    /// Installed declarations with semantic changes at the same identity.
    pub changed: Vec<CatalogDefinition>,
    /// Extra managed definitions present in the installed catalog.
    pub unexpected: Vec<CatalogDefinition>,
}

/// Passive observation on one connection authenticated to the selected issuing store.
/// The maintenance owner separately attests quiescence, physical instance and installed module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, schemars::JsonSchema)]
pub struct CatalogObservation {
    /// Exact authenticated selection.
    pub target: CatalogTarget,
    /// Marker observed alongside the catalog. The module hash is the marker's claim.
    pub marker: Marker,
    /// Managed schema differences against the current generated target.
    pub comparison: CatalogComparison,
}

impl CatalogTarget {
    fn address(&self) -> Result<SocketAddr, CanonicalError> {
        if self.issuing_store.is_nil()
            || [&self.namespace, &self.database].iter().any(|identifier| {
                identifier.is_empty()
                    || identifier.len() > 128
                    || !identifier
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            })
        {
            return Err(invalid(
                "catalog target differs from the exact configured selection",
            ));
        }
        self.endpoint
            .strip_prefix("grpc://")
            .and_then(|address| address.parse::<SocketAddr>().ok())
            .filter(|address| address.ip().is_loopback() && address.port() != 0)
            .ok_or_else(|| invalid("catalog observation requires a numeric loopback gRPC endpoint"))
    }
}

/// Acquired private Viewer credential and exact configured coordinates for one observation.
#[derive(Debug)]
pub struct CatalogRead {
    target: CatalogTarget,
    address: SocketAddr,
    viewer: Root,
}

impl CatalogRead {
    /// Select the configured deployment and an existing database under the expected issuer.
    pub fn from_state(
        state: &Path,
        database: Option<&str>,
        expected_issuer: &str,
    ) -> Result<Self, CanonicalError> {
        if !state.is_absolute() {
            return Err(invalid(
                "catalog observation requires an absolute private state path",
            ));
        }
        let issuing_store = Uuid::parse_str(expected_issuer)
            .ok()
            .filter(|issuer| !issuer.is_nil())
            .ok_or_else(|| {
                invalid("catalog observation requires the original issuing-store UUID")
            })?;
        let (options, connection) = CanonicalOptions::read_connection_options(state, database)?;
        let target = CatalogTarget {
            endpoint: options.endpoint,
            namespace: options.namespace,
            database: options.database,
            issuing_store,
        };
        let address = target.address()?;
        if !Path::new(&connection.credentials_file).is_absolute()
            || options.selection_username == options.username
            || options.selection_username.is_empty()
            || options.selection_username.len() > 128
            || options.selection_password.is_empty()
            || options.selection_password.len() > 4096
        {
            return Err(invalid(
                "catalog observation requires distinct bounded viewer credentials",
            ));
        }
        let viewer = Root {
            username: options.selection_username,
            password: options.selection_password,
        };
        Ok(Self {
            target,
            address,
            viewer,
        })
    }

    /// Supply the sole transmitted credential to the response owner's redaction policy.
    pub fn viewer_password_for_redaction(&self) -> &str {
        &self.viewer.password
    }

    /// Read the marker and managed definitions in a bounded read transaction.
    pub async fn inspect(self) -> Result<CatalogObservation, CanonicalError> {
        let deadline = original_deadline(REQUEST_TIMEOUT);
        operation_scope(deadline.into_std(), async {
            let client = Client::connect_pinned(self.address, MESSAGE_BYTES).await?;
            let result = within_clock(
                deadline,
                observe(&client, &self.target, self.viewer, deadline),
            )
            .await;
            let drained = client.disconnect().await;
            let observation = result?;
            drained?;
            Ok(observation)
        })
        .await
    }
}

const MARKER: &str = "SELECT interpretation,schema_digest,study_module_sha256,issuing_store,payload_incarnation,schema_epoch,schema_version,admission_open FROM ONLY canonical_interpretations:current;";
const DATABASE: &str = "INFO FOR DB;";
const VIEWER: &str = "INFO FOR USER $viewer ON ROOT STRUCTURE;";
const LIMITS: ResponseLimits = ResponseLimits {
    wire_bytes: MESSAGE_BYTES,
    nodes: 32 * 1024,
    depth: 8,
    container_items: 16 * 1024,
    string_bytes: PAYLOAD_BYTES,
    string_length: 128 * 1024,
    blob_bytes: 0,
    blob_length: 0,
};

/// Read the installed catalog using only the configured viewer credentials.
/// The selected database and marker must already exist. All returned definitions
/// are observed in one bounded read transaction on the original native connection.
pub async fn inspect_catalog(
    state: &Path,
    target: &CatalogTarget,
) -> Result<CatalogObservation, CanonicalError> {
    let read = CatalogRead::from_state(
        state,
        Some(&target.database),
        &target.issuing_store.to_string(),
    )?;
    if &read.target != target {
        return Err(invalid(
            "catalog target differs from the exact configured selection",
        ));
    }
    read.inspect().await
}

async fn observe(
    client: &Client,
    target: &CatalogTarget,
    viewer: Root,
    deadline: Instant,
) -> Result<CatalogObservation, CanonicalError> {
    let context = || RequestContext::control(deadline.into_std());
    let query_context = || context().with_response_limits(LIMITS);
    let viewer_name = viewer.username.clone();
    client.signin(viewer).request_context(context()).await?;
    let mut response = bounded_query(
        client
            .query(VIEWER)
            .bind(("viewer", viewer_name.clone()))
            .request_context(query_context()),
    )
    .await?;
    let account = take_object(&mut response, 0)?;
    if account.get("name") != Some(&Value::String(viewer_name))
        || !matches!(account.get("roles"), Some(Value::Array(roles)) if matches!(roles.as_slice(), [Value::String(role)] if role.eq_ignore_ascii_case("viewer")))
    {
        return Err(invalid(
            "configured catalog account must have the viewer role",
        ));
    }
    client
        .use_ns(&target.namespace)
        .use_db(&target.database)
        .request_context(context())
        .await?;
    let mut response = bounded_query(client.query(MARKER).request_context(query_context())).await?;
    let initial_marker = take_object(&mut response, 0)?;
    let marker = read_marker(&initial_marker, target.issuing_store)?;

    let mut response =
        bounded_query(client.query(DATABASE).request_context(query_context())).await?;
    let initial = take_object(&mut response, 0)?;
    let tables = managed_table_names(&initial)?;
    let mut query = format!("BEGIN;\n{MARKER}\n{DATABASE}\n");
    for table in &tables {
        use std::fmt::Write as _;
        writeln!(query, "INFO FOR TABLE {table};").map_err(|_| CanonicalError::PayloadLimit)?;
    }
    query.push_str("COMMIT;");
    let mut response = bounded_query(
        client
            .query(query)
            .request_context(query_context())
            .statement_window(1, 1),
    )
    .await?;
    if response.num_statements() != tables.len() + 2 {
        return Err(CanonicalError::IncompleteResponse);
    }
    if take_object(&mut response, 0)? != initial_marker {
        return Err(invalid("catalog predecessor changed during observation"));
    }
    let database = take_object(&mut response, 1)?;
    if managed_table_names(&database)? != tables {
        return Err(invalid("managed table set changed during observation"));
    }
    let mut installed = ManagedCatalog::from_database(&database)?;
    for (index, table) in tables.into_iter().enumerate() {
        installed.append_table(table, &take_object(&mut response, index + 2)?)?;
    }
    Ok(CatalogObservation {
        target: target.clone(),
        marker,
        comparison: compare_generated(&parse_definitions(wire::SCHEMA)?, &installed.definitions),
    })
}

fn take_object(response: &mut IndexedResults, index: usize) -> Result<Object, CanonicalError> {
    let Value::Object(object) = response.take::<Value>(index)? else {
        return Err(CanonicalError::IncompleteResponse);
    };
    Ok(object)
}

fn read_marker(marker: &Object, issuer: Uuid) -> Result<Marker, CanonicalError> {
    let text = |name| match marker.get(name) {
        Some(Value::String(value)) if !value.is_empty() && value.len() <= 128 => Ok(value.clone()),
        _ => Err(invalid("installed catalog marker is absent or malformed")),
    };
    let issuing_store = text("issuing_store")?;
    if Uuid::parse_str(&issuing_store).ok() != Some(issuer) {
        return Err(invalid(
            "installed catalog issuing store differs from the selected issuer",
        ));
    }
    Ok(Marker {
        interpretation: text("interpretation")?,
        schema_digest: text("schema_digest")?,
        study_module_sha256: text("study_module_sha256")?,
        issuing_store,
    })
}

fn compare_generated(expected: &Definitions, installed: &Definitions) -> CatalogComparison {
    let identity = |(kind, table, name): &super::DefinitionKey| CatalogDefinition {
        kind: kind.clone(),
        table: (!table.is_empty()).then(|| table.clone()),
        name: name.clone(),
    };
    let mut comparison = CatalogComparison {
        generated_declaration_sha256: sha256(wire::SCHEMA.as_bytes()),
        installed: installed.keys().map(identity).collect(),
        missing: Vec::new(),
        changed: Vec::new(),
        unexpected: Vec::new(),
    };
    for (key, definition) in expected {
        match installed.get(key) {
            None => comparison.missing.push(identity(key)),
            Some(observed) if observed != definition => comparison.changed.push(identity(key)),
            Some(_) => {}
        }
    }
    for key in installed.keys().filter(|key| !expected.contains_key(*key)) {
        comparison.unexpected.push(identity(key));
    }
    comparison
}
