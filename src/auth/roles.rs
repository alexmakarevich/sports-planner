use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension,
};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use strum_macros::{Display, EnumString};

use crate::{
    auth::utils::AuthContext,
    utils::api::{handle_unexpected_db_err, ApiResult, AppState},
};

// TODO: consider a bitmask/bit-flags
// E.g. https://github.com/Lukas3674/rust-bitmask-enum

// TODO: consider saving roles directly into the users table as a special Postgres type

// hardcoding roles, since they shouldn't be adjustable in the UI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, Display, EnumString)]
#[sqlx(type_name = "user_role")] // must match the Postgres type name
#[sqlx(rename_all = "snake_case")] // map enum variants to lowercase strings
#[strum(serialize_all = "snake_case")]
pub enum Role {
    SuperAdmin,

    OrgAdmin,

    Coach,
    Player,
}

pub fn check_user_roles(auth_ctx: &AuthContext, role_whitelist: &[Role]) -> Result<(), Response> {
    debug!(
        "ROLE CHECK, expected {:?} - received {:?}",
        role_whitelist, auth_ctx.roles
    );
    let roles = &auth_ctx.roles;
    for ele in roles {
        if role_whitelist.contains(&ele) {
            debug!("ROLE CHECK SUCCEEDED");
            return Ok(());
        }
    }
    // TODO: is the error text actually getting sent back?
    let error_text = format!("Access denied. Needs one of roles: {:?}", role_whitelist);
    error!("ROLE CHECK FAILED: {}", error_text);
    return Err((StatusCode::UNAUTHORIZED, error_text).into_response());
}

#[derive(FromRow)]
pub struct RoleAssignment {
    role: Role,
}

pub async fn list_role_assignments(
    State(state): State<AppState>,
    auth_ctx: Extension<AuthContext>,
) -> ApiResult<String> {
    let user_with_session = sqlx::query_as!(
        RoleAssignment,
        r#"SELECT 
        role as "role: Role" 
        FROM role_assignments"#
    )
    .fetch_one(&state.pg_pool)
    .await
    .map_err(handle_unexpected_db_err);

    Ok((StatusCode::CREATED, axum::Json("ok".to_string())))
}

// TODO: pub async fn assign_role
