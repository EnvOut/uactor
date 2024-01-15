use self::supervisors::Subscriber;
use crate::actors::{Actor1, ActorRef};
use crate::dispatchers::SubscriptionProvider;
use crate::dispatchers::{BrodcastMessagesDispatcher, MessagesDispatcher};
use crate::etc::NoOpHasher;
use crate::messages::AbstractMessage;
use crate::supervisors::Supervisor;
use crate::system::ActorSystem;
use ractor::Message;
use std::any::{Any, TypeId};
use std::hash::BuildHasherDefault;
use std::{collections::HashMap, sync::Arc};

pub mod etc {
    use std::hash::Hasher;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[derive(Debug, Default)]
    pub struct NoOpHasher(u64);

    impl Hasher for NoOpHasher {
        fn finish(&self) -> u64 {
            self.0
        }

        fn write(&mut self, _bytes: &[u8]) {
            unimplemented!("This NoOpHasher can only handle u64s")
        }

        fn write_u64(&mut self, i: u64) {
            self.0 = i;
        }
    }

    pub type MutArc<T> = Arc<RwLock<T>>;
}

pub mod system {
    use crate::actors::ActorErrors;
    use crate::dispatchers::MessagesDispatcher;
    use crate::etc::MutArc;
    use crate::messages::AbstractMessage;
    use anyhow::{Context, Error};
    use std::any::TypeId;
    use std::collections::HashMap;
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[derive(thiserror::Error, Debug)]
    pub enum ActorSystemErrors {
        #[error("Invalid message")]
        InvalidMessage,

        #[error(transparent)]
        ActorErrors(#[from] ActorErrors),

        #[error(transparent)]
        AnyhowError(#[from] anyhow::Error),
    }

    #[derive(Default)]
    pub struct ActorSystem {
        // mailboxes
        pub dispatchers: MutArc<HashMap<TypeId, Arc<dyn MessagesDispatcher>>>,
    }

    impl ActorSystem {
        pub async fn send_message<M: AbstractMessage>(
            &self,
            message: M,
        ) -> Result<(), ActorSystemErrors> {
            let type_id = TypeId::of::<M>();
            let dispatchers = self.dispatchers.clone();
            let dispatchers = dispatchers.read().await;

            let dispatcher: Arc<dyn MessagesDispatcher + 'static> = dispatchers
                .get(&type_id)
                .map(|it| it.clone())
                .context("no dispatchers for this type of message")?;
            dispatcher.dispatch(Arc::new(message)).await?;
            Ok(())
        }

        pub async fn add_dispatcher<D, M>(&mut self, dispatcher: D) -> Result<(), ActorSystemErrors>
            where
                D: MessagesDispatcher + 'static,
                M: AbstractMessage,
        {
            let type_id = TypeId::of::<M>();
            let mut dispatchers = self.dispatchers.write().await;
            dispatchers.insert(type_id, Arc::new(dispatcher));
            Ok(())
        }
    }
}

pub mod dispatchers {
    use crate::actors::ActorErrors;
    use crate::etc::{MutArc, NoOpHasher};
    use crate::messages::AbstractMessage;
    use crate::supervisors::Subscriber;
    use async_trait::async_trait;
    use futures::{FutureExt, StreamExt, TryStreamExt};
    use std::any::{Any, TypeId};
    use std::collections::HashMap;
    use std::future::Future;
    use std::hash::BuildHasherDefault;
    use std::pin::Pin;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[async_trait::async_trait]
    pub trait SubscriptionProvider<M: AbstractMessage> {
        async fn subscribe(
            &mut self,
            subscriber: impl Subscriber<Message=M> + 'static,
        );
    }

    #[async_trait::async_trait]
    pub trait MessagesDispatcher {
        async fn dispatch(&self, message: Arc<dyn Any + Send + Sync>) -> Result<(), ActorErrors>;
    }

    #[derive(Default)]
    pub struct BrodcastMessagesDispatcher<M>
        where
            M: AbstractMessage,
    {
        pub message_type_subscriber_map: MutArc<HashMap<TypeId, Arc<dyn Subscriber<Message=M>>>>,
    }

    #[async_trait::async_trait]
    impl<M: AbstractMessage> SubscriptionProvider<M> for BrodcastMessagesDispatcher<M> {
        async fn subscribe(
            &mut self,
            subscriber: impl Subscriber<Message=M> + 'static,
        ) {
            let type_id = TypeId::of::<M>();
            let subscriber: Arc<dyn Subscriber<Message=M>> = Arc::new(subscriber);
            let mut message_type_subscriber_map = self.message_type_subscriber_map.write().await;

            let option = message_type_subscriber_map.insert(type_id, subscriber.clone());
            println!(
                "message_type_subscriber_map count: {:?}",
                message_type_subscriber_map.len()
            );
        }
    }

    #[async_trait::async_trait]
    impl<T: AbstractMessage> MessagesDispatcher for BrodcastMessagesDispatcher<T> {
        async fn dispatch(&self, message: Arc<dyn Any + Send + Sync>) -> Result<(), ActorErrors> {
            let message_type_subscriber_map = self.message_type_subscriber_map.read().await;

            let values = message_type_subscriber_map.values();

            let mut futures = Vec::with_capacity(values.len());
            for subscriber in values {
                let message = message.clone();
                let message = subscriber.cast_message(message).await?;

                futures.push(async move { subscriber.on_message(message).await });
            }

            futures::future::join_all(futures)
                .await
                .into_iter()
                .collect::<Result<Vec<()>, ActorErrors>>()?;
            Ok(())
        }
    }
}

pub mod supervisors {
    use crate::actors::{Actor1, Actor2, ActorErrors};
    use crate::messages::AbstractMessage;
    use anyhow::Context;
    use async_trait::async_trait;
    use std::any::Any;
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Arc;

