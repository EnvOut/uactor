use crate::actor::Actor;
use crate::system::System;

pub struct MyState {
    var1: String,
    var2: String,
}

pub type InjectError = Box<dyn std::error::Error + Send + Sync + 'static>;

pub trait Inject {
    async fn inject(system: &System) -> Self;
    // async fn inject(system: &System) -> Result<Self, InjectError>;
}

// impl Inject for MyState {
//     async fn inject(system: &System) -> Result<Self, InjectError> {
//         todo!()
//     }
// }

