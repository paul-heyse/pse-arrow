//! G3-P1 probe: vendored SDK (bounded gRPC profile) vs direct generated client.
//! Usage: probe <port> <password-file>

mod direct;

use std::fmt::Debug;
use std::net::SocketAddr;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use surrealdb::Surreal;
use surrealdb::engine::remote::grpc::{Client, Grpc};
use surrealdb::opt::auth::Root;
use surrealdb::opt::{Config, GrpcConfig, RequestContext};
use surrealdb::types::{Error, Object, QueryError, Value};

const NS: &str = "probe";
static DB_NAME: std::sync::OnceLock<String> = std::sync::OnceLock::new();
fn db_name() -> &'static str { DB_NAME.get_or_init(|| format!("probe_{}", uuid::Uuid::new_v4().simple())) }

/// Copied verbatim from pse-operations `canonical::PROTECTED_BEGIN`.
const PROTECTED_BEGIN: &str = r#"BEGIN;
SELECT * FROM type::record('canonical_guards', 'retention:' + $problem) FOR UPDATE; fn::pse_execution_v1::deadline($pse_rpc_expires_at);
LET $pin = SELECT * FROM ONLY type::record('canonical_protections', $protection) FOR UPDATE;
IF $pin = NONE OR $pin.problem != $problem OR $pin.revision != $revision OR $pin.sequence != $sequence OR $pin.released OR ($pin.expires_at <= time::micros() AND !fn::pse_files_v1::ticket_open($pse_rpc_expires_at,$protection)) { THROW 'PSE_REFUSAL:immutable_selection_protection_expired'; };
LET $grant=fn::pse_execution_v1::protection_grant($pse_rpc_expires_at,$problem,$protection,false);
IF $pin.grant_deadline!=$grant.grant_deadline OR $pin.grant_nonce!=$grant.grant_nonce OR $pin.authority_incarnation!=$grant.authority_incarnation { THROW 'PSE_REFUSAL:immutable_selection_grant_differs'; };
LET $result_pin=SELECT * FROM ONLY type::record('canonical_result_protections',$protection) FOR UPDATE;
IF $result_pin!=NONE {
    LET $retirement=SELECT * FROM ONLY type::record('canonical_result_retirements',$result_pin.run) FOR UPDATE;
    LET $selected_manifest=SELECT * FROM ONLY type::record('canonical_result_manifests',$result_pin.manifest) FOR UPDATE;
    IF $retirement!=NONE OR $selected_manifest=NONE OR $selected_manifest.attempt!=$result_pin.attempt { THROW 'PSE_REFUSAL:result_protection_retired_or_manifest_unavailable'; };
};"#;

