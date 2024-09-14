use jsonwebtoken::{decode, encode, Header, Validation};
use jwt::{create_jwt, validate_jwt, Claims};
use rocket::outcome::Outcome;
use rocket::request::{self, FromRequest, Request};
use rocket::response::status::BadRequest;

use crate::endpoints::{HttpErrorJson, ServerState};
use aw_models::TeamDetailModel;
use aw_models::TeamRequestModel;
use aw_models::TeamResponseModel;
use aw_models::User;
use aw_models::{Team, TeamUserModel};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use serde::Deserialize;
use serde::Serialize;

use std::array;
use std::collections::HashMap;
mod jwt;

#[derive(Deserialize, Clone, Copy)]
pub struct TeamModel<'r> {
    name: &'r str,
    description: &'r str,
}

#[derive(Deserialize, Clone)]
pub struct Token(String);
#[rocket::async_trait]
impl<'r> FromRequest<'r> for Token {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        // Look for the "Authorization" header
        if let Some(token_header) = request.headers().get_one("Authorization") {
            if let Some(token) = token_header.strip_prefix("Bearer ") {
                return Outcome::Success(Token(token.to_string()));
            }
        }

        Outcome::Forward(Status::Unauthorized)
    }
}

#[get("/")]
pub fn getOwnerTeams(
    state: &State<ServerState>,
    token: Token,
) -> Result<Json<Vec<TeamResponseModel>>, HttpErrorJson> {
    let tokenString = token.clone().0;
    let userId = match validate_jwt(&tokenString) {
        Ok(userId) => userId,
        Err(_) => todo!(),
    };
    let datastore = endpoints_get_lock!(state.datastore);
    let mut response: Vec<TeamResponseModel> = Vec::new();
    match datastore.get_owner_teams(userId) {
        Ok(teams) => {
            for team in teams {
                let count = datastore.get_team_members_count(team.id)?;
                response.push(TeamResponseModel {
                    id: team.id,
                    name: team.name,
                    description: team.description,
                    count,
                })
            }
        }
        Err(_) => todo!(),
    }
    Ok(Json(response))
}

#[post("/", data = "<team>")]
pub fn addTeam(
    state: &State<ServerState>,
    team: Json<TeamModel>,
    token: Token,
) -> Result<Json<bool>, HttpErrorJson> {
    let name = team.name.to_string();
    if name.is_empty() {
        let err_msg = format!("No name was provided");
        return Err(HttpErrorJson::new(Status::BadRequest, err_msg));
    }
    let tokenString = token.clone().0;
    let ownerId = match validate_jwt(&tokenString) {
        Ok(ownerId) => ownerId,
        Err(_) => todo!(),
    };
    let datastore = endpoints_get_lock!(state.datastore);
    let teamModel = TeamRequestModel {
        description: team.description.to_string(),
        name: team.name.to_string(),
        ownerId: ownerId,
    };
    match datastore.add_team(teamModel, ownerId) {
        Ok(team) => Ok(Json(true)),
        Err(_) => Ok(Json(false)),
    }
    // match datastore.get_user_by_email(input.email.to_string()) {
    //     Ok(user) => {
    //         if user.password.clone() == password {
    //             let claims = Claims {
    //                 userId: user.id,
    //                 exp: 10000000000, // Set your expiration logic
    //             };

    //             match create_jwt(&claims) {
    //                 Ok(token) => Ok(Json(token)),
    //                 Err(_) => Err(HttpErrorJson::new(
    //                     Status::BadRequest,
    //                     "could not generate token".to_string(),
    //                 )),
    //             }
    //         } else {
    //             return Err(HttpErrorJson::new(
    //                 Status::BadRequest,
    //                 "No user with this password found".to_string(),
    //             ));
    //         }
    //     }
    //     Err(err) => Err(err.into()),
    // }
}

#[get("/team/<id>")]
pub fn getTeam(
    state: &State<ServerState>,
    token: Token,
    id: i32,
) -> Result<Json<TeamDetailModel>, HttpErrorJson> {
    // let tokenString = token.clone().0;
    // let userId = match validate_jwt(&tokenString) {
    //     Ok(userId) => userId,
    //     Err(_) => todo!(),
    // };
    let datastore = endpoints_get_lock!(state.datastore);
    let members = datastore.get_team_members(id).unwrap();
    match datastore.get_team(id) {
        Ok(team) => Ok(Json(TeamDetailModel {
            id: team.id,
            description: team.description,
            name: team.name,
            members: members,
        })),
        Err(err) => Err(HttpErrorJson::new(
            Status::BadRequest,
            "Something went wrong!".to_string(),
        )),
    }
}

#[post("/<teamId>/members", data = "<members>")]
pub fn addMembers(
    state: &State<ServerState>,
    teamId: i32,
    members: Json<Vec<i32>>,
    token: Token,
) -> Result<Json<bool>, HttpErrorJson> {
    let tokenString = token.clone().0;
    let ownerId = match validate_jwt(&tokenString) {
        Ok(ownerId) => ownerId,
        Err(_) => todo!(),
    };
    let datastore = endpoints_get_lock!(state.datastore);
    let memberIds = members.0;
    match datastore.add_members(teamId, memberIds) {
        Ok(team) => Ok(Json(true)),
        Err(_) => Ok(Json(false)),
    }
}

#[delete("/<teamId>/member/<memberId>")]
pub fn removeMember(
    state: &State<ServerState>,
    teamId: i32,
    memberId: i32,
    token: Token,
) -> Result<Json<bool>, HttpErrorJson> {
    let tokenString = token.clone().0;
    let ownerId = match validate_jwt(&tokenString) {
        Ok(ownerId) => ownerId,
        Err(_) => todo!(),
    };
    let datastore = endpoints_get_lock!(state.datastore);
    match datastore.remove_member(teamId, memberId) {
        Ok(team) => Ok(Json(true)),
        Err(_) => Ok(Json(false)),
    }
}

#[get("/user")]
pub fn getUserTeams(
    state: &State<ServerState>,
    token: Token,
) -> Result<Json<Vec<TeamUserModel>>, HttpErrorJson> {
    let token_string = token.clone().0;
    let user_id = match validate_jwt(&token_string) {
        Ok(user_id) => user_id,
        Err(_) => todo!(),
    };
    let datastore = endpoints_get_lock!(state.datastore);
    match datastore.get_user_teams(user_id) {
        Ok(teams) => Ok(Json(teams)),
        Err(err) => {
            return Err(err.into())
        }
    }
    // Ok(Json(response))
}
