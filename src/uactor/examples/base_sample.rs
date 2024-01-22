use std::time::Duration;
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
    use uactor::message_impl;

    #[derive(Debug)]
    pub enum PingPongMsg {
        Ping,
        Pong,
    }

    #[derive(Debug)]
    pub enum ReqMsg {
        GET
    }

    #[derive(Debug)]
    pub enum RespMsg {
        Ok,
        Err,
    }

    message_impl!(PingPongMsg, ReqMsg, RespMsg);
}

pub mod actor1 {
    use uactor::actor::{Actor, Handler, HandleResult};
    use uactor::context::Context;
    use crate::messages::{PingPongMsg, ReqMsg, RespMsg};

    pub struct Actor1 {
        pub resp_tx: tokio::sync::mpsc::Sender<RespMsg>,
    }

    impl Actor for Actor1 { type Context = Context; }


    impl Handler<PingPongMsg> for Actor1 {
        async fn handle(&mut self, msg: PingPongMsg, ctx: &mut Self::Context) -> HandleResult {
            println!("actor1 handle PingPongMsg: {msg:?}");
            Ok(())
        }
    }


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


    impl Handler<RespMsg> for Actor2 {
        async fn handle(&mut self, msg: RespMsg, ctx: &mut Self::Context) -> HandleResult {
            println!("actor2 handle RespMsg: {msg:?}");
            Ok(())
        }
    }

    impl Actor for Actor2 { type Context = Context; }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(LevelFilter::TRACE)
        .with(tracing_subscriber::fmt::layer())
        .init();

    let (ping_tx, ping_rx) = tokio::sync::mpsc::channel::<PingPongMsg>(10);
    let (req_tx, req_rx) = tokio::sync::mpsc::channel::<ReqMsg>(10);
    let (resp_tx, resp_rx) = tokio::sync::mpsc::channel::<RespMsg>(10);

    let actor1 = Actor1 { resp_tx };
    let actor2  = Actor2;

    let system = System::global()
        .build();

    let handle1 = system.run(actor1, None, (ping_rx, req_rx)).await;
    let handle2 = system.run(actor2, None, resp_rx).await;

    ping_tx.send(PingPongMsg::Ping).await.unwrap();
    req_tx.send(ReqMsg::GET).await.unwrap();

    // Tokio aspects to stop spawned tasks without errors
    tokio::time::sleep(Duration::from_nanos(1)).await;
    handle1.abort_handle().abort();
    handle2.abort_handle().abort();
}
