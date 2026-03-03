use super::OauthClient;
use crate::webapp::WebappError;
use openidconnect::{
    ClientId, ClientSecret, IssuerUrl, RedirectUrl,
    core::{CoreClient, CoreProviderMetadata},
    reqwest,
};
use std::env;

pub async fn oauth_client() -> Result<OauthClient, WebappError> {
    let client_id = ClientId::new(
        env::var("GOOGLE_CLIENT_ID").expect("Missing the GOOGLE_CLIENT_ID environment variable."),
    );
    let client_secret = ClientSecret::new(
        env::var("GOOGLE_CLIENT_SECRET")
            .expect("Missing the GOOGLE_CLIENT_SECRET environment variable."),
    );
    // let tenant_id = env::var("GOOGLE_TENANT_ID").expect("Missing GOOGLE_TENANT_ID");
    let redirect_url = env::var("GOOGLE_REDIRECT_URL").expect("Missing GOOGLE_REDIRECT_URL");
    let http_client = reqwest::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("HTTP Client should build");
    let provider_metadata = CoreProviderMetadata::discover_async(
        IssuerUrl::new(format!("https://accounts.google.com"))?,
        &http_client,
    )
    .await?;
    let client =
        CoreClient::from_provider_metadata(provider_metadata, client_id, Some(client_secret))
            .set_redirect_uri(RedirectUrl::new(redirect_url)?);

    Ok(client)
}
