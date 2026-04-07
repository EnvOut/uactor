//! Demonstrates the macro-based handler API:
//! - `#[derive(Message)]` instead of `message_impl!`
//! - `#[uactor::actor]` + `#[uactor::handler]` instead of manual `impl Handler<M>`
//! - `generate_actor_ref!` with aliased variants (`NextId: i32`)

use uactor::actor::abstract_actor::MessageSender;
use uactor::system::System;

use crate::counter::{CounterActor, CounterActorMpscRef, RawId};
use crate::messages::{GetCount, Increment, SetValue};

mod messages {
    use uactor::actor::message::Reply;

    #[derive(uactor::Message)]
    pub struct Increment;

    #[derive(uactor::Message)]
    pub struct SetValue(pub u32);

    #[derive(uactor::Message, Debug)]
    pub struct GetCount(pub Reply<CountResponse>);

    #[derive(uactor::Message, Debug)]
    pub struct CountResponse(pub u32);
}

mod counter {
    use uactor::actor::abstract_actor::Actor;
    use uactor::actor::context::{ActorContext, Context};

    use crate::messages::{CountResponse, GetCount, Increment, SetValue};

    pub struct CounterActor {
        count: u32,
    }

    impl CounterActor {
        pub fn new() -> Self {
            Self { count: 0 }
        }
    }

    impl Actor for CounterActor {
        type Context = Context;
        type RouteMessage = CounterActorMsg;
        type Inject = ();
        type State = ();
    }

    #[uactor::actor]
    impl CounterActor {
        /// Increments the internal counter by 1.
        #[uactor::handler]
        async fn handle_increment(&mut self, _msg: Increment) -> HandleResult {
            self.count += 1;
            println!("[counter] incremented to {}", self.count);
            Ok(())
        }

        /// Replaces the counter value.
        #[uactor::handler]
        async fn handle_set(&mut self, SetValue(val): SetValue) -> HandleResult {
            println!("[counter] set value {} -> {}", self.count, val);
            self.count = val;
            Ok(())
        }

        /// Responds with the current count, then stops the actor.
        #[uactor::handler]
        async fn handle_get(&self, GetCount(reply): GetCount, ctx: &mut Context) -> HandleResult {
            println!("[counter] reporting count = {}", self.count);
            let _ = reply.send(CountResponse(self.count));
            ctx.kill();
            Ok(())
        }

        /// Handles a RawId newtype (wraps i32), created by aliased variant `RawId: i32`.
        #[uactor::handler]
        async fn handle_raw_id(&mut self, RawId(id): RawId) -> HandleResult {
            println!("[counter] received raw id = {id}, adding to count");
            self.count += id as u32;
            Ok(())
        }
    }

    // Aliased variant: `RawId: i32` generates `pub struct RawId(pub i32)` newtype
    uactor::generate_actor_ref!(CounterActor, {
        Increment,
        SetValue,
        GetCount,
        RawId: i32
    });
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use crate::messages::CountResponse;

    let mut system = System::global();

    let (actor_ref, stream) = system
        .register_ref::<CounterActor, _, CounterActorMpscRef>("counter")
        .await;

    let actor = CounterActor::new();
    let (_, handle) = system
        .spawn_actor(actor_ref.name(), actor, (), stream)
        .await?;

    // Increment 3 times
    actor_ref.send(Increment)?;
    actor_ref.send(Increment)?;
    actor_ref.send(Increment)?;

    // Send via the aliased `RawId: i32` — uses the generated newtype
    actor_ref.send(RawId(10))?;

    // Override with SetValue
    actor_ref.send(SetValue(100))?;

    // Increment once more
    actor_ref.send(Increment)?;

    // Ask for the count (also stops the actor)
    let CountResponse(count) = actor_ref.ask::<CountResponse>(GetCount).await?;
    // 3 increments + 10 (raw id) + set(100) + 1 increment = 101
    println!("[main] final count = {count}");
    assert_eq!(count, 101);

    // Actor should have stopped
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    assert!(handle.is_finished(), "actor should be stopped");

    println!("[main] done");
    Ok(())
}
