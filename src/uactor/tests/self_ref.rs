use std::sync::atomic::{AtomicU32, Ordering};
use uactor::actor::abstract_actor::{Actor, MessageSender};
use uactor::actor::context::Context;
use uactor::system::System;

static PROCESSED_COUNT: AtomicU32 = AtomicU32::new(0);

#[derive(uactor::Message)]
struct TriggerMsg {
    count: u32,
}

#[derive(uactor::Message)]
struct WorkItem {
    index: u32,
}

struct SelfRefActor;

impl Actor for SelfRefActor {
    type Context = Context;
    type RouteMessage = SelfRefActorMsg;
    type Inject = ();
    type State = ();
}

#[uactor::actor]
impl SelfRefActor {
    /// Receives TriggerMsg and spawns `count` WorkItem messages to self
    #[uactor::handler]
    async fn handle_trigger(&mut self, msg: TriggerMsg, self_ref: SelfRefActorMpscRef) -> HandleResult {
        for i in 0..msg.count {
            self_ref.send(WorkItem { index: i })?;
        }
        Ok(())
    }

    #[handler]
    async fn handle_work(&mut self, msg: WorkItem) -> HandleResult {
        PROCESSED_COUNT.fetch_add(1, Ordering::SeqCst);
        let _ = msg.index;
        Ok(())
    }
}

uactor::generate_actor_ref!(SelfRefActor, { TriggerMsg, WorkItem });

#[tokio::test]
async fn actor_sends_messages_to_itself() {
    PROCESSED_COUNT.store(0, Ordering::SeqCst);

    let mut system = System::global();

    let (actor_ref, stream) =
        system.register_ref::<SelfRefActor, _, SelfRefActorMpscRef>("self_ref_actor").await;

    let actor = SelfRefActor;
    system
        .spawn_actor(actor_ref.name(), actor, (), stream)
        .await
        .unwrap();

    // Send a trigger that will spawn 5 WorkItem messages to self
    actor_ref.send(TriggerMsg { count: 5 }).unwrap();

    // Give the actor time to process all messages
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    assert_eq!(PROCESSED_COUNT.load(Ordering::SeqCst), 5);
}
