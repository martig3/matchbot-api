mod models;
mod utils;

use crate::models::DatHostMatch;
use crate::models::DatHostServer;
use crate::utils::{get_series_id, get_server_map, get_team_one_id, update_score};
use actix_web::error::HttpError;
use actix_web::middleware::Logger;
use actix_web::web::Json;
use actix_web::{post, web, App, HttpResponse, HttpServer};
use awc::http::StatusCode;
use awc::Client;
use dotenv::dotenv;
use s3::creds::Credentials;
use s3::{Bucket, Region};
use sqlx::{PgPool, Pool, Postgres};
use std::env;
use std::time::Duration;
use log::LevelFilter;

#[post("/api/map-end")]
async fn map_end(
    dathost_match: Json<DatHostMatch>,
    pool: web::Data<Pool<Postgres>>,
) -> Result<HttpResponse, HttpError> {
    let client = Client::builder()
        .basic_auth(
            env::var("DATHOST_USER").unwrap(),
            Some(&env::var("DATHOST_PASSWORD").unwrap()),
        )
        .timeout(Duration::from_secs(60 * 10))
        .finish();
    let demo_path = format!("{}.dem", &dathost_match.id);
    log::debug!("fetching '{}'", &demo_path);
    let demo = client
        .get(format!(
            "https://dathost.net/api/0.1/game-servers/{}/files/{}",
            &dathost_match.game_server_id, demo_path
        ))
        .send()
        .await
        .unwrap()
        .body()
        .limit(300_000_000)
        .await
        .unwrap();
    let region = env::var("AWS_REGION").unwrap_or("auto".to_string());
    let endpoint = env::var("AWS_ENDPOINT").unwrap();
    let bucket = Bucket::new(
        env::var("BUCKET_NAME")
            .expect("Expected BUCKET_NAME")
            .as_str(),
        Region::Custom { region, endpoint },
        Credentials::default().unwrap(),
    )
    .unwrap();
    log::debug!("uploading '{}'", &demo_path);
    let put_resp = bucket
        .put_object(format!("{}", demo_path), &demo.to_vec())
        .await;
    if let Err(e) = put_resp {
        log::error!("{:#?}", e);
        return Ok(HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR));
    } else {
        let status_code = put_resp.unwrap().status_code();
        if status_code != 200 {
            log::error!("S3 upload error, status code {}", status_code);
            return Ok(HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR));
        }
    }
    log::debug!("demo upload complete");
    let match_series_id = get_series_id(pool.as_ref(), &dathost_match).await.unwrap();
    let team_one_id: i32 = get_team_one_id(pool.as_ref(), &match_series_id)
        .await
        .unwrap();
    let map = get_server_map(&client, &dathost_match).await;
    let match_id: i32 = sqlx::query_scalar!(
        "select m.id from match m 
         join maps on maps.id = m.map 
         where m.match_series = $1 and maps.name = $2",
        &match_series_id,
        &map,
    )
    .fetch_one(pool.as_ref())
    .await
    .unwrap();
    update_score(
        pool.get_ref(),
        &dathost_match.0,
        match_series_id,
        team_one_id,
        map,
    )
    .await
    .unwrap();
    log::debug!("updated match score");
    sqlx::query!(
        "update match
            SET completed_at = now()
        where id = $1",
        match_id,
    )
    .execute(pool.as_ref())
    .await
    .unwrap();
    let maps_remaining: i64 = sqlx::query_scalar!(
        "select count(m)
        from match_series ms
         join match m on ms.id = m.match_series
        where ms.id = $1
        and m.completed_at is null",
        match_series_id
    )
    .fetch_one(pool.as_ref())
    .await
    .unwrap()
    .unwrap();
    if maps_remaining == 0 {
        sqlx::query!(
            "update match_series set completed_at = now() where match_series.id = $1",
            match_series_id,
        )
        .execute(pool.as_ref())
        .await
        .unwrap();
    }
    Ok(HttpResponse::new(StatusCode::OK))
}

#[post("/api/round-end")]
async fn round_end(
    dathost_match: Json<DatHostMatch>,
    pool: web::Data<Pool<Postgres>>,
) -> Result<HttpResponse, HttpError> {
    let match_series_id = get_series_id(pool.as_ref(), &dathost_match).await.unwrap();
    let team_one_id: i32 = get_team_one_id(pool.as_ref(), &match_series_id)
        .await
        .unwrap();
    let client = Client::builder()
        .basic_auth(
            env::var("DATHOST_USER").unwrap(),
            Some(&env::var("DATHOST_PASSWORD").unwrap()),
        )
        .timeout(Duration::from_secs(60 * 10))
        .finish();
    let map = get_server_map(&client, &dathost_match).await;
    update_score(
        pool.get_ref(),
        &dathost_match,
        match_series_id,
        team_one_id,
        map,
    )
    .await
    .unwrap();
    Ok(HttpResponse::new(StatusCode::OK))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::builder()
        .filter_module(module_path!(), LevelFilter::Info)
        .parse_default_env()
        .init();
    log::info!("Starting matchbot-api");
    let pool = PgPool::connect(&env::var("DATABASE_URL").expect("Expected DATABASE_URL"))
        .await
        .unwrap();
    let url = match env::var("ENV") {
        Ok(v) => {
            if v == "prd" {
                "0.0.0.0"
            } else {
                "127.0.0.1"
            }
        }
        Err(_) => "127.0.0.1",
    };
    let port = match env::var("PORT") {
        Ok(v) => v.parse::<u16>().expect("PORT not valid u16"),
        Err(_) => 8080,
    };
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(Logger::default())
            .service(map_end)
            .service(round_end)
    })
    .bind((url, port))?
    .run()
    .await
}
