use log::debug;
use serde::{Deserialize, Serialize};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};

use crate::{
    entities::auth::AuthContext,
    utils::api::{ApiResult, EmptyApiResult},
    AppState,
};

// For sqlx
// all the fields - not sure if needed, but will leave here for now
#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
#[allow(non_snake_case)] // TODO: why though? it's all snaky anyway
pub struct UserModel {
    pub id: String,
    pub username: String,
    pub password: String,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

// the input to our `create_user` handler
#[derive(Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub password: String,
}

// user from DB wihtout security and unnecessary util fields
#[derive(Serialize, Debug, Clone, sqlx::FromRow)]
pub struct UserClean {
    pub id: String,
    pub username: String,
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUser>,
) -> ApiResult<String> {
    let username = payload.username;
    let password = payload.password;

    let query_result = sqlx::query!(
        r#"INSERT INTO users (username, password) VALUES ($1, $2) RETURNING id"#,
        username,
        password
    )
    .fetch_one(&state.pg_pool)
    .await;

    match query_result {
        Err(e) => {
            let error_response = serde_json::json!({
            "status": "error",
            "message": format!("Database error: { }", e),
            })
            .to_string();
            Err((StatusCode::INTERNAL_SERVER_ERROR, error_response))
        }
        Ok(record) => Ok((StatusCode::CREATED, Json(record.id))),
    }
}

pub async fn delete_user_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> EmptyApiResult {
    debug!("delete by id called");
    debug!("{}", id);

    let query_result = sqlx::query!(r#"DELETE FROM users WHERE id = $1"#, id)
        .execute(&state.pg_pool)
        .await;

    match query_result {
        Err(e) => {
            let error_response = serde_json::json!({
            "status": "error",
            "message": format!("Database error: { }", e),
            })
            .to_string();
            Err((StatusCode::INTERNAL_SERVER_ERROR, error_response))
        }
        Ok(result_info) => {
            if result_info.rows_affected() == 0 {
                Err((
                    StatusCode::NOT_ACCEPTABLE,
                    "User with given ID does not exist - possibly already deleted".to_string(),
                ))
            } else {
                Ok(StatusCode::NO_CONTENT)
            }
        }
    }
}

pub async fn list_users(
    State(state): State<AppState>,
    auth_ctx: Extension<AuthContext>,
) -> ApiResult<Vec<UserClean>> {
    debug!(
        "list_users, auth ctx - user: {} session: {}",
        auth_ctx.user_id, auth_ctx.session_id
    );

    let query_result = sqlx::query_as!(UserClean, r#"SELECT id, username FROM users ORDER by id"#)
        .fetch_all(&state.pg_pool)
        .await;

    match query_result {
        Err(e) => {
            let error_response = serde_json::json!({
            "status": "error",
            "message": format!("Database error: { }", e),
            })
            .to_string();
            Err((StatusCode::INTERNAL_SERVER_ERROR, error_response))
        }
        Ok(users) => Ok((StatusCode::OK, Json(users))),
    }
}
