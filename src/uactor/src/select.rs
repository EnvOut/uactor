use std::future::pending;
use crate::actor::{Actor, Handler};
use crate::context::Context;
use crate::datasource::DataSource;
use crate::message::Message;

#[async_trait::async_trait]
pub trait ActorSelect<Z: Actor> {
    async fn select(&mut self, ctx: &mut Context, actor: &mut Z) -> SelectResult;
}

pub type SelectResult = Result<(), Box<dyn std::error::Error>>;

#[doc(hidden)]
#[allow(non_snake_case)]
mod select_from_tuple {
    use super::*;

    macro_rules! select_from_tuple {
        ($($T: ident),*) => {
            #[async_trait::async_trait]
            impl<A, $($T),+> ActorSelect<A> for ($($T),+)
                where
                    $($T::Item: Message + Send, )*
                    $($T: DataSource + Send, )*
                    A: $(Handler<$T::Item> + )* Send,
            {
                async fn select(&mut self, ctx: &mut Context, actor: &mut A) -> SelectResult {
                    let ($($T, )*) = self;
                    tokio::select! {
                        $(
                        Ok(msg) = $T.next() => {
                            let _ = actor.handle(msg, ctx).await?;
                        }
                        )*
                    }
                    Ok(())
                }
            }
        };
    }

    #[async_trait::async_trait]
    impl<A: Actor> ActorSelect<A> for () {
        async fn select(&mut self, _: &mut Context, _: &mut A) -> SelectResult {
            let never = pending::<()>();
            never.await;
            Ok(())
        }
    }

    #[async_trait::async_trait]
    impl<A, S1> ActorSelect<A> for S1
        where
            S1::Item: Message + Send,
            S1: DataSource + Send,
            A: Handler<S1::Item> + Send,
    {
        async fn select(&mut self, ctx: &mut Context, actor: &mut A) -> SelectResult {
            if let Ok(msg) = self.next().await {
                let _ = actor.handle(msg, ctx).await?;
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

