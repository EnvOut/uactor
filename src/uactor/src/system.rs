use tokio::task::JoinHandle;
use crate::actor::Actor;
use crate::select::ActorSelect;

pub struct System {
    name: String,
}

impl System {
    pub fn global() -> Self {
        System { name: "Global".to_string() }
    }
}

impl System {
    pub async fn run<A, S>(&self, mut actor: A, mut select: S) -> JoinHandle<()>
        where
            A: Actor + Send,
            S: ActorSelect<A> + Send + 'static
    {
        let system_name = self.name.clone();
        let process_name = std::any::type_name::<A>().to_owned();

        let handle = tokio::spawn(async move {
            tracing::debug!("The system: {:?} spawned process: {:?}", system_name, process_name);

            let mut ctx = A::default_context();

            loop {
                tracing::debug!("iteration of the process: {process_name:?}");
                let result = select.select(&mut ctx, &mut actor).await;
                tracing::debug!("{process_name:?} result: {result:?}");
            }
        });
        handle
    }

    pub async fn run_fn<A, F, S>(&self, f: F, mut select: S) -> JoinHandle<()>
        where
            A: Actor + Send,
            F: FnOnce(&mut A::Context) -> A,
            S: ActorSelect<A> + Send + 'static
    {
        let mut ctx = A::default_context();
        let mut actor = f(&mut ctx);

        let process_name = std::any::type_name::<A>().to_owned();
        let handle = tokio::spawn(async move {
            tracing::debug!("Spawn process: {process_name:?}");

            loop {
                tracing::debug!("iteration of the process: {process_name:?}");
                let result = select.select(&mut ctx, &mut actor).await;
                tracing::debug!("{process_name:?} result: {result:?}");
            }
        });
        handle
    }
}
