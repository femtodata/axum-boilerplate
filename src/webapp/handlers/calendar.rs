use std::collections::HashMap;

use axum::extract::{Json, Query};
use axum::response::{Html, IntoResponse};
use chrono::{DateTime, NaiveDate, Utc};
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
    user_datetime: Query<UserDateTime>,
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

#[cfg(test)]
mod tests {

    #[test]
    fn test_dates() {
        println!("testing dates");
    }
}
