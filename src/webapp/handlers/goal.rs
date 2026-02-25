use super::super::WebappError;
use super::super::state::AppState;
use crate::db::models::Goal;
use crate::db::models::NewGoal;
use crate::db::models::User;
use crate::db::models::goal::GoalForm;
use crate::db::schema::users;
use axum::extract::Form;
use axum::extract::State;
use axum::response::Html;
use axum::response::IntoResponse;
use axum::response::Response;
use axum_extra::extract::PrivateCookieJar;
use axum_htmx::HxEvent;
use axum_htmx::HxRequest;
use axum_htmx::HxResponseTrigger;
use axum_htmx::HxTrigger;
use diesel::prelude::*;
use tracing::info;

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

pub async fn get_new_goal(
    jar: PrivateCookieJar,
    State(state): State<AppState>,
    State(tera): State<tera::Tera>,
    HxRequest(hx_request): HxRequest,
) -> Result<Response, WebappError> {
    if hx_request {
        let context = tera::Context::new();
        let rendered = tera.render("goal-form.html", &context)?;

        return Ok(rendered.into_response());
    }
    let mut context = tera::Context::new();
    context.insert("trigger_new", "true");
    let rendered = render_goals(jar, state, tera, &mut context)?;

    Ok(Html(rendered).into_response())
}

pub async fn post_new_goal(
    jar: PrivateCookieJar,
    State(state): State<AppState>,
    State(tera): State<tera::Tera>,
    HxRequest(hx_request): HxRequest,
    Form(goal_form): Form<GoalForm>,
) -> Result<Response, WebappError> {
    if !hx_request {
        return Err(WebappError::HxRequestExpectedError);
    }

    let username = match jar.get("user") {
        Some(user) => user.value().to_string(),
        None => return Err(WebappError::NotLoggedInError),
    };
    let mut conn = state.pool.clone().get()?;
    let user = users::table
        .filter(users::username.eq(&username))
        .first::<User>(&mut conn)?;

    let new_goal = NewGoal {
        title: goal_form.title,
        description: goal_form.description,
        notes: goal_form.notes,
        user_id: user.id,
    };

    // TODO: rename create_new_goal

    let trigger = HxResponseTrigger::normal([HxEvent::new("trigger_close")]);

    Ok((trigger, "").into_response())
}
