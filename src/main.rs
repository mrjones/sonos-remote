extern crate actix_session;
extern crate actix_web;
extern crate oauth2;
#[macro_use] extern crate serde_derive;
extern crate url;

// ssh -L 127.0.0.1:6060:linode.mrjon.es:6060 linode.mrjon.es

use oauth2::prelude::*;

#[derive(Deserialize)]
struct OauthToken {
    pub code: String
}

fn handler2(token: actix_web::web::Query<OauthToken>) -> String {
    return format!("Code is: {}", token.code);
}

fn main() -> std::io::Result<()> {
    println!("Hello, world!");

    let client_id = std::env::var("CLIENT_ID").expect("must set CLIENT_ID");
    let client_secret = std::env::var("CLIENT_SECRET").expect("must set CLIENT_SECRET");
    println!("ClientID: {}, ClientSecret: {}", client_id, client_secret);

    let client = oauth2::basic::BasicClient::new(
        oauth2::ClientId::new(client_id),
        Some(oauth2::ClientSecret::new(client_secret)),
        oauth2::AuthUrl::new(url::Url::parse("https://api.sonos.com/login/v3/oauth").expect("auth url")),
        Some(oauth2::TokenUrl::new(url::Url::parse("https://api.sonos.com/login/v3/oauth/access").expect("token url")))
    ).set_redirect_url(oauth2::RedirectUrl::new(url::Url::parse("http://localhost:6060").expect("redirect url")));

    let (auth_url, _csrf_token) = client.authorize_url(oauth2::CsrfToken::new_random);

    println!("Browse to:");
    println!("{}", auth_url);

    return actix_web::HttpServer::new(
        || actix_web::App::new().service(
            actix_web::web::resource("/").route(
                actix_web::web::get().to(handler2))))
        .bind("0.0.0.0:6060")?
        .run();
}
