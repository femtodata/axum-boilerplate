use super::super::WebappError;
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::PrivateCookieJar;
use axum_htmx::{HxRedirect, HxRequest};
use tracing::debug;

// to be used as middleware
pub async fn auth_middleware(
    jar: PrivateCookieJar,
    HxRequest(hx_request): HxRequest,
    request: Request,
    next: Next,
) -> Result<Response, WebappError> {
    if let Some(user) = jar.get("user") {
        debug!("logged in user: {}", user);
    } else {
        let redirect_url = "/login?next_url=".to_string() + request.uri().to_string().as_str();
        if hx_request {
            return Ok((HxRedirect(redirect_url), "").into_response());
        }
        return Ok((StatusCode::FOUND, Redirect::to(redirect_url.as_str())).into_response());
    }
    let response = next.run(request).await;

    Ok(response)
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
