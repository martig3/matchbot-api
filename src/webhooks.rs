use crate::{
    dathost::DathostClient,
    db::{
        complete_match, complete_match_series, get_match_id, get_series_id, get_team_one_id,
        maps_remaining,
    },
    error::Error,
    models::DatHostMatch,
    utils::{teardown_server, update_scores},
};
use actix_web::post;
use actix_web::{web, Responder};
use http::StatusCode;
use s3::Bucket;
use sqlx::PgPool;

#[post("/api/map-end")]
pub async fn map_end(
    dathost_match: web::Json<DatHostMatch>,
    pool: web::Data<PgPool>,
    client: web::Data<DathostClient>,
    bucket: web::Data<Bucket>,
) -> Result<impl Responder, Error> {
    // TODO: update this to support Bo3 by appending '_match#'
    let path = format!("{}.dem", dathost_match.id);
    let demo = client.get_file(&dathost_match.server_id, &path).await?;
    if bucket.put_object(path, &demo).await?.status_code() != 200 {
        return Err(Error::DemoUploadError);
    }

    let match_series_id = get_series_id(pool.as_ref(), &dathost_match.server_id).await?;
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

    complete_match(pool.as_ref(), match_id).await?;
    let maps_remaining = maps_remaining(pool.as_ref(), match_series_id).await?;
    if maps_remaining == 0 {
        // TODO: add Bo3 logic here
        complete_match_series(pool.as_ref(), match_series_id).await?;
        teardown_server(dathost_match.0, client).await?;
    }

    Ok((Vec::new(), StatusCode::NO_CONTENT))
}

#[post("/api/round-end")]
pub async fn round_end(
    dathost_match: web::Json<DatHostMatch>,
    pool: web::Data<PgPool>,
    client: web::Data<DathostClient>,
) -> Result<impl Responder, Error> {
    let match_series_id = get_series_id(pool.as_ref(), &dathost_match.server_id).await?;
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