/// Fixture schema: the three functions PROTECTED_BEGIN calls, copied verbatim
/// from the generated module, and one tiny function for the single-call shape.
const FIXTURE_DDL: &str = r#"
DEFINE TABLE canonical_guards; DEFINE TABLE canonical_protections; DEFINE TABLE canonical_result_sets; DEFINE TABLE canonical_memberships;
DEFINE TABLE canonical_manifest_tickets; DEFINE TABLE canonical_file_owners; DEFINE TABLE canonical_file_admission;
DEFINE TABLE canonical_result_protections; DEFINE TABLE canonical_result_retirements; DEFINE TABLE canonical_result_manifests; DEFINE TABLE conflict_cells;
DEFINE FUNCTION fn::pse_execution_v1::deadline($rpc_expiry: int) -> bool {
    IF time::micros() >= $rpc_expiry { THROW 'PSE_REFUSAL:original_canonical_rpc_deadline_expired'; };
    RETURN true;
};
DEFINE FUNCTION fn::pse_execution_v1::protection_grant($rpc_expiry:int,$problem:string,$key:string,$acquiring:bool) -> object {
    LET $parts=string::split($key,':');
    IF array::len($parts)!=4 OR $parts[0]!='pse.protection.v2' OR string::len($parts[2])!=36 { THROW 'PSE_REFUSAL:protection_immutable_grant_binding'; };
    LET $authority=SELECT * FROM ONLY type::record('canonical_guards','retention:'+$problem) FOR UPDATE;
    LET $deadline=<int>$parts[3];
    IF $authority=NONE OR $authority.incarnation!=$parts[1] OR $deadline<=0 { THROW 'PSE_REFUSAL:protection_authority_incarnation_differs'; };
    IF $acquiring AND ($deadline<=time::micros() OR $deadline<=($authority.protection_closed_through ?? 0dec)) { THROW 'PSE_REFUSAL:protection_acquisition_window_closed'; };
    SELECT * FROM ONLY type::record('canonical_protections',$key) FOR UPDATE;
    fn::pse_execution_v1::deadline($rpc_expiry);
    RETURN {grant_deadline:$deadline,grant_nonce:$parts[2],authority_incarnation:$parts[1],closed:$deadline<=($authority.protection_closed_through ?? 0dec)};
};
DEFINE FUNCTION fn::pse_files_v1::ticket_open($expiry:int,$key:string) -> bool {
    LET $ticket=SELECT * FROM ONLY type::record('canonical_manifest_tickets',$key) FOR UPDATE;
    IF $ticket=NONE OR $ticket.released { RETURN false; };
    LET $owner=SELECT * FROM ONLY type::record('canonical_file_owners',$ticket.owner) FOR UPDATE;
    LET $gate=SELECT * FROM ONLY canonical_file_admission:current FOR UPDATE;
    IF $owner=NONE OR $gate=NONE { THROW 'PSE_REFUSAL:read_ticket_actual_process_owner_unavailable'; };
    IF $owner.drained { RETURN false; };
    IF $ticket.storage_incarnation!=$gate.storage_incarnation OR $owner.storage_incarnation!=$gate.storage_incarnation { THROW 'PSE_REFUSAL:read_ticket_actual_process_incarnation_unavailable'; };
    fn::pse_execution_v1::deadline($expiry); RETURN true;
};
DEFINE FUNCTION fn::probe_v1::echo($x: any) -> object { RETURN {echo:$x, n:1dec}; };
CREATE type::record('canonical_guards','retention:P1') SET key='retention:P1', incarnation='inc1', protection_closed_through=0dec;
CREATE type::record('canonical_protections',$live) SET problem='P1', revision='R1', sequence=7dec, released=false, expires_at=9999999999999999, grant_deadline=9999999999999999, grant_nonce='00000000-0000-4000-8000-000000000001', authority_incarnation='inc1';
CREATE type::record('canonical_protections',$expired) SET problem='P1', revision='R1', sequence=7dec, released=false, expires_at=1, grant_deadline=9999999999999999, grant_nonce='00000000-0000-4000-8000-000000000002', authority_incarnation='inc1';
CREATE type::record('canonical_result_sets','S1') SET key='S1', run='run-1', rows=3dec, digest=<bytes>'abc';
CREATE canonical_memberships:m1 SET problem='P1', name='alpha', value=1.5f;
CREATE canonical_memberships:m2 SET problem='P1', name='beta', value=2.5f;
CREATE conflict_cells:k SET v=0dec;
"#;

const LIVE: &str = "pse.protection.v2:inc1:00000000-0000-4000-8000-000000000001:9999999999999999";
const EXPIRED: &str = "pse.protection.v2:inc1:00000000-0000-4000-8000-000000000002:9999999999999999";

fn now_micros() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros() as i64
}

fn vars(protection: &str, expires_at: i64) -> Vec<(&'static str, Value)> {
    vec![
        ("pse_rpc_expires_at", Value::from_t(expires_at)),
        ("pse_rpc_timeout", Value::from_t(surrealdb::types::Duration::from_std(Duration::from_secs(30)))),
        ("problem", Value::from_t("P1".to_string())),
        ("protection", Value::from_t(protection.to_string())),
        ("revision", Value::from_t("R1".to_string())),
        ("sequence", Value::from_t(surrealdb::types::Number::Decimal(7.into()))),
        ("set", Value::from_t("S1".to_string())),
        ("x", Value::from_t("hello".to_string())),
    ]
}


#[derive(Debug, Clone, Copy)]
enum Shape {
    OptObject,
    VecObject,
    Value,
}

#[derive(Debug, PartialEq)]
enum Taken {
    OptObject(Result<Option<Object>, Error>),
    VecObject(Result<Vec<Object>, Error>),
    Value(Result<Value, Error>),
}

/// Observation of one script under one take shape.
#[derive(Debug, PartialEq)]
enum Obs {
    Failed(Error),
    Response { num_statements: usize, takes: Vec<(usize, Taken)> },
}

