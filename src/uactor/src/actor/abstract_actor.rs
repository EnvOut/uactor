use crate::actor::context::ActorContext;
use crate::actor::message::Message;
use std::future::Future;
use std::sync::Arc;
use crate::actor::select::{ActorSelect, SelectError, SelectResult};

pub trait State: std::any::Any + Send + 'static {}
impl<T: std::any::Any + Send + 'static> State for T {}

use crate::dependency_injection::Inject;

#[allow(unused_variables)]
pub trait Actor: Sized + Unpin + 'static {
    /// Actor execution context type
    type Context: ActorContext + Send;

    type RouteMessage: Message + Send;

    type Inject: Inject + Sized;

    type State: Default + Send + Sync + Clone;

    fn create_state(&mut self) -> Arc<Self::State> {
        Arc::new(Default::default())
    }

    fn on_start(
        &mut self,
        inject: &mut Self::Inject,
        ctx: &mut Self::Context,
    ) -> impl Future<Output = ()> + Send {
        async {}
    }

    fn select_message<S>(
        &mut self,
        ctx: &mut Self::Context,
        aggregator: &mut S
    ) -> impl Future<Output = SelectResult<Self>> + Send
    where
        S: ActorSelect<Self> + Send + 'static, Self: Send
    {
        aggregator.select()
    }

    #[inline]
    fn on_select_error(&mut self, err: SelectError, ctx: &mut Self::Context) -> impl Future<Output=()> + Send {
        tracing::error!("Received error on datasource select: {:?}", err);
        ctx.kill();
        async {}
    }

    fn on_error(
        &mut self,
        ctx: &mut Self::Context,
        error: HandleError,
    ) -> impl Future<Output = ()> + Send {
        async move {
            tracing::error!("Actor error: {:?}", error);
        }
    }

    #[inline]
    fn on_die(self, ctx: &mut Self::Context, state: &Self::State) -> impl Future<Output = ()> + Send {
        async {}
    }
}

pub trait Handler<M>
where
    Self: Actor,
    M: Message,
{
    /// This method is called for every message received by this actor.
    fn handle(
        &mut self,
        inject: &mut Self::Inject,
        msg: M,
        ctx: &mut Self::Context,
        state: &Self::State,
    ) -> impl Future<Output = HandleResult> + Send;
}

pub type HandleError = Box<dyn std::error::Error + Send + Sync>;

pub type HandleResult = Result<(), HandleError>;

#[cfg(not(feature = "async_sender"))]
pub trait MessageSender<M>
where
    M: Message,
{
    fn send(&self, msg: M) -> crate::data::data_publisher::DataPublisherResult;
    fn ask<A>(
        &self,
        f: impl FnOnce(tokio::sync::oneshot::Sender<A>) -> M,
    ) -> impl Future<Output = Result<A, crate::data::data_publisher::DataPublisherErrors>>;
}

#[cfg(feature = "async_sender")]
pub trait MessageSender<M>
where
    M: Message,
{
    fn send(&self, msg: M) -> impl std::future::Future<Output = crate::data::data_publisher::DataPublisherResult> + Send;
    fn ask<A>(
        &self,
        f: impl FnOnce(tokio::sync::oneshot::Sender<A>) -> M,
    ) -> impl std::future::Future<Output = Result<A, crate::data::data_publisher::DataPublisherErrors>> + Send;
}
