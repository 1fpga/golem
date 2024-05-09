use crate::utils::get_core;
use rocket::tokio::sync::Mutex;
use rocket::{request, Request};
use std::sync::Arc;

#[derive(Clone)]
pub struct CoreGuard(Arc<Mutex<mister_fpga::core::MisterFpgaCore>>);
unsafe impl Send for CoreGuard {}
unsafe impl Sync for CoreGuard {}

#[rocket::async_trait]
impl<'r> request::FromRequest<'r> for CoreGuard {
    type Error = String;

    async fn from_request(_request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        match get_core().await {
            Ok(core) => request::Outcome::Success(CoreGuard(Arc::new(Mutex::new(core)))),
            Err(e) => request::Outcome::Error((rocket::http::Status::InternalServerError, e)),
        }
    }
}

impl CoreGuard {
    pub async fn lock(
        &self,
    ) -> rocket::tokio::sync::MutexGuard<'_, mister_fpga::core::MisterFpgaCore> {
        self.0.lock().await
    }
}
