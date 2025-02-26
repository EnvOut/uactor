use time::ext::NumericalStdDuration;
use tokio::sync::mpsc;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use uactor::data::datasource_combinators::DataSourceMapExt;
use uactor::system::System;

use crate::actor1::{Actor1, Actor1Msg};
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
    use crate::messages::{PingPongMsg, ReqMsg, RespMsg};
    use tokio::sync::mpsc;
    use uactor::actor::abstract_actor::{Actor, HandleResult, Handler};
    use uactor::actor::context::Context;
    use uactor::actor::message::Message;

    pub enum Actor1Msg {
        PingPongMsg(PingPongMsg),
        ReqMsg(ReqMsg),
    }

    impl Message for Actor1Msg {
        fn static_name() -> &'static str {
            "Actor1Msg"
        }
    }

    pub struct Actor1 {
        pub resp_tx: mpsc::Sender<RespMsg>,
    }

    impl Actor for Actor1 {
        type Context = Context;
        type RouteMessage = Actor1Msg;
        type Inject = ();
        type State = ();
    }

    // Converter from RouteMessage to Handler
    // use uactor::generate_actor_ref macros to generate Actor::RouteMessage and Actor's reference
    impl Handler<Actor1Msg> for Actor1 {
        async fn handle(
            &mut self,
            inject: &mut Self::Inject,
            msg: Actor1Msg,
            ctx: &mut Self::Context,
            state: &Self::State,
        ) -> HandleResult {
            match msg {
                Actor1Msg::PingPongMsg(ping) => {
                    self.handle(inject, ping, ctx, state).await
                }
                Actor1Msg::ReqMsg(req) => {
                    self.handle(inject, req, ctx, state).await
                }
            }
        }
    }

    impl Handler<PingPongMsg> for Actor1 {
        async fn handle(
            &mut self,
            _: &mut Self::Inject,
            msg: PingPongMsg,
            _ctx: &mut Self::Context,
            _state: &Self::State,
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
            _state: &Self::State,
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
            _state: &Self::State,
        ) -> HandleResult {
            println!("actor2 handle RespMsg: {msg:?}");
            Ok(())
        }
    }

    impl Actor for Actor2 {
        type Context = Context;
        type RouteMessage = RespMsg;
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
    let mut system = System::global();

    // Initialize channels
    let (ping_tx, ping_rx) = mpsc::channel::<PingPongMsg>(10);
    let (req_tx, req_rx) = mpsc::channel::<ReqMsg>(10);
    let (resp_tx, resp_rx) = mpsc::channel::<RespMsg>(10);

    // Initialize actors
    let actor1 = Actor1 { resp_tx };
    let actor2 = Actor2;

    // convert channels to Actor::RouteMessage message
    let actor1_ch1 = ping_rx.map(Actor1Msg::PingPongMsg);
    let actor1_ch2 = req_rx.map(Actor1Msg::ReqMsg);
    // Run actors
    let (_, handle1) = system.spawn_actor("actor1".into(), actor1, (), (actor1_ch1, actor1_ch2)).await?;
    let (_, handle2) = system.spawn_actor("actor2".into(), actor2, (), resp_rx).await?;

    // Send messages
    ping_tx.send(PingPongMsg::Ping).await?;
    req_tx.send(ReqMsg::GET).await?;

    // Tokio aspects to stop spawned tasks without errors
    tokio::time::sleep(1.std_nanoseconds()).await;
    handle1.abort_handle().abort();
    handle2.abort_handle().abort();

    Ok(())
}
