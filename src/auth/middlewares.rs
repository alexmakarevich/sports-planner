use axum::{
    extract::{Request, State},
    http::{header::SET_COOKIE, HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    
};
use axum_extra::extract::cookie::{ CookieJar};
use log::{debug};
use time::OffsetDateTime;

use crate::{auth::utils::AuthContext, AppState};




#[derive(sqlx::FromRow)]
pub struct UserWithSessionModel {
    pub user_id: String,
    pub session_id: String,
    pub org_id: String,
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
