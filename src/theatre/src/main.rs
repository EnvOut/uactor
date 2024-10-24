pub mod sample;

fn main() {
    println!("Hello, world!");
}

mod tests {
    use crate::theatre::Theatre;
    use crate::user_module::Actor1Impl;
    // mod rr {
    //     struct Service {
    //         actor1: Box<dyn Actor1>,
    //     }
    //
    //     impl Service {
    //         fn method1(&self) -> Self {
    //             self.actor1.calculate_invoice_price(10);
    //             self.actor2.calculate_invoice_price(10);
    //             self.actor3.calculate_invoice_price(10);
    //         }
    //     }
    // }
    //
    // mod rr1 {
    //     struct Actor3<input: Pong, Request, output: Response> {
    //
    //     }
    // }

    #[test]
    fn base_test() {
        let mut theatre1 = Theatre::new();
        let mut theatre2 = Theatre::new();
        let mut theatre3 = Theatre::new();

        theatre1.connect_theatre(&theatre2);
        theatre1.connect_theatre(&theatre3);

        theatre2.connect_theatre(&theatre1);
        theatre2.connect_theatre(&theatre3);

        theatre3.connect_theatre(&theatre1);
        theatre3.connect_theatre(&theatre2);

        // theatre1.register_local_actor("actor1", Actor1Impl {});

        // let actor1 = user_module::Actor1Impl {};
        // let msg = generated_actor::Actor1Msg::CalculateInvoicePrice(10);
        // generated_actor::call_handler(actor1, msg);
    }
}

pub mod user_module {
    pub trait Actor1 {
        fn calculate_invoice_price(&self, msg: u32);
        fn calculate_something(&self, msg: String);
    }

    pub struct Actor1Impl {}

    // 1. Actor должен иметь слушать несколько каналов
    // 2.
    impl Actor1 for Actor1Impl {
        fn calculate_invoice_price(&self, msg: u32) {
            todo!()
        }

        fn calculate_something(&self, msg: String) {
            todo!()
        }
    }
}

pub mod generated_actor {
    use crate::user_module::{Actor1, Actor1Impl};

    pub enum Actor1Msg {
        CalculateInvoicePrice(u32),
        CalculateSomething(String),
    }

    pub fn message_dispatcher(actor: &Actor1Impl, msg: Actor1Msg) {
        match msg {
            Actor1Msg::CalculateInvoicePrice(msg) => actor.calculate_invoice_price(msg),
            Actor1Msg::CalculateSomething(msg) => actor.calculate_something(msg),
        }
    }
}

pub mod theatre {
    use derive_new::new;
    use std::any::{type_name, Any};
    use std::collections::HashMap;
    use std::future::Future;
    use std::pin::Pin;
    use futures::stream::FuturesUnordered;
    use futures::StreamExt;
    use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
    use uactor::datasource::{DataSource, DataSourceResult};
    use uactor::message::Message;

    pub trait Actor: Send + Sync {
        type Message: Send + Sync;
    }

    pub enum ActorControlMessage<A: Actor> {
        // ListenChannel(Box<dyn uactor::datasource::DataSource>),
        ListenChannel(Box<dyn uactor::datasource::DataSource<Item = A::Message>>),
    }

    // type Message = ActorControlMessage<Box<dyn Actor<Message = M>>>;
    #[derive(new)]
    pub struct Theatre {
        #[new(value = "uuid::Uuid::new_v4().to_string()")]
        name: String,
        // #[new(default)]
        // name_actor: HashMap<String, Box<dyn Any>>,
        // #[new(default)]
        // type_actor: HashMap<String, Box<dyn Any>>,
        #[new(default)]
        actor_command_channel: HashMap<String, Box<dyn Any>>,
    }

    impl Theatre {
        pub fn local_actor<T: 'static>(&self, name: &str) -> Option<&T> {
            todo!()
        }

        // fn init_channel<A, D>() -> Vec<Pin<Box<dyn DataSource<Item = A::Message>>>>
        // where
        //     A:Actor + Send + Sync + 'static,
        //     D: uactor::datasource::DataSource<Item = A::Message> + Send + Sync + 'static,
        // {
        //     Vec::new()
        // }

        pub fn register_local_actor<A>(&mut self, name: &str, actor: A)
        where
            A: Actor + Send + Sync + 'static,
        {
            let type_name = type_name::<A>().to_owned();

            // prepare channel for commands to actor
            let (command_rx, mut command_rc): (UnboundedSender<ActorControlMessage<A>>, UnboundedReceiver<ActorControlMessage<A>>) = tokio::sync::mpsc::unbounded_channel::<ActorControlMessage<A>>();
            self.actor_command_channel.insert(name.to_string(), Box::new(command_rx.clone()));

            tokio::spawn(async move {
                let mut actor = actor;

                let mut channels: HashMap<String, UnboundedReceiver<A::Message>> = HashMap::new();

                let mut channels: Vec<Pin<Box<dyn DataSource<Item=A::Message>>>> = Vec::new();
                let mut command_rc = command_rc;

                let mut futures: FuturesUnordered<Pin<Box<dyn Future<Output=DataSourceResult<<A as Actor>::Message>> + Send>>> = FuturesUnordered::new();
                channels.iter_mut()
                    .map(|channel| channel.next())
                    .for_each(|f| futures.push(Box::pin(f)));
                loop {
                    tokio::select! {
                        command = command_rc.recv() => {
                            let mut c = command.unwrap();

                            println!("Received command");
                        }
                        // _ = futures.next() => {
                        //     println!("Received message");
                        // }
                    }

                    // actor.calculate_invoice_price(10);
                    // actor.calculate_something("Hello".to_string());

                    println!("Actor is running");
                }
            });

            // println!("Type name: {}", type_name);
            // self.name_actor.insert(name.to_string(), actor);
            // self.type_actor.insert(type_name, actor);
        }

        pub fn connect_theatre(&self, other: &Theatre) {
            // self.actors.insert()
        }
    }
}

pub mod generated_proxy {
    use crate::user_module::Actor1;

    pub struct Actor1LocalProxy {
        chanel: tokio::sync::mpsc::Sender<u32>,
    }

    impl Actor1 for Actor1LocalProxy {
        fn calculate_invoice_price(&self, msg: u32) {
            // self.chanel.send(msg)
        }

        fn calculate_something(&self, msg: String) {
            todo!()
        }
    }

    pub struct Actor1RemoteProxy {
        // chanel: ..
    }

    impl Actor1 for Actor1RemoteProxy {
        fn calculate_invoice_price(&self, msg: u32) {
            // chanel.send(msg)
            // chanel.ser
            todo!()
        }

        fn calculate_something(&self, msg: String) {
            todo!()
        }
    }
}
