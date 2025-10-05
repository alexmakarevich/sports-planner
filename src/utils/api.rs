use axum::{http::StatusCode, Json};
use log::error;
use sqlx::{Error, PgPool};

pub type ApiResult<T> = Result<(StatusCode, Json<T>), (StatusCode, String)>;

pub type EmptyApiResult = Result<StatusCode, (StatusCode, String)>;
#[derive(Clone)]
pub struct AppState {
    // FYI: no Arc+Mutex necessary, because pool implements
    // clone and send+sync
    pub pg_pool: PgPool,
}

pub fn handle_unexpected_db_err(err: Error) -> (StatusCode, String) {
    error!("{}", err);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Unexpected Error".to_string(),
    )
}
