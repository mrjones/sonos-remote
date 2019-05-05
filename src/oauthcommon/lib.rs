extern crate oauth2;

use oauth2::prelude::*;

pub fn make_oauth_client(client_id: &str, client_secret: &str) -> oauth2::basic::BasicClient {
    return oauth2::basic::BasicClient::new(
        oauth2::ClientId::new(client_id.to_string()),
        Some(oauth2::ClientSecret::new(client_secret.to_string())),
        oauth2::AuthUrl::new(url::Url::parse("https://api.sonos.com/login/v3/oauth").expect("auth url")),
        Some(oauth2::TokenUrl::new(url::Url::parse("https://api.sonos.com/login/v3/oauth/access").expect("token url")))
    )
        .add_scope(oauth2::Scope::new("playback-control-all".to_string()))
        .set_redirect_url(oauth2::RedirectUrl::new(url::Url::parse("http://localhost:6060/oauth_redirect").expect("redirect url")));
}
