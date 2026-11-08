// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit maintenance validation and activation. Ordinary open never calls this.

use super::{
    ACTIVATION_REQUEST_TIMEOUT, CanonicalError, CanonicalStore, WriteAdmission, bounded_query,
    original_deadline, wire, within_clock,
};
#[cfg(test)]
use crate::canonical_script::ResultSlot;
use crate::canonical_script::{Envelope, GuardedScript};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};
use surrealdb_expr::{
    expr::{
        AccessType, Algorithm, Expr, Literal,
        access_type::JwtAccessVerify,
        statements::define::{DefineKind, DefineStatement},
    },
    parser::TopLevelExpr,
};
use surrealdb_types::{Object, ToSql, Value};

#[path = "canonical_maintenance/catalog.rs"]
mod catalog;
pub use catalog::{
    CatalogComparison, CatalogDefinition, CatalogObservation, CatalogRead, CatalogTarget,
    inspect_catalog,
};
#[path = "canonical_maintenance/frozen_rollout.rs"]
mod frozen_rollout;
pub use frozen_rollout::{FrozenRollout, RolloutPhase};

/// Exact predecessor, distinct from the generated target and beta schema hash.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
pub struct Marker {
    /// Original interpretation.
    pub interpretation: String,
    /// Original repository schema plus codec digest.
    pub schema_digest: String,
    /// Exact deployed pure-policy artifact.
    pub study_module_sha256: String,
    /// Issuing authority retained only by compatible forward activation.
    pub issuing_store: String,
}

/// Actual installed-definition observation under the selected live maintenance owner.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DefinitionWitness {
    /// A phase postcondition cannot authorize activation of the generated target.
    pub scope: String,
    /// SHA256 of the exact ordered declaration inputs, distinct from codec identity.
    pub definition_inputs: Vec<String>,
    /// Current generated interpretation.
    pub interpretation: String,
    /// Current generated declaration plus codec digest.
    pub schema_digest: String,
    /// Exact deployed pure-policy artifact.
    pub study_module_sha256: String,
    /// Exact selected namespace.
    pub namespace: String,
    /// Exact selected database.
    pub database: String,
    /// Supervisor instance, not a substitute for database issuing authority.
    pub instance_id: String,
    /// Current authenticated initializer reservation.
    pub initializer_nonce: String,
    /// Number of exact managed definitions checked.
    pub catalog_definition_count: usize,
    /// Marker before activation; absent for a fresh schema-only installation.
    pub previous_marker: Option<Marker>,
}

type DefinitionKey = (String, String, String);
type Definitions = BTreeMap<DefinitionKey, DefineStatement>;

fn sha256(bytes: &[u8]) -> String {
    use sha2::Digest as _;
    sha2::Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn invalid(reason: impl Into<String>) -> CanonicalError {
    CanonicalError::Configuration(reason.into())
}

fn connection_matches(
    options: &super::CanonicalOptions,
    config: &serde_json::Value,
    credentials: &serde_json::Value,
) -> bool {
    config.get("endpoint").and_then(serde_json::Value::as_str) == Some(options.endpoint.as_str())
        && config.get("namespace").and_then(serde_json::Value::as_str)
            == Some(options.namespace.as_str())
        && config.get("database").and_then(serde_json::Value::as_str)
            == Some(options.database.as_str())
        && [
            ("username", &options.username),
            ("password", &options.password),
            ("selection_username", &options.selection_username),
            ("selection_password", &options.selection_password),
        ]
        .iter()
        .all(|(key, value)| {
            credentials.get(*key).and_then(serde_json::Value::as_str) == Some(value.as_str())
        })
}

fn listener_matches(configured: &serde_json::Value, observed: &serde_json::Value) -> bool {
    configured
        .get("instance_id")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|id| !id.is_empty())
        && [
            ("instance_id", "instance_id"),
            ("service_supervisor", "supervisor"),
            ("server", "server"),
            ("resources", "resources"),
        ]
        .iter()
        .all(|(config_key, listener_key)| {
            configured.get(*config_key).is_some_and(|expected| {
                !expected.is_null() && observed.get(*listener_key) == Some(expected)
            })
        })
}

/// Compare the owned implicit JWT policy, not secret bytes hidden by server INFO.
/// Other access kinds/algorithms remain exact; signin/token tests qualify deployed keys.
fn normalize_control_access(
    input: &str,
    access: &mut surrealdb_expr::expr::statements::define::DefineAccessStatement,
) -> Result<(), CanonicalError> {
    if access.name.to_sql() != "pse_control" {
        return Ok(());
    }
    let AccessType::Record(record) = &mut access.access_type else {
        return Err(invalid("control access must retain record authentication"));
    };
    let JwtAccessVerify::Key(verify) = &mut record.jwt.verify else {
        return Err(invalid("control access must retain server-managed HS512"));
    };
    let Some(issue) = &mut record.jwt.issue else {
        return Err(invalid("control access issuer missing"));
    };
    if verify.alg != Algorithm::Hs512 || issue.alg != Algorithm::Hs512 || verify.key != issue.key {
        return Err(invalid("control access JWT policy differs"));
    }
    let Expr::Literal(Literal::String(key)) = &verify.key else {
        return Err(invalid("control access key is not server managed"));
    };
    let key = key.as_str();
    if key != "[REDACTED]"
        && (key.len() != 128
            || !key.bytes().all(|byte| byte.is_ascii_alphanumeric())
            || input.contains(key))
    {
        return Err(invalid(
            "explicit control access key requires a separate policy decision",
        ));
    }
    verify.key = Expr::Literal(Literal::String("[server-managed]".into()));
    issue.key = verify.key.clone();
    Ok(())
}

fn parse_definitions(input: &str) -> Result<Definitions, CanonicalError> {
    parse_definition_input(input, DefinitionInput::Schema)
}

#[derive(Clone, Copy)]
enum DefinitionInput {
    Schema,
    CatalogEntry,
    FrozenRollout,
}

