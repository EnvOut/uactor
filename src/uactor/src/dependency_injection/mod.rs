use std::future::Future;

use crate::actor::context::actor_registry::ActorRegistryErrors;
use crate::actor::context::extensions::ExtensionErrors;
use crate::system::System;

#[derive(thiserror::Error, Debug)]
pub enum InjectError {
    #[error(transparent)]
    ExtensionErrors(#[from] ExtensionErrors),
    #[error(transparent)]
    ActorRegistryErrors(#[from] ActorRegistryErrors),
}

/// Sample:
/// ```
///# use uactor::actor::context::extensions::Service;
/// use uactor::dependency_injection::{Inject, InjectError};
///# use uactor::system::System;
///
/// pub struct References {
///     var1: Service<String>,
///     var2: Service<String>,
/// }
///
/// impl Inject for References {
///     async fn inject(system: &System) -> Result<Self, InjectError>
///         where
///             Self: Sized
///     {
///         let var1 = system.get_service::<String>()?.clone();
///         let var2 = system.get_service::<String>()?.clone();
///         Ok(Self { var1, var2 })
///     }
/// }
/// ```
pub trait Inject {
    fn inject(system: &System) -> impl Future<Output = Result<Self, InjectError>> + Send
    where
        Self: Sized;
}

pub mod inject_impls {
    use crate::dependency_injection::{Inject, InjectError};
    use crate::system::System;

    impl Inject for () {
        async fn inject(_: &System) -> Result<Self, InjectError>
        where
            Self: Sized,
        {
            Ok(())
        }
    }
}