/// Observation of the pse-operations `checked` + `complete_response` semantics.
#[derive(Debug, PartialEq)]
enum Checked {
    Failed(Error),
    Incomplete,
    Ok { num_statements: usize, error_indices: Vec<usize> },
}

fn checked_pick(errors: Vec<(usize, Error)>) -> Option<Error> {
    // pse-operations transport::checked: first substantive (non NotExecuted /
    // Cancelled) error; otherwise the first error. SDK order is HashMap order.
    if errors.is_empty() {
        return None;
    }
    let substantive = errors.iter().position(|(_, e)| {
        !matches!(e.query_details(), Some(QueryError::NotExecuted | QueryError::Cancelled))
    });
    errors.into_iter().nth(substantive.unwrap_or(0)).map(|(_, e)| e)
}

// ---------------------------------------------------------------- SDK path

struct Sdk {
    db: Surreal<Client>,
}

fn ctx() -> RequestContext {
    RequestContext::new(Instant::now() + Duration::from_secs(30))
}
fn control() -> RequestContext {
    RequestContext::control(Instant::now() + Duration::from_secs(30))
}

impl Sdk {
    async fn bounded(address: SocketAddr) -> Result<Surreal<Client>, Error> {
        let config = Config::new()
            .bounded_requests()
            .grpc(GrpcConfig::new().max_message_size(direct::MESSAGE_BYTES))?;
        Surreal::new::<Grpc>((address, config)).await
    }
    async fn connect(address: SocketAddr, pass: &str) -> Result<Self, Error> {
        let db = Self::bounded(address).await?;
        db.signin(Root { username: "root".into(), password: pass.into() })
            .request_context(control())
            .await?;
        db.use_ns(NS).use_db(db_name()).request_context(control()).await?;
        Ok(Self { db })
    }
    async fn run(&self, script: &str, vars: &[(&'static str, Value)]) -> Result<surrealdb::IndexedResults, Error> {
        let mut q = self.db.query(script.to_string()).request_context(ctx());
        for (k, v) in vars {
            q = q.bind((*k, v.clone()));
        }
        q.await
    }
    async fn observe(&self, script: &str, vars: &[(&'static str, Value)], shape: Shape) -> Obs {
        match self.run(script, vars).await {
            Err(e) => Obs::Failed(e),
            Ok(mut r) => {
                let n = r.num_statements();
                let mut takes = Vec::new();
                for i in 0..=n + 1 {
                    let t = match shape {
                        Shape::OptObject => Taken::OptObject(r.take::<Option<Object>>(i)),
                        Shape::VecObject => Taken::VecObject(r.take::<Vec<Object>>(i)),
                        Shape::Value => Taken::Value(r.take::<Value>(i)),
                    };
                    takes.push((i, t));
                }
                Obs::Response { num_statements: n, takes }
            }
        }
    }
    async fn checked(&self, script: &str, vars: &[(&'static str, Value)]) -> Checked {
        match self.run(script, vars).await {
            Err(e) => Checked::Failed(e),
            Ok(mut r) => {
                let mut errors: Vec<(usize, Error)> = r.take_errors().into_iter().collect();
                let mut idx: Vec<usize> = errors.iter().map(|(i, _)| *i).collect();
                idx.sort();
                // Deterministic order for comparison; pse-operations sees HashMap order.
                errors.sort_by_key(|(i, _)| *i);
                if let Some(e) = checked_pick(errors) {
                    return Checked::Failed(e);
                }
                if r.num_statements() == 0 {
                    return Checked::Incomplete;
                }
                Checked::Ok { num_statements: r.num_statements(), error_indices: idx }
            }
        }
    }
}

// ------------------------------------------------------------- Direct path

async fn direct_connect(address: SocketAddr, pass: &str) -> Result<direct::Direct, Error> {
    let d = direct::Direct::connect(address).await?;
    d.signin_root("root", pass).await?;
    d.use_ns_db(NS, db_name()).await?;
    Ok(d)
}

async fn direct_observe(d: &direct::Direct, script: &str, vars: &[(&'static str, Value)], shape: Shape) -> Obs {
    match d.query(script, vars.to_vec(), None).await {
        Err(e) => Obs::Failed(e),
        Ok(mut r) => {
            let n = r.num_statements();
            let mut takes = Vec::new();
            for i in 0..=n + 1 {
                let t = match shape {
                    Shape::OptObject => Taken::OptObject(r.take_option::<Object>(i)),
                    Shape::VecObject => Taken::VecObject(r.take_vec::<Object>(i)),
                    Shape::Value => Taken::Value(r.take_value(i)),
                };
                takes.push((i, t));
            }
            Obs::Response { num_statements: n, takes }
        }
    }
}

async fn direct_checked(d: &direct::Direct, script: &str, vars: &[(&'static str, Value)]) -> Checked {
    match d.query(script, vars.to_vec(), None).await {
        Err(e) => Checked::Failed(e),
        Ok(mut r) => {
            let errors: Vec<(usize, Error)> = r.take_errors().into_iter().collect();
            let idx = errors.iter().map(|(i, _)| *i).collect();
            if let Some(e) = checked_pick(errors) {
                return Checked::Failed(e);
            }
            if r.num_statements() == 0 {
                return Checked::Incomplete;
            }
            Checked::Ok { num_statements: r.num_statements(), error_indices: idx }
        }
    }
}

// ------------------------------------------------------------------ report

struct Report {
    rows: Vec<(String, bool)>,
    detail: String,
}
impl Report {
    fn compare<T: Debug + PartialEq>(&mut self, name: &str, sdk: &T, direct: &T) {
        let same = sdk == direct;
        self.rows.push((name.to_string(), same));
        self.detail.push_str(&format!(
            "=== {name}: {}\n--- sdk\n{sdk:#?}\n--- direct\n{}\n",
            if same { "PARITY" } else { "DIVERGENT" },
            if same { "(identical)".to_string() } else { format!("{direct:#?}") }
        ));
    }
    fn note(&mut self, name: &str, text: String) {
        self.detail.push_str(&format!("=== {name}: NOTE\n{text}\n"));
    }
}

fn describe_error(e: &Error) -> String {
    format!(
        "kind={} query_details={:?} not_allowed={:?} is_thrown={} message={:?} cause={:?}",
        e.kind_str(),
        e.query_details(),
        e.not_allowed_details(),
        e.is_thrown(),
        e.message(),
        e.cause().map(|c| (c.kind_str(), c.message().to_string()))
    )
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let port: u16 = args[1].parse().unwrap();
    let pass = std::fs::read_to_string(&args[2]).unwrap().trim().to_string();
    let address: SocketAddr = ([127, 0, 0, 1], port).into();
    let mut report = Report { rows: Vec::new(), detail: String::new() };

    // ---- fixture setup through the plain SDK profile (same as the repo's held()).
    {
        let db = Surreal::new::<Grpc>(address).await.unwrap();
        db.signin(Root { username: "root".into(), password: pass.clone() }).await.unwrap();
        db.query(format!("DEFINE NAMESPACE IF NOT EXISTS {NS};")).await.unwrap().check().unwrap();
        db.use_ns(NS).await.unwrap();
        db.query(format!("DEFINE DATABASE IF NOT EXISTS {};", db_name())).await.unwrap().check().unwrap();
        db.use_db(db_name()).await.unwrap();
        db.query(FIXTURE_DDL)
            .bind(("live", LIVE))
            .bind(("expired", EXPIRED))
            .await
            .unwrap()
            .check()
            .unwrap();
    }

    // ---- Case 1: capabilities + attach + root signin + use ns/db.
    let sdk = Sdk::connect(address, &pass).await.expect("sdk connect");
    let dir = direct_connect(address, &pass).await.expect("direct connect");
    {
        let caps = &dir.capabilities;
        report.note(
            "1 capabilities (direct only; the SDK consumes them internally)",
            format!(
                "server_version={:?} limits={:?} capabilities={:?} denied_methods={:?} accepted_encodings={:?}",
                caps.server_version, caps.limits, caps.capabilities, caps.denied_methods, caps.accepted_message_encodings
            ),
        );
        // Signin token shape and use echo on fresh handles.
        let sdk_db = Sdk::bounded(address).await.unwrap();
        let sdk_token = sdk_db
            .signin(Root { username: "root".into(), password: pass.clone() })
            .request_context(control())
            .await
            .map(|t| format!("{t:?}").split('(').next().unwrap_or_default().to_string());
        let sdk_use = sdk_db.use_ns(NS).use_db(db_name()).request_context(control()).await.map(|_| ());
        let d2 = direct::Direct::connect(address).await.unwrap();
        let d_token = d2
            .signin_root("root", &pass)
            .await
            .map(|t| if t.refresh.is_empty() { "Access".to_string() } else { "WithRefresh".to_string() });
        let d_use = d2.use_ns_db(NS, db_name()).await;
        report.note(
            "1 signin/use",
            format!("sdk token variant={sdk_token:?} sdk use={sdk_use:?}\ndirect token variant={d_token:?} direct use echo={d_use:?} session={}", d2.session),
        );
        report.compare("1 signin succeeds (token variant)", &sdk_token.map(|s| s.contains("WithRefresh")), &d_token.map(|s| s == "WithRefresh"));
        report.compare("1 use succeeds", &sdk_use.is_ok(), &d_use.is_ok());
        // Session context check: a query sees the selected ns/db.
        let probe = "RETURN [session::ns(), session::db()];";
        report.compare(
            "1 session selection visible to queries",
            &sdk.observe(probe, &[], Shape::Value).await,
            &direct_observe(&dir, probe, &[], Shape::Value).await,
        );
    }

    let far = now_micros() + 60_000_000;
    let live = vars(LIVE, far);
    let expired = vars(EXPIRED, far);
    let past_deadline = vars(LIVE, now_micros() - 1);

    // ---- Case 2: guarded read scripts, three take shapes.
    let scripts: Vec<(&str, String)> = vec![
        (
            "2a protected SELECT ONLY (canonical_results shape)",
            format!("{PROTECTED_BEGIN}\nSELECT * FROM ONLY type::record('canonical_result_sets',$set);\nCOMMIT;"),
        ),
        (
            "2b protected LET/RETURN rows (select_names shape)",
            format!("{PROTECTED_BEGIN}\nLET $rpc_result = SELECT * FROM canonical_memberships WHERE problem = $problem ORDER BY name TIMEOUT $pse_rpc_timeout;\nfn::pse_execution_v1::deadline($pse_rpc_expires_at);\nIF $pin.expires_at <= time::micros() AND !fn::pse_files_v1::ticket_open($pse_rpc_expires_at,$protection) {{ THROW 'PSE_REFUSAL:immutable_selection_protection_expired'; }};\nRETURN $rpc_result;\nCOMMIT;"),
        ),
        (
            "2c unguarded RETURN object (testing::result_payload_counts shape)",
            "RETURN {inline:<decimal>array::len((SELECT name FROM canonical_memberships WHERE problem=$problem)),sealed:0dec};".to_string(),
        ),
        (
            "2d unguarded multi-statement (rows, empty, scalar, NONE)",
            "SELECT * FROM canonical_memberships ORDER BY name; SELECT * FROM canonical_memberships WHERE name='zzz'; RETURN 42; RETURN NONE;".to_string(),
        ),
    ];
    for (name, script) in &scripts {
        for shape in [Shape::OptObject, Shape::VecObject, Shape::Value] {
            report.compare(
                &format!("{name} take::<{shape:?}>"),
                &sdk.observe(script, &live, shape).await,
                &direct_observe(&dir, script, &live, shape).await,
            );
        }
        report.compare(&format!("{name} checked"), &sdk.checked(script, &live).await, &direct_checked(&dir, script, &live).await);
    }

    // ---- Case 3: THROW inside the guard; THROW inside a called function.
    let guarded = &scripts[0].1;
    for (name, v) in [("3a THROW expired protection", &expired), ("3b THROW from fn deadline", &past_deadline)] {
        let s = sdk.observe(guarded, v, Shape::OptObject).await;
        let d = direct_observe(&dir, guarded, v, Shape::OptObject).await;
        report.compare(&format!("{name} take::<OptObject> per index"), &s, &d);
        let sc = sdk.checked(guarded, v).await;
        let dc = direct_checked(&dir, guarded, v).await;
        if let Checked::Failed(e) = &sc {
            report.note(&format!("{name} sdk checked error"), describe_error(e));
        }
        report.compare(&format!("{name} checked"), &sc, &dc);
    }

    // ---- Case 4a: interactive SSI conflict; failure arrives on a unary commit.
    {
        // SDK: plain profile (bounded rejects begin() without a RequestContext,
        // which is why the repository's conflict tests use a plain `held()` client).
        async fn plain(address: SocketAddr, pass: &str) -> Surreal<Client> {
            let db = Surreal::new::<Grpc>(address).await.unwrap();
            db.signin(Root { username: "root".into(), password: pass.into() }).await.unwrap();
            db.use_ns(NS).use_db(db_name()).await.unwrap();
            db
        }
        let bounded_begin = sdk.db.clone().begin().await.map(|_| ());
        report.note("4a bounded SDK begin()", format!("{bounded_begin:?}"));
        let a = plain(address, &pass).await.begin().await.unwrap();
        let b = plain(address, &pass).await.begin().await.unwrap();
        a.query("UPDATE conflict_cells:k SET v += 1dec;").await.unwrap().check().unwrap();
        b.query("UPDATE conflict_cells:k SET v += 1dec;").await.unwrap().check().unwrap();
        let first = b.commit().await.map(|_| ());
        let sdk_conflict = a.commit().await.map(|_| ());

        let d1 = direct_connect(address, &pass).await.unwrap();
        let d2 = direct_connect(address, &pass).await.unwrap();
        let t1 = d1.begin().await.unwrap();
        let t2 = d2.begin().await.unwrap();
        d1.query("UPDATE conflict_cells:k SET v += 1dec;", vec![], Some(t1)).await.unwrap();
        d2.query("UPDATE conflict_cells:k SET v += 1dec;", vec![], Some(t2)).await.unwrap();
        let d_first = d2.commit(t2).await;
        let d_conflict = d1.commit(t1).await;
        report.compare("4a first commit succeeds", &first, &d_first);
        report.compare("4a second commit error", &sdk_conflict, &d_conflict);
        if let Err(e) = &sdk_conflict {
            report.note("4a sdk error", describe_error(e));
        }
        report.note("4a direct unary status/trailer", format!("{:?}", d1.last_status.lock().unwrap()));
    }

    // ---- Case 4b: conflict inside a single-call BEGIN..COMMIT script (query frames).
    {
        let slow = "BEGIN; UPDATE conflict_cells:k SET v += 1dec; sleep(600ms); UPDATE conflict_cells:k SET w = 1; COMMIT;";
        let fast = "BEGIN; UPDATE conflict_cells:k SET v += 10dec; COMMIT;";
        async fn sdk_pair(sdk: &Sdk, sdk2: &Sdk, slow: &str, fast: &str) -> (Obs, Checked) {
            let s = async { sdk.run(slow, &[]).await };
            let f = async {
                tokio::time::sleep(Duration::from_millis(150)).await;
                sdk2.run(fast, &[]).await.map(|_| ())
            };
            let (s, f) = tokio::join!(s, f);
            f.expect("fast writer");
            match s {
                Err(e) => (Obs::Failed(e.clone()), Checked::Failed(e)),
                Ok(mut r) => {
                    let n = r.num_statements();
                    let mut errors: Vec<(usize, Error)> = r.take_errors().into_iter().collect();
                    errors.sort_by_key(|(i, _)| *i);
                    let idx = errors.iter().map(|(i, _)| *i).collect();
                    let takes = errors.iter().map(|(i, e)| (*i, Taken::Value(Err(e.clone())))).collect();
                    let c = match checked_pick(errors) {
                        Some(e) => Checked::Failed(e),
                        None => Checked::Ok { num_statements: n, error_indices: idx },
                    };
                    (Obs::Response { num_statements: n, takes }, c)
                }
            }
        }
        async fn direct_pair(d: &direct::Direct, d2: &direct::Direct, slow: &str, fast: &str) -> (Obs, Checked) {
            let s = async { d.query(slow, vec![], None).await };
            let f = async {
                tokio::time::sleep(Duration::from_millis(150)).await;
                d2.query(fast, vec![], None).await.map(|_| ())
            };
            let (s, f) = tokio::join!(s, f);
            f.expect("fast writer");
            match s {
                Err(e) => (Obs::Failed(e.clone()), Checked::Failed(e)),
                Ok(mut r) => {
                    let n = r.num_statements();
                    let errors: Vec<(usize, Error)> = r.take_errors().into_iter().collect();
                    let idx = errors.iter().map(|(i, _)| *i).collect();
                    let takes = errors.iter().map(|(i, e)| (*i, Taken::Value(Err(e.clone())))).collect();
                    let c = match checked_pick(errors) {
                        Some(e) => Checked::Failed(e),
                        None => Checked::Ok { num_statements: n, error_indices: idx },
                    };
                    (Obs::Response { num_statements: n, takes }, c)
                }
            }
        }
        let sdk2 = Sdk::connect(address, &pass).await.unwrap();
        let dir2 = direct_connect(address, &pass).await.unwrap();
        let (so, sc) = sdk_pair(&sdk, &sdk2, slow, fast).await;
        let (dobs, dc) = direct_pair(&dir, &dir2, slow, fast).await;
        report.compare("4b script conflict per-statement errors", &so, &dobs);
        if let Checked::Failed(e) = &sc {
            report.note("4b sdk checked error", describe_error(e));
        }
        report.compare("4b script conflict checked", &sc, &dc);
    }

    // ---- Case 5: single-call BEGIN; RETURN fn::...; COMMIT;
    {
        let script = "BEGIN; RETURN fn::probe_v1::echo($x); COMMIT;";
        for shape in [Shape::OptObject, Shape::VecObject, Shape::Value] {
            report.compare(
                &format!("5 BEGIN/RETURN fn/COMMIT take::<{shape:?}>"),
                &sdk.observe(script, &live, shape).await,
                &direct_observe(&dir, script, &live, shape).await,
            );
        }
        report.compare("5 checked", &sdk.checked(script, &live).await, &direct_checked(&dir, script, &live).await);
    }

    // ---- Case 6: failing signin (unary status + error trailer).
    {
        let sdk_db = Sdk::bounded(address).await.unwrap();
        let s = sdk_db
            .signin(Root { username: "root".into(), password: "wrong-password".into() })
            .request_context(control())
            .await
            .map(|_| ());
        let d = direct::Direct::connect(address).await.unwrap();
        let r = d.signin_root("root", "wrong-password").await.map(|_| ());
        report.compare("6 failing signin error", &s, &r);
        if let Err(e) = &s {
            report.note("6 sdk error", describe_error(e));
        }
        report.note("6 direct unary status/trailer", format!("{:?}", d.last_status.lock().unwrap()));
        // Usability after a failed signin: the session stays usable for a correct signin.
        let s2 = sdk_db.signin(Root { username: "root".into(), password: pass.clone() }).request_context(control()).await.map(|_| ());
        let d2 = d.signin_root("root", &pass).await.map(|_| ());
        report.compare("6 subsequent correct signin", &s2.is_ok(), &d2.is_ok());
    }

    // ---- Case 7: parse error (whole-request failure).
    {
        let bad = "BEGIN; SELEC * FROM nothing; COMMIT;";
        let s = sdk.observe(bad, &[], Shape::Value).await;
        let d = direct_observe(&dir, bad, &[], Shape::Value).await;
        report.compare("7 parse error", &s, &d);
        report.note("7 direct unary status/trailer (None = in-stream)", format!("{:?}", dir.last_status.lock().unwrap()));
    }

    // ---- Case 8: statements that never emit (RETURN short-circuit) -> compaction.
    {
        let script = "BEGIN; RETURN 1; RETURN 2; CREATE conflict_cells:never SET v=0dec; COMMIT;";
        for shape in [Shape::Value, Shape::OptObject] {
            report.compare(
                &format!("8 BEGIN/RETURN/RETURN/CREATE/COMMIT take::<{shape:?}>"),
                &sdk.observe(script, &[], shape).await,
                &direct_observe(&dir, script, &[], shape).await,
            );
        }
        let probe = "SELECT * FROM conflict_cells:never;";
        report.compare("8 skipped CREATE had no effect", &sdk.observe(probe, &[], Shape::Value).await, &direct_observe(&dir, probe, &[], Shape::Value).await);
    }

    // ---- Output
    let parity = report.rows.iter().filter(|(_, s)| *s).count();
    println!("PARITY {parity}/{} trim_submitted_script={}", report.rows.len(), std::env::var("PROBE_TRIM").is_ok());
    for (name, same) in &report.rows {
        println!("{} | {name}", if *same { "parity   " } else { "DIVERGENT" });
    }
    std::fs::write(std::env::var("PROBE_DETAIL").unwrap_or("detail.txt".into()), report.detail).unwrap();
}
