#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
extern crate dotenv;

use rocket::serde::json::Json;
use dotenv::dotenv;
use bcrypt::{hash, verify, DEFAULT_COST};
mod schema;
mod models;
mod database;
mod handler;

use database::database::{establish_connection, DbConn};
use handler::handler::{signup, signin};


#[launch]
fn rocket() -> _ {
    dotenv().ok();

    match establish_connection() {
        Ok(_) => println!("Successful connection to the database"),
        Err(e) => panic!("Failed to connect to the database: {:?}", e),
    }

    rocket::build()
        .attach(DbConn::fairing())
        .mount("/", routes![signup,signin])
}