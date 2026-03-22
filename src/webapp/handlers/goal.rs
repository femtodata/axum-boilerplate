use super::{
    super::{WebappError, state::AppState},
    middleware::UserContext,
};
use crate::db::{
    models::{
        Goal, NewGoal, User,
        goal::{GoalContext, GoalForm, create_new_goal},
    },
    schema::{goals, users},
};
use axum::{
    Extension,
    extract::{Form, Path, State},
    response::{Html, IntoResponse, Response},
};
use axum_extra::extract::PrivateCookieJar;
use axum_htmx::{HxEvent, HxResponseTrigger};
use diesel::prelude::*;
use indoc::formatdoc;
use tracing::debug;
use validator::{ValidateArgs, ValidationErrorsKind};

pub async fn get_goals(
    Extension(user_context): Extension<Option<UserContext>>,
    State(state): State<AppState>,
    State(tera): State<tera::Tera>,
) -> Result<Response, WebappError> {
    let mut context = tera::Context::new();
    let Some(user_context) = user_context else {
        return Err(WebappError::NotLoggedInError);
    };
    context.insert("user_context", &user_context);

    let mut conn = state.pool.clone().get()?;
    let goals = goals::table
        .select(Goal::as_select())
        .filter(goals::user_id.eq(user_context.user_id))
        .load(&mut conn)?;

    context.insert("goals", &goals);
    context.insert("title", "axum-boilerplate | Goals");
    context.insert("active", "goals");
    let rendered = tera.render("goals.html", &context)?;

    Ok(Html(rendered).into_response())
}

