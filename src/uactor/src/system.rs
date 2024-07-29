use crate::actor::Actor;
use crate::context::extensions::{ExtensionErrors, Extensions, Service};
use crate::context::{ActorContext, Context, ContextInitializationError, ContextResult};
use crate::di::{Inject, InjectError};
use crate::select::ActorSelect;
use crate::system::builder::SystemBuilder;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use futures::StreamExt;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use crate::context::actor_registry::{ActorRegistry, ActorRegistryErrors};
use crate::data_publisher::{TryClone, TryCloneError};
use crate::datasource::DataSource;

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
    initialized_actors: HashMap<Arc<str>, oneshot::Sender<Box<dyn Any + Send>>>,
    actor_registry: ActorRegistry,
}

impl System {}

impl System {
    pub fn global() -> SystemBuilder {
        SystemBuilder::new_global()
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

    pub fn get_actor<A>(&self, actor_name: Arc<str>) -> Result<A, ActorRegistryErrors>
        where
            A: TryClone + Send + Sync + 'static,
    {
        let actor_ref: &A = self.actor_registry.get_actor::<A>(actor_name.clone())
            .ok_or_else(|| {
                let system_name = self.name.clone();
                let kind = utils::type_name::<A>();
                let actor_name = actor_name.clone();
                ActorRegistryErrors::NotRegisteredActor { system_name, kind, actor_name }
            })?;
        let a = actor_ref.try_clone()?;
        Ok(a)
    }

    pub fn get_actors<A>(&self) -> Result<Vec<A>, ActorRegistryErrors>
        where
            A: TryClone + Send + Sync + 'static,
    {
        let actor_ref = self.actor_registry.get_all()
            .ok_or_else(|| {
                let system_name = self.name.clone();
                let kind = std::any::type_name::<A>().to_owned();
                ActorRegistryErrors::NotRegisteredActorKind { system_name, kind }
            })?.into_iter()
            .map(|i: &A| i.try_clone())
            .collect::<Result<Vec<A>, TryCloneError>>()?;
        Ok(actor_ref)
    }

    pub fn insert_actor<T: Send + Sync + TryClone + 'static>(&mut self, actor_name: Arc<str>, actor_ref: T) {
        tracing::info!("Insert actor: {actor_name:?}: {} into system context: {:?}", std::any::type_name::<T>(), self.name);
        self.actor_registry.insert::<T>(actor_name, actor_ref);
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

impl System {
    pub async fn run_actor<A>(&mut self, actor_name: Arc<str>) -> Result<(), ActorRunningError>
        where
            A: Actor + Any,
            <A as Actor>::Inject: Inject + Sized + Send,
    {
        if let Some(tx) = self.initialized_actors.remove(&actor_name) {
            let state_res = A::Inject::inject(self).await;

            let ctx = A::Context::create(self, actor_name.clone()).await.map_err(|err| ActorRunningError::ContextError(err))?;

            if let Err(err) = state_res.as_ref() {
                tracing::error!("Can't inject dependencies for {actor_name:?}, actor not started. Err: {err:?}")
            }
            let state = state_res?;

            if tx.send(Box::new((state, ctx))).is_err() {
                return Err(ActorRunningError::Dropped(actor_name.clone()));
            }
        } else {
            eprintln!("actor_name: {:?} already started", actor_name);
            return Err(ActorRunningError::MissedInitializationOrAlreadyStarted(actor_name.clone()))
        }
        Ok(())
    }

    pub fn init_actor<A, S>(&mut self, mut actor: A, actor_name: Option<Arc<str>>, mut select: S) -> (Arc<str>, JoinHandle<()>)
        where
            A: Actor + Send,
            S: ActorSelect<A> + Send + 'static,
            <A as Actor>::Inject: Inject + Sized + Send,
    {
        let system_name = self.name.clone();

        let actor_name: Arc<str> = actor_name.unwrap_or_else(|| {
            let type_name = utils::type_name::<A>();
            format!("{}-{}", type_name, (&type_name as *const String as i32)).into()
        });
        let (actor_state_tx, actor_state_rx) = oneshot::channel::<Box<dyn Any + Send>>();

        self.initialized_actors.insert(actor_name.clone(), actor_state_tx);

        let name = actor_name.clone();
        let handle = tokio::spawn(async move {
            tracing::debug!("The system: {:?} spawned actor: {:?}", system_name, name);

            if let Ok(boxed_state) = actor_state_rx.await {
                let  (mut state, mut ctx) = {
                    let boxed_state = boxed_state
                        .downcast::<(<A as Actor>::Inject, <A as Actor>::Context)>()
                        .expect("failed to downcast state");
                    *boxed_state
                };

                // call on_start
                match ctx.on_start() {
                    Ok(_) => {
                        tracing::trace!("Starting the actor: {name:?}");
                    }
                    Err(err) => {
                        tracing::error!("Error during actor start: {err:?}");
                        ctx.kill();
                    }
                }

                // main loop
                while ctx.is_alive() {
                    tracing::trace!("iteration of the process: {name:?}");
                    let res = select.select(&mut state, &mut ctx, &mut actor).await;

                    if let Err(err) = res {
                        tracing::error!("An error occurred while message handling by the \"{}\", error message: \"{}\"", ctx.get_name(), err);
                    } else {
                        tracing::trace!("{name:?} successful iteration");
                    }
                }
                // call on_die
                match ctx.on_die(name.clone()) {
                    Ok(_) => {
                        tracing::trace!("The actor: {name:?} is dead");
                    }
                    Err(err) => {
                        tracing::error!("Error during actor die: {err:?}");
                        return;
                    }
                };
            } else {
                tracing::error!("Can't run {name:?}, system dropped");
                ()
            }
        });
        (actor_name, handle)
    }
}

pub mod builder {
    use std::sync::Arc;
    use crate::context::extensions::{Extensions, Service};
    use crate::system::System;

    const GLOBAL_SYSTEM_NAME: &str = "Global";

    #[derive(derive_more::Constructor)]
    pub struct SystemBuilder {
        name: String,
        extensions: Extensions,
    }

    impl SystemBuilder {
        pub fn new_global() -> Self {
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
            System::new(Arc::from(self.name.as_str()), self.extensions, Default::default(), Default::default())
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