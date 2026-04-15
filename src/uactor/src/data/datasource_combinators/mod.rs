use crate::actor::message::Message;
use crate::data::datasource::DataSource;
use crate::data::datasource_combinators::timeout_impl::TimeoutDecorator;
use std::time::Duration;

mod filter_impl;
mod filter_map_impl;
mod map_impl;
mod merge_impl;
mod timeout_impl;

pub use filter_impl::DataSourceFilterExt;
pub use filter_map_impl::DataSourceFilterMapExt;
pub use map_impl::DataSourceMapExt;
pub use merge_impl::DataSourceMergeExt;

pub fn timeout<M, D>(duration: Duration, datasource: D) -> TimeoutDecorator<M, D>
where
    M: Message + Send,
    D: DataSource<Item = M> + Send + Sync,
{
    TimeoutDecorator::new(duration, datasource)
}
