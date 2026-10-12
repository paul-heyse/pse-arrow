use std::borrow::Cow;
use std::future::IntoFuture;
use std::marker::PhantomData;

use uuid::Uuid;

use super::transaction::WithTransaction;
use crate::conn::ctx_txn;
use crate::method::{BoxFuture, Live, OnceLockExt};
use crate::opt::Resource;
use crate::types::{RecordIdKeyRange, SurrealValue, Value, Variables};
use crate::{Connection, Result, Surreal};

/// Returned by [`Surreal::select`](crate::Surreal::select) for `SELECT` (including
/// [`Select::live`](Self::live) when using [`Live`](crate::method::Live)).
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Select<'r, C: Connection, R, T = ()> {
    pub(crate) request_context: Option<crate::opt::RequestContext>,
    pub(super) txn: Option<Uuid>,
    pub(super) client: Cow<'r, Surreal<C>>,
    pub(super) resource: Result<Resource>,
    pub(super) response_type: PhantomData<R>,
    pub(super) query_type: PhantomData<T>,
    // Actual resource bytes precede the undispatched ownership cleanup.
    pub(super) request_drain: Option<surrealdb_engine_api::request::UndispatchedPayloadDrain>,
}

impl<C, R, T> WithTransaction for Select<'_, C, R, T>
where
    C: Connection,
{
    fn with_transaction(mut self, id: Uuid) -> Self {
        self.txn = Some(id);
        self
    }
}

impl<C, R, T> Select<'_, C, R, T>
where
    C: Connection,
{
    /// Attach the original operation clock without changing the session.
    pub fn request_context(mut self, context: crate::opt::RequestContext) -> Self {
        let context = match self.request_context.as_ref() {
            Some(previous) => context.inherit_request_ownership(previous),
            None => context,
        };
        if let Some(drain) = &mut self.request_drain {
            drain.update_context(Some(context.clone()));
        } else if context.has_request_ownership() {
            self.request_drain = Some(surrealdb_engine_api::request::UndispatchedPayloadDrain::caller(Some(context.clone())));
        }
        self.request_context = Some(context);
        self
    }

    /// Converts to an owned type which can easily be moved to a different
    /// thread
    pub fn into_owned(self) -> Select<'static, C, R, T> {
        Select {
            client: Cow::Owned(self.client.into_owned()),
            ..self
        }
    }
}

macro_rules! into_future {
    ($method:ident) => {
        fn into_future(self) -> Self::IntoFuture {
            let Select { request_context, txn, client, resource, request_drain, .. } = self;
            // Exclude response PhantomData from the Send future. Ordered fields
            // still drain actual resources before cleanup if never polled.
            let owned = (client, resource, request_context, txn, request_drain);
            Box::pin(async move {
                let (client, resource, request_context, txn, request_drain) = owned;
                let _request_drain = request_drain;
                let client = client;
                let resource = resource;
                let router = client.inner.router.extract()?;

                let what = resource?;

                let mut variables = Variables::new();
                let what = what.for_sql_query(&mut variables)?;

                router
                    .$method(
                        ctx_txn(client.session_id, txn).with_request_context(request_context),
                        Cow::Owned(format!("SELECT * FROM {what}")),
                        variables,
                    )
                    .await
            })
        }
    };
}

impl<'r, Client> IntoFuture for Select<'r, Client, Value>
where
    Client: Connection,
{
    type Output = Result<Value>;
    type IntoFuture = BoxFuture<'r, Self::Output>;

    into_future! {query_value}
}

impl<'r, Client, R> IntoFuture for Select<'r, Client, Option<R>>
where
    Client: Connection,
    R: SurrealValue,
{
    type Output = Result<Option<R>>;
    type IntoFuture = BoxFuture<'r, Self::Output>;

    into_future! {query_opt}
}

