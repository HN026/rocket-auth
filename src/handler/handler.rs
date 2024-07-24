use base32::Alphabet;
use totp_rs::{Algorithm, TOTP, Secret};
use bcrypt::{hash, verify, DEFAULT_COST};
use diesel::insert_into;
use diesel::prelude::*;
use rocket::serde::json::Json;
use base32::encode as base32_encode;
use base32::decode as base32_decode;

use crate::database::database::DbConn;
use crate::jwt::jwt::generate_token;
use crate::models::models::{LoginUser, NewUser, SignupUser, User, OtpVerification};
use crate::schema::users::dsl::*;
use crate::handler::otp::send_otp_via_email;

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

    let secret = Secret::generate_secret();
    let secret_bytes = match secret.to_bytes() {
        Ok(bytes) => bytes,
        Err(_) => return Json(Err("Failed to convert OTP secret to bytes".into())),
    };

    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        Secret::Raw("TestSecretSuperSecret".as_bytes().to_vec()).to_bytes().unwrap(),
    ).unwrap();

    let otp_code = totp.generate_current().unwrap();

    match send_otp_via_email(&signup_user.email, &otp_code).await {
        Ok(_) => (),
        Err(err) => return Json(Err(format!("Failed to send OTP: {}", err))),
    }
    let username_clone = signup_user.username.to_string();
    let email_clone = signup_user.email.to_string();
    let password_hash_clone = hashed_password.clone();
    let otp_secret_base32 = base32_encode(base32::Alphabet::RFC4648 { padding: false }, &secret_bytes);

    let result = conn
        .run(move |c| {
            let new_user = NewUser {
                username: &username_clone,
                email: &email_clone,
                password_hash: &password_hash_clone,
                otp_secret: &otp_secret_base32,
                otp_verified: false,
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
                if user.otp_verified.unwrap_or(false) {
                    let token = generate_token(&user.username)
                        .unwrap_or_else(|_| "Failed to generate token".to_string());
                    Json(Ok(token))
                } else {
                    let secret_bytes = match base32_decode(Alphabet::RFC4648 { padding: false }, &user.otp_secret) {
                        Some(bytes) => bytes,
                        None => return Json(Err("Failed to decode OTP secret".into())),
                    };
                    let totp = TOTP::new(
                        Algorithm::SHA1,
                        6,
                        1,
                        30,
                        secret_bytes,
                    ).unwrap();

                    let otp_code = totp.generate_current().unwrap();

                    if let Err(err) = send_otp_via_email(&user.email, &otp_code).await {
                        return Json(Err(format!("Failed to send OTP: {}", err)));
                    }

                    Json(Ok("OTP sent. Please verify to complete sign in.".to_string()))
                }
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

#[post("/verify_otp", format = "json", data = "<otp_info>")]
pub async fn verify_otp<'a>(
    conn: DbConn,
    otp_info: Json<OtpVerification<'a>>,
) -> Json<Result<String, String>> {
    let username_clone = otp_info.username.to_string();

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
            // let secret_bytes = match base32_decode(Alphabet::RFC4648 { padding: false }, &user.otp_secret) {
            //     Some(bytes) => bytes,
            //     None => return Json(Err("Failed to decode OTP secret".into())),
            // };
            let totp = TOTP::new(
                Algorithm::SHA1,
                6,
                1,
                30,
                Secret::Raw("TestSecretSuperSecret".as_bytes().to_vec()).to_bytes().unwrap(),
            ).unwrap();

            let current_otp = totp.generate_current().unwrap();
            if current_otp == otp_info.otp  {
                let update_result = conn
                    .run(move |c| {
                        diesel::update(users.filter(id.eq(user.id)))
                            .set(otp_verified.eq(true))
                            .execute(c)
                    })
                    .await;

                match update_result {
                    Ok(_) => {
                        let token = generate_token(&user.username)
                            .unwrap_or_else(|_| "Failed to generate token".to_string());
                        Json(Ok(token))
                    }
                    Err(err) => {
                        eprintln!("Error updating user OTP status: {:?}", err);
                        Json(Err("Error verifying OTP".into()))
                    }
                }
            } else {
                Json(Err("Invalid OTP".into()))
            }
        }
        Ok(None) => Json(Err("User not found".into())),
        Err(err) => {
            eprintln!("Error during OTP verification: {:?}", err);
            Json(Err("Error during OTP verification".into()))
        }
    }
}
