use time::ext::NumericalStdDuration;
use tokio_stream::StreamExt;
use uactor::actor::abstract_actor::MessageSender;

use uactor::system::System;

use crate::actor1::Actor1Msg;
use crate::actor1::{Actor1, Actor1MpscRef};
use crate::messages::{AskTicksCountMsg, TicksCount, UpdateMetrics};
use more_asserts as ma;
use uactor::actor::message::IntervalMessage;
use uactor::data::datasource_decorator::DataSourceMapExt;

mod messages {
    use uactor::actor::message::{Message, Reply};

    pub struct AskTicksCountMsg(pub Reply<TicksCount>);
    pub struct UpdateMetrics;

    #[derive(Debug)]
    pub struct TicksCount(pub usize);

    uactor::message_impl!(AskTicksCountMsg, TicksCount, UpdateMetrics);
}

mod actor1 {
    use uactor::actor::abstract_actor::{Actor, HandleResult, Handler};
    use uactor::actor::context::{ActorContext, Context};
    use uactor::actor::message::IntervalMessage;

    use crate::messages::{AskTicksCountMsg, TicksCount, UpdateMetrics};

    #[derive(Default)]
    pub struct Actor1 {
        interval_count: u8,
    }

    impl Actor for Actor1 {
        type Context = Context;
        type RouteMessage = Actor1Msg;
        type Inject = ();
        type State = ();
    }

    impl Handler<AskTicksCountMsg> for Actor1 {
        async fn handle(
            &mut self,
            _: &mut Self::Inject,
            AskTicksCountMsg(reply): AskTicksCountMsg,
            ctx: &mut Context,
            _state: &Self::State,
        ) -> HandleResult {
            println!("actor1: Received ping message");
            let _ = reply.send(TicksCount(self.interval_count as usize));
            ctx.kill();
            Ok(())
        }
    }

    impl Handler<UpdateMetrics> for Actor1 {
        async fn handle(
            &mut self,
            _: &mut Self::Inject,
            _: UpdateMetrics,
            _ctx: &mut Context,
            _state: &Self::State,
        ) -> HandleResult {
            self.interval_count += 1;
            println!(
                "actor1: received {}nd UpdateMetrics message",
                self.interval_count
            );
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
            _state: &Self::State,
        ) -> HandleResult {
            // some important logic
            Ok(())
        }
    }

    uactor::generate_actor_ref!(Actor1, { AskTicksCountMsg, UpdateMetrics, IntervalMessage });
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use uactor::data::datasource_decorator::DataSourceMapExt;
    let mut system = System::global();

    // 100 milliseconds interval for updating metrics
    let metrics_update_interval = tokio::time::interval(100.std_milliseconds())
        .map(|_: IntervalMessage| Actor1Msg::UpdateMetrics(UpdateMetrics));
    // 1 second interval for other tasks
    let other_interval = tokio::time::interval(1.std_seconds())
        .map(Actor1Msg::IntervalMessage);

    // Initialize actor's reference
    let (actor1_ref, actor1_stream) = system.register_ref::<Actor1, _, Actor1MpscRef>("actor1").await;

    // Spawn actor
    let actor1 = Actor1::default();
    let (_state, actor_handle) = system.spawn_actor(actor1_ref.name(), actor1, *actor1_ref.state(), (actor1_stream, metrics_update_interval, other_interval)).await?;

    // Wait for 5 ticks
    tokio::time::sleep(500.std_milliseconds()).await;
    let TicksCount(ticks_count) = actor1_ref.ask::<TicksCount>(AskTicksCountMsg).await?;
    ma::assert_ge!(ticks_count, 5, "waiting 5 ticks and expecting at least 5 messages received");

    tokio::time::sleep(1.std_microseconds()).await;
    assert!(actor_handle.is_finished(), "actor should be finished after receiving AskTicksCountMsg");
    Ok(())
}
