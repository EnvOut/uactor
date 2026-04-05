use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use uactor::actor::abstract_actor::{Actor, MessageSender};
use uactor::actor::context::{ActorContext, Context};
use uactor::actor::message::Reply;
use uactor::system::System;

#[derive(uactor::Message)]
struct Increment;

#[derive(uactor::Message, Debug)]
struct GetCount(Reply<CountResponse>);

#[derive(uactor::Message, Debug)]
struct CountResponse(u32);

#[derive(uactor::Message)]
struct SetValue(u32);

struct CounterActor {
    count: u32,
}

impl Actor for CounterActor {
    type Context = Context;
    type RouteMessage = CounterActorMsg;
    type Inject = ();
    type State = ();
}

#[uactor::actor]
impl CounterActor {
    #[uactor::handler]
    async fn handle_increment(&mut self, _msg: Increment) -> HandleResult {
        self.count += 1;
        Ok(())
    }

    #[uactor::handler]
    async fn handle_get_count(&self, GetCount(reply): GetCount) -> HandleResult {
        let _ = reply.send(CountResponse(self.count));
        Ok(())
    }

    #[uactor::handler]
    async fn handle_set_value(&mut self, SetValue(val): SetValue) -> HandleResult {
        self.count = val;
        Ok(())
    }
}

uactor::generate_actor_ref!(CounterActor, { Increment, GetCount, SetValue });

#[tokio::test]
async fn handler_macro_with_mut_self() {
    let mut system = System::global();

    let (actor_ref, stream) =
        system.register_ref::<CounterActor, _, CounterActorMpscRef>("counter").await;

    let actor = CounterActor { count: 0 };
    system
        .spawn_actor(actor_ref.name(), actor, (), stream)
        .await
        .unwrap();

    actor_ref.send(Increment).unwrap();
    actor_ref.send(Increment).unwrap();
    actor_ref.send(Increment).unwrap();

    let CountResponse(count) = actor_ref.ask::<CountResponse>(GetCount).await.unwrap();
    assert_eq!(count, 3);
}

#[tokio::test]
async fn handler_macro_set_and_get() {
    let mut system = System::global();

    let (actor_ref, stream) =
        system.register_ref::<CounterActor, _, CounterActorMpscRef>("counter_set").await;

    let actor = CounterActor { count: 0 };
    system
        .spawn_actor(actor_ref.name(), actor, (), stream)
        .await
        .unwrap();

    actor_ref.send(SetValue(42)).unwrap();

    let CountResponse(count) = actor_ref.ask::<CountResponse>(GetCount).await.unwrap();
    assert_eq!(count, 42);
}

// --- Test handler with ctx parameter ---

#[derive(uactor::Message)]
struct StopMsg;

struct StoppableActor;

impl Actor for StoppableActor {
    type Context = Context;
    type RouteMessage = StoppableActorMsg;
    type Inject = ();
    type State = Arc<AtomicU32>;
}

#[uactor::actor]
impl StoppableActor {
    #[uactor::handler]
    async fn handle_stop(_msg: StopMsg, ctx: &mut Context, state: &Arc<AtomicU32>) -> HandleResult {
        state.fetch_add(1, Ordering::Relaxed);
        ctx.kill();
        Ok(())
    }
}

uactor::generate_actor_ref!(StoppableActor, { StopMsg });

#[tokio::test]
async fn handler_macro_with_ctx_and_state() {
    let mut system = System::global();

    let (actor_ref, stream) =
        system.register_ref::<StoppableActor, _, StoppableActorMpscRef>("stoppable").await;

    let state = Arc::new(AtomicU32::new(0));
    let actor = StoppableActor;
    let (_, handle) = system
        .spawn_actor(actor_ref.name(), actor, state.clone(), stream)
        .await
        .unwrap();

    actor_ref.send(StopMsg).unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    assert!(handle.is_finished());
    assert_eq!(state.load(Ordering::Relaxed), 1);
}
