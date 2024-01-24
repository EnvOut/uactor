use crate::system::System;

pub type InjectError = Box<dyn std::error::Error + Send + Sync + 'static>;

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
///     async fn inject(system: &System) -> Result<Self, InjectError> where Self: Sized {
///         let var1 = system.get_service::<String>()?.clone();
///         let var2 = system.get_service::<String>()?.clone();
///         Ok(Self { var1: var1, var2: var2 })
///     }
/// }
/// ```
pub trait Inject {
    async fn inject(system: &System) -> Result<Self, InjectError> where Self: Sized;
}