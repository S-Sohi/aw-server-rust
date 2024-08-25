mod jwt;

use jsonwebtoken::{decode, encode, Header, Validation};
use jwt::{create_jwt, validate_jwt, Claims};
use rocket::response::status::BadRequest;

use crate::endpoints::{HttpErrorJson, ServerState};
use aw_models::User;
use rocket::http::hyper::server::conn::Http;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use serde::Deserialize;
use std::collections::HashMap;

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
    match datastore.get_user(input.email.to_string()) {
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
    let isUserExisted = match datastore.get_user(input.email.to_string()) {
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
