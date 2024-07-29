use crate::context::{ActorContext, Context};
use crate::message::Message;

pub trait State: std::any::Any + Send + 'static {}
impl<T: std::any::Any + Send + 'static> State for T {}
pub type ActorPreStartResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

use crate::di::Inject;

#[allow(unused_variables)]
pub trait Actor: Sized + Unpin + 'static {
    /// Actor execution context type
    type Context: ActorContext + Send;

    type Inject: Inject + Sized;

    async fn pre_start(
        &mut self,
        state: &mut Self::Inject,
        ctx: &mut Self::Context,
    ) -> ActorPreStartResult<()> {
        Ok(())
    }
}
#[macro_export]
macro_rules! spawn_with_ref {
    ($S: ident, $ActorName: ident, $ActorInstance: ident: $ActorType: ident, $($Timeout: ident),*) => {{
        uactor::paste! {
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<[<$ActorType Msg>]>();
            let (name, handle): (std::sync::Arc<str>, tokio::task::JoinHandle<()>) = $S.init_actor($ActorInstance, Some($ActorName.to_owned()), ($($Timeout,)* rx));
            let actor_ref = [<$ActorType Ref>]::new(name, tx);
            $S.insert_actor(actor_ref.name(), uactor::data_publisher::TryClone::try_clone(&actor_ref)?);
            (actor_ref, handle)
        }
    }};

    ($S: ident, $ActorInstance: ident: $ActorType: ident, $($Timeout: ident),*) => {{
        let actor_name: std::sync::Arc<str> = stringify!($ActorInstance).into();

        uactor::paste! {
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<[<$ActorType Msg>]>();
            let (name, handle): (std::sync::Arc<str>, tokio::task::JoinHandle<()>) = $S.init_actor($ActorInstance, Some(actor_name), ($($Timeout,)* rx));
            let actor_ref = [<$ActorType Ref>]::new(name, tx);
            $S.insert_actor(actor_ref.name(), uactor::data_publisher::TryClone::try_clone(&actor_ref)?);
            (actor_ref, handle)
        }
    }};

    ($S: ident, $ActorName: ident, $ActorInstance: ident: $ActorType: ident) => {{
        let actor_name: std::sync::Arc<str> = stringify!($ActorInstance).into();
        uactor::paste! {
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<[<$ActorType Msg>]>();
            let (name, handle): (std::sync::Arc<str>, tokio::task::JoinHandle<()>) = $S.init_actor($ActorInstance, Some(actor_name), (rx));
            let actor_ref = [<$ActorType Ref>]::new(name, tx);
            $S.insert_actor(actor_ref.name(), uactor::data_publisher::TryClone::try_clone(&actor_ref)?);
            (actor_ref, handle)
        }
    }};

    ($S: ident, $ActorInstance: ident: $ActorType: ident) => {{
        let actor_name: std::sync::Arc<str> = stringify!($ActorInstance).into();
        uactor::paste! {
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<[<$ActorType Msg>]>();
            let (name, handle): (std::sync::Arc<str>, tokio::task::JoinHandle<()>) = $S.init_actor($ActorInstance, Some(actor_name), (rx));
            let actor_ref = [<$ActorType Ref>]::new(name, tx);
            $S.insert_actor(actor_ref.name(), uactor::data_publisher::TryClone::try_clone(&actor_ref)?);
            (actor_ref, handle)
        }
    }};
}

#[cfg(not(feature = "async_sender"))]
pub trait MessageSender<M>
where
    M: Message,
{
    fn send(&self, msg: M) -> crate::data_publisher::DataPublisherResult;
    async fn ask<A>(
        &self,
        f: impl FnOnce(tokio::sync::oneshot::Sender<A>) -> M,
    ) -> Result<A, crate::data_publisher::DataPublisherErrors>;
}

#[cfg(feature = "async_sender")]
pub trait MessageSender<M>
where
    M: Message,
{
    async fn send(&self, msg: M) -> crate::data_publisher::DataPublisherResult;
    async fn ask<A>(
        &self,
        f: impl FnOnce(tokio::sync::oneshot::Sender<A>) -> M,
    ) -> Result<A, crate::data_publisher::DataPublisherErrors>;
}

pub trait NamedActorRef {
    fn name() -> &'static str;
}

