use crate::data::datasource::{DataSource, DataSourceResult};

pub struct DataSourceFilter<D, F>
where
    F: Fn(&D::Item) -> bool + Send,
    D: DataSource + Send,
{
    datasource: D,
    filter_fn: F,
}

impl<D, F> DataSourceFilterExt<D, F> for DataSourceFilter<D, F>
where
    F: Fn(&D::Item) -> bool + Send,
    D: DataSource + Send,
{
    fn map(self, map_fn: F) -> DataSourceFilter<D, F> {
        DataSourceFilter {
            datasource: self.datasource,
            filter_fn: map_fn,
        }
    }
}

impl<D, F> DataSource for DataSourceFilter<D, F>
where
    F: Fn(&D::Item) -> bool + Send,
    D: DataSource + Send,
{
    type Item = D::Item;

    async fn next(&mut self) -> DataSourceResult<Self::Item> {
        loop {
            let value = DataSource::next(&mut self.datasource).await?;
            if (self.filter_fn)(&value) {
                return Ok(value);
            } else {
                continue;
            }
        }
    }
}

pub trait DataSourceFilterExt<D, F>
where
    F: Fn(&D::Item) -> bool + Send,
    D: DataSource + Send,
{
    #[allow(dead_code)]
    fn map(self, map_fn: F) -> DataSourceFilter<D, F>;
}

impl<D, F> DataSourceFilterExt<D, F> for D
where
    F: Fn(&D::Item) -> bool + Send,
    D: DataSource + Send,
{
    fn map(self, map_fn: F) -> DataSourceFilter<D, F> {
        DataSourceFilter {
            datasource: self,
            filter_fn: map_fn,
        }
    }
}