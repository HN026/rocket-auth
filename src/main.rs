#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
extern crate dotenv;

use dotenv::dotenv;
mod database;
mod handler;
mod jwt;
mod models;
mod schema;

use database::database::{establish_connection, DbConn};
use handler::handler::{healthcheck, signin, signup};

#[launch]
fn rocket() -> _ {
    dotenv().ok();
    let figment = rocket::Config::figment().merge((
        "databases.pg_database.url",
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
    ));

    match establish_connection() {
        Ok(_) => println!("Successful connection to the database"),
        Err(e) => panic!("Failed to connect to the database: {:?}", e),
    }

    rocket::custom(figment)
        .mount("/", routes![signup, signin, healthcheck])
        .attach(DbConn::fairing())
}
