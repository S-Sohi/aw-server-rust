use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct Team {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub ownerId: i32,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]

pub struct TeamRequestModel {
    pub name: String,
    pub description: String,
    pub ownerId: i32,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]

pub struct TeamResponseModel {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub count: i64,
}
