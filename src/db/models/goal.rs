use crate::db::{models::user::User, schema::goals};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

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

#[derive(Debug, Deserialize, Validate)]
pub struct GoalForm {
    pub title: String,
    pub description: String,
    pub notes: Option<String>,
}

fn validate_goal_title(title: &str, conn: &mut PgConnection) -> Result<(), ValidationError> {
    let res = goals::table
        .filter(goals::title.eq(title))
        .select(goals::id)
        .execute(conn);

    if let Ok(rows) = res {
        if rows > 0 {
            return Err(ValidationError::new("duplicate_title")
                .with_message("A goal with this title already exists."));
        }
        return Ok(());
    } else {
        return Err(ValidationError::new("duplicate_title")
            .with_message("A goal with this title already exists."));
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
