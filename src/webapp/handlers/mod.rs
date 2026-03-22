use crate::db::models::user::{get_user_by_username, verify_password};
use axum::{
    Extension,
    extract::{Form, Query, State},
    http::HeaderMap,
    middleware::Next,
    response::{Html, IntoResponse, Redirect, Response},
};
use axum_extra::extract::{PrivateCookieJar, cookie::Cookie};
use middleware::UserContext;
use serde::Deserialize;
use std::str::FromStr;
use tracing::debug;
use url::Url;
use validator::{Validate, ValidationErrorsKind};

pub mod calendar;
pub mod goal;
pub mod middleware;

use super::{WebappError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct Params {
    next_url: Option<String>,
    alert: Option<bool>,
}

pub async fn get_login(
    Extension(user_context): Extension<Option<UserContext>>,
    jar: PrivateCookieJar,
    State(state): State<AppState>,
) -> Result<(PrivateCookieJar, Response), WebappError> {
    // you only get here if you manually go to url, so we don't worry about query params / next
    if user_context.is_some() {
        return Ok((jar, Redirect::to("/").into_response()));
    }

    Ok((jar, render_login_with_context(state, tera::Context::new())?))
}

#[derive(Deserialize, Debug, Validate)]
pub struct LoginPayload {
    #[validate(length(min = 1, message = "Username cannot be blank."))]
    username: String,

    #[validate(length(min = 1, message = "Password cannot be blank"))]
    password: String,
}

pub async fn post_login(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    headers: HeaderMap,
    Form(login_payload): Form<LoginPayload>,
) -> Result<(PrivateCookieJar, Response), WebappError> {
    let validation = login_payload.validate();

    // if validation errors, render login with messages in alert
    if let Err(validation_errors) = validation {
        let errors = validation_errors.errors();
        let validation_messages: Vec<_> = errors
            .values()
            .filter_map(|x| match x {
                ValidationErrorsKind::Field(validation_errors) => Some(validation_errors),
                _ => None,
            })
            .flatten()
            .filter_map(|x| x.message.clone())
            .collect();
        let message = validation_messages.join("<br>");
        let mut context = tera::Context::new();
        context.insert("alert", &message);
        return Ok((jar, render_login_with_context(state, context)?));
    }

    let mut conn = state.pool.clone().get()?;

    if let Some(user) = get_user_by_username(&login_payload.username, &mut conn) {
        // empty password means no password login
        if let Some(hashed_password) = user.hashed_password
            && verify_password(&login_payload.password, &hashed_password)
                .ok()
                .unwrap_or(false)
        {
            let updated_jar = jar
                .add(Cookie::build(("username", user.username)).path("/"))
                .add(Cookie::build(("user_id", user.id.to_string())).path("/"));

            // get next_url from REFERER header
            let next_url = get_next_url_from_headers(headers);

            return Ok((updated_jar, Redirect::to(next_url.as_str()).into_response()));
        }
    };

    let mut context = tera::Context::new();
    context.insert("alert", "Wrong username or password");
    Ok((jar, render_login_with_context(state, context)?))
}

pub fn get_next_url_from_headers(headers: HeaderMap) -> String {
    headers
        .get("REFERER")
        .and_then(|x| x.to_str().ok())
        .and_then(|x| Url::from_str(x).ok())
        .and_then(|referer_url| {
            referer_url
                .query_pairs()
                .find_map(|(k, v)| (k == "next_url").then(|| v.into_owned()))
        })
        .unwrap_or_else(|| "/".to_string())
}

pub fn render_login_with_context(
    state: AppState,
    context: tera::Context,
) -> Result<Response, tera::Error> {
    let rendered = state.tera.render("login.html", &context)?;

    Ok(Html(rendered).into_response())
}

pub async fn get_logout(
    jar: PrivateCookieJar,
) -> Result<(PrivateCookieJar, Response), WebappError> {
    let updated_jar = jar.remove(Cookie::from("username"));
    Ok((updated_jar, Redirect::to("/").into_response()))
}

pub async fn get_index(
    Extension(user_context): Extension<Option<UserContext>>,
    jar: PrivateCookieJar,
    State(tera): State<tera::Tera>,
) -> Result<Html<String>, WebappError> {
    let mut context = tera::Context::new();

    if let Some(user_context) = user_context {
        context.insert("user_context", &user_context)
    }

    context.insert("content", "Home Content");

    let rendered = tera.render("home.html", &context)?;

    Ok(Html(rendered))
}

pub async fn get_error_page(State(tera): State<tera::Tera>) -> Result<Response, WebappError> {
    let mut context = tera::Context::new();
    context.insert(
        "content",
        "Unfortunately, we've encountered an error. Please try again.",
    );

    let rendered = tera.render("error.html", &context)?;
    Ok(Html(rendered).into_response())
}

pub async fn get_test_error_page() -> Result<Response, WebappError> {
    Err(WebappError::TestError)
}
