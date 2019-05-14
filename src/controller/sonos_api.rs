extern crate reqwest;
extern crate std;

#[derive(Deserialize)]
pub struct SonosHousehold {
    pub id: String,
}

#[derive(Deserialize)]
pub struct SonosHouseholdsReply {
    pub households: Vec<SonosHousehold>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SonosGroup {
    pub id: String,
    pub name: String,
    pub coordinator_id: String,
    pub playback_state: String, // Enum?
    pub player_ids: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SonosPlayer {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize)]
pub struct SonosGroupsReply {
    pub groups: Vec<SonosGroup>,
    pub players: Vec<SonosPlayer>
}

#[derive(Deserialize)]
pub struct SonosAlbum {
    pub name: String,
}

#[derive(Deserialize)]
pub struct SonosArtist {
    pub name: String,
}

#[derive(Deserialize)]
pub struct SonosTrack {
    pub name: String,
    pub album: SonosAlbum,
    pub artist: SonosArtist,
}

#[derive(Deserialize)]
pub struct SonosCurrentItem {
    pub track: SonosTrack,
}

#[derive(Deserialize)]
pub struct SonosPlaybackContainer {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub input_type: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SonosPlaybackMetadata {
    pub container: SonosPlaybackContainer,
    pub current_item: Option<SonosCurrentItem>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SonosModifyGroupMembersRequest {
    pub player_ids_to_add: Vec<String>,
    pub player_ids_to_remove: Vec<String>,
}

// Reconcile with SonosGroup (which has playback State)?
#[derive(Debug,Deserialize)]
#[serde(rename_all = "camelCase")]
struct SonosGroupInfo {
    pub player_ids: Vec<String>,
    pub coordinator_id: String,
    pub id: String,
    pub name: String,
}

#[derive(Debug,Deserialize)]
struct SonosGroupInfoReply {
    pub group: SonosGroupInfo,
}

pub struct Client {
    pub access_token: String,
    pub client_id: String,
    pub http_client: reqwest::Client,
}

impl Client {
    pub fn new(access_token: &str, client_id: &str) -> Client {
        return Client {
            access_token: access_token.to_string(),
            client_id: client_id.to_string(),
            http_client: reqwest::Client::new(),
        }
    }

    pub fn get_households(&self) -> super::result::Result<SonosHouseholdsReply> {
        return Ok(self.issue_get("https://api.ws.sonos.com/control/api/v1/households")?);
    }

    pub fn current_playback_state(&self, group_id: &str) -> super::result::Result<SonosPlaybackMetadata> {
        return Ok(self.issue_get(
            &format!("https://api.ws.sonos.com/control/api/v1/groups/{}/playbackMetadata", group_id))?);
    }

    pub fn get_groups(&self, household_id: &str) -> super::result::Result<SonosGroupsReply> {
        return Ok(self.issue_get(
            &format!("https://api.ws.sonos.com/control/api/v1/households/{}/groups:1", household_id))?);
    }

    pub fn break_group(&self) -> super::result::Result<()> {
        let group_id = "RINCON_B8E937B1D25601400:232";

        let request = SonosModifyGroupMembersRequest{
            player_ids_to_add: vec![],
            player_ids_to_remove: vec!["RINCON_949F3E7DA95401400".to_string()],
        };
        let request_body = serde_json::to_string(&request)?;

        let mut response = self.http_client
            .post(
                &format!("https://api.ws.sonos.com/control/api/v1/groups/{}/groups/modifyGroupMembers", group_id))
            .body(request_body)
            .bearer_auth(&self.access_token)
            .header(reqwest::header::HeaderName::from_static("x-sonos-api-key"),
                    reqwest::header::HeaderValue::from_str(&self.client_id)?)
            .send()
            .unwrap();

        let response_body = response.text()?;

        debug!("Raw response: {:?}",  response_body);

        let parsed_response: SonosGroupInfoReply =
            serde_json::from_str(&response_body)?;
        debug!("Parsed response: {:?}",  parsed_response);

        return Ok(());
    }

    fn issue_get<T: serde::de::DeserializeOwned>(&self, url: &str) -> super::result::Result<T> {
        debug!("Issuing GET {}", url);
        let mut response = self.http_client
            .get(url)
            .bearer_auth(&self.access_token)
            .header(reqwest::header::HeaderName::from_static("x-sonos-api-key"),
                    reqwest::header::HeaderValue::from_str(&self.client_id)?)
            .send()
            .unwrap();

        let response_body = response.text()?;
        debug!("Response {:?}", response_body);

        return Ok(serde_json::from_str(&response_body)?);
    }
}
