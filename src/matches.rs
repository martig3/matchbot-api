use crate::error::Error;
use actix_web::get;
use actix_web::{web, Responder};
use futures::future::try_join_all;
use matchbot_core::maps::Map;
use matchbot_core::matches::{Match, MatchScore, MatchSeries, Server};
use matchbot_core::team::Team;
use serde::Serialize;
use sqlx::types::time::OffsetDateTime;
use sqlx::{PgPool, Pool, Postgres};

#[derive(Serialize)]
struct MatchesResponse {
    completed: Vec<DetailedMatch>,
    in_progress: Vec<DetailedMatch>,
    scheduled: Vec<DetailedMatch>,
}
#[derive(Serialize)]
struct DetailedMatch {
    id: i32,
    team_one: Team,
    team_two: Team,
    scores: Vec<MatchScore>,
    server: Option<Server>,
    maps_info: Vec<DetailedMap>,
    dathost_match: Option<String>,
    completed_at: Option<OffsetDateTime>,
}
#[derive(Serialize)]
struct DetailedMap {
    pub map: String,
    pub picked_by: String,
    pub start_ct_team: String,
    pub start_t_team: String,
    pub completed: bool,
}

#[get("/api/matches")]
pub async fn matches(pool: web::Data<PgPool>) -> Result<impl Responder, Error> {
    let completed = MatchSeries::get_all(pool.as_ref(), 500, true).await?;
    let in_progress = MatchSeries::get_in_progress(pool.as_ref()).await?;
    let scheduled = MatchSeries::get_scheduled(pool.as_ref()).await?;
    let completed = try_join_all(completed.iter().map(|m| match_info(pool.as_ref(), m))).await?;
    let in_progress =
        try_join_all(in_progress.iter().map(|m| match_info(pool.as_ref(), m))).await?;
    let scheduled = try_join_all(scheduled.iter().map(|m| match_info(pool.as_ref(), m))).await?;
    let resp = MatchesResponse {
        completed,
        in_progress,
        scheduled,
    };
    Ok(web::Json(resp))
}

async fn match_info(pool: &Pool<Postgres>, series: &MatchSeries) -> Result<DetailedMatch, Error> {
    let team_one = Team::get(pool, series.team_one).await?;
    let team_two = Team::get(pool, series.team_two).await?;
    let scores: Vec<MatchScore> = MatchScore::get_by_series(pool, series.id).await?;
    let servers = Server::get_live(pool).await?;
    let server = servers.into_iter().find(|s| s.match_series == series.id);
    let series_matches: Vec<Match> = Match::get_by_series(pool, series.id).await?;

    let maps = Map::get_all(pool, false).await?;
    let maps_info: Vec<DetailedMap> = series_matches
        .into_iter()
        .map(|m| {
            return DetailedMap {
                map: maps
                    .iter()
                    .find(|map| map.id == m.map)
                    .unwrap()
                    .name
                    .clone(),
                picked_by: if m.picked_by == team_one.id {
                    team_one.name.clone()
                } else {
                    team_two.name.clone()
                },
                start_ct_team: if m.start_ct_team == team_one.id {
                    team_one.name.clone()
                } else {
                    team_two.name.clone()
                },
                start_t_team: if m.start_t_team == team_one.id {
                    team_one.name.clone()
                } else {
                    team_two.name.clone()
                },
                completed: m.completed_at.is_some(),
            };
        })
        .collect();
    Ok(DetailedMatch {
        id: series.id,
        team_one,
        team_two,
        scores,
        server,
        maps_info,
        dathost_match: series.dathost_match.clone(),
        completed_at: series.completed_at,
    })
}
