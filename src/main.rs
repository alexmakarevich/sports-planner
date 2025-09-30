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

#[derive(Clone)]
struct AppState {
    users: Arc<Mutex<Vec<User>>>,
    pg_pool: PgPool,
}

// For sqlx
#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
#[allow(non_snake_case)] // TODO: why though? it's all snaky anyway
pub struct NoteModel {
    pub id: String,
    pub title: String,
    pub content: String,
    pub is_published: i8, // BOOLEAN in MySQL is TINYINT(1) so we can use i8 to retrieve the record and later we can parse to Boolean
    pub created_at: Option<chrono::NaiveDateTime>,
    // pub updated_at: Option<sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>>,
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
        .route("/in-memory/users", get(list_users))
        .route("/notes", get(list_notes))
        .route("/in-memory/create-user", post(create_user))
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
    State(user_state): State<AppState>,
    Json(payload): Json<CreateUser>,
) -> StatusCode {
    let new_user = User {
        id: 1337,
        username: payload.username,
    };

    let mut users = user_state.users.lock().await;

    users.push(new_user);

    println!("Users: {:?}", *users);

    StatusCode::CREATED
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

async fn list_users(State(user_state): State<AppState>) -> (StatusCode, Json<Vec<User>>) {
    let users = user_state.users.lock().await;
    (StatusCode::OK, Json(users.to_vec()))
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

// the output to our `create_user` handler
#[derive(Serialize, Debug, Clone)]
struct User {
    id: u64,
    username: String,
}
