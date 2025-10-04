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

mod auth;
mod entities;
mod utils;

// TODO: don't expose internals in error responses (though they are helpful in the early stages of dev)
// TODO: generically handle DB errors for extra-pretty code
// soft-deletes via deleted_at (not super high-prio now)

use crate::{
    auth::routes::{cookie_auth_middleware, log_in},
    entities::{
        org::delete_own_org,
        user::{create_user, delete_user_by_id, list_users, sign_up_with_new_org},
    },
    utils::api::AppState,
};

#[derive(sqlx::FromRow)]
pub struct ConfigModel {
    pub is_initialized: bool,
}

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

    // load persistent config
    let Ok(persistent_config) = sqlx::query_as!(ConfigModel, "SELECT * FROM config",)
        .fetch_one(&pool)
        .await
    else {
        panic!("could not get persistent config")
    };

    // check if app has already been initialized
    if !persistent_config.is_initialized {
        let initial_org_title = dotenv::var("INITIAL_ORG").expect("INITIAL_ORG is not configured");
        let initial_user_name =
            dotenv::var("INITIAL_USER").expect("INITIAL_USERNAME is not configured");
        let initial_user_password =
            dotenv::var("INITIAL_PASSWORD").expect("INITIAL_PASSWORD is not configured");

        let Ok(mut tx) = pool.begin().await else {
            panic!("could not start init transaction")
        };

        let Ok(created_org) = sqlx::query!(
            r#"INSERT INTO orgs (title) VALUES ($1) RETURNING id"#,
            initial_org_title,
        )
        .fetch_one(&mut *tx)
        .await
        else {
            panic!("could not create initial org")
        };

        let Ok(created_user) = sqlx::query!(
            r#"INSERT INTO users (username, password, org_id) VALUES ($1, $2, $3) RETURNING id"#,
            initial_user_name,
            initial_user_password,
            created_org.id,
        )
        .fetch_one(&mut *tx)
        .await
        else {
            panic!("could not create initial user")
        };

        // role assginment
        let _ = sqlx::query!(
            r#"INSERT INTO role_assignments (user_id, role) VALUES ($1, 'super_admin')"#,
            created_user.id,
        )
        .execute(&mut *tx)
        .await
        .map_err(|err| {
            error!("{}", err);
            panic!("could not create initial role assignment")
        });

        let _ = sqlx::query!("UPDATE config SET is_initialized=TRUE",)
            .execute(&mut *tx)
            .await
            .map_err(|err| {
                error!("{}", err);
                panic!("could set initial config")
            });

        let _ = tx.commit().await.map_err(|err| {
            error!("{}", err);
            panic!("failed to initialize application")
        });
    }

    let state = AppState { pg_pool: pool };

    // build our application with a route
    let app = Router::new()
        .nest("/api", api_routes(state.clone()))
        .layer(ServiceBuilder::new().layer(middleware::from_fn(logging_middleware)))
        .with_state(state);

    fn unprotected_api_routes<S>(state: AppState) -> Router<S> {
        Router::new()
            .route("/log-in", post(log_in))
            .route("/sign-up-with-new-org", post(sign_up_with_new_org))
            .with_state(state)
    }

    fn protected_api_routes<S>(state: AppState) -> Router<S> {
        Router::new()
            .route("/users/list", get(list_users))
            .route("/users/create", post(create_user))
            .route("/users/delete-by-id/{id}", delete(delete_user_by_id))
            .route("/orgs/delete-own", delete(delete_own_org))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                cookie_auth_middleware,
            ))
            .with_state(state)
    }

    fn api_routes<S>(state: AppState) -> Router<S> {
        Router::new()
            .merge(protected_api_routes(state.clone()))
            .merge(unprotected_api_routes(state.clone()))
            .with_state(state)
    }

    // two lines below and their respective imports are necessary to remove trailing slashes from URLs (otherwise routes with and without them are treated as separate)
    // see https://github.com/tokio-rs/axum/issues/2659
    let app = NormalizePathLayer::trim_trailing_slash().layer(app);
    let app = ServiceExt::<Request>::into_make_service(app);

    info!("running rust server on localhost:3333");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3333").await.unwrap();
    axum::serve(listener, app).await.unwrap();
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
