use tokio::task::JoinHandle;
use crate::select::ActorSelect;

pub trait ActorContext: Default + Sized + Unpin + 'static {
// pub trait ActorContext: Sized {

}

pub struct Addr<A: Actor> {
    tx: A,
}

pub struct Context<A>
    where
        A: Actor<Context = Context<A>>,
{
    // parts: ContextParts<A>,
    // mb: Option<Mailbox<A>>,
    ll: Option<A>,
}

impl<A> Context<A> where A: Actor<Context=Context<A>>, {
    pub fn new() -> Self {
        Context {ll: None}
    }
}

impl<A> ActorContext for Context<A> where
    A: Actor<Context=Context<A>>, {}

impl<A> Default for Context<A> where
    A: Actor<Context=Context<A>>, {
    fn default() -> Self {
        Self {
            ll: None,
        }
    }
}

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


    // async fn select(&mut self, ctx: &mut Self::Context) -> Box<dyn select::ActorSelect<Self>>;
}

pub mod select {
    use crate::{Actor, Handler, Message, SelectResult};

    #[async_trait::async_trait]
    pub trait ActorSelect<Z: Actor> {
        async fn select(&mut self, ctx: &mut Z::Context, actor: &mut Z) -> SelectResult;
    }

    pub type MpscReceiver<T> = tokio::sync::mpsc::Receiver<T>;

    #[async_trait::async_trait]
    impl <Z, A> ActorSelect<Z> for MpscReceiver<A>
        where Z: Handler<A> + Send,
              A: Message + Send,
    {
        async fn select(&mut self, ctx: &mut Z::Context, actor: &mut Z) -> SelectResult {
            tokio::select! {
                Some(msg) = self.recv() => {
                    actor.handle(msg, ctx).await?;
                }
            }
            Ok(())
        }
    }

    #[async_trait::async_trait]
    impl <Z, A, B> ActorSelect<Z> for (MpscReceiver<A>, MpscReceiver<B>)
        where Z: Handler<A> + Handler<B> + Send,
              A: Message + Send, B: Message + Send,
    {
        async fn select(&mut self, ctx: &mut Z::Context, actor: &mut Z) -> SelectResult {
            tokio::select! {
                Some(msg) = self.0.recv() => {
                    actor.handle(msg, ctx).await?;
                }
                Some(msg) = self.1.recv() => {
                    actor.handle(msg, ctx).await?;
                }
            }
            Ok(())
        }
    }

    #[async_trait::async_trait]
    impl <Z, A, B, C> ActorSelect<Z> for (MpscReceiver<A>, MpscReceiver<B>, MpscReceiver<C>)
        where Z: Handler<A> + Handler<B> + Handler<C> + Send,
              A: Message + Send, B: Message + Send, C: Message + Send,
    {
        async fn select(&mut self, ctx: &mut Z::Context, actor: &mut Z) -> SelectResult {
            tokio::select! {
                Some(msg) = self.0.recv() => {
                    actor.handle(msg, ctx).await?;
                }
                Some(msg) = self.1.recv() => {
                    actor.handle(msg, ctx).await?;
                }
                Some(msg) = self.2.recv() => {
                    actor.handle(msg, ctx).await?;
                }
            }
            Ok(())
        }
    }
}

pub trait Message { }

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
pub type SelectResult = Result<(), Box<dyn std::error::Error>>;

pub struct System {
    name: String,
}

impl System {
    pub fn global() -> Self {
        System { name: "Global".to_string() }
    }
}

impl System {
    pub async fn run<A, S>(&self, mut actor: A, mut select: S) -> JoinHandle<()>
        where
            A: Actor + Send,
            S: ActorSelect<A> + Send + 'static
    {
        let system_name = self.name.clone();
        let process_name = std::any::type_name::<A>().to_owned();

        let handle = tokio::spawn(async move {
            tracing::debug!("The system: {:?} spawned process: {:?}", system_name, process_name);

            let mut ctx = A::default_context();

            loop {
                tracing::debug!("iteration of the process: {process_name:?}");
                let result = select.select(&mut ctx, &mut actor).await;
                tracing::debug!("{process_name:?} result: {result:?}");
            }
        });
        handle
    }

    pub async fn run_fn<A, F, S>(&self, f: F, mut select: S) -> JoinHandle<()>
        where
            A: Actor + Send,
            F: FnOnce(&mut A::Context) -> A,
            S: ActorSelect<A> + Send + 'static
    {
        let mut ctx = A::default_context();
        let mut actor = f(&mut ctx);

        let process_name = std::any::type_name::<A>().to_owned();
        let handle = tokio::spawn(async move {
            tracing::debug!("Spawn process: {process_name:?}");

            loop {
                tracing::debug!("iteration of the process: {process_name:?}");
                let result = select.select(&mut ctx, &mut actor).await;
                tracing::debug!("{process_name:?} result: {result:?}");
            }
        });
        handle
    }
}
