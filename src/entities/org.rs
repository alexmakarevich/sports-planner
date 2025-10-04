use axum::{extract::State, http::StatusCode, Extension};
use serde::{Deserialize, Serialize};

use crate::{
    auth::utils::AuthContext,
    utils::api::{handle_unexpected_db_err, AppState, EmptyApiResult},
};

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
#[allow(non_snake_case)]
pub struct OrgModel {
    pub id: String,
    pub title: String,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

// TODO: modify org
// TODO: more granular checks and readable errors
pub async fn delete_own_org(
    State(state): State<AppState>,
    auth_ctx: Extension<AuthContext>,
) -> EmptyApiResult {
    let mut tx = state
        .pg_pool
        .begin()
        .await
        .map_err(handle_unexpected_db_err)?;

    let _ = sqlx::query!(r#"DELETE FROM users WHERE id = $1"#, auth_ctx.user_id)
        .execute(&mut *tx)
        .await
        .map_err(handle_unexpected_db_err)?;

    let _ = sqlx::query!(r#"DELETE FROM orgs WHERE id = $1"#, auth_ctx.org_id)
        .execute(&mut *tx)
        .await
        .map_err(handle_unexpected_db_err)?;

    // if query_result.rows_affected() == 0 {
    //     Err((
    //         StatusCode::NOT_ACCEPTABLE,
    //         "Org with given ID does not exist - possibly already deleted".to_string(),
    //     ))
    // };

    let _ = tx.commit().await.map_err(handle_unexpected_db_err)?;

    Ok(StatusCode::NO_CONTENT)
}
