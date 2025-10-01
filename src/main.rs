use axum::{
    extract::{Path, Request, State},
    http::{header::SET_COOKIE, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router, ServiceExt,
};
use dotenv::dotenv;
use log::{debug, error, info};
use serde::Deserialize;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::normalize_path::NormalizePathLayer;
use tower_layer::Layer;
use uuid::Uuid;

mod entities;
use entities::user::UserClean;

use crate::entities::{note::NoteModel, user::CreateUser};

use axum_extra::{
    extract::cookie::{self, Cookie, CookieJar},
    TypedHeader,
};

#[derive(Clone)]
struct AppState {
    users: Arc<Mutex<Vec<UserClean>>>,
    pg_pool: PgPool,
}

type ApiResult<T> = Result<(StatusCode, Json<T>), (StatusCode, String)>;
type EmptyApiResult = Result<StatusCode, (StatusCode, String)>;

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

    let state = AppState {
        users: Arc::new(Mutex::new(vec![])),
        pg_pool: pool,
    };

    // build our application with a route
    let app = Router::new()
        .route("/", get(root))
        // `GET /` goes to `root`
        // .route("/new-state", get(handler))
        .route("/in-memory/users", get(list_users_inmem))
        .route("/notes", get(list_notes))
        .route("/users/list", get(list_users))
        .route("/users/create", post(create_user))
        .route("/users/delete-by-id/{id}", delete(delete_user_by_id))
        // .route("/in-memory/create-user", post(create_user))
        // .with_state(app_state)
        .layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn(logging_middleware))
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    dumb_cookie_middleware,
                )),
        )
        .with_state(state);

    // .with_state(state)
    // `POST /users` goes to `create_user`
    // .route("/users", post(create_user))

    // two lines below an their respective imports are necessary to remove trailing slashes from URLs (otherwise routes with and without them are treated as separate)
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

async fn logging_middleware(req: Request, next: Next) -> (Response) {
    info!(
        "request received, path: {}",
        req.uri().path_and_query().unwrap()
    );

    return next.run(req).await;
}

async fn dumb_cookie_middleware(
    State(state): State<AppState>,
    jar: CookieJar,
    mut req: Request,
    next: Next,
) -> (CookieJar, Response) {
    debug!("middleware called");

    if let Some(cookie) = jar.get("session_id") {
        debug!("cookie found {}", cookie.value());
        let mut res = next.run(req).await;
        res.headers_mut()
            .append(SET_COOKIE, cookie.to_string().parse().unwrap());

        return (jar, res);
    } else {
        debug!("cookie NOT found");

        // Create new session
        // TODO: replace with better randomized value
        let session_id = Uuid::new_v4().to_string();
        // TODO: does the cookie have all the corrc tsecurity sesttings by default?
        let cookie = Cookie::new("session_id", session_id.clone());

        // Save to DB
        let _ = sqlx::query!(
            "INSERT INTO sessions (id, user_id) VALUES ($1, '1aed463a-164d-4067-a0e7-da1daf44a218')",
            session_id
        )
        .execute(&state.pg_pool)
        .await;
        // jar.add(cookie);
        req.extensions_mut().insert(cookie.clone());
        let mut res = next.run(req).await;
        res.headers_mut()
            .append(SET_COOKIE, cookie.to_string().parse().unwrap());
        return (jar, res);
    }
}

// // Simple middleware function to authenticate requests
// async fn auth_middleware(
//     State(state): State<AppState>,
//     headers: HeaderMap,
//     mut request: Request,
//     next: Next,
// ) -> Result<impl IntoResponse, StatusCode> {
//     // Extract the Authorization header
//     let cookie_header = headers
//         .get("Cookie")
//         .and_then(|header| header.to_str().ok());

//     // Insert the decoded claims into request extensions for use in handlers
//     request.extensions_mut().insert(token_data.claims);

//     // Proceed to the next handler
//     Ok(next.run(request).await)
// }

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

// the input to our `create_user` handler
#[derive(Deserialize, sqlx::FromRow)]
pub struct JustId {
    pub id: String,
}

async fn delete_user_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> EmptyApiResult {
    debug!("delete by id called");
    debug!("{}", id);

    let query_result = sqlx::query!(r#"DELETE FROM users WHERE id = $1"#, id)
        .execute(&state.pg_pool)
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
        Ok(result_info) => {
            if result_info.rows_affected() == 0 {
                Err((
                    StatusCode::NOT_ACCEPTABLE,
                    "User with given ID does not exist - possibly already deleted".to_string(),
                ))
            } else {
                Ok(StatusCode::NO_CONTENT)
            }
        }
    }
}

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
