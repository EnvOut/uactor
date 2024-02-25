use crate::message::IntervalMessage;
use tokio::sync::{broadcast, mpsc, oneshot, watch};
use tokio::time::Interval;

pub type DataSourceResult<T> = Result<T, DataSourceErrors>;

#[derive(thiserror::Error, Debug)]
pub enum DataSourceErrors {
    #[error("Channel closed")]
    ChannelClosed,

    #[error("Channel lagged by {0}")]
    ChannelLagged(u64),

    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

impl From<broadcast::error::RecvError> for DataSourceErrors {
    fn from(err: broadcast::error::RecvError) -> Self {
        use broadcast::error::RecvError;

        match err {
            RecvError::Closed => Self::ChannelClosed,
            RecvError::Lagged(number_skipped_messages) => {
                Self::ChannelLagged(number_skipped_messages)
            }
        }
    }
}

pub trait DataSource {
    type Item;
    fn next(&mut self) -> impl std::future::Future<Output = DataSourceResult<Self::Item>> + Send;
}

impl<T> DataSource for mpsc::Receiver<T>
    where
        T: Send,
{
    type Item = T;

    async fn next(&mut self) -> DataSourceResult<Self::Item> {
        if let Some(value) = self.recv().await {
            Ok(value)
        } else {
            Err(DataSourceErrors::ChannelClosed)
        }
    }
}

impl<T> DataSource for mpsc::UnboundedReceiver<T>
    where
        T: Send,
{
    type Item = T;

    async fn next(&mut self) -> DataSourceResult<Self::Item> {
        if let Some(value) = self.recv().await {
            Ok(value)
        } else {
            Err(DataSourceErrors::ChannelClosed)
        }
    }
}

impl<T> DataSource for watch::Receiver<T>
    where
        T: Clone + Send + Sync,
{
    type Item = T;

    async fn next(&mut self) -> DataSourceResult<Self::Item> {
        let _ = self
            .changed()
            .await
            .map_err(|_| DataSourceErrors::ChannelClosed)?;
        let value = self.borrow().clone();
        Ok(value)
    }
}

impl<T> DataSource for broadcast::Receiver<T>
    where
        T: Clone + Send + Sync,
{
    type Item = T;

    async fn next(&mut self) -> DataSourceResult<Self::Item> {
        self.recv().await.map_err(DataSourceErrors::from)
    }
}

impl<T> DataSource for oneshot::Receiver<T>
    where
        T: Send,
{
    type Item = T;

    async fn next(&mut self) -> DataSourceResult<Self::Item> {
        self.await.map_err(|_| DataSourceErrors::ChannelClosed)
    }
}

impl DataSource for Interval {
    type Item = IntervalMessage;

    async fn next(&mut self) -> DataSourceResult<Self::Item> {
        let instant = self.tick().await;
        Ok(IntervalMessage {
            time: instant,
            duration: self.period(),
        })
    }
}