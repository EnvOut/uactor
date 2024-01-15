use std::error::Error;
use std::future::Future;
use actix_web::{HttpRequest, web};
use actix_web::web::Payload;

pub trait FromRequest: Sized {
    type Error: Into<actix_web::error::Error>;
    type Future: Future<Output = Result<Self, Self::Error>>;

    // Required method
    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future;

    // Provided method
    fn extract(req: &HttpRequest) -> Self::Future { unimplemented!() }
}

#[derive(serde::Deserialize)]
pub struct Info {
    pub name: String
}

#[post("/")]
async fn index(info: web::Json<Info>) -> String {
    format!("Welcome {}!", info.username)
}