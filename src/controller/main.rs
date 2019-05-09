extern crate env_logger;
#[macro_use] extern crate log;
extern crate reqwest;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;

use oauth2::prelude::*;

#[derive(Deserialize)]
struct SonosHousehold {
    pub id: String,
}

#[derive(Deserialize)]
struct SonosHouseholdsReply {
    pub households: Vec<SonosHousehold>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SonosGroup {
    pub id: String,
    pub name: String,
    pub coordinator_id: String,
    pub playback_state: String, // Enum?
    pub player_ids: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SonosPlayer {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize)]
struct SonosGroupsReply {
    pub groups: Vec<SonosGroup>,
    pub players: Vec<SonosPlayer>
}

fn get_households(access_token: &str, client_id: &str, http_client: &reqwest::Client) -> SonosHouseholdsReply {
    let mut response = http_client
        .get("https://api.ws.sonos.com/control/api/v1/households")
        .bearer_auth(access_token)
        .header(reqwest::header::HeaderName::from_static("x-sonos-api-key"),
                reqwest::header::HeaderValue::from_str(&client_id).expect("header value"))
        .send()
        .unwrap();

    let response_body = response.text().expect("response_body");

    debug!("{:?}",  response_body);

    return serde_json::from_str(&response_body).expect("parse json");
}

fn get_groups(household_id: &str, access_token: &str, client_id: &str, http_client: &reqwest::Client) -> SonosGroupsReply {
    let mut response = http_client
        .get(
            &format!("https://api.ws.sonos.com/control/api/v1/households/{}/groups:1", household_id))
        .bearer_auth(access_token)
        .header(reqwest::header::HeaderName::from_static("x-sonos-api-key"),
                reqwest::header::HeaderValue::from_str(&client_id).expect("header value"))
        .send()
        .unwrap();

    let response_body = response.text().expect("response_body");

    debug!("{:?}",  response_body);

    return serde_json::from_str(&response_body).expect("parse json");
}

#[derive(Deserialize)]
struct SonosAlbum {
    pub name: String,
}

#[derive(Deserialize)]
struct SonosArtist {
    pub name: String,
}

#[derive(Deserialize)]
struct SonosTrack {
    pub name: String,
    pub album: SonosAlbum,
    pub artist: SonosArtist,
}

#[derive(Deserialize)]
struct SonosCurrentItem {
    pub track: SonosTrack,
}

#[derive(Deserialize)]
struct SonosPlaybackContainer {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub input_type: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SonosPlaybackMetadata {
    pub container: SonosPlaybackContainer,
    pub current_item: Option<SonosCurrentItem>,
}

fn get_playback_state(group_id: &str, access_token: &str, client_id: &str, http_client: &reqwest::Client) -> SonosPlaybackMetadata {
    let mut response = http_client
        .get(
            &format!("https://api.ws.sonos.com/control/api/v1/groups/{}/playbackMetadata", group_id))
        .bearer_auth(access_token)
        .header(reqwest::header::HeaderName::from_static("x-sonos-api-key"),
                reqwest::header::HeaderValue::from_str(&client_id).expect("header value"))
        .send()
        .unwrap();

    let response_body = response.text().expect("response_body");

    debug!("{:?}",  response_body);

    return serde_json::from_str(&response_body).expect("parse json");
}

fn load_oauth_tokens(oauth_client: &oauth2::basic::BasicClient) -> oauthcommon::OauthTokenState {
    let token_state = oauthcommon::load_oauth_token_state(&"sonostoken".to_string()).expect("load token");

    use oauth2::TokenResponse;

    let unix_now = std::time::SystemTime::now().duration_since(
        std::time::SystemTime::UNIX_EPOCH).expect("epoch duration").as_secs();

    if token_state.refresh_token.is_none() ||
        token_state.expiration_timestamp.is_none() ||
        token_state.expiration_timestamp.unwrap() > unix_now {
            // no need to refresh, or unable to refresh, return what we have
            return token_state;
        }

    info!("Refreshing OAuth token");
    let rt_string = token_state.refresh_token.unwrap();
    let rt = oauth2::RefreshToken::new(rt_string);
    let response = oauth_client.exchange_refresh_token(&rt)
        .expect("refreshing token");
    debug!("Refresh response: {:?}", response);
    info!("Using {} as refreshed access_token",
          response.access_token().secret().to_string());

    let new_token_state = oauthcommon::OauthTokenState{
        access_token: response.access_token().secret().to_string(),
        refresh_token: response.refresh_token().map(|x| x.secret().to_string()),
        expiration_timestamp: response.expires_in().map(
            |x| (std::time::SystemTime::now() + x).duration_since(
                std::time::SystemTime::UNIX_EPOCH).expect("duration since").as_secs()),
    };

    oauthcommon::save_oauth_token_state(&new_token_state, &"sonostoken".to_string());

    return new_token_state;
}

fn main() {
    env_logger::init();

//    let mut access_token = std::env::var("ACCESS_TOKEN").expect("must set ACCESS_TOKEN");
    let client_id = std::env::var("CLIENT_ID").expect("must set CLIENT_ID");
    let client_secret = std::env::var("CLIENT_SECRET").expect("must set CLIENT_SECRET");

    let oauth_client = oauthcommon::make_oauth_client(&client_id, &client_secret);

    let oauth_tokens = load_oauth_tokens(&oauth_client);
    debug!("Oauth Token State {:?}", oauth_tokens);

    let http_client = reqwest::Client::new();
    {
        let reply = get_households(&oauth_tokens.access_token, &client_id, &http_client);
        for household in reply.households {
            println!("Household {}", household.id);
            let groups_reply = get_groups(&household.id, &oauth_tokens.access_token, &client_id, &http_client);
            let mut players = std::collections::HashMap::new();
            for player in &groups_reply.players {
                players.insert(&player.id, player);
            }

            for group in groups_reply.groups {
                println!(" - {} {}", group.name, group.id);
                let playback = get_playback_state(&group.id, &oauth_tokens.access_token, &client_id, &http_client);
                match playback.current_item {
                    Some(current_item) => println!("   {} - {}", current_item.track.name, current_item.track.artist.name),
                    None => println!("   {}", playback.container.input_type.unwrap_or("UNKOWN".to_string())),
                }

                if group.player_ids.len() > 1 {
                    for player_id in group.player_ids {
                        println!("    - {} {}", players.get(&player_id).map(|p| p.name.clone()).unwrap_or("UNKNOWN PLAYER".to_string()), player_id);
                    }
                }
            }
        }
    }
}
