use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Response,
    Extension,
};
use rand::{
    distr::{Alphanumeric, SampleString},
    rng,
};

use crate::{
    auth::{
        roles::{check_user_roles, Role},
        utils::AuthContext,
    },
    utils::api::{db_err_to_response, AppState},
};

pub async fn create_service_invite(
    State(state): State<AppState>,
    auth_ctx: Extension<AuthContext>,
) -> Result<String, Response> {
    let _ = check_user_roles(&auth_ctx, &[Role::ClubAdmin, Role::SuperAdmin])?;
    let id = Alphanumeric.sample_string(&mut rng(), 16);

    let result = sqlx::query!(
        r#"INSERT INTO service_invites (id, club_id) VALUES ($1, $2) RETURNING id"#,
        id,
        auth_ctx.club_id
    )
    .fetch_one(&state.pg_pool)
    .await
    .map_err(db_err_to_response)?;

    Ok(result.id)
}

pub async fn delete_service_invite_by_id(
    State(state): State<AppState>,
    auth_ctx: Extension<AuthContext>,
    Path(id): Path<String>,
) -> Result<StatusCode, Response> {
    let _ = check_user_roles(&auth_ctx, &[Role::ClubAdmin, Role::SuperAdmin])?;
    let _ = sqlx::query!(
        r#"DELETE FROM service_invites WHERE club_id = $1 AND id = $2"#,
        &auth_ctx.club_id,
        id
    )
    .execute(&state.pg_pool)
    .await
    .map_err(db_err_to_response)?;

    Ok(StatusCode::NO_CONTENT)
}
