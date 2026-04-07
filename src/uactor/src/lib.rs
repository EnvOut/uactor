pub mod dependency_injection;
pub mod system;

pub use paste::paste;
pub use uactor_derive::{actor, handler, Message};

pub mod actor;
pub mod aliases;
pub mod data;

/// Common imports for working with uactor actors.
///
/// ```
/// use uactor::prelude::*;
/// ```
pub mod prelude {
    pub use crate::actor::abstract_actor::{
        Actor, HandleError, HandleResult, Handler, MessageSender,
    };
    pub use crate::actor::context::{ActorContext, Context};
    pub use crate::actor::message::{Message, Reply};
    pub use crate::system::System;
    pub use crate::{actor, handler, Message};
}
