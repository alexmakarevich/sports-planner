use std::{collections::HashMap, fmt::Write};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use strum_macros::{Display, EnumString};

use crate::{
    auth::{roles, utils::AuthContext},
    entities::user::UserClean,
    utils::api::{db_err_to_response, AppState},
};

// TODO: consider a bitmask/bit-flags
// E.g. https://github.com/Lukas3674/rust-bitmask-enum

// TODO: consider saving roles directly into the users table as a special Postgres type

// hardcoding roles, since they shouldn't be adjustable in the UI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, Display, EnumString)]
#[sqlx(type_name = "user_roles", rename_all = "snake_case")] // must match the Postgres type name
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Role {
    SuperAdmin,

    ClubAdmin,

    Coach,
    Player,
}

pub fn check_user_roles(auth_ctx: &AuthContext, role_whitelist: &[Role]) -> Result<(), Response> {
    debug!(
        "ROLE CHECK, expected {:?} - received {:?}",
        role_whitelist, auth_ctx.roles
    );
    let roles = &auth_ctx.roles;
    for r in roles {
        if role_whitelist.contains(&r) {
            debug!("ROLE CHECK SUCCEEDED");
            return Ok(());
        }
    }
    let mut printable_whitelist = String::new();

    for (i, role) in role_whitelist.iter().enumerate() {
        if i > 0 {
            printable_whitelist.push_str(", ");
        }
        write!(
            &mut printable_whitelist,
            "{}",
            role.to_string().to_lowercase()
        )
        .unwrap();
    }

    let error_text = format!(
        "Access denied. Needs one of roles: [{}]",
        printable_whitelist
    );
    error!("ROLE CHECK FAILED: {}", error_text);
    return Err((StatusCode::FORBIDDEN, error_text).into_response());
}

#[derive(FromRow, Serialize)]
pub struct SelectRoleAssignments {
    roles: Option<Vec<Role>>,
    user_id: String,
}

#[derive(FromRow, Serialize)]
pub struct SelectOwnRoleAssignment {
    roles: Option<Vec<Role>>,
    user_id: String,
}

// #[axum::debug_handler]
pub async fn list_own_role_assignments(
    State(state): State<AppState>,
    auth_ctx: Extension<AuthContext>,
) -> Result<(StatusCode, Json<Vec<Role>>), Response> {
    let role_assignment = sqlx::query_as!(
        SelectOwnRoleAssignment,
        r#"SELECT
        user_id,
        COALESCE(array_agg(role) FILTER (WHERE role IS NOT NULL), '{}') AS "roles: Vec<Role>" 
        FROM role_assignments
        WHERE user_id = $1
        GROUP BY (user_id)
        "#,
        auth_ctx.user_id
    )
    .fetch_optional(&state.pg_pool)
    .await
    .map_err(db_err_to_response)?;

    match role_assignment {
        Some(ra) => Ok((StatusCode::OK, Json(ra.roles.unwrap_or(vec![])))),
        None => Ok((StatusCode::OK, Json(vec![]))),
    }
}

#[derive(Deserialize)]
pub struct Params {
    user_id: Option<String>,
}

pub async fn list_role_assignments(
    State(state): State<AppState>,
    auth_ctx: Extension<AuthContext>,
    Query(params): Query<Params>,
) -> Result<(StatusCode, Json<HashMap<String, Vec<Role>>>), Response> {
    let _ = check_user_roles(&auth_ctx, &[Role::ClubAdmin, Role::SuperAdmin])?;
    let query = match params.user_id {
        Some(user_id) => {
            sqlx::query_as!(
                SelectRoleAssignments,
                r#"SELECT
                ra.user_id,
                COALESCE(array_agg(ra.role) FILTER (WHERE ra.role IS NOT NULL), '{}') AS "roles: Vec<Role>" 
                FROM role_assignments ra
                JOIN users u
                ON ra.user_id = u.id
                WHERE ra.user_id = $1 AND u.club_id = $2
                GROUP BY (ra.user_id)
                "#,
                user_id,
                auth_ctx.club_id
            )
            .fetch_all(&state.pg_pool)
            .await
        }
        None => {
            sqlx::query_as!(
                SelectRoleAssignments,
                r#"SELECT
                ra.user_id,
                COALESCE(array_agg(ra.role) FILTER (WHERE ra.role IS NOT NULL), '{}') AS "roles: Vec<Role>" 
                FROM role_assignments ra
                JOIN users u
                ON ra.user_id = u.id
                WHERE u.club_id = $1
                GROUP BY (ra.user_id)
                "#,
                auth_ctx.club_id
            )
            .fetch_all(&state.pg_pool)
            .await
        }
    };

    let role_assignments = query.map_err(db_err_to_response)?;

    let mut user_to_role_map: HashMap<String, Vec<Role>> = HashMap::new();

    for assignment in role_assignments {
        user_to_role_map.insert(assignment.user_id, assignment.roles.unwrap_or(vec![]));
    }

    Ok((StatusCode::CREATED, Json(user_to_role_map)))
}

