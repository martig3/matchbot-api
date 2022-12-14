use crate::{
    dathost::DathostClient,
    db::{complete_match, complete_match_series, get_match_id, get_series, get_team_one_id},
    error::Error,
    models::{DatHostMatch, DatHostMatchSeries, MatchSeriesId, SeriesType},
    utils::{teardown_server, update_scores},
};
use actix_web::post;
use actix_web::{web, Responder};
use http::StatusCode;
use s3::Bucket;
use sqlx::PgPool;
use std::env;
use std::time::Duration;
use tokio::time::sleep;

#[post("/api/map-end")]
pub async fn map_end(
    dathost_match: web::Json<DatHostMatch>,
    pool: web::Data<PgPool>,
    client: web::Data<DathostClient>,
    bucket: web::Data<Bucket>,
) -> Result<impl Responder, Error> {
    let match_series = get_series(pool.as_ref(), &dathost_match.server_id).await?;
    if match_series.series_type != SeriesType::Bo1 {
        return Ok((Vec::new(), StatusCode::NO_CONTENT));
    }
    let match_series_id = MatchSeriesId(match_series.id);
    let map = match &dathost_match.map {
        // TODO: Remove this clone.
        Some(map) => map.clone(),
        None => client.get_server_map(&dathost_match.server_id).await?,
    };
    let match_id = get_match_id(pool.as_ref(), match_series_id, &map).await?;
    complete_match(pool.as_ref(), match_id).await?;
    complete_match_series(pool.as_ref(), match_series_id).await?;
    let team_one_id = get_team_one_id(pool.as_ref(), match_series_id).await?;
    update_scores(
        pool.as_ref(),
        &dathost_match,
        match_series_id,
        match_id,
        team_one_id,
    )
    .await?;

    // wait for GOTV
    let tv_delay = env::var("TV_DELAY").unwrap_or("105".to_string());
    sleep(Duration::from_secs(tv_delay.parse::<u64>().unwrap() + 30)).await;
    let path = format!("{}.dem", dathost_match.id);
    let demo = client.get_file(&dathost_match.server_id, &path).await?;

    if bucket.put_object(path, &demo).await?.status_code() != 200 {
        return Err(Error::DemoUploadError);
    }

    teardown_server(&dathost_match.server_id, client).await?;

    Ok((Vec::new(), StatusCode::NO_CONTENT))
}

#[post("/api/round-end")]
pub async fn round_end(
    dathost_match: web::Json<DatHostMatch>,
    pool: web::Data<PgPool>,
    client: web::Data<DathostClient>,
) -> Result<impl Responder, Error> {
    let match_series_id = MatchSeriesId(
        get_series(pool.as_ref(), &dathost_match.server_id)
            .await?
            .id,
    );
    let team_one_id = get_team_one_id(pool.as_ref(), match_series_id).await?;

    let map = match &dathost_match.map {
        // TODO: Remove this clone.
        Some(map) => map.clone(),
        None => client.get_server_map(&dathost_match.server_id).await?,
    };

    let match_id = get_match_id(pool.as_ref(), match_series_id, &map).await?;

    update_scores(
        pool.as_ref(),
        &dathost_match,
        match_series_id,
        match_id,
        team_one_id,
    )
    .await?;

    Ok((Vec::new(), StatusCode::NO_CONTENT))
}

#[post("/api/series-end")]
pub async fn series_end(
    dathost_series: web::Json<DatHostMatchSeries>,
    pool: web::Data<PgPool>,
    client: web::Data<DathostClient>,
    bucket: web::Data<Bucket>,
) -> Result<impl Responder, Error> {
    let server_id = &dathost_series.matches.get(0).unwrap().server_id;
    let match_series = get_series(pool.as_ref(), &server_id).await?;
    let match_series_id = MatchSeriesId(match_series.id);
    complete_match_series(pool.as_ref(), match_series_id).await?;

    // wait for GOTV
    let tv_delay = env::var("TV_DELAY").unwrap_or("105".to_string());
    sleep(Duration::from_secs(tv_delay.parse::<u64>().unwrap() + 30)).await;

    let mut i = 1;
    for dathost_match in &dathost_series.matches {
        if dathost_match.team1_stats.as_ref().unwrap().score.is_none() {
            continue;
        }
        let match_id = get_match_id(
            pool.as_ref(),
            match_series_id,
            &dathost_match.map.as_ref().unwrap(),
        )
        .await?;
        complete_match(pool.as_ref(), match_id).await?;
        let demo_path = format!("{}.dem", dathost_match.id);
        let demo = client
            .get_file(&dathost_match.server_id, &demo_path)
            .await?;
        let file_path = format!(
            "{}_{}.dem",
            dathost_match.match_series_id.as_ref().unwrap(),
            i
        );
        i += 1;
        if bucket.put_object(file_path, &demo).await?.status_code() != 200 {
            return Err(Error::DemoUploadError);
        }
    }
    teardown_server(server_id, client).await?;
    Ok((Vec::new(), StatusCode::NO_CONTENT))
}
