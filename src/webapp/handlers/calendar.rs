use axum::Extension;
use axum::extract::{Path, Query};
use axum::response::{Html, IntoResponse};
use chrono::{Datelike, Days, Months, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::db::models::applied_goal::get_applied_goals_for_dates;
use crate::webapp::state::AppState;

use super::super::WebappError;
use super::middleware::UserContext;

use axum::response::Response;

use axum::extract::State;

pub async fn get_calendar_month(
    Extension(user_context): Extension<Option<UserContext>>,
    State(tera): State<tera::Tera>,
) -> Result<Response, WebappError> {
    let mut context = tera::Context::new();

    let Some(user_context) = user_context else {
        return Err(WebappError::NotLoggedInError);
    };
    context.insert("user_context", &user_context);
    context.insert("fixedHeight", &true);

    let rendered = tera.render("calendar.html", &context)?;

    Ok(Html(rendered).into_response())
}

pub async fn get_calendar_month_ymd(
    Extension(user_context): Extension<Option<UserContext>>,
    Path(CalendarParams { year, month, day }): Path<CalendarParams>,
    State(tera): State<tera::Tera>,
) -> Result<Response, WebappError> {
    let mut context = tera::Context::new();

    if let Some(user_context) = user_context {
        context.insert("user_context", &user_context)
    } else {
        return Err(WebappError::NotLoggedInError);
    }

    let calendar_params = CalendarParams { year, month, day };
    context.insert("fixedHeight", &true);
    context.insert("calendar_params", &calendar_params);

    let rendered = tera.render("calendar.html", &context)?;

    Ok(Html(rendered).into_response())
}

pub async fn hx_get_calendar_month_content(
    Extension(user_context): Extension<Option<UserContext>>,
    State(tera): State<tera::Tera>,
    Query(calendar_params): Query<CalendarParams>,
    // Json(payload): Json<UserDate>,
) -> Result<Response, WebappError> {
    let Some(user_context) = user_context else {
        return Err(WebappError::NotLoggedInError);
    };

    let date = NaiveDate::from_ymd_opt(
        calendar_params.year,
        calendar_params.month,
        calendar_params.day,
    )
    .ok_or(WebappError::DateCreationError { calendar_params })?;

    let start_date = date
        .with_day(1)
        .ok_or(WebappError::UnreachableDateError)?
        .week(Weekday::Sun)
        .checked_first_day()
        .ok_or(WebappError::UnreachableDateError)?;

    let end_date = date
        .with_day(1)
        .ok_or(WebappError::UnreachableDateError)?
        .checked_add_months(Months::new(1))
        .ok_or(WebappError::UnreachableDateError)?
        .checked_sub_days(Days::new(1))
        .ok_or(WebappError::UnreachableDateError)?
        .week(Weekday::Sun)
        .checked_last_day()
        .ok_or(WebappError::UnreachableDateError)?;

    let mut last_pushed = start_date;

    let mut date_iter = start_date.iter_days();

    let mut weeks_vec = Vec::new();

    // not at all sure this is the best way to check
    while last_pushed != end_date {
        let mut days_vec = Vec::new();
        for _ in 0..7 {
            days_vec.push(CalendarDay::new(
                date_iter.next().ok_or(WebappError::UnreachableDateError)?,
            ));
        }
        last_pushed = days_vec
            .last()
            .ok_or(WebappError::UnreachableDateError)?
            .date;
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
        .ok_or(WebappError::UnreachableDateError)?
        .checked_add_months(Months::new(1))
        .ok_or(WebappError::UnreachableDateError)?;
    let next_month_params = CalendarParams {
        year: next_month.year(),
        month: next_month.month(),
        day: next_month.day(),
    };
    let prev_month = date
        .with_day(1)
        .ok_or(WebappError::UnreachableDateError)?
        .checked_sub_months(Months::new(1))
        .ok_or(WebappError::UnreachableDateError)?;
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
    pub year: i32,
    pub month: u32,
    pub day: u32,
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
    Extension(user_context): Extension<Option<UserContext>>,
    State(state): State<AppState>,
    State(tera): State<tera::Tera>,
    Query(calendar_params): Query<CalendarParams>,
    // Json(payload): Json<UserDate>,
) -> Result<Response, WebappError> {
    let Some(user_context) = user_context else {
        return Err(WebappError::NotLoggedInError);
    };

    let date = NaiveDate::from_ymd_opt(
        calendar_params.year,
        calendar_params.month,
        calendar_params.day,
    )
    .ok_or(WebappError::DateCreationError { calendar_params })?;

    let start_date = date
        .week(Weekday::Sun)
        .checked_first_day()
        .ok_or(WebappError::UnreachableDateError)?;

    let end_date = date
        .week(Weekday::Sun)
        .checked_last_day()
        .ok_or(WebappError::UnreachableDateError)?;

    // start layering the cake!
    let dates = start_date.iter_days().take(7).collect::<Vec<NaiveDate>>();

    let mut context = tera::Context::new();

    let days_of_week = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    context.insert("days_of_week", &days_of_week);

    let days_vec = dates
        .iter()
        .map(|date| CalendarDay::new(*date))
        .collect::<Vec<CalendarDay>>();

    context.insert("days", &days_vec);

    let mut conn = state.pool.clone().get()?;

    let applied_goal_map =
        get_applied_goals_for_dates(user_context.user_id, None, start_date, end_date, &mut conn)?;

    // we use weeks as the basic unit, instead of days in the month view:
    for (goal, date_map) in applied_goal_map.into_iter() {
        dates.iter().map(|date| match date_map.get(date) {
            Some(applied_goal) => todo!(),
            None => todo!(),
        });
    }

    let month_str = if start_date.month0() == end_date.month0() {
        date.format("%B %Y").to_string()
    } else {
        let first = if start_date.year() == end_date.year() {
            date.format("%B")
        } else {
            date.format("%B %Y")
        };
        format!("{} - {}", first, end_date.format("%B %Y"))
    };

    context.insert("week_string", &month_str);

    let next_week = start_date
        .checked_add_days(Days::new(7))
        .ok_or(WebappError::UnreachableDateError)?;
    let next_week_params = CalendarParams {
        year: next_week.year(),
        month: next_week.month(),
        day: next_week.day(),
    };
    let prev_week = start_date
        .checked_sub_days(Days::new(7))
        .ok_or(WebappError::UnreachableDateError)?;
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
    fn test_calendar_content() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();

        let start_date = date
            .with_day(1)
            .ok_or(WebappError::UnreachableDateError)
            .unwrap()
            .week(Weekday::Sun)
            .checked_first_day()
            .ok_or(WebappError::UnreachableDateError)
            .unwrap();

        let end_date = date
            .with_day(1)
            .ok_or(WebappError::UnreachableDateError)
            .unwrap()
            .checked_add_months(Months::new(1))
            .ok_or(WebappError::UnreachableDateError)
            .unwrap()
            .checked_sub_days(Days::new(1))
            .ok_or(WebappError::UnreachableDateError)
            .unwrap()
            .week(Weekday::Sun)
            .checked_last_day()
            .ok_or(WebappError::UnreachableDateError)
            .unwrap();

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
