use crate::schema::users;
use rocket::serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Queryable, Serialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub otp_secret: String,
    pub otp_verified: Option<bool>,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub email: &'a str,
    pub password_hash: &'a str,
    pub otp_secret: &'a str,
    pub otp_verified: bool,
}

#[derive(Deserialize)]
pub struct SignupUser<'a> {
    pub username: &'a str,
    pub email: &'a str,
    pub password: &'a str,
}

#[derive(Deserialize, Clone)]
pub struct LoginUser<'a> {
    pub username: &'a str,
    pub password: &'a str,
}
#[derive(Deserialize)]
pub struct OtpVerification<'a> {
    pub username: &'a str,
    pub otp: &'a str,
}
