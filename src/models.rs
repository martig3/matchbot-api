use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Debug)]
pub struct DatHostMatch {
    pub id: String,
    pub game_server_id: String,
    pub match_series_id: Option<String>,
    pub map: String,
    pub finished: bool,
    pub team1_stats: Option<TeamStats>,
    pub team2_stats: Option<TeamStats>,
}

#[derive(Deserialize, Debug)]
pub struct TeamStats {
    pub score: u32,
}

#[derive(Serialize, Deserialize, sqlx::FromRow, Debug)]
pub struct SeriesMap {
    pub id: i32,
    pub match_id: i32,
    pub map: String,
    pub picked_by_role_id: i64,
    pub start_attack_team_role_id: Option<i64>,
    pub start_defense_team_role_id: Option<i64>,
}
