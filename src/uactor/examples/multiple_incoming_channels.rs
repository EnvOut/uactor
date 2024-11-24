use std::time::Duration;

use tracing::level_filters::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use uactor::system::System;

use crate::actor1::Actor1;
use crate::actor2::Actor2;
use crate::messages::{PingPongMsg, ReqMsg, RespMsg};

pub mod messages {
    use uactor::actor::message::Message;
    use uactor::message_impl;

    #[derive(Debug)]
    pub enum PingPongMsg {
        Ping,
        Pong,
    }

    #[derive(Debug)]
    pub enum ReqMsg {
        GET,
    }

    #[derive(Debug)]
    pub enum RespMsg {
        Ok,
        Err,
    }

    message_impl!(PingPongMsg, ReqMsg, RespMsg);
}

pub mod actor1 {
    use uactor::actor::abstract_actor::{Actor, HandleResult, Handler};
    use uactor::actor::context::Context;

    use crate::messages::{PingPongMsg, ReqMsg, RespMsg};

    pub struct Actor1 {
        pub resp_tx: tokio::sync::mpsc::Sender<RespMsg>,
    }

    impl Actor for Actor1 {
        type Context = Context;
        type Inject = ();
        type State = ();
    }

    impl Handler<PingPongMsg> for Actor1 {
        async fn handle(
            &mut self,
            _: &mut Self::Inject,
            msg: PingPongMsg,
            _ctx: &mut Self::Context,
            state: &Self::State,
        ) -> HandleResult {
            println!("actor1 handle PingPongMsg: {msg:?}");
            Ok(())
        }
    }

    impl Handler<ReqMsg> for Actor1 {
        async fn handle(
            &mut self,
            _: &mut Self::Inject,
            msg: ReqMsg,
            _ctx: &mut Self::Context,
            state: &Self::State,
        ) -> HandleResult {
            println!("actor1 handle ReqMsg: {msg:?}");
            self.resp_tx.send(RespMsg::Ok).await?;
            Ok(())
        }
    }
}

pub mod actor2 {
    use uactor::actor::abstract_actor::{Actor, HandleResult, Handler};
    use uactor::actor::context::Context;

    use crate::messages::RespMsg;

    pub struct Actor2;

    impl Handler<RespMsg> for Actor2 {
        async fn handle(
            &mut self,
            _: &mut Self::Inject,
            msg: RespMsg,
            _: &mut Self::Context,
            state: &Self::State,
        ) -> HandleResult {
            println!("actor2 handle RespMsg: {msg:?}");
            Ok(())
        }
    }

    impl Actor for Actor2 {
        type Context = Context;
        type Inject = ();
        type State = ();
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(LevelFilter::TRACE)
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize system
    let mut system = System::global().build();

    // Initialize channels
    let (ping_tx, ping_rx) = tokio::sync::mpsc::channel::<PingPongMsg>(10);
    let (req_tx, req_rx) = tokio::sync::mpsc::channel::<ReqMsg>(10);
    let (resp_tx, resp_rx) = tokio::sync::mpsc::channel::<RespMsg>(10);

    // Initialize actors
    let actor1 = Actor1 { resp_tx };
    let actor2 = Actor2;

    // Run actors
    let (_, handle1) = system.spawn_actor("actor1".into(), actor1, (), (ping_rx, req_rx)).await?;
    let (_, handle2) = system.spawn_actor("actor2".into(), actor2, (), (resp_rx)).await?;

    // Send messages
    ping_tx.send(PingPongMsg::Ping).await?;
    req_tx.send(ReqMsg::GET).await?;

    // Tokio aspects to stop spawned tasks without errors
    tokio::time::sleep(Duration::from_nanos(1)).await;
    handle1.abort_handle().abort();
    handle2.abort_handle().abort();

    Ok(())
}
