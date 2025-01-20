/// Example:
/// ```
///# use std::future::Future;
/// use uactor::actor::abstract_actor::{Actor, HandleResult};
/// use uactor::actor::context::Context;
/// use uactor::system::System;
/// let mut system = System::global();
///
/// use uactor::actor::message::Message;
/// use uactor::message_impl;
/// pub struct Ping;
/// impl Message for Ping { fn static_name() -> &'static str { "Ping" } }
///
/// pub struct Actor1;
/// impl Actor for Actor1 { type Context = Context;type RouteMessage = (); type Inject = (); type State = (); }
/// let actor1 = Actor1;
///
/// impl uactor::actor::abstract_actor::Handler<Ping> for Actor1 { async fn handle(&mut self, inject: &mut Self::Inject, msg: Ping, ctx: &mut Self::Context, state: &Self::State) -> HandleResult { todo!() }  }
///
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

            impl uactor::actor::message::Message for [<$ActorType Msg>] {
                fn static_name() -> &'static str {
                    stringify!([<$ActorType Msg>])
                }

                fn name(&self) -> String {
                    match self {
                    $(
                        Self::$Message(m) => m.name(),
                    )*
                        _ => Self::static_name().to_owned(),
                    }
                }
            }

            impl uactor::actor::abstract_actor::Handler<[<$ActorType Msg>]> for $ActorType {
                async fn handle(
                    &mut self,
                    inject: &mut <Self as uactor::actor::abstract_actor::Actor>::Inject,
                    msg: [<$ActorType Msg>],
                    ctx: &mut <Self as uactor::actor::abstract_actor::Actor>::Context,
                    state: &<$ActorType as uactor::actor::abstract_actor::Actor>::State,
                ) -> uactor::actor::abstract_actor::HandleResult {
                    match msg {
                        $(
                        [<$ActorType Msg>]::$Message(m) => {
                            self.handle(inject, m, ctx, state).await?;
                        }
                        ),*
                    }
                    Ok(())
                }
            }

            pub type [<$ActorType MpscRef>] = [<$ActorType Ref>]<tokio::sync::mpsc::UnboundedSender<[<$ActorType Msg>]>>;

            pub struct [<$ActorType Ref>]<T> where T: uactor::data::data_publisher::DataPublisher<Item=[<$ActorType Msg>]> + Clone {
                name: std::sync::Arc<str>,
                state: <$ActorType as uactor::actor::abstract_actor::Actor>::State,
                sender: T,
            }

            impl<T> uactor::data::data_publisher::TryClone for [<$ActorType Ref>]<T> where T: uactor::data::data_publisher::DataPublisher<Item=[<$ActorType Msg>]> + Clone {
                fn try_clone(&self) -> Result<Self, uactor::data::data_publisher::TryCloneError> {
                    self.sender.try_clone().map(|sender| Self { name: self.name.clone(), sender, state: self.state.clone() })
                }
            }

            impl Clone for [<$ActorType Ref>]<tokio::sync::mpsc::UnboundedSender<[<$ActorType Msg>]>> {
                fn clone(&self) -> Self {
                    [<$ActorType Ref>]::new(self.name.clone(), self.sender.clone(), self.state.clone())
                }
            }

            impl From<(
                uactor::aliases::ActorName,
                tokio::sync::mpsc::UnboundedSender<[<$ActorType Msg>]>,
                <$ActorType as uactor::actor::abstract_actor::Actor>::State
            )> for [<$ActorType Ref>]<tokio::sync::mpsc::UnboundedSender<[<$ActorType Msg>]>>
            {
                fn from((name, sender, state): (
                    uactor::aliases::ActorName,
                    tokio::sync::mpsc::UnboundedSender<[<$ActorType Msg>]>,
                    <$ActorType as uactor::actor::abstract_actor::Actor>::State
                )) -> Self {
                    Self {
                        name,
                        sender,
                        state,
                    }
                }
            }

            $(
                // #[cfg(not(feature = "async_sender"))]
                impl <T>uactor::actor::abstract_actor::MessageSender<$Message> for [<$ActorType Ref>]<T> where T: uactor::data::data_publisher::DataPublisher<Item=[<$ActorType Msg>]> + Clone {
                    fn send(&self, msg: $Message) -> uactor::data::data_publisher::DataPublisherResult {
                        self.sender.publish([<$ActorType Msg>]::$Message(msg))
                    }

                    async fn ask<A>(&self, f: impl FnOnce(tokio::sync::oneshot::Sender<A>) -> $Message) -> Result<A, uactor::data::data_publisher::DataPublisherErrors> {
                        let (tx, rx) = tokio::sync::oneshot::channel::<A>();
                        let message = f(tx);
                        self.sender.publish([<$ActorType Msg>]::$Message(message))?;
                        rx.await.map_err(uactor::data::data_publisher::DataPublisherErrors::from)
                    }
                }

                // #[cfg(feature = "async_sender")]
                // impl <T>uactor::actor::abstract_actor::MessageSender<$Message> for [<$ActorType Ref>]<T> where T: uactor::data::data_publisher::DataPublisher<Item=[<$ActorType Msg>]> + Clone {
                //     async fn send(&self, msg: $Message) -> uactor::data::data_publisher::DataPublisherResult {
                //         self.sender.publish([<$ActorType Msg>]::$Message(msg)).await
                //     }
                //
                //     async fn ask<A>(&self, f: impl FnOnce(tokio::sync::oneshot::Sender<A>) -> $Message) -> Result<A, uactor::data::data_publisher::DataPublisherErrors> {
                //         let (tx, rx) = tokio::sync::oneshot::channel::<A>();
                //         let message = f(tx);
                //         self.sender.publish([<$ActorType Msg>]::$Message(message))?;
                //         rx.await.map_err(uactor::data::data_publisher::DataPublisherErrors::from)
                //     }
                // }
            )*

            impl<T> [<$ActorType Ref>]<T> where T: uactor::data::data_publisher::DataPublisher<Item=[<$ActorType Msg>]> + Clone {
                pub fn new(name: std::sync::Arc<str>, sender: T, state: <$ActorType as uactor::actor::abstract_actor::Actor>::State) -> Self {
                    let name = std::sync::Arc::from(name);
                    Self { name, sender, state }
                }

                pub fn name(&self) -> std::sync::Arc<str> {
                    self.name.clone()
                }

                pub fn state(&self) -> &<$ActorType as uactor::actor::abstract_actor::Actor>::State {
                    &self.state
                }
            }
        }

    };
}
