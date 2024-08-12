use std::future::Future;

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
///# use uactor::context::extensions::Service;
/// use uactor::di::{Inject, InjectError};
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
    use crate::actor::NamedActorRef;
    use crate::context::extensions::Service;
    use crate::data_publisher::TryClone;
    use crate::di::{Inject, InjectError};
    use crate::system::System;
    use std::sync::Arc;

    impl Inject for () {
        async fn inject(_: &System) -> Result<Self, InjectError>
        where
            Self: Sized,
        {
            Ok(())
        }
    }

    impl<T1> Inject for T1
    where
        T1: DependencyProvider<Dependency = T1>,
    {
        async fn inject(system: &System) -> Result<Self, InjectError>
        where
            Self: Sized,
        {
            let result = T1::get_dependency(system)?;
            Ok(result)
        }
    }

    impl<T1, T2> Inject for (T1, T2)
    where
        T1: DependencyProvider<Dependency = T1>,
        T2: DependencyProvider<Dependency = T2>,
    {
        async fn inject(system: &System) -> Result<Self, InjectError>
        where
            Self: Sized,
        {
            let t1 = T1::get_dependency(system)?;
            let t2 = T2::get_dependency(system)?;
            Ok((t1, t2))
        }
    }

    pub trait DependencyProvider {
        type Dependency;
        fn get_dependency(system: &System) -> Result<Self::Dependency, InjectError>;
    }

    impl<T> DependencyProvider for Service<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        type Dependency = Service<T>;

        fn get_dependency(system: &System) -> Result<Self::Dependency, InjectError> {
            let service = system.get_service()?;
            Ok(service)
        }
    }

    impl<T> DependencyProvider for T
    where
        T: NamedActorRef + TryClone + Clone + Send + Sync + 'static,
    {
        type Dependency = Self;

        fn get_dependency(system: &System) -> Result<Self::Dependency, InjectError> {
            let actor_name = Self::static_name();
            let actor = system.get_actor(Arc::from(actor_name))?;
            Ok(actor)
        }
    }
}