    pub struct Supervisor<M>
        where
            M: AbstractMessage,
    {
        pub children: Vec<Arc<dyn Subscriber<Message=M>>>,
    }

    #[async_trait::async_trait]
    impl<M> Subscriber for Supervisor<M>
        where
            M: AbstractMessage,
    {
        type Message = M;

        async fn on_message(&self, message: Arc<Self::Message>) -> Result<(), ActorErrors> {
            let mut calls = Vec::with_capacity(self.children.len());
            for child in self.children.iter() {
                calls.push(child.on_message(message.clone()));
            }

            futures::future::join_all(calls)
                .await
                .into_iter()
                .collect::<Result<Vec<()>, ActorErrors>>()?;
            Ok(())
        }
    }

    #[async_trait::async_trait]
    pub trait Subscriber: Send + Sync {
        type Message: AbstractMessage;
        async fn on_message(&self, message: Arc<Self::Message>) -> Result<(), ActorErrors>;

        async fn cast_message(
            &self,
            message: Arc<dyn Any + Send + Sync>,
        ) -> Result<Arc<Self::Message>, ActorErrors> {
            match message.downcast::<Self::Message>() {
                Ok(message) => Ok(message),
                Err(message) => Err(ActorErrors::CantCastMessage(message)),
            }
        }
    }
}

pub mod messages {
    #[async_trait::async_trait]
    pub trait AbstractMessage: Send + Sync + 'static {
        async fn get_content(&self);
    }

    #[async_trait::async_trait]
    impl AbstractMessage for String {
        async fn get_content(&self) {
            todo!()
        }
    }

    #[derive(Clone)]
    pub struct ComplexMessage {
        pub content: String,
        pub items: Vec<i32>,
    }

    #[async_trait::async_trait]
    impl AbstractMessage for ComplexMessage {
        async fn get_content(&self) {
            todo!()
        }
    }
}

pub mod actors {
    use anyhow::Context;
    use std::any::Any;
    use std::marker::PhantomData;
    use std::sync::Arc;

    use crate::messages::{AbstractMessage, ComplexMessage};
    use crate::supervisors::Subscriber;
    use crate::system::{ActorSystem, ActorSystemErrors};

    #[derive(thiserror::Error, Debug)]
    pub enum ActorErrors {
        #[error("Invalid message")]
        InvalidMessage,

        #[error("Failed to cast message")]
        CantCastMessage(Arc<dyn Any + Send + Sync>),

        #[error(transparent)]
        AnyhowError(#[from] anyhow::Error),
    }

    pub struct ActorRef<'a, T> where T: Subscriber {
        inner: PhantomData<T>,
        system: &'a ActorSystem,
    }

    impl<'a, T> ActorRef<'a, T> where T: Subscriber {
        pub fn new(system: &'a ActorSystem) -> Self {
            Self {
                inner: PhantomData,
                system,
            }
        }

        pub async fn send_message(&self, message: T::Message) -> Result<(), ActorSystemErrors> {
            self.system.send_message(message).await
        }
    }

    pub struct Actor1 {}

    pub struct Actor2 {}

    pub struct Actor3 {}

    #[async_trait::async_trait]
    impl Subscriber for Actor1 {
        type Message = String;
        async fn on_message(&self, message: Arc<Self::Message>) -> Result<(), ActorErrors> {
            let value = message
                .trim()
                .parse::<i32>()
                .map_err(|_| ActorErrors::InvalidMessage)?;
            // self.service.do_something(value).await;

            println!("Actor1: {}", value);
            Ok(())
        }
    }

    #[async_trait::async_trait]
    impl Subscriber for Actor2 {
        type Message = ComplexMessage;

        async fn on_message(&self, message: Arc<Self::Message>) -> Result<(), ActorErrors> {
            let ComplexMessage { content, items } = message.as_ref();
            println!("Actor2: {}", content);
            // self.service.do_something_else(content, items).await;
            Ok(())
        }
    }

    #[async_trait::async_trait]
    impl Subscriber for Actor3 {
        type Message = String;
        async fn on_message(&self, message: Arc<Self::Message>) -> Result<(), ActorErrors> {
            let value = message
                .trim()
                .parse::<i32>()
                .map_err(|_| ActorErrors::InvalidMessage)?;
            println!("Actor3: {}", value);
            // self.service.do_something(value).await;
            Ok(())
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let supervisor = Supervisor {
        children: vec![Arc::new(actors::Actor1 {}), Arc::new(actors::Actor3 {})],
    };

    let mut dispatcher: BrodcastMessagesDispatcher<String> = BrodcastMessagesDispatcher::default();
    dispatcher.subscribe(supervisor).await;

    let mut system = ActorSystem::default();
    system
        .add_dispatcher::<BrodcastMessagesDispatcher<String>, String>(dispatcher)
        .await?;

    system.send_message("1231".to_string()).await?;

    let actor1_ref: ActorRef<Actor1> = ActorRef::new(&system);
    actor1_ref.send_message("333".to_string()).await?;

    Ok(())
}
