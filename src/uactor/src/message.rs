use std::any::type_name;
use std::num::{
    NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use tokio::time::Instant;

pub trait Message {
    fn static_name() -> &'static str;

    fn name(&self) -> String {
        Self::static_name().to_owned()
    }
}

impl<A, B> Message for Result<A, B> where A: Message, B: Message { fn static_name() -> &'static str { type_name::<Result<A, B>>() } }
impl<A> Message for Option<A> where A: Message { fn static_name() -> &'static str { type_name::<Option<String>>() }}
impl<A> Message for Arc<A> where A: Message { fn static_name() -> &'static str { type_name::<Arc<String>>() }}
impl<A> Message for Mutex<A> where A: Message { fn static_name() -> &'static str { type_name::<Mutex<String>>() }}
impl<A> Message for RwLock<A> where A: Message { fn static_name() -> &'static str { type_name::<RwLock<String>>() }}

#[macro_export]
macro_rules! message_impl {
    ($($T: ident),*) => {
        $(
            impl Message for $T { fn static_name() -> &'static str {
                stringify!($T)
            }}
        )*
    };
}

type Empty = ();

pub struct IntervalMessage {
    pub time: Instant,
    pub duration: Duration,
}

pub type Reply<T> = tokio::sync::oneshot::Sender<T>;

message_impl! { IntervalMessage, Empty, i64, i32, i16, i8, u64, u32, u16, u8, f64, f32, String, NonZeroI64, NonZeroI32, NonZeroI16, NonZeroI8, NonZeroU64, NonZeroU32, NonZeroU16, NonZeroU8 }

#[cfg(feature = "bytes")]
impl Message for bytes::BytesMut { }