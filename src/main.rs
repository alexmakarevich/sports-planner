use axum::{
    extract::{Extension, Path, Request, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router, ServiceExt,
};
use log::info;
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicU16, Ordering::Relaxed},
    Arc,
};
use tokio::sync::Mutex;
use tower_http::normalize_path::NormalizePathLayer;
use tower_layer::Layer;

#[derive(Serialize, Deserialize)]
struct Greeting {
    greeting: String,
    visitor: String,
    visits: u16,
}

impl Greeting {
    fn new(greeting: &str, visitor: String, visits: u16) -> Self {
        Greeting {
            greeting: greeting.to_string(),
            visitor,
            visits,
        }
    }
}

struct AppState {
    number_of_visits: AtomicU16,
    users: Vec<User>,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // needed so that logs are actually printed to the console
    env_logger::init();

    // Create a shared state for our application. We use an Arc so that we clone the pointer to the state and
    // not the state itself. The AtomicU16 is a thread-safe integer that we use to keep track of the number of visits.
    let app_state = Arc::new(Mutex::new(AppState {
        number_of_visits: AtomicU16::new(1),
        users: vec![],
    }));

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/hello/{visitor}", get(greet_visitor))
        .route("/bye", delete(say_goodbye))
        .with_state(app_state);
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

/// Extract the `visitor` path parameter and use it to greet the visitor.
/// We also use the `State` extractor to access the shared `AppState` and increment the number of visits.
/// We use `Json` to automatically serialize the `Greeting` struct to JSON.
async fn greet_visitor(
    State(app_state): State<Arc<Mutex<AppState>>>,
    Path(visitor): Path<String>,
) -> Json<Greeting> {
    let state = app_state.lock().await;

    let visits = state.number_of_visits.fetch_add(1, Relaxed);
    Json(Greeting::new("Hello", visitor, visits))
}

/// Say goodbye to the visitor.
async fn say_goodbye() -> String {
    info!("bye called");

    "Goodbye".to_string()
}

// async fn create_user(
//     State(app_state): State<Arc<AppState>>,
//     // this argument tells axum to parse the request body
//     // as JSON into a `CreateUser` type
//     Json(payload): Json<CreateUser>,
// ) -> (StatusCode, Json<User>) {
//     // insert your application logic here
//     let user = User {
//         id: 1337,
//         username: payload.username,
//     };

//     // let users = &app_state.clone().users;

//     // users.push(user);

//     // // this will be converted into a JSON response
//     // // with a status code of `201 Created`
//     // (StatusCode::CREATED, Json(user))
// }

// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}
