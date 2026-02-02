use std::{collections::HashMap, env, sync::Arc};

use axum::{
    Router,
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post},
};
use axum_extra::extract::cookie::Key;
use minijinja::Environment;
use rand::distr::{Alphanumeric, SampleString};
use state::{AppState, InnerState};
use tokio::net::TcpListener;
use tracing::info;

use crate::get_config;

mod handlers;
mod sso;
pub mod state;

#[derive(Debug, thiserror::Error)]
pub enum WebappError {
    // template errors
    // ---------------
    #[error(transparent)]
    MinijinjaError(#[from] minijinja::Error),

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
        (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response()
    }
}

pub async fn run_server() {
    tracing_subscriber::fmt::init();

    get_config();

    let env = add_templates();
    let secret = env::var("SECRET").unwrap_or_else(|_| {
        info!("no secret in env, generating...");
        Alphanumeric.sample_string(&mut rand::rng(), 64)
    });
    let key = Key::from(secret.as_bytes());

    let oauth_client_map = HashMap::from([
        (
            "microsoft".to_string(),
            sso::microsoft_sso::oauth_client().await.unwrap(),
        ),
        (
            "google".to_string(),
            sso::google_sso::oauth_client().await.unwrap(),
        ),
    ]);

    let app_state = AppState(Arc::new(InnerState {
        env,
        oauth_client_map,
        key,
    }));

    let app = Router::new()
        .route("/", get(handlers::get_index))
        // .route_layer(middleware::from_fn_with_state(
        //     app_state.clone(),
        //     handlers::check_auth,
        // ))
        .route("/login", get(handlers::get_login))
        .route("/login", post(handlers::post_login))
        .route("/logout", get(handlers::get_logout))
        .merge(sso::sso_router())
        .with_state(app_state);

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

fn add_templates<'a>() -> Environment<'a> {
    let mut env = Environment::new();

    env.add_template("login", include_str!("./templates/login.html"))
        .unwrap();
    env.add_template("layout", include_str!("./templates/layout.html"))
        .unwrap();
    env.add_template("home", include_str!("./templates/home.html"))
        .unwrap();

    env
}
