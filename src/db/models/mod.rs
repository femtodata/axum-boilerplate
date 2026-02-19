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

pub mod goal;
pub use crate::db::models::goal::Goal;
pub use crate::db::models::goal::NewGoal;
// Goal
