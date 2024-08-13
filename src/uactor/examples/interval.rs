use time::ext::NumericalStdDuration;
use uactor::actor::MessageSender;

use uactor::system::System;

use crate::actor1::Actor1;
use crate::actor1::Actor1Msg;
use crate::actor1::Actor1Ref;
use crate::messages::{PingMsg, PongMsg};

mod messages {
    use tokio::sync::oneshot::Sender;

    use uactor::message::Message;

    pub struct PingMsg(pub Sender<PongMsg>);

    #[derive(Debug)]
    pub struct PongMsg;

    uactor::message_impl!(PingMsg, PongMsg);
}

mod actor1 {
    use uactor::actor::{Actor, EmptyState, HandleResult, Handler};
    use uactor::context::Context;
    use uactor::message::IntervalMessage;

    use crate::messages::{PingMsg, PongMsg};

    #[derive(Default)]
    pub struct Actor1 {
        interval_count: u8,
    }

    impl Actor for Actor1 {
        type Context = Context;
        type Inject = ();
    }

    impl Handler<PingMsg> for Actor1 {
        async fn handle(
            &mut self,
            _: &mut Self::Inject,
            ping: PingMsg,
            _ctx: &mut Context,
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
        ) -> HandleResult {
            self.interval_count += 1;
            println!(
                "actor1: received {}nd interval message",
                self.interval_count
            );
            Ok(())
        }
    }

    uactor::generate_actor_ref!(Actor1, { PingMsg }, EmptyState);
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let actor1 = Actor1::default();

    let mut system = System::global().build();

    // 1 second interval
    let interval = tokio::time::interval(1.std_seconds());

    let (actor1_ref, _) = uactor::spawn_with_ref!(system, actor1: Actor1, interval);

    system.run_actor::<Actor1>(actor1_ref.name()).await?;

    let pong = actor1_ref.ask::<PongMsg>(PingMsg).await?;
    println!("main: received {pong:?} message");

    // waiting 10 seconds and expecting new message each 1 second
    tokio::time::sleep(10.std_seconds()).await;
    Ok(())
}