pub async fn hx_get_goals_table(
    jar: PrivateCookieJar,
    State(state): State<AppState>,
    State(tera): State<tera::Tera>,
) -> Result<Response, WebappError> {
    let username = match jar.get("username") {
        Some(username) => username.value().to_string(),
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

pub async fn hx_get_new_goal(State(tera): State<tera::Tera>) -> Result<Response, WebappError> {
    let context = tera::Context::new();
    let rendered = tera.render("fragments/goal-form.html", &context)?;

    Ok(Html(rendered).into_response())
}

pub async fn hx_post_new_goal(
    jar: PrivateCookieJar,
    State(state): State<AppState>,
    Form(goal_form): Form<GoalForm>,
) -> Result<Response, WebappError> {
    let username = match jar.get("username") {
        Some(username) => username.value().to_string(),
        None => return Err(WebappError::NotLoggedInError),
    };
    let mut conn = state.pool.clone().get()?;
    let user = users::table
        .filter(users::username.eq(&username))
        .first::<User>(&mut conn)?;

    let mut context = GoalContext {
        conn: &mut conn,
        current_title: None,
    };
    let alert = validate_goal_form_extract_alert(&goal_form, &mut context);

    if let Some(alert) = alert {
        return Ok(Html(alert).into_response());
    }

    let new_goal = NewGoal {
        title: goal_form.title,
        description: goal_form.description,
        notes: goal_form.notes,
        user_id: user.id,
    };

    let _goal = create_new_goal(&new_goal, &mut conn)?;

    // don't need to push url, closing modal via trigger handles url history
    let trigger = HxResponseTrigger::normal([
        HxEvent::new("trigger_close_modal"),
        HxEvent::new("trigger_table_reload"),
    ]);

    Ok((trigger, "").into_response())
}

fn validate_goal_form_extract_alert<'a>(
    goal_form: &GoalForm,
    context: &'a mut GoalContext<'a>,
) -> Option<String> {
    // validate form, see GoalForm impl
    let validation_result = goal_form.validate_with_args(context);
    let validation_error_messages = validation_result.err().map(|errors| {
        errors
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
            .collect::<Vec<_>>()
    });

    // if errors, pull out messages and return as bullet list fragment

    validation_error_messages.map(|messages| {
        formatdoc!(
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
        )
    })
}

pub async fn hx_get_goal(
    Extension(user_context): Extension<Option<UserContext>>,
    Path(id): Path<i32>,
    State(state): State<AppState>,
    State(tera): State<tera::Tera>,
) -> Result<Response, WebappError> {
    debug!("getting goal with id {}", id);

    let mut context = tera::Context::new();

    let Some(user_context) = user_context else {
        return Err(WebappError::NotLoggedInError);
    };

    let mut conn = state.pool.clone().get()?;

    let goal = goals::table
        .filter(
            goals::user_id
                .eq(user_context.user_id)
                .and(goals::id.eq(id)),
        )
        .first::<Goal>(&mut conn)?;
    debug!("goal: {:#?}", goal);

    context.insert("goal", &goal);
    let rendered = tera.render("fragments/goal-detail.html", &context)?;

    Ok(Html(rendered).into_response())
}

pub async fn hx_delete_goal(
    Extension(user_context): Extension<Option<UserContext>>,
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> Result<Response, WebappError> {
    debug!("getting goal with id {}", id);
    let Some(user_context) = user_context else {
        return Err(WebappError::NotLoggedInError);
    };
    let mut conn = state.pool.clone().get()?;

    let res = diesel::delete(
        goals::table.filter(
            goals::id
                .eq(id)
                .and(goals::user_id.eq(user_context.user_id)),
        ),
    )
    .execute(&mut conn)?;

    if res == 0 {
        return Err(WebappError::DieselResultError(
            diesel::result::Error::NotFound,
        ));
    }

    // don't need to push url, closing modal via trigger handles url history
    // ignore above, not pushing url for modal actions
    let trigger = HxResponseTrigger::normal([
        HxEvent::new("trigger_close_modal"),
        HxEvent::new("trigger_table_reload"),
    ]);

    Ok((trigger, "").into_response())
}

pub async fn hx_get_edit_goal(
    Extension(user_context): Extension<Option<UserContext>>,
    Path(id): Path<i32>,
    State(state): State<AppState>,
    State(tera): State<tera::Tera>,
) -> Result<Response, WebappError> {
    let Some(user_context) = user_context else {
        return Err(WebappError::NotLoggedInError);
    };
    let mut conn = state.pool.clone().get()?;

    let goal = goals::table
        .filter(
            goals::id
                .eq(id)
                .and(goals::user_id.eq(user_context.user_id)),
        )
        .first::<Goal>(&mut conn)?;

    let mut context = tera::Context::new();
    context.insert("goal", &goal);
    context.insert("edit", &true);
    let rendered = tera.render("fragments/goal-form.html", &context)?;

    Ok(Html(rendered).into_response())
}

pub async fn hx_patch_goal(
    Extension(user_context): Extension<Option<UserContext>>,
    Path(id): Path<i32>,
    State(state): State<AppState>,
    Form(goal_form): Form<GoalForm>,
) -> Result<Response, WebappError> {
    let Some(user_context) = user_context else {
        return Err(WebappError::NotLoggedInError);
    };
    let mut conn = state.pool.clone().get()?;
    let goal = goals::table
        .filter(
            goals::user_id
                .eq(user_context.user_id)
                .and(goals::id.eq(id)),
        )
        .first::<Goal>(&mut conn)?;
    debug!("goal: {:#?}", goal);

    let mut context = GoalContext {
        conn: &mut conn,
        current_title: Some(&goal.title),
    };
    let alert = validate_goal_form_extract_alert(&goal_form, &mut context);

    if let Some(alert) = alert {
        return Ok(Html(alert).into_response());
    }

    let _ = diesel::update(&goal).set(&goal_form).execute(&mut conn)?;

    // don't need to push url, closing modal via trigger handles url history
    // ignore above, not pushing url for modal actions
    let trigger = HxResponseTrigger::normal([
        HxEvent::new("trigger_close_modal"),
        HxEvent::new("trigger_table_reload"),
    ]);

    Ok((trigger, "").into_response())
}
