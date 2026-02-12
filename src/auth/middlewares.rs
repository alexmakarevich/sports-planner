use axum::{
    extract::{Request, State},
    http::{header::SET_COOKIE, HeaderMap, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::CookieJar;
use log::{debug, error};
use sqlx::prelude::FromRow;
use time::{Duration, OffsetDateTime};

use crate::{
    auth::{roles::Role, utils::AuthContext, utils::EXPIRED_EMPTY_COOKIE},
    AppState,
};

#[derive(FromRow)]
pub struct UserWithSessionModel {
    pub user_id: String,
    pub session_id: String,
    pub club_id: String,
    pub roles: Option<Vec<Role>>,
}

pub async fn cookie_auth_middleware(
    State(state): State<AppState>,
    jar: CookieJar,
    mut req: Request,
    next: Next,
) -> Result<Response, (Response)> {
    debug!("cookie middleware called");

    let Some(cookie) = jar.get("session_id") else {
        debug!("cookie NOT found");

        // err -> 401, not allowed,
        // this is an API middleware
        // for direct client-facing-routes, we may want to redirect to the login page (or we do it in the FE anyway)

        return Err((StatusCode::UNAUTHORIZED, "Not logged in").into_response());
    };

    debug!("cookie found {}", cookie.value());

    let user_with_session = sqlx::query_as!(
        UserWithSessionModel,
        r#"
        SELECT u.id as user_id, s.id as session_id, u.club_id as club_id,
        COALESCE(array_agg(ra.role) FILTER (WHERE ra.role IS NOT NULL), '{}') AS "roles: Vec<Role>" 
        FROM users u
        LEFT JOIN role_assignments ra on ra.user_id = u.id
        JOIN sessions s ON u.id = s.user_id WHERE s.id = $1
        GROUP BY (u.id, s.id)
        ;
        "#,
        cookie.value()
    )
    .fetch_one(&state.pg_pool)
    .await
    // .map_err(handle_unexpected_db_err)?;
    .map_err(|err| {
        error!("Error in user cookie middleware: {}", err);
        // force-expire given bad cookie

        return (
            StatusCode::UNAUTHORIZED,
            [(SET_COOKIE, EXPIRED_EMPTY_COOKIE)],
            "Unauthorized",
        )
            .into_response();
    })?;

    let roles = user_with_session.roles.unwrap_or(vec![]);

    let auth_context = AuthContext {
        roles: roles,
        user_id: user_with_session.user_id,
        club_id: user_with_session.club_id,
        session_id: user_with_session.session_id,
    };

    req.extensions_mut().insert(auth_context);
    let res = next.run(req).await;

    Ok(res)
}

pub async fn admin_cookie_auth_middleware(
    State(state): State<AppState>,
    jar: CookieJar,
    mut req: Request,
    next: Next,
) -> Result<Response, (Response)> {
    debug!("cookie middleware called");

    let Some(cookie) = jar.get("session_id") else {
        debug!("cookie NOT found");

        // err -> 401, not allowed,
        // this is an API middleware
        // for direct client-facing-routes, we may want to redirect to the login page (or we do it in the FE anyway)

        return Err((StatusCode::UNAUTHORIZED, "Not logged in").into_response());
    };

    debug!("cookie found {}", cookie.value());

    let user_with_session = sqlx::query_as!(
        UserWithSessionModel,
        r#"
        SELECT u.id as user_id, s.id as session_id, u.club_id as club_id,
        COALESCE(array_agg(ra.role) FILTER (WHERE ra.role IS NOT NULL), '{}') AS "roles: Vec<Role>" 
        FROM users u
        LEFT JOIN role_assignments ra on ra.user_id = u.id
        LEFT JOIN global_role_assignments gra ON gra.user_id = u.id AND gra.role = 'admin'
        JOIN sessions s ON u.id = s.user_id WHERE s.id = $1
        GROUP BY (u.id, s.id)
        ;
        "#,
        cookie.value()
    )
    .fetch_one(&state.pg_pool)
    .await
    // .map_err(handle_unexpected_db_err)?;
    .map_err(|err| {
        error!("Error in admin cookie middleware: {}", err);
        // force-expire given bad cookie

        return (
            StatusCode::UNAUTHORIZED,
            [(SET_COOKIE, EXPIRED_EMPTY_COOKIE)],
            "Unauthorized",
        )
            .into_response();
    })?;

    let roles = user_with_session.roles.unwrap_or(vec![]);

    let auth_context = AuthContext {
        roles: roles,
        user_id: user_with_session.user_id,
        club_id: user_with_session.club_id,
        session_id: user_with_session.session_id,
    };

    req.extensions_mut().insert(auth_context);
    let res = next.run(req).await;

    Ok(res)
}
