use uactor::actor::abstract_actor::{Actor, HandleResult, Handler, MessageSender};
use uactor::actor::context::Context;
use uactor::actor::message::Message;
use uactor::system::System;

struct UnitActor {
    start_count: u32,
    stop_count: u32,
}

impl Actor for UnitActor {
    type Context = Context;
    type RouteMessage = UnitActorMsg;
    type Inject = ();
    type State = ();
}

impl Handler<StartJob> for UnitActor {
    async fn handle(
        &mut self,
        _: &mut Self::Inject,
        _msg: StartJob,
        _ctx: &mut Self::Context,
        _state: &Self::State,
    ) -> HandleResult {
        self.start_count += 1;
        Ok(())
    }
}

impl Handler<StopJob> for UnitActor {
    async fn handle(
        &mut self,
        _: &mut Self::Inject,
        _msg: StopJob,
        _ctx: &mut Self::Context,
        _state: &Self::State,
    ) -> HandleResult {
        self.stop_count += 1;
        Ok(())
    }
}

uactor::generate_actor_ref!(UnitActor, {
    StartJob: (),
    StopJob: ()
});

#[test]
fn unit_enum_variants_exist() {
    let _start = UnitActorMsg::StartJob;
    let _stop = UnitActorMsg::StopJob;
}

#[test]
fn unit_message_name() {
    assert_eq!(UnitActorMsg::static_name(), "UnitActorMsg");
    assert_eq!(StartJob::static_name(), "StartJob");
    assert_eq!(StopJob::static_name(), "StopJob");

    let msg = UnitActorMsg::StartJob;
    assert_eq!(msg.name(), "StartJob");
}

#[tokio::test]
async fn unit_actor_ref_send() {
    let mut system = System::global();

    let (actor_ref, stream) = system
        .register_ref::<UnitActor, _, UnitActorMpscRef>("unit_actor")
        .await;

    let actor = UnitActor {
        start_count: 0,
        stop_count: 0,
    };
    system
        .spawn_actor(actor_ref.name(), actor, (), stream)
        .await
        .unwrap();

    actor_ref.send(StartJob).unwrap();
    actor_ref.send(StopJob).unwrap();
    actor_ref.send(StartJob).unwrap();
}

#[test]
fn unit_mixed_with_aliased() {
    struct MixedActor;
    impl Actor for MixedActor {
        type Context = Context;
        type RouteMessage = MixedActorMsg;
        type Inject = ();
        type State = ();
    }

    #[derive(uactor::Message)]
    struct DataMsg(pub String);

    impl Handler<DataMsg> for MixedActor {
        async fn handle(
            &mut self,
            _: &mut Self::Inject,
            _msg: DataMsg,
            _ctx: &mut Self::Context,
            _state: &Self::State,
        ) -> HandleResult {
            Ok(())
        }
    }

    impl Handler<Tick> for MixedActor {
        async fn handle(
            &mut self,
            _: &mut Self::Inject,
            _msg: Tick,
            _ctx: &mut Self::Context,
            _state: &Self::State,
        ) -> HandleResult {
            Ok(())
        }
    }

    impl Handler<Refresh> for MixedActor {
        async fn handle(
            &mut self,
            _: &mut Self::Inject,
            _msg: Refresh,
            _ctx: &mut Self::Context,
            _state: &Self::State,
        ) -> HandleResult {
            Ok(())
        }
    }

    uactor::generate_actor_ref!(MixedActor, {
        DataMsg,
        Tick: (),
        Refresh: ()
    });

    let _data = MixedActorMsg::DataMsg(DataMsg("hello".into()));
    let _tick = MixedActorMsg::Tick;
    let _refresh = MixedActorMsg::Refresh;
}
