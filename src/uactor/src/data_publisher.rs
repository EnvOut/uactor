use tokio::sync::{mpsc, oneshot};

#[async_trait::async_trait]
pub trait DataPublisher {
    type Item;
    async fn publish(&mut self, data: Self::Item) -> DataPublisherResult;
}

#[derive(thiserror::Error, Debug)]
pub enum DataPublisherErrors {
    #[error("Channel closed")]
    Closed
}

impl<T> From<mpsc::error::SendError<T>> for DataPublisherErrors {
    fn from(_: mpsc::error::SendError<T>) -> Self {
        DataPublisherErrors::Closed
    }
}

impl From<oneshot::error::RecvError> for DataPublisherErrors {
    fn from(_: oneshot::error::RecvError) -> Self {
        DataPublisherErrors::Closed
    }
}

pub type DataPublisherResult = Result<(), DataPublisherErrors>;

#[async_trait::async_trait]
impl<T> DataPublisher for mpsc::Sender<T> where T: Send {
    type Item = T;

    async fn publish(&mut self, data: T) -> DataPublisherResult {
        self.send(data).await.map_err(|err| DataPublisherErrors::from(err))
    }
}

#[async_trait::async_trait]
impl<T> DataPublisher for mpsc::UnboundedSender<T> where T: Send {
    type Item = T;

    async fn publish(&mut self, data: T) -> DataPublisherResult {
        self.send(data).map_err(|err| DataPublisherErrors::from(err))
    }
}