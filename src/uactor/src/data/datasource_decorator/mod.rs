use crate::actor::message::Message;
use crate::data::datasource::DataSource;
use crate::data::datasource_decorator::timeout_impl::TimeoutDecorator;
use std::time::Duration;

mod timeout_impl;
mod map_impl;
mod filter_impl;
pub use map_impl::DataSourceMapExt;

pub fn timeout<M, D>(duration: Duration, datasource: D) -> TimeoutDecorator<M, D>
where
    M: Message + Send,
    D: DataSource<Item = M> + Send + Sync,
{
    TimeoutDecorator::new(duration, datasource)
}
