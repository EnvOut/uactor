use std::sync::Arc;
use tokio::task::JoinHandle;
use crate::actor::Actor;
use crate::context::Context;
use crate::context::extensions::{Extension, ExtensionErrors, Extensions};
use crate::errors::process_iteration_result;
use crate::select::ActorSelect;
use crate::system::builder::SystemBuilder;

#[derive(derive_more::Constructor)]
pub struct System {
    name: String,
    extensions: Arc<Extensions>,
}

impl System {
    pub fn global() -> SystemBuilder {
        SystemBuilder::new_global()
    }
}

impl System {
    pub fn get_extension<T>(&self) -> Result<&Extension<T>, ExtensionErrors> where T: Send + Sync + 'static {
        let option = self.extensions.get::<Extension<T>>();
        if let Some(extension) = option {
            return Ok(extension);
        } else {
            return Err(ExtensionErrors::MissingExtension(format!("{:?}", std::any::type_name::<T>())));
        }
    }
}

impl System {
    pub fn run<A, S>(&self, mut actor: A, actor_name: Option<String>, mut select: S) -> JoinHandle<()>
        where
            A: Actor + Send,
            S: ActorSelect<A> + Send + 'static
    {
        let mut ctx: Context = self.create_context();

        let system_name = self.name.clone();

        let actor_name= actor_name.unwrap_or_else(|| std::any::type_name::<A>().to_owned());

        let handle = tokio::spawn(async move {
            tracing::debug!("The system: {:?} spawned actor: {:?}", system_name, actor_name);

            loop {
                tracing::debug!("iteration of the process: {actor_name:?}");
                let result = select.select(&mut ctx, &mut actor).await;
                process_iteration_result(&actor_name, result);
            }
        });
        handle
    }

    // pub async fn run_fn<A, F, S>(&self, f: F, mut select: S) -> JoinHandle<()>
    //     where
    //         A: Actor + Send,
    //         F: FnOnce(&mut A::Context) -> A,
    //         S: ActorSelect<A> + Send + 'static
    // {
    //     let mut ctx = self.create_context::<A>();
    //     let mut actor = f(&mut ctx);
    //
    //     let process_name = std::any::type_name::<A>().to_owned();
    //     let handle = tokio::spawn(async move {
    //         tracing::debug!("Spawn process: {process_name:?}");
    //
    //         loop {
    //             tracing::debug!("iteration of the process: {process_name:?}");
    //             let result = select.select(&mut ctx, &mut actor).await;
    //             tracing::debug!("{process_name:?} result: {result:?}");
    //         }
    //     });
    //     handle
    // }

    pub fn create_context(&self) -> Context {
        Context::new(self.extensions.clone())
    }
}

pub mod builder {
    use std::sync::Arc;
    use crate::context::extensions::{Extension, Extensions};
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
            self.extensions.insert(Extension(data));
            self
        }

        pub fn build(self) -> System {
            System::new(self.name, Arc::new(self.extensions))
        }
    }
}
