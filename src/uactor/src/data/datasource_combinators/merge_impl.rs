use crate::data::datasource::{DataSource, DataSourceErrors, DataSourceResult};

pub struct DataSourceMerge<D1, D2>
where
    D1: DataSource + Send,
    D2: DataSource<Item = D1::Item> + Send,
{
    left: Option<D1>,
    right: Option<D2>,
}

impl<D1, D2> DataSource for DataSourceMerge<D1, D2>
where
    D1: DataSource + Send,
    D2: DataSource<Item = D1::Item> + Send,
{
    type Item = D1::Item;

    async fn next(&mut self) -> DataSourceResult<Self::Item> {
        loop {
            match (&mut self.left, &mut self.right) {
                (Some(left), Some(right)) => {
                    tokio::select! {
                        result = left.next() => match result {
                            ok @ Ok(_) => return ok,
                            Err(DataSourceErrors::ChannelClosed) => self.left = None,
                            err => return err,
                        },
                        result = right.next() => match result {
                            ok @ Ok(_) => return ok,
                            Err(DataSourceErrors::ChannelClosed) => self.right = None,
                            err => return err,
                        },
                    }
                }
                (Some(left), None) => return left.next().await,
                (None, Some(right)) => return right.next().await,
                (None, None) => return Err(DataSourceErrors::ChannelClosed),
            }
        }
    }
}

pub trait DataSourceMergeExt: DataSource + Send + Sized {
    fn merge<D2>(self, other: D2) -> DataSourceMerge<Self, D2>
    where
        D2: DataSource<Item = Self::Item> + Send;
}

impl<D> DataSourceMergeExt for D
where
    D: DataSource + Send,
{
    fn merge<D2>(self, other: D2) -> DataSourceMerge<Self, D2>
    where
        D2: DataSource<Item = Self::Item> + Send,
    {
        DataSourceMerge {
            left: Some(self),
            right: Some(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::data::datasource::DataSource;
    use crate::data::datasource_combinators::DataSourceMergeExt;

    #[tokio::test]
    async fn test_merge_interleaved() {
        let (tx_1, rx_1) = tokio::sync::mpsc::unbounded_channel();
        tx_1.send(1).unwrap();
        tx_1.send(3).unwrap();
        tx_1.send(5).unwrap();
        tx_1.send(7).unwrap();
        tx_1.send(9).unwrap();

        let (tx_2, rx_2) = tokio::sync::mpsc::unbounded_channel();
        tx_2.send(2).unwrap();
        tx_2.send(4).unwrap();
        tx_2.send(6).unwrap();
        tx_2.send(8).unwrap();
        tx_2.send(10).unwrap();

        drop(tx_1);
        drop(tx_2);

        let mut stream = rx_1.merge(rx_2);

        let mut sum = 0;
        while let Ok(value) = stream.next().await {
            sum += value;
        }

        assert_eq!(sum, 55);
    }

    #[tokio::test]
    async fn test_merge_one_empty() {
        let (tx_1, rx_1) = tokio::sync::mpsc::unbounded_channel();
        tx_1.send(1).unwrap();
        tx_1.send(2).unwrap();
        tx_1.send(3).unwrap();
        drop(tx_1);

        let (_tx_2, rx_2) = tokio::sync::mpsc::unbounded_channel::<i32>();
        drop(_tx_2);

        let mut stream = rx_1.merge(rx_2);

        let mut sum = 0;
        while let Ok(value) = stream.next().await {
            sum += value;
        }

        assert_eq!(sum, 6);
    }

    #[tokio::test]
    async fn test_merge_different_channel_types() {
        let (tx_1, rx_1) = tokio::sync::mpsc::unbounded_channel();
        tx_1.send(10).unwrap();
        tx_1.send(20).unwrap();
        drop(tx_1);

        let (tx_2, rx_2) = tokio::sync::mpsc::channel(8);
        tx_2.send(30).await.unwrap();
        tx_2.send(40).await.unwrap();
        drop(tx_2);

        let mut stream = rx_1.merge(rx_2);

        let mut sum = 0;
        while let Ok(value) = stream.next().await {
            sum += value;
        }

        assert_eq!(sum, 100);
    }
}
