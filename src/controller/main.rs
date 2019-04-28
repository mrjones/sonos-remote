extern crate reqwest;

fn main() {
    println!("Hello, world!");

    let access_token = std::env::var("ACCESS_TOKEN").expect("must set ACCESS_TOKEN");
    let client_id = std::env::var("CLIENT_ID").expect("must set CLIENT)ID");

    let http_client = reqwest::Client::new();
    let mut response = http_client
        .get("https://api.ws.sonos.com/control/api/v1/households")
        .bearer_auth(access_token)
        .header(reqwest::header::HeaderName::from_static("x-sonos-api-key"),
                reqwest::header::HeaderValue::from_str(&client_id).expect("header value"))
        .send()
        .unwrap();

    println!("{:?}",  response.text());
}
