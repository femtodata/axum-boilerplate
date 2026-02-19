use std::env;

use axum_boilerplate::db::models::{
    EmailAddress, NewUser,
    user::{create_new_user, hash_password},
};
use database::run_migrations;
use dotenvy::dotenv;

mod database;

fn get_user_01() -> NewUser {
    let email_address = EmailAddress::new("test-01@test.com").unwrap();
    let hashed_password = hash_password("blahblahblah".to_string()).unwrap();
    NewUser {
        username: "test-01".to_string(),
        email: Some(email_address),
        hashed_password: Some(hashed_password),
    }
}

#[test]
fn test_create_user() {
    dotenv().ok();

    let db_url = env::var("DATABASE_TEST_URL").expect("DATABASE_TEST_URL env var needed");
    let db = database::Database::new(&db_url);

    let db = db.create();

    let mut conn = db.conn();

    run_migrations(&mut conn);

    let new_user = get_user_01();
    let user = create_new_user(&new_user, &mut conn)
        .unwrap_or_else(|err| panic!("error creating new user: {}", err));

    assert_eq!(user.username, new_user.username);
    assert_eq!(user.email, new_user.email);
    // assert_eq!(user.hashed_password, new_user.hashed_password);
    assert_eq!(user.hashed_password, Some("blah".to_string()));
}
