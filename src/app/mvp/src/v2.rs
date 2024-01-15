use std::collections::HashMap;
use std::sync::Arc;
use self::messages::AbstractMessage;
use self::supervisors::{Subscriber};

pub struct MessageDispatcher {
    pub subscribers: Arc<dyn Subscriber<Message=/**erase generic type**/>>,
}

pub mod supervisors {
    use std::marker::PhantomData;
    use std::sync::Arc;
    use crate::v2::objects::ObjectErrors;
    use crate::v2::messages::AbstractMessage;

    pub struct Supervisor<M> where M: AbstractMessage {
        children: Vec<Arc<dyn Subscriber<Message=M>>>,
    }

    #[async_trait::async_trait]
    impl<M> Subscriber for Supervisor<M> where M: AbstractMessage {
        type Message = M;

        async fn send_message(&self, message: impl AbstractMessage) -> Result<(), ObjectErrors> {
            let mut calls = Vec::with_capacity(self.children.len());
            for child in self.children.iter() {
                calls.push(child.send_message(message.clone()));
            }

            futures::future::join_all(calls).await.into_iter().collect::<Result<Vec<()>, ObjectErrors>>()?;
            Ok(())
        }
    }

    #[async_trait::async_trait]
    pub trait Subscriber: Sized {
        type Message: AbstractMessage;

        async fn send_message(&self, message: Self::Message) -> Result<(), ObjectErrors>;
    }
}

pub mod messages {
    #[async_trait::async_trait]
    pub trait AbstractMessage: Send + Sync + Clone + Sized {
        async fn get_content(&self) -> Self;
    }

    #[async_trait::async_trait]
    impl AbstractMessage for String {
        async fn get_content(&self) -> Self {
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
        async fn get_content(&self) -> Self {
            todo!()
        }
    }
}

pub mod objects {
    use crate::v2::messages::{AbstractMessage, ComplexMessage};
    use crate::v2::supervisors::Subscriber;

    pub enum ObjectErrors {}

    pub struct Object1 {}

    pub struct Object2 {}

    #[async_trait::async_trait]
    impl Subscriber for Object1 {
        type Message = String;

        async fn send_message(&self, message: Self::Message) -> Result<(), ObjectErrors> {
            todo!()
        }
    }

    #[async_trait::async_trait]
    impl Subscriber for Object2 {
        type Message = ComplexMessage;

        async fn send_message(&self, message: Self::Message) -> Result<(), ObjectErrors> {
            todo!()
        }
    }
}

#[tokio::main]
async fn main() {
    let mut map: HashMap<String, Box<dyn Fn() -> Box<dyn Subscriber>>> = HashMap::new();
    map.insert("Object1".to_string(), Box::new(|| Box::new(objects::Object1 {})));
    map.insert("Object2".to_string(), Box::new(|| Box::new(objects::Object2 {})));

    let mut children = Vec::new();
    for (name, factory) in map.iter() {
        children.push(factory());
    }

    let supervisor = supervisors::Supervisor::<String> {
        children,
    };

    let message = String::from("Hello world");
    let result = supervisor.send_message(message).await;
    println!("{:?}", result);
}