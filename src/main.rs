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
    Arc, OnceLock,
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

#[derive(Clone)]
struct UserState {
    users: Arc<Mutex<Vec<User>>>,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // needed so that logs are actually printed to the console
    env_logger::init();

    let user_state = UserState {
        users: Arc::new(Mutex::new(vec![])),
    };

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // .route("/new-state", get(handler))
        .route("/hello/{visitor}", get(greet_visitor))
        .route("/bye", delete(say_goodbye))
        // .route("/create-user", get(create_user(_, json)))
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

/// Extract the `visitor` path parameter and use it to greet the visitor.
/// We also use the `State` extractor to access the shared `AppState` and increment the number of visits.
/// We use `Json` to automatically serialize the `Greeting` struct to JSON.
async fn greet_visitor(State(user_state): State<UserState>, Path(visitor): Path<String>) {
    let new_user = User {
        id: 1337,
        username: visitor,
    };

    let mut users = user_state.users.lock().await;

    users.push(new_user);

    println!("Users: {:?}", *users);

    // Json(Greeting::new("Hello", "dd".to_string(), 10 as u16))
}

/// Say goodbye to the visitor.
async fn say_goodbye() -> String {
    info!("bye called");

    "Goodbye".to_string()
}

// async fn create_user(
//     State(app_state): &State<Arc<Mutex<&mut AppState>>>,
//     // this argument tells axum to parse the request body
//     // as JSON into a `CreateUser` type
//     Json(payload): Json<CreateUser>,
// ) -> StatusCode {
//     // insert your application logic here
//     let new_user = User {
//         id: 1337,
//         username: payload.username,
//     };

//     let mut state = app_state.lock().await;

//     state.users.push(new_user);

//     // this will be converted into a JSON response
//     // with a status code of `201 Created`
//     StatusCode::CREATED
// }

// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

// the output to our `create_user` handler
#[derive(Serialize, Debug)]
struct User {
    id: u64,
    username: String,
}
