use crate::actor::abstract_actor::{Actor, Handler};
use crate::actor::context::actor_registry::{ActorRegistry, ActorRegistryErrors};
use crate::actor::context::extensions::{ExtensionErrors, Extensions, Service};
use crate::actor::context::ActorContext;
use crate::actor::message::Message;
use crate::actor::select::ActorSelect;
use crate::aliases::ActorName;
use crate::data::data_publisher::{DataPublisher, TryCloneError};
use crate::dependency_injection::{Inject, InjectError};
use crate::system::builder::SystemBuilder;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;
use crate::system::global::GlobalSystem;

pub mod global;

#[derive(thiserror::Error, Debug)]
pub enum ActorRunningError {
    #[error("The actor: {0:?} has dropped")]
    Dropped(Arc<str>),
    #[error("The actor: {0:?} has not been initialized or has already been started")]
    MissedInitializationOrAlreadyStarted(Arc<str>),
    #[error(transparent)]
    InjectError(#[from] InjectError),
    #[error("Can't create actor context: {0}")]
    ContextError(String),
}

#[derive(derive_more::Constructor)]
pub struct System {
    name: Arc<str>,
    extensions: Extensions,
    actor_registry: ActorRegistry,
}

impl System {
    pub fn extension<T: Send + Sync + 'static>(&mut self, data: T) -> &mut Self {
        self.extensions.insert(Service(data));
        self
    }

    pub async fn register_ref_with_state<A, M, R>(
        &mut self,
        actor_name: &str,
        state: A::State,
    ) -> (R, tokio::sync::mpsc::UnboundedReceiver<M>)
    where
        A: Actor,
        M: Message + Send + 'static,
        R: From<(ActorName, UnboundedSender<M>, A::State)>,
    {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<M>();

        let actor_name: Arc<str> = actor_name.to_owned().into();
        let old_ref = self.actor_registry.register_ref::<A, M, UnboundedSender<M>>(actor_name.clone(), tx.clone(), state.clone());
        if let Some(_old_ref) = old_ref {
            tracing::warn!("The actor: {actor_name:?} has already been registered, old ref has been replaced");
        }

        (R::from((actor_name, tx, state)), rx)
    }

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
        let state = A::State::default();
        self.register_ref_with_state::<A, M, R>(actor_name, state).await
    }

    pub async fn spawn_actor<A, S>(
        &mut self,
        actor_name: Arc<str>,
        mut actor: A,
        state: A::State,
        mut aggregator: S,
    ) -> Result<(<A as Actor>::State, JoinHandle<()>), ActorRunningError>
    where
        A: Handler<<A as Actor>::RouteMessage> + Actor + Send,
        S: ActorSelect<A> + Send + 'static,
        <A as Actor>::Inject: Inject + Sized + Send,
    {
        let system_name = self.name.clone();

        let mut ctx = A::Context::create::<A>(self, actor_name.clone())
            .await
            .map_err(ActorRunningError::ContextError)?;

        let mut inject = A::Inject::inject(self).await?;

        let handle = {
            let state = state.clone();
            tokio::spawn(async move {
                tracing::debug!("The system: {:?} spawned actor: {:?}", system_name, actor_name);

                // call on_start
                match ctx.on_start() {
                    Ok(_) => {
                        tracing::trace!("Starting the actor: {actor_name:?}");
                    }
                    Err(err) => {
                        tracing::error!("Error during actor start: {err:?}");
                        ctx.kill();
                    }
                }

                actor.on_start(&mut inject, &mut ctx).await;

                // main loop
                while ctx.is_alive() {
                    tracing::trace!("iteration of the process: {actor_name:?}");

                    let message_res = actor.select_message(&mut ctx, &mut aggregator).await;

                    match message_res {
                        Ok(message) => {
                            let res = actor.handle(&mut inject, message, &mut ctx, &state).await;

                            ctx.after_iteration();

                            if let Err(err) = res {
                                tracing::error!("An error occurred while message handling by the \"{}\", error message: \"{}\"", ctx.get_name(), err);

                                ctx.on_error(&err);
                                actor.on_error(&mut ctx, err).await;
                            } else {
                                tracing::trace!("{actor_name:?} successful iteration");
                            }
                        }
                        Err(channel_error) => {
                            actor.on_select_error(channel_error, &mut ctx).await;
                            continue;
                        }
                    }
                }

                // destroy context
                match ctx.on_die(actor_name.clone()) {
                    Ok(_) => {
                        tracing::trace!("Context of the actor: {actor_name:?} destroyed");
                    }
                    Err(err) => {
                        tracing::error!("Error during context die of the actor. {actor_name:?}: {err:?}");
                    }
                }

                // destroy actor as object

                actor.on_die(&mut ctx, &state).await
            })
        };

        Ok((state, handle))
    }
}

