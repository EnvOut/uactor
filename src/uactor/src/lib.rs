use tokio::sync::{broadcast, mpsc, oneshot, watch};
use tokio::task::JoinHandle;
use crate::select::{ActorSelect};

pub trait ActorContext: Default + Sized + Unpin + 'static { }

pub struct Context<A>
    where
        A: Actor<Context = Context<A>>,
{
    // parts: ContextParts<A>,
    // mb: Option<Mailbox<A>>,
    #[allow(dead_code)]
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
}

pub mod select {
    use std::future::pending;
    use tokio::sync::mpsc;
    use crate::{Actor, DataSource, Handler, Message, SelectResult};

    #[async_trait::async_trait]
    pub trait ActorSelect<Z: Actor> {
        async fn select(&mut self, ctx: &mut Z::Context, actor: &mut Z) -> SelectResult;
    }

    pub type MpscReceiver<T> = mpsc::Receiver<T>;

    #[async_trait::async_trait]
    impl <Z: Actor> ActorSelect<Z> for (){
        async fn select(&mut self, _: &mut Z::Context, _: &mut Z) -> SelectResult {
            let never = pending::<()>();
            never.await;
            Ok(())
        }
    }

    #[async_trait::async_trait]
    impl<Z, S1> ActorSelect<Z> for S1
        where
            S1::Item: Message + Send,
            S1: DataSource + Send,
            Z: Handler<S1::Item> + Send,
    {
        async fn select(&mut self, ctx: &mut Z::Context, actor: &mut Z) -> SelectResult {
            tokio::select! {
                Ok(msg) = self.next() => {
                    let _ = actor.handle(msg, ctx).await;
                }
            }
            Ok(())
        }
    }

    #[async_trait::async_trait]
    impl<Z, S1, S2> ActorSelect<Z> for (S1, S2)
        where
            S1::Item: Message + Send, S2::Item: Message + Send,
            S1: DataSource + Send, S2: DataSource + Send,
            Z: Handler<S1::Item> + Handler<S2::Item> + Send,
    {
        async fn select(&mut self, ctx: &mut Z::Context, actor: &mut Z) -> SelectResult {
            tokio::select! {
                Ok(msg) = self.0.next() => {
                    let _ = actor.handle(msg, ctx).await;
                }
                Ok(msg) = self.1.next() => {
                    let _ = actor.handle(msg, ctx).await?;
                }
            }
            Ok(())
        }
    }

    #[async_trait::async_trait]
    impl<Z, S1, S2, S3> ActorSelect<Z> for (S1, S2, S3)
        where
            S1::Item: Message + Send, S2::Item: Message + Send, S3::Item: Message + Send + Send,
            S1: DataSource + Send, S2: DataSource + Send, S3: DataSource + Send,
            Z: Handler<S1::Item> + Handler<S2::Item> + Handler<S3::Item> + Send,
    {
        async fn select(&mut self, ctx: &mut Z::Context, actor: &mut Z) -> SelectResult {
            tokio::select! {
                Ok(msg) = self.0.next() => {
                    let _ = actor.handle(msg, ctx).await?;
                }
                Ok(msg) = self.1.next() => {
                    let _ = actor.handle(msg, ctx).await?;
                }
                Ok(msg) = self.2.next() => {
                    let _ = actor.handle(msg, ctx).await?;
                }
            }
            Ok(())
        }
    }

    #[async_trait::async_trait]
    impl<Z, S1, S2, S3, S4> ActorSelect<Z> for (S1, S2, S3, S4)
        where
            S1::Item: Message + Send, S2::Item: Message + Send, S3::Item: Message + Send, S4::Item: Message + Send,
            S1: DataSource + Send, S2: DataSource + Send, S3: DataSource + Send, S4: DataSource + Send,
            Z: Handler<S1::Item> + Handler<S2::Item> + Handler<S3::Item> + Handler<S4::Item> + Send,
    {
        async fn select(&mut self, ctx: &mut Z::Context, actor: &mut Z) -> SelectResult {
            tokio::select! {
                Ok(msg) = self.0.next() => {
                    let _ = actor.handle(msg, ctx).await?;
                }
                Ok(msg) = self.1.next() => {
                    let _ = actor.handle(msg, ctx).await?;
                }
                Ok(msg) = self.2.next() => {
                    let _ = actor.handle(msg, ctx).await?;
                }
                Ok(msg) = self.3.next() => {
                    let _ = actor.handle(msg, ctx).await?;
                }
            }
            Ok(())
        }
    }

