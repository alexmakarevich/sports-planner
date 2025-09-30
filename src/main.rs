use axum::{
    extract::{Request, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router, ServiceExt,
};
use dotenv::dotenv;
use log::{error, info};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::normalize_path::NormalizePathLayer;
use tower_layer::Layer;

mod entities;
use entities::user::UserClean;

use crate::entities::{
    note::NoteModel,
    user::{CreateUser, UserModel},
};

#[derive(Clone)]
struct AppState {
    users: Arc<Mutex<Vec<UserClean>>>,
    pg_pool: PgPool,
}

// const POSTGRES_URL: &str = "postgres://postgres:password@localhost:15432/postgres";

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // needed so that logs are actually printed to the console
    env_logger::init();

    dotenv().ok();
    let postgres_url = dotenv::var("DATABASE_URL").expect("DATABASE_URL is not configured");

    let pool = match PgPoolOptions::new()
        .max_connections(3)
        .connect(&postgres_url)
        .await
    {
        Ok(pool) => {
            info!("Connected to Postgres");
            pool
        }
        Err(err) => {
            error!("Failed to connect to Postgres: {:?}", err);
            std::process::exit(1);
        }
    };

    let user_state = AppState {
        users: Arc::new(Mutex::new(vec![])),
        pg_pool: pool,
    };

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // .route("/new-state", get(handler))
        .route("/in-memory/users", get(list_users_inmem))
        .route("/notes", get(list_notes))
        .route("/users/list", get(list_users))
        .route("/users/create", post(create_user))
        // .route("/in-memory/create-user", post(create_user))
        // .with_state(app_state)
        .with_state(user_state);
    // .with_state(state)
    // `POST /users` goes to `create_user`
    // .route("/users", post(create_user))

    // two lines below an their respective improts are necessary to remove trailing slashes from URLs (otherwise routes with and without them are treated as separate)
    // see https://github.com/tokio-rs/axum/issues/2659
    let app = NormalizePathLayer::trim_trailing_slash().layer(app);
    let app = ServiceExt::<Request>::into_make_service(app);

    // run our app with hyper, listening globally
    info!("running rust server on localhost:3333");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3333").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    info!("base route called");

    "Hello, World!"
}

async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUser>,
) -> ApiResult<String> {
    let username = payload.username;
    let password = payload.password;

    let query_result = sqlx::query!(
        r#"INSERT INTO users (username, password) VALUES ($1, $2) RETURNING id"#,
        username,
        password
    )
    .fetch_one(&state.pg_pool)
    .await;

    match query_result {
        Err(e) => {
            let error_response = serde_json::json!({
            "status": "error",
            "message": format!("Database error: { }", e),
            })
            .to_string();
            Err((StatusCode::INTERNAL_SERVER_ERROR, error_response))
        }
        Ok(record) => Ok((StatusCode::CREATED, Json(record.id))),
    }
}

type ApiResult<T> = Result<(StatusCode, Json<T>), (StatusCode, String)>;

async fn list_notes(State(state): State<AppState>) -> ApiResult<Vec<NoteModel>> {
    let query_result = sqlx::query_as!(NoteModel, r#"SELECT * FROM notes ORDER by id"#)
        .fetch_all(&state.pg_pool)
        .await;

    match query_result {
        Err(e) => {
            let error_response = serde_json::json!({
            "status": "error",
            "message": format!("Database error: { }", e),
            })
            .to_string();
            Err((StatusCode::INTERNAL_SERVER_ERROR, error_response))
        }
        Ok(notes) => Ok((StatusCode::OK, Json(notes))),
    }
}

async fn list_users(State(state): State<AppState>) -> ApiResult<Vec<UserClean>> {
    let query_result = sqlx::query_as!(UserClean, r#"SELECT id, username FROM users ORDER by id"#)
        .fetch_all(&state.pg_pool)
        .await;

    match query_result {
        Err(e) => {
            let error_response = serde_json::json!({
            "status": "error",
            "message": format!("Database error: { }", e),
            })
            .to_string();
            Err((StatusCode::INTERNAL_SERVER_ERROR, error_response))
        }
        Ok(users) => Ok((StatusCode::OK, Json(users))),
    }
}

async fn list_users_inmem(
    State(user_state): State<AppState>,
) -> (StatusCode, Json<Vec<UserClean>>) {
    let users = user_state.users.lock().await;
    (StatusCode::OK, Json(users.to_vec()))
}
