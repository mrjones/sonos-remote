extern crate env_logger;
#[macro_use] extern crate log;
extern crate oauth2;
extern crate oauthcommon;
extern crate simple_server;
extern crate url;

// ssh -L 127.0.0.1:6060:linode.mrjon.es:6060 linode.mrjon.es

use oauth2::prelude::*;

fn redirect_handler(request: &simple_server::Request<Vec<u8>>, response: &mut simple_server::ResponseBuilder, client: &oauth2::basic::BasicClient) -> simple_server::ResponseResult {
    let url_string = request.uri().to_string();
    let url = url::form_urlencoded::parse(url_string.as_bytes());
    let code: String = url
        .filter(|(k, _)| k == "code")
        .map(|(_, v)| v.to_string())
        .next()
        .expect(&format!("no code query param: {}", url_string));
    debug!("Code {}", code);

    use oauth2::TokenResponse;

    match client.exchange_code(oauth2::AuthorizationCode::new(code)) {
        // I think response is: &oauth2::basic::BasicTokenResponse
        Ok(response) => {
            assert_eq!(response.token_type(),
                       &oauth2::basic::BasicTokenType::Bearer);
            let access_token: &oauth2::AccessToken = response.access_token();

            let save_state = oauthcommon::OauthTokenState{
                access_token: response.access_token().secret().to_string(),
                refresh_token: response.refresh_token().map(|x| x.secret().to_string()),
                expiration_timestamp: Some(0),
//                expiration_timestamp: response.expires_in().map(
//                    |x| (std::time::SystemTime::now() + x).duration_since(
//                        std::time::SystemTime::UNIX_EPOCH).expect("epoch duration").as_secs()),
            };
            oauthcommon::save_oauth_token_state(&save_state, &"sonostoken".to_string());
            println!("Token: {}", access_token.secret());
            println!("Expires in: {:?}", response.expires_in());
            match response.refresh_token() {
                Some(rt) => println!("Refresh token: {}", rt.secret()),
                None => error!("No refresh token!"),
            }
        },
        Err(err) => error!("Err: {}", err),
    };

    return Ok(response.body("Hello Rust!".as_bytes().to_vec())?);
}

fn main() -> std::io::Result<()> {
    env_logger::init();

    let client_id = std::env::var("CLIENT_ID").expect("must set CLIENT_ID");
    let client_secret = std::env::var("CLIENT_SECRET").expect("must set CLIENT_SECRET");
    info!("ClientID: {}, ClientSecret: {}", client_id, client_secret);
    let client = oauthcommon::make_oauth_client(&client_id, &client_secret);

    let (auth_url, _csrf_token) = client.authorize_url(oauth2::CsrfToken::new_random);

    println!("Browse to:");
    println!("{}", auth_url);

    let server = simple_server::Server::new(
        move |request, mut response| {
            debug!("Request received. {} {}", request.method(), request.uri());
            match request.uri().path() {
                "/oauth_redirect" => redirect_handler(&request, &mut response, &client),
                _ => {
                    response.status(simple_server::StatusCode::NOT_FOUND);
                    Ok(response.body("<h1>404</h1><p>Not found!<p>".as_bytes().to_vec())?)
                }
            }
        });
    server.listen("linode.mrjon.es", "6060");
}