fn parse_definition_input(
    input: &str,
    mode: DefinitionInput,
) -> Result<Definitions, CanonicalError> {
    let ast = surrealdb_expr::parser::parse(input)
        .map_err(|error| invalid(format!("generated catalog parse: {error}")))?;
    if matches!(mode, DefinitionInput::CatalogEntry) && ast.expressions.len() != 1 {
        return Err(invalid("catalog entry must contain one declaration"));
    }
    let mut definitions = BTreeMap::new();
    for expression in ast.expressions {
        let TopLevelExpr::Expr(Expr::Define(statement)) = expression else {
            return Err(invalid(
                "only generated schema declarations may be validated",
            ));
        };
        let mut statement = *statement;
        if matches!(mode, DefinitionInput::FrozenRollout)
            && (matches!(&statement, DefineStatement::Table(table) if table.drop || table.view.is_some())
                || matches!(&statement, DefineStatement::Field(field) if field.migrate.is_some())
                || matches!(&statement, DefineStatement::Index(index) if index.concurrently))
        {
            return Err(invalid(
                "frozen rollout requires independent effect handling",
            ));
        }
        // These flags affect installation, not the installed definition. Every
        // semantic clause (including permission/assertion/function body) survives.
        let key = match &mut statement {
            DefineStatement::Table(table) => {
                table.kind = DefineKind::Default;
                table.id = None;
                ("table".into(), String::new(), table.name.to_sql())
            }
            DefineStatement::Field(field) => {
                field.kind = DefineKind::Default;
                ("field".into(), field.what.to_sql(), field.name.to_sql())
            }
            DefineStatement::Index(index) => {
                index.kind = DefineKind::Default;
                ("index".into(), index.what.to_sql(), index.name.to_sql())
            }
            DefineStatement::Event(event) => {
                event.kind = DefineKind::Default;
                (
                    "event".into(),
                    event.target_table.to_sql(),
                    event.name.to_sql(),
                )
            }
            DefineStatement::Api(api) => {
                api.kind = DefineKind::Default;
                ("api".into(), String::new(), api.path.to_sql())
            }
            DefineStatement::Access(access) => {
                access.kind = DefineKind::Default;
                normalize_control_access(input, access)?;
                ("access".into(), String::new(), access.name.to_sql())
            }
            DefineStatement::Function(function) => {
                function.kind = DefineKind::Default;
                (
                    "function".into(),
                    String::new(),
                    function.name.as_str().into(),
                )
            }
            _ => return Err(invalid("unsupported generated declaration family")),
        };
        // OVERWRITE may replace the generated relation endpoint declaration.
        definitions.insert(key, statement);
    }
    Ok(definitions)
}

fn catalog_map<'a>(info: &'a Object, category: &str) -> Result<&'a Object, CanonicalError> {
    match info.get(category) {
        Some(Value::Object(values)) => Ok(values),
        _ => Err(invalid(format!(
            "catalog {category} is absent or malformed"
        ))),
    }
}

#[derive(Default)]
struct ManagedCatalog {
    definitions: Definitions,
    declaration_bytes: usize,
}

fn managed_table_names(info: &Object) -> Result<Vec<&str>, CanonicalError> {
    let names = catalog_map(info, "tables")?
        .keys()
        .filter(|name| name.starts_with("canonical_"))
        .map(String::as_str)
        .collect::<Vec<_>>();
    if names.len() > 256 {
        return Err(CanonicalError::PayloadLimit);
    }
    if names.iter().any(|name| {
        name.len() > 128
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    }) {
        return Err(invalid("managed table identifier differs"));
    }
    Ok(names)
}

impl ManagedCatalog {
    fn from_database(info: &Object) -> Result<Self, CanonicalError> {
        let mut catalog = Self::default();
        for (category, kind, prefix) in [
            ("tables", "table", "canonical_"),
            ("functions", "function", "pse_"),
            ("apis", "api", "/pse/"),
            ("accesses", "access", "pse_"),
        ] {
            for (name, value) in catalog_map(info, category)? {
                let name = if kind == "function" {
                    name.trim_start_matches("fn::")
                } else {
                    name
                };
                if name.starts_with(prefix) {
                    catalog.append(kind, "", value)?;
                }
            }
        }
        Ok(catalog)
    }

    fn append_table(&mut self, name: &str, info: &Object) -> Result<(), CanonicalError> {
        for (category, kind) in [
            ("fields", "field"),
            ("indexes", "index"),
            ("events", "event"),
        ] {
            for value in catalog_map(info, category)?.values() {
                self.append(kind, name, value)?;
            }
        }
        match info.get("lives") {
            None => Ok(()),
            Some(Value::Object(entries)) if entries.is_empty() => Ok(()),
            _ => Err(invalid(format!(
                "unowned or malformed lives on managed table {name}"
            ))),
        }
    }

    fn append(&mut self, kind: &str, table: &str, value: &Value) -> Result<(), CanonicalError> {
        let Value::String(sql) = value else {
            return Err(invalid("catalog definition is not a string"));
        };
        self.declaration_bytes = self.declaration_bytes.saturating_add(sql.len());
        if self.declaration_bytes > super::PAYLOAD_BYTES || self.definitions.len() >= 8192 {
            return Err(CanonicalError::PayloadLimit);
        }
        let mut parsed = parse_definition_input(sql.as_str(), DefinitionInput::CatalogEntry)
            .map_err(|_| invalid("installed managed declaration is malformed"))?;
        let Some((key, definition)) = parsed.pop_first() else {
            return Err(invalid("catalog entry must contain one declaration"));
        };
        if key.0 != kind || key.1 != table {
            return Err(invalid(
                "catalog entry does not match its definition family and table",
            ));
        }
        if self.definitions.insert(key, definition).is_some() {
            return Err(invalid("duplicate installed declaration"));
        }
        Ok(())
    }
}

fn compare(expected: &Definitions, observed: &Definitions) -> Result<(), CanonicalError> {
    for (key, definition) in expected {
        if observed.get(key) != Some(definition) {
            return Err(invalid(format!(
                "installed generated definition differs: {} {} {}",
                key.0, key.1, key.2
            )));
        }
    }
    if let Some(key) = observed.keys().find(|key| !expected.contains_key(*key)) {
        return Err(invalid(format!(
            "unexpected managed definition: {} {} {}",
            key.0, key.1, key.2
        )));
    }
    Ok(())
}

