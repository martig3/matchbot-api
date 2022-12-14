use sqlx::{PgExecutor, Result};

use crate::models::{MatchId, MatchSeries, MatchSeriesId, ServerId, TeamId};

pub async fn get_team_one_id(
    executor: impl PgExecutor<'_>,
    match_series_id: MatchSeriesId,
) -> Result<TeamId> {
    sqlx::query_scalar!(
        "select t.id
            from match_series ms
            join teams t on t.id = ms.team_one
         where ms.id = $1",
        match_series_id.0
    )
    .fetch_one(executor)
    .await
    .map(TeamId)
}

pub async fn get_series(
    executor: impl PgExecutor<'_>,
    server_id: &ServerId,
) -> Result<MatchSeries> {
    Ok(sqlx::query_as(
        "select ms.* from servers s
            join match_series ms on ms.id = s.match_series
         where s.server_id = $1",
    )
    .bind(server_id.as_ref())
    .fetch_one(executor)
    .await?)
}

pub async fn is_player_on_team(
    executor: impl PgExecutor<'_>,
    series_id: MatchSeriesId,
    team_id: TeamId,
    steam_id: i64,
) -> Result<bool> {
    sqlx::query_scalar!(
        "select count(si.*) from match_series ms \
        join teams t on ms.team_one = t.id or ms.team_two = t.id \
        join team_members tm on tm.team = t.id \
        join steam_ids si on si.discord = tm.member \
        where ms.id = $1 \
            and t.id = $2 \
            and si.steam = $3",
        series_id.0,
        team_id.0,
        steam_id
    )
    .fetch_one(executor)
    .await
    .map(Option::unwrap_or_default)
    .map(|count| count > 0)
}

pub async fn get_match_id(
    executor: impl PgExecutor<'_>,
    match_series_id: MatchSeriesId,
    map: &str,
) -> Result<MatchId> {
    sqlx::query_scalar!(
        "select m.id from match m 
         join maps on maps.id = m.map 
         where m.match_series = $1 and maps.name = $2",
        match_series_id.0,
        map
    )
    .fetch_one(executor)
    .await
    .map(MatchId)
}

pub async fn complete_match(executor: impl PgExecutor<'_>, match_id: MatchId) -> Result<u64> {
    sqlx::query!(
        "update match
            SET completed_at = now()
        where id = $1",
        match_id.0,
    )
    .execute(executor)
    .await
    .map(|result| result.rows_affected())
}

pub async fn complete_match_series(
    executor: impl PgExecutor<'_>,
    match_series_id: MatchSeriesId,
) -> Result<u64> {
    // TODO: Replace with macro.
    sqlx::query(
        "update match_series
            SET completed_at = now()
        where id = $1",
    )
    .bind(match_series_id.0)
    .execute(executor)
    .await
    .map(|result| result.rows_affected())
}

pub async fn update_scores(
    executor: impl PgExecutor<'_>,
    match_id: MatchId,
    team_one_score: i32,
    team_two_score: i32,
) -> Result<u64> {
    // TODO: Replace with macro.
    sqlx::query(
        "update match_scores
            set team_one_score = $1,
                team_two_score = $2
        where match_id = $3",
    )
    .bind(team_one_score)
    .bind(team_two_score)
    .bind(match_id.0)
    .execute(executor)
    .await
    .map(|result| result.rows_affected())
}
