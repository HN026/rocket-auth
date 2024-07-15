use diesel::prelude::*;
use rocket_sync_db_pools::database;
use std::env;

#[database("pg_database")]
pub struct DbConn(diesel::PgConnection);

pub fn establish_connection() -> Result<diesel::PgConnection, diesel::ConnectionError> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");
    PgConnection::establish(&database_url)
}
