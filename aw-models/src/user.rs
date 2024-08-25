use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub name: String,
    pub role: i8,
    pub lastname: String,
    pub password: String,
}
