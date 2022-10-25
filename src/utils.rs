use rusoto_s3::{PutObjectRequest, StreamingBody};
use std::env;
use sqlx::{Pool, Postgres};
use anyhow::Result;

pub async fn update_score(pool: &Pool<Postgres>, team_one_score: &i32, team_two_score: &i32, game_server_id: &String, map: &String) -> Result<()> {
    Ok(sqlx::query!(
        "update match_scores
            SET team_one_score = $1, team_two_score= $2
        where match_id =
            (select m.id
            from servers s
            join match m on s.match_series = m.match_series
            join maps m2 on m2.id = m.map
            where s.server_id = $3
            and m2.name = $4)",
        team_one_score,
        team_two_score,
        game_server_id,
        map,
    )
        .execute(pool)
        .await)
}
pub fn get_put_object(contents: Vec<u8>, dathost_match_id: &String) -> PutObjectRequest {
    PutObjectRequest {
        acl: None,
        body: Some(StreamingBody::from(contents)),
        bucket: env::var("BUCKET_NAME").expect("Expected BUCKET_NAME"),
        bucket_key_enabled: None,
        cache_control: None,
        content_disposition: None,
        content_encoding: None,
        content_language: None,
        content_length: None,
        content_md5: None,
        content_type: Some("application/octet-stream".to_string()),
        expected_bucket_owner: None,
        expires: None,
        grant_full_control: None,
        grant_read: None,
        grant_read_acp: None,
        grant_write_acp: None,
        key: format!("{}.dem", &dathost_match_id),
        metadata: None,
        object_lock_legal_hold_status: None,
        object_lock_mode: None,
        object_lock_retain_until_date: None,
        request_payer: None,
        sse_customer_algorithm: None,
        sse_customer_key: None,
        sse_customer_key_md5: None,
        ssekms_encryption_context: None,
        ssekms_key_id: None,
        server_side_encryption: None,
        storage_class: None,
        tagging: None,
        website_redirect_location: None,
    }
}
