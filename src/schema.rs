// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Uuid,
        username -> Varchar,
        email -> Varchar,
        password_hash -> Varchar,
        otp_secret -> Varchar,
        otp_verified -> Nullable<Bool>,
    }
}
