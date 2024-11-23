// use crate::actor1::{Actor1, Actor1LocalProxy, Actor1Messages};
// use crate::actor2::{Actor2Impl, Actor2Messages};
use tokio::sync::mpsc::unbounded_channel;
use tokio::time::{sleep, Duration};
use crate::actor1::{Actor1, Actor1LocalProxy, Actor1Messages};
use crate::actor2::Actor2Messages;

pub mod example;
pub mod actor1 {
    use std::future::Future;
    use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
    use crate::actor2::{Actor2, Actor2LocalProxy, Actor2Messages};

    pub enum Actor1Messages {
        HandlePing(()),
        PingCounter((), tokio::sync::oneshot::Sender<usize>),
    }

    pub trait Actor1 {
        async fn handle_ping(&mut self, msg: ()) -> ();
        async fn ping_counter(&mut self, msg: ()) -> usize;
    }

    pub trait Actor1Ref {
        fn handle_ping(&self, msg: ()) -> Result<(), ()>;
        fn ping_counter(&self, msg: ()) -> impl Future<Output=Result<usize, ()>> + '_;
    }

    pub async fn run_actor1(sender: UnboundedSender<Actor1Messages>, channel: UnboundedReceiver<Actor1Messages>, actor2_ref: Actor2LocalProxy) -> Actor1LocalProxy {
        tokio::spawn(async move {
            let ping_counter = 0;
            let mut actor1 = Actor1Impl {
                channel,
                actor2_ref,
                ping_counter,
            };

            loop {
                let message = actor1.channel.recv().await.unwrap();
                match message {
                    Actor1Messages::HandlePing(msg) => {
                        println!("Actor1 received: Ping");
                        actor1.handle_ping(msg).await;
                    }
                    Actor1Messages::PingCounter(msg, sender) => {
                        println!("Actor1 received: PingCounter");
                        let value = actor1.ping_counter(msg).await;
                        sender.send(value).unwrap();
                    }
                }
            }
        });

        Actor1LocalProxy {
            channel: sender
        }
    }

    #[derive(Clone)]
    pub struct Actor1LocalProxy {
        channel: UnboundedSender<Actor1Messages>,
    }

    impl From<UnboundedSender<Actor1Messages>> for Actor1LocalProxy {
        fn from(channel: UnboundedSender<Actor1Messages>) -> Self {
            Self {
                channel
            }
        }
    }

    impl Actor1 for Actor1LocalProxy {
        async fn handle_ping(&mut self, msg: ()) -> () {
            self.channel.send(Actor1Messages::HandlePing(msg)).unwrap();
        }

        async fn ping_counter(&mut self, msg: ()) -> usize {
            let (tx, rx) = tokio::sync::oneshot::channel();
            self.channel.send(Actor1Messages::PingCounter(msg, tx)).unwrap();
            rx.await.unwrap()
        }
    }

    pub struct Actor1Impl {
        channel: UnboundedReceiver<Actor1Messages>,
        actor2_ref: Actor2LocalProxy,
        ping_counter: usize,
    }

    impl Actor1 for Actor1Impl {
        async fn handle_ping(&self, _: ()) -> () {
            self.ping_counter += 1;
            println!("Actor1 received: Ping");
            self.actor2_ref.handle_pong(()).await;
        }

        async fn ping_counter(&mut self, _: ()) -> usize {
            // let self_ref = self.clone();

            self.ping_counter
        }
    }
}

pub mod actor2 {
    use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
    use crate::actor1::{Actor1, Actor1LocalProxy};

    pub enum Actor2Messages {
        HandlePong(()),
    }

    pub trait Actor2 {
        async fn handle_pong(&mut self, msg: ()) -> ();
    }

    pub async fn run_actor2(sender: UnboundedSender<Actor2Messages>, channel: UnboundedReceiver<Actor2Messages>, actor1_ref: Actor1LocalProxy) -> Actor2LocalProxy {
        tokio::spawn(async move {
            let ping_counter = 0;
            let mut actor2 = Actor2Impl {
                channel,
                actor1_ref,
                ping_counter,
            };

            loop {
                let message = actor2.channel.recv().await.unwrap();
                match message {
                    Actor2Messages::HandlePong(msg) => {
                        println!("Actor2 received: Pong");
                        actor2.handle_pong(msg).await;
                    }
                }
            }
        });

        Actor2LocalProxy {
            channel: sender
        }
    }

    #[derive(Clone)]
    pub struct Actor2LocalProxy {
        channel: UnboundedSender<Actor2Messages>,
    }

    impl From<UnboundedSender<Actor2Messages>> for Actor2LocalProxy {
        fn from(channel: UnboundedSender<Actor2Messages>) -> Self {
            Self {
                channel
            }
        }
    }

    impl Actor2 for Actor2LocalProxy {
        async fn handle_pong(&mut self, msg: ()) -> () {
            self.channel.send(Actor2Messages::HandlePong(msg)).unwrap();
        }
    }

    pub struct Actor2Impl {
        channel:  UnboundedReceiver<Actor2Messages>,
        actor1_ref: Actor1LocalProxy,
        ping_counter: usize,
    }

    impl Actor2 for Actor2Impl {
        async fn handle_pong(&mut self, _: ()) -> () {
            println!("Actor2 received: Pong");
            self.actor1_ref.handle_ping(()).await;
        }
    }
}

#[tokio::main]
async fn main() {
    let (actor1_tx, actor1_rx) = unbounded_channel::<Actor1Messages>();
    let (actor2_tx, actor2_rx) = unbounded_channel::<Actor2Messages>();

    let mut actor1 = actor1::run_actor1(
        actor1_tx.clone(), actor1_rx,
        actor2_tx.clone().into()
    ).await;

    let mut actor2 = actor2::run_actor2(actor2_tx, actor2_rx, actor1.clone()).await;

    actor1.handle_ping(()).await;

    sleep(Duration::from_secs(1)).await;

    let ping_count = actor1.ping_counter(()).await;
    println!("Ping count: {}", ping_count);
}



// struct Service1 {
//     actor: Actor1LocalProxy,
//     actor_dyn: Box<dyn Actor1>,
// }
//
// impl Service1 {
//     async fn do_something(&self) {
//         let mut actor = self.actor.clone();
//         actor.handle_ping(()).await;
//
//         self.actor.handle_ping(()).await;
//     }
// }
//
// // То чего не хватает:
// // можно слушать каналы
// // можно слушать таймер
//
// // Идеи:
// // 1. Builder
// // let builder = Actor2.builder(actor2_tx, actor2_rx, actor1.clone())
// // let actor2 = builder.reference(&self)
// // builder.run(self)
// //
// // 2. Получение актор с помощью статического метода
// // let actors: Vec<(String, Box<dyn Actor2>)> = Actor2::mailboxext()
// // let actro2_1: Box<dyn Actor2> = Actor2::mailboxext_by_name("fdjsfkjds-fdsf-fdsmf")
