use crate::actor::context::ActorContext;
use crate::actor::message::Message;
use std::future::Future;
use std::sync::Arc;

pub trait State: std::any::Any + Send + 'static {}
impl<T: std::any::Any + Send + 'static> State for T {}
pub type ActorPreStartResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

use crate::dependency_injection::Inject;

#[allow(unused_variables)]
pub trait Actor: Sized + Unpin + 'static {
    /// Actor execution context type
    type Context: ActorContext + Send;

    type Inject: Inject + Sized;

    type State: Default + Send + Sync;

    fn create_state(&mut self) -> Arc<Self::State> {
        Arc::new(Default::default())
    }

    fn pre_start(
        &mut self,
        inject: &mut Self::Inject,
        ctx: &mut Self::Context,
    ) -> impl Future<Output = ActorPreStartResult<()>> + Send {
        async { Ok(()) }
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

pub type HandleResult = Result<(), Box<dyn std::error::Error>>;


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
    async fn send(&self, msg: M) -> crate::data::data_publisher::DataPublisherResult;
    async fn ask<A>(
        &self,
        f: impl FnOnce(tokio::sync::oneshot::Sender<A>) -> M,
    ) -> Result<A, crate::data::data_publisher::DataPublisherErrors>;
}

pub trait NamedActorRef {
    fn static_name() -> &'static str;
}