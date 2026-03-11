use chrono::NaiveDate;
use diesel::prelude::*;
use serde::Deserialize;

use crate::db::models::Goal;

#[derive(Debug, Deserialize, Queryable, Identifiable, Associations, Selectable, AsChangeset)]
#[diesel(table_name = crate::db::schema::applied_goals)]
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
