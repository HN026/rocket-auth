use crate::database::database::DbConn;
use crate::jwt::jwt::generate_token;
use crate::models::models::{LoginUser, NewUser, SignupUser, User};
use crate::schema::users::dsl::*;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use diesel::insert_into;
use diesel::prelude::*;
use rocket::http::{CookieJar, Status};
use rocket::response::Redirect;
use rocket::serde::json::Json;
use serde_json::json;

use super::google_oauth::{get_google_user, request_token};

#[get("/")]
pub fn healthcheck() -> &'static str {
    "Hello, world!"
}

#[post("/signup", format = "json", data = "<signup_user>")]
pub async fn signup<'a>(
    conn: DbConn,
    signup_user: Json<SignupUser<'a>>,
) -> Json<Result<String, String>> {
    let hashed_password = match hash(signup_user.password, DEFAULT_COST) {
        Ok(hp) => hp,
        Err(_) => return Json(Err("Failed to hash password".into())),
    };

    let username_clone = signup_user.username.to_string();
    let email_clone = signup_user.email.to_string();
    let password_hash_clone = hashed_password.clone();

    let result = conn
        .run(move |c| {
            let new_user = NewUser {
                username: &username_clone,
                email: &email_clone,
                password_hash: &password_hash_clone,
            };

            insert_into(users).values(&new_user).get_result::<User>(c)
        })
        .await;

    match result {
        Ok(user) => {
            let token = generate_token(&user.username)
                .unwrap_or_else(|_| "Failed to generate token".to_string());
            Json(Ok(token))
        }
        Err(err) => {
            eprintln!("Error saving new user: {:?}", err);
            Json(Err("Error saving new user".into()))
        }
    }
}

#[post("/signin", format = "json", data = "<login_info>")]
pub async fn signin<'a>(
    conn: DbConn,
    login_info: Json<LoginUser<'a>>,
) -> Json<Result<String, String>> {
    let username_clone = login_info.username.to_string();

    let result = conn
        .run(move |c| {
            users
                .filter(username.eq(username_clone))
                .first::<User>(c)
                .optional()
        })
        .await;

    match result {
        Ok(Some(user)) => {
            if verify(&login_info.password, &user.password_hash).unwrap_or(false) {
                let token = generate_token(&user.username)
                    .unwrap_or_else(|_| "Failed to generate token".to_string());
                Json(Ok(token))
            } else {
                Json(Err("Invalid credentials".into()))
            }
        }
        Ok(None) => Json(Err("Invalid credentials".into())),
        Err(err) => {
            eprintln!("Error during sign in: {:?}", err);
            Json(Err("Error during sign in".into()))
        }
    }
}

#[get("/api/sessions/oauth/google?<code>&<state>")]
pub async fn google_oauth(
    code: String,
    state: String,
    conn: DbConn,
    // jar: &CookieJar<'_>,
) -> Result<Redirect, (Status, String)> {
    if code.is_empty() {
        return Err((
            Status::Unauthorized,
            json!({"status": "fail", "message": "Authorization code not provided!"}).to_string(),
        ));
    }

    let token_response = request_token(&code).await;
    if let Err(e) = token_response {
        return Err((
            Status::BadGateway,
            json!({"status": "fail", "message": e.to_string()}).to_string(),
        ));
    }

    let token_response = token_response.unwrap();
    let google_user = get_google_user(&token_response.access_token, &token_response.id_token).await;
    if let Err(e) = google_user {
        return Err((
            Status::BadGateway,
            json!({"status": "fail", "message": e.to_string()}).to_string(),
        ));
    }

    let google_user = google_user.unwrap();
    let user_email = google_user.email;

    let result = conn
        .run(move |c| {
            // We wanted to verify if in database it is existed or not
            // If not then insert in database else just get user info

            let user = users
                .filter(email.eq(user_email))
                .first::<User>(c)
                .optional();

            // User not existed
            if let Ok(Some(_user)) = &user {
                // Insert user to db as well
            }

            return user;
        })
        .await;

    // Making of JWT token
    let _: Json<Result<String, String>> = match result {
        Ok(Some(user)) => {
            let token = generate_token(&user.username)
                .unwrap_or_else(|_| "Failed to generate token".to_string());
            Json(Ok(token))
        }
        Ok(None) => Json(Err("Invalid credentials".into())),
        Err(err) => {
            eprintln!("Error during sign in: {:?}", err);
            Json(Err("Error during sign in".into()))
        }
    };

    let frontend_origin = std::env::var("CLIENT_ORIGIN").unwrap();
    Ok(Redirect::to(format!("{}{}", frontend_origin, state)))
}
