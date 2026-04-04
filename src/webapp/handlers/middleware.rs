use super::super::WebappError;
use axum::{
    Extension,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::PrivateCookieJar;
use axum_htmx::{HxRedirect, HxRequest};
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct UserContext {
    pub username: String,
    pub user_id: i32,
}

/// Inserts user_context extension into request
pub async fn base_middleware(
    jar: PrivateCookieJar,
    mut request: Request,
    next: Next,
) -> Result<Response, WebappError> {
    let mut user_context = None;
    if let Some(username_cookie) = jar.get("username")
        && let Some(user_id_cookie) = jar.get("user_id")
    {
        let Ok(user_id) = user_id_cookie.value().parse::<i32>() else {
            return Err(WebappError::UserIDParseError);
        };
        user_context = Some(UserContext {
            username: username_cookie.value().to_string(),
            user_id,
        });
    }
    request.extensions_mut().insert(user_context);
    Ok(next.run(request).await)
}

/// Checks user_context extension, redirects to login if None
pub async fn auth_middleware(
    Extension(user_context): Extension<Option<UserContext>>,
    HxRequest(hx_request): HxRequest,
    request: Request,
    next: Next,
) -> Result<Response, WebappError> {
    if user_context.is_some() {
        Ok(next.run(request).await)
    } else {
        let redirect_url = "/login?next_url=".to_string() + request.uri().to_string().as_str();
        if hx_request {
            return Ok((HxRedirect(redirect_url), "").into_response());
        }
        Ok((StatusCode::FOUND, Redirect::to(redirect_url.as_str())).into_response())
    }
}

/// Prints error, display error page for user
pub async fn error_middleware(
    HxRequest(hx_request): HxRequest,
    request: Request,
    next: Next,
) -> Result<Response, WebappError> {
    let response = next.run(request).await;

    let status_code = response.status();

    if status_code.is_server_error() || status_code.is_client_error() {
        tracing::error!("{:#?}", response);

        if hx_request {
            return Ok((status_code, HxRedirect("/error".to_string()), "").into_response());
        }

        Ok(Redirect::to("/error").into_response())
    } else {
        Ok(response)
    }
}
