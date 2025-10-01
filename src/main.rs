use axum::{
    extract::{Request, State},
    http::header::SET_COOKIE,
    middleware::{self, Next},
    response::Response,
    routing::{delete, get, post},
    Router, ServiceExt,
};
use dotenv::dotenv;
use log::{debug, error, info};
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use tower::ServiceBuilder;
use tower_http::normalize_path::NormalizePathLayer;
use tower_layer::Layer;
use uuid::Uuid;

mod entities;
mod utils;

use crate::{
    entities::user::{create_user, delete_user_by_id, list_users},
    utils::api::AppState,
};

use axum_extra::extract::cookie::{Cookie, CookieJar};

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

    let state = AppState { pg_pool: pool };

    // build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .merge(protected_routes(state.clone()))
        .layer(ServiceBuilder::new().layer(middleware::from_fn(logging_middleware)))
        .with_state(state);

    fn protected_routes<S>(state: AppState) -> Router<S> {
        Router::new()
            .nest("/api", api_routes(state.clone()))
            .layer(ServiceBuilder::new().layer(middleware::from_fn_with_state(
                state.clone(),
                dumb_cookie_middleware,
            )))
            .with_state(state)
    }

    fn api_routes<S>(state: AppState) -> Router<S> {
        Router::new()
            .route("/users/list", get(list_users))
            .route("/users/create", post(create_user))
            .route("/users/delete-by-id/{id}", delete(delete_user_by_id))
            .with_state(state)
    }

    // two lines below and their respective imports are necessary to remove trailing slashes from URLs (otherwise routes with and without them are treated as separate)
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

// the input to our `create_user` handler
#[derive(Deserialize, sqlx::FromRow)]
pub struct JustId {
    pub id: String,
}
