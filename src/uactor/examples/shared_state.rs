use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use time::ext::NumericalStdDuration;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use uactor::actor::abstract_actor::{Actor, HandleResult, Handler, MessageSender};
use uactor::actor::context::Context;
use uactor::system::System;

use uactor::actor::message::Message;
pub struct PingMsg;

uactor::message_impl!(PingMsg);

pub struct Actor1;

#[derive(Default)]
pub struct Actor1State {
    counter: AtomicU8,
}

impl Actor1State {
    pub fn get_counter(&self) -> u8 {
        self.counter.load(Ordering::Relaxed)
    }
}

impl Actor for Actor1 {
    type Context = Context;
    type RouteMessage = Actor1Msg;
    type Inject = ();
    type State = Arc<Actor1State>;
}

impl Handler<PingMsg> for Actor1 {
    async fn handle(
        &mut self,
        _: &mut Self::Inject,
        _ping: PingMsg,
        _ctx: &mut Self::Context,
        state: &Self::State,
    ) -> HandleResult {
        state.counter.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}

uactor::generate_actor_ref!(Actor1, { PingMsg });

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(LevelFilter::INFO)
        .with(tracing_subscriber::fmt::layer())
        .init();

    let mut system = System::global();

    let (actor1_ref, actor1_stream) = system.register_ref::<Actor1, _, Actor1MpscRef>("actor1").await;

    let actor1 = Actor1;
    let (_, handle) = system.spawn_actor(actor1_ref.name(), actor1, actor1_ref.state().clone(), actor1_stream).await?;

    actor1_ref.send(PingMsg)?;
    actor1_ref.send(PingMsg)?;
    actor1_ref.send(PingMsg)?;

    tokio::time::sleep(1.std_nanoseconds()).await;

    assert_eq!(actor1_ref.state.get_counter(), 3, "Counter should be 3");

    // stop the actor
    handle.abort_handle().abort();
    Ok(())
}
