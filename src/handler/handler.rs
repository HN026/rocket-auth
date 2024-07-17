use crate::database::database::DbConn;
use crate::jwt::jwt::generate_token;
use crate::models::models::{LoginUser, NewUser, SignupUser, User};
use crate::schema::users::dsl::*;
use bcrypt::{hash, verify, DEFAULT_COST};
use diesel::insert_into;
use diesel::prelude::*;
use rocket::serde::json::Json;

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
