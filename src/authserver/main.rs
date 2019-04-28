extern crate oauth2;
extern crate simple_server;
extern crate url;

// ssh -L 127.0.0.1:6060:linode.mrjon.es:6060 linode.mrjon.es

use oauth2::prelude::*;


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
    )
        .add_scope(oauth2::Scope::new("playback-control-all".to_string()))
        .set_redirect_url(oauth2::RedirectUrl::new(url::Url::parse("http://localhost:6060").expect("redirect url")));

    let (auth_url, _csrf_token) = client.authorize_url(oauth2::CsrfToken::new_random);

    println!("Browse to:");
    println!("{}", auth_url);

    let server = simple_server::Server::new(
        move |request, mut response| {
            println!("Request received. {} {}", request.method(), request.uri());
            let url_string = request.uri().to_string();
            let url = url::form_urlencoded::parse(url_string.as_bytes());
            let code: String = url
                .filter(|(k, _)| k == "code")
                .map(|(_, v)| v.to_string())
                .next()
                .expect("no code query param");
            println!("Code {}", code);

            use oauth2::TokenResponse;

            match client.exchange_code(oauth2::AuthorizationCode::new(code)) {
                // I think response is: &oauth2::basic::BasicTokenResponse
                Ok(response) => {
                    assert_eq!(response.token_type(),
                               &oauth2::basic::BasicTokenType::Bearer);
                    let access_token: &oauth2::AccessToken = response.access_token();

                    println!("Token: {}", access_token.secret());
                    println!("Expires in: {:?}", response.expires_in());
                },
                Err(err) => println!("Err: {}", err),
            };

            return Ok(response.body("Hello Rust!".as_bytes().to_vec())?);
        });
    server.listen("linode.mrjon.es", "6060");
}