impl<'r, Client, R> IntoFuture for Select<'r, Client, Vec<R>>
where
    Client: Connection,
    R: SurrealValue,
{
    type Output = Result<Vec<R>>;
    type IntoFuture = BoxFuture<'r, Self::Output>;

    into_future! {query_vec}
}

impl<C> Select<'_, C, Value>
where
    C: Connection,
{
    /// Restricts the records selected to those in the specified range
    pub fn range(mut self, range: impl Into<RecordIdKeyRange>) -> Self {
        self.resource = self.resource.and_then(|x| x.with_range(range.into()));
        self
    }
}

impl<C, R> Select<'_, C, Vec<R>>
where
    C: Connection,
{
    /// Restricts the records selected to those in the specified range
    pub fn range(mut self, range: impl Into<RecordIdKeyRange>) -> Self {
        self.resource = self.resource.and_then(|x| x.with_range(range.into()));
        self
    }
}

impl<'r, C, R> Select<'r, C, R>
where
    C: Connection,
    R: SurrealValue,
{
    /// Turns a normal select query into a live query
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use futures::StreamExt;
    /// # use surrealdb::opt::Resource;
    /// # use surrealdb::Result;
    /// # use surrealdb::Notification;
    /// # use surrealdb::types::SurrealValue;
    /// # #[derive(Debug, SurrealValue)]
    /// # struct Person;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> surrealdb::Result<()> {
    /// # let db = surrealdb::engine::any::connect("mem://").await?;
    /// #
    /// // Select the namespace/database to use
    /// db.use_ns("main").use_db("main").await?;
    ///
    /// // Listen to all updates on a table
    /// let mut stream = db.select("person").live().await?;
    /// # let _: Option<Result<Notification<Person>>> = stream.next().await;
    ///
    /// // Listen to updates on a range of records
    /// let mut stream = db.select("person").range("jane".."john").live().await?;
    /// # let _: Option<Result<Notification<Person>>> = stream.next().await;
    ///
    /// // Listen to updates on a specific record
    /// let mut stream = db.select(("person", "h5wxrf2ewk8xjxosxtyc")).live().await?;
    ///
    /// // The returned stream implements `futures::Stream` so we can
    /// // use it with `futures::StreamExt`, for example.
    /// while let Some(result) = stream.next().await {
    ///     handle(result);
    /// }
    ///
    /// // Handle the result of the live query notification
    /// fn handle(result: Result<Notification<Person>>) {
    ///     match result {
    ///         Ok(notification) => println!("{notification:?}"),
    ///         Err(error) => eprintln!("{error}"),
    ///     }
    /// }
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub fn live(self) -> Select<'r, C, R, Live> {
        Select {
            request_context: self.request_context,
            txn: self.txn,
            client: self.client,
            resource: self.resource,
            response_type: self.response_type,
            query_type: PhantomData,
            request_drain: self.request_drain,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_context_replacement_clears_composed_activity_on_unpolled_drop() {
        for future in [false, true] {
            let db = crate::Surreal::<crate::engine::any::Any>::init();
            let first = std::sync::Arc::new(());
            let second = std::sync::Arc::new(());
            let first_weak = std::sync::Arc::downgrade(&first);
            let second_weak = std::sync::Arc::downgrade(&second);
            let old = crate::opt::RequestContext::new(std::time::Instant::now() + std::time::Duration::from_secs(30))
                .with_lifetime_owner(first);
            let current = crate::opt::RequestContext::new(old.deadline()).with_lifetime_owner(second);
            let builder = db.select::<Vec<Value>>("item").request_context(old.clone()).into_owned().request_context(current);
            assert!(first_weak.upgrade().is_some() && second_weak.upgrade().is_some());
            if future { drop(builder.into_future()); } else { drop(builder); }
            assert!(first_weak.upgrade().is_none() && second_weak.upgrade().is_none(),
                "metadata clones cannot strand an undispatched activity");
            assert!(old.payload_owner().is_none());
        }
    }
}
