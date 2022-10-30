use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize, Debug)]
pub struct DatHostMatch {
    pub id: String,
    pub game_server_id: String,
    pub match_series_id: Option<String>,
    pub map: Option<String>,
    pub finished: bool,
    pub team1_steam_ids: Vec<String>,
    pub team2_steam_ids: Vec<String>,
    pub team1_stats: Option<TeamStats>,
    pub team2_stats: Option<TeamStats>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TeamStats {
    pub score: u32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DatHostServer {
    pub csgo_settings: CsgoSettings,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct CsgoSettings {
    pub mapgroup_start_map: String,
}
