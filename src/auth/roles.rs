use axum::{extract::State, http::StatusCode, Extension};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use strum_macros::{Display, EnumString};

use crate::{
    auth::utils::AuthContext,
    utils::api::{handle_unexpected_db_err, ApiResult, AppState},
};

// hardcoding roles, since they shouldn't be adjustable in the UI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, Display, EnumString)]
#[sqlx(type_name = "user_role")] // must match the Postgres type name
#[sqlx(rename_all = "snake_case")] // map enum variants to lowercase strings
#[strum(serialize_all = "snake_case")]

pub enum Role {
    SuperAdmin,
    OrgAdmin,
    Player,
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
