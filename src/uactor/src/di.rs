use crate::context::actor_registry::ActorRegistryErrors;
use crate::context::extensions::ExtensionErrors;
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
///# use uactor::di::{Inject, InjectError};
///# use uactor::system::System;
///
/// pub struct References {
///     var1: String,
///     var2: String,
/// }
///
/// impl Inject for References {
///     async fn inject(system: &System) -> Result<Self, InjectError>
///         where
///             Self: Sized
///     {
///         let var1 = system.get_service::<String>()?.clone();
///         let var2 = system.get_service::<String>()?.clone();
///         Ok(Self { var1: var1, var2: var2 })
///     }
/// }
/// ```
pub trait Inject {
    async fn inject(system: &System) -> Result<Self, InjectError>
        where
            Self: Sized;
}

impl Inject for () {
    async fn inject(_: &System) -> Result<Self, InjectError>
        where
            Self: Sized,
    {
        Ok(())
    }
}
