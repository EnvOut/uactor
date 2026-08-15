//! Demonstrates the service-style actor API (`#[uactor::service]`):
//! - `Result<(), E>` / `()` → fire-and-forget `send_<method>(...)`
//! - `Result<T, E>` / `T` → `ask_<method>(...).await` (the method result, including `Err`)

use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use uactor::system::System;

use crate::my_actor::{MyActor, MyActorMpscRef};

mod my_actor {
    use std::fmt::{Display, Formatter};
    use tracing::info;
    use uactor::actor::abstract_actor::Actor;
    use uactor::actor::context::Context;

    #[derive(Debug, PartialEq, Eq)]
    pub struct AppError(pub &'static str);

    impl Display for AppError {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl std::error::Error for AppError {}

    pub struct MyActor {
        last_site_id: Option<i32>,
    }

    impl MyActor {
        pub fn new() -> Self {
            Self { last_site_id: None }
        }
    }

    impl Actor for MyActor {
        type Context = Context;
        type RouteMessage = MyActorMsg;
        type Inject = ();
        type State = ();
    }

    #[uactor::service]
    impl MyActor {
        #[tracing::instrument(skip(self))]
        async fn do_something(&mut self, site_id: i32, user_id: i32) -> Result<(), AppError> {
            info!("do_something site_id={site_id} user_id={user_id}");
            self.last_site_id = Some(site_id);
            Ok(())
        }

        async fn sum(&mut self, a: i32, b: i32) -> Result<i32, AppError> {
            if a < 0 || b < 0 {
                return Err(AppError("negative numbers are not supported"));
            }
            Ok(a + b)
        }

        async fn last_site_id(&self) -> Option<i32> {
            self.last_site_id
        }
    }
}

async fn logic_example(my_actor_ref: &MyActorMpscRef) -> anyhow::Result<()> {
    let site_id = 1;
    let user_id = 1;
    my_actor_ref.send_do_something(site_id, user_id)?;

    let sum = my_actor_ref.ask_sum(1, 2).await?;
    info!("sum = {sum:?}");
    assert_eq!(sum, Ok(3));

    let failed = my_actor_ref.ask_sum(-1, 2).await?;
    info!("failed sum = {failed:?}");
    assert_eq!(
        failed,
        Err(my_actor::AppError(
            "negative numbers are not supported"
        ))
    );

    let last = my_actor_ref.ask_last_site_id().await?;
    assert_eq!(last, Some(1));

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(LevelFilter::INFO)
        .with(tracing_subscriber::fmt::layer())
        .init();

    let mut system = System::global();

    let (my_actor_ref, stream) = system
        .register_ref::<MyActor, _, MyActorMpscRef>("my_actor")
        .await;

    system
        .spawn_actor(my_actor_ref.name(), MyActor::new(), (), stream)
        .await?;

    logic_example(&my_actor_ref).await?;

    info!("done");
    Ok(())
}
