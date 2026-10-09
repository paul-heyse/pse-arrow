use std::borrow::Cow;
use std::future::IntoFuture;

use crate::conn::ctx;
use crate::method::{BoxFuture, OnceLockExt};
use crate::{Connection, Error, Result, Surreal};

/// Returned by [`Surreal::version`](crate::Surreal::version), yields the server version string.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Version<'r, C: Connection> {
    pub(super) request_context: Option<crate::opt::RequestContext>,
    pub(super) client: Cow<'r, Surreal<C>>,
}

impl<C> Version<'_, C>
where
    C: Connection,
{
    /// Attach the original connection/control operation lifetime.
    pub fn request_context(mut self, context: crate::opt::RequestContext) -> Self {
        self.request_context = Some(context);
        self
    }
    /// Converts to an owned type which can easily be moved to a different
    /// thread
    pub fn into_owned(self) -> Version<'static, C> {
        Version {
            request_context: self.request_context,
            client: Cow::Owned(self.client.into_owned()),
        }
    }
}

impl<'r, Client> IntoFuture for Version<'r, Client>
where
    Client: Connection,
{
    type Output = Result<semver::Version>;
    type IntoFuture = BoxFuture<'r, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let router = self.client.inner.router.extract()?;
            let version = router
                .engine
                .version(ctx(self.client.session_id).with_request_context(self.request_context))
                .await?;
            let semantic = version.trim_start_matches("surrealdb-");
            semantic
                .parse()
                .map_err(|_| Error::internal(format!("Invalid semantic version: \"{version}\"")))
        })
    }
}
