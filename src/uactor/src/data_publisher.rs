#[cfg(feature = "async_sender")]
pub use async_sender::*;

#[cfg(feature = "async_sender")]
mod async_sender {
    use tokio::sync::{broadcast, mpsc, oneshot, watch};
    use tokio::sync::mpsc::{Sender, UnboundedSender};

    pub trait DataPublisher: TryClone {
        type Item;
        fn publish(&self, data: Self::Item) -> impl std::future::Future<Output = DataPublisherResult> + Send;
    }

    #[derive(thiserror::Error, Debug)]
    pub enum DataPublisherErrors {
        #[error("Channel closed")]
        Closed
    }

    pub trait TryClone: Sized {
        fn try_clone(&self) -> Result<Self, TryCloneError>;
    }

    #[derive(thiserror::Error, Debug)]
    pub enum TryCloneError {
        #[error("Can't be cloned")]
        CantClone
    }

    impl Clone for TryCloneError {
        fn clone(&self) -> Self {
            todo!()
        }
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


    impl<T> DataPublisher for mpsc::Sender<T> where T: Send {
        type Item = T;

        async fn publish(&self, data: Self::Item) -> DataPublisherResult {
            self.send(data).await.map_err(|err| DataPublisherErrors::from(err))
        }


    }

    impl<T> TryClone for Sender<T> where T: Send {
        fn try_clone(&self) -> Result<Self, TryCloneError> {
            Ok(self.clone())
        }
    }


    impl<T> DataPublisher for mpsc::UnboundedSender<T> where T: Send {
        type Item = T;

        async fn publish(&self, data: Self::Item) -> DataPublisherResult {
            self.send(data).map_err(|err| DataPublisherErrors::from(err))
        }
    }

    impl<T> TryClone for UnboundedSender<T> where T: Send {
        fn try_clone(&self) -> Result<Self, TryCloneError> {
            Ok(self.clone())
        }
    }


    impl<T> DataPublisher for watch::Sender<T> where T: Send + Sync {
        type Item = T;

        async fn publish(&self, data: Self::Item) -> DataPublisherResult {
            self.send(data).map_err(|_| DataPublisherErrors::Closed)
        }
    }

    impl<T> TryClone for watch::Sender<T> where T: Send {
        fn try_clone(&self) -> Result<Self, TryCloneError> {
            Err(TryCloneError::CantClone)
        }
    }

    impl<T> DataPublisher for broadcast::Sender<T> where T: Send + Sync {
        type Item = T;

        async fn publish(&self, data: Self::Item) -> DataPublisherResult {
            self.send(data).map(|_receivers_count|()).map_err(|_| DataPublisherErrors::Closed)
        }
    }

    impl<T> TryClone for broadcast::Sender<T> where T: Send {
        fn try_clone(&self) -> Result<Self, TryCloneError> {
            Ok(self.clone())
        }
    }
}

#[cfg(not(feature = "async_sender"))]
pub use sync_sender::*;

#[cfg(not(feature = "async_sender"))]
mod sync_sender {
    use tokio::sync::{broadcast, mpsc, oneshot, watch};
    use tokio::sync::mpsc::UnboundedSender;

    pub trait DataPublisher: TryClone {
        type Item;
        fn publish(&self, data: Self::Item) -> DataPublisherResult;
    }

    #[derive(thiserror::Error, Debug)]
    pub enum DataPublisherErrors {
        #[error("Channel closed")]
        Closed
    }

    pub trait TryClone: Sized {
        fn try_clone(&self) -> Result<Self, TryCloneError>;
    }

    #[derive(thiserror::Error, Debug)]
    pub enum TryCloneError {
        #[error("Can't be cloned")]
        CantClone
    }

    impl Clone for TryCloneError {
        fn clone(&self) -> Self {
            todo!()
        }
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


    impl<T> DataPublisher for mpsc::UnboundedSender<T> where T: Send {
        type Item = T;

        fn publish(&self, data: Self::Item) -> DataPublisherResult {
            self.send(data).map_err(|err| DataPublisherErrors::from(err))
        }
    }

    impl<T> TryClone for UnboundedSender<T> where T: Send {
        fn try_clone(&self) -> Result<Self, TryCloneError> {
            Ok(self.clone())
        }
    }

    impl<T> DataPublisher for watch::Sender<T> where T: Send + Sync {
        type Item = T;

        fn publish(&self, data: Self::Item) -> DataPublisherResult {
            self.send(data).map_err(|_| DataPublisherErrors::Closed)
        }
    }

    impl<T> TryClone for watch::Sender<T> where T: Send {
        fn try_clone(&self) -> Result<Self, TryCloneError> {
            Err(TryCloneError::CantClone)
        }
    }

    impl<T> DataPublisher for broadcast::Sender<T> where T: Send + Sync {
        type Item = T;

        fn publish(&self, data: Self::Item) -> DataPublisherResult {
            self.send(data).map(|_receivers_count|()).map_err(|_| DataPublisherErrors::Closed)
        }
    }

    impl<T> TryClone for broadcast::Sender<T> where T: Send {
        fn try_clone(&self) -> Result<Self, TryCloneError> {
            Ok(self.clone())
        }
    }
}