use axum::{
    extract::Request,
    middleware::{self, Next},
    response::Response,
    routing::{delete, get, post},
    Router, ServiceExt,
};
use dotenv::dotenv;
use log::{error, info};
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use tower::ServiceBuilder;
use tower_http::normalize_path::NormalizePathLayer;
use tower_layer::Layer;

mod entities;
mod utils;

use crate::{
    entities::{
        auth::{dumb_cookie_middleware, log_in},
        user::{create_user, delete_user_by_id, list_users},
    },
    utils::api::AppState,
};

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
        .route("/login", post(log_in))
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

// the input to our `create_user` handler
#[derive(Deserialize, sqlx::FromRow)]
pub struct JustId {
    pub id: String,
}
