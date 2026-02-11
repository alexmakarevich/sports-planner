// src/entities/team.rs
//! Team entity – a concrete team inside an club
//! (e.g. “men's senior football team”)

use log::debug;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::{
    auth::{
        roles::{check_user_roles, Role},
        utils::AuthContext,
    },
    utils::api::db_err_to_response,
    AppState, JustId,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Extension, Json, Router,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
#[allow(non_snake_case)]
pub struct TeamModel {
    pub id: String,
    /// The club that owns this team
    pub club_id: String,
    /// Human‑readable name
    pub name: String,
    /// Optional short slug – e.g. "MU18, WO45"
    pub slug: String,

    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

/// Core representation of a *team* used across the API.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Team {
    pub id: String,
    pub club_id: String,
    pub name: String,
    pub slug: String,
}

pub fn team_router<S>(state: AppState) -> Router<S> {
    Router::new()
        .route("/create", post(create_team))
        .route("/get/{id}", get(get_team))
        .route("/list", get(list_teams))
        .route("/update/{id}", put(update_team))
        .route("/delete-by-id/{id}", delete(delete_team))
        .with_state(state.clone())
}

/// ---------- CREATE ----------------------------------------------------------
#[derive(Debug, Clone, Deserialize)]
pub struct CreateTeamPayload {
    pub name: String,
    pub slug: String,
}

pub async fn create_team(
    State(state): State<AppState>,
    auth_ctx: Extension<AuthContext>,
    Json(payload): Json<CreateTeamPayload>,
) -> Result<(StatusCode, Json<String>), Response> {
    // clubAdmin or SuperAdmin can create a team
    check_user_roles(&auth_ctx, &[Role::SuperAdmin, Role::ClubAdmin])?;

    let new_id = sqlx::query!(
        r#"INSERT INTO teams (id, club_id, name, slug) 
           VALUES ($1, $2, $3, $4) 
           RETURNING id"#,
        uuid::Uuid::new_v4().to_string(),
        auth_ctx.club_id,
        payload.name,
        payload.slug
    )
    .fetch_one(&state.pg_pool)
    .await
    .map_err(db_err_to_response)?;

    Ok((StatusCode::CREATED, Json(new_id.id)))
}

/// ---------- READ ALL --------------------------------------------------------
pub async fn list_teams(
    State(state): State<AppState>,
    auth_ctx: Extension<AuthContext>,
) -> Result<(StatusCode, Json<Vec<Team>>), Response> {
    let teams = sqlx::query_as!(
        Team,
        r#"SELECT id, club_id, name, slug 
           FROM teams 
           WHERE club_id = $1"#,
        auth_ctx.club_id
    )
    .fetch_all(&state.pg_pool)
    .await
    .map_err(db_err_to_response)?;

    Ok((StatusCode::OK, Json(teams)))
}

/// ---------- READ ONE --------------------------------------------------------
pub async fn get_team(
    State(state): State<AppState>,
    auth_ctx: Extension<AuthContext>,
    Path(team_id): Path<String>,
) -> Result<(StatusCode, Json<Team>), Response> {
    let team = sqlx::query_as!(
        Team,
        r#"SELECT id, club_id, name, slug 
           FROM teams 
           WHERE id = $1 AND club_id = $2"#,
        team_id,
        auth_ctx.club_id
    )
    .fetch_one(&state.pg_pool)
    .await
    .map_err(db_err_to_response)?;

    Ok((StatusCode::OK, Json(team)))
}

/// ---------- UPDATE ---------------------------------------------------------
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTeamPayload {
    pub name: Option<String>,
    pub slug: Option<String>,
}

pub async fn update_team(
    State(state): State<AppState>,
    auth_ctx: Extension<AuthContext>,
    Path(team_id): Path<String>,
    Json(payload): Json<UpdateTeamPayload>,
) -> Result<(StatusCode, Json<Team>), Response> {
    // Only ClubAdmin / SuperAdmin may change team data
    check_user_roles(&auth_ctx, &[Role::SuperAdmin, Role::ClubAdmin])?;

    // Update name & slug (short_name) only if provided
    let updated = sqlx::query_as!(
        Team,
        r#"
        UPDATE teams
        SET name = COALESCE($1, name),
            slug = COALESCE($2, slug)
        WHERE id = $3
          AND club_id = $4
        RETURNING id, club_id, name, slug
        "#,
        payload.name.as_deref(), // $1
        payload.slug.as_deref(), // $2
        team_id,                 // $3
        auth_ctx.club_id         // $4
    )
    .fetch_one(&state.pg_pool)
    .await
    .map_err(db_err_to_response)?;

    Ok((StatusCode::OK, Json(updated)))
}
/// ---------- DELETE ---------------------------------------------------------
pub async fn delete_team(
    State(state): State<AppState>,
    auth_ctx: Extension<AuthContext>,
    Path(team_id): Path<String>,
) -> Result<StatusCode, Response> {
    check_user_roles(&auth_ctx, &[Role::ClubAdmin, Role::SuperAdmin])?;

    sqlx::query!(
        r#"DELETE FROM teams WHERE id = $1 AND club_id = $2"#,
        team_id,
        auth_ctx.club_id
    )
    .execute(&state.pg_pool)
    .await
    .map_err(db_err_to_response)?;

    Ok(StatusCode::NO_CONTENT)
}
