use axum::extract::{Path, Query};
use axum::response::{Html, IntoResponse};
use chrono::{DateTime, Datelike, Days, Months, NaiveDate, Utc, Weekday};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::webapp::handlers::calendar;

use super::super::WebappError;

use axum::response::Response;

use axum::extract::State;

use axum_extra::extract::PrivateCookieJar;

pub async fn get_calendar_month(
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

pub async fn get_calendar_month_ymd(
    Path(CalendarParams { year, month, day }): Path<CalendarParams>,
    jar: PrivateCookieJar,
    State(tera): State<tera::Tera>,
) -> Result<Response, WebappError> {
    let mut context = tera::Context::new();

    if let Some(user) = jar.get("user") {
        debug!("logged in user: {:#?}", user);
        context.insert("user", &user.to_string())
    }

    let calendar_params = CalendarParams { year, month, day };
    context.insert("fixedHeight", &true);
    context.insert("calendar_params", &calendar_params);

    let rendered = tera.render("calendar.html", &context)?;

    Ok(Html(rendered).into_response())
}

pub async fn hx_get_calendar_month_content(
    jar: PrivateCookieJar,
    State(tera): State<tera::Tera>,
    Query(calendar_params): Query<CalendarParams>,
    // Json(payload): Json<UserDate>,
) -> Result<Response, WebappError> {
    debug!("{:#?}", calendar_params);

    let date = NaiveDate::from_ymd_opt(
        calendar_params.year,
        calendar_params.month,
        calendar_params.day,
    )
    .ok_or(DateError::UnreachableError)?;

    // let (start_date, end_date) = calendar_month_start_end_dates(&date)?;
    let start_date = date
        .with_day(1)
        .ok_or(DateError::UnreachableError)?
        .week(Weekday::Sun)
        .checked_first_day()
        .ok_or(DateError::UnreachableError)?;

    let end_date = date
        .with_day(1)
        .ok_or(DateError::UnreachableError)?
        .checked_add_months(Months::new(1))
        .ok_or(DateError::UnreachableError)?
        .checked_sub_days(Days::new(1))
        .ok_or(DateError::UnreachableError)?
        .week(Weekday::Sun)
        .checked_last_day()
        .ok_or(DateError::UnreachableError)?;

    let mut last_pushed = start_date;

    let mut date_iter = start_date.iter_days();

    let mut weeks_vec = Vec::new();

    // not at all sure this is the best way to check
    while last_pushed != end_date {
        let mut days_vec = Vec::new();
        for _ in 0..7 {
            days_vec.push(CalendarDay::new(
                date_iter.next().ok_or(DateError::UnreachableError)?,
            ));
        }
        last_pushed = days_vec.last().ok_or(DateError::UnreachableError)?.date;
        weeks_vec.push(days_vec);
    }

    let days_of_week = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

    let mut context = tera::Context::new();
    context.insert("weeks", &weeks_vec);
    context.insert("days_of_week", &days_of_week);

    let month_str = date.format("%B %Y").to_string();
    context.insert("month_string", &month_str);

    let next_month = date
        .with_day(1)
        .ok_or(DateError::UnreachableError)?
        .checked_add_months(Months::new(1))
        .ok_or(DateError::UnreachableError)?;
    let next_month_params = CalendarParams {
        year: next_month.year(),
        month: next_month.month(),
        day: next_month.day(),
    };
    let prev_month = date
        .with_day(1)
        .ok_or(DateError::UnreachableError)?
        .checked_sub_months(Months::new(1))
        .ok_or(DateError::UnreachableError)?;
    let prev_month_params = CalendarParams {
        year: prev_month.year(),
        month: prev_month.month(),
        day: prev_month.day(),
    };

    context.insert("next_month_params", &next_month_params);
    context.insert("prev_month_params", &prev_month_params);

    let rendered = tera.render("fragments/calendar-month-content.html", &context)?;

    Ok(Html(rendered).into_response())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CalendarParams {
    year: i32,
    month: u32,
    day: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum DateError {
    #[error("This date error should be unreachable")]
    UnreachableError,
}

#[derive(Serialize)]
struct CalendarDay {
    date: NaiveDate,
    display_str: String,
}

impl CalendarDay {
    fn new(date: NaiveDate) -> Self {
        Self {
            date,
            display_str: match date.day() {
                1 => date.format("%b %-d").to_string(),
                _ => date.format("%-d").to_string(),
            },
        }
    }
}

pub async fn hx_get_calendar_week_content(
    jar: PrivateCookieJar,
    State(tera): State<tera::Tera>,
    Query(calendar_params): Query<CalendarParams>,
    // Json(payload): Json<UserDate>,
) -> Result<Response, WebappError> {
    let date = NaiveDate::from_ymd_opt(
        calendar_params.year,
        calendar_params.month,
        calendar_params.day,
    )
    .ok_or(DateError::UnreachableError)?;

    let start_date = date
        .week(Weekday::Sun)
        .checked_first_day()
        .ok_or(DateError::UnreachableError)?;

    let days_vec = start_date
        .iter_days()
        .take(7)
        .map(CalendarDay::new)
        .collect::<Vec<CalendarDay>>();

    let days_of_week = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

    let mut context = tera::Context::new();

    context.insert("days", &days_vec);
    context.insert("days_of_week", &days_of_week);

    let month_str = date.format("%B %Y").to_string();
    context.insert("month_string", &month_str);

    let next_week = start_date
        .checked_add_days(Days::new(7))
        .ok_or(DateError::UnreachableError)?;
    let next_week_params = CalendarParams {
        year: next_week.year(),
        month: next_week.month(),
        day: next_week.day(),
    };
    let prev_week = start_date
        .checked_sub_days(Days::new(7))
        .ok_or(DateError::UnreachableError)?;
    let prev_week_params = CalendarParams {
        year: prev_week.year(),
        month: prev_week.month(),
        day: prev_week.day(),
    };

    context.insert("next_week_params", &next_week_params);
    context.insert("prev_week_params", &prev_week_params);

    let rendered = tera.render("fragments/calendar-week-content.html", &context)?;
    Ok(Html(rendered).into_response())
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

    #[test]
    fn test_calendar_content() {
        let today = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();

        let (start_date, end_date) = calendar_month_start_end_dates(&today).unwrap();
        let mut last_pushed = start_date;

        let mut date_iter = start_date.iter_days();

        let mut weeks_vec = Vec::new();

        while last_pushed != end_date {
            let mut days_vec = Vec::new();
            for _ in 0..7 {
                days_vec.push(CalendarDay::new(date_iter.next().unwrap()));
            }
            last_pushed = days_vec.last().unwrap().date;
            weeks_vec.push(days_vec);
        }

        for week in weeks_vec.iter() {
            let day_strings = week
                .iter()
                .map(|day| day.display_str.clone())
                .collect::<Vec<String>>();
            print!("{}", day_strings.join(" | "));
            println!();
        }
    }
}