#[derive(Deserialize)]
pub struct AssignRole {
    pub user_id: String,
    pub role: Role,
}

// higher roles may assign all lower roles
pub async fn assign_role(
    State(state): State<AppState>,
    auth_ctx: Extension<AuthContext>,
    Json(payload): Json<AssignRole>,
) -> Result<(StatusCode, String), Response> {
    let mut tx: sqlx::Transaction<'static, sqlx::Postgres> =
        state.pg_pool.begin().await.map_err(db_err_to_response)?;

    let _ = sqlx::query_as!(
        UserClean,
        r#"SELECT id, username FROM users WHERE club_id = $1 AND id = $2"#,
        auth_ctx.club_id,
        payload.user_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|err| {
        error!("{}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Given user not present in your club or doesn't exist",
        )
            .into_response()
    })?;

    // access checks
    let _ = match payload.role {
        Role::SuperAdmin => check_user_roles(&auth_ctx, &[Role::SuperAdmin])?,
        Role::ClubAdmin => check_user_roles(&auth_ctx, &[Role::SuperAdmin, Role::ClubAdmin])?,
        Role::Coach => {
            check_user_roles(&auth_ctx, &[Role::SuperAdmin, Role::ClubAdmin, Role::Coach])?
        }
        Role::Player => {
            check_user_roles(&auth_ctx, &[Role::SuperAdmin, Role::ClubAdmin, Role::Coach])?
        }
    };

    let new_assignment = sqlx::query!(
        r#"INSERT INTO role_assignments (user_id, role) VALUES ($1, $2) RETURNING id"#,
        payload.user_id,
        payload.role as Role
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(db_err_to_response)?;

    let _ = tx.commit().await.map_err(db_err_to_response)?;

    Ok((StatusCode::CREATED, new_assignment.id.to_string()))
}

// higher roles may assign all lower roles
pub async fn unassign_role(
    State(state): State<AppState>,
    auth_ctx: Extension<AuthContext>,
    Json(payload): Json<AssignRole>,
) -> Result<StatusCode, Response> {
    // access checks
    let _ = match payload.role {
        Role::SuperAdmin => check_user_roles(&auth_ctx, &[Role::SuperAdmin])?,
        Role::ClubAdmin => check_user_roles(&auth_ctx, &[Role::SuperAdmin, Role::ClubAdmin])?,
        Role::Coach => {
            check_user_roles(&auth_ctx, &[Role::SuperAdmin, Role::ClubAdmin, Role::Coach])?
        }
        Role::Player => {
            check_user_roles(&auth_ctx, &[Role::SuperAdmin, Role::ClubAdmin, Role::Coach])?
        }
    };

    let _ = sqlx::query!(
        r#"
        DELETE FROM role_assignments AS ra
            USING users AS u
            WHERE ra.user_id = $1
            AND ra.role = $3
            AND ra.user_id = u.id
            AND u.club_id = $2
        RETURNING ra.id
        "#,
        payload.user_id,
        auth_ctx.club_id,
        payload.role as Role
    )
    .fetch_one(&state.pg_pool)
    .await
    .map_err(db_err_to_response)?;

    Ok(StatusCode::CREATED)
}
