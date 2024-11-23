use crate::actor::message::Message;
use std::future::Future;

#[macro_export]
macro_rules! spawn_with_ref {
    ($S: ident, $ActorName: ident, $ActorInstance: ident: $ActorType: ident, $($Timeout: ident),*) => {{
        uactor::paste! {
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<[<$ActorType Msg>]>();
            let (name, shared_state, handle): (std::sync::Arc<str>, std::sync::Arc<<$ActorType as uactor::actor::abstract_actor::Actor>::State>, tokio::task::JoinHandle<()>) = $S.init_actor($ActorInstance, Some($ActorName.to_owned()), ($($Timeout,)* rx));
            let actor_ref = [<$ActorType Ref>]::new(name, tx, shared_state);
            $S.insert_actor(actor_ref.name(), uactor::data::data_publisher::TryClone::try_clone(&actor_ref)?);
            (actor_ref, handle)
        }
    }};

    ($S: ident, $ActorInstance: ident: $ActorType: ident, $($Timeout: ident),*) => {{
        let actor_name: std::sync::Arc<str> = stringify!($ActorInstance).into();

        uactor::paste! {
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<[<$ActorType Msg>]>();
            let (name, shared_state, handle): (std::sync::Arc<str>, std::sync::Arc<<$ActorType as uactor::actor::abstract_actor::Actor>::State>, tokio::task::JoinHandle<()>) = $S.init_actor($ActorInstance, Some(actor_name), ($($Timeout,)* rx));
            let actor_ref = [<$ActorType Ref>]::new(name, tx, shared_state);
            $S.insert_actor(actor_ref.name(), uactor::data::data_publisher::TryClone::try_clone(&actor_ref)?);
            (actor_ref, handle)
        }
    }};

    ($S: ident, $ActorName: ident, $ActorInstance: ident: $ActorType: ident) => {{
        let actor_name: std::sync::Arc<str> = stringify!($ActorInstance).into();
        uactor::paste! {
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<[<$ActorType Msg>]>();
            let (name, shared_state, handle): (std::sync::Arc<str>, std::sync::Arc<<$ActorType as uactor::actor::abstract_actor::Actor>::State>, tokio::task::JoinHandle<()>) = $S.init_actor($ActorInstance, Some(actor_name), (rx));
            let actor_ref = [<$ActorType Ref>]::new(name, tx, shared_state);
            $S.insert_actor(actor_ref.name(), uactor::data::data_publisher::TryClone::try_clone(&actor_ref)?);
            (actor_ref, handle)
        }
    }};

    ($S: ident, $ActorInstance: ident: $ActorType: ident) => {{
        let actor_name: std::sync::Arc<str> = stringify!($ActorInstance).into();
        uactor::paste! {
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<[<$ActorType Msg>]>();
            let (name, shared_state, handle): (std::sync::Arc<str>, std::sync::Arc<<$ActorType as uactor::actor::abstract_actor::Actor>::State>, tokio::task::JoinHandle<()>) = $S.init_actor($ActorInstance, Some(actor_name), (rx));
            let actor_ref = [<$ActorType Ref>]::new(name, tx, shared_state);
            $S.insert_actor(actor_ref.name(), uactor::data::data_publisher::TryClone::try_clone(&actor_ref)?);
            (actor_ref, handle)
        }
    }};
}
