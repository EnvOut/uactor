use std::ops::Shl;
use std::time::Duration;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use uactor::actor::MessageSender;
use uactor::system::System;

use crate::actor1::Actor1;
use crate::actor1::Actor1Msg;
use crate::actor1::Actor1Ref;
use crate::messages::PingMsg;
use crate::supervisor::{Supervisor, SupervisorMsg, SupervisorRef};

mod messages {
    use tokio::sync::oneshot::Sender;

    use uactor::message::Message;

    pub struct PingMsg(pub Sender<PongMsg>);

    #[derive(Debug)]
    pub struct PongMsg;

    uactor::message_impl!(PingMsg, PongMsg);
}

mod actor1 {
    use tokio::sync::mpsc;
    use uactor::actor::{Actor, EmptyState, Handler, HandleResult, MessageSender};
    use uactor::context::ActorContext;
    use uactor::context::supervised::SupervisedContext;
    use crate::messages::{PingMsg, PongMsg};
    use crate::supervisor::{SupervisorMsg, SupervisorRef};

    pub struct Actor1;

    impl Actor for Actor1 {
        type Context = SupervisedContext<SupervisorRef<mpsc::UnboundedSender<SupervisorMsg>>>;
        type Inject = ();
    }

    impl Handler<PingMsg> for Actor1 {
        async fn handle(&mut self, _: &mut Self::Inject, ping: PingMsg, ctx: &mut Self::Context) -> HandleResult {
            println!("actor1: Received ping message");
            let PingMsg(reply) = ping;
            let _ = reply.send(PongMsg);
            ctx.kill();
            Ok(())
        }
    }

    uactor::generate_actor_ref!(Actor1, { PingMsg }, EmptyState);
}

mod supervisor {
    use uactor::actor::{Actor, EmptyState, Handler, HandleResult, MessageSender};
    use uactor::context::{ActorDied, Context};
    use uactor::data_publisher::{DataPublisher, DataPublisherResult, TryClone};

    pub struct Supervisor;

    impl Actor for Supervisor {
        type Context = Context;
        type Inject = ();
    }

    impl Handler<ActorDied> for Supervisor {
        async fn handle(&mut self, _: &mut Self::Inject, ActorDied(name): ActorDied, _: &mut Context) -> HandleResult {
            println!("Actor with name: {name:?} - died");
            Ok(())
        }
    }

    uactor::generate_actor_ref!(Supervisor, { ActorDied }, EmptyState);
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(LevelFilter::INFO)
        .with(tracing_subscriber::fmt::layer())
        .init();

    let mut system = System::global().build();

    let actor1 = Actor1;
    let supervisor = Supervisor;

    let (actor1_ref, _) = uactor::spawn_with_ref!(system, actor1: Actor1);
    let (supervisor_ref, _) = uactor::spawn_with_ref!(system, supervisor: Supervisor);

    system.run_actor::<Supervisor>(supervisor_ref.name()).await?;
    system.run_actor::<Actor1>(actor1_ref.name()).await?;

    let pong = actor1_ref.ask(|reply| PingMsg(reply))
        .await?;
    println!("main: received {pong:?} message");

    tokio::time::sleep(Duration::from_secs(1)).await;

    Ok(())
}
