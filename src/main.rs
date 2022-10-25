mod models;
mod utils;

use crate::models::DatHostMatch;
use actix_web::error::HttpError;
use actix_web::web::Json;
use actix_web::{post, web, App, HttpResponse, HttpServer};
use awc::http::StatusCode;
use awc::Client;
use dotenv::dotenv;
use rusoto_core::Region;
use rusoto_s3::{S3Client, S3};
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Pool, Postgres};
use std::env;
use std::time::Duration;
use anyhow::Result;
use crate::utils::update_score;

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
    let demo = client
        .get(format!(
            "https://dathost.net/api/0.1/game-servers/{}/files/{}",
            &dathost_match.game_server_id, demo_path
        ))
        .send()
        .await?
        .body()
        .limit(300_000_000)
        .await?;
    let client = S3Client::new(Region::Custom {
        name: env::var("AWS_REGION").unwrap_or("auto".to_string()),
        endpoint: env::var("AWS_ENDPOINT").expect("Expected AWS_ENDPOINT"),
    });
    client
        .put_object(utils::get_put_object(demo.to_vec(), &dathost_match.id))
        .await?;
    update_score(pool.get_ref(), &(dathost_match.team1_stats.as_ref().unwrap().score as i32),
                 &(dathost_match.team1_stats.as_ref().unwrap().score as i32),
                 &dathost_match.game_server_id,
                 &dathost_match.map
    ).await?;
    sqlx::query!(
            "update match
            SET completed_at = now()
        where id =
            (select m.id
            from servers s
            join match m on s.match_series = m.match_series
            join maps m2 on m2.id = m.map
            where s.server_id = $1
            and m2.name = $2)",
            &dathost_match.game_server_id,
            &dathost_match.map,
        )
        .execute(&pool)
        .await?;
    Ok(HttpResponse::new(StatusCode::OK))
}

#[post("/api/round-end")]
async fn round_end(
    dathost_match: Json<DatHostMatch>,
    pool: web::Data<Pool<Postgres>>,
) -> Result<HttpResponse, HttpError> {
    update_score(pool.get_ref(), &(dathost_match.team1_stats.as_ref().unwrap().score as i32),
                 &(dathost_match.team1_stats.as_ref().unwrap().score as i32),
                 &dathost_match.game_server_id,
                 &dathost_match.map
    ).await?;
    Ok(HttpResponse::new(StatusCode::OK))
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let pool = PgPool::connect(&env::var("DATABASE_URL").expect("Expected DATABASE_URL"))
        .await
        .unwrap();
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(map_end)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