impl CanonicalStore {
    pub(crate) fn maintenance_config(&self) -> Result<serde_json::Value, CanonicalError> {
        let configuration_bytes = crate::deployment::CONFIGURATION_FILE_BYTES as usize;
        let read = |name, max_bytes| {
            frozen_rollout::bounded_file(&self.state_path.join(name), max_bytes, true)
        };
        let raw = read("config.json", configuration_bytes)?;
        let admission: WriteAdmission = serde_json::from_str(&raw)
            .map_err(|_| invalid("invalid maintenance admission configuration"))?;
        admission.require_initializer(
            &self.state_path,
            &self.namespace,
            &self.database,
            self.connection_options.initializer_nonce.as_deref(),
        )?;
        let config =
            serde_json::from_str(&raw).map_err(|_| invalid("invalid maintenance configuration"))?;
        let owner_record = |name| {
            let raw = read(name, configuration_bytes).map_err(|_| CanonicalError::Quiesced)?;
            serde_json::from_str::<serde_json::Value>(&raw).map_err(|_| CanonicalError::Quiesced)
        };
        let listener = owner_record(".maintenance-listener.json")?;
        let owner = owner_record("lifecycle-owner.json")?;
        if listener.get("owner") != Some(&owner) || !listener_matches(&config, &listener) {
            return Err(CanonicalError::Quiesced);
        }
        let credentials = read("credentials.json", super::CREDENTIALS_FILE_BYTES as usize)
            .map_err(|_| CanonicalError::Quiesced)?;
        let credential_document: serde_json::Value =
            serde_json::from_str(&credentials).map_err(|_| CanonicalError::Quiesced)?;
        if !connection_matches(&self.connection_options, &config, &credential_document) {
            return Err(CanonicalError::Quiesced);
        }
        let digest = sha256(credentials.as_bytes());
        if listener
            .get("credentials_sha256")
            .and_then(serde_json::Value::as_str)
            != Some(digest.as_str())
        {
            return Err(CanonicalError::Quiesced);
        }
        Ok(config)
    }

    async fn maintenance_marker(&self) -> Result<Option<Marker>, CanonicalError> {
        let mut response = bounded_query(
            self.db
                .query("SELECT * FROM ONLY canonical_interpretations:current;"),
        )
        .await?;
        match response.take::<Value>(0)? {
            Value::None | Value::Null => Ok(None),
            Value::Object(object) => {
                let marker = wire::decode_canonical_interpretations(object)?;
                if marker.payload_incarnation != self.files.incarnation {
                    return Err(invalid("maintenance selected payload incarnation differs"));
                }
                Ok(Some(Marker {
                    interpretation: marker.interpretation,
                    schema_digest: marker.schema_digest,
                    study_module_sha256: marker.study_module_sha256,
                    issuing_store: marker.issuing_store,
                }))
            }
            _ => Err(CanonicalError::IncompleteResponse),
        }
    }

    /// Check every managed installed definition with the exact server's parse-only AST.
    /// This never installs schema, changes a marker or opens write admission.
    pub async fn validate_definitions(&self) -> Result<DefinitionWitness, CanonicalError> {
        self.validate_definition_inputs(&[wire::SCHEMA.to_owned()], "generated-target")
            .await
    }

    /// Observe a producer-selected phase postcondition. This read-only result is
    /// explicitly distinct from the fixed generated target required for activation.
    pub async fn validate_definition_files(
        &self,
        files: &[std::path::PathBuf],
    ) -> Result<DefinitionWitness, CanonicalError> {
        if files.is_empty() || files.len() > 2 {
            return Err(invalid(
                "one target or ordered source/target inputs required",
            ));
        }
        let mut inputs = Vec::new();
        for file in files {
            let bytes = std::fs::read(file).map_err(|error| invalid(error.to_string()))?;
            if bytes.len() > super::PAYLOAD_BYTES {
                return Err(CanonicalError::PayloadLimit);
            }
            inputs.push(String::from_utf8(bytes).map_err(|error| invalid(error.to_string()))?);
        }
        self.validate_definition_inputs(&inputs, "rollout-phase")
            .await
    }

    async fn validate_definition_inputs(
        &self,
        inputs: &[String],
        scope: &str,
    ) -> Result<DefinitionWitness, CanonicalError> {
        let activity = self.admit_activity()?;
        activity
            .scope(async {
                within_clock(original_deadline(ACTIVATION_REQUEST_TIMEOUT), async {
                    let config = self.maintenance_config()?;
                    let module =
                        crate::canonical::study_module::StudyModuleReceipt::read(&self.state_path)?;
                    self.validate_study_module(&module).await?;
                    let mut expected = BTreeMap::new();
                    for input in inputs {
                        expected.extend(parse_definitions(input)?);
                    }
                    let info = self
                        .database_inventory()
                        .await?
                        .ok_or_else(|| invalid("selected database absent"))?;
                    let mut observed = ManagedCatalog::from_database(&info)?;
                    for name in managed_table_names(&info)? {
                        self.maintenance_config()?;
                        let mut response =
                            bounded_query(self.db.query(format!("INFO FOR TABLE {name};"))).await?;
                        let Value::Object(table_info) = response.take::<Value>(0)? else {
                            return Err(CanonicalError::IncompleteResponse);
                        };
                        observed.append_table(name, &table_info)?;
                    }
                    compare(&expected, &observed.definitions)?;
                    let previous_marker = self.maintenance_marker().await?;
                    if self.maintenance_config()? != config {
                        return Err(CanonicalError::Quiesced);
                    }
                    Ok(DefinitionWitness {
                        scope: scope.into(),
                        definition_inputs: inputs
                            .iter()
                            .map(|input| sha256(input.as_bytes()))
                            .collect(),
                        interpretation: wire::INTERPRETATION.into(),
                        schema_digest: wire::SCHEMA_DIGEST.into(),
                        study_module_sha256: module.sha256.clone(),
                        namespace: self.namespace.clone(),
                        database: self.database.clone(),
                        instance_id: config
                            .get("instance_id")
                            .and_then(serde_json::Value::as_str)
                            .ok_or_else(|| invalid("instance identity absent"))?
                            .into(),
                        initializer_nonce: self
                            .connection_options
                            .initializer_nonce
                            .clone()
                            .ok_or(CanonicalError::Quiesced)?,
                        catalog_definition_count: expected.len(),
                        previous_marker,
                    })
                })
                .await
            })
            .await
    }

