use std::collections::HashMap;

use chrono::NaiveDate;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::db::{
    models::Goal,
    schema::{applied_goals, goals},
};

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

pub fn get_applied_goals_for_dates(
    user_id: i32,
    goal_id: Option<i32>,
    start_date: NaiveDate,
    end_date: NaiveDate,
    conn: &mut PgConnection,
) -> Result<HashMap<Goal, HashMap<NaiveDate, AppliedGoal>>, diesel::result::Error> {
    let goals: Vec<Goal> = match goal_id {
        Some(goal_id) => vec![goals::table.find(goal_id).first(conn)?],
        None => goals::table
            .select(Goal::as_select())
            .filter(goals::user_id.eq(user_id))
            .load(conn)?,
    };

    let applied_goals = AppliedGoal::belonging_to(&goals)
        .select(AppliedGoal::as_select())
        .filter(applied_goals::date.between(start_date, end_date))
        .load(conn)?
        .grouped_by(&goals);

    let applied_goals = goals.into_iter().zip(applied_goals).collect::<Vec<_>>();

    let mut return_val: HashMap<Goal, HashMap<NaiveDate, AppliedGoal>> = HashMap::new();

    for (goal, ag_vec) in applied_goals.into_iter() {
        todo!();
    }

    Ok(return_val)
}

pub fn create_applied_goals_for_dates(
    goal_id: i32,
    dates: Vec<NaiveDate>,
    points_possible: i32,
) -> Result<AppliedGoal, diesel::result::Error> {
    todo!()
}
