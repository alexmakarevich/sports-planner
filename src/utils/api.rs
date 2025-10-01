use axum::{http::StatusCode, Json};
use sqlx::PgPool;

pub type ApiResult<T> = Result<(StatusCode, Json<T>), (StatusCode, String)>;
pub type EmptyApiResult = Result<StatusCode, (StatusCode, String)>;
#[derive(Clone)]
pub struct AppState {
    pub pg_pool: PgPool,
}
