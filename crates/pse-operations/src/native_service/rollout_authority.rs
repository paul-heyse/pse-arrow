// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The pinned standalone tool against an authored v4 predecessor on an owned listener.

use super::*;
use crate::{
    canonical::maintenance::{FrozenRollout, RolloutPhase},
    transport::{
        bounded_query,
        client::{Credentials, Database, IndexedResults},
        operation_scope,
    },
};
use pse_diagnostics::{DrainFailurePolicy, StderrCapture, SupervisedChild};
use std::{os::unix::process::ExitStatusExt as _, process::ExitStatus};
use surrealdb_types::Object;

const NS: &str = "rollout_control";
const DB: &str = "target";
const SIBLING: &str = "sibling";
const OWNER: &str = "rollout_owner";
const VIEWER: &str = "rollout_viewer";
const ROLLOUT: &str = "20261108000000__authority_fixture";
const ALLOWANCE: Duration = Duration::from_secs(20);
const EXPAND: &str = include_str!("../../../../scripts/tests/fixtures/surrealkit/rollout-schema.surql");
const MANIFEST: &str = r#"
id = "20261108000000__authority_fixture"
name = "authority_fixture"
source_schema_hash = "ad301433f67ec7108b54797ac5d7a61df8c92f62fa52ae631da3f49731576ac3"
target_schema_hash = "35b61970be4274e0b076742815fdbdc94b21924583ce19874c9d1def95afcafa"
compatibility = "phased"
renames = []

[[steps]]
id = "apply_expand_schema"
phase = "start"
kind = "apply_files"

[[steps.files]]
path = "schema/canonical.surql"
hash = "1efe932eeb7b8b327cf7fe0bed23c89eda32cc89ece8a2964a8f146905cc987f"

[[steps]]
id = "contract_fixture_old_field"
phase = "complete"
kind = "remove_entities"

[[steps.entities]]
kind = "field"
scope = "fixture_marker"
name = "obsolete"
"#;

const OUTPUT: StderrCapture = StderrCapture {
    bytes: 8 * 1024,
    stack_bytes: 64 * 1024,
};

fn stage(project: &Path) -> (PathBuf, PathBuf) {
    let rollout = project.join("rollouts").join(ROLLOUT);
    let manifest = rollout.with_extension("toml");
    let sql = rollout.join("schema/canonical.surql");
    for directory in [
        project.join("schema"),
        rollout.join("schema"),
        rollout.join("snapshots"),
    ] {
        fs::create_dir_all(directory).unwrap();
    }
    fs::write(&manifest, MANIFEST).unwrap();
    fs::write(&sql, EXPAND).unwrap();
    fs::write(project.join("schema/canonical.surql"), EXPAND).unwrap();
    fs::write(project.join("setup.surql"), "").unwrap();
    fs::write(
        rollout.join("snapshots/schema_snapshot.json"),
        r#"{"version":1,"files":[]}"#,
    )
    .unwrap();
    fs::write(
        rollout.join("snapshots/catalog_snapshot.json"),
        r#"{"version":2,"entities":[]}"#,
    )
    .unwrap();
    (manifest, sql)
}

