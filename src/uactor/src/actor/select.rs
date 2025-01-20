use crate::actor::abstract_actor::Actor;
use crate::actor::message::Message;
use crate::data::datasource::{DataSource, DataSourceErrors, DataSourceResult};
use std::future::pending;

pub type SelectError = DataSourceErrors;

pub type SelectResult<A: Actor> = DataSourceResult<A::RouteMessage>;

pub trait ActorSelect<A: Actor + Send> {
    fn select(&mut self) -> impl std::future::Future<Output = SelectResult<A>> + Send;
}

#[doc(hidden)]
#[allow(non_snake_case)]
mod select_from_tuple {
    use super::*;
    use std::any::type_name;

    macro_rules! select_from_tuple {
        ($($T: ident),*) => {

            impl<MESSAGE, A, $($T),+> ActorSelect<A> for ($($T),+)
                where
                    MESSAGE: Message + Send,
                    A: Actor<RouteMessage=MESSAGE> + Send,
                    $($T: DataSource<Item=MESSAGE> + Send, )*
                    <A as Actor>::Inject: Send,
            {
                async fn select(&mut self) -> SelectResult<A> {
                    let ($($T, )*) = self;
                    tokio::select! {
                        $(
                        msg_res = $T.next() => {
                            msg_res
                        }
                        )*
                    }
                }
            }
        };
    }

    impl<A: Actor + Send> ActorSelect<A> for ()
    where
        <A as Actor>::Inject: Send,
    {
        async fn select(&mut self) -> SelectResult<A> {
            pending::<SelectResult<A>>().await
        }
    }

    impl<A, C1, M> ActorSelect<A> for C1
    where
        M: Message + Send,
        A: Actor<RouteMessage=M> + Send,
        C1: DataSource<Item=M> + Send,
        <A as Actor>::Inject: Send,
    {
        #[cfg(feature = "tokio_tracing")]
        async fn select(&mut self) -> SelectResult<A> {
            // let message_name = <S1 as DataSource>::Item::static_name();
            let _: &'static str = type_name::<<C1 as DataSource>::Item>();
            self.next().await
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
