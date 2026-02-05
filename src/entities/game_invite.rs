use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    auth::utils::AuthContext,
    entities::game::{InviteResponse, InviteResponseFromUser},
    utils::api::{db_err_to_response, AppState},
};

#[derive(Serialize)]
struct SelectInvites {
    invite_id: String,
    game_id: String,
    opponent: String,
    response: InviteResponse,
}

pub async fn list_own_game_invites(
    State(state): State<AppState>,
    auth_ctx: Extension<AuthContext>,
) -> Result<Response, Response> {
    let invites = sqlx::query_as!(
        SelectInvites,
        r#"
        SELECT g.id as game_id, g.opponent as opponent, i.response AS "response: InviteResponse", i.id as invite_id
        FROM game_invites i 
        JOIN games g ON g.id = i.game_id
        WHERE user_id = $1
        "#,
        auth_ctx.user_id
    )
    .fetch_all(&state.pg_pool)
    .await
    .map_err(db_err_to_response)?;

    Ok((StatusCode::OK, Json(invites)).into_response())
}

#[derive(Serialize)]
struct SelectInvitesToGame {
    user_id: String,
    invite_id: String,
    username: String,
    response: InviteResponse,
}

pub async fn list_invites_to_game(
    State(state): State<AppState>,
    auth_ctx: Extension<AuthContext>,
    Path(game_id): Path<String>,
) -> Result<Response, Response> {
    let invites = sqlx::query_as!(
        SelectInvitesToGame,
        r#"
        SELECT
            u.id       as user_id,
            u.username as username,
            i.response AS "response: InviteResponse",
            i.id       as invite_id
        FROM game_invites i
        JOIN users u ON u.id = i.user_id
        JOIN games g ON g.id = i.game_id
        JOIN teams t ON t.id = g.team_id  
        WHERE i.game_id = $1
          AND t.org_id = $2
        "#,
        game_id,
        auth_ctx.org_id
    )
    .fetch_all(&state.pg_pool)
    .await
    .map_err(db_err_to_response)?;

    Ok((StatusCode::OK, Json(invites)).into_response())
}

#[derive(Deserialize)]
pub struct AnswerInviteToGame {
    invite_id: String,
    response: InviteResponseFromUser,
}

pub async fn answer_invite_to_game(
    State(state): State<AppState>,
    auth_ctx: Extension<AuthContext>,
    Json(payload): Json<AnswerInviteToGame>,
) -> Result<Response, Response> {
    let _ = sqlx::query!(
        r#"
        UPDATE game_invites AS i
        SET response = $1
        FROM users AS u
        WHERE u.id = i.user_id
        AND i.id = $2
        AND u.id = $3
        "#,
        payload.response as InviteResponseFromUser,
        payload.invite_id,
        auth_ctx.user_id
    )
    .execute(&state.pg_pool)
    .await
    .map_err(db_err_to_response)?;

    Ok((StatusCode::OK).into_response())
}
