use super::super::{WebappError, state::AppState};
use crate::db::{
    models::{
        Goal, NewGoal, User,
        goal::{GoalForm, create_new_goal},
    },
    schema::{goals, users},
};
use axum::{
    extract::{Form, Path, State},
    response::{Html, IntoResponse, Response},
};
use axum_extra::extract::PrivateCookieJar;
use axum_htmx::{HxEvent, HxRequest, HxResponseTrigger};
use diesel::prelude::*;
use indoc::formatdoc;
use tracing::{debug, info};
use validator::{ValidateArgs, ValidationErrorsKind};

pub async fn get_goals(
    jar: PrivateCookieJar,
    State(state): State<AppState>,
    State(tera): State<tera::Tera>,
) -> Result<Response, WebappError> {
    let mut context = tera::Context::new();
    let rendered = render_goals(jar, state, tera, &mut context)?;

    Ok(Html(rendered).into_response())
}

fn render_goals(
    jar: PrivateCookieJar,
    state: AppState,
    tera: tera::Tera,
    context: &mut tera::Context,
) -> Result<String, WebappError> {
    let username = match jar.get("user") {
        Some(user) => user.value().to_string(),
        None => return Err(WebappError::NotLoggedInError),
    };
    let mut conn = state.pool.clone().get()?;
    let user = users::table
        .filter(users::username.eq(&username))
        .first::<User>(&mut conn)?;
    let goals = Goal::belonging_to(&user).load::<Goal>(&mut conn)?;
    context.insert("user", &username);
    context.insert("title", "axum-boilerplate | Goals");
    context.insert("goals", &goals);
    context.insert("active", "goals");
    let rendered = tera.render("goals.html", &context)?;
    Ok(rendered)
}

pub async fn hx_get_goals_table(
    jar: PrivateCookieJar,
    State(state): State<AppState>,
    State(tera): State<tera::Tera>,
) -> Result<Response, WebappError> {
    let username = match jar.get("user") {
        Some(user) => user.value().to_string(),
        None => return Err(WebappError::NotLoggedInError),
    };
    let mut conn = state.pool.clone().get()?;
    let user = users::table
        .filter(users::username.eq(&username))
        .first::<User>(&mut conn)?;
    let goals = Goal::belonging_to(&user).load::<Goal>(&mut conn)?;

    let mut context = tera::Context::new();
    context.insert("goals", &goals);
    let rendered = tera.render("fragments/goals-table.html", &context)?;

    Ok(Html(rendered).into_response())
}

pub async fn hx_get_new_goal(
    jar: PrivateCookieJar,
    State(state): State<AppState>,
    State(tera): State<tera::Tera>,
    HxRequest(hx_request): HxRequest,
) -> Result<Response, WebappError> {
    let context = tera::Context::new();
    let rendered = tera.render("fragments/goal-form.html", &context)?;

    return Ok(Html(rendered).into_response());
}

pub async fn hx_post_new_goal(
    jar: PrivateCookieJar,
    State(state): State<AppState>,
    State(tera): State<tera::Tera>,
    HxRequest(hx_request): HxRequest,
    Form(goal_form): Form<GoalForm>,
) -> Result<Response, WebappError> {
    let username = match jar.get("user") {
        Some(user) => user.value().to_string(),
        None => return Err(WebappError::NotLoggedInError),
    };
    let mut conn = state.pool.clone().get()?;
    let user = users::table
        .filter(users::username.eq(&username))
        .first::<User>(&mut conn)?;

    // validate form, see GoalForm impl
    let validation_result = goal_form.validate_with_args(&mut conn);
    let validation_error_messages = validation_result.err().and_then(|errors| {
        let es = errors
            .0 // inner HashMap
            .into_iter()
            .filter_map(|(_k, v)| match v {
                // only want the Field types
                ValidationErrorsKind::Field(validation_errors) => Some(validation_errors),
                _ => None,
            })
            .flatten() // because fields can have multiple errors
            .filter_map(|validation_error| validation_error.message)
            .map(|message| message.to_string())
            .collect::<Vec<_>>();
        Some(es)
    });

    // if errors, pull out messages and return as bullet list fragment
    if let Some(messages) = validation_error_messages {
        let alert = formatdoc!(
            "
            <div id='alert'
                hx-swap-oob='true'
                class='alert alert-danger'
                role='alert'>
                <ul class='mb-0'>
                    {}
                </ul
            </div>
            ",
            messages
                .iter()
                .map(|x| format!("<li>{x}</li>"))
                .collect::<Vec<_>>()
                .join("")
        );
        return Ok(Html(alert).into_response());
    };

    let new_goal = NewGoal {
        title: goal_form.title,
        description: goal_form.description,
        notes: goal_form.notes,
        user_id: user.id,
    };

    let _goal = create_new_goal(&new_goal, &mut conn)?;

    // don't need to push url, closing modal via trigger handles url history
    let trigger = HxResponseTrigger::normal([
        HxEvent::new("trigger_close"),
        HxEvent::new("trigger_table_reload"),
    ]);

    Ok((trigger, "").into_response())
}

pub async fn hx_get_goal(
    Path(id): Path<i32>,
    State(state): State<AppState>,
    State(tera): State<tera::Tera>,
    jar: PrivateCookieJar,
) -> Result<Response, WebappError> {
    debug!("getting goal with id {}", id);
    let username = match jar.get("user") {
        Some(user) => user.value().to_string(),
        None => return Err(WebappError::NotLoggedInError),
    };
    let mut conn = state.pool.clone().get()?;
    let user = users::table
        .filter(users::username.eq(&username))
        .first::<User>(&mut conn)?;

    let goal = goals::table
        .filter(goals::user_id.eq(user.id).and(goals::id.eq(id)))
        .first::<Goal>(&mut conn)?;
    debug!("goal: {:#?}", goal);

    let mut context = tera::Context::new();
    context.insert("goal", &goal);
    let rendered = tera.render("fragments/goal-detail.html", &context)?;

    Ok(Html(rendered).into_response())
}
