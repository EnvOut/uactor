use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use uactor::select::ActorSelect;
use uactor::system::System;
use crate::actor1::Actor1;
use crate::actor2::Actor2;
use crate::messages::{PingPongMsg, ReqMsg, RespMsg};

pub mod messages {
    use uactor::message::Message;

    #[derive(Debug)]
    pub enum PingPongMsg {
        Ping,
        Pong,
    }
    impl Message for PingPongMsg {}

    #[derive(Debug)]
    pub enum ReqMsg {
        GET
    }
    impl Message for ReqMsg {}

    #[derive(Debug)]
    pub enum RespMsg {
        Ok,
        Err,
    }
    impl Message for RespMsg {}
}

pub mod actor1 {
    use uactor::actor::{Actor, Handler, HandleResult};
    use uactor::context::Context;
    use crate::messages::{PingPongMsg, ReqMsg, RespMsg};

    pub struct Actor1 {
        pub resp_tx: tokio::sync::mpsc::Sender<RespMsg>,
    }

    impl Actor for Actor1 { type Context = Context<Self>; }

    #[async_trait::async_trait]
    impl Handler<PingPongMsg> for Actor1 {
        async fn handle(&mut self, msg: PingPongMsg, ctx: &mut Self::Context) -> HandleResult {
            println!("actor1 handle PingPongMsg: {msg:?}");
            Ok(())
        }
    }

    #[async_trait::async_trait]
    impl Handler<ReqMsg> for Actor1 {
        async fn handle(&mut self, msg: ReqMsg, ctx: &mut Self::Context) -> HandleResult {
            println!("actor1 handle ReqMsg: {msg:?}");
            self.resp_tx.send(RespMsg::Ok).await?;
            Ok(())
        }
    }
}

pub mod actor2 {
    use uactor::actor::{Actor, Handler, HandleResult};
    use uactor::context::Context;
    use crate::messages::RespMsg;

    pub struct Actor2;

    #[async_trait::async_trait]
    impl Handler<RespMsg> for Actor2 {
        async fn handle(&mut self, msg: RespMsg, ctx: &mut Self::Context) -> HandleResult {
            println!("actor2 handle RespMsg: {msg:?}");
            Ok(())
        }
    }

    impl Actor for Actor2 { type Context = Context<Self>; }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(LevelFilter::TRACE)
        .with(tracing_subscriber::fmt::layer())
        .init();

    let (ping_tx, mut ping_rx) = tokio::sync::mpsc::channel::<PingPongMsg>(10);
    let (req_tx, mut req_rx) = tokio::sync::mpsc::channel::<ReqMsg>(10);
    let (resp_tx, resp_rx) = tokio::sync::mpsc::channel::<RespMsg>(10);

    let actor1 = Actor1 { resp_tx };
    let actor2  = Actor2;

    let system = System::global();

    let handle1 = system.run(actor1, (ping_rx, req_rx)).await;
    let handle2 = system.run(actor2, resp_rx).await;

    ping_tx.send(PingPongMsg::Ping).await.unwrap();
    req_tx.send(ReqMsg::GET).await.unwrap();

    tokio::join!(handle1, handle2);
}
