use crate::actor::{Actor, HandleResult, Handler};
use crate::context::Context;
use crate::datasource::DataSource;
use crate::message::Message;
use std::future::pending;

pub trait ActorSelect<A: Actor + Send> {
    fn select(&mut self, inject: &mut A::Inject, ctx: &mut <A as Actor>::Context, actor: &mut A) -> impl std::future::Future<Output = SelectResult> + Send;
}

pub type SelectResult = HandleResult;

#[doc(hidden)]
#[allow(non_snake_case)]
mod select_from_tuple {
    use std::any::type_name;
    use tracing::Instrument;
    use crate::context::ActorContext;
    use super::*;

    macro_rules! select_from_tuple {
        ($($T: ident),*) => {

            impl<A, $($T),+> ActorSelect<A> for ($($T),+)
                where
                    $($T::Item: Message + Send, )*
                    $($T: DataSource + Send, )*
                    A: $(Handler<$T::Item> + )* Send,
                    <A as Actor>::Inject: Send
            {
                async fn select(&mut self, inject: &mut A::Inject, ctx: &mut <A as Actor>::Context, actor: &mut A) -> SelectResult {
                    let ($($T, )*) = self;
                    tokio::select! {
                        $(
                        Ok(msg) = $T.next() => {
                            let _ = actor.handle(inject, msg, ctx).await?;
                        }
                        )*
                    }
                    Ok(())
                }
            }
        };
    }

    impl<A: Actor + Send> ActorSelect<A> for ()
    where
        <A as Actor>::Inject: Send,
    {
        async fn select(&mut self, _: &mut A::Inject, _: &mut <A as Actor>::Context, _: &mut A) -> SelectResult {
            pending::<SelectResult>().await
        }
    }

    impl<A, S1> ActorSelect<A> for S1
        where
            S1::Item: Message + Send,
            S1: DataSource + Send,
            A: Handler<S1::Item> + Send,
            <A as Actor>::Inject: Send,
    {
        #[cfg(feature = "tokio_tracing")]
        async fn select(&mut self, inject: &mut A::Inject, ctx: &mut <A as Actor>::Context, actor: &mut A) -> SelectResult {
            // let message_name = <S1 as DataSource>::Item::static_name();
            let message_name: &'static str = type_name::<<S1 as DataSource>::Item>();
            if let Ok(msg) = self.next().await {
                let mut span = tracing::span!(tracing::Level::INFO, "Actor::handle", actor.name = ctx.get_name(), actor.message = message_name);
                let _enter = span.enter();
                let _ = actor.handle(inject, msg, ctx).await?;
            } else {
                tracing::error!("Channel closed");
            }
            Ok(())
        }

        #[cfg(not(feature = "tokio_tracing"))]
        async fn select(&mut self, inject: &mut A::Inject, ctx: &mut <A as Actor>::Context, actor: &mut A) -> SelectResult {
            if let Ok(msg) = self.next().await {
                let _ = actor.handle(inject, msg, ctx).await?;
            } else {
                tracing::error!("Channel closed");
            }
            Ok(())
        }
    }

    select_from_tuple! { S1, S2 }
    select_from_tuple! { S1, S2, S3 }
    select_from_tuple! { S1, S2, S3, S4 }
    select_from_tuple! { S1, S2, S3, S4, S5 }
    select_from_tuple! { S1, S2, S3, S4, S5, S6 }
    select_from_tuple! { S1, S2, S3, S4, S5, S6, S7 }
    select_from_tuple! { S1, S2, S3, S4, S5, S6, S7, S8 }
    select_from_tuple! { S1, S2, S3, S4, S5, S6, S7, S8, S9 }
    select_from_tuple! { S1, S2, S3, S4, S5, S6, S7, S8, S9, S10 }
    select_from_tuple! { S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11 }
    select_from_tuple! { S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12 }
    select_from_tuple! { S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13 }
    select_from_tuple! { S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14 }
    select_from_tuple! { S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14, S15 }
    select_from_tuple! { S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14, S15, S16 }
    select_from_tuple! { S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14, S15, S16, S17 }
    select_from_tuple! { S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14, S15, S16, S17, S18 }
    select_from_tuple! { S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14, S15, S16, S17, S18, S19 }
    select_from_tuple! { S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14, S15, S16, S17, S18, S19, S20 }
    select_from_tuple! { S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14, S15, S16, S17, S18, S19, S20, S21 }
    select_from_tuple! { S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14, S15, S16, S17, S18, S19, S20, S21, S22 }
    select_from_tuple! { S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14, S15, S16, S17, S18, S19, S20, S21, S22, S23 }
    select_from_tuple! { S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14, S15, S16, S17, S18, S19, S20, S21, S22, S23, S24 }
    select_from_tuple! { S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14, S15, S16, S17, S18, S19, S20, S21, S22, S23, S24, S25 }
    select_from_tuple! { S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14, S15, S16, S17, S18, S19, S20, S21, S22, S23, S24, S25, S26 }
    select_from_tuple! { S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14, S15, S16, S17, S18, S19, S20, S21, S22, S23, S24, S25, S26, S27 }
    select_from_tuple! { S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14, S15, S16, S17, S18, S19, S20, S21, S22, S23, S24, S25, S26, S27, S28 }
    select_from_tuple! { S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14, S15, S16, S17, S18, S19, S20, S21, S22, S23, S24, S25, S26, S27, S28, S29 }
    select_from_tuple! { S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14, S15, S16, S17, S18, S19, S20, S21, S22, S23, S24, S25, S26, S27, S28, S29, S30 }
}
