use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use dotenvy::dotenv;
use models::User;
use std::env;

pub mod models;
pub mod schema;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL");
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

pub fn get_user_by_email(email: &str, conn: Option<&mut PgConnection>) -> Option<User> {
    let conn = match conn {
        Some(conn) => conn,
        None => &mut establish_connection(),
    };

    let user = schema::users::table
        .filter(schema::users::email.eq(email))
        .first(conn)
        .optional()
        .unwrap();

    user
}

pub fn get_user_by_username(username: &str, conn: Option<&mut PgConnection>) -> Option<User> {
    let conn = match conn {
        Some(conn) => conn,
        None => &mut establish_connection(),
    };

    let user = schema::users::table
        .filter(schema::users::username.eq(username))
        .first(conn)
        .optional()
        .unwrap();

    user
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email() {
        let user = get_user_by_email("alexou@gmail.com", None);
        println!("{user:#?}");
    }
}
