#[macro_use] extern crate serde_derive;
extern crate oauth2;
extern crate serde_json;

use oauth2::prelude::*;

#[derive(Deserialize, Serialize)]
pub struct OauthTokenState {
    pub access_token: String,
    pub refresh_token: Option<String>,
}

pub fn save_oauth_token_state<P: AsRef<std::path::Path>>(state: &OauthTokenState, filename: &P) {
    use std::io::Write;

    let contents = serde_json::to_string(state).expect("save token to string");
    let mut file = std::fs::File::create(filename).expect("creating token savefile");
    file.write_all(contents.as_bytes()).expect("writing token savefile");
}

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
