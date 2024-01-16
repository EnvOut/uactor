use std::future::pending;
use crate::actor::{Actor, Handler};
use crate::datasource::DataSource;
use crate::message::Message;

#[async_trait::async_trait]
pub trait ActorSelect<Z: Actor> {
    async fn select(&mut self, ctx: &mut Z::Context, actor: &mut Z) -> SelectResult;
}

pub type SelectResult = Result<(), Box<dyn std::error::Error>>;

#[async_trait::async_trait]
impl <A: Actor> ActorSelect<A> for (){
    async fn select(&mut self, _: &mut A::Context, _: &mut A) -> SelectResult {
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
    async fn select(&mut self, ctx: &mut A::Context, actor: &mut A) -> SelectResult {
        tokio::select! {
            Ok(msg) = self.next() => {
                let _ = actor.handle(msg, ctx).await;
            }
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl<A, S1, S2> ActorSelect<A> for (S1, S2)
    where
        S1::Item: Message + Send, S2::Item: Message + Send,
        S1: DataSource + Send, S2: DataSource + Send,
        A: Handler<S1::Item> + Handler<S2::Item> + Send,
{
    async fn select(&mut self, ctx: &mut A::Context, actor: &mut A) -> SelectResult {
        tokio::select! {
            Ok(msg) = self.0.next() => {
                let _ = actor.handle(msg, ctx).await;
            }
            Ok(msg) = self.1.next() => {
                let _ = actor.handle(msg, ctx).await?;
            }
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl<A, S1, S2, S3> ActorSelect<A> for (S1, S2, S3)
    where
        S1::Item: Message + Send, S2::Item: Message + Send, S3::Item: Message + Send + Send,
        S1: DataSource + Send, S2: DataSource + Send, S3: DataSource + Send,
        A: Handler<S1::Item> + Handler<S2::Item> + Handler<S3::Item> + Send,
{
    async fn select(&mut self, ctx: &mut A::Context, actor: &mut A) -> SelectResult {
        tokio::select! {
            Ok(msg) = self.0.next() => {
                let _ = actor.handle(msg, ctx).await?;
            }
            Ok(msg) = self.1.next() => {
                let _ = actor.handle(msg, ctx).await?;
            }
            Ok(msg) = self.2.next() => {
                let _ = actor.handle(msg, ctx).await?;
            }
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl<A, S1, S2, S3, S4> ActorSelect<A> for (S1, S2, S3, S4)
    where
        S1::Item: Message + Send, S2::Item: Message + Send, S3::Item: Message + Send, S4::Item: Message + Send,
        S1: DataSource + Send, S2: DataSource + Send, S3: DataSource + Send, S4: DataSource + Send,
        A: Handler<S1::Item> + Handler<S2::Item> + Handler<S3::Item> + Handler<S4::Item> + Send,
{
    async fn select(&mut self, ctx: &mut A::Context, actor: &mut A) -> SelectResult {
        tokio::select! {
            Ok(msg) = self.0.next() => {
                let _ = actor.handle(msg, ctx).await?;
            }
            Ok(msg) = self.1.next() => {
                let _ = actor.handle(msg, ctx).await?;
            }
            Ok(msg) = self.2.next() => {
                let _ = actor.handle(msg, ctx).await?;
            }
            Ok(msg) = self.3.next() => {
                let _ = actor.handle(msg, ctx).await?;
            }
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl<A, S1, S2, S3, S4, S5> ActorSelect<A> for (S1, S2, S3, S4, S5)
    where
        S1::Item: Message + Send, S2::Item: Message + Send, S3::Item: Message + Send, S4::Item: Message + Send, S5::Item: Message + Send,
        S1: DataSource + Send, S2: DataSource + Send, S3: DataSource + Send, S4: DataSource + Send, S5: DataSource + Send,
        A: Handler<S1::Item> + Handler<S2::Item> + Handler<S3::Item> + Handler<S4::Item> + Handler<S5::Item> + Send,
{
    async fn select(&mut self, ctx: &mut A::Context, actor: &mut A) -> SelectResult {
        tokio::select! {
            Ok(msg) = self.0.next() => {
                let _ = actor.handle(msg, ctx).await?;
            }
            Ok(msg) = self.1.next() => {
                let _ = actor.handle(msg, ctx).await?;
            }
            Ok(msg) = self.2.next() => {
                let _ = actor.handle(msg, ctx).await?;
            }
            Ok(msg) = self.3.next() => {
                let _ = actor.handle(msg, ctx).await?;
            }
            Ok(msg) = self.4.next() => {
               let _ = actor.handle(msg, ctx).await?;
            }
        }
        Ok(())
    }
}
