use crate::{
    dathost::DathostClient,
    db,
    error::Error,
    models::{MatchId, MatchSeriesId, ServerId, TeamId},
    steam::SteamWebClient,
    DatHostMatch,
};
use actix_web::web;
use sqlx::PgPool;
use steamid::{AccountType, Instance, SteamId, Universe};

trait ParseWithDefaults: Sized {
    fn parse<S: AsRef<str>>(value: S) -> Result<Self, steamid::Error>;
}

impl ParseWithDefaults for SteamId {
    fn parse<S: AsRef<str>>(value: S) -> Result<Self, steamid::Error> {
        let mut steamid = Self::parse_steam2id(value, AccountType::Individual, Instance::Desktop)?;
        steamid.set_universe(Universe::Public);
        Ok(steamid)
    }
}

pub async fn update_scores(
    pool: &PgPool,
    dathost_match: &DatHostMatch,
    series_id: MatchSeriesId,
    match_id: MatchId,
    team_one_id: TeamId,
) -> Result<u64, Error> {
    let mut team_one_score =
        i32::try_from(dathost_match.team1_stats.as_ref().unwrap().score.unwrap())?;
    let mut team_two_score =
        i32::try_from(dathost_match.team2_stats.as_ref().unwrap().score.unwrap())?;
    let swap_teams = db::is_player_on_team(
        pool,
        series_id,
        team_one_id,
        i64::try_from(u64::from(SteamId::parse(
            &dathost_match.team2_steam_ids[0],
        )?))?,
    )
    .await?;
    if swap_teams {
        std::mem::swap(&mut team_one_score, &mut team_two_score);
    }
    Ok(db::update_scores(pool, match_id, team_one_score, team_two_score).await?)
}

pub async fn teardown_server(
    server_id: &ServerId,
    dathost_client: web::Data<DathostClient>,
) -> Result<reqwest::Response, Error> {
    let server = dathost_client.get_server_info(server_id).await?;
    let steam_client = SteamWebClient::new()?;
    let steamid = steam_client
        .query_login_token(server.csgo_settings.gslt)
        .await?;
    steam_client.delete_gslt(steamid).await?;
    dathost_client.stop_server(server_id).await?;
    Ok(dathost_client.delete_server(server_id).await?)
}
