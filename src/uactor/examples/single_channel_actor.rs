use uactor::actor::{Actor, HandleResult};
use uactor::system::System;
use crate::actor1::Actor1;
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
    use uactor::actor::{Actor, Handler, HandleResult};
    use uactor::context::Context;
    use crate::messages::{PingMsg, PongMsg};

    pub struct Actor1;

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
}

#[tokio::main]
async fn main() -> anyhow::Result<()>{
    let actor1 = Actor1;

    let system = System::global();

    let (mut actor1_ref, _) = uactor::spawn_with_ref!(system, Actor1, actor1, Actor1Msg; PingMsg);

    let pong = actor1_ref.send_and_wait_ping_msg::<PongMsg>(|reply| PingMsg(reply)).await?;
    println!("main: received {pong:?} message");

    Ok(())
}