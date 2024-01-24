use crate::actor::Actor;
use crate::context::extensions::{ExtensionErrors, Extensions, Service};
use crate::context::Context;
use crate::di::{Inject, InjectError};
use crate::errors::process_iteration_result;
use crate::select::ActorSelect;
use crate::system::builder::SystemBuilder;
use std::any::Any;
use std::collections::HashMap;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

#[derive(thiserror::Error, Debug)]
pub enum ActorRunningError {
    #[error("The actor: {0:?} has dropped")]
    Dropped(String),
    #[error("The actor: {0:?} has not been initialized or has already been started")]
    MissedInitializationOrAlreadyStarted(String),
    #[error(transparent)]
    InjectError(#[from] InjectError),
}

#[derive(derive_more::Constructor)]
pub struct System {
    name: String,
    extensions: Extensions,
    initialized_actors: HashMap<String, oneshot::Sender<Box<dyn Any + Send>>>,
}

impl System {}

impl System {
    pub fn global() -> SystemBuilder {
        SystemBuilder::new_global()
    }
}

impl System {
    pub fn get_service<T>(&self) -> Result<T, ExtensionErrors>
        where
            T: Clone + Send + Sync + 'static,
    {
        let Service(data) = self.get::<Service<T>>()?;
        Ok(data.clone())
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
    pub async fn run_actor<A>(&mut self, actor_name: &str) -> Result<(), ActorRunningError>
        where
            A: Actor + Any,
            <A as Actor>::Inject: Inject + Sized + Send,
    {
        if let Some(tx) = self.initialized_actors.remove(actor_name) {
            let state = A::Inject::inject(self).await?;
            if tx.send(Box::new(state)).is_err() {
                return Err(ActorRunningError::Dropped(actor_name.to_owned()));
            }
        } else {
            return Err(ActorRunningError::MissedInitializationOrAlreadyStarted(actor_name.to_owned()))
        }
        Ok(())
    }

    pub fn init_actor<A, S>(&mut self, mut actor: A, actor_name: Option<String>, mut select: S) -> (String, JoinHandle<()>)
        where
            A: Actor + Send,
            S: ActorSelect<A> + Send + 'static,
            <A as Actor>::Inject: Inject + Sized + Send
    {
        let mut ctx: Context = self.create_context();

        let system_name = self.name.clone();

        let actor_name = actor_name.unwrap_or_else(|| {
            let type_full_name = std::any::type_name::<A>();
            let type_name = type_full_name
                .split("::")
                .last()
                .unwrap_or(type_full_name)
                .to_owned();
            format!("{}-{}", type_name, (&type_name as *const String as i32))
        });
        let (actor_state_tx, actor_state_rx) = oneshot::channel::<Box<dyn Any + Send>>();

        self.initialized_actors.insert(actor_name.clone(), actor_state_tx);

        let name = actor_name.clone();
        let handle = tokio::spawn(async move {
            tracing::debug!("The system: {:?} spawned actor: {:?}", system_name, name);

            if let Ok(boxed_state) = actor_state_rx.await {
                let mut state = {
                    let boxed_state = boxed_state
                        .downcast::<<A as Actor>::Inject>()
                        .expect("failed to downcast state");
                    *boxed_state
                };

                loop {
                    tracing::debug!("iteration of the process: {name:?}");
                    let result = select.select(&mut state, &mut ctx, &mut actor).await;
                    process_iteration_result(&name, result);
                }
            } else {
                tracing::error!("Can't run {name:?}, system dropped");
                ()
            }
        });
        (actor_name, handle)
    }

    pub fn create_context(&self) -> Context {
        Context::new()
    }
}

pub mod builder {
    use crate::context::extensions::{Extensions, Service};
    use crate::system::System;

    const GLOBAL_SYSTEM_NAME: &str = "Global";

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
            System::new(self.name, self.extensions, Default::default())
        }
    }
}
