use uactor::actor::abstract_actor::{Actor, HandleResult, Handler, MessageSender};
use uactor::actor::context::Context;
use uactor::actor::message::{Message, Reply};
use uactor::system::System;

#[derive(uactor::Message, Debug)]
struct PingMsg(Reply<PongMsg>);

#[derive(uactor::Message, Debug)]
struct PongMsg;

struct AliasActor {
    last_next_id: Option<i32>,
    last_label: Option<String>,
    completed_count: u32,
    failed_count: u32,
}

impl Actor for AliasActor {
    type Context = Context;
    type RouteMessage = AliasActorMsg;
    type Inject = ();
    type State = ();
}

impl Handler<PingMsg> for AliasActor {
    async fn handle(
        &mut self,
        _: &mut Self::Inject,
        PingMsg(reply): PingMsg,
        _ctx: &mut Self::Context,
        _state: &Self::State,
    ) -> HandleResult {
        let _ = reply.send(PongMsg);
        Ok(())
    }
}

// Handler for the NextId newtype (wraps i32)
impl Handler<NextId> for AliasActor {
    async fn handle(
        &mut self,
        _: &mut Self::Inject,
        NextId(id): NextId,
        _ctx: &mut Self::Context,
        _state: &Self::State,
    ) -> HandleResult {
        self.last_next_id = Some(id);
        Ok(())
    }
}

// Handler for the Label newtype (wraps String)
impl Handler<Label> for AliasActor {
    async fn handle(
        &mut self,
        _: &mut Self::Inject,
        Label(label): Label,
        _ctx: &mut Self::Context,
        _state: &Self::State,
    ) -> HandleResult {
        self.last_label = Some(label);
        Ok(())
    }
}

// Two aliases with the SAME inner type — each gets its own newtype and handler
impl Handler<TaskCompleted> for AliasActor {
    async fn handle(
        &mut self,
        _: &mut Self::Inject,
        _msg: TaskCompleted,
        _ctx: &mut Self::Context,
        _state: &Self::State,
    ) -> HandleResult {
        self.completed_count += 1;
        Ok(())
    }
}

impl Handler<TaskFailed> for AliasActor {
    async fn handle(
        &mut self,
        _: &mut Self::Inject,
        _msg: TaskFailed,
        _ctx: &mut Self::Context,
        _state: &Self::State,
    ) -> HandleResult {
        self.failed_count += 1;
        Ok(())
    }
}

uactor::generate_actor_ref!(AliasActor, {
    PingMsg,
    NextId: i32,
    Label: String,
    TaskCompleted: u64,
    TaskFailed: u64
});

#[test]
fn aliased_enum_variants_exist() {
    let _ping = AliasActorMsg::PingMsg(PingMsg(tokio::sync::oneshot::channel().0));
    let _id = AliasActorMsg::NextId(NextId(42));
    let _label = AliasActorMsg::Label(Label("test".to_string()));
    let _done = AliasActorMsg::TaskCompleted(TaskCompleted(1));
    let _fail = AliasActorMsg::TaskFailed(TaskFailed(2));
}

#[test]
fn aliased_message_name() {
    assert_eq!(AliasActorMsg::static_name(), "AliasActorMsg");
    assert_eq!(NextId::static_name(), "NextId");
    assert_eq!(TaskCompleted::static_name(), "TaskCompleted");
    assert_eq!(TaskFailed::static_name(), "TaskFailed");
}

#[tokio::test]
async fn aliased_actor_ref_send_and_ask() {
    let mut system = System::global();

    let (actor_ref, stream) =
        system.register_ref::<AliasActor, _, AliasActorMpscRef>("alias_actor").await;

    let actor = AliasActor {
        last_next_id: None,
        last_label: None,
        completed_count: 0,
        failed_count: 0,
    };
    system
        .spawn_actor(actor_ref.name(), actor, (), stream)
        .await
        .unwrap();

    // Send aliased newtypes — same inner type (u64) but different handlers
    actor_ref.send(TaskCompleted(100)).unwrap();
    actor_ref.send(TaskFailed(200)).unwrap();
    actor_ref.send(TaskCompleted(101)).unwrap();

    // Send other aliased types
    actor_ref.send(NextId(42)).unwrap();

    // Ask with regular PingMsg
    let pong = actor_ref.ask::<PongMsg>(PingMsg).await.unwrap();
    assert_eq!(pong.name(), "PongMsg");
}
