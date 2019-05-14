extern crate env_logger;
#[macro_use] extern crate error_chain;
#[macro_use] extern crate log;
extern crate reqwest;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;

use oauth2::prelude::*;

mod result;
mod sonos_api;

fn load_oauth_tokens(oauth_client: &oauth2::basic::BasicClient) -> result::Result<oauthcommon::OauthTokenState> {
    let token_state = oauthcommon::load_oauth_token_state(&"sonostoken".to_string())?;

    use oauth2::TokenResponse;

    let unix_now = std::time::SystemTime::now().duration_since(
        std::time::SystemTime::UNIX_EPOCH)?.as_secs();

    if token_state.refresh_token.is_none() ||
        token_state.expiration_timestamp.is_none() ||
        token_state.expiration_timestamp.unwrap() > unix_now {
            // no need to refresh, or unable to refresh, return what we have
            return Ok(token_state);
        }

    info!("Refreshing OAuth token");
    let rt_string = token_state.refresh_token.unwrap();
    let rt = oauth2::RefreshToken::new(rt_string);
    let response = oauth_client.exchange_refresh_token(&rt).expect("exchanging refresh token failed");
    debug!("Refresh response: {:?}", response);
    info!("Using {} as refreshed access_token",
          response.access_token().secret().to_string());

    let new_token_state = oauthcommon::OauthTokenState{
        access_token: response.access_token().secret().to_string(),
        refresh_token: response.refresh_token().map(|x| x.secret().to_string()),
        expiration_timestamp: response.expires_in().map(
            |x| (std::time::SystemTime::now() + x).duration_since(
                std::time::SystemTime::UNIX_EPOCH).expect("foo").as_secs()),
    };

    oauthcommon::save_oauth_token_state(&new_token_state, &"sonostoken".to_string());

    return Ok(new_token_state);
}

fn print_current_state(api: &sonos_api::Client) -> result::Result<()> {
    let reply = api.get_households()?;
    for household in reply.households {
        println!("Household {}", household.id);
        let groups_reply = api.get_groups(&household.id)?;
        let mut players = std::collections::HashMap::new();
        for player in &groups_reply.players {
            players.insert(&player.id, player);
        }

        for group in groups_reply.groups {
            println!(" - {} {}", group.name, group.id);
            let playback = api.current_playback_state(&group.id)?;
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

    return Ok(());
}

fn main() -> result::Result<()> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let client_id = std::env::var("CLIENT_ID")?;
    let client_secret = std::env::var("CLIENT_SECRET")?;
    let oauth_client = oauthcommon::make_oauth_client(&client_id, &client_secret);
    let oauth_tokens = load_oauth_tokens(&oauth_client)?;
    debug!("Oauth Token State {:?}", oauth_tokens);

    let api = sonos_api::Client::new(&oauth_tokens.access_token, &client_id);

    let args: Vec<String> = std::env::args().collect();

    for arg in args.iter().skip(1) {
        info!("[Handling {}]", arg);
        if arg == "print_state" {
            print_current_state(&api)?;
        } else if arg == "break_group" {
            api.break_group()?;
        } else {
            error!("Unknown arg {}", arg);
        }
    }

    return Ok(());
}
