#[tokio::test]
async fn test() {
    // let theatre = theatre::Theatre{};
    let actor = theatre::Theatre::take_actor::<impl actor1::Actor1>()
        .await
        .unwrap();

    actor.handle_ping("Hello".to_string()).await;
    println!("Hello, world!");
}

pub mod theatre {
    #[derive(Debug)]
    pub enum TakeActorError {
        NoActor,
    }

    pub struct Theatre {}

    impl Theatre {
        pub async fn spawn_actor<A>() {
            println!("Spawned actor");
        }

        pub async fn take_actor<T>() -> Result<T, TakeActorError> {
            todo!()
            // let actor: T = todo!();
            // Ok(Box::new(actor))
        }
    }
}

pub mod actor1 {
    pub trait Actor1 {
        async fn handle_ping(&self, msg: String) -> ();
    }

    pub mod actor1_impl {
        pub trait Actor1MutImpl {
            async fn handle_ping(&mut self, msg: String) -> ();
        }

        pub struct Actor1Impl {
            ping_counter: u32,
        }

        impl Actor1MutImpl for Actor1Impl {
            async fn handle_ping(&mut self, msg: String) -> () {
                self.ping_counter += 1;
                println!("Actor1 received: Ping and message: {}", msg);
            }
        }
    }

    pub mod actor1_wrap {
        mod actor1_wrap_mpsc {
            use super::super::Actor1;

            pub struct Actor1MpscWrap {
                channel: tokio::sync::mpsc::UnboundedSender<String>,
            }

            impl Actor1 for Actor1MpscWrap {
                async fn handle_ping(&self, msg: String) -> () {
                    self.channel.send(msg);
                }
            }
        }

        pub mod actor1_wrap_remote {
            use super::super::Actor1;

            pub struct Actor1RemoteWrap {
                channel: tokio::sync::mpsc::UnboundedSender<String>,
            }

            impl Actor1 for Actor1RemoteWrap {
                // If not Result<_, Err> then ignore even if error. Only log error.
                async fn handle_ping(&self, msg: String) -> () {
                    self.channel.send(msg);
                }
                // // If return Result<_, Err> then error should implement From::from Actor sending error
                // async fn handle_ping(&self, msg: String) -> Result<(), Err> {
                //     self.channel.send(msg);
                // }

                // // if it has Result<Data, _> or Data without Result then it should be handled by the caller with oneshot
                // async fn handle_ping(&self, msg: String) -> oneshot::Receiver<Result<(), Err>> {
                //     let (tx, rx) = oneshot::channel();
                //     // timout..
                //     self.channel.send(msg, tx);
                //     timout(duration, rx.await.into())
                // }
            }
        }
    }
}
