use axum::{
    extract::{Request, State},
    http::{header::SET_COOKIE, HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use log::{debug, error};
use rand::{
    distr::{Alphanumeric, SampleString},
    rng,
};
use serde::Deserialize;
use time::OffsetDateTime;

use crate::{auth::utils::AuthContext, entities::user::UserClean, utils::api::AppState};


#[derive(sqlx::FromRow)]
pub struct SessionModel {
    pub id: String,
    pub user_id: String,
}

#[derive(sqlx::FromRow)]
pub struct UserWithSessionModel {
    pub user_id: String,
    pub session_id: String,
    pub org_id: String,
}

#[derive(Deserialize)]
pub struct LoginParams {
    pub username: String,
    pub password: String,
}

// TODO: log out

pub async fn log_in(
    State(state): State<AppState>,
    Json(payload): Json<LoginParams>,
) -> impl IntoResponse {
    debug!("logging in");
    let username = payload.username;
    let password = payload.password;
    let query_result = sqlx::query_as!(
        UserClean,
        r#"SELECT id, username FROM users WHERE username = $1 AND password = $2"#,
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
            error!("Log in error, failed to get user w/ password: {}", error_response);

            // TODO: nicely avoid these empty headers? currently there for consistent return shape
            let headers = HeaderMap::new();
            return (
                StatusCode::UNAUTHORIZED,
                headers,
                "Unauthorized".to_string(),
            );
        }
        Ok(user) => {
            // Create new session
            let session_id = Alphanumeric.sample_string(&mut rng(), 16);
            // TODO: does the cookie have all the correct security settings by default?
            let cookie = Cookie::new("session_id", session_id.clone());

            // Save to DB
            // TODO: session TTL in DB same as expires in browser
            let query_result = sqlx::query!(
                "INSERT INTO sessions (id, user_id) VALUES ($1, $2)",
                session_id,
                user.id
            )
            .execute(&state.pg_pool)
            .await;

            match query_result {
                Ok(_) => {
                    let mut headers = HeaderMap::new();
                    headers.insert(SET_COOKIE, cookie.to_string().parse().unwrap());
                    return (StatusCode::OK, headers, "Login successful".to_string());
                }
                Err(err) => {
                    let error_response = serde_json::json!({
                    "status": "error",
                    "message": format!("Database error: { }", err),
                    })
                    .to_string();
                    error!("Log in error - failed to start session: {}", error_response);
                    // TODO: nicely avoid these empty headers? currently there for consistent return shape
                    let headers = HeaderMap::new();

                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        headers,
                        "Unexpected error with login".to_string(),
                    );
                }
            }
        }
    }

}

pub async fn cookie_auth_middleware(
    State(state): State<AppState>,
    jar: CookieJar,
    mut req: Request,
    next: Next,
) -> Result<Response, Response> {
    debug!("cookie middleware called");


    let Some(cookie) = jar.get("session_id") else {
        debug!("cookie NOT found");

        // err -> 401, not allowed,
        // this is an API middleware
        // for direct client-facing-routes, we may want to redirect to the login page (or we do it in the FE anyway)

        return Err((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()).into_response());
    };

    debug!("cookie found {}", cookie.value());

    let Ok(user_with_session) = sqlx::query_as!(
        UserWithSessionModel,
        r#"SELECT u.id as user_id, s.id as session_id, u.org_id as org_id FROM users u JOIN sessions s ON u.id = s.user_id WHERE s.id = $1"#, cookie.value()
    )
    .fetch_one(&state.pg_pool)
    .await 
    else {
        // force-expire given bad cookie
        let mut headers = HeaderMap::new();
        let mut expired_cookie = cookie.clone();
        expired_cookie.set_expires( OffsetDateTime::UNIX_EPOCH);
        headers.insert(SET_COOKIE, cookie.to_string().parse().unwrap());
        return Err((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()).into_response());
    };

    let auth_context = AuthContext{
        user_id: user_with_session.user_id,
        session_id: user_with_session.session_id,
        org_id: user_with_session.org_id
    };

    req.extensions_mut().insert(auth_context);
    let mut res = next.run(req).await;
    res.headers_mut()
        .append(SET_COOKIE, cookie.to_string().parse().unwrap());

    Ok(res)
} 

