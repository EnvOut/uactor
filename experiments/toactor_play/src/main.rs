#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
use tokactor::{Actor, Ask, AskResult, Ctx};

struct Router {
}

pub struct PingPong {
    counter: u8,
}

/// This is the types of message [PingPong] supports
#[derive(Debug, Clone)]
pub enum Msg {
    Ping,
    Pong,
}
impl Msg {
    // retrieve the next message in the sequence
    fn next(&self) -> Self {
        match self {
            Self::Ping => Self::Pong,
            Self::Pong => Self::Ping,
        }
    }
    // print out this message
    fn print(&self) {
        match self {
            Self::Ping => print!("ping.."),
            Self::Pong => print!("pong.."),
        }
    }
}

impl Actor for PingPong {}
impl Ask<Msg> for PingPong {
    type Result = Msg;

    // This is our main message handler
    fn handle(&mut self, message: Msg, _: &mut Ctx<Self>) -> AskResult<Self::Result> {
        message.print();
        self.counter += 1;
        AskResult::Reply(message.next())
    }
}

#[tokio::main]
async fn main() {
    // let world = World::new().unwrap();
    //
    // let db = world.with_state(async || Db::connect().await.unwrap());
    //
    // let router = Router { db };
    //
    // let tcp_input = world.tcp_component("localhost:8080", router);
    //
    // world.on_input(tcp_input);
    //
    // world.block_until_completion();
    let handle = PingPong { counter: 0 }.start();
    let mut message = Msg::Ping;
    for _ in 0..10 {
        message = handle.ask(message).await.unwrap();
    }
    let actor = handle
        .await
        .expect("Ping-pong actor failed to exit properly");
    assert_eq!(actor.counter, 10);
    println!("\nProcessed {} messages", actor.counter);
}