use tokio::sync::{broadcast, mpsc, oneshot, watch};

pub type DataSourceResult<T> = Result<T, DataSourceErrors>;

#[derive(thiserror::Error, Debug)]
pub enum DataSourceErrors {
    #[error("Channel closed")]
    ChannelClosed,

    #[error("Channel lagged by {0}")]
    ChannelLagged(u64)
}

impl From<broadcast::error::RecvError> for DataSourceErrors {
    fn from(err: broadcast::error::RecvError) -> Self {
        use broadcast::error::RecvError;

        match err {
            RecvError::Closed => {Self::ChannelClosed}
            RecvError::Lagged(number_skipped_messages) => {Self::ChannelLagged(number_skipped_messages)}
        }
    }
}

#[async_trait::async_trait]
pub trait DataSource {
    type Item;
    async fn next(&mut self) -> DataSourceResult<Self::Item>;
}

#[async_trait::async_trait]
impl<T> DataSource for mpsc::Receiver<T> where T: Send {
    type Item = T;

    async fn next(&mut self) -> DataSourceResult<Self::Item> {
        if let Some(value) = self.recv().await {
            Ok(value)
        } else {
            Err(DataSourceErrors::ChannelClosed)
        }
    }
}

#[async_trait::async_trait]
impl<T> DataSource for mpsc::UnboundedReceiver<T> where T: Send {
    type Item = T;

    async fn next(&mut self) -> DataSourceResult<Self::Item> {
        if let Some(value) = self.recv().await {
            Ok(value)
        } else {
            Err(DataSourceErrors::ChannelClosed)
        }
    }
}

#[async_trait::async_trait]
impl<T> DataSource for watch::Receiver<T> where T: Clone + Send + Sync {
    type Item = T;

    async fn next(&mut self) -> DataSourceResult<Self::Item> {
        let _ = self.changed().await.map_err(|_| DataSourceErrors::ChannelClosed)?;
        let value = self.borrow().clone();
        Ok(value)
    }
}

#[async_trait::async_trait]
impl<T> DataSource for broadcast::Receiver<T>  where T: Clone + Send + Sync {
    type Item = T;

    async fn next(&mut self) -> DataSourceResult<Self::Item> {
        self.recv().await.map_err(|err| DataSourceErrors::from(err))
    }
}

#[async_trait::async_trait]
impl<T> DataSource for oneshot::Receiver<T> where T: Send {
    type Item = T;

    async fn next(&mut self) -> DataSourceResult<Self::Item> {
        self.await.map_err(|_| DataSourceErrors::ChannelClosed)
    }
}