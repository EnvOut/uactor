use std::collections::HashMap;
use std::sync::Arc;
use self::messages::AbstractMessage;
use self::supervisors::{Suscriber, Suscriber2};

// pub struct ActorSystem {
//     // mailboes
//     pub dispatchers: HashMap<String, MessageDispatcher>,
// }

pub struct MessageDispatcher {
    // rr: M,
    // rr1: C,
    pub subscribers: Arc<dyn Suscriber2<Message=()>>,
}

pub mod supervisors {
    use std::marker::PhantomData;
    use crate::v1::actors::{ActorErrors, ActorRef};
    use crate::v1::messages::AbstractMessage;

    pub struct Supervisor {
        pub active_actors: Vec<ActorRef>,
    }

    #[async_trait::async_trait]
    impl Suscriber for Supervisor {
        async fn send_message(&self, message: impl AbstractMessage) -> Result<(), ActorErrors> {
            let mut calls = Vec::with_capacity(self.active_actors.len());
            for child in self.active_actors.iter() {
                calls.push(child.send_message(message.clone()));
            }

            futures::future::join_all(calls).await.into_iter().collect::<Result<Vec<()>, ActorErrors>>()?;
            Ok(())
        }
    }

    #[async_trait::async_trait]
    pub trait Suscriber {
        async fn send_message(&self, message: impl AbstractMessage) -> Result<(), ActorErrors>;
    }

    #[async_trait::async_trait]
    pub trait Suscriber2 {
        type Message: AbstractMessage;
        async fn send_message(&self, message: Self::Message) -> Result<(), ActorErrors>;
    }

    #[async_trait::async_trait]
    pub trait Suscriber3: Sized {
        type Message: AbstractMessage;
        async fn send_message(&self, message: Self::Message) -> Result<(), ActorErrors>;
    }


    // impl<T: ?Sized + 'static> FromRequest for Data<T> {
    //     type Error = Error;
    //     type Future = Ready<Result<Self, Error>>;
}

pub mod messages {
    #[async_trait::async_trait]
    pub trait AbstractMessage: Send + Sync + Clone {
        async fn get_content(&self) -> Self;
    }

    #[async_trait::async_trait]
    impl AbstractMessage for String {
        async fn get_content(&self) -> Self {
            todo!()
        }
    }
}

pub mod actors {
    use crate::v1::messages::AbstractMessage;
    use crate::v1::supervisors::Suscriber;

    pub enum ActorErrors {}

    pub struct Actor {}

    pub struct ActorRef {}

    #[async_trait::async_trait]
    impl Suscriber for ActorRef {
        async fn send_message(&self, message: impl AbstractMessage) -> Result<(), ActorErrors> {

            todo!()
        }
    }
}

fn main() {}