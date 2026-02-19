use diesel::prelude::*;
use std::env;

use axum_boilerplate::db::models::{
    EmailAddress, Goal, NewGoal, NewUser, User,
    goal::create_new_goal,
    user::{create_new_user, hash_password, verify_password},
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

fn get_goal_01(user_id: i32) -> NewGoal {
    NewGoal {
        title: "Goal-01".to_string(),
        description: "Goal-01 Description".to_string(),
        notes: Some("Goal-01 notes".to_string()),
        user_id,
    }
}

#[test]
fn test_db_ops() {
    dotenv().unwrap();

    let db_url = env::var("DATABASE_TEST_URL").expect("DATABASE_TEST_URL env var needed");
    let db = database::Database::new(&db_url);

    let db = db.create();

    let mut conn = db.conn();

    run_migrations(&mut conn);

    let user = test_user(&mut conn);
    let goal = test_goal(&mut conn, &user);
    test_user_goal(&mut conn, &user, &goal);
}

fn test_user(conn: &mut diesel::PgConnection) -> User {
    println!("testing user");
    let new_user = get_user_01();
    let user = create_new_user(&new_user, conn)
        .unwrap_or_else(|err| panic!("error creating new user: {}", err));

    assert_eq!(user.username, new_user.username);
    assert_eq!(user.email, new_user.email);
    assert_eq!(user.hashed_password, new_user.hashed_password);
    assert_eq!(
        true,
        verify_password(
            "blahblahblah",
            new_user.hashed_password.as_ref().unwrap().as_str()
        )
        .unwrap()
    );

    user
}

fn test_goal(conn: &mut diesel::PgConnection, user: &User) -> Goal {
    println!("testing goal");
    let new_goal = get_goal_01(user.id);
    let goal = create_new_goal(&new_goal, conn)
        .unwrap_or_else(|err| panic!("error creating new goal: {}", err));
    assert_eq!(goal.title, new_goal.title);
    assert_eq!(goal.description, new_goal.description);
    assert_eq!(goal.notes, new_goal.notes);
    assert_eq!(goal.user_id, new_goal.user_id);

    goal
}

fn test_user_goal(conn: &mut diesel::PgConnection, user: &User, goal: &Goal) {
    println!("testing goal user relation");
    let goals: Vec<Goal> = Goal::belonging_to(user)
        .select(Goal::as_select())
        .load(conn)
        .unwrap();

    assert_eq!(true, goals.contains(goal));
}
