use crate::data::datasource::{DataSource, DataSourceResult};

pub struct DataSourceMap<M, D, F>
where
    F: Fn(D::Item) -> M + Send,
    D: DataSource + Send,
{
    datasource: D,
    map_fn: F,
}

impl<M, D, F> DataSourceMapExt<M, D, F> for DataSourceMap<M, D, F>
where
    F: Fn(D::Item) -> M + Send,
    D: DataSource + Send,
{
    fn map(self, map_fn: F) -> DataSourceMap<M, D, F> {
        DataSourceMap {
            datasource: self.datasource,
            map_fn,
        }
    }
}

impl<M, D, F> DataSource for DataSourceMap<M, D, F>
where
    F: Fn(D::Item) -> M + Send,
    D: DataSource + Send,
{
    type Item = M;

    async fn next(&mut self) -> DataSourceResult<Self::Item> {
        let value = DataSource::next(&mut self.datasource).await?;
        Ok((self.map_fn)(value))
    }
}

pub trait DataSourceMapExt<M, D, F>
where
    F: Fn(D::Item) -> M + Send,
    D: DataSource + Send,
{
    fn map(self, map_fn: F) -> DataSourceMap<M, D, F>;
}

impl<M, D, F> DataSourceMapExt<M, D, F> for D
where
    F: Fn(D::Item) -> M + Send,
    D: DataSource + Send,
{
    fn map(self, map_fn: F) -> DataSourceMap<M, D, F> {
        DataSourceMap {
            datasource: self,
            map_fn,
        }
    }
}