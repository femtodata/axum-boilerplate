use std::{collections::HashMap, env, sync::Arc};

use axum::{
    Router,
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post},
};
use axum_extra::extract::cookie::Key;
use rand::distr::{Alphanumeric, SampleString};
use state::{AppState, InnerState};
use tera::Tera;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::{db::get_connection_pool, get_config};

mod handlers;
mod sso;
pub mod state;

#[derive(Debug, thiserror::Error)]
pub enum WebappError {
    // template errors
    // ---------------
    #[error(transparent)]
    TeraError(#[from] tera::Error),

    // sso errors
    // ----------
    #[error(transparent)]
    ParseError(#[from] url::ParseError),

    #[error(transparent)]
    ConfigurationError(#[from] openidconnect::ConfigurationError),

    #[error(transparent)]
    DiscoveryError(
        #[from]
        openidconnect::DiscoveryError<
            openidconnect::HttpClientError<openidconnect::reqwest::Error>,
        >,
    ),

    #[error(transparent)]
    RequestTokenError(
        #[from]
        openidconnect::RequestTokenError<
            openidconnect::HttpClientError<openidconnect::reqwest::Error>,
            openidconnect::StandardErrorResponse<openidconnect::core::CoreErrorResponseType>,
        >,
    ),

    #[error("missing oauth client")]
    MissingOauthClientError,

    #[error(transparent)]
    ClaimsVerificationError(#[from] openidconnect::ClaimsVerificationError),

    #[error("no id_token in token_response")]
    MissingIdToken,

    #[error("no email in id_token")]
    MissingEmailError,

    #[error("No user with that email found")]
    NoMatchingUserError,

    #[error(transparent)]
    R2d2Error(#[from] diesel::r2d2::PoolError),

    #[error(transparent)]
    BcryptError(#[from] bcrypt::BcryptError),

    #[error("Not logged in")]
    NotLoggedInError,
    // #[error(transparent)]
    // PolarsError(#[from] polars::prelude::PolarsError),
    //#[error(transparent)]
    //DatarrameError(#[from] DataFrameError),
    // #[error(transparent)]
    // Error(#[from] Box<dyn std::error::Error>),
}

impl IntoResponse for WebappError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("WebappError: {:#?}", self);
        println!("WebappError: {:#?}", self);
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{:#?}", self)).into_response()
    }
}

pub async fn run_server() {
    tracing_subscriber::fmt::init();

    get_config();

    let tera = match Tera::new("src/webapp/templates/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };

    let secret = env::var("SECRET").unwrap_or_else(|_| {
        info!("no secret in env, generating...");
        Alphanumeric.sample_string(&mut rand::rng(), 64)
    });

    let key = Key::from(secret.as_bytes());

    let pool = get_connection_pool();

    let app_state = AppState(Arc::new(InnerState { tera, key, pool }));

    let app = Router::new()
        .route("/goals", get(handlers::get_goals))
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            handlers::check_auth,
        ))
        .route("/", get(handlers::get_index))
        .route("/login", get(handlers::get_login))
        .route("/login", post(handlers::post_login))
        .route("/logout", get(handlers::get_logout))
        .merge(sso::sso_router())
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(middleware::from_fn_with_state(
                    app_state.clone(),
                    handlers::error_page,
                )),
        )
        .with_state(app_state);

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

// fn add_templates<'a>() -> Environment<'a> {
fn add_templates<'a>() {
    todo!()
}
