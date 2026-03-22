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
use tracing::debug;

#[derive(Clone, Serialize)]
pub struct UserContext {
    pub username: String,
    pub user_id: i32,
}

// ensures user_context
pub async fn base_middleware(
    jar: PrivateCookieJar,
    HxRequest(hx_request): HxRequest,
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

// to be used as middleware
pub async fn auth_middleware(
    Extension(user_context): Extension<Option<UserContext>>,
    jar: PrivateCookieJar,
    HxRequest(hx_request): HxRequest,
    mut request: Request,
    next: Next,
) -> Result<Response, WebappError> {
    if let Some(user_context) = user_context {
        Ok(next.run(request).await)
    } else {
        let redirect_url = "/login?next_url=".to_string() + request.uri().to_string().as_str();
        if hx_request {
            return Ok((HxRedirect(redirect_url), "").into_response());
        }
        Ok((StatusCode::FOUND, Redirect::to(redirect_url.as_str())).into_response())
    }
}

// to be used with middleware::from_fn_with_state
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

        return Ok(Redirect::to("/error").into_response());
    } else {
        Ok(response)
    }
}
