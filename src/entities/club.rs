use crate::{
    auth::utils::AuthContext,
    utils::api::{handle_unexpected_db_err, AppState, EmptyApiResult},
};
use axum::{extract::State, http::StatusCode, Extension};
use log::error;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use sqlx::PgTransaction;
use tokio::time::{sleep, Duration};

const ID_LEN: usize = 6;
const MAX_RETRIES: usize = 5;
const RETRY_BACKOFF_MS: u64 = 50;

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
#[allow(non_snake_case)]
pub struct ClubModel {
    pub id: String,
    pub title: String,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

// async fn create_club_bad(
//     mut exec: PgTransaction<'_>,
//     mut title: String,
// ) -> Result<String, sqlx::Error> {
//     let mut attempt = 0;

//     loop {
//         let id = nanoid!(ID_LEN);

//         let res = sqlx::query_as!(
//             JustId,
//             r#"INSERT INTO clubs (id, title) VALUES ($1, $2) RETURNING id"#,
//             id,
//             title,
//         )
//         .fetch_one(&mut exec)
//         .await;

//         match res {
//             Ok(row) => return Ok(row.id),
//             Err(raw_err) => {
//                 if let sqlx::Error::Database(db_err) = &raw_err {
//                     if db_err.is_unique_violation() {
//                         if attempt < MAX_RETRIES {
//                             attempt += 1;
//                             sleep(Duration::from_millis(RETRY_BACKOFF_MS)).await;
//                             continue;
//                         } else {
//                             error!(
//                                 "Failed to create a new club with a unique ID after {} retries",
//                                 MAX_RETRIES
//                             );
//                             return Err(raw_err);
//                         }
//                     }
//                 };
//                 return Err(raw_err);
//             }
//         }
//     }
// }

pub async fn create_club(tx: &mut PgTransaction<'_>, title: &str) -> Result<String, sqlx::Error> {
    let mut retries = 0;

    loop {
        let id = nanoid!(ID_LEN);

        // FYI: scalar is used, because query_as miserably fails in the retry scenario

        match sqlx::query_scalar::<_, String>(
            "INSERT INTO clubs (id, title) VALUES ($1, $2) RETURNING id",
        )
        .bind(id)
        .bind(title)
        .fetch_one(&mut **tx)
        .await
        {
            Ok(id) => return Ok(id),
            Err(e) => {
                error!("{}", e);
                if e.as_database_error()
                    .map(|e| e.code().as_deref() == Some("23505"))
                    .unwrap_or(false)
                {
                    retries += 1;
                    if retries >= MAX_RETRIES {
                        return Err(e);
                    }
                    sleep(Duration::from_millis(RETRY_BACKOFF_MS)).await;
                    // Continue to retry
                } else {
                    return Err(e);
                }
            }
        }
    }
}

// SAMPLE CODE BY KLAUDIUS
async fn retry_insert(tx: &mut PgTransaction<'_>, title: &str) -> Result<(), sqlx::Error> {
    let mut retries = 0;
    let max_retries = 5;

    loop {
        match sqlx::query("INSERT INTO table_name (title) VALUES ($1)")
            .bind(title)
            .execute(&mut **tx)
            .await
        {
            Ok(_) => return Ok(()),
            Err(e) => {
                if e.as_database_error()
                    .map(|e| e.code().as_deref() == Some("23505"))
                    .unwrap_or(false)
                {
                    retries += 1;
                    if retries >= max_retries {
                        return Err(e);
                    }
                    // Continue to retry
                } else {
                    return Err(e);
                }
            }
        }
    }
}

// TODO: modify club
// TODO: more granular checks and readable errors
pub async fn delete_own_club(
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

    let _ = sqlx::query!(r#"DELETE FROM clubs WHERE id = $1"#, auth_ctx.club_id)
        .execute(&mut *tx)
        .await
        .map_err(handle_unexpected_db_err)?;

    let _ = tx.commit().await.map_err(handle_unexpected_db_err)?;

    Ok(StatusCode::NO_CONTENT)
}
