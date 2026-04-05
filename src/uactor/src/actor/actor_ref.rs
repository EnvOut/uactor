/// Generates a route-message enum, an actor reference type, and `MessageSender` impls.
///
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
///
/// Supports aliased variants — a newtype wrapper is generated for each alias,
/// so multiple aliases with the same inner type work correctly:
/// ```ignore
/// uactor::generate_actor_ref!(MyActor, { PingMsg, TaskDone: TaskId, TaskFailed: TaskId });
/// // Generates:
/// //   pub struct TaskDone(pub TaskId);
/// //   pub struct TaskFailed(pub TaskId);
/// //   enum MyActorMsg { PingMsg(PingMsg), TaskDone(TaskDone), TaskFailed(TaskFailed) }
/// ```
#[macro_export]
macro_rules! generate_actor_ref {
    ($ActorType:ident, { $($rest:tt)* }) => {
        $crate::generate_actor_ref!(@munch $ActorType [] [] $($rest)*);
    };

    // --- TT muncher: two accumulators [plain...] [aliased...] ---

    // Aliased with trailing comma
    (@munch $ActorType:ident [$($plain:tt)*] [$($aliased:tt)*] $Alias:ident : $Type:ty, $($rest:tt)*) => {
        $crate::generate_actor_ref!(@munch $ActorType [$($plain)*] [$($aliased)* ($Alias, $Type)] $($rest)*);
    };

    // Aliased without trailing comma (last)
    (@munch $ActorType:ident [$($plain:tt)*] [$($aliased:tt)*] $Alias:ident : $Type:ty) => {
        $crate::generate_actor_ref!(@munch $ActorType [$($plain)*] [$($aliased)* ($Alias, $Type)]);
    };

    // Unaliased with trailing comma
    (@munch $ActorType:ident [$($plain:tt)*] [$($aliased:tt)*] $Message:ident, $($rest:tt)*) => {
        $crate::generate_actor_ref!(@munch $ActorType [$($plain)* ($Message)] [$($aliased)*] $($rest)*);
    };

    // Unaliased without trailing comma (last)
    (@munch $ActorType:ident [$($plain:tt)*] [$($aliased:tt)*] $Message:ident) => {
        $crate::generate_actor_ref!(@munch $ActorType [$($plain)* ($Message)] [$($aliased)*]);
    };

    // Base case: all entries collected
    (@munch $ActorType:ident [$(($Plain:ident))*] [$(($Alias:ident, $AType:ty))*]) => {
        $crate::generate_actor_ref!(@emit $ActorType [$(($Plain))*] [$(($Alias, $AType))*]);
    };

    // --- Emit ---

    (@emit $ActorType:ident [$(($Plain:ident))*] [$(($Alias:ident, $AType:ty))*]) => {
        $crate::paste! {
            $(
                pub struct $Alias(pub $AType);

                impl $crate::actor::message::Message for $Alias {
                    fn static_name() -> &'static str {
                        stringify!($Alias)
                    }
                }
            )*

            #[allow(private_interfaces)]
            #[warn(clippy::large_enum_variant)]
            pub enum [<$ActorType Msg>] {
                $($Plain($Plain),)*
                $($Alias($Alias),)*
            }

            impl $crate::actor::message::Message for [<$ActorType Msg>] {
                fn static_name() -> &'static str {
                    stringify!([<$ActorType Msg>])
                }

                #[allow(unreachable_patterns)]
                fn name(&self) -> String {
                    match self {
                    $(
                        Self::$Plain(m) => m.name(),
                    )*
                    $(
                        Self::$Alias(m) => m.name(),
                    )*
                        _ => Self::static_name().to_owned(),
                    }
                }
            }

            impl $crate::actor::abstract_actor::Handler<[<$ActorType Msg>]> for $ActorType {
                async fn handle(
                    &mut self,
                    inject: &mut <Self as $crate::actor::abstract_actor::Actor>::Inject,
                    msg: [<$ActorType Msg>],
                    ctx: &mut <Self as $crate::actor::abstract_actor::Actor>::Context,
                    state: &<$ActorType as $crate::actor::abstract_actor::Actor>::State,
                ) -> $crate::actor::abstract_actor::HandleResult {
                    match msg {
                        $(
                        [<$ActorType Msg>]::$Plain(m) => {
                            self.handle(inject, m, ctx, state).await?;
                        }
                        )*
                        $(
                        [<$ActorType Msg>]::$Alias(m) => {
                            self.handle(inject, m, ctx, state).await?;
                        }
                        )*
                    }
                    Ok(())
                }
            }

            pub type [<$ActorType MpscRef>] = [<$ActorType Ref>]<tokio::sync::mpsc::UnboundedSender<[<$ActorType Msg>]>>;

            pub struct [<$ActorType Ref>]<T> where T: $crate::data::data_publisher::DataPublisher<Item=[<$ActorType Msg>]> + Clone {
                name: std::sync::Arc<str>,
                state: <$ActorType as $crate::actor::abstract_actor::Actor>::State,
                sender: T,
            }

            impl<T> $crate::data::data_publisher::TryClone for [<$ActorType Ref>]<T> where T: $crate::data::data_publisher::DataPublisher<Item=[<$ActorType Msg>]> + Clone {
                fn try_clone(&self) -> Result<Self, $crate::data::data_publisher::TryCloneError> {
                    self.sender.try_clone().map(|sender| Self { name: self.name.clone(), sender, state: self.state.clone() })
                }
            }

            impl Clone for [<$ActorType Ref>]<tokio::sync::mpsc::UnboundedSender<[<$ActorType Msg>]>> {
                fn clone(&self) -> Self {
                    [<$ActorType Ref>]::new(self.name.clone(), self.sender.clone(), self.state.clone())
                }
            }

            impl From<(
                $crate::aliases::ActorName,
                tokio::sync::mpsc::UnboundedSender<[<$ActorType Msg>]>,
                <$ActorType as $crate::actor::abstract_actor::Actor>::State
            )> for [<$ActorType Ref>]<tokio::sync::mpsc::UnboundedSender<[<$ActorType Msg>]>>
            {
                fn from((name, sender, state): (
                    $crate::aliases::ActorName,
                    tokio::sync::mpsc::UnboundedSender<[<$ActorType Msg>]>,
                    <$ActorType as $crate::actor::abstract_actor::Actor>::State
                )) -> Self {
                    Self {
                        name,
                        sender,
                        state,
                    }
                }
            }

            impl [<$ActorType Ref>]<tokio::sync::mpsc::UnboundedSender<[<$ActorType Msg>]>> {
                pub fn send_msg(&self, msg: [<$ActorType Msg>]) -> $crate::data::data_publisher::DataPublisherResult {
                    $crate::data::data_publisher::DataPublisher::publish(&self.sender, msg)
                }
            }

            $(
                impl <T>$crate::actor::abstract_actor::MessageSender<$Plain> for [<$ActorType Ref>]<T> where T: $crate::data::data_publisher::DataPublisher<Item=[<$ActorType Msg>]> + Clone {
                    fn send(&self, msg: $Plain) -> $crate::data::data_publisher::DataPublisherResult {
                        self.sender.publish([<$ActorType Msg>]::$Plain(msg))
                    }

                    async fn ask<A>(&self, f: impl FnOnce(tokio::sync::oneshot::Sender<A>) -> $Plain) -> Result<A, $crate::data::data_publisher::DataPublisherErrors> {
                        let (tx, rx) = tokio::sync::oneshot::channel::<A>();
                        let message = f(tx);
                        self.sender.publish([<$ActorType Msg>]::$Plain(message))?;
                        rx.await.map_err($crate::data::data_publisher::DataPublisherErrors::from)
                    }
                }
            )*

            $(
                impl <T>$crate::actor::abstract_actor::MessageSender<$Alias> for [<$ActorType Ref>]<T> where T: $crate::data::data_publisher::DataPublisher<Item=[<$ActorType Msg>]> + Clone {
                    fn send(&self, msg: $Alias) -> $crate::data::data_publisher::DataPublisherResult {
                        self.sender.publish([<$ActorType Msg>]::$Alias(msg))
                    }

                    async fn ask<A>(&self, f: impl FnOnce(tokio::sync::oneshot::Sender<A>) -> $Alias) -> Result<A, $crate::data::data_publisher::DataPublisherErrors> {
                        let (tx, rx) = tokio::sync::oneshot::channel::<A>();
                        let message = f(tx);
                        self.sender.publish([<$ActorType Msg>]::$Alias(message))?;
                        rx.await.map_err($crate::data::data_publisher::DataPublisherErrors::from)
                    }
                }
            )*

            impl<T> [<$ActorType Ref>]<T> where T: $crate::data::data_publisher::DataPublisher<Item=[<$ActorType Msg>]> + Clone {
                pub fn new(name: std::sync::Arc<str>, sender: T, state: <$ActorType as $crate::actor::abstract_actor::Actor>::State) -> Self {
                    let name = std::sync::Arc::from(name);
                    Self { name, sender, state }
                }

                pub fn name(&self) -> std::sync::Arc<str> {
                    self.name.clone()
                }

                pub fn state(&self) -> &<$ActorType as $crate::actor::abstract_actor::Actor>::State {
                    &self.state
                }
            }
        }

    };
}
