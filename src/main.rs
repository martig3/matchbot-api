mod dathost;
mod db;
mod error;
mod models;
mod utils;

use std::env;

use actix_web::{middleware::Logger, post, web, App, HttpServer, Responder};
use http::StatusCode;
use s3::{creds::Credentials, Bucket, Region};
use sqlx::PgPool;

use self::{
    dathost::DathostClient,
    db::{
        complete_match, complete_match_series, get_match_id, get_series_id, get_team_one_id,
        maps_remaining,
    },
    error::Error,
    models::DatHostMatch,
    utils::update_scores,
};

#[post("/api/map-end")]
async fn map_end(
    dathost_match: web::Json<DatHostMatch>,
    pool: web::Data<PgPool>,
    client: web::Data<DathostClient>,
    bucket: web::Data<Bucket>,
) -> Result<impl Responder, Error> {
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
        complete_match_series(pool.as_ref(), match_series_id).await?;
    }

    Ok((Vec::new(), StatusCode::NO_CONTENT))
}

#[post("/api/round-end")]
async fn round_end(
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::builder()
        .filter_module(module_path!(), log::LevelFilter::Info)
        .parse_default_env()
        .init();

    log::info!("Starting matchbot-api");

    let pool = PgPool::connect(&env::var("DATABASE_URL").expect("Expected DATABASE_URL"))
        .await
        .unwrap();

    let dathost = DathostClient::new().expect("unable to create DatHost client");

    let region = env::var("AWS_REGION").unwrap_or_else(|_| "auto".to_string());
    let endpoint = env::var("AWS_ENDPOINT").expect("AWS_ENDPOINT must be set");
    let bucket = Bucket::new(
        env::var("BUCKET_NAME")
            .expect("BUCKET_NAME must be set")
            .as_str(),
        Region::Custom { region, endpoint },
        Credentials::default().unwrap(),
    )
    .expect("unable to connect to S3 bucket");

    let host = matches!(env::var("ENV").as_deref(), Ok("prd"))
        .then_some("0.0.0.0")
        .unwrap_or("127.0.0.1");
    let port = env::var("PORT")
        .map(|p| p.parse::<u16>().expect("PORT is not a valid u16"))
        .unwrap_or(8080);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(dathost.clone()))
            .app_data(web::Data::new(bucket.clone()))
            .wrap(Logger::default())
            .service(map_end)
            .service(round_end)
    })
    .bind((host, port))?
    .run()
    .await
}
