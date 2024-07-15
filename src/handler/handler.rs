use crate::database::database::DbConn;
use crate::models::models::{LoginUser, NewUser, SignupUser, User};
use crate::schema::users::dsl::*;
use bcrypt::{hash, verify, DEFAULT_COST};
use diesel::insert_into;
use diesel::prelude::*;
use rocket::serde::json::Json;

#[post("/signup", format = "json", data = "<signup_user>")]
pub async fn signup<'a>(
    conn: DbConn,
    signup_user: Json<SignupUser<'a>>,
) -> Json<Result<User, String>> {
    println!("IN SIGNUP fn");
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
        Ok(user) => Json(Ok(user)),
        Err(err) => {
            eprintln!("Error saving new user: {:?}", err);
            Json(Err("Error saving new user".into()))
        }
    }
}

#[post("/signin", format = "json", data = "<login_info>")]
pub async fn signin<'a>(conn: DbConn, login_info: Json<LoginUser<'a>>) -> Json<Option<User>> {
    println!("IN SIGNIN fn");
    let username_clone = login_info.username.to_string();
    let password_clone = login_info.password.to_string();

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
            if verify(&password_clone, &user.password_hash).unwrap_or(false) {
                Json(Some(user))
            } else {
                Json(None)
            }
        }
        Err(err) => {
            eprintln!("Error during sign in: {:?}", err);
            Json(None)
        }
        _ => Json(None),
    }
}

