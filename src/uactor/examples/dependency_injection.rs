use time::ext::NumericalStdDuration;
use uactor::actor::abstract_actor::MessageSender;

use uactor::system::System;

use crate::actor1::{Actor1, Actor1MpscRef};
use crate::actor2::{Actor2, Actor2MpscRef};
use crate::messages::{MessageWithoutReply, PingMsg, PongMsg};
use crate::services::{Service1, Service2};

mod messages {
    use uactor::actor::message::{Message, Reply};

    pub struct PingMsg(pub Reply<PongMsg>);

    #[derive(derive_more::Into, Debug)]
    pub struct MessageWithoutReply(pub String);

    #[derive(derive_more::Constructor, Debug)]
    pub struct PrintMessage(pub String);

    #[derive(Debug)]
    pub struct PongMsg;

    uactor::message_impl!(PingMsg, PongMsg, MessageWithoutReply, PrintMessage);
}

mod actor1 {
    use uactor::actor::abstract_actor::{Actor, HandleResult, Handler, MessageSender};
    use uactor::actor::context::extensions::Service;
    use uactor::actor::context::Context;
    use uactor::dependency_injection::{Inject, InjectError};
    use uactor::system::System;

    use crate::actor2::{Actor2, Actor2MpscRef};
    use crate::messages::{MessageWithoutReply, PingMsg, PongMsg, PrintMessage};
    use crate::services::Service1;

    pub struct Actor1;

    #[derive(derive_more::Constructor)]
    pub struct Services {
        service1: Service<Service1>,
        actor2_ref: Actor2MpscRef,
    }

    impl Inject for Services {
        async fn inject(system: &System) -> Result<Self, InjectError>
        where
            Self: Sized,
        {
            let service1 = system.get_service()?;
            let actor2_ref = system.get_actor::<Actor2, _, _, _>("actor2".into())?;
            Ok(Services::new(service1, actor2_ref))
        }
    }

    impl Actor for Actor1 {
        type Context = Context;
        type RouteMessage = Actor1Msg;
        type Inject = Services;
        type State = ();
    }

    impl Handler<PingMsg> for Actor1 {
        async fn handle(
            &mut self,
            Services { service1, .. }: &mut Self::Inject,
            ping: PingMsg,
            _ctx: &mut Context,
            _state: &Self::State,
        ) -> HandleResult {
            println!("actor1: Received ping message");

            service1.do_something();

            let PingMsg(reply) = ping;
            let _ = reply.send(PongMsg);
            Ok(())
        }
    }

    impl Handler<MessageWithoutReply> for Actor1 {
        async fn handle(
            &mut self,
            Services { actor2_ref, .. }: &mut Self::Inject,
            msg: MessageWithoutReply,
            _ctx: &mut Context,
            _state: &Self::State,
        ) -> HandleResult {
            println!("actor1: Received {msg:?} message, sending PrintMessage to the actor2");
            actor2_ref.send(PrintMessage::new(msg.into()))?;
            Ok(())
        }
    }

    uactor::generate_actor_ref!(Actor1, { PingMsg, MessageWithoutReply });
}

mod actor2 {
    use uactor::actor::abstract_actor::{Actor, HandleResult, Handler};
    use uactor::actor::context::extensions::Service;
    use uactor::actor::context::Context;
    use uactor::dependency_injection::{Inject, InjectError};
    use uactor::system::System;

    use crate::messages::{PingMsg, PongMsg, PrintMessage};
    use crate::services::Service2;

    pub struct Actor2;

    #[derive(derive_more::Constructor)]
    pub struct Services(Service<Service2>);

    impl Inject for Services {
        async fn inject(system: &System) -> Result<Self, InjectError>
        where
            Self: Sized,
        {
            let service2 = system.get_service::<Service2>()?;
            Ok(Services(service2))
        }
    }

    impl Actor for Actor2 {
        type Context = Context;
        type RouteMessage = Actor2Msg;
        type Inject = Services;
        type State = ();
    }

    impl Handler<PingMsg> for Actor2 {
        async fn handle(
            &mut self,
            Services(service2): &mut Self::Inject,
            ping: PingMsg,
            _ctx: &mut Context,
            _state: &Self::State,
        ) -> HandleResult {
            println!("actor2: Received ping message");

            service2.do_something();

            let PingMsg(reply) = ping;
            let _ = reply.send(PongMsg);
            Ok(())
        }
    }

    impl Handler<PrintMessage> for Actor2 {
        async fn handle(
            &mut self,
            _: &mut Self::Inject,
            PrintMessage(msg): PrintMessage,
            _ctx: &mut Context,
            _state: &Self::State,
        ) -> HandleResult {
            println!("actor2: Received message: {msg:?}");
            Ok(())
        }
    }

    uactor::generate_actor_ref!(Actor2, { PingMsg, PrintMessage });
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
    // Init services
    let service1 = Service1 {};
    let service2 = Service2 {};

    // Init system and register services
    let mut system = System::global()
        .with(|system| {
            system.extension(service1.clone())
                .extension(service2.clone());
        }).await;

    // Init actor (instance + spawn actor)
    let (actor1_ref, actor1_stream) = system.register_ref::<Actor1, _, Actor1MpscRef>("actor1").await;

    // Init actor2 (instance + spawn actor)
    let (actor2_ref, actor2_stream) = system.register_ref::<Actor2, _, Actor2MpscRef>("actor2").await;

    // Run actors
    let actor1 = Actor1;
    system.spawn_actor(actor1_ref.name(), actor1, *actor1_ref.state(), actor1_stream).await?;

    let actor2 = Actor2;
    system.spawn_actor(actor2_ref.name(), actor2, *actor2_ref.state(), actor2_stream).await?;

    // Case #1: send messages and call injected (not from &self) services inside handlers
    println!("-- Case #1: send messages and call injected (not from &self) services inside handlers");
    let pong1 = actor1_ref.ask::<PongMsg>(PingMsg).await?;
    let pong2 = actor2_ref.ask::<PongMsg>(PingMsg).await?;
    println!("main: received {pong1:?} and {pong2:?} messages\n");

    // Case #2: send message#1 to actor1 and reply to actor2 without actor2 reference inside message#1
    println!("-- Case #2: send message#1 to actor1 and reply to actor2 without actor2 reference inside message#1");
    actor1_ref.send(MessageWithoutReply("login:password".to_owned()))?;

    // wait for actors to finish
    tokio::time::sleep(1.std_milliseconds()).await;
    Ok(())
}
