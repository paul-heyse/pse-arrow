// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Test-only commit-order fixture. It uses the production bounded transport and
//! protobuf decoder while retaining one native transaction on its original driver.
#![allow(
    clippy::expect_used,
    reason = "bounded fixture transaction remains open until its consuming settlement; misuse is a test failure"
)]
use super::*;

struct Held {
    client: Client,
    transaction: uuid::Uuid,
    session: uuid::Uuid,
    deadline: Instant,
}
/// An isolated fixture for observing intermediate state before choosing commit order.
pub(crate) struct HeldTransaction(Option<Held>);
impl Client {
    pub(crate) async fn test_begin(self, deadline: Instant) -> EngineResult<HeldTransaction> {
        let (transaction, session) = self
            .execute(RequestContext::control(deadline), |generation| async move {
                let response = generation
                    .client
                    .clone()
                    .begin_transaction(generation.checked(rpc::BeginTransactionRequest {
                        context: Some(generation.context()?),
                    })?)
                    .await
                    .map_err(errors::status_to_error)?
                    .into_inner();
                let transaction = response
                    .transaction
                    .ok_or_else(|| connection("Fixture server omitted transaction"))?
                    .to_uuid()
                    .map_err(connection)?;
                Ok((transaction, generation.session))
            })
            .await?;
        Ok(HeldTransaction(Some(Held {
            client: self,
            transaction,
            session,
            deadline,
        })))
    }
}
impl HeldTransaction {
    #[cfg(any(feature = "canonical-tests", feature = "test-support"))]
    pub(crate) fn query(&self, query: impl Into<Cow<'static, str>>) -> Query<'static> {
        let held = self
            .0
            .as_ref()
            .expect("held fixture remains open until consumed");
        let mut query = held
            .client
            .query(query)
            .request_context(RequestContext::new(held.deadline));
        query.transaction = Some((held.transaction, held.session));
        query
    }
    pub(crate) async fn commit(mut self) -> EngineResult<()> {
        finish(self.0.take().expect("open commit-order fixture"), true).await
    }
    #[cfg(any(feature = "canonical-tests", feature = "test-support"))]
    pub(crate) async fn cancel(mut self) -> EngineResult<()> {
        finish(self.0.take().expect("open rollback fixture"), false).await
    }
}
async fn finish(held: Held, commit: bool) -> EngineResult<()> {
    let transaction = held.transaction;
    let session = held.session;
    let result = held
        .client
        .execute(
            RequestContext::control(held.deadline),
            move |generation| async move {
                if generation.session != session {
                    return Err(connection("Held fixture lost original transaction driver"));
                }
                let context = rpc::RequestContext {
                    transaction: Some(proto::Uuid::from_uuid(transaction)),
                    ..generation.context()?
                };
                if commit {
                    generation
                        .client
                        .clone()
                        .commit_transaction(generation.checked(rpc::CommitTransactionRequest {
                            context: Some(context),
                        })?)
                        .await
                        .map_err(errors::status_to_error)?;
                } else {
                    generation
                        .client
                        .clone()
                        .cancel_transaction(generation.checked(rpc::CancelTransactionRequest {
                            context: Some(context),
                        })?)
                        .await
                        .map_err(errors::status_to_error)?;
                }
                Ok(())
            },
        )
        .await;
    // Closing the original driver also settles failed/expired fixture requests.
    // No transaction or application effect is recreated on a replacement driver.
    let closed = held.client.disconnect().await;
    result.and(closed)
}
impl Drop for HeldTransaction {
    fn drop(&mut self) {
        if let Some(held) = self.0.take() {
            tokio::spawn(async move {
                let _ = finish(held, false).await;
            });
        }
    }
}
