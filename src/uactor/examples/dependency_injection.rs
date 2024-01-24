use crate::actor1::Actor1;
use crate::actor1::Actor1Msg;
use crate::actor1::Actor1Ref;
use crate::actor2::Actor2;
use crate::actor2::Actor2Msg;
use crate::actor2::Actor2Ref;
use crate::messages::{PingMsg, PongMsg};
use crate::services::{Service1, Service2};
use uactor::system::System;

mod messages {
    use tokio::sync::oneshot::Sender;
    use uactor::message::Message;

    pub struct PingMsg(pub Sender<PongMsg>);
    #[derive(Debug)]
    pub struct PongMsg;

    uactor::message_impl!(PingMsg, PongMsg);
}

mod actor1 {
    use crate::messages::{PingMsg, PongMsg};
    use crate::services::Service1;
    use uactor::actor::{Actor, HandleResult, Handler};
    use uactor::context::Context;
    use uactor::di::{Inject, InjectError};
    use uactor::system::System;

    pub struct Actor1;

    pub struct Services(Service1);

    impl Inject for Services {
        async fn inject(system: &System) -> Result<Self, InjectError>
            where
                Self: Sized,
        {
            let service1 = system.get_service()?;
            Ok(Services(service1))
        }
    }

    impl Actor for Actor1 {
        type Context = Context;
        type Inject = Services;
    }

    impl Handler<PingMsg> for Actor1 {
        async fn handle(&mut self, Services(service1): &mut Self::Inject, ping: PingMsg, ctx: &mut Context) -> HandleResult {
            println!("actor1: Received ping message");

            service1.do_something();

            let PingMsg(reply) = ping;
            let _ = reply.send(PongMsg);
            Ok(())
        }
    }

    uactor::generate_actor_ref!(Actor1, { PingMsg });
}

mod actor2 {
    use crate::messages::{PingMsg, PongMsg};
    use crate::services::{Service1, Service2};
    use uactor::actor::{Actor, ActorPreStartResult, HandleResult, Handler};
    use uactor::context::extensions::Service;
    use uactor::context::Context;
    use uactor::di::{Inject, InjectError};
    use uactor::system::System;

    pub struct Actor2;

    pub struct Services(Service1);

    impl Inject for Services {
        async fn inject(system: &System) -> Result<Self, InjectError>
            where
                Self: Sized,
        {
            let service2 = system.get_service()?;
            Ok(Services(service2))
        }
    }

    impl Actor for Actor2 {
        type Context = Context;
        type Inject = Services;
    }

    impl Handler<PingMsg> for Actor2 {
        async fn handle(&mut self, Services(service2): &mut Self::Inject, ping: PingMsg, ctx: &mut Context) -> HandleResult {
            println!("actor2: Received ping message");

            service2.do_something();

            let PingMsg(reply) = ping;
            let _ = reply.send(PongMsg);
            Ok(())
        }
    }

    uactor::generate_actor_ref!(Actor2, { PingMsg });
}

pub mod services {
    #[derive(Clone)]
    pub struct Service1 {
        // repository
        // other services
    }

    impl Service1 {
        pub fn do_something(&self) {
            println!("Service1: Called do_something");
        }
    }

    #[derive(Clone)]
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

    let mut system = System::global()
        .extension(service1)
        .extension(service2)
        .build();

    let actor1 = Actor1;
    let (actor1_ref, _) = uactor::spawn_with_ref!(system, actor1: Actor1);

    let actor2 = Actor2;
    let (actor2_ref, _) = uactor::spawn_with_ref!(system, actor2: Actor2);

    system.run_actor::<Actor1>(actor1_ref.name()).await;
    system.run_actor::<Actor2>(actor2_ref.name()).await;

    let pong1 = actor1_ref
        .ask_ping_msg::<PongMsg>(|reply| PingMsg(reply))
        .await?;
    let pong2 = actor2_ref
        .ask_ping_msg::<PongMsg>(|reply| PingMsg(reply))
        .await?;
    println!("main: received {pong1:?} and {pong2:?} messages");

    Ok(())
}
