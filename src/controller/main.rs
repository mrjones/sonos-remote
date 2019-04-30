extern crate reqwest;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;

#[derive(Deserialize)]
struct SonosHousehold {
    pub id: String,
}

#[derive(Deserialize)]
struct SonosHouseholdsReply {
    pub households: Vec<SonosHousehold>,
}

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

    let response_body = response.text().expect("response_body");

    println!("{:?}",  response_body);

    let reply: SonosHouseholdsReply = serde_json::from_str(&response_body).expect("parse json");

    for household in reply.households {
        println!("Household {}", household.id);
    }
}
