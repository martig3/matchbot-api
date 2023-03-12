mod dathost;
mod db;
mod error;
mod matches;
mod models;
mod steam;
mod utils;
mod webhooks;

use crate::matches::matches as matches_all;
use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use s3::{creds::Credentials, Bucket, Region};
use sqlx::PgPool;
use std::env;

use self::{
    dathost::DathostClient,
    models::DatHostMatch,
    webhooks::{map_end, round_end, series_end},
};

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
        let cors = Cors::default()
            .allowed_origin("http://localhost:5173")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(dathost.clone()))
            .app_data(web::Data::new(bucket.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .service(map_end)
            .service(round_end)
            .service(series_end)
            .service(matches_all)
    })
    .bind((host, port))?
    .run()
    .await
}
