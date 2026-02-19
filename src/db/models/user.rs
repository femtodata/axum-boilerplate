use std::io::Write;

use diesel::{
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::{Pg, PgValue},
    prelude::*,
    serialize::{IsNull, ToSql},
};

use thiserror::Error;

use crate::{
    db::schema::{self, users},
    webapp::WebappError,
};

#[derive(Debug, Clone, PartialEq, AsExpression, FromSqlRow)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub struct EmailAddress(String);

impl FromSql<diesel::sql_types::Text, Pg> for EmailAddress {
    fn from_sql(bytes: PgValue) -> diesel::deserialize::Result<Self> {
        let string = String::from_utf8(bytes.as_bytes().to_vec())?;
        unsafe { Ok(EmailAddress::new_unchecked(&string)) }
    }
}

impl ToSql<diesel::sql_types::Text, Pg> for EmailAddress {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Pg>,
    ) -> diesel::serialize::Result {
        out.write_all(self.0.as_bytes())?;
        Ok(IsNull::No)
    }
}

impl AsRef<str> for EmailAddress {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
#[error("{0} is not a valid email address")]
pub struct EmailAddressError(String);

impl EmailAddress {
    pub fn new(raw_email: &str) -> Result<Self, EmailAddressError> {
        if email_address::EmailAddress::is_valid(raw_email) {
            Ok(Self(raw_email.into()))
        } else {
            Err(EmailAddressError(raw_email.into()))
        }
    }

    pub unsafe fn new_unchecked(raw_email: &str) -> Self {
        Self(raw_email.to_string())
    }
}

#[derive(Debug, PartialEq, Queryable, Identifiable, Selectable, AsChangeset)]
#[diesel(table_name = crate::db::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(treat_none_as_null = true)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub hashed_password: Option<String>,
    pub email: Option<EmailAddress>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::db::schema::users)]
pub struct NewUser {
    pub username: String,
    pub email: Option<EmailAddress>,
    pub hashed_password: Option<String>,
}

pub fn hash_password(password: String) -> Result<String, WebappError> {
    bcrypt::hash(password.trim(), bcrypt::DEFAULT_COST)
        .map_err(|err| return WebappError::BcryptError(err))
}

pub fn verify_password(password: &str, hashed_password: &str) -> Result<bool, WebappError> {
    bcrypt::verify(password, hashed_password).map_err(|err| return WebappError::BcryptError(err))
}

pub fn get_user_by_email(email: &str, conn: &mut PgConnection) -> Option<User> {
    let user = schema::users::table
        .filter(schema::users::email.eq(email))
        .first(conn)
        .optional()
        .unwrap();

    user
}

pub fn get_user_by_username(username: &str, conn: &mut PgConnection) -> Option<User> {
    let user = schema::users::table
        .filter(schema::users::username.eq(username))
        .first(conn)
        .optional()
        .unwrap();

    user
}

pub fn create_new_user(
    new_user: &NewUser,
    conn: &mut PgConnection,
) -> Result<User, diesel::result::Error> {
    diesel::insert_into(users::table)
        .values(new_user)
        .returning(User::as_returning())
        .get_result(conn)
}
