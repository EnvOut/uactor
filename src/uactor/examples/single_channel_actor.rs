use uactor::actor::abstract_actor::MessageSender;
use uactor::system::System;

use crate::actor1::Actor1Msg;
use crate::actor1::{Actor1, Actor1MpscRef};
use crate::messages::PingMsg;

mod messages {
    use uactor::actor::message::{Message, Reply};

    pub struct PingMsg(pub Reply<PongMsg>);

    #[derive(Debug)]
    pub struct PongMsg;

    uactor::message_impl!(PingMsg, PongMsg);
}

mod actor1 {
    use uactor::actor::abstract_actor::{Actor, HandleResult, Handler};
    use uactor::actor::context::Context;

    use crate::messages::{PingMsg, PongMsg};

    pub struct Actor1;

    impl Actor for Actor1 {
        type Context = Context;
        type RouteMessage = Actor1Msg;
        type Inject = ();
        type State = ();
    }

    impl Handler<PingMsg> for Actor1 {
        async fn handle(
            &mut self,
            _: &mut Self::Inject,
            ping: PingMsg,
            _: &mut Context,
            _state: &Self::State,
        ) -> HandleResult {
            println!("actor1: Received ping message");
            let PingMsg(reply) = ping;
            let _ = reply.send(PongMsg);
            Ok(())
        }
    }

    uactor::generate_actor_ref!(Actor1, { PingMsg });
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let actor1 = Actor1;

    let mut system = System::global();

    let (actor1_ref, actor1_stream) = system.register_ref::<Actor1, Actor1Msg, Actor1MpscRef>("actor1").await;

    system.spawn_actor(actor1_ref.name(), actor1, (), actor1_stream).await?;

    let pong = actor1_ref.ask(PingMsg).await?;
    println!("main: received {pong:?} message");

    Ok(())
}
