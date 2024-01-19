use time::ext::NumericalStdDuration;
use uactor::system::System;
use crate::actor1::Actor1;
use crate::messages::{PingMsg, PongMsg};
use crate::actor1::Actor1Msg;
use crate::actor1::Actor1Ref;

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
    use uactor::message::IntervalMessage;
    use crate::messages::{PingMsg, PongMsg};

    #[derive(Default)]
    pub struct Actor1 {
        interval_count: u8,
    }

    impl Actor for Actor1 { type Context = Context<Actor1>; }

    #[async_trait::async_trait]
    impl Handler<PingMsg> for Actor1 {
        async fn handle(&mut self, ping: PingMsg, _: &mut <Self as Actor>::Context) -> HandleResult {
            println!("actor1: Received ping message");
            let PingMsg(reply) = ping;
            let _ = reply.send(PongMsg);
            Ok(())
        }
    }

    #[async_trait::async_trait]
    impl Handler<IntervalMessage> for Actor1 {
        async fn handle(&mut self, IntervalMessage { time: _, duration }: IntervalMessage, _: &mut <Self as Actor>::Context) -> HandleResult {
            self.interval_count += 1;
            println!("actor1: received {}nd interval message", self.interval_count);
            Ok(())
        }
    }

    uactor::generate_actor_ref!(Actor1, { PingMsg });
}

#[tokio::main]
async fn main() -> anyhow::Result<()>{
    let actor1 = Actor1::default();

    let system = System::global();

    let interval = tokio::time::interval(1.std_seconds());

    let (mut actor1_ref, _) = uactor::spawn_with_ref!(system, actor1: Actor1, interval);

    let pong = actor1_ref.send_and_wait_ping_msg::<PongMsg>(|reply| PingMsg(reply)).await?;
    println!("main: received {pong:?} message");

    tokio::time::sleep(10.std_seconds()).await;
    Ok(())
}