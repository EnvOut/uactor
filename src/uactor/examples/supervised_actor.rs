use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use uactor::actor::abstract_actor::MessageSender;
use uactor::system::System;

use crate::actor1::{Actor1, Actor1MpscRef};
use crate::actor1::Actor1Msg;
use crate::actor1::Actor1Ref;
use crate::messages::PingMsg;
use crate::supervisor::{Supervisor, SupervisorMpscRef, SupervisorMsg, SupervisorRef};

mod messages {
    use uactor::actor::message::{Message, Reply};

    pub struct PingMsg(pub Reply<PongMsg>);

    #[derive(Debug)]
    pub struct PongMsg;

    uactor::message_impl!(PingMsg, PongMsg);
}

mod actor1 {
    use crate::messages::{PingMsg, PongMsg};
    use uactor::actor::abstract_actor::{Actor, HandleResult, Handler};
    use uactor::actor::context::{ActorContext, Context};

    pub struct Actor1;

    impl Actor for Actor1 {
        type Context = Context;
        type Inject = ();
        type State = ();
    }

    impl Handler<PingMsg> for Actor1 {
        async fn handle(
            &mut self,
            _: &mut Self::Inject,
            ping: PingMsg,
            ctx: &mut Self::Context,
            _state: &Self::State,
        ) -> HandleResult {
            println!("actor1: Received ping message");
            let PingMsg(reply) = ping;
            let _ = reply.send(PongMsg);
            ctx.kill();
            Ok(())
        }
    }

    uactor::generate_actor_ref!(Actor1, { PingMsg });
}

mod supervisor {
    use uactor::actor::abstract_actor::{Actor, HandleResult, Handler};
    use uactor::actor::context::{ActorDied, Context};

    pub struct Supervisor;

    impl Actor for Supervisor {
        type Context = Context;
        type Inject = ();
        type State = ();
    }

    impl Handler<ActorDied> for Supervisor {
        async fn handle(
            &mut self,
            _: &mut Self::Inject,
            ActorDied(name): ActorDied,
            _: &mut Context,
            state: &Self::State,
        ) -> HandleResult {
            println!("Actor with name: {name:?} - died");
            Ok(())
        }
    }

    uactor::generate_actor_ref!(Supervisor, { ActorDied });
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(LevelFilter::INFO)
        .with(tracing_subscriber::fmt::layer())
        .init();

    let mut system = System::global().build();

    let (supervisor_ref, supervisor_stream) = system.register_ref::<Supervisor, _, SupervisorMpscRef>("supervisor");
    let (actor1_ref, actor1_stream) = system.register_ref::<Actor1, _, Actor1MpscRef>("actor1");

    // Run supervisor
    let supervisor = Supervisor;
    system.spawn_actor(supervisor_ref.name(), supervisor, (), supervisor_stream).await?;

    // Run actor1
    let actor1 = Actor1;
    system.spawn_actor(actor1_ref.name(), actor1, (), actor1_stream).await?;

    // ask actor1 to send a pong message
    let pong = actor1_ref.ask(PingMsg).await?;
    println!("main: received {pong:?} message");

    tokio::time::sleep(Duration::from_secs(1)).await;

    Ok(())
}
