use crate::data::datasource::{DataSource, DataSourceResult};

pub struct DataSourceFilterMap<D, F, R>
where
    F: Fn(D::Item) -> Option<R> + Send,
    D: DataSource + Send,
{
    datasource: D,
    filter_fn: F,
}

impl<D, F, R> DataSourceFilterMapExt<D, F, R> for DataSourceFilterMap<D, F, R>
where
    F: Fn(D::Item) -> Option<R> + Send,
    D: DataSource + Send,
{
    fn filter_map(self, map_fn: F) -> DataSourceFilterMap<D, F, R> {
        DataSourceFilterMap {
            datasource: self.datasource,
            filter_fn: map_fn,
        }
    }
}

impl<D, F, R> DataSource for DataSourceFilterMap<D, F, R>
where
    F: Fn(D::Item) -> Option<R> + Send,
    D: DataSource + Send,
{
    type Item = R;

    async fn next(&mut self) -> DataSourceResult<Self::Item> {
        loop {
            let value = DataSource::next(&mut self.datasource).await?;
            if let Some(value) = (self.filter_fn)(value) {
                return Ok(value);
            } else {
                continue;
            }
        }
    }
}

pub trait DataSourceFilterMapExt<D, F, R>
where
    F: Fn(D::Item) -> Option<R> + Send,
    D: DataSource + Send,
{
    #[allow(dead_code)]
    fn filter_map(self, map_fn: F) -> DataSourceFilterMap<D, F, R>;
}

impl<D, F, R> DataSourceFilterMapExt<D, F, R> for D
where
    F: Fn(D::Item) -> Option<R> + Send,
    D: DataSource + Send,
{
    fn filter_map(self, map_fn: F) -> DataSourceFilterMap<D, F, R> {
        DataSourceFilterMap {
            datasource: self,
            filter_fn: map_fn,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DataSourceFilterMapExt;
    use crate::data::datasource::DataSource;

    #[tokio::test]
    async fn test_map() {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        tx.send(1).unwrap();
        tx.send(-2).unwrap();
        tx.send(3).unwrap();
        tx.send(-4).unwrap();
        tx.send(5).unwrap();

        drop(tx);

        let mut stream = rx.filter_map(|i| if i > 0 { Some(i) } else { None });

        let mut sum = 0;
        while let Ok(value) = stream.next().await {
            assert!(value > 0);
            sum += value;
        }

        assert_eq!(sum, 9);
    }
}
