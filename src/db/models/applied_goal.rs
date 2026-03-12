use chrono::NaiveDate;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::db::{models::Goal, schema::applied_goals};

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
#[diesel(table_name = crate::db::schema::applied_goals)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(Goal))]
pub struct AppliedGoal {
    pub id: i32,
    pub goal_id: i32,
    pub date: NaiveDate,
    pub points_possible: i32,
    pub points_scored: i32,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::db::schema::applied_goals)]
pub struct NewAppliedGoal {
    pub goal_id: i32,
    pub date: NaiveDate,
    pub points_possible: i32,
}

#[derive(Debug, Deserialize, AsChangeset)]
#[diesel(table_name = crate::db::schema::applied_goals)]
pub struct AppliedGoalForm {
    pub points_possible: Option<i32>,
    pub points_scored: Option<i32>,
}

pub fn create_new_applied_goal(
    new_applied_goal: &NewAppliedGoal,
    conn: &mut PgConnection,
) -> Result<AppliedGoal, diesel::result::Error> {
    diesel::insert_into(applied_goals::table)
        .values(new_applied_goal)
        .returning(AppliedGoal::as_returning())
        .get_result(conn)
}
