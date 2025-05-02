use std::sync::{Arc, LazyLock};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use crate::actor::abstract_actor::{Actor, Handler};
use crate::actor::message::Message;
use crate::actor::select::ActorSelect;
use crate::aliases::ActorName;
use crate::dependency_injection::Inject;
use crate::system::builder::SystemBuilder;
use super::{ActorRunningError, System};

static GLOBAL_SYSTEM: LazyLock<Arc<RwLock<System>>> = LazyLock::new(|| {
    Arc::new(RwLock::new(SystemBuilder::new_global().build()))
});

pub struct GlobalSystem {}

impl GlobalSystem {
    pub async fn register_ref<A, M, R>(
        &mut self,
        actor_name: &str,
    ) -> (R, tokio::sync::mpsc::UnboundedReceiver<M>)
    where
        A: Actor,
        A::State: Default,
        M: Message + Send + 'static,
        R: From<(ActorName, UnboundedSender<M>, A::State)>,
    {
        let mut system = GLOBAL_SYSTEM.write().await;
        system.register_ref::<A, M, R>(actor_name).await
    }

    pub async fn register_ref_with_state<A, M, R>(
        &mut self,
        actor_name: &str,
        state: A::State,
    ) -> (R, tokio::sync::mpsc::UnboundedReceiver<M>)
    where
        A: Actor,
        A::State: Default,
        M: Message + Send + 'static,
        R: From<(ActorName, UnboundedSender<M>, A::State)>,
    {
        let mut system = GLOBAL_SYSTEM.write().await;
        system.register_ref_with_state::<A, M, R>(actor_name, state).await
    }

    pub async fn spawn_actor<A, S>(
        &mut self,
        actor_name: Arc<str>,
        actor: A,
        state: A::State,
        aggregator: S,
    ) -> Result<(<A as Actor>::State, JoinHandle<()>), ActorRunningError>
    where
        A: Actor + Send + Handler<<A as Actor>::RouteMessage>,
        S: ActorSelect<A> + Send + 'static,
        <A as Actor>::Inject: Inject + Sized + Send,
    {
        let mut system = GLOBAL_SYSTEM.write().await;
        system.spawn_actor::<A, S>(actor_name, actor, state, aggregator).await
    }

    pub async fn with(&mut self, with: impl FnOnce(&mut System)) -> Self {
        let mut system = GLOBAL_SYSTEM.write().await;
        with(&mut system);
        GlobalSystem {}
    }
}