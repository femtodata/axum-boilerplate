use std::borrow::Cow;

use crate::db::{models::user::User, schema::goals};
use diesel::PgConnection;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidateArgs, ValidationError};

#[derive(
    Debug,
    PartialEq,
    Serialize,
    Deserialize,
    Queryable,
    Identifiable,
    Associations,
    Selectable,
    AsChangeset,
)]
#[diesel(belongs_to(User))]
#[diesel(table_name = crate::db::schema::goals)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(treat_none_as_null = true)]
pub struct Goal {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub notes: Option<String>,
    pub user_id: i32,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::db::schema::goals)]
pub struct NewGoal {
    pub title: String,
    pub description: String,
    pub notes: Option<String>,
    pub user_id: i32,
}

#[derive(Debug, Deserialize, Validate, AsChangeset)]
#[diesel(table_name = crate::db::schema::goals)]
#[validate(context = "GoalContext<'v_a>", mutable)]
pub struct GoalForm {
    #[validate(custom(function = "validate_goal_title", use_context))]
    pub title: String,
    pub description: String,
    pub notes: Option<String>,
}

pub struct GoalContext<'a> {
    pub conn: &'a mut PgConnection,
    pub current_title: Option<&'a str>,
}

fn validate_goal_title(title: &str, context: &mut GoalContext) -> Result<(), ValidationError> {
    let mut query = goals::table
        .select(goals::id)
        .filter(goals::title.eq(title))
        .into_boxed();

    if let Some(current_title) = context.current_title {
        query = query.filter(goals::title.ne(current_title));
    };

    let res = query.execute(context.conn);

    if let Ok(rows) = res {
        if rows > 0 {
            return Err(ValidationError::new("duplicate_title")
                .with_message(Cow::from("A goal with this title already exists.")));
        }
        return Ok(());
    } else {
        return Err(
            ValidationError::new("db_error").with_message(Cow::from("An error has occurred"))
        );
    }
}

pub fn create_new_goal(
    new_goal: &NewGoal,
    conn: &mut PgConnection,
) -> Result<Goal, diesel::result::Error> {
    diesel::insert_into(goals::table)
        .values(new_goal)
        .returning(Goal::as_returning())
        .get_result(conn)
}
