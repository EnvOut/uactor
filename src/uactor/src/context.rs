use crate::actor::Actor;

pub trait ActorContext: Default + Sized + Unpin + 'static { }

pub struct Context<A>
    where
        A: Actor<Context=Context<A>>,
{
    // parts: ContextParts<A>,
    // mb: Option<Mailbox<A>>,
    #[allow(dead_code)]
    ll: Option<A>,
}

impl<A> Context<A> where A: Actor<Context=Context<A>>, {
    pub fn new() -> Self {
        Context {ll: None}
    }
}

impl<A> ActorContext for Context<A> where
    A: Actor<Context=Context<A>>, {}

impl<A> Default for Context<A> where
    A: Actor<Context=Context<A>>, {
    fn default() -> Self {
        Self {
            ll: None,
        }
    }
}