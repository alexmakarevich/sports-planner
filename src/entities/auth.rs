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
use uuid::Uuid;

use crate::{entities::user::UserClean, utils::api::AppState};

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
            error!("Log in error: {}", error_response);

            // TODO: nicely avoid these empty headers? currently there for consistent return shape
            let headers = HeaderMap::new();
            return (StatusCode::UNAUTHORIZED, headers, "It works!".to_string());
        }
        Ok(user) => {
            // Create new session
            let session_id = Alphanumeric.sample_string(&mut rng(), 16);
            // TODO: does the cookie have all the correct security settings by default?
            let cookie = Cookie::new("session_id", session_id.clone());

            // Save to DB
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
                    error!("Log in error: {}", error_response);
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

    // let query_result = sqlx::query_as!(
    //     SessionModel,
    //     r#"SELECT id, username FROM session WHERE id = $1"#,
    // )
    // .fetch_one(&state.pg_pool)
    // .await;
}

pub async fn dumb_cookie_middleware(
    State(state): State<AppState>,
    jar: CookieJar,
    mut req: Request,
    next: Next,
) -> (CookieJar, Response) {
    debug!("cookie middleware called");

    if let Some(cookie) = jar.get("session_id") {
        debug!("cookie found {}", cookie.value());
        let mut res = next.run(req).await;
        res.headers_mut()
            .append(SET_COOKIE, cookie.to_string().parse().unwrap());

        return (jar, res);
    } else {
        debug!("cookie NOT found");

        // Create new session
        // TODO: replace with better randomized value
        let session_id = Uuid::new_v4().to_string();
        // TODO: does the cookie have all the corrc tsecurity sesttings by default?
        let cookie = Cookie::new("session_id", session_id.clone());

        // Save to DB
        let _ = sqlx::query!(
            "INSERT INTO sessions (id, user_id) VALUES ($1, '1aed463a-164d-4067-a0e7-da1daf44a218')",
            session_id
        )
        .execute(&state.pg_pool)
        .await;
        // jar.add(cookie);
        req.extensions_mut().insert(cookie.clone());
        let mut res = next.run(req).await;
        res.headers_mut()
            .append(SET_COOKIE, cookie.to_string().parse().unwrap());
        return (jar, res);
    }
}
