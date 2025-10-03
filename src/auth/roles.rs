use strum_macros::{Display, EnumString};

// hardcoding roles, since they shouldn't be adjustable in the UI
#[derive(Debug, EnumString, Display, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum Roles {
    SuperAdmin,

    OrgAdmin,

    Coach,
    Player,
    User,
}
