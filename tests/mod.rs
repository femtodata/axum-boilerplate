use std::env;

use axum_boilerplate::db::models::{EmailAddress, NewUser, user::hash_password};
use database::run_migrations;
use dotenvy::dotenv;

mod database;

#[test]
fn test_create_user() {
    dotenv().ok();

    let db_url = env::var("DATABASE_TEST_URL").expect("DATABASE_TEST_URL env var needed");
    let db = database::Database::new(&db_url);

    let db = db.create();

    let mut conn = db.conn();

    run_migrations(&mut conn);

    let new_user = NewUser {
        username: "test-01",
        email: Some(&EmailAddress::new("test-01@test.com").unwrap()),
        hashed_password: Some(hash_password("blahblahblah".to_string()).unwrap().as_str()),
    };
}