impl System {}

impl System {
    pub fn global() -> GlobalSystem {
        GlobalSystem {}
    }

    pub fn name(system_name: String) -> SystemBuilder {
        SystemBuilder::new(system_name, Extensions::new())
    }
}

impl System {
    pub fn get_service<T>(&self) -> Result<Service<T>, ExtensionErrors>
    where
        T: Clone + Send + Sync + 'static,
    {
        let service = self.get::<Service<T>>()?;
        Ok(service.clone())
    }

    pub fn get_actor<A, M, D, R>(&self, actor_name: Arc<str>) -> Result<R, ActorRegistryErrors>
    where
        A: Actor + Send + Sync + 'static,
        M: Message,
        D: DataPublisher<Item = M> + Send + Sync + 'static,
        R: From<(ActorName, D, A::State)>,
    {
        let (channel, state): &(D, A::State) = self
            .actor_registry
            .get_actor_ref::<A, _>(actor_name.clone())
            .ok_or_else(|| {
                let system_name = self.name.clone();
                let kind = utils::type_name::<A>();
                let actor_name = actor_name.clone();
                ActorRegistryErrors::NotRegisteredActor {
                    system_name,
                    kind,
                    actor_name,
                }
            })?;

        let reference = R::from((actor_name, channel.try_clone()?, state.clone()));
        Ok(reference)
    }

    pub fn get_actors<A, M, D>(&self) -> Result<Vec<(D, A::State)>, ActorRegistryErrors>
    where
        A: Actor,
        M: Message,
        D: DataPublisher<Item = M> + Send + Sync + 'static,
    {
        let actor_ref = self
            .actor_registry
            .get_all::<A, M, D>()
            .ok_or_else(|| {
                let system_name = self.name.clone();
                let kind = std::any::type_name::<A>().to_owned();
                ActorRegistryErrors::NotRegisteredActorKind { system_name, kind }
            })?
            .into_iter()
            .map(|(c, state): &(D, A::State)| Ok((c.try_clone()?, state.clone())))
            .collect::<Result<Vec<_>, TryCloneError>>()?;
        Ok(actor_ref)
    }

    pub fn insert_service<T: Send + Sync + 'static>(&mut self, data: T) {
        self.extensions.insert(Service(data));
    }

    pub fn insert<T: Send + Sync + 'static>(&mut self, data: T) {
        self.extensions.insert(data);
    }

    pub fn get<T>(&self) -> Result<&T, ExtensionErrors>
    where
        T: Clone + Send + Sync + 'static,
    {
        let option = self.extensions.get::<T>();
        if let Some(extension) = option {
            Ok(extension)
        } else {
            let type_name = std::any::type_name::<T>().to_owned();
            Err(ExtensionErrors::NotRegisteredType {
                kind: type_name,
                system_name: self.name.clone(),
            })
        }
    }
}

pub mod builder {
    use crate::actor::context::extensions::{Extensions, Service};
    use crate::system::System;
    use std::sync::Arc;

    const GLOBAL_SYSTEM_NAME: &str = "Global";

    #[derive(derive_more::Constructor)]
    pub struct SystemBuilder {
        name: String,
        extensions: Extensions,
    }

    impl SystemBuilder {
        pub (crate) fn new_global() -> Self {
            Self {
                name: GLOBAL_SYSTEM_NAME.to_owned(),
                extensions: Extensions::new(),
            }
        }

        pub fn global(mut self) -> Self {
            self.name = GLOBAL_SYSTEM_NAME.to_owned();
            self
        }

        pub fn name(mut self, name: String) -> Self {
            self.name = name;
            self
        }

        pub fn extension<T: Send + Sync + 'static>(mut self, data: T) -> Self {
            self.extensions.insert(Service(data));
            self
        }

        pub fn build(self) -> System {
            System::new(
                Arc::from(self.name.as_str()),
                self.extensions,
                Default::default(),
            )
        }
    }
}

pub mod utils {
    pub fn type_name<T>() -> String {
        let type_full_name = std::any::type_name::<T>();
        let type_name = type_full_name
            .split("::")
            .last()
            .unwrap_or(type_full_name)
            .to_owned();
        type_name
    }
}
