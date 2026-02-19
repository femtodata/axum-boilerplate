use std::io::Write;

use diesel::{
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::{Pg, PgValue},
    prelude::*,
    serialize::{IsNull, ToSql},
};

use thiserror::Error;

use crate::webapp::WebappError;

pub mod user;
pub use crate::db::models::user::EmailAddress;
pub use crate::db::models::user::NewUser;
pub use crate::db::models::user::User;

// Goal

#[derive(Debug, Queryable, Selectable, AsChangeset)]
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
pub struct NewGoal<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub notes: Option<&'a str>,
    pub user_id: i32,
}
