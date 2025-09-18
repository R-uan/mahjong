use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct JoinRequest {
    pub id: String,
    pub alias: String,
}
