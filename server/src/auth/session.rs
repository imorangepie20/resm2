use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{Duration, NaiveDateTime, Utc};
use sha2::{Digest, Sha256};
use sqlx::PgPool;

const TTL_DAYS: i64 = 30;

pub fn new_token() -> String {
    let bytes: [u8; 32] = rand::random();
    URL_SAFE_NO_PAD.encode(bytes)
}

pub fn hash_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

pub async fn create(db: &PgPool, user_id: &str) -> sqlx::Result<(String, NaiveDateTime)> {
    let token = new_token();
    let expires_at = Utc::now().naive_utc() + Duration::days(TTL_DAYS);
    sqlx::query(
        r#"INSERT INTO "Session" ("id", "userId", "tokenHash", "expiresAt") VALUES ($1, $2, $3, $4)"#,
    )
    .bind(cuid2::create_id())
    .bind(user_id)
    .bind(hash_token(&token))
    .bind(expires_at)
    .execute(db)
    .await?;
    Ok((token, expires_at))
}

pub async fn invalidate(db: &PgPool, token: &str) -> sqlx::Result<()> {
    sqlx::query(r#"DELETE FROM "Session" WHERE "tokenHash" = $1"#)
        .bind(hash_token(token))
        .execute(db)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_is_43_char_base64url() {
        let t = new_token();
        assert_eq!(t.len(), 43);
        assert!(
            t.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        );
    }

    #[test]
    fn token_hash_matches_node_sha256_hex() {
        // node: crypto.createHash("sha256").update("abc").digest("hex")
        assert_eq!(
            hash_token("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
