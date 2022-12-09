use derive_more::{AsRef, Deref, Display, From, Into};
use serde::{Deserialize, Serialize};
use sqlx::types::time::OffsetDateTime;
use sqlx::{FromRow, Type};
use strum::EnumIter;

#[derive(Clone, Copy, Debug, From, Into, Deref, AsRef, Display, Serialize, Deserialize)]
#[repr(transparent)]
pub struct MatchId(pub(crate) i32);

#[derive(Clone, Copy, Debug, From, Into, Deref, AsRef, Display, Serialize, Deserialize)]
#[repr(transparent)]
pub struct MatchSeriesId(pub(crate) i32);

#[derive(Clone, Copy, Debug, From, Into, Deref, AsRef, Display, Serialize, Deserialize)]
#[repr(transparent)]
pub struct TeamId(pub(crate) i32);

#[derive(Debug, From, Into, Deref, AsRef, Display, Serialize, Deserialize)]
#[repr(transparent)]
pub struct ServerId(pub(crate) String);

#[derive(Clone, Copy, Debug)]
pub struct SteamUser {
    pub discord: i64,
    pub steam: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DatHostMatch {
    pub id: String,
    #[serde(rename = "game_server_id")]
    pub server_id: ServerId,
    pub match_series_id: Option<String>,
    pub map: Option<String>,
    pub finished: bool,
    pub team1_steam_ids: Vec<String>,
    pub team2_steam_ids: Vec<String>,
    pub team1_stats: Option<TeamStats>,
    pub team2_stats: Option<TeamStats>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TeamStats {
    pub score: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DatHostServer {
    pub csgo_settings: CsgoSettings,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CsgoSettings {
    pub mapgroup_start_map: String,
    #[serde(rename = "steam_game_server_login_token")]
    pub gslt: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteGsltRequest {
    pub(crate) steamid: u64,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryLoginTokenRequest {
    pub(crate) login_token: String,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SteamApiRootResponse {
    pub response: QueryLoginTokenResponse,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryLoginTokenResponse {
    pub steamid: String,
    pub is_banned: bool,
    pub expires: u64,
}

#[derive(Debug, Type, EnumIter)]
#[sqlx(rename_all = "lowercase", type_name = "series_type")]
pub enum SeriesType {
    Bo1,
    Bo3,
    Bo5,
}
#[allow(unused)]
#[derive(Debug, FromRow)]
pub struct MatchSeries {
    pub id: i32,
    pub team_one: i32,
    pub team_two: i32,
    pub series_type: SeriesType,
    pub dathost_match: Option<String>,
    pub created_at: OffsetDateTime,
    pub completed_at: Option<OffsetDateTime>,
}
