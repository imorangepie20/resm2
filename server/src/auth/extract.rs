use axum::extract::{FromRef, FromRequestParts};
use axum::http::HeaderMap;
use axum::http::header::{AUTHORIZATION, COOKIE};
use axum::http::request::Parts;
use chrono::{NaiveDateTime, Utc};
use sqlx::FromRow;

use super::session::hash_token;
use crate::{ApiError, AppState};

pub const COOKIE_NAME: &str = "session";

pub struct SessionToken(pub String);

#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: String,
    pub agency_id: String,
    pub email: String,
    pub name: Option<String>,
    pub role: String,
}

/// `Authorization: Bearer` 가 있으면 그것만 사용(유효하지 않아도 쿠키로 폴백하지 않음), 없으면 `session` 쿠키.
fn token_from_headers(headers: &HeaderMap) -> Result<String, ApiError> {
    if let Some(value) = headers.get(AUTHORIZATION) {
        let token = value
            .to_str()
            .ok()
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(str::trim)
            .filter(|t| !t.is_empty());
        return token.map(String::from).ok_or(ApiError::Unauthorized);
    }
    headers
        .get_all(COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(name, value)| *name == COOKIE_NAME && !value.is_empty())
        .map(|(_, value)| value.to_string())
        .ok_or(ApiError::Unauthorized)
}

impl<S> FromRequestParts<S> for SessionToken
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, ApiError> {
        token_from_headers(&parts.headers).map(SessionToken)
    }
}

#[derive(FromRow)]
struct Row {
    session_id: String,
    expires_at: NaiveDateTime,
    id: String,
    agency_id: String,
    email: String,
    name: Option<String>,
    role: String,
}

impl<S> FromRequestParts<S> for CurrentUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, ApiError> {
        let state = AppState::from_ref(state);
        let token = token_from_headers(&parts.headers)?;
        let row: Option<Row> = sqlx::query_as(
            r#"SELECT s."id" AS session_id, s."expiresAt" AS expires_at,
                      u."id", u."agencyId" AS agency_id, u."email", u."name", u."role"
               FROM "Session" s JOIN "User" u ON u."id" = s."userId"
               WHERE s."tokenHash" = $1"#,
        )
        .bind(hash_token(&token))
        .fetch_optional(&state.db)
        .await?;
        let row = row.ok_or(ApiError::Unauthorized)?;
        if row.expires_at < Utc::now().naive_utc() {
            sqlx::query(r#"DELETE FROM "Session" WHERE "id" = $1"#)
                .bind(&row.session_id)
                .execute(&state.db)
                .await?;
            return Err(ApiError::Unauthorized);
        }
        Ok(CurrentUser {
            id: row.id,
            agency_id: row.agency_id,
            email: row.email,
            name: row.name,
            role: row.role,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    fn headers(pairs: &[(axum::http::HeaderName, &str)]) -> HeaderMap {
        let mut h = HeaderMap::new();
        for (k, v) in pairs {
            h.append(k.clone(), HeaderValue::from_str(v).unwrap());
        }
        h
    }

    #[test]
    fn bearer_wins_over_cookie() {
        let h = headers(&[(AUTHORIZATION, "Bearer abc"), (COOKIE, "session=zzz")]);
        assert_eq!(token_from_headers(&h).unwrap(), "abc");
    }

    #[test]
    fn invalid_bearer_does_not_fall_back_to_cookie() {
        let h = headers(&[(AUTHORIZATION, "Basic abc"), (COOKIE, "session=zzz")]);
        assert!(matches!(
            token_from_headers(&h),
            Err(ApiError::Unauthorized)
        ));
    }

    #[test]
    fn reads_session_cookie_among_others() {
        let h = headers(&[(COOKIE, "a=1; session=tok-_9; b=2")]);
        assert_eq!(token_from_headers(&h).unwrap(), "tok-_9");
    }

    #[test]
    fn nothing_is_unauthorized() {
        assert!(matches!(
            token_from_headers(&HeaderMap::new()),
            Err(ApiError::Unauthorized)
        ));
    }
}
