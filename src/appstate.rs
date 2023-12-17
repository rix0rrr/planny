use std::sync::atomic::AtomicUsize;

use rocket::serde::Serialize;

#[derive(Serialize)]
pub struct AppState {
    pub count: AtomicUsize,
}

impl AppState {
    pub fn new() -> Self {
        AppState { count: 0.into() }
    }
}
