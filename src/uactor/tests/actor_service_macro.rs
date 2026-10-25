use std::fmt::{Display, Formatter};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use uactor::actor::abstract_actor::Actor;
use uactor::actor::context::{ActorContext, Context};
use uactor::actor::message::Message;
use uactor::dependency_injection::{Inject, InjectError};
use uactor::system::System;

#[derive(Debug, PartialEq, Eq)]
struct AppError(&'static str);

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for AppError {}

struct Calculator {
    last_site: Option<i32>,
    last_user: Option<i32>,
}

impl Actor for Calculator {
    type Context = Context;
    type RouteMessage = CalculatorMsg;
    type Inject = ();
    type State = ();
}

#[uactor::service]
impl Calculator {
    async fn do_something(&mut self, site_id: i32, user_id: i32) -> Result<(), AppError> {
        self.last_site = Some(site_id);
        self.last_user = Some(user_id);
        Ok(())
    }

    async fn fail_command(&mut self) -> Result<(), AppError> {
        Err(AppError("command failed"))
    }

    async fn sum(&mut self, a: i32, b: i32) -> Result<i32, AppError> {
        if a < 0 || b < 0 {
            return Err(AppError("negative"));
        }
        Ok(a + b)
    }

    async fn ping(&self) {
        // fire-and-forget with no payload
    }

    async fn last_site(&self) -> Option<i32> {
        self.last_site
    }

    #[skip]
    fn helper_scale(&self, value: i32) -> i32 {
        value * 2
    }

    async fn scaled_sum(&mut self, a: i32, b: i32) -> i32 {
        self.helper_scale(a) + self.helper_scale(b)
    }
}

async fn spawn_calculator() -> CalculatorMpscRef {
    let mut system = System::global();
    let (actor_ref, stream) = system
        .register_ref::<Calculator, _, CalculatorMpscRef>("calculator")
        .await;
    let actor = Calculator {
        last_site: None,
        last_user: None,
    };
    system
        .spawn_actor(actor_ref.name(), actor, (), stream)
        .await
        .unwrap();
    actor_ref
}

#[test]
fn generated_message_types_exist() {
    let _ = CalculatorMsg::DoSomething(DoSomething {
        site_id: 1,
        user_id: 2,
    });
    let _ = CalculatorMsg::Ping(Ping);
    assert_eq!(DoSomething::static_name(), "DoSomething");
    assert_eq!(Sum::static_name(), "Sum");
    assert_eq!(Ping::static_name(), "Ping");
}

#[tokio::test]
async fn send_updates_actor_state() {
    let actor_ref = spawn_calculator().await;

    actor_ref.send_do_something(10, 20).unwrap();

    let site = actor_ref.ask_last_site().await.unwrap();
    assert_eq!(site, Some(10));
}

#[tokio::test]
async fn send_and_ask_roundtrip() {
    let actor_ref = spawn_calculator().await;

    actor_ref.send_do_something(7, 8).unwrap();
    actor_ref.send_ping().unwrap();

    let sum = actor_ref.ask_sum(1, 2).await.unwrap();
    assert_eq!(sum, Ok(3));
}

#[tokio::test]
async fn ask_returns_handler_error_without_stopping_actor() {
    let actor_ref = spawn_calculator().await;

    let failed = actor_ref.ask_sum(-1, 2).await.unwrap();
    assert_eq!(failed, Err(AppError("negative")));

    let ok = actor_ref.ask_sum(4, 5).await.unwrap();
    assert_eq!(ok, Ok(9));
}

#[tokio::test]
async fn ask_plain_value() {
    let actor_ref = spawn_calculator().await;
    let scaled = actor_ref.ask_scaled_sum(3, 4).await.unwrap();
    assert_eq!(scaled, 14);
}

#[tokio::test]
async fn expect_unwraps_only_the_actor_transport_result() {
    let actor_ref = spawn_calculator().await;

    let failed = actor_ref.expect_sum(-1, 2).await;
    assert_eq!(failed, Err(AppError("negative")));

    let scaled = actor_ref.expect_scaled_sum(3, 4).await;
    assert_eq!(scaled, 14);
}

#[tokio::test]
#[should_panic(expected = "Calculator is expected to be alive for message ScaledSum")]
async fn expect_panics_when_actor_is_unavailable() {
    let mut system = System::global();
    let (actor_ref, stream) = system
        .register_ref::<Calculator, _, CalculatorMpscRef>("unavailable_calculator")
        .await;
    drop(stream);

    let _ = actor_ref.expect_scaled_sum(1, 2).await;
}

#[tokio::test]
async fn send_error_does_not_stop_actor() {
    let actor_ref = spawn_calculator().await;

    actor_ref.send_fail_command().unwrap();

    let sum = actor_ref.ask_sum(2, 3).await.unwrap();
    assert_eq!(sum, Ok(5));
}

struct Stoppable;

impl Actor for Stoppable {
    type Context = Context;
    type RouteMessage = StoppableMsg;
    type Inject = ();
    type State = Arc<AtomicU32>;
}

#[uactor::service]
impl Stoppable {
    async fn bump(&self, state: &Arc<AtomicU32>) -> Result<(), AppError> {
        state.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    async fn stop(&self, ctx: &mut Context, state: &Arc<AtomicU32>) {
        state.fetch_add(10, Ordering::Relaxed);
        ctx.kill();
    }
}

#[tokio::test]
async fn service_methods_receive_ctx_and_state() {
    let mut system = System::global();
    let (actor_ref, stream) = system
        .register_ref::<Stoppable, _, StoppableMpscRef>("stoppable_service")
        .await;

    let state = Arc::new(AtomicU32::new(0));
    let (_, handle) = system
        .spawn_actor(actor_ref.name(), Stoppable, state.clone(), stream)
        .await
        .unwrap();

    actor_ref.send_bump().unwrap();
    actor_ref.send_stop().unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    assert!(handle.is_finished());
    assert_eq!(state.load(Ordering::Relaxed), 11);
}

struct Deps {
    value: i32,
}

impl Inject for Deps {
    async fn inject(_: &System) -> Result<Self, InjectError> {
        Ok(Self { value: 41 })
    }
}

struct WithInject;

impl Actor for WithInject {
    type Context = Context;
    type RouteMessage = WithInjectMsg;
    type Inject = Deps;
    type State = ();
}

#[uactor::service]
impl WithInject {
    async fn add(&self, n: i32, #[inject] value: &mut i32) -> i32 {
        *value + n
    }
}

#[tokio::test]
async fn inject_param_is_passed_from_inject_struct() {
    let mut system = System::global();
    let (actor_ref, stream) = system
        .register_ref::<WithInject, _, WithInjectMpscRef>("with_inject")
        .await;
    system
        .spawn_actor(actor_ref.name(), WithInject, (), stream)
        .await
        .unwrap();

    let result = actor_ref.ask_add(1).await.unwrap();
    assert_eq!(result, 42);
}
