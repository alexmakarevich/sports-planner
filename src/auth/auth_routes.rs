use axum::{
    extract::{Path, State},
    http::{header::SET_COOKIE, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Extension, Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use log::{debug, error};
use rand::{
    distr::{Alphanumeric, SampleString},
    rng,
};
use serde::Deserialize;
use time::{Duration, OffsetDateTime};

use crate::{
    auth::utils::{AuthContext, EXPIRED_EMPTY_COOKIE},
    entities::{org::create_org, user::UserClean},
    utils::api::{db_err_to_response, handle_unexpected_db_err, AppState},
};

#[derive(sqlx::FromRow)]
pub struct SessionModel {
    pub id: String,
    pub user_id: String,
}

#[derive(Deserialize)]
pub struct LoginParams {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct SignUpWithNewOrgParams {
    pub username: String,
    pub password: String,
    pub org_title: String,
}

#[derive(Deserialize)]
pub struct SignUpViaInviteParams {
    pub username: String,
    pub password: String,
}

struct InviteModel {
    org_id: String,
}

// pub async fn sign_up_via_invite

pub async fn sign_up_via_invite(
    State(state): State<AppState>,
    Path(invite_id): Path<String>,
    Json(payload): Json<SignUpViaInviteParams>,
) -> Result<Response, (StatusCode, String)> {
    let mut tx = state
        .pg_pool
        .begin()
        .await
        .map_err(handle_unexpected_db_err)?;

    let service_invite = sqlx::query_as!(
        InviteModel,
        r#"SELECT org_id FROM service_invites WHERE id = $1"#,
        invite_id,
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(handle_unexpected_db_err)?;

    let new_user = sqlx::query!(
        r#"INSERT INTO users (username, password, org_id) VALUES ($1, $2, $3) RETURNING id"#,
        payload.username,
        payload.password,
        service_invite.org_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(handle_unexpected_db_err)?;

    // Create new session
    let session_id = Alphanumeric.sample_string(&mut rng(), 16);

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

    let cookie = Cookie::build(("session_id", session_id.clone()))
        .secure(true)
        .http_only(true)
        .same_site(SameSite::Strict)
        .expires(OffsetDateTime::now_utc() + Duration::days(7));

    Ok((
        StatusCode::CREATED,
        [(SET_COOKIE, cookie.to_string())],
        Json(new_user.id),
    )
        .into_response())
}

pub async fn sign_up_with_new_org(
    State(state): State<AppState>,
    Json(payload): Json<SignUpWithNewOrgParams>,
) -> Result<(StatusCode, HeaderMap, Json<String>), (StatusCode, String)> {
    let mut tx = state
        .pg_pool
        .begin()
        .await
        .map_err(handle_unexpected_db_err)?;

    let created_org_id = create_org(&mut tx, &payload.org_title)
        .await
        .map_err(handle_unexpected_db_err)?;

    let new_user = sqlx::query!(
        r#"INSERT INTO users (username, password, org_id) VALUES ($1, $2, $3) RETURNING id"#,
        payload.username,
        payload.password,
        created_org_id
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
    let cookie = Cookie::build(("session_id", session_id.clone()))
        .secure(true)
        .http_only(true)
        .same_site(SameSite::Strict)
        .expires(OffsetDateTime::now_utc() + Duration::days(7));
    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.to_string().parse().unwrap());

    Ok((StatusCode::CREATED, headers, Json(new_user.id)))
}

pub async fn log_out(
    State(state): State<AppState>,
    auth_ctx: Extension<AuthContext>,
) -> Result<Response, Response> {
    let _ = sqlx::query!("DELETE FROM sessions WHERE id = $1", auth_ctx.session_id,)
        .execute(&state.pg_pool)
        .await
        .map_err(|err| {
            error!("{}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(SET_COOKIE, EXPIRED_EMPTY_COOKIE)],
                "Unexpected Error",
            )
                .into_response()
        })?;

    Ok((StatusCode::NO_CONTENT, [(SET_COOKIE, EXPIRED_EMPTY_COOKIE)]).into_response())
}

pub async fn log_in(
    State(state): State<AppState>,
    Json(payload): Json<LoginParams>,
) -> Result<Response, Response> {
    let mut tx = state.pg_pool.begin().await.map_err(db_err_to_response)?;

    debug!("logging in");
    let username = payload.username;
    let password = payload.password;
    let user = sqlx::query_as!(
        UserClean,
        r#"SELECT id, username FROM users WHERE username = $1 AND password = $2"#,
        username,
        password
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|err| {
        error!("Log in error, failed to get user w/ password: {:?}", err);
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    })?;

    // Create new session
    let session_id = Alphanumeric.sample_string(&mut rng(), 16);
    let cookie = Cookie::build(("session_id", session_id.clone()))
        .secure(true)
        .http_only(true)
        .same_site(SameSite::Strict)
        .expires(OffsetDateTime::now_utc() + Duration::days(7));
    // Save to DB
    // TODO: session TTL in DB same as expires in browser
    let _ = sqlx::query!(
        "INSERT INTO sessions (id, user_id) VALUES ($1, $2)",
        session_id,
        user.id
    )
    .execute(&mut *tx)
    .await
    .map_err(|err| {
        let error_response = serde_json::json!({
        "status": "error",
        "message": format!("Database error: { }", err),
        })
        .to_string();
        error!("Log in error - failed to start session: {}", error_response);

        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Unexpected error with login",
        )
            .into_response();
    })?;

    let _ = tx.commit().await.map_err(db_err_to_response)?;

    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.to_string().parse().unwrap());
    return Ok((StatusCode::OK, headers, user.id).into_response());
}
