use std::iter;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use sqlx::Type;
use strum_macros::{Display, EnumString};

use crate::{
    auth::{
        roles::{check_user_roles, Role},
        utils::AuthContext,
    },
    utils::api::db_err_to_response,
    AppState, JustId,
};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};

// TODO: rewrite game to be from generic perspective - not from your club's
// I.e. have defined host_id and and guest_id, referring to teams, most likely - from different clubs.
// Potentially, even more generic - team1, team2, then a field defining who the host is (1,2 or "neutral/other").
// This would remove the need for location_kind.
// Would possibly need to have both - service may be used by whole leagues and by select teams.

// user from DB wihtout security and unnecessary util fields
#[derive(Deserialize)]
pub struct CreateGamePayload {
    pub team_id: String,
    pub opponent: String,
    pub start: DateTime<Utc>,
    pub end: Option<DateTime<Utc>>,
    pub location: String,
    pub location_kind: LocationKind, // home|away|other
    pub invited_roles: Vec<Role>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, Display, EnumString)]
#[sqlx(type_name = "location_kind", rename_all = "snake_case")] // must match the Postgres type name
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum LocationKind {
    Home,
    Away,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, Display, EnumString)]
#[sqlx(type_name = "game_invite_response", rename_all = "snake_case")] // must match the Postgres type name
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum InviteResponse {
    Pending,
    Accepted,
    Declined,
    Unsure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, Display, EnumString)]
#[sqlx(type_name = "game_invite_response", rename_all = "snake_case")] // must match the Postgres type name
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
/** A user may not reset response back to "pending" */
pub enum InviteResponseFromUser {
    // Pending,
    Accepted,
    Declined,
    Unsure,
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, Display, EnumString)]
// #[sqlx(type_name = "game_invite_response", rename_all = "snake_case")] // must match the Postgres type name
// #[serde(rename_all = "snake_case")]
// #[strum(serialize_all = "snake_case")]
// pub enum InviteStatus {
//     Pending,
//     Accepted,
//     Declined,
//     Unsure,
//     Uninvited,
// }

pub async fn create_game(
    State(state): State<AppState>,
    auth_ctx: Extension<AuthContext>,
    Json(payload): Json<CreateGamePayload>,
) -> Result<Response, Response> {
    // Only admins/coaches can create games – keep the same guard
    let _ = check_user_roles(&auth_ctx, &[Role::OrgAdmin, Role::SuperAdmin, Role::Coach])?;

    // Optional: verify that `payload.team_id` actually belongs to the authenticated org
    sqlx::query!(
        "SELECT 1 as ok FROM teams WHERE id = $1 AND org_id = $2",
        payload.team_id,
        auth_ctx.org_id
    )
    .fetch_one(&state.pg_pool)
    .await
    .map_err(db_err_to_response)?;

    let mut tx = state.pg_pool.begin().await.map_err(db_err_to_response)?;

    let end_conv = payload.end.map(|e| e.naive_utc());

    let new_event = sqlx::query!(
        r#"INSERT INTO events (start_time, stop_time) VALUES ($1, $2) RETURNING id"#,
        payload.start.naive_utc(),
        end_conv,
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(db_err_to_response)?;

    let new_game = sqlx::query!(
        r#"INSERT INTO games
                (opponent, location, location_kind, event_id, invited_roles, team_id)
            VALUES ($1, $2, $3, $4, $5, $6) RETURNING id"#,
        payload.opponent,
        payload.location,
        payload.location_kind as LocationKind,
        new_event.id,
        payload.invited_roles.clone() as Vec<Role>,
        payload.team_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(db_err_to_response)?;

    let users_to_invite = sqlx::query_as!(
        JustId,
        r#"SELECT u.id FROM users u
        JOIN role_assignments ra ON ra.user_id = u.id
        WHERE u.org_id = $1
        AND ra.role = ANY($2)
        ORDER by u.username"#,
        auth_ctx.org_id,
        payload.invited_roles as Vec<Role>
    )
    .fetch_all(&state.pg_pool)
    .await
    .map_err(db_err_to_response)?;

    // ensuring, one user only gets one invite
    let mut user_ids: Vec<String> = users_to_invite.iter().map(|u| u.id.clone()).collect();
    user_ids.sort_unstable();
    user_ids.dedup();
    let game_ids: Vec<String> = iter::repeat(new_game.id.clone())
        .take(user_ids.len())
        .collect();
    let statuses: Vec<InviteResponse> = iter::repeat(InviteResponse::Pending)
        .take(user_ids.len())
        .collect();

    sqlx::query!(
        r#"
        INSERT INTO game_invites (user_id, game_id, response)
        SELECT * FROM UNNEST($1::text[], $2::text[], $3::game_invite_response[])
    "#,
        &user_ids,
        &game_ids,
        &statuses as &Vec<InviteResponse>
    )
    .execute(&mut *tx)
    .await
    .map_err(db_err_to_response)?;

    let _ = tx.commit().await.map_err(db_err_to_response)?;

    Ok((StatusCode::CREATED, Json(&new_game.id)).into_response())
}
