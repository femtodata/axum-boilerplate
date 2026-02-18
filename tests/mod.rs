use std::env;

use axum_boilerplate::db::{
    self,
    models::{EmailAddress, NewUser, User, hash_password},
};
use diesel::RunQueryDsl;
use dotenvy::dotenv;
use email_address::EmailAddress;

mod database;

#[test]
fn test_add() {
    assert_eq!(1 + 2, 3);
}

fn test_create_user() {
    dotenv().ok();

    let db_url = env::var("DATABASE_TEST_URL").expect("DATABASE_TEST_URL env var needed");
    let db = database::Database::new(&db_url);

    let conn = db.conn();

    let new_user = NewUser {
        username: "test-01",
        email: Some(&EmailAddress::new("test-01@test.com").unwrap()),
        hashed_password: Some(hash_password("blahblahblah".to_string()).unwrap().as_str()),
    };
}
