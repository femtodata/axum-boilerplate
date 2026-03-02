use super::super::WebappError;
use super::super::state::AppState;
use crate::db::models::Goal;
use crate::db::models::NewGoal;
use crate::db::models::User;
use crate::db::models::goal::GoalForm;
use crate::db::models::goal::create_new_goal;
use crate::db::schema::users;
use axum::extract::Form;
use axum::extract::State;
use axum::response::Html;
use axum::response::IntoResponse;
use axum::response::Response;
use axum_extra::extract::PrivateCookieJar;
use axum_htmx::HxEvent;
use axum_htmx::HxPushUrl;
use axum_htmx::HxRequest;
use axum_htmx::HxResponseTrigger;
use axum_htmx::HxTrigger;
use diesel::prelude::*;
use indoc::formatdoc;
use tracing::info;
use validator::ValidateArgs;
use validator::ValidationErrorsKind;

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

pub async fn hy_get_new_goal(
    jar: PrivateCookieJar,
    State(state): State<AppState>,
    State(tera): State<tera::Tera>,
    HxRequest(hx_request): HxRequest,
) -> Result<Response, WebappError> {
    if hx_request {
        let context = tera::Context::new();
        let rendered = tera.render("fragments/goal-form.html", &context)?;

        return Ok(rendered.into_response());
    }
    let mut context = tera::Context::new();
    context.insert("trigger_new", "true");
    let rendered = render_goals(jar, state, tera, &mut context)?;

    Ok(Html(rendered).into_response())
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

    let goal = create_new_goal(&new_goal, &mut conn)?;

    // don't need to push url, closing modal via trigger handles url history
    let trigger = HxResponseTrigger::normal([HxEvent::new("trigger_close")]);

    Ok((trigger, "").into_response())
}
