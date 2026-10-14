// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit maintenance validation and activation. Ordinary open never calls this.

use super::{
    ACTIVATION_REQUEST_TIMEOUT, CanonicalError, CanonicalStore, WriteAdmission, bounded_query,
    original_deadline, wire, within_clock,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};
use surrealdb::types::{Object, ToSql, Value};
use surrealdb_sql::{
    Expr,
    ast::TopLevelExpr,
    statements::{DefineStatement, define::DefineKind},
};

/// Exact predecessor, distinct from the generated target and beta schema hash.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Marker {
    /// Original interpretation.
    pub interpretation: String,
    /// Original repository schema plus codec digest.
    pub schema_digest: String,
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

fn parse_definitions(input: &str) -> Result<Definitions, CanonicalError> {
    let ast = surrealdb_syn::parse(input)
        .map_err(|error| invalid(format!("generated catalog parse: {error}")))?;
    let mut definitions = BTreeMap::new();
    for expression in ast.expressions {
        let TopLevelExpr::Expr(Expr::Define(statement)) = expression else {
            return Err(invalid(
                "only generated schema declarations may be validated",
            ));
        };
        let mut statement = *statement;
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

fn catalog_map(info: &Object, category: &str) -> Result<Object, CanonicalError> {
    match info.get(category) {
        Some(Value::Object(values)) => Ok(values.clone()),
        _ => Err(invalid(format!(
            "catalog {category} is absent or malformed"
        ))),
    }
}

fn append_catalog(definitions: &mut Definitions, values: &Object) -> Result<(), CanonicalError> {
    for (_, value) in values.iter() {
        let Value::String(sql) = value else {
            return Err(invalid("catalog definition is not a string"));
        };
        let parsed = parse_definitions(sql.as_str())?;
        if parsed.len() != 1 {
            return Err(invalid("catalog entry must contain one declaration"));
        }
        for (key, definition) in parsed {
            if definitions.insert(key, definition).is_some() {
                return Err(invalid("duplicate installed declaration"));
            }
        }
    }
    Ok(())
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
    fn maintenance_config(&self) -> Result<serde_json::Value, CanonicalError> {
        let admission = WriteAdmission::read(&self.state_path)?;
        admission.require_initializer(
            &self.state_path,
            &self.namespace,
            &self.database,
            self.connection_options.initializer_nonce.as_deref(),
        )?;
        let config: serde_json::Value = serde_json::from_slice(
            &std::fs::read(self.state_path.join("config.json"))
                .map_err(|error| invalid(error.to_string()))?,
        )
        .map_err(|error| invalid(error.to_string()))?;
        let listener: serde_json::Value = serde_json::from_slice(
            &std::fs::read(self.state_path.join(".maintenance-listener.json"))
                .map_err(|_| CanonicalError::Quiesced)?,
        )
        .map_err(|_| CanonicalError::Quiesced)?;
        let owner: serde_json::Value = serde_json::from_slice(
            &std::fs::read(self.state_path.join("lifecycle-owner.json"))
                .map_err(|_| CanonicalError::Quiesced)?,
        )
        .map_err(|_| CanonicalError::Quiesced)?;
        if listener.get("owner") != Some(&owner)
            || listener.get("instance_id") != config.get("instance_id")
            || listener.get("supervisor") != config.get("service_supervisor")
            || listener.get("server") != config.get("server")
            || listener.get("resources") != config.get("resources")
        {
            return Err(CanonicalError::Quiesced);
        }
        let credentials = std::fs::read(self.state_path.join("credentials.json"))
            .map_err(|_| CanonicalError::Quiesced)?;
        let credential_document: serde_json::Value =
            serde_json::from_slice(&credentials).map_err(|_| CanonicalError::Quiesced)?;
        if !connection_matches(&self.connection_options, &config, &credential_document) {
            return Err(CanonicalError::Quiesced);
        }
        let digest = sha256(&credentials);
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
                    let mut expected = BTreeMap::new();
                    for input in inputs {
                        expected.extend(parse_definitions(input)?);
                    }
                    let info = self
                        .database_inventory()
                        .await?
                        .ok_or_else(|| invalid("selected database absent"))?;
                    let tables = catalog_map(&info, "tables")?;
                    let functions = catalog_map(&info, "functions")?;
                    let mut observed = BTreeMap::new();
                    let managed_tables: Object = tables
                        .iter()
                        .filter(|(name, _)| name.starts_with("canonical_"))
                        .map(|(name, value)| (name.clone(), value.clone()))
                        .collect();
                    append_catalog(&mut observed, &managed_tables)?;
                    let managed_functions: Object = functions
                        .iter()
                        .filter(|(name, _)| name.trim_start_matches("fn::").starts_with("pse_"))
                        .map(|(name, value)| (name.clone(), value.clone()))
                        .collect();
                    append_catalog(&mut observed, &managed_functions)?;
                    for (name, _) in managed_tables.iter() {
                        if !name
                            .bytes()
                            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
                        {
                            return Err(invalid("managed table identifier differs"));
                        }
                        self.maintenance_config()?;
                        let mut response =
                            bounded_query(self.db.query(format!("INFO FOR TABLE {name};"))).await?;
                        let Value::Object(table_info) = response.take::<Value>(0)? else {
                            return Err(CanonicalError::IncompleteResponse);
                        };
                        append_catalog(&mut observed, &catalog_map(&table_info, "fields")?)?;
                        append_catalog(&mut observed, &catalog_map(&table_info, "indexes")?)?;
                        for category in ["events", "lives"] {
                            if let Some(Value::Object(entries)) = table_info.get(category) {
                                if !entries.is_empty() {
                                    return Err(invalid(format!(
                                        "unowned {category} on managed table {name}"
                                    )));
                                }
                            }
                        }
                    }
                    compare(&expected, &observed)?;
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
                if nonce.len() != 32 || !nonce.bytes().all(|byte| byte.is_ascii_hexdigit()) || settlement_path != self.state_path.join(".surrealkit").join(nonce).join("settled.json") || settlement_path.symlink_metadata().map_err(|error| invalid(error.to_string()))?.file_type().is_symlink() { return Err(invalid("exact owned settlement path required")); }
                let settled: serde_json::Value = serde_json::from_slice(&std::fs::read(settlement_path).map_err(|error| invalid(error.to_string()))?).map_err(|error| invalid(error.to_string()))?;
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
                if recorded.scope != "generated-target" || recorded.definition_inputs != witness.definition_inputs || recorded.interpretation != witness.interpretation || recorded.schema_digest != witness.schema_digest || recorded.namespace != witness.namespace || recorded.database != witness.database || recorded.instance_id != witness.instance_id || recorded.catalog_definition_count != witness.catalog_definition_count { return Err(invalid("settled target differs from actual installed target")); }
                let predecessor: Option<Marker> = serde_json::from_value(pending.get("previous_marker").cloned().ok_or_else(|| invalid("original predecessor absent"))?).map_err(|error| invalid(error.to_string()))?;
                let predecessor_bytes = serde_json::to_vec(&predecessor).map_err(|error| invalid(error.to_string()))?;
                let input_sha256 = pending.get("input_sha256").and_then(serde_json::Value::as_str).ok_or_else(|| invalid("pending input identity absent"))?;
                let mut existing = bounded_query(self.db.query("SELECT * FROM ONLY type::record('canonical_maintenance_activations',$nonce);").bind(("nonce",nonce.to_owned()))).await?;
                if let Some(mut receipt) = existing.take::<Option<Object>>(0)? {
                    receipt.remove("id");
                    let field = |receipt: &mut Object, name| crate::canonical_codec::required(receipt,name);
                    let issuing = crate::canonical_codec::decode_string(field(&mut receipt,"issuing_store")?)?;
                    if crate::canonical_codec::decode_string(field(&mut receipt,"key")?)? != nonce
                        || crate::canonical_codec::decode_string(field(&mut receipt,"input_sha256")?)? != input_sha256
                        || crate::canonical_codec::decode_bytes(field(&mut receipt,"predecessor")?)?.as_slice() != predecessor_bytes.as_slice()
                        || crate::canonical_codec::decode_string(field(&mut receipt,"interpretation")?)? != wire::INTERPRETATION
                        || crate::canonical_codec::decode_string(field(&mut receipt,"schema_digest")?)? != wire::SCHEMA_DIGEST
                        || !receipt.is_empty()
                        || witness.previous_marker != Some(Marker { interpretation: wire::INTERPRETATION.into(), schema_digest: wire::SCHEMA_DIGEST.into(), issuing_store: issuing }) {
                        return Err(invalid("immutable activation receipt differs"));
                    }
                    self.maintenance_config()?;
                    self.open().await?;
                    return Ok(witness);
                }
                if witness.previous_marker != predecessor { return Err(invalid("activation predecessor changed")); }
                if predecessor.as_ref().is_some_and(|marker| marker.interpretation != wire::INTERPRETATION || uuid::Uuid::parse_str(&marker.issuing_store).is_err()) { return Err(invalid("incompatible predecessor requires explicit fresh reset")); }
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
                let query = "BEGIN; fn::pse_execution_v1::deadline($pse_rpc_expires_at); LET $old=SELECT * FROM ONLY canonical_interpretations:current FOR UPDATE; LET $receipt=SELECT * FROM ONLY type::record('canonical_maintenance_activations',$nonce) FOR UPDATE; IF $receipt!=NONE { THROW 'activation operation already exists; settle exact receipt'; }; IF ($predecessor=NONE AND $old!=NONE) OR ($predecessor!=NONE AND ($old.interpretation!=$predecessor.interpretation OR $old.schema_digest!=$predecessor.schema_digest OR $old.issuing_store!=$predecessor.issuing_store)) { THROW 'activation predecessor changed'; }; IF $old!=NONE AND $old.payload_incarnation!=$payload_incarnation { THROW 'activation payload incarnation differs'; }; LET $gate=SELECT * FROM ONLY canonical_file_admission:current FOR UPDATE; IF $gate!=NONE AND $gate.storage_incarnation!=$payload_incarnation { THROW 'activation file placement differs'; }; IF $gate=NONE { CREATE canonical_file_admission:current SET key='current',storage_incarnation=$payload_incarnation,frozen=false,gc_frozen=false; }; UPSERT canonical_interpretations:current SET key='current', interpretation=$interpretation, schema_digest=$schema_digest, issuing_store=$issuing,payload_incarnation=$payload_incarnation; CREATE ONLY type::record('canonical_maintenance_activations',$nonce) SET key=$nonce,input_sha256=$input_sha256,predecessor=$predecessor_bytes,interpretation=$interpretation,schema_digest=$schema_digest,issuing_store=$issuing; fn::pse_execution_v1::deadline($pse_rpc_expires_at); COMMIT;";
                let predecessor_value = match predecessor { Some(marker) => { let mut value=Object::new(); value.insert("interpretation", marker.interpretation); value.insert("schema_digest", marker.schema_digest); value.insert("issuing_store", marker.issuing_store); Value::Object(value) }, None => Value::None };
                let response = bounded_query(self.activation_db.query(query).bind(("predecessor",predecessor_value)).bind(("interpretation",wire::INTERPRETATION)).bind(("schema_digest",wire::SCHEMA_DIGEST)).bind(("issuing",issuing)).bind(("payload_incarnation",self.files.incarnation.clone())).bind(("nonce",nonce.to_owned())).bind(("input_sha256",input_sha256.to_owned())).bind(("predecessor_bytes",surrealdb::types::Bytes::from(predecessor_bytes)))).await?;
                super::complete_response(response)?;
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
    fn maintenance_authority_binds_original_connection_and_credentials() {
        let options = super::super::CanonicalOptions {
            endpoint: "ws://127.0.0.1:12345".into(),
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
    fn catalog_parser_refuses_data_effects_and_retains_function_semantics() {
        assert!(parse_definitions("CREATE canonical_probe:one;").is_err());
        let original = parse_definitions("DEFINE FUNCTION fn::pse_probe() { RETURN 1; };").unwrap();
        let changed = parse_definitions("DEFINE FUNCTION fn::pse_probe() { RETURN 2; };").unwrap();
        assert!(compare(&original, &changed).is_err());
        assert!(!parse_definitions(wire::SCHEMA).unwrap().is_empty());
    }
}
