use std::collections::HashMap;

use axum::extract::{Json, Query};
use axum::response::{Html, IntoResponse};
use chrono::{DateTime, Datelike, Days, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use tracing::debug;

use super::super::WebappError;

use axum::response::Response;

use axum::extract::State;

use axum_extra::extract::PrivateCookieJar;

pub async fn get_calendar(
    jar: PrivateCookieJar,
    State(tera): State<tera::Tera>,
) -> Result<Response, WebappError> {
    let mut context = tera::Context::new();

    if let Some(user) = jar.get("user") {
        debug!("logged in user: {:#?}", user);
        context.insert("user", &user.to_string())
    }
    context.insert("fixedHeight", &true);

    let rendered = tera.render("calendar.html", &context)?;

    Ok(Html(rendered).into_response())
}

pub async fn hx_get_calendar_content(
    jar: PrivateCookieJar,
    State(tera): State<tera::Tera>,
    Query(user_datetime): Query<UserDateTime>,
    // Json(payload): Json<UserDate>,
) -> Result<Response, WebappError> {
    println!("{:#?}", user_datetime);
    // println!("{:#?}", payload);

    let mut context = tera::Context::new();

    let rendered = tera.render("fragments/calendar-content.html", &context)?;

    Ok(Html(rendered).into_response())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDateTime {
    today: DateTime<Utc>,
}

fn calendar_month_start_end_dates(date: &NaiveDate) -> Result<(NaiveDate, NaiveDate), DateError> {
    let month_first = date
        .with_day(1)
        .ok_or_else(|| DateError::UnreachableError)?;

    let prefix_days = month_first.weekday().number_from_sunday() - 1;

    let start_date = month_first
        .checked_sub_days(Days::new(prefix_days.into()))
        .ok_or_else(|| DateError::UnreachableError)?;

    let month_last = date
        .with_day(date.num_days_in_month().into())
        .ok_or_else(|| DateError::UnreachableError)?;

    let suffix_days = 7 - month_last.weekday().number_from_sunday();

    let end_date = month_last
        .checked_add_days(Days::new(suffix_days.into()))
        .ok_or_else(|| DateError::UnreachableError)?;

    Ok((start_date, end_date))
}

#[derive(Debug, thiserror::Error)]
pub enum DateError {
    #[error("This error should be unreachable")]
    UnreachableError,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dates() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        assert_eq!(
            calendar_month_start_end_dates(&date).unwrap(),
            (
                NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
                NaiveDate::from_ymd_opt(2026, 4, 4).unwrap()
            )
        );
    }
}
