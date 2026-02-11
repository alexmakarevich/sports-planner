use crate::auth::roles::Role;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    pub user_id: String,
    pub session_id: String,
    pub club_id: String,
    pub roles: Vec<Role>,
}

// TODO: IMPORTANT! hash passwords

pub const EXPIRED_EMPTY_COOKIE: &str =
    "session_id=; HttpOnly; SameSite=Strict; Secure; Expires=1 Jan 1970 00:00:00 GMT";