    /// Publish the generated target only after the exact production owner settled it.
    /// A fresh installation receives fresh issuing authority. Forward activation
    /// preserves authority only under the same scientific interpretation.
    pub async fn activate_definitions(
        &self,
        settlement_path: &Path,
    ) -> Result<DefinitionWitness, CanonicalError> {
        let activity = self.admit_activity()?;
        activity.scope(async {
            within_clock(original_deadline(ACTIVATION_REQUEST_TIMEOUT), async {
                let config = self.maintenance_config()?;
                let pending = config.get("surrealkit_rollout_pending").ok_or_else(|| invalid("exact pending rollout required"))?;
                if pending.get("phase").and_then(serde_json::Value::as_str) != Some("settled") {
                    return Err(invalid("exact rollout phase is not completely settled"));
                }
                let nonce = pending.get("nonce").and_then(serde_json::Value::as_str).ok_or_else(|| invalid("pending nonce absent"))?;
                if nonce.len() != 32 || !nonce.bytes().all(|byte| byte.is_ascii_hexdigit()) || settlement_path != self.state_path.join(".surrealkit").join(nonce).join("settled.json") { return Err(invalid("exact owned settlement path required")); }
                let settled: serde_json::Value = serde_json::from_str(
                    &frozen_rollout::bounded_file(settlement_path, crate::deployment::CONFIGURATION_FILE_BYTES as usize, true)?,
                ).map_err(|_| invalid("invalid owned settlement record"))?;
                for key in ["nonce", "input_sha256", "instance_id", "namespace", "database", "settlement", "previous_marker"] {
                    if pending.get(key).is_none() || pending.get(key) != settled.get(key) { return Err(invalid(format!("settlement binding differs: {key}"))); }
                }
                let evidence = settled.get("settlement").ok_or_else(|| invalid("settlement absent"))?;
                if evidence.get("phase_complete") != Some(&serde_json::Value::Bool(true)) {
                    return Err(invalid("SQL settlement alone cannot activate an incomplete rollout"));
                }
                if evidence.get("outcome").and_then(serde_json::Value::as_str) != Some("settled") || evidence.get("nonce").and_then(serde_json::Value::as_str) != Some(nonce) || evidence.get("input_sha256") != pending.get("input_sha256") { return Err(invalid("actual operation settlement required")); }
                let recorded: DefinitionWitness = serde_json::from_value(evidence.get("canonical").cloned().ok_or_else(|| invalid("canonical definition witness absent"))?).map_err(|error| invalid(error.to_string()))?;
                let witness = self.validate_definitions().await?;
                if recorded.scope != "generated-target" || recorded.definition_inputs != witness.definition_inputs || recorded.interpretation != witness.interpretation || recorded.schema_digest != witness.schema_digest || recorded.study_module_sha256 != witness.study_module_sha256 || recorded.namespace != witness.namespace || recorded.database != witness.database || recorded.instance_id != witness.instance_id || recorded.catalog_definition_count != witness.catalog_definition_count { return Err(invalid("settled target differs from actual installed target")); }
                let predecessor: Option<Marker> = serde_json::from_value(pending.get("previous_marker").cloned().ok_or_else(|| invalid("original predecessor absent"))?).map_err(|error| invalid(error.to_string()))?;
                let predecessor_bytes = serde_json::to_vec(&predecessor).map_err(|error| invalid(error.to_string()))?;
                let input_sha256 = pending.get("input_sha256").and_then(serde_json::Value::as_str)
                    .filter(|digest| is_sha256(digest)).ok_or_else(|| invalid("pending input identity absent or invalid"))?;
                if recorded.previous_marker != predecessor
                    || pending.get("instance_id").and_then(serde_json::Value::as_str) != Some(witness.instance_id.as_str())
                    || pending.get("namespace").and_then(serde_json::Value::as_str) != Some(witness.namespace.as_str())
                    || pending.get("database").and_then(serde_json::Value::as_str) != Some(witness.database.as_str())
                {
                    return Err(invalid("original rollout deployment or predecessor differs"));
                }
                let mut existing = bounded_query(self.db.query("SELECT * FROM ONLY type::record('canonical_maintenance_activations',$nonce);").bind(("nonce",nonce.to_owned()))).await?;
                if let Some(receipt) = existing.take::<Option<Object>>(0)?.map(wire::decode_canonical_maintenance_activations).transpose()? {
                    if receipt.key != nonce
                        || receipt.input_sha256 != input_sha256
                        || receipt.predecessor.as_slice() != predecessor_bytes.as_slice()
                        || receipt.interpretation != wire::INTERPRETATION
                        || receipt.schema_digest != wire::SCHEMA_DIGEST
                        || receipt.study_module_sha256 != witness.study_module_sha256
                        || predecessor.as_ref().is_some_and(|marker| marker.issuing_store != receipt.issuing_store)
                        || witness.previous_marker != Some(Marker { interpretation: wire::INTERPRETATION.into(), schema_digest: wire::SCHEMA_DIGEST.into(), study_module_sha256: witness.study_module_sha256.clone(), issuing_store: receipt.issuing_store }) {
                        return Err(CanonicalError::OperationReused);
                    }
                    self.maintenance_config()?;
                    self.open().await?;
                    return Ok(witness);
                }
                if witness.previous_marker != predecessor { return Err(invalid("activation predecessor changed")); }
                if predecessor.as_ref().is_some_and(|marker| marker.interpretation != wire::INTERPRETATION || marker.study_module_sha256!=witness.study_module_sha256 || uuid::Uuid::parse_str(&marker.issuing_store).is_err()) { return Err(invalid("incompatible predecessor requires explicit fresh reset")); }
                if predecessor.is_none() {
                    let definitions = parse_definitions(wire::SCHEMA)?;
                    for key in definitions.keys().filter(|key| key.0 == "table") {
                        let table = &key.2;
                        if !table.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_') { return Err(invalid("generated table identity differs")); }
                        let mut response = bounded_query(self.db.query(format!("RETURN array::len(SELECT id FROM {table} LIMIT 1);"))).await?;
                        let count: Value = response.take(0)?;
                        if crate::canonical_codec::decode_int(count)? != 0 { return Err(invalid("unmarked scientific data requires explicit fresh reset")); }
                    }
                }
                if self.maintenance_config()? != config { return Err(CanonicalError::Quiesced); }
                let issuing = predecessor.as_ref().map(|marker| marker.issuing_store.clone()).unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
                let script = GuardedScript::new(Envelope::DeadlineOnly).returning::<Value>("", "fn::pse_maintenance_v1::activate($pse_rpc_expires_at,$predecessor,$interpretation,$schema_digest,$study_module_sha256,$issuing,$payload_incarnation,$nonce,$input_sha256,$predecessor_bytes)")?;
                let predecessor_value = match predecessor { Some(marker) => { let mut value=Object::new(); value.insert("interpretation", marker.interpretation); value.insert("schema_digest", marker.schema_digest); value.insert("study_module_sha256", marker.study_module_sha256); value.insert("issuing_store", marker.issuing_store); Value::Object(value) }, None => Value::None };
                let request = serde_json::to_vec(&(nonce, input_sha256, &predecessor_bytes, wire::INTERPRETATION, wire::SCHEMA_DIGEST, &witness.study_module_sha256, &issuing, &self.files.incarnation)).map_err(|error| invalid(error.to_string()))?;
                let expected = pse_model::generated::runtime::canonical_maintenance_activations::Row {
                    key: nonce.to_owned(), input_sha256: input_sha256.to_owned(),
                    predecessor: predecessor_bytes.clone().into(),
                    interpretation: wire::INTERPRETATION.into(), schema_digest: wire::SCHEMA_DIGEST.into(),
                    study_module_sha256: witness.study_module_sha256.clone(), issuing_store: issuing.clone(),
                };
                let write = bounded_query(self.activation_db.script(script).bind(("predecessor",predecessor_value)).bind(("interpretation",wire::INTERPRETATION)).bind(("schema_digest",wire::SCHEMA_DIGEST)).bind(("study_module_sha256",witness.study_module_sha256.clone())).bind(("issuing",issuing)).bind(("payload_incarnation",self.files.incarnation.clone())).bind(("nonce",nonce.to_owned())).bind(("input_sha256",input_sha256.to_owned())).bind(("predecessor_bytes",surrealdb_types::Bytes::from(predecessor_bytes)))).await.and_then(super::complete_response).map(|_| expected.clone());
                super::settlement::settle_exact(
                    "maintenance-activation", nonce, &request, write,
                    async {
                        let mut response = bounded_query(self.db.query("SELECT * FROM ONLY type::record('canonical_maintenance_activations',$nonce);").bind(("nonce",nonce.to_owned()))).await?;
                        response.take::<Option<Object>>(0)?.map(wire::decode_canonical_maintenance_activations).transpose().map_err(Into::into)
                    },
                    |receipt| receipt == &expected,
                    CanonicalError::source_transaction_refused,
                ).await?;
                self.maintenance_config()?;
                self.open().await?;
                Ok(witness)
            }).await
        }).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maintenance_listener_requires_every_physical_owner_identity() {
        let mut configuration = serde_json::json!({
            "instance_id": "live",
            "service_supervisor": {"invocation": "exact"},
            "server": {"recipe": "exact"},
            "resources": {"unit": "exact"}
        });
        let listener = serde_json::json!({
            "instance_id": "live",
            "supervisor": {"invocation": "exact"},
            "server": {"recipe": "exact"},
            "resources": {"unit": "exact"}
        });
        assert!(listener_matches(&configuration, &listener));
        for key in ["instance_id", "service_supervisor", "server", "resources"] {
            let mut changed = configuration.clone();
            changed[key] = serde_json::Value::Null;
            assert!(!listener_matches(&changed, &listener));
            let mut absent = configuration.clone();
            absent.as_object_mut().unwrap().remove(key);
            assert!(!listener_matches(&absent, &listener));
        }
        configuration["service_supervisor"]["invocation"] = serde_json::json!("foreign");
        assert!(!listener_matches(&configuration, &listener));
        let absent = serde_json::json!({});
        assert!(!listener_matches(&absent, &absent));
    }

    #[test]
    fn maintenance_authority_binds_original_connection_and_credentials() {
        let options = super::super::CanonicalOptions {
            endpoint: "grpc://127.0.0.1:12345".into(),
            namespace: "selected".into(),
            database: "selected".into(),
            username: "owner".into(),
            password: "private".into(),
            selection_username: "reader".into(),
            selection_password: "private-reader".into(),
            state_path: std::path::PathBuf::new(),
            initializer_nonce: Some("nonce".into()),
        };
        let config = serde_json::json!({"endpoint":options.endpoint,"namespace":options.namespace,"database":options.database});
        let credentials = serde_json::json!({"username":options.username,"password":options.password,"selection_username":options.selection_username,"selection_password":options.selection_password});
        assert!(connection_matches(&options, &config, &credentials));
        for key in ["endpoint", "namespace", "database"] {
            let mut changed = config.clone();
            changed[key] = serde_json::json!("foreign");
            assert!(!connection_matches(&options, &changed, &credentials));
        }
        for key in [
            "username",
            "password",
            "selection_username",
            "selection_password",
        ] {
            let mut changed = credentials.clone();
            changed[key] = serde_json::json!("foreign");
            assert!(!connection_matches(&options, &config, &changed));
        }
    }
    #[test]
    fn catalog_comparison_preserves_assertions_permissions_and_numeric_meaning() {
        let expected = parse_definitions("DEFINE TABLE canonical_probe SCHEMAFULL; DEFINE FIELD value ON canonical_probe TYPE int ASSERT $value > 1;").unwrap();
        let equivalent = parse_definitions("DEFINE TABLE OVERWRITE canonical_probe TYPE NORMAL SCHEMAFULL PERMISSIONS NONE; DEFINE FIELD OVERWRITE value ON canonical_probe TYPE int ASSERT $value > 1 PERMISSIONS FULL;").unwrap();
        compare(&expected, &equivalent).unwrap();
        for changed in [
            "DEFINE TABLE canonical_probe TYPE ANY SCHEMAFULL; DEFINE FIELD value ON canonical_probe TYPE int ASSERT $value > 1;",
            "DEFINE TABLE canonical_probe SCHEMALESS; DEFINE FIELD value ON canonical_probe TYPE int ASSERT $value > 1;",
            "DEFINE TABLE canonical_probe SCHEMAFULL; DEFINE FIELD value ON canonical_probe TYPE int ASSERT $value > 0;",
            "DEFINE TABLE canonical_probe SCHEMAFULL; DEFINE FIELD value ON canonical_probe TYPE int ASSERT $value > 1 PERMISSIONS NONE;",
        ] {
            assert!(compare(&expected, &parse_definitions(changed).unwrap()).is_err());
        }
    }
    #[test]
    fn control_api_catalog_comparison_retains_permissions_middleware_and_signin() {
        let endpoint = "DEFINE API '/pse/control/v1/probe' FOR post PERMISSIONS WHERE $auth!=NONE MIDDLEWARE api::req::max_body(65536),api::res::body('json') THEN {LET $body=fn::pse_control_v1::json($request); RETURN {status:200,body:$body};} SECURITY DEFINER;";
        let expected = parse_definitions(endpoint).unwrap();
        assert!(
            compare(
                &expected,
                &parse_definitions(
                    &endpoint.replace("PERMISSIONS WHERE $auth!=NONE", "PERMISSIONS FULL")
                )
                .unwrap()
            )
            .is_err()
        );
        compare(
            &expected,
            &parse_definitions(&endpoint.replace(" SECURITY DEFINER", "")).unwrap(),
        )
        .unwrap();
        assert!(
            compare(
                &expected,
                &parse_definitions(&endpoint.replace("SECURITY DEFINER", "SECURITY INVOKER"))
                    .unwrap()
            )
            .is_err(),
            "effective v4 SECURITY INVOKER drift must be refused"
        );
        assert!(
            compare(
                &expected,
                &parse_definitions(&endpoint.replace("65536", "131072")).unwrap()
            )
            .is_err()
        );
        assert!(
            compare(
                &expected,
                &parse_definitions(&endpoint.replace(",api::res::body('json')", "")).unwrap()
            )
            .is_err()
        );
        assert!(
            compare(
                &expected,
                &parse_definitions(
                    &endpoint.replace("fn::pse_control_v1::json($request)", "$request.body")
                )
                .unwrap()
            )
            .is_err()
        );
        let decoder = "DEFINE FUNCTION fn::pse_control_v1::json($request:object) -> object { IF $request.headers['content-type']!='application/json' OR !type::is_bytes($request.body) { THROW 'PSE_REFUSAL:control_request_invalid'; }; RETURN encoding::json::decode(<string>$request.body); } PERMISSIONS NONE;";
        assert!(
            compare(
                &parse_definitions(decoder).unwrap(),
                &parse_definitions(&decoder.replace(
                    "encoding::json::decode(<string>$request.body)",
                    "$request.body"
                ))
                .unwrap()
            )
            .is_err()
        );
        let access = "DEFINE ACCESS pse_control ON DATABASE TYPE RECORD SIGNIN (SELECT * FROM canonical_control_actors WHERE key=$actor AND enabled=true AND crypto::argon2::compare(password,$password)) DURATION FOR TOKEN 1h,FOR SESSION 1h;";
        let expected = parse_definitions(access).unwrap();
        compare(&expected, &parse_definitions(access).unwrap()).unwrap();
        let info = access.replace(
            " DURATION",
            " WITH JWT ALGORITHM HS512 KEY '[REDACTED]' WITH ISSUER KEY '[REDACTED]' DURATION",
        );
        compare(&expected, &parse_definitions(&info).unwrap()).unwrap();
        assert!(parse_definitions(&info.replace("HS512", "HS256")).is_err());
        assert!(parse_definitions(&info.replace("[REDACTED]", "caller-chosen-secret")).is_err());
        assert!(
            compare(
                &expected,
                &parse_definitions(&access.replace("enabled=true", "enabled=false")).unwrap()
            )
            .is_err()
        );
        assert!(!parse_definitions(wire::SCHEMA).unwrap().is_empty());
    }
    #[test]
    fn catalog_parser_refuses_data_effects_and_retains_function_semantics() {
        for sql in [
            "CREATE canonical_probe:one;",
            "UPDATE canonical_probe SET value=1;",
            "DELETE canonical_probe;",
            "REMOVE EVENT probe ON canonical_probe;",
            "DEFINE EVENT probe ON canonical_probe WHEN true THEN {}; CREATE canonical_probe:one;",
        ] {
            assert!(parse_definitions(sql).is_err());
        }
        let original = parse_definitions("DEFINE FUNCTION fn::pse_probe() { RETURN 1; };").unwrap();
        let changed = parse_definitions("DEFINE FUNCTION fn::pse_probe() { RETURN 2; };").unwrap();
        assert!(compare(&original, &changed).is_err());
        assert!(!parse_definitions(wire::SCHEMA).unwrap().is_empty());
    }

    #[test]
    fn catalog_event_comparison_retains_condition_body_scope_and_synchronous_mode() {
        let source = "DEFINE EVENT terminal_seed ON canonical_probe WHEN $event='UPDATE' AND $before.seed!=$after.seed THEN { THROW 'immutable'; };";
        let expected = parse_definitions(source).unwrap();
        for equivalent in [
            source.to_owned(),
            source.replace("DEFINE EVENT ", "DEFINE EVENT OVERWRITE "),
            source.replace("DEFINE EVENT ", "DEFINE EVENT IF NOT EXISTS "),
        ] {
            compare(&expected, &parse_definitions(&equivalent).unwrap()).unwrap();
        }
        for changed in [
            source.replace("$event='UPDATE'", "$event='CREATE'"),
            source.replace("THROW 'immutable'", "RETURN 'immutable'"),
            source.replace("ON canonical_probe", "ON canonical_other"),
            source.replace("terminal_seed", "other_seed"),
            source.replace(" WHEN ", " ASYNC RETRY 1 MAXDEPTH 2 WHEN "),
            source.replace("};", "} COMMENT 'different';"),
        ] {
            assert!(
                compare(&expected, &parse_definitions(&changed).unwrap()).is_err(),
                "event semantic drift must refuse: {changed}"
            );
        }
        assert!(compare(&expected, &Definitions::new()).is_err());
        let mut extra = expected.clone();
        extra.extend(parse_definitions(&source.replace("terminal_seed", "unowned_event")).unwrap());
        assert!(compare(&expected, &extra).is_err());
    }

    #[test]
    fn installed_event_catalog_is_required_and_exactly_parsed() {
        let sql = "DEFINE EVENT OVERWRITE terminal_seed ON canonical_probe WHEN $event='UPDATE' THEN { THROW 'immutable'; };";
        let mut entries = Object::new();
        entries.insert("terminal_seed", sql);
        let mut table = Object::new();
        table.insert("events", entries);
        table.insert("fields", Object::new());
        table.insert("indexes", Object::new());
        let mut observed = ManagedCatalog::default();
        observed.append_table("canonical_probe", &table).unwrap();
        compare(&parse_definitions(sql).unwrap(), &observed.definitions).unwrap();
        assert!(
            ManagedCatalog::default()
                .append_table("canonical_other", &table)
                .is_err()
        );
        assert!(observed.append_table("canonical_probe", &table).is_err());
        assert!(catalog_map(&Object::new(), "events").is_err());
        table.insert("events", true);
        assert!(catalog_map(&table, "events").is_err());
        assert!(
            ManagedCatalog::default()
                .append(
                    "event",
                    "canonical_probe",
                    &Value::String("CREATE canonical_probe:one;".into())
                )
                .is_err()
        );
    }

    #[tokio::test]
    async fn generated_maintenance_pages_fence_generations_and_replay_exact_effects() {
        use crate::canonical::RefusalCode;
        let store = crate::native_fixture::canonical_fixture_store().unwrap();
        let revision = store
            .edit("zz-maintenance-protected", None, "source", &[])
            .await
            .unwrap();
        let selection = store
            .protect(revision, std::time::Duration::from_secs(300))
            .await
            .unwrap();
        let guards = (0..65)
            .map(|ordinal| format!("retention:maintenance-{ordinal:03}"))
            .collect::<Vec<_>>();
        async fn call(store: &CanonicalStore, expression: &str) -> Result<Value, CanonicalError> {
            let mut response = bounded_query(store.db.script(
                GuardedScript::new(Envelope::DeadlineOnly).returning::<Value>("", expression)?,
            ))
            .await?;
            ResultSlot::<Value>::returned().take(&mut response)
        }
        let mut gate = bounded_query(
            store
                .db
                .query("RETURN canonical_file_admission:current.storage_incarnation;"),
        )
        .await
        .unwrap();
        let incarnation =
            crate::canonical_codec::decode_string(gate.take::<Value>(0).unwrap()).unwrap();
        let incarnation = serde_json::to_string(&incarnation).unwrap();
        let claim = |generation: &str, previous: &str| {
            format!(
                "fn::pse_maintenance_v1::claim(time::micros()+30000000,'rotation',\
            '{generation}','rebuilding',{incarnation},{incarnation},{previous})"
            )
        };
        assert!(matches!(
            call(&store, &claim("g1", "NONE")).await,
            Err(CanonicalError::Refused {
                code: RefusalCode::MaintenanceFrozenPlacementRequired,
                ..
            })
        ));
        bounded_query(store.db.script(GuardedScript::new(Envelope::DeadlineOnly).returning::<Value>(
            "UPDATE ONLY canonical_file_admission:current SET frozen=true,gc_frozen=true; \
            FOR $key IN $guards { \
                CREATE ONLY type::record('canonical_guards',$key) SET key=$key,generation=0dec,incarnation=string::concat('original:',$key); \
            };", "NONE"
        ).unwrap()).bind(("guards",guards.clone()))
        ).await.unwrap();
        let first_owner = call(&store, &claim("g1", "NONE")).await.unwrap();
        assert!(matches!(
            call(&store, &claim("g2", "NONE")).await,
            Err(CanonicalError::Refused {
                code: RefusalCode::MaintenanceGenerationChanged,
                ..
            })
        ));
        let page = |generation: &str, after: &str| {
            format!(
                "fn::pse_maintenance_v1::page(time::micros()+30000000,'rotation',\
            '{generation}','rebuilding','rotate',{})",
                serde_json::to_string(after).unwrap()
            )
        };
        assert!(matches!(
            call(&store, &page("g1", "")).await,
            Err(CanonicalError::Refused {
                code: RefusalCode::MaintenanceProtectedSource,
                ..
            })
        ));
        store.release(&selection).await.unwrap();
        let first_page = call(&store, &page("g1", "")).await.unwrap();
        let Value::Object(first_page_fields) = &first_page else {
            panic!("maintenance page must be an object")
        };
        let next =
            crate::canonical_codec::decode_string(first_page_fields.get("after").unwrap().clone())
                .unwrap();
        assert_eq!(next, guards[63]);
        assert_eq!(first_page_fields.get("complete"), Some(&Value::Bool(false)));
        let inventory = "RETURN SELECT key,generation,incarnation FROM canonical_guards \
            WHERE string::starts_with(key,'retention:maintenance-') ORDER BY key;";
        let mut response = bounded_query(store.db.query(inventory)).await.unwrap();
        let first_effect: Value = response.take(0).unwrap();
        assert_eq!(call(&store, &page("g1", "")).await.unwrap(), first_page);
        let previous = "{operation:'rotation',generation:'g1',phase:'rebuilding'}";
        call(&store, &claim("g2", previous)).await.unwrap();
        assert!(matches!(
            call(&store, &page("g1", "")).await,
            Err(CanonicalError::Refused {
                code: RefusalCode::MaintenanceGenerationChanged,
                ..
            })
        ));
        assert_eq!(call(&store, &page("g2", "")).await.unwrap(), first_page);
        assert!(matches!(
            call(&store, &page("g2", "unrelated-cursor")).await,
            Err(CanonicalError::Refused {
                code: RefusalCode::MaintenanceOriginalInputsDiffer,
                ..
            })
        ));
        let mut response = bounded_query(store.db.query(inventory)).await.unwrap();
        let replayed_effect: Value = response.take(0).unwrap();
        assert_eq!(
            replayed_effect, first_effect,
            "lost acknowledgements cannot rotate authority twice"
        );
        let Value::Array(rows) = first_effect else {
            panic!("guard inventory must be an array")
        };
        assert_eq!(rows.len(), 65);
        for row in rows {
            let Value::Object(mut row) = row else {
                panic!("guard row must be an object")
            };
            let key = crate::canonical_codec::decode_string(row.remove("key").unwrap()).unwrap();
            assert_eq!(
                crate::canonical_codec::decode_uint(row.remove("generation").unwrap()).unwrap(),
                u64::from(key != guards[64])
            );
            let incarnation =
                crate::canonical_codec::decode_string(row.remove("incarnation").unwrap()).unwrap();
            assert_eq!(incarnation == format!("original:{key}"), key == guards[64]);
        }
        // Resume the remaining page after a generation handoff. Replaying the
        // original first page above did not rotate any guard a second time.
        let tail = call(&store, &page("g2", &next)).await.unwrap();
        assert_eq!(call(&store, &page("g2", &next)).await.unwrap(), tail);
        let Value::Object(tail_fields) = tail else {
            panic!("maintenance tail must be an object")
        };
        let after =
            crate::canonical_codec::decode_string(tail_fields.get("after").unwrap().clone())
                .unwrap();
        let terminal = call(&store, &page("g2", &after)).await.unwrap();
        let Value::Object(terminal_fields) = &terminal else {
            panic!("maintenance completion must be an object")
        };
        assert_eq!(terminal_fields.get("complete"), Some(&Value::Bool(true)));
        assert_eq!(call(&store, &page("g2", &after)).await.unwrap(), terminal);
        let mut response = bounded_query(store.db.query(inventory)).await.unwrap();
        let Value::Array(rows) = response.take::<Value>(0).unwrap() else {
            panic!("guard inventory must be an array")
        };
        for row in rows {
            let Value::Object(mut row) = row else {
                panic!("guard row must be an object")
            };
            assert_eq!(
                crate::canonical_codec::decode_uint(row.remove("generation").unwrap()).unwrap(),
                1
            );
        }
        let Value::Object(mut expected) = first_owner else {
            panic!("maintenance owner must be an object")
        };
        expected.insert("generation", "g2");
        assert_eq!(
            Value::Object(expected),
            call(
                &store,
                "fn::pse_maintenance_v1::current(time::micros()+30000000)"
            )
            .await
            .unwrap()
        );
        store.remove_isolated_fixture().await.unwrap();
    }

    #[tokio::test]
    async fn native_maintenance_event_catalog_roundtrips_and_refuses_changed_or_extra_events() {
        let state = std::env::var("PSE_SURREAL_STATE").unwrap();
        let mut options = super::super::CanonicalOptions::from_state(Path::new(&state)).unwrap();
        options.database = format!(
            "canonical_test_event_catalog_{}",
            uuid::Uuid::new_v4().simple()
        );
        let store = crate::native_fixture::canonical_fixture_with_options(&options, true).unwrap();
        let target = CatalogTarget {
            endpoint: options.endpoint.clone(),
            namespace: options.namespace.clone(),
            database: options.database.clone(),
            issuing_store: uuid::Uuid::parse_str(store.issuing_store().unwrap()).unwrap(),
        };
        let foreign = CatalogTarget {
            issuing_store: uuid::Uuid::new_v4(),
            ..target.clone()
        };
        let refused = inspect_catalog(Path::new(&state), &foreign).await;
        assert!(
            matches!(&refused, Err(CanonicalError::Configuration(_))),
            "{refused:?}"
        );
        let installed = inspect_catalog(Path::new(&state), &target).await.unwrap();
        assert_eq!(installed.target, target);
        assert!(
            installed.comparison.missing.is_empty(),
            "{:?}",
            installed.comparison.missing
        );
        assert!(
            installed.comparison.changed.is_empty(),
            "{:?}",
            installed.comparison.changed
        );
        assert!(
            installed.comparison.unexpected.is_empty(),
            "{:?}",
            installed.comparison.unexpected
        );
        let expected: Definitions = parse_definitions(wire::SCHEMA)
            .unwrap()
            .into_iter()
            .filter(|(key, _)| key.0 == "event" && key.1 == "canonical_attempts")
            .collect();
        assert_eq!(
            expected.len(),
            2,
            "the terminal seal and exact capture witness each have one declaration"
        );
        async fn observed(store: &CanonicalStore) -> Definitions {
            let mut response = bounded_query(store.db.query("INFO FOR TABLE canonical_attempts;"))
                .await
                .unwrap();
            let Value::Object(info) = response.take::<Value>(0).unwrap() else {
                panic!("native table catalog must be an object")
            };
            let mut catalog = ManagedCatalog::default();
            catalog.append_table("canonical_attempts", &info).unwrap();
            catalog
                .definitions
                .into_iter()
                .filter(|(key, _)| key.0 == "event")
                .collect()
        }
        compare(&expected, &observed(&store).await).unwrap();
        let mut original = expected.values().next().unwrap().clone();
        let DefineStatement::Event(event) = &mut original else {
            panic!("registered event must retain its AST family")
        };
        let event_name = event.name.to_sql();
        event.kind = DefineKind::Overwrite;
        bounded_query(store.db.query(format!("DEFINE EVENT OVERWRITE {event_name} ON canonical_attempts WHEN false THEN {{ RETURN true; }};"))).await.unwrap();
        assert!(compare(&expected, &observed(&store).await).is_err());
        let changed = inspect_catalog(Path::new(&state), &target)
            .await
            .unwrap()
            .comparison;
        assert_eq!(changed.changed.len(), 1);
        assert_eq!(changed.changed[0].name, event_name);
        bounded_query(store.db.query(format!("{};", original.to_sql())))
            .await
            .unwrap();
        compare(&expected, &observed(&store).await).unwrap();
        bounded_query(store.db.query(
            "DEFINE EVENT unexpected_catalog_event ON canonical_attempts WHEN false THEN {}; ",
        ))
        .await
        .unwrap();
        assert!(compare(&expected, &observed(&store).await).is_err());
        let extra = inspect_catalog(Path::new(&state), &target)
            .await
            .unwrap()
            .comparison;
        assert_eq!(extra.unexpected.len(), 1);
        assert_eq!(extra.unexpected[0].name, "unexpected_catalog_event");
        store.remove_isolated_fixture().await.unwrap();
    }
}
