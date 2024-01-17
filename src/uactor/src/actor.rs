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

#[macro_export]
macro_rules! spawn_with_ref {
    ($S: ident, $ActorType: ident, $ActorInstance: ident, $ActorMessageName: ident; $($Message: ident),*) => {{
        // $(impl Message for $T {})*
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<$ActorMessageName>();
        let handle = $S.run($ActorInstance, (rx)).await;

        pub enum $ActorMessageName {
            $($Message($Message)),*
        }
        impl uactor::message::Message for $ActorMessageName { }

        #[async_trait::async_trait]
        impl uactor::actor::Handler<$ActorMessageName> for $ActorType {
            async fn handle(&mut self, msg: $ActorMessageName, ctx: &mut <Self as Actor>::Context) -> HandleResult {
                match msg {
                    $(
                    $ActorMessageName::$Message(m) => {
                        self.handle(m, ctx).await?;
                    }
                    ),*
                }
                Ok(())
            }
        }

        paste::paste! {
            pub struct [<$ActorType Ref>]<T>(T) where T: uactor::data_publisher::DataPublisher<Item=$ActorMessageName>;
            impl<T> [<$ActorType Ref>]<T> where T: uactor::data_publisher::DataPublisher<Item=$ActorMessageName> {
                $(
                pub async fn [<send_ $Message:snake>](&mut self, msg: $Message) -> uactor::data_publisher::DataPublisherResult {
                    self.0.publish($ActorMessageName::$Message(msg)).await
                }
                pub async fn [<send_and_wait_ $Message:snake>]<A>(&mut self, f: impl FnOnce(tokio::sync::oneshot::Sender<A>) -> $Message) -> Result<A, uactor::data_publisher::DataPublisherErrors>
                    where A: uactor::message::Message
                {
                    let (tx, rx) = tokio::sync::oneshot::channel::<A>();
                    let message = f(tx);
                    self.0.publish($ActorMessageName::$Message(message)).await?;
                    rx.await.map_err(|err| uactor::data_publisher::DataPublisherErrors::from(err))
                }
                )*

            }

            let actor1_ref = [<$ActorType Ref>](tx);
        }

        (actor1_ref, handle)
    }};
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