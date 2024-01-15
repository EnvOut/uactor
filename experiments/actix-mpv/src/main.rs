use actix::{Actor, ActorContext, AsyncContext, Context, Handler, Message};

fn main() {
    struct MyActor;

    struct WhoAmI;

    impl Message for WhoAmI {
        type Result = Result<actix::Addr<MyActor>, ()>;
    }

    impl Actor for MyActor {
        type Context = Context<Self>;
    }

    impl Handler<WhoAmI> for MyActor {
        type Result = Result<actix::Addr<MyActor>, ()>;

        fn handle(&mut self, msg: WhoAmI, ctx: &mut Context<Self>) -> Self::Result {
            ctx.stop();
            Ok(ctx.address())
        }
    }

    // let who_addr = addr.do_send(WhoAmI{});
    println!("Hello, world!");
}
