use time::ext::NumericalStdDuration;
use tokio::sync::mpsc::UnboundedSender;
use uactor::actor::abstract_actor::MessageSender;

use uactor::system::System;

use crate::actor1::{Actor1, Actor1MpscRef};
use crate::actor1::Actor1Msg;
use crate::actor1::Actor1Ref;
use crate::messages::{PingMsg, PongMsg};

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
    use uactor::actor::message::IntervalMessage;

    use crate::messages::{PingMsg, PongMsg};

    #[derive(Default)]
    pub struct Actor1 {
        interval_count: u8,
    }

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
            _ctx: &mut Context,
            state: &Self::State,
        ) -> HandleResult {
            println!("actor1: Received ping message");
            let PingMsg(reply) = ping;
            let _ = reply.send(PongMsg);
            Ok(())
        }
    }

    impl Handler<IntervalMessage> for Actor1 {
        async fn handle(
            &mut self,
            _: &mut Self::Inject,
            IntervalMessage {
                time: _,
                duration: _,
            }: IntervalMessage,
            _ctx: &mut Context,
            state: &Self::State,
        ) -> HandleResult {
            self.interval_count += 1;
            println!(
                "actor1: received {}nd interval message",
                self.interval_count
            );
            Ok(())
        }
    }

    uactor::generate_actor_ref!(Actor1, { PingMsg });
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut system = System::global().build();

    // 1 second interval
    let interval = tokio::time::interval(1.std_seconds());

    let (actor1_ref, actor1_stream) = system.register_ref::<Actor1, _, Actor1MpscRef>("actor1");

    let actor1 = Actor1::default();
    system.spawn_actor(actor1_ref.name(), actor1, *actor1_ref.state(), (actor1_stream, interval)).await?;

    let pong = actor1_ref.ask::<PongMsg>(PingMsg).await?;
    println!("main: received {pong:?} message");

    // waiting 10 seconds and expecting new message each 1 second
    tokio::time::sleep(10.std_seconds()).await;
    Ok(())
}
