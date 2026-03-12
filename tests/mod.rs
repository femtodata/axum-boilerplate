use axum_boilerplate::db::{
    models::{
        AppliedGoal, EmailAddress, Goal, NewAppliedGoal, NewGoal, NewUser, User,
        applied_goal::create_new_applied_goal,
        goal::{GoalContext, GoalForm, create_new_goal},
        user::{create_new_user, hash_password, verify_password},
    },
    schema::{applied_goals, goals},
};
use chrono::{Local, NaiveDate};
use database::run_migrations;
use diesel::{debug_query, pg::Pg, prelude::*};
use dotenvy::dotenv;
use std::env;
use validator::ValidateArgs;

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

fn get_goal_02_form() -> GoalForm {
    GoalForm {
        title: "Goal-02".to_string(),
        description: "Goal-02 Description".to_string(),
        notes: Some("Goal-02 notes".to_string()),
    }
}

fn get_applied_goal_01(goal_id: i32, date: NaiveDate) -> NewAppliedGoal {
    NewAppliedGoal {
        goal_id,
        date,
        points_possible: 3,
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
    let new_goal = test_goal_form(&mut conn, &user);

    let applied_goal = test_applied_goal(&mut conn, &goal);

    test_applied_goal_relation(&mut conn, &applied_goal, &goal, &user);
}

fn test_user(conn: &mut diesel::PgConnection) -> User {
    println!("testing user");
    let new_user = get_user_01();
    let user = create_new_user(&new_user, conn)
        .unwrap_or_else(|err| panic!("error creating new user: {}", err));

    assert_eq!(user.username, new_user.username);
    assert_eq!(user.email, new_user.email);
    assert_eq!(user.hashed_password, new_user.hashed_password);
    assert!(
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

    assert!(goals.contains(goal));
}

fn test_goal_form(conn: &mut PgConnection, user: &User) -> Goal {
    println!("testing goal form");

    let mut context = GoalContext {
        conn,
        current_title: None,
    };

    let goal_form = get_goal_02_form();
    let validation_result = goal_form.validate_with_args(&mut context);
    assert!(validation_result.is_ok());
    let new_goal = NewGoal {
        title: goal_form.title,
        description: goal_form.description,
        notes: goal_form.notes,
        user_id: user.id,
    };
    let goal = create_new_goal(&new_goal, conn)
        .unwrap_or_else(|err| panic!("error creating new goal: {}", err));
    assert_eq!(goal.title, new_goal.title);
    assert_eq!(goal.description, new_goal.description);
    assert_eq!(goal.notes, new_goal.notes);
    assert_eq!(goal.user_id, new_goal.user_id);
    goal
}

fn test_applied_goal(conn: &mut diesel::PgConnection, goal: &Goal) -> AppliedGoal {
    println!("testing applied_goal");

    let today = Local::now().date_naive();

    let new_applied_goal = get_applied_goal_01(goal.id, today);
    let applied_goal = create_new_applied_goal(&new_applied_goal, conn)
        .unwrap_or_else(|err| panic!("error create new applied_goal: {err}"));
    assert_eq!(applied_goal.goal_id, goal.id);
    assert_eq!(applied_goal.date, new_applied_goal.date);
    assert_eq!(
        applied_goal.points_possible,
        new_applied_goal.points_possible
    );
    assert_eq!(applied_goal.points_scored, 0);
    println!("{applied_goal:#?}");
    applied_goal
}

fn test_applied_goal_relation(
    conn: &mut PgConnection,
    applied_goal: &AppliedGoal,
    goal: &Goal,
    user: &User,
) {
    println!("testing applied_goal, goal, user relation");
    let subquery = goals::table
        .filter(goals::user_id.eq(user.id))
        .select(goals::id)
        .into_boxed();

    let query = applied_goals::table
        .filter(applied_goals::goal_id.eq_any(subquery))
        .select(AppliedGoal::as_select())
        .into_boxed();

    println!("subquery version: {}", debug_query::<Pg, _>(&query));

    let applied_goals = query
        .load(conn)
        .unwrap_or_else(|err| panic!("Error loading applied_goals: {err:#?}"));

    assert!(applied_goals.contains(applied_goal));

    let goal_applied_goals = AppliedGoal::belonging_to(goal)
        .select(AppliedGoal::as_select())
        .load(conn)
        .unwrap_or_else(|err| panic!("Error loading applied_goals: {err:#?}"));

    assert!(goal_applied_goals.contains(applied_goal));
}
