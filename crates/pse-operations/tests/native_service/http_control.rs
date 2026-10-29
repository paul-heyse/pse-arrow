// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual actor API tests using an owned native deployment, without a Python client.
use super::*;
use serde_json::{Value, json};

async fn post(
    client: &reqwest::Client,
    url: &str,
    token: Option<&str>,
    body: Vec<u8>,
) -> std::io::Result<(u16, Value)> {
    let mut request = client
        .post(url)
        .header("content-type", "application/json")
        .header("accept", "application/json")
        .body(body);
    if let Some(token) = token {
        request = request.bearer_auth(token);
    }
    read_response(request.send().await.map_err(std::io::Error::other)?).await
}
async fn read_response(mut response: reqwest::Response) -> std::io::Result<(u16, Value)> {
    let status = response.status().as_u16();
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(std::io::Error::other)? {
        assert!(
            bytes.len() + chunk.len() <= 262144,
            "bounded actor response"
        );
        bytes.extend_from_slice(&chunk);
    }
    let value = serde_json::from_slice(&bytes)
        .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()));
    Ok((status, value))
}
fn encode(value: Value) -> Vec<u8> {
    value.to_string().into_bytes()
}
fn envelope(issuer: &str, request: Value) -> std::io::Result<Value> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(std::io::Error::other)?
        .as_micros();
    let expires_at = u64::try_from(now)
        .map_err(std::io::Error::other)?
        .checked_add(30_000_000)
        .ok_or_else(|| std::io::Error::other("actor request expiry exceeds the wire clock"))?;
    Ok(json!({"issuing_store":issuer, "expires_at":expires_at,"request":request}))
}
async fn sign_in(
    client: &reqwest::Client,
    endpoint: &str,
    actor: &str,
    password: &str,
) -> std::io::Result<String> {
    let (status, body) = post(client, &format!("{endpoint}/signin"), None, encode(json!({
        "NS":"native_fixture","DB":"canonical","AC":"pse_control","actor":actor,"password":password,
    }))).await?;
    assert_eq!(status, 200, "record-actor signin: {body:?}");
    Ok(body["token"]
        .as_str()
        .ok_or_else(|| {
            std::io::Error::other(format!(
                "record-actor token absent from actual signin response: {body:?}"
            ))
        })?
        .to_owned())
}
#[tokio::test]
async fn native_control_http_actor_grants_original_receipts_and_raw_envelopes() {
    let mut service = fixture_with_schema(false).unwrap();
    service.run(Instant::now()+Duration::from_secs(240), |service| Box::pin(async move {
        service.launch(Instant::now()+Duration::from_secs(90)).await?;
        let store = service.ready()?.store();
        let issuer = store.issuing_store()?.to_owned();
        let endpoint = store.endpoint().replacen("grpc://", "http://", 1);
        let base = format!("{endpoint}/api/native_fixture/canonical/pse/control/v1");
        let password = "private-actor-fixture-password";
        store.issue_control_actor("allowed", password, true, true).await?;
        store.issue_control_actor("denied", password, false, false).await?;
        store.issue_control_actor("viewer", password, true, false).await?;
        store.issue_control_actor("revoked", password, true, true).await?;
        let client = reqwest::Client::builder().no_proxy().redirect(reqwest::redirect::Policy::none()).timeout(Duration::from_secs(20)).build().unwrap();
        pse_operations::testing::install_control_transaction_probe(store).await?;
        for (route, expected_status, marker) in [
            ("success",200,"committed-success"),
            ("refusal",409,"committed-refusal"),
        ] {
            let (status, body) = post(&client,&format!("{endpoint}/api/native_fixture/canonical/pse-fixture/control/{route}"),None,vec![]).await?;
            assert_eq!(status,expected_status,"positive API transaction control: {body:?}");
            assert_eq!(body["marker"],marker);
        }
        let (status, body) = post(&client,&format!("{endpoint}/api/native_fixture/canonical/pse-fixture/control/failure"),None,vec![]).await?;
        assert_eq!(status,500,"after-write THROW must fail: {body:?}");
        assert!(body.to_string().contains("private control after-write failure witness"),"unrelated API failure: {body:?}");
        assert_eq!(pse_operations::testing::control_transaction_probe_markers(store).await?,vec!["committed-refusal","committed-success"],
            "engine errors roll back while explicit success/refusal responses commit");
        let allowed = sign_in(&client,&endpoint,"allowed",password).await?;
        let denied = sign_in(&client,&endpoint,"denied",password).await?;
        let viewer = sign_in(&client,&endpoint,"viewer",password).await?;
        let revoked = sign_in(&client,&endpoint,"revoked",password).await?;
        let filter = serde_json::to_value(pse_operations::canonical_study_workbench::StudyWorkbenchFilter::default()).unwrap();
        let browse = json!({"document_version":1,"filter":filter,"maximum":1,"cursor":null});
        let (status, body) = post(&client,&format!("{base}/browse"),Some(&allowed),encode(envelope(&issuer,browse.clone())?)).await?;
        assert_eq!(status,200,"{body:?}");
        assert_eq!(body["headers"],json!([]));
        let (status, body) = post(&client,&format!("{base}/cancel"),Some(&denied),vec![0xff]).await?;
        assert_eq!(status,403,"grant denial must precede invalid UTF-8 parsing: {body:?}");
        assert_eq!(body["error"],"control_actor_refused");
        pse_operations::testing::disable_control_actor(store,"revoked").await?;
        let (status, body) = post(&client,&format!("{base}/cancel"),Some(&revoked),vec![0xff]).await?;
        assert_eq!(status,403,"previously issued token ignored actor revocation: {body:?}");
        assert_eq!(body["error"],"control_actor_refused");
        for action in ["cancel", "reconcile"] {
            let (status, body) = post(&client,&format!("{base}/{action}"),Some(&viewer),vec![0xff]).await?;
            assert_eq!(status,403,"browse-only actor gained {action} authority: {body:?}");
            assert_eq!(body["error"],"control_actor_refused");
        }
        let (status, body) = post(&client,&format!("{base}/browse"),Some(&viewer),encode(envelope(&issuer,browse.clone())?)).await?;
        assert_eq!(status,200,"browse-only actor lost its declared grant: {body:?}");

        // Raw decoder failures must precede the command and leave no receipt.
        // Readback uses the same exact identity rather than retrying a fresh effect.
        for invalid_utf8 in [false, true] {
            let operation = uuid::Uuid::new_v4().to_string();
            let request = json!({"document_version":1,"issuing_store":issuer,"operation_id":operation,"study":"absent-malformed-target"});
            let mut raw = encode(envelope(&issuer,request)?);
            if invalid_utf8 {
                let position = raw.iter().position(|byte| *byte == b'a').unwrap();
                raw[position] = 0xff;
            } else {
                raw.pop();
            }
            let (status, body) = post(&client,&format!("{base}/cancel"),Some(&allowed),raw).await?;
            assert_eq!(status,500,"malformed command unexpectedly reached effect: {body:?}");
            let request = json!({"document_version":1,"issuing_store":issuer,"operation_id":operation});
            let (status, body) = post(&client,&format!("{base}/reconcile"),Some(&allowed),encode(envelope(&issuer,request)?)).await?;
            assert_eq!(status,200,"{body:?}");
            assert!(body["receipt"].is_null(),"decoder failure invented a receipt: {body:?}");
        }

        // Token authority is confined to the API, not an arbitrary query surface.
        // The interpretation marker is known to exist from successful construction.
        for query in [
            "RETURN fn::pse_control_v1::actor('cancel');",
            "SELECT * FROM canonical_interpretations WHERE id=canonical_interpretations:current;",
            "UPDATE canonical_interpretations:current SET issuing_store=issuing_store RETURN AFTER;",
        ] {
            let response = client.post(format!("{endpoint}/sql")).bearer_auth(&allowed)
                .header("surreal-ns","native_fixture").header("surreal-db","canonical")
                .header("accept","application/json").body(query).send().await.unwrap();
            let (status, body) = read_response(response).await?;
            if status == 200 {
                let rows = body.as_array().expect("SQL statement response");
                assert_eq!(rows.len(),1);
                let row = &rows[0];
                if row["status"] == "ERR" {
                    let refusal = row["result"].to_string().to_lowercase();
                    assert!(["permission","not allowed","access"].iter().any(|word| refusal.contains(word)),"unrelated SQL error: {body:?}");
                } else {
                    assert_eq!(row["status"],"OK");
                    assert!(row["result"].is_null() || row["result"] == json!([]),"actor accessed protected records: {body:?}");
                }
            } else {
                assert!([401,403].contains(&status),"unrelated SQL failure: {body:?}");
            }
        }
        let operation = "000123";
        let cancel = json!({"document_version":1,"issuing_store":issuer,"operation_id":operation,"study":"2026-10-29T00:00:00Z"});
        let (status, receipt) = post(&client,&format!("{base}/cancel"),Some(&allowed),encode(envelope(&issuer,cancel.clone())?)).await?;
        assert_eq!(status,200,"{receipt:?}");
        assert_eq!(receipt["status"],"refused");
        assert_eq!(receipt["operation_id"],operation);
        assert_eq!(receipt["study"],cancel["study"]);
        let (_, replay) = post(&client,&format!("{base}/cancel"),Some(&allowed),encode(envelope(&issuer,cancel.clone())?)).await?;
        assert_eq!(receipt,replay);
        let reconcile = json!({"document_version":1,"issuing_store":issuer,"operation_id":operation});
        let (status, observed) = post(&client,&format!("{base}/reconcile"),Some(&allowed),encode(envelope(&issuer,reconcile)?)).await?;
        assert_eq!(status,200,"{observed:?}");
        assert_eq!(observed["receipt"],receipt);
        let mut changed = cancel;
        changed["study"] = json!("different-opaque-target");
        let (status, refusal) = post(&client,&format!("{base}/cancel"),Some(&allowed),encode(envelope(&issuer,changed)?)).await?;
        assert_eq!(status,409,"{refusal:?}");
        assert_eq!(refusal["reason"],"control_command_identity_reused");
        for spelling in [
            "ABCDEFAB-CDEF-4ABC-8DEF-ABCDEFABCDEF",
            "CaseSensitiveTable:CaseSensitiveId",
            "escaped/\"slash\\🧪",
        ] {
            let request = json!({"document_version":1,"issuing_store":issuer,"operation_id":spelling,"study":spelling});
            let (status, original) = post(&client,&format!("{base}/cancel"),Some(&allowed),encode(envelope(&issuer,request.clone())?)).await?;
            assert_eq!(status,200,"opaque identity failed: {original:?}");
            assert_eq!(original["operation_id"],spelling);
            assert_eq!(original["study"],spelling);
            assert_eq!(original["status"],"refused");
            let (status, replay) = post(&client,&format!("{base}/cancel"),Some(&allowed),encode(envelope(&issuer,request)?)).await?;
            assert_eq!(status,200);
            assert_eq!(replay,original,"opaque identity replay differs");
        }
        let mut invalid_cursor = browse.clone();
        invalid_cursor["cursor"] = json!({"issuing_store":issuer,"filter":filter,"after":"last","unexpected":true});
        let (status, body) = post(&client,&format!("{base}/browse"),Some(&allowed),encode(envelope(&issuer,invalid_cursor)?)).await?;
        assert_eq!(status,500,"unknown cursor field admitted: {body:?}");
        assert!(body.to_string().contains("control_request_invalid"),"unrelated cursor failure: {body:?}");
        let mut huge_integer = browse;
        huge_integer["maximum"] = json!(u64::MAX);
        for body in [vec![0xff], encode(envelope(&issuer,huge_integer)?), vec![b'x';65537]] {
            let (status, reply) = post(&client,&format!("{base}/browse"),Some(&allowed),body).await?;
            assert!(!(200..300).contains(&status),"invalid raw/extent/integer admitted: {reply:?}");
        }
        let (status, reply) = post(&client,&format!("{base}/browse"),Some(&allowed),encode(envelope(&uuid::Uuid::new_v4().to_string(),json!({"document_version":1,"filter":filter,"maximum":1,"cursor":null}))?)).await?;
        assert!(!(200..300).contains(&status),"foreign issuer admitted: {reply:?}");
        Ok(())
    })).await.unwrap();
    service.dispose().unwrap();
}
