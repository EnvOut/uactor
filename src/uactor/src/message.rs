use std::num::{NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

pub trait Message { }

impl<A, B> Message for Result<A, B> where A: Message, B: Message {}

impl<A> Message for Option<A> where A: Message {}

impl<A> Message for Arc<A> where A: Message {}

impl<A> Message for Mutex<A> where A: Message {}

impl<A> Message for RwLock<A> where A: Message {}

impl Message for i64 {}

impl Message for i32 {}

impl Message for i16 {}

impl Message for i8 {}

impl Message for u64 {}

impl Message for u32 {}

impl Message for u16 {}

impl Message for u8 {}

impl Message for f64 {}

impl Message for f32 {}

impl Message for String {}

impl Message for NonZeroI64 {}

impl Message for NonZeroI32 {}

impl Message for NonZeroI16 {}

impl Message for NonZeroI8 {}

impl Message for NonZeroU64 {}

impl Message for NonZeroU32 {}

impl Message for NonZeroU16 {}

impl Message for NonZeroU8 {}
