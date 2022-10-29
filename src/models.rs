use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct DatHostMatch {
    pub id: String,
    pub game_server_id: String,
    pub match_series_id: Option<String>,
    pub map: String,
    pub finished: bool,
    pub team1_steam_ids: Vec<String>,
    pub team2_steam_ids: Vec<String>,
    pub team1_stats: Option<TeamStats>,
    pub team2_stats: Option<TeamStats>,
}

#[derive(Deserialize, Debug)]
pub struct TeamStats {
    pub score: u32,
}
