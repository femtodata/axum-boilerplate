use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use dotenvy::dotenv;
use std::env;

pub mod models;
pub mod schema;

pub fn establish_connection(db_url: Option<String>) -> PgConnection {
    dotenv().ok();

    let database_url =
        db_url.unwrap_or_else(|| env::var("DATABASE_URL").expect("DATABASE_URL needs definition"));

    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn get_connection_pool() -> Pool<ConnectionManager<PgConnection>> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Could not build connection pool!");

    pool
}
