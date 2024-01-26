use time::ext::NumericalStdDuration;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use uactor::system::System;

use crate::actor1::Actor1;
use crate::actor1::Actor1Msg;
use crate::actor1::Actor1Ref;
use crate::actor2::Actor2;
use crate::actor2::Actor2Msg;
use crate::actor2::Actor2Ref;
use crate::messages::{MessageWithoutReply, PingMsg, PongMsg};
use crate::services::{Service1, Service2};

mod messages {
    use tokio::sync::oneshot::Sender;

    use uactor::message::Message;

    pub struct PingMsg(pub Sender<PongMsg>);

    #[derive(derive_more::Into, Debug)]
    pub struct MessageWithoutReply(pub String);

    #[derive(derive_more::Constructor, Debug)]
    pub struct PrintMessage(String);

    #[derive(Debug)]
    pub struct PongMsg;

    uactor::message_impl!(PingMsg, PongMsg, MessageWithoutReply, PrintMessage);
}

mod actor1 {
    use tokio::sync::mpsc::UnboundedSender;

    use uactor::actor::{Actor, Handler, HandleResult};
    use uactor::context::Context;
    use uactor::context::extensions::Service;
    use uactor::di::{Inject, InjectError};
    use uactor::system::System;

    use crate::actor2::{Actor2Msg, Actor2Ref};
    use crate::messages::{MessageWithoutReply, PingMsg, PongMsg, PrintMessage};
    use crate::services::Service1;

    pub struct Actor1;

    #[derive(derive_more::Constructor)]
    pub struct Services {
        service1: Service<Service1>,
        actor2_ref: Actor2Ref<UnboundedSender<Actor2Msg>>,
    }

    impl Inject for Services {
        async fn inject(system: &System) -> Result<Self, InjectError>
            where
                Self: Sized,
        {
            let service1 = system.get_service()?;
            let actor2_ref = system.get_actor::<Actor2Ref<UnboundedSender<Actor2Msg>>>("actor2".into())?;
            Ok(Services::new(service1, actor2_ref))
        }
    }

    impl Actor for Actor1 {
        type Context = Context;
        type Inject = Services;
    }

    impl Handler<PingMsg> for Actor1 {
        async fn handle(&mut self, Services { service1, .. }: &mut Self::Inject, ping: PingMsg, ctx: &mut Context) -> HandleResult {
            println!("actor1: Received ping message");

            service1.do_something();

            let PingMsg(reply) = ping;
            let _ = reply.send(PongMsg);
            Ok(())
        }
    }

    impl Handler<MessageWithoutReply> for Actor1 {
        async fn handle(&mut self, Services { actor2_ref, .. }: &mut Self::Inject, msg: MessageWithoutReply, ctx: &mut Context) -> HandleResult {
            println!("actor1: Received {msg:?} message, sending PrintMessage to the actor2");
            actor2_ref.send_print_message(PrintMessage::new(msg.into()))?;
            Ok(())
        }
    }

    uactor::generate_actor_ref!(Actor1, { PingMsg, MessageWithoutReply });
}

mod actor2 {
    use uactor::actor::{Actor, Handler, HandleResult};
    use uactor::context::Context;
    use uactor::context::extensions::Service;
    use uactor::di::{Inject, InjectError};
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
        type Inject = Services;
    }

    impl Handler<PingMsg> for Actor2 {
        async fn handle(&mut self, Services(service2): &mut Self::Inject, ping: PingMsg, _: &mut Context) -> HandleResult {
            println!("actor2: Received ping message");

            service2.do_something();

            let PingMsg(reply) = ping;
            let _ = reply.send(PongMsg);
            Ok(())
        }
    }

    impl Handler<PrintMessage> for Actor2 {
        async fn handle(&mut self, _: &mut Self::Inject, msg: PrintMessage, _: &mut Context) -> HandleResult {
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
        .extension(service1)
        .extension(service2)
        .build();

    // Init actor (instance + spawn actor)
    let actor1 = Actor1;
    let (actor1_ref, _) = uactor::spawn_with_ref!(system, actor1: Actor1);

    // Init actor2 (instance + spawn actor)
    let actor2 = Actor2;
    let (actor2_ref, _) = uactor::spawn_with_ref!(system, actor2: Actor2);

    // Run actors
    system.run_actor::<Actor1>(actor1_ref.name()).await?;
    system.run_actor::<Actor2>(actor2_ref.name()).await?;

    // Case #1: send messages and call injected (not from &self) services inside handlers
    println!("-- Case #1: send messages and call injected (not from &self) services inside handlers");
    let pong1 = actor1_ref
        .ask_ping_msg::<PongMsg>(|reply| PingMsg(reply))
        .await?;
    let pong2 = actor2_ref
        .ask_ping_msg::<PongMsg>(|reply| PingMsg(reply))
        .await?;
    println!("main: received {pong1:?} and {pong2:?} messages");

    // Case #2: send message#1 to actor1 and reply to actor2 without actor2 reference inside message#1
    println!("\n-- Case #2: send message#1 to actor1 and reply to actor2 without actor2 reference inside message#1");
    let pong1 = actor1_ref
        .send_message_without_reply(MessageWithoutReply("login:password".to_owned()))?;

    tokio::time::sleep(1.std_milliseconds()).await;
    Ok(())
}
