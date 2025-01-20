use crate::actor::message::Message;
use crate::data::datasource::{DataSource, DataSourceErrors, DataSourceResult};
use futures::FutureExt;
use std::future::Future;
use std::time::Duration;
#[derive(derive_more::Constructor)]
pub struct TimeoutDecorator<M, D>
where
    M: Message,
    D: DataSource<Item = M>,
{
    duration: Duration,
    inner: D,
}

impl<M, D> DataSource for TimeoutDecorator<M, D>
where
    M: Message + Send,
    D: DataSource<Item = M> + Send + Sync,
{
    type Item = M;

    fn next(&mut self) -> impl Future<Output = DataSourceResult<Self::Item>> + Send {
        tokio::time::timeout(self.duration, self.inner.next()).then(|result| async move {
            result
                .map_err(|err| DataSourceErrors::CustomError(Box::new(err)))
                .and_then(|result| result)
        })
    }
}