fn launch(
    server: &PreparedServer,
    scratch: &Path,
    address: SocketAddr,
    root: &Root,
) -> SupervisedChild {
    let mut command = Command::new(server.executable());
    command
        .args([
            "start", "--no-banner", "--no-defaults", "--deny-guests", "--deny-net", "--log",
            "warn", "--query-timeout=8s", "--transaction-timeout=8s", "--bind",
        ])
        .arg(address.to_string())
        .arg(format!(
            "mem://{}?aol=sync&sync=every&versioned=false&snapshot=60s",
            scratch.join("database").display()
        ))
        .env_clear()
        .env("HOME", scratch)
        .env("TMPDIR", scratch)
        .env("SURREAL_USER", &root.username)
        .env("SURREAL_PASS", &root.password)
        .stdin(Stdio::null())
        .stdout(Stdio::null());
    let mut child = SupervisedChild::spawn(&mut command, DrainFailurePolicy::Park, Some(OUTPUT))
        .expect("launch source-selected fixture server");
    let deadline = Instant::now() + ALLOWANCE;
    while Instant::now() < deadline && child.try_wait().unwrap().is_none() {
        if owns_listener(child.pid(), address).unwrap_or(false) {
            return child;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("original server did not own the fixture listener");
}

fn stop(mut child: SupervisedChild) {
    let pid = rustix::process::Pid::from_raw(child.pid().cast_signed()).unwrap();
    let fd = rustix::process::pidfd_open(pid, rustix::process::PidfdFlags::empty()).unwrap();
    rustix::process::pidfd_send_signal(fd, rustix::process::Signal::TERM).unwrap();
    child.wait_ready(None, Some(Instant::now() + ALLOWANCE)).unwrap();
    assert!(child.wait().unwrap().success(), "server must acknowledge graceful stop");
}

async fn query(
    address: SocketAddr,
    credentials: Credentials,
    sql: &str,
) -> Result<IndexedResults, CanonicalError> {
    let deadline = Instant::now() + ALLOWANCE;
    operation_scope(deadline, async {
        let selected = match &credentials {
            Credentials::Database(account) => Some((account.namespace.clone(), account.database.clone())),
            Credentials::Root(_) => None,
        };
        let client = Client::connect_pinned(address, MESSAGE_BYTES).await?;
        let result = async {
            client.signin(credentials).request_context(RequestContext::control(deadline)).await?;
            if let Some((namespace, database)) = selected {
                client.use_ns(&namespace).use_db(&database)
                    .request_context(RequestContext::control(deadline)).await?;
            }
            bounded_query(client.query(sql).bind(("rollout_id", ROLLOUT)).request_context(RequestContext::control(deadline))).await
        }.await;
        let drained = client.disconnect().await;
        let result = result?;
        drained?;
        Ok(result)
    }).await
}

fn spawn_tool(
    binary: &Path,
    project: &Path,
    address: SocketAddr,
    credentials: &Database,
    args: &[&str],
) -> SupervisedChild {
    let mut command = Command::new(binary);
    command
        .env_clear()
        .env("HOME", project)
        .current_dir(project)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .arg("rollout")
        .args(args)
        .args(["--host", &format!("grpc://{address}"), "--ns", &credentials.namespace,
            "--db", &credentials.database, "--auth-level", "database",
            "--connect-timeout-secs", "5", "--query-timeout-secs", "8", "--json",
            "--folder"])
        .arg(project)
        .env("SURREALDB_USER", &credentials.username)
        .env("SURREALDB_PASSWORD", &credentials.password);
    SupervisedChild::spawn(&mut command, DrainFailurePolicy::Park, Some(OUTPUT))
        .expect("launch verified tool")
}

fn tool(
    binary: &Path,
    project: &Path,
    address: SocketAddr,
    credentials: &Database,
    args: &[&str],
) -> ExitStatus {
    let mut child = spawn_tool(binary, project, address, credentials, args);
    child.wait_ready(None, Some(Instant::now() + ALLOWANCE)).unwrap();
    let status = child.wait().unwrap();
    assert!(child.take_stderr_prefix().unwrap().len() <= OUTPUT.bytes);
    status
}

#[derive(Debug, PartialEq, Eq)]
struct Observation {
    catalog: Object,
    fields: Object,
    marker: Object,
    journal: Option<Object>,
}

async fn observe(address: SocketAddr, viewer: &Database) -> Observation {
    let mut response = query(
        address,
        viewer.clone().into(),
        "INFO FOR DB; INFO FOR TABLE fixture_marker; SELECT * FROM ONLY fixture_marker:current; \
         RETURN IF '__rollout' IN object::keys((INFO FOR DB).tables) THEN \
             (SELECT id,status,manifest_checksum,source_schema_hash,target_schema_hash,steps \
              FROM ONLY type::record('__rollout',$rollout_id)) \
         ELSE NONE END;",
    )
    .await
    .expect("independent scoped readback");
    let catalog = response.take::<Option<Object>>(0).unwrap().unwrap();
    assert_eq!(catalog.get("compatibility"), Some(&Value::Number(Number::Int(4))));
    Observation {
        catalog,
        fields: response.take::<Option<Object>>(1).unwrap().unwrap(),
        marker: response.take::<Option<Object>>(2).unwrap().unwrap(),
        journal: response.take(3).unwrap(),
    }
}

impl Observation {
    fn has_table(&self, name: &str) -> bool {
        matches!(self.catalog.get("tables"), Some(Value::Object(tables)) if tables.contains_key(name))
    }

    fn has_obsolete_field(&self) -> bool {
        matches!(self.fields.get("fields"), Some(Value::Object(fields)) if fields.contains_key("obsolete"))
    }
}

async fn provision_database(
    address: SocketAddr,
    root: &Root,
    owner: &Database,
    viewer: &Database,
    marker: &str,
) {
    let provision = format!(
        "USE NS {NS}; USE DB {}; DEFINE USER {OWNER} ON DATABASE PASSWORD '{}' ROLES OWNER; \
         DEFINE USER {VIEWER} ON DATABASE PASSWORD '{}' ROLES VIEWER; \
         DEFINE TABLE fixture_marker SCHEMAFULL; DEFINE FIELD nonce ON fixture_marker TYPE string; \
         DEFINE FIELD obsolete ON fixture_marker TYPE option<string>; \
         CREATE fixture_marker:current SET nonce='{marker}',obsolete='old value';",
        owner.database, owner.password, viewer.password
    );
    // Database errors can include credential-bearing SQL. Classify without rendering it.
    assert!(query(address, root.clone().into(), &provision).await.is_ok(), "private predecessor setup");
}

#[tokio::test]
#[ignore = "requires the explicitly prepared exact SurrealKit release and v4 server"]
async fn frozen_rollout_matches_independent_v4_readback_after_drain() {
    let environment = std::env::vars_os().collect();
    let server = server::select(&environment).unwrap();
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap();
    let kit = pse_host::surrealkit::verify(&checkout, &environment).unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let project = scratch.path().join("project");
    let (manifest, sql) = stage(&project);
    let frozen = FrozenRollout::read(&manifest).unwrap();
    assert!(frozen.requires_independent_data_witness());
    let bind = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    let address = bind.local_addr().unwrap();
    drop(bind);
    let root = Root { username: "fixture_root".into(), password: secret() };
    let owner = Database {
        namespace: NS.into(),
        database: DB.into(),
        username: OWNER.into(),
        password: secret(),
    };
    let viewer = Database { username: VIEWER.into(), password: secret(), ..owner.clone() };
    let listener = launch(&server, scratch.path(), address, &root);
    let marker = uuid::Uuid::new_v4().simple().to_string();
    query(address, root.clone().into(), &format!(
        "DEFINE NAMESPACE {NS}; USE NS {NS}; DEFINE DATABASE {DB}; DEFINE DATABASE {SIBLING};"
    )).await.unwrap();
    provision_database(address, &root, &owner, &viewer, &marker).await;
    let predecessor = observe(address, &viewer).await;
    assert!(!predecessor.has_table("__rollout"));
    assert!(predecessor.has_obsolete_field());
    assert_eq!(predecessor.marker.get("nonce"), Some(&Value::String(marker.clone())));

    assert!(tool(&kit, &project, address, &owner, &["lint"]).success());
    fs::write(&sql, format!("{EXPAND}\n-- changed bytes\n")).unwrap();
    assert!(FrozenRollout::read(&manifest).is_err());
    assert_eq!(observe(address, &viewer).await, predecessor);
    fs::write(&sql, EXPAND).unwrap();
    assert_eq!(FrozenRollout::read(&manifest).unwrap(), frozen);

    let foreign = Database { database: SIBLING.into(), ..owner.clone() };
    assert!(!tool(&kit, &project, address, &foreign, &["start", ROLLOUT]).success());
    assert_eq!(observe(address, &viewer).await, predecessor);

    assert!(tool(&kit, &project, address, &owner, &["start", ROLLOUT]).success());
    let started = observe(address, &viewer).await;
    assert!(started.has_table("canonical_interpretations"));
    assert!(started.has_obsolete_field());
    assert_eq!(started.marker, predecessor.marker);
    assert!(frozen.journal_records_phase(started.journal.as_ref().unwrap(), RolloutPhase::Start));
    assert!(!frozen.journal_records_phase(started.journal.as_ref().unwrap(), RolloutPhase::Complete));
    assert_eq!(FrozenRollout::read(&manifest).unwrap(), frozen);

    assert!(tool(&kit, &project, address, &owner, &["complete", ROLLOUT]).success());
    stop(listener);
    let reader = launch(&server, scratch.path(), address, &root);
    let completed = observe(address, &viewer).await;

    assert!(completed.has_table("canonical_interpretations"));
    assert!(!completed.has_obsolete_field());
    assert_eq!(completed.marker.get("nonce"), Some(&Value::String(marker)));
    assert!(!completed.marker.contains_key("obsolete"));
    let journal = completed.journal.unwrap();
    assert!(frozen.journal_records_phase(&journal, RolloutPhase::Complete));
    let mut no_steps = journal;
    no_steps.insert("steps", Value::Array(Default::default()));
    assert!(!frozen.journal_records_phase(&no_steps, RolloutPhase::Complete));
    assert_eq!(FrozenRollout::read(&manifest).unwrap(), frozen);

    let sibling_marker = uuid::Uuid::new_v4().simple().to_string();
    let sibling_owner = Database { password: secret(), ..foreign };
    let sibling_viewer = Database { database: SIBLING.into(), password: secret(), ..viewer };
    provision_database(address, &root, &sibling_owner, &sibling_viewer, &sibling_marker).await;
    assert!(tool(&kit, &project, address, &sibling_owner, &["start", ROLLOUT]).success());
    query(
        address,
        root.clone().into(),
        &format!(
            "USE NS {NS}; USE DB {SIBLING}; \
             DEFINE EVENT delay_journal ON TABLE __rollout WHEN $event='UPDATE' \
             AND $after.status='running_complete' \
             AND $after.steps[1].step_id='contract_fixture_old_field' \
             AND $after.steps[1].status='completed' THEN {{ RETURN sleep(20s); }};"
        ),
    )
    .await
    .unwrap();
    let mut child = spawn_tool(&kit, &project, address, &sibling_owner, &["complete", ROLLOUT]);
    // Observe the actual DDL commit before killing the child. Its separate
    // journal write remains inside the delayed event, beyond the query clock.
    let deadline = Instant::now() + Duration::from_secs(5);
    let observed = loop {
        assert!(Instant::now() < deadline, "fixture never reached the journal gap");
        assert!(child.try_wait().unwrap().is_none(), "tool left the journal gap early");
        let current = observe(address, &sibling_viewer).await;
        if !current.has_obsolete_field() {
            break current;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    };
    assert!(child.cancel().unwrap().signal().is_some());
    stop(reader);
    let reader = launch(&server, scratch.path(), address, &root);
    let unknown = observe(address, &sibling_viewer).await;
    stop(reader);
    assert_eq!(unknown, observed);
    assert_eq!(unknown.marker.get("nonce"), Some(&Value::String(sibling_marker)));
    assert!(!unknown.marker.contains_key("obsolete"));
    let journal = unknown.journal.unwrap();
    assert_eq!(journal.get("status"), Some(&Value::String("running_complete".into())));
    assert!(matches!(journal.get("steps"), Some(Value::Array(steps))
        if matches!(steps.as_slice(), [_, Value::Object(step)]
            if step.get("status") == Some(&Value::String("running".into())))));
    assert!(!frozen.journal_records_phase(&journal, RolloutPhase::Complete));
}