/// Example:
/// ```
///# use std::future::Future;
/// use uactor::actor::{Actor, HandleResult};
/// use uactor::context::Context;
/// use uactor::system::System;
/// let mut system = System::global().build();
/// pub struct Actor1;
/// impl Actor for Actor1 { type Context = Context; type Inject = (); }
/// let actor1 = Actor1;
/// use uactor::message::{Message};
/// use uactor::message_impl;
/// pub struct Ping;
/// impl Message for Ping { fn static_name() -> &'static str { "Ping" } }
/// impl uactor::actor::Handler<Ping> for Actor1 { async fn handle(&mut self, inject: &mut Self::Inject, msg: Ping, ctx: &mut Self::Context) -> HandleResult { todo!() }  }
/// uactor::generate_actor_ref!(Actor1, { });
/// ```
/// let (mut actor1_ref, handle) = uactor::spawn_with_ref!(system, actor1: Actor1);
#[macro_export]
macro_rules! generate_actor_ref {
    ($ActorType: ident, { $($Message: ident),* }) => {
        uactor::paste! {
            pub enum [<$ActorType Msg>] {
                $($Message($Message)),*
            }

            impl NamedActorRef for [<$ActorType Msg>] {
                fn static_name() -> &'static str {
                    stringify!([<$ActorType Msg>])
                }
            }

            impl uactor::message::Message for [<$ActorType Msg>] {
                fn name(&self) -> String {
                    match self {
                    $(
                        Self::$Message(m) => m.name(),
                    )*
                        _ => Self::static_name().to_owned(),
                    }
                }
            }

            impl uactor::actor::Handler<[<$ActorType Msg>]> for $ActorType {
                async fn handle(&mut self, inject: &mut  <Self as uactor::actor::Actor>::Inject, msg: [<$ActorType Msg>], ctx: &mut <Self as uactor::actor::Actor>::Context) -> uactor::actor::HandleResult {
                    match msg {
                        $(
                        [<$ActorType Msg>]::$Message(m) => {
                            self.handle(inject, m, ctx).await?;
                        }
                        ),*
                    }
                    Ok(())
                }
            }

            pub struct [<$ActorType Ref>]<T> where T: uactor::data_publisher::DataPublisher<Item=[<$ActorType Msg>]> + Clone {
                name: std::sync::Arc<str>,
                sender: T,
            }

            impl<T> uactor::data_publisher::TryClone for [<$ActorType Ref>]<T> where T: uactor::data_publisher::DataPublisher<Item=[<$ActorType Msg>]> + Clone {
                fn try_clone(&self) -> Result<Self, uactor::data_publisher::TryCloneError> {
                    self.sender.try_clone().map(|sender| Self { name: self.name.clone(), sender })
                }
            }

            impl Clone for [<$ActorType Ref>]<tokio::sync::mpsc::UnboundedSender<[<$ActorType Msg>]>> {
                fn clone(&self) -> Self {
                    [<$ActorType Ref>]::new(self.name.clone(), self.sender.clone())
                }
            }

            $(
                #[cfg(not(feature = "async_sender"))]
                impl <T>uactor::actor::MessageSender<$Message> for [<$ActorType Ref>]<T> where T: uactor::data_publisher::DataPublisher<Item=[<$ActorType Msg>]> + Clone {
                    fn send(&self, msg: $Message) -> uactor::data_publisher::DataPublisherResult {
                        self.sender.publish([<$ActorType Msg>]::$Message(msg))
                    }

                    async fn ask<A>(&self, f: impl FnOnce(tokio::sync::oneshot::Sender<A>) -> $Message) -> Result<A, uactor::data_publisher::DataPublisherErrors> {
                        let (tx, rx) = tokio::sync::oneshot::channel::<A>();
                        let message = f(tx);
                        self.sender.publish([<$ActorType Msg>]::$Message(message))?;
                        rx.await.map_err(uactor::data_publisher::DataPublisherErrors::from)
                    }
                }

                #[cfg(feature = "async_sender")]
                impl <T>uactor::actor::MessageSender<$Message> for [<$ActorType Ref>]<T> where T: uactor::data_publisher::DataPublisher<Item=[<$ActorType Msg>]> + Clone {
                    async fn send(&self, msg: $Message) -> uactor::data_publisher::DataPublisherResult {
                        self.sender.publish([<$ActorType Msg>]::$Message(msg)).await
                    }

                    async fn ask<A>(&self, f: impl FnOnce(tokio::sync::oneshot::Sender<A>) -> $Message) -> Result<A, uactor::data_publisher::DataPublisherErrors> {
                        let (tx, rx) = tokio::sync::oneshot::channel::<A>();
                        let message = f(tx);
                        self.sender.publish([<$ActorType Msg>]::$Message(message))?;
                        rx.await.map_err(uactor::data_publisher::DataPublisherErrors::from)
                    }
                }
            )*

            impl<T> [<$ActorType Ref>]<T> where T: uactor::data_publisher::DataPublisher<Item=[<$ActorType Msg>]> + Clone {
                pub fn new(name: std::sync::Arc<str>, sender: T) -> Self {
                    let name = std::sync::Arc::from(name);
                    Self { name, sender }
                }

                pub fn name(&self) -> std::sync::Arc<str> {
                    self.name.clone()
                }
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
    fn handle(
        &mut self,
        inject: &mut Self::Inject,
        msg: M,
        ctx: &mut Self::Context,
    ) -> impl std::future::Future<Output = HandleResult> + Send;
}

pub type HandleResult = Result<(), Box<dyn std::error::Error>>;
