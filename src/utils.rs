use crate::{DatHostMatch, DatHostServer};
use anyhow::Result;
use awc::Client;
use sqlx::postgres::PgQueryResult;
use sqlx::{FromRow, Pool, Postgres};
use steamid::{AccountType, Instance, SteamId, Universe};

trait ParseWithDefaults: Sized {
    fn parse<S: AsRef<str>>(value: S) -> Result<Self>;
}

impl ParseWithDefaults for SteamId {
    fn parse<S: AsRef<str>>(value: S) -> Result<Self> {
        let mut steamid =
            SteamId::parse_steam2id(value, AccountType::Individual, Instance::Desktop)?;
        steamid.set_universe(Universe::Public);
        Ok(steamid)
    }
}

#[derive(Debug, FromRow)]
pub struct SteamUser {
    pub discord: i64,
    pub steam: i64,
}

pub async fn get_team_one_id(pool: &Pool<Postgres>, match_series_id: &i32) -> Result<i32> {
    Ok(sqlx::query_scalar!(
        "select t.id
            from match_series ms
            join teams t on t.id = ms.team_one
         where ms.id = $1",
        match_series_id,
    )
    .fetch_one(pool)
    .await?)
}

pub async fn get_series_id(pool: &Pool<Postgres>, dathost_match: &DatHostMatch) -> Result<i32> {
    Ok(sqlx::query_scalar!(
        "select s.match_series from servers s where s.server_id = $1",
        dathost_match.game_server_id,
    )
    .fetch_one(pool)
    .await?)
}

pub async fn is_player_on_team(
    pool: &Pool<Postgres>,
    series_id: i32,
    team_id: i32,
    user_steam_id: &String,
) -> Result<bool> {
    let user_steamid64 = u64::from(SteamId::parse(user_steam_id).unwrap()) as i64;
    let steam_id64_count = sqlx::query_scalar!(
        "select count(si.*) from match_series ms \
        join teams t on ms.team_one = t.id or ms.team_two = t.id \
        join team_members tm on tm.team = t.id \
        join steam_ids si on si.discord = tm.member \
        where ms.id = $1 \
            and t.id = $2 \
            and si.steam = $3",
        series_id,
        team_id,
        user_steamid64
    )
    .fetch_one(pool)
    .await?
    .unwrap();
    Ok(steam_id64_count > 0)
}

pub async fn update_score(
    pool: &Pool<Postgres>,
    dathost_match: &DatHostMatch,
    series_id: i32,
    team_one_id: i32,
    map: String,
) -> Result<PgQueryResult> {
    let ds_one = &(dathost_match.team1_stats.as_ref().unwrap().score as i32);
    let ds_two = &(dathost_match.team2_stats.as_ref().unwrap().score as i32);
    // team1 on the server is not guaranteed to match team_one in the database
    // check to see if team one on the server is actually team one in the
    // database, could be the other way around depending on the veto
    let is_on_team1 = is_player_on_team(
        pool,
        series_id,
        team_one_id,
        &dathost_match.team1_steam_ids[0],
    )
    .await?;
    let team_one_score;
    let team_two_score;
    if is_on_team1 {
        team_one_score = *ds_one;
        team_two_score = *ds_two;
    } else {
        team_two_score = *ds_one;
        team_one_score = *ds_two;
    }
    Ok(sqlx::query!(
        "update match_scores
            SET team_one_score = $1, team_two_score = $2
        where match_id =
            (select m.id
            from servers s
            join match m on s.match_series = m.match_series
            join maps m2 on m2.id = m.map
            where s.server_id = $3
            and m2.name = $4)",
        team_one_score,
        team_two_score,
        &dathost_match.game_server_id,
        &map,
    )
    .execute(pool)
    .await?)
}

pub async fn get_server_map(client: &Client, dathost_match: &DatHostMatch) -> String {
    let map = match &dathost_match.map {
        Some(m) => m.clone(),
        None => {
            let server: DatHostServer = client
                .get(format!(
                    "https://dathost.net/api/0.1/game-servers/{}",
                    &dathost_match.game_server_id
                ))
                .send()
                .await
                .unwrap()
                .json::<DatHostServer>()
                .await
                .unwrap();
            server.csgo_settings.mapgroup_start_map.clone()
        }
    };
    map
}