    #[async_trait::async_trait]
    impl<Z, S1, S2, S3, S4, S5> ActorSelect<Z> for (S1, S2, S3, S4, S5)
        where
            S1::Item: Message + Send, S2::Item: Message + Send, S3::Item: Message + Send, S4::Item: Message + Send, S5::Item: Message + Send,
            S1: DataSource + Send, S2: DataSource + Send, S3: DataSource + Send, S4: DataSource + Send, S5: DataSource + Send,
            Z: Handler<S1::Item> + Handler<S2::Item> + Handler<S3::Item> + Handler<S4::Item> + Handler<S5::Item> + Send,
    {
        async fn select(&mut self, ctx: &mut Z::Context, actor: &mut Z) -> SelectResult {
            tokio::select! {
                Ok(msg) = self.0.next() => {
                    let _ = actor.handle(msg, ctx).await?;
                }
                Ok(msg) = self.1.next() => {
                    let _ = actor.handle(msg, ctx).await?;
                }
                Ok(msg) = self.2.next() => {
                    let _ = actor.handle(msg, ctx).await?;
                }
                Ok(msg) = self.3.next() => {
                    let _ = actor.handle(msg, ctx).await?;
                }
                Ok(msg) = self.4.next() => {
                   let _ = actor.handle(msg, ctx).await?;
                }
            }
            Ok(())
        }
    }
}

pub type DataSourceResult<T> = Result<T, DataSourceErrors>;

#[derive(thiserror::Error, Debug)]
pub enum DataSourceErrors {
    #[error("Channel closed")]
    ChannelClosed,

    #[error("Channel lagged by {0}")]
    ChannelLagged(u64)
}

impl From<broadcast::error::RecvError> for DataSourceErrors {
    fn from(err: broadcast::error::RecvError) -> Self {
        use broadcast::error::RecvError;

        match err {
            RecvError::Closed => {Self::ChannelClosed}
            RecvError::Lagged(number_skipped_messages) => {Self::ChannelLagged(number_skipped_messages)}
        }
    }
}

#[async_trait::async_trait]
pub trait DataSource {
    type Item;
    async fn next(&mut self) -> DataSourceResult<Self::Item>;
}

#[async_trait::async_trait]
impl<T> DataSource for mpsc::Receiver<T> where T: Send {
    type Item = T;

    async fn next(&mut self) -> DataSourceResult<Self::Item> {
        if let Some(value) = self.recv().await {
            Ok(value)
        } else {
            Err(DataSourceErrors::ChannelClosed)
        }
    }
}

#[async_trait::async_trait]
impl<T> DataSource for mpsc::UnboundedReceiver<T> where T: Send {
    type Item = T;

    async fn next(&mut self) -> DataSourceResult<Self::Item> {
        if let Some(value) = self.recv().await {
            Ok(value)
        } else {
            Err(DataSourceErrors::ChannelClosed)
        }
    }
}

#[async_trait::async_trait]
impl<T> DataSource for watch::Receiver<T> where T: Clone + Send + Sync {
    type Item = T;

    async fn next(&mut self) -> DataSourceResult<Self::Item> {
        let _ = self.changed().await.map_err(|_| DataSourceErrors::ChannelClosed)?;
        let value = self.borrow().clone();
        Ok(value)
    }
}

#[async_trait::async_trait]
impl<T> DataSource for broadcast::Receiver<T>  where T: Clone + Send + Sync {
    type Item = T;

    async fn next(&mut self) -> DataSourceResult<Self::Item> {
        self.recv().await.map_err(|err| DataSourceErrors::from(err))
    }
}

#[async_trait::async_trait]
impl<T> DataSource for oneshot::Receiver<T> where T: Send {
    type Item = T;

    async fn next(&mut self) -> DataSourceResult<Self::Item> {
        self.await.map_err(|_| DataSourceErrors::ChannelClosed)
    }
}

pub trait Message { }

impl<A, B> Message for Result<A, B> where A: Message, B: Message {}


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
