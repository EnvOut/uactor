use crate::context::{ActorContext, Context};
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
    ($S: ident, $ActorInstance: ident: $ActorType: ident, $($Timeout: ident),*) => {{
        let actor_name: String = stringify!($ActorType).to_owned();
        uactor::paste! {
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<[<$ActorType Msg>]>();
            let handle = $S.run($ActorInstance, Some(actor_name), ($($Timeout,)* rx));
            let actor_ref = [<$ActorType Ref>]::new(tx);
            (actor_ref, handle)
        }
    }};

    ($S: ident, $ActorInstance: ident: $ActorType: ident) => {{
        let actor_name: String = stringify!($ActorType).to_owned();
        uactor::paste! {
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<[<$ActorType Msg>]>();
            let handle = $S.run($ActorInstance, Some(actor_name), (rx));

            let actor_ref = [<$ActorType Ref>]::new(tx);
            (actor_ref, handle)
        }
    }};
}

/// let (mut actor1_ref, _) = uactor::spawn_with_ref!(system, actor1: Actor1);
#[macro_export]
macro_rules! generate_actor_ref {
    ($ActorType: ident, { $($Message: ident),* }) => {
        uactor::paste! {
            pub enum [<$ActorType Msg>] {
                $($Message($Message)),*
            }
            impl uactor::message::Message for [<$ActorType Msg>] { }


            impl uactor::actor::Handler<[<$ActorType Msg>]> for $ActorType {
                async fn handle(&mut self, msg: [<$ActorType Msg>], ctx: &mut <Self as uactor::actor::Actor>::Context) -> uactor::actor::HandleResult {
                    match msg {
                        $(
                        [<$ActorType Msg>]::$Message(m) => {
                            self.handle(m, ctx).await?;
                        }
                        ),*
                    }
                    Ok(())
                }
            }

            pub struct [<$ActorType Ref>]<T>(T) where T: uactor::data_publisher::DataPublisher<Item=[<$ActorType Msg>]> + Clone;

            impl<T> uactor::data_publisher::TryClone for [<$ActorType Ref>]<T> where T: uactor::data_publisher::DataPublisher<Item=[<$ActorType Msg>]> + Clone {
                fn try_clone(&self) -> Result<Self, uactor::data_publisher::TryCloneError> {
                    self.0.try_clone().map(|publisher| Self(publisher))
                }
            }

            impl<T> [<$ActorType Ref>]<T> where T: uactor::data_publisher::DataPublisher<Item=[<$ActorType Msg>]> + Clone {
                pub fn new(channel: T) -> Self {
                    Self(channel)
                }

                $(
                #[cfg(not(feature = "async_sender"))]
                pub fn [<send_ $Message:snake>](&self, msg: $Message) -> uactor::data_publisher::DataPublisherResult {
                    self.0.publish([<$ActorType Msg>]::$Message(msg))
                }
                #[cfg(not(feature = "async_sender"))]
                pub async fn [<ask_ $Message:snake>]<A>(&self, f: impl FnOnce(tokio::sync::oneshot::Sender<A>) -> $Message) -> Result<A, uactor::data_publisher::DataPublisherErrors> {
                    let (tx, rx) = tokio::sync::oneshot::channel::<A>();
                    let message = f(tx);
                    self.0.publish([<$ActorType Msg>]::$Message(message))?;
                    rx.await.map_err(|err| uactor::data_publisher::DataPublisherErrors::from(err))
                }

                #[cfg(feature = "async_sender")]
                pub async fn [<send_ $Message:snake>](&self, msg: $Message) -> uactor::data_publisher::DataPublisherResult {
                    self.0.publish([<$ActorType Msg>]::$Message(msg)).await
                }
                #[cfg(feature = "async_sender")]
                pub async fn [<ask_ $Message:snake>]<A>(&self, f: impl FnOnce(tokio::sync::oneshot::Sender<A>) -> $Message) -> Result<A, uactor::data_publisher::DataPublisherErrors> {
                    let (tx, rx) = tokio::sync::oneshot::channel::<A>();
                    let message = f(tx);
                    self.0.publish([<$ActorType Msg>]::$Message(message)).await?;
                    rx.await.map_err(|err| uactor::data_publisher::DataPublisherErrors::from(err))
                }

                // pub async fn [<send_with_reply_ $Message:snake>]<A>(&mut self, f: impl FnOnce(uactor::data_publisher::DataPublisher<Item=A>) -> $Message) -> Result<A, uactor::data_publisher::DataPublisherErrors>
                //     where A: uactor::message::Message
                // {
                //     let (tx, rx) = tokio::sync::oneshot::channel::<A>();
                //     let message = f(tx);
                //     self.0.publish([<$ActorType Msg>]::$Message(message)).await?;
                //     rx.await.map_err(|err| uactor::data_publisher::DataPublisherErrors::from(err))
                // }
                )*

            }
        }

    };
}


pub trait Handler<M>
    where
        Self: Actor,
        M: Message,
{
    /// This method is called for every message received by this actor.
    fn handle(&mut self, msg: M, ctx: &mut Context) -> impl std::future::Future<Output = HandleResult> + Send;
}

pub type HandleResult = Result<(), Box<dyn std::error::Error>>;