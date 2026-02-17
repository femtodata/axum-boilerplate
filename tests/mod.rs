use std::env;

use axum_boilerplate::db::{
    self,
    models::{EmailAddress, NewUser, User, hash_password},
};
use diesel::RunQueryDsl;
use dotenvy::dotenv;
use email_address::EmailAddress;

#[test]
fn test_add() {
    assert_eq!(1 + 2, 3);
}

fn test_create_user() {
    dotenv().ok();

    let db_url = env::var("DATABASE_TEST_URL").expect("DATABASE_TEST_URL env var needed");

    let conn = db::establish_connection(Some(db_url));

    let new_user = NewUser {
        username: "test-01",
        email: Some(&EmailAddress::new("test-01@test.com").unwrap()),
        hashed_password: Some(hash_password("blahblahblah".to_string()).unwrap().as_str()),
    };
}

fn create_test_db(url: &str) {
    let (database, db_url) = split_url(url.to_owned());
    let mut conn = db::establish_connection(Some(db_url));
    diesel::sql_query(format!(r#"CREATE DATABASE "{}""#, database))
        .execute(&mut conn)
        .unwrap();
}

fn split_url(url: String) -> (String, String) {
    let mut split: Vec<&str> = url.split('/').collect();
    let database = split.pop().unwrap();
    let postgres_url = format!("{}/{}", split.join("/"), "postgres");
    (database.into(), postgres_url)
}
