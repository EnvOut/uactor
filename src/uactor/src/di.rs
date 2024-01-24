use std::any::Any;
use crate::actor::Actor;
use crate::context::extensions::Service;
use crate::system::System;

pub struct MyState {
    var1: String,
    var2: String,
}

pub type InjectError = Box<dyn std::error::Error + Send + Sync + 'static>;

pub trait Inject {
    async fn inject(system: &System) -> Result<Self, InjectError> where Self: Sized;
}


impl Inject for MyState {
    async fn inject(system: &System) -> Result<Self, InjectError> where Self: Sized {
        let var1 = system.get_service::<String>()?.clone();
        let var2 = system.get_service::<String>()?.clone();
        Ok(Self { var1: var1, var2: var2 })
    }
}