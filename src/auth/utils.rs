// For now same as sqlx model, but has a different domain and may be changed a lot
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: String,
    pub session_id: String,
    pub org_id: String,
    // pub roles: Vec<Roles>
}

// TODO: IMPORTANT! hash passwords
