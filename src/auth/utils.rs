use crate::auth::roles::Role;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    pub user_id: String,
    pub session_id: String,
    pub org_id: String,
    pub roles: Vec<Role>,
}

// TODO: IMPORTANT! hash passwords
