use jsonwebtoken::{decode, encode, Header, Validation};
use jwt::{create_jwt, validate_jwt, Claims};
use rocket::outcome::Outcome;
use rocket::request::{self, FromRequest, Request};
use rocket::response::status::BadRequest;

use crate::endpoints::{HttpErrorJson, ServerState};
use aw_models::User;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use serde::Deserialize;
use std::collections::HashMap;
mod jwt;

#[derive(Deserialize, Clone, Copy)]
pub struct LoginModel<'r> {
    email: &'r str,
    password: &'r str,
}

#[derive(Deserialize, Clone, Copy)]
pub struct SignupModel<'r> {
    password: &'r str,
    email: &'r str,
    name: &'r str,
    lastname: &'r str,
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

#[post("/login", data = "<input>")]
pub fn login(
    state: &State<ServerState>,
    input: Json<LoginModel>,
) -> Result<Json<String>, HttpErrorJson> {
    let email = input.email.to_string();
    let password = input.password.to_string();
    if (email.is_empty() || password.is_empty()) {
        let err_msg = format!("No user");
        return Err(HttpErrorJson::new(Status::BadRequest, err_msg));
    }
    let datastore = endpoints_get_lock!(state.datastore);
    match datastore.get_user_by_email(input.email.to_string()) {
        Ok(user) => {
            if user.password.clone() == password {
                let claims = Claims {
                    userId: user.id,
                    exp: 10000000000, // Set your expiration logic
                };

                match create_jwt(&claims) {
                    Ok(token) => Ok(Json(token)),
                    Err(_) => Err(HttpErrorJson::new(
                        Status::BadRequest,
                        "could not generate token".to_string(),
                    )),
                }
            } else {
                return Err(HttpErrorJson::new(
                    Status::BadRequest,
                    "No user with this password found".to_string(),
                ));
            }
        }
        Err(err) => Err(err.into()),
    }
}

#[post("/signup", data = "<input>")]
pub fn signup(
    state: &State<ServerState>,
    input: Json<SignupModel>,
) -> Result<Json<bool>, HttpErrorJson> {
    let password = input.password.to_string();
    let email = input.email.to_string();
    let name = input.name.to_string();
    let lastname = input.lastname.to_string();
    if (email.is_empty() || password.is_empty()) {
        let err_msg = format!("No user");
        return Err(HttpErrorJson::new(Status::BadRequest, err_msg));
    }
    let user = User {
        id: 0,
        email: email,
        password: password,
        name: name,
        lastname: lastname,
        role: 1,
    };

    let datastore = endpoints_get_lock!(state.datastore);
    let isUserExisted = match datastore.get_user_by_email(input.email.to_string()) {
        Ok(user) => true,
        Err(_) => false,
    };
    if (isUserExisted == true) {
        return Err(HttpErrorJson::new(
            Status::BadRequest,
            "Email is used".to_string(),
        ));
    }
    match datastore.add_user(user) {
        Ok(user) => Ok(Json(true)),
        Err(err) => Err(err.into()),
    }
}

#[get("/getuser")]
pub fn getUser(state: &State<ServerState>, token: Token) -> Result<Json<User>, HttpErrorJson> {
    let tokenString = token.clone().0;
    let userId = match validate_jwt(&tokenString) {
        Ok(userId) => userId,
        Err(_) => todo!(),
    };
    let userId = 1;
    let datastore = endpoints_get_lock!(state.datastore);

    match datastore.get_user(userId) {
        Ok(user) => Ok(Json(user)),
        Err(_) => Err(HttpErrorJson::new(
            Status::BadRequest,
            "Email is used".to_string(),
        )),
    }
}
