use axum_extra::extract::cookie::Cookie;
use log::{debug, error};
use rand::{
    distr::{Alphanumeric, SampleString},
    rng,
};
use serde::{Deserialize, Serialize};

use axum::{
    extract::{Path, State},
    http::{header::SET_COOKIE, HeaderMap, StatusCode},
    response::IntoResponse,
    Extension, Json,
};
use sqlx::Error;

use crate::{
    auth::utils::AuthContext,
    utils::api::{handle_unexpected_db_err, ApiResult, EmptyApiResult},
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

#[derive(Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct SignUpWithNewOrgParams {
    pub username: String,
    pub password: String,
    pub org_title: String,
}

// user from DB wihtout security and unnecessary util fields
#[derive(Serialize, Debug, Clone, sqlx::FromRow)]
pub struct UserClean {
    pub id: String,
    pub username: String,
}

// TODO: sign_up_via_invite

pub async fn sign_up_with_new_org(
    State(state): State<AppState>,
    Json(payload): Json<SignUpWithNewOrgParams>,
) -> Result<(StatusCode, HeaderMap, Json<String>), (StatusCode, String)> {
    let mut tx = state
        .pg_pool
        .begin()
        .await
        .map_err(handle_unexpected_db_err)?;

    let created_org = sqlx::query!(
        r#"INSERT INTO orgs (title) VALUES ($1) RETURNING id"#,
        payload.org_title,
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(handle_unexpected_db_err)?;

    let new_user = sqlx::query!(
        r#"INSERT INTO users (username, password, org_id) VALUES ($1, $2, $3) RETURNING id"#,
        payload.username,
        payload.password,
        created_org.id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(handle_unexpected_db_err)?;

    let _ = sqlx::query!(
        r#"INSERT INTO role_assignments (user_id, role) VALUES ($1, 'org_admin') RETURNING id"#,
        new_user.id,
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(handle_unexpected_db_err)?;

    // Create new session
    let session_id = Alphanumeric.sample_string(&mut rng(), 16);
    // TODO: does the cookie have all the correct security settings by default?

    // Save to DB
    // TODO: session TTL in DB same as expires in browser
    let _ = sqlx::query!(
        "INSERT INTO sessions (id, user_id) VALUES ($1, $2)",
        session_id,
        new_user.id
    )
    .execute(&mut *tx)
    .await
    .map_err(handle_unexpected_db_err)?;

    let _ = tx.commit().await.map_err(handle_unexpected_db_err)?;

    let cookie = Cookie::new("session_id", session_id.clone());
    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.to_string().parse().unwrap());

    Ok((StatusCode::CREATED, headers, Json(new_user.id)))
}

// TODO: disable entirely or conver to global superadmin fn
pub async fn create_user(
    State(state): State<AppState>,
    auth_ctx: Extension<AuthContext>,
    Json(payload): Json<CreateUser>,
) -> ApiResult<String> {
    let username = payload.username;
    let password = payload.password;

    let query_result = sqlx::query!(
        r#"INSERT INTO users (username, password, org_id) VALUES ($1, $2, $3) RETURNING id"#,
        username,
        password,
        auth_ctx.org_id
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

// TODO: rights
pub async fn delete_user_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
    auth_ctx: Extension<AuthContext>,
) -> EmptyApiResult {
    debug!("delete user by id called");
    debug!("{}", id);
    debug!(
        "delete_user_by_id, auth ctx - user: {} session: {}, org: {}",
        auth_ctx.user_id, auth_ctx.session_id, auth_ctx.org_id
    );

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
        "list_users, auth ctx - user: {} session: {}, org: {}",
        auth_ctx.user_id, auth_ctx.session_id, auth_ctx.org_id
    );

    let query_result = sqlx::query_as!(
        UserClean,
        r#"SELECT id, username FROM users WHERE org_id = $1 ORDER by id"#,
        auth_ctx.org_id
    )
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
