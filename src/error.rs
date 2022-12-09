use std::num::TryFromIntError;

use actix_web::{body::BoxBody, HttpResponse, ResponseError};
use http::header;
use serde_json::json;

// TODO: Possibly give the variants better descriptions.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    S3(#[from] s3::error::S3Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    SteamId(#[from] steamid::Error),
    #[error(transparent)]
    TryFromInt(#[from] TryFromIntError),

    #[error("failed to upload demo to S3")]
    DemoUploadError,
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        let mut res = HttpResponse::new(self.status_code());
        res.headers_mut()
            .insert(header::CONTENT_TYPE, "application/json".parse().unwrap());

        let json = json!({
            "error": self.to_string(),
        });
        res.set_body(BoxBody::new(serde_json::to_string(&json).unwrap()))
    }
}
