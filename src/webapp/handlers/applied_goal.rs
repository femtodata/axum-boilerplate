use axum::{
    extract::State,
    response::{Html, IntoResponse, Response},
};

use crate::webapp::WebappError;

pub async fn hx_get_new_applied_goal(
    State(tera): State<tera::Tera>,
) -> Result<Response, WebappError> {
    let context = tera::Context::new();
    let rendered = tera.render("fragments/applied-goal-form.html", &context)?;

    Ok(Html(rendered).into_response())
}
