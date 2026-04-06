use axum::{
    Extension,
    extract::State,
    response::{Html, IntoResponse, Response},
};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};

use crate::{
    db::{models::Goal, schema::goals},
    webapp::{WebappError, state::AppState},
};

use super::middleware::UserContext;

pub async fn hx_get_new_applied_goal(
    Extension(user_context): Extension<Option<UserContext>>,
    State(state): State<AppState>,
    State(tera): State<tera::Tera>,
) -> Result<Response, WebappError> {
    let Some(user_context) = user_context else {
        return Err(WebappError::NotLoggedInError);
    };

    let mut context = tera::Context::new();
    let mut conn = state.pool.clone().get()?;
    let goals = goals::table
        .select(Goal::as_select())
        .filter(goals::user_id.eq(user_context.user_id))
        .load(&mut conn)?;
    context.insert("goals", &goals);
    let rendered = tera.render("fragments/applied-goal-form.html", &context)?;

    Ok(Html(rendered).into_response())
}
