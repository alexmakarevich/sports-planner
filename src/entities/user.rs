use serde::{Deserialize, Serialize};

// For sqlx
// all the fields - not sure if needed, but will leave here for now
#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
#[allow(non_snake_case)] // TODO: why though? it's all snaky anyway
pub struct UserModel {
    pub id: String,
    pub username: String,
    pub password: String,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

// the input to our `create_user` handler
#[derive(Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub password: String,
}

// user from DB wihtout security and unnecessary util fields
#[derive(Serialize, Debug, Clone, sqlx::FromRow)]
pub struct UserClean {
    pub id: String,
    pub username: String,
}
