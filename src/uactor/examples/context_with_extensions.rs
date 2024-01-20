use uactor::system::System;
use crate::actor1::Actor1;
use crate::messages::{PingMsg, PongMsg};
use crate::actor1::Actor1Msg;
use crate::actor1::Actor1Ref;
use crate::actor2::Actor2;
use crate::actor2::Actor2Msg;
use crate::actor2::Actor2Ref;
use crate::services::{Service1, Service2};

mod messages {
    use tokio::sync::oneshot::Sender;
    use uactor::message::Message;

    pub struct PingMsg(pub Sender<PongMsg>);
    #[derive(Debug)]
    pub struct PongMsg;

    uactor::message_impl!(PingMsg, PongMsg);
}

mod actor1 {
    use uactor::actor::{Actor, Handler, HandleResult};
    use uactor::context::Context;
    use uactor::context::extensions::Extension;
    use crate::messages::{PingMsg, PongMsg};
    use crate::services::Service1;

    pub struct Actor1;

    impl Actor for Actor1 { type Context = Context; }

    #[async_trait::async_trait]
    impl Handler<PingMsg> for Actor1 {
        async fn handle(&mut self, ping: PingMsg, ctx: &mut Context) -> HandleResult {
            println!("actor1: Received ping message");

            let Extension(service1) = ctx.get_extension::<Service1>()?;
            service1.do_something();

            let PingMsg(reply) = ping;
            let _ = reply.send(PongMsg);
            Ok(())
        }
    }

    uactor::generate_actor_ref!(Actor1, { PingMsg });
}

mod actor2 {
    use uactor::actor::{Actor, Handler, HandleResult};
    use uactor::context::Context;
    use uactor::context::extensions::Extension;
    use crate::messages::{PingMsg, PongMsg};
    use crate::services::{Service1, Service2};

    pub struct Actor2;

    impl Actor for Actor2 { type Context = Context; }

    #[async_trait::async_trait]
    impl Handler<PingMsg> for Actor2 {
        async fn handle(&mut self, ping: PingMsg, ctx: &mut Context) -> HandleResult {
            println!("actor2: Received ping message");

            let Extension(service2) = ctx.get_extension::<Service2>()?;
            service2.do_something();

            let PingMsg(reply) = ping;
            let _ = reply.send(PongMsg);
            Ok(())
        }
    }

    uactor::generate_actor_ref!(Actor2, { PingMsg });
}

pub mod services {
    pub struct Service1 {
        // repository
        // other services
    }

    impl Service1 {
        pub fn do_something(&self) {
            println!("Service1: Called do_something");
        }
    }


    pub struct Service2 {
        // repository
        // other services
    }

    impl Service2 {
        pub fn do_something(&self) {
            println!("Service2: Called do_something");
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let service1 = Service1 {};
    let service2 = Service2 {};

    let system = System::global()
        .extension(service1)
        .extension(service2).build();

    let actor1 = Actor1;
    let (mut actor1_ref, _) = uactor::spawn_with_ref!(system, actor1: Actor1);

    let actor2 = Actor2;
    let (mut actor2_ref, _) = uactor::spawn_with_ref!(system, actor2: Actor2);

    let pong1 = actor1_ref.ask_ping_msg::<PongMsg>(|reply| PingMsg(reply)).await?;
    let pong2 = actor2_ref.ask_ping_msg::<PongMsg>(|reply| PingMsg(reply)).await?;
    println!("main: received {pong1:?} and {pong2:?} messages");

    Ok(())
}