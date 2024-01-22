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
    use crate::messages::{PingMsg, PongMsg};

    pub struct Actor1;

    impl Actor for Actor1 { type Context = Context; }


    impl Handler<PingMsg> for Actor1 {
        async fn handle(&mut self, ping: PingMsg, _: &mut Context) -> HandleResult {
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

    let system = System::global().build();

    let (actor1_ref, _) = uactor::spawn_with_ref!(system, actor1: Actor1);

    let pong = actor1_ref.ask_ping_msg::<PongMsg>(|reply| PingMsg(reply)).await?;
    println!("main: received {pong:?} message");

    Ok(())
}