use axum::extract::Query;
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
    debug!("{:#?}", user_datetime);

    let today = user_datetime.user_utc.date_naive();

    let (start_date, end_date) = calendar_month_start_end_dates(&today)?;

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

    let mut context = tera::Context::new();
    context.insert("weeks", &weeks_vec);

    let rendered = tera.render("fragments/calendar-content.html", &context)?;

    Ok(Html(rendered).into_response())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDateTime {
    user_utc: DateTime<Utc>,
}

fn calendar_month_start_end_dates(date: &NaiveDate) -> Result<(NaiveDate, NaiveDate), DateError> {
    let month_first = date.with_day(1).ok_or(DateError::UnreachableError)?;

    let prefix_days = month_first.weekday().number_from_sunday() - 1;

    let start_date = month_first
        .checked_sub_days(Days::new(prefix_days.into()))
        .ok_or(DateError::UnreachableError)?;

    let month_last = date
        .with_day(date.num_days_in_month().into())
        .ok_or(DateError::UnreachableError)?;

    let suffix_days = 7 - month_last.weekday().number_from_sunday();

    let end_date = month_last
        .checked_add_days(Days::new(suffix_days.into()))
        .ok_or(DateError::UnreachableError)?;

    Ok((start_date, end_date))
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
