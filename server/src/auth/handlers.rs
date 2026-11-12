use axum::Router;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Json, State};
use axum::http::header::SET_COOKIE;
use axum::http::{HeaderName, HeaderValue, StatusCode};
use axum::response::{AppendHeaders, IntoResponse, Response};
use axum::routing::{get, post};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use super::extract::{COOKIE_NAME, CurrentUser, SessionToken};
use super::{password, session};
use crate::{ApiError, ApiResult, AppState, notify};

const MSG_DUPLICATE: &str = "이미 가입된 이메일입니다";
const MSG_BAD_EMAIL: &str = "올바른 이메일을 입력하세요";
const MSG_LOGIN_FAILED: &str = "이메일 또는 비밀번호가 올바르지 않습니다";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/signup", post(signup))
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
        .route("/auth/me", get(me))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UserDto {
    id: String,
    email: String,
    name: Option<String>,
    role: String,
    agency_id: String,
}

#[derive(Serialize)]
struct AuthResponse {
    token: String,
    user: UserDto,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct SignupRequest {
    agency_name: String,
    agency_zipcode: Option<String>,
    agency_address: Option<String>,
    agency_phone: Option<String>,
    name: String,
    phone: Option<String>,
    email: String,
    password: String,
    password_confirm: String,
    agree: bool,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct LoginRequest {
    email: String,
    password: String,
}

fn bad_body(_: JsonRejection) -> ApiError {
    ApiError::BadRequest("잘못된 요청입니다".to_string())
}

/// zod `z.email` 과 같은 형태 검사(로컬파트 문자 집합, 앞 점/연속 점 금지, 도메인 라벨, 알파벳 TLD 2자 이상).
fn is_valid_email(email: &str) -> bool {
    let Some((local, domain)) = email.split_once('@') else {
        return false;
    };
    let local_ok = !local.starts_with('.')
        && !local.contains("..")
        && local
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_'+-.".contains(c))
        && local
            .chars()
            .last()
            .is_some_and(|c| c.is_ascii_alphanumeric() || "_+-".contains(c));
    if !local_ok {
        return false;
    }
    let labels: Vec<&str> = domain.split('.').collect();
    let Some((tld, rest)) = labels.split_last() else {
        return false;
    };
    !rest.is_empty()
        && rest.iter().all(|l| {
            l.chars().next().is_some_and(|c| c.is_ascii_alphanumeric())
                && l.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        })
        && tld.len() >= 2
        && tld.chars().all(|c| c.is_ascii_alphabetic())
}

/// trim 후 비면 `None`.
fn clean(v: Option<String>) -> Option<String> {
    v.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

fn set_cookie_header(value: String) -> AppendHeaders<[(HeaderName, HeaderValue); 1]> {
    AppendHeaders([(
        SET_COOKIE,
        HeaderValue::from_str(&value).expect("쿠키 값은 ASCII"),
    )])
}

fn session_cookie(token: &str, expires_at: NaiveDateTime, secure: bool) -> String {
    let expires = expires_at.format("%a, %d %b %Y %H:%M:%S GMT");
    let mut c = format!("{COOKIE_NAME}={token}; Path=/; Expires={expires}; HttpOnly; SameSite=Lax");
    if secure {
        c.push_str("; Secure");
    }
    c
}

fn cleared_cookie(secure: bool) -> String {
    let mut c = format!(
        "{COOKIE_NAME}=; Path=/; Max-Age=0; Expires=Thu, 01 Jan 1970 00:00:00 GMT; HttpOnly; SameSite=Lax"
    );
    if secure {
        c.push_str("; Secure");
    }
    c
}

async fn hash_blocking(pw: String) -> ApiResult<String> {
    tokio::task::spawn_blocking(move || password::hash(&pw))
        .await
        .map_err(|e| ApiError::Internal(e.into()))?
        .map_err(ApiError::Internal)
}

async fn verify_blocking(pw: String, hash: String) -> ApiResult<bool> {
    tokio::task::spawn_blocking(move || password::verify(&pw, &hash))
        .await
        .map_err(|e| ApiError::Internal(e.into()))
}

fn is_unique_violation(err: &sqlx::Error) -> bool {
    err.as_database_error()
        .and_then(|d| d.code())
        .is_some_and(|c| c == "23505")
}

async fn signup(
    State(state): State<AppState>,
    body: Result<Json<SignupRequest>, JsonRejection>,
) -> ApiResult<Response> {
    let Json(req) = body.map_err(bad_body)?;
    let bad = |m: &str| Err(ApiError::BadRequest(m.to_string()));
    let name = req.name.trim().to_string();
    if req.agency_name.is_empty() {
        return bad("상호명을 입력하세요");
    }
    if name.is_empty() {
        return bad("이름을 입력하세요");
    }
    if !is_valid_email(&req.email) {
        return bad(MSG_BAD_EMAIL);
    }
    // zod 의 string.min 은 UTF-16 코드 유닛 수 기준
    if req.password.encode_utf16().count() < 8 {
        return bad("비밀번호는 8자 이상이어야 합니다");
    }
    if req.password_confirm.is_empty() {
        return bad("비밀번호 확인을 입력하세요");
    }
    if req.password != req.password_confirm {
        return bad("비밀번호가 일치하지 않습니다");
    }
    if !req.agree {
        return bad("이용약관에 동의해주세요");
    }

    let taken: bool =
        sqlx::query_scalar(r#"SELECT EXISTS (SELECT 1 FROM "User" WHERE "email" = $1)"#)
            .bind(&req.email)
            .fetch_one(&state.db)
            .await?;
    if taken {
        return bad(MSG_DUPLICATE);
    }

    let password_hash = hash_blocking(req.password.clone()).await?;

    let mut tx = state.db.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(824201409141::bigint)")
        .execute(&mut *tx)
        .await?;
    let user_count: i64 = sqlx::query_scalar(r#"SELECT COUNT(*) FROM "User""#)
        .fetch_one(&mut *tx)
        .await?;
    let is_first_user = user_count == 0;
    let role = if is_first_user {
        "superadmin"
    } else {
        "member"
    };
    let agency_id = cuid2::create_id();
    sqlx::query(
        r#"INSERT INTO "Agency" ("id", "name", "zipcode", "address", "phone") VALUES ($1, $2, $3, $4, $5)"#,
    )
    .bind(&agency_id)
    .bind(&req.agency_name)
    .bind(clean(req.agency_zipcode))
    .bind(clean(req.agency_address))
    .bind(clean(req.agency_phone))
    .execute(&mut *tx)
    .await?;
    let user_id = cuid2::create_id();
    let inserted = sqlx::query(
        r#"INSERT INTO "User" ("id", "agencyId", "email", "passwordHash", "name", "phone", "role")
           VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
    )
    .bind(&user_id)
    .bind(&agency_id)
    .bind(&req.email)
    .bind(&password_hash)
    .bind(&name)
    .bind(clean(req.phone))
    .bind(role)
    .execute(&mut *tx)
    .await;
    if let Err(err) = inserted {
        return if is_unique_violation(&err) {
            bad(MSG_DUPLICATE)
        } else {
            Err(err.into())
        };
    }
    tx.commit().await?;

    let (token, expires_at) = session::create(&state.db, &user_id).await?;
    // 첫 가입자(자동 superadmin)는 제외 — 본인 외 알릴 대상이 없음. 알림 실패가 가입을 되돌리지는 않는다.
    if !is_first_user
        && let Err(err) = notify::notify_superadmins(
            &state.db,
            "member",
            &format!("새 회원 가입: {name}"),
            Some(&format!("{} · {}", req.email, req.agency_name)),
            Some("/settings/members"),
        )
        .await
    {
        tracing::error!(error = ?err, "가입 알림 기록 실패");
    }

    let cookie = session_cookie(&token, expires_at, state.cfg.cookie_secure);
    Ok((
        set_cookie_header(cookie),
        Json(AuthResponse {
            token,
            user: UserDto {
                id: user_id,
                email: req.email,
                name: Some(name),
                role: role.to_string(),
                agency_id,
            },
        }),
    )
        .into_response())
}

#[derive(FromRow)]
struct LoginRow {
    id: String,
    agency_id: String,
    email: String,
    name: Option<String>,
    role: String,
    password_hash: String,
}

async fn login(
    State(state): State<AppState>,
    body: Result<Json<LoginRequest>, JsonRejection>,
) -> ApiResult<Response> {
    let Json(req) = body.map_err(bad_body)?;
    if !is_valid_email(&req.email) {
        return Err(ApiError::BadRequest(MSG_BAD_EMAIL.to_string()));
    }
    if req.password.is_empty() {
        return Err(ApiError::BadRequest("비밀번호를 입력하세요".to_string()));
    }

    let row: Option<LoginRow> = sqlx::query_as(
        r#"SELECT "id", "agencyId" AS agency_id, "email", "name", "role", "passwordHash" AS password_hash
           FROM "User" WHERE "email" = $1"#,
    )
    .bind(&req.email)
    .fetch_optional(&state.db)
    .await?;
    let Some(row) = row else {
        return Err(ApiError::UnauthorizedWith(MSG_LOGIN_FAILED));
    };
    if !verify_blocking(req.password, row.password_hash.clone()).await? {
        return Err(ApiError::UnauthorizedWith(MSG_LOGIN_FAILED));
    }

    let (token, expires_at) = session::create(&state.db, &row.id).await?;
    let cookie = session_cookie(&token, expires_at, state.cfg.cookie_secure);
    Ok((
        set_cookie_header(cookie),
        Json(AuthResponse {
            token,
            user: UserDto {
                id: row.id,
                email: row.email,
                name: row.name,
                role: row.role,
                agency_id: row.agency_id,
            },
        }),
    )
        .into_response())
}

async fn logout(State(state): State<AppState>, token: SessionToken) -> ApiResult<Response> {
    session::invalidate(&state.db, &token.0).await?;
    Ok((
        StatusCode::NO_CONTENT,
        set_cookie_header(cleared_cookie(state.cfg.cookie_secure)),
    )
        .into_response())
}

async fn me(user: CurrentUser) -> Json<UserDto> {
    Json(UserDto {
        id: user.id,
        email: user.email,
        name: user.name,
        role: user.role,
        agency_id: user.agency_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_shape() {
        for ok in ["a@b.co", "john.doe+x@mail.example.com", "a_b-c@x-y.io"] {
            assert!(is_valid_email(ok), "{ok}");
        }
        for bad in [
            "",
            "a",
            "a@b",
            "a@b.c",
            "@b.com",
            ".a@b.com",
            "a..b@b.com",
            "a.@b.com",
            "a@-b.com",
            "a@b..com",
            "a b@c.com",
            "a@b.c0m",
            "한글@b.com",
        ] {
            assert!(!is_valid_email(bad), "{bad}");
        }
    }
}
