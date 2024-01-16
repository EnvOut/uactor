use crate::context::ActorContext;
use crate::message::Message;

#[allow(unused_variables)]
pub trait Actor: Sized + Unpin + 'static {
    /// Actor execution context type
    type Context: ActorContext + Send;

    // fn started(&mut self, ctx: &mut Self::Context) {}
    // fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
    //     Running::Stop
    // }
    // fn stopped(&mut self, ctx: &mut Self::Context) {}

    fn default_context() -> Self::Context{
        let ctx: Self::Context = Default::default();
        ctx
    }
}

#[async_trait::async_trait]
pub trait Handler<M>
    where
        Self: Actor,
        M: Message,
{
    /// This method is called for every message received by this actor.
    async fn handle(&mut self, msg: M, ctx: &mut <Self as Actor>::Context) -> HandleResult;
}

pub type HandleResult = Result<(), Box<dyn std::error::Error>>;