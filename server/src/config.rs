use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::Context;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub bind: SocketAddr,
    pub allowed_origins: Vec<String>,
    pub cookie_secure: bool,
    pub static_dir: Option<PathBuf>,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Config> {
        let database_url =
            std::env::var("DATABASE_URL").context("DATABASE_URL 환경변수가 필요합니다")?;
        let bind = std::env::var("RESM_BIND")
            .unwrap_or_else(|_| "127.0.0.1:3104".to_string())
            .parse()
            .context("RESM_BIND 형식이 올바르지 않습니다")?;
        let allowed_origins =
            parse_origins(&std::env::var("RESM_ALLOWED_ORIGINS").unwrap_or_default())?;
        let cookie_secure = matches!(
            std::env::var("RESM_COOKIE_SECURE").as_deref(),
            Ok("true" | "1")
        );
        let static_dir = std::env::var("RESM_STATIC_DIR")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .map(PathBuf::from);

        Ok(Config {
            database_url,
            bind,
            allowed_origins,
            cookie_secure,
            static_dir,
        })
    }
}

/// 쉼표로 구분된 출처 목록을 파싱한다. 빈 항목은 건너뛰고, 형식이 틀린 항목은 그 항목을 밝혀 오류로 돌려준다.
/// 유효한 출처: 헤더 값으로 파싱되고 `scheme://host[:port]` 꼴(경로·쿼리·끝 슬래시 없음).
fn parse_origins(raw: &str) -> anyhow::Result<Vec<String>> {
    raw.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|o| {
            let valid = axum::http::HeaderValue::from_str(o).is_ok()
                && o.split_once("://").is_some_and(|(scheme, rest)| {
                    !scheme.is_empty() && !rest.is_empty() && !rest.contains(['/', '?', '#', ' '])
                });
            if valid {
                Ok(o.to_string())
            } else {
                anyhow::bail!(
                    "RESM_ALLOWED_ORIGINS 항목이 올바르지 않습니다: {o:?} (scheme://host[:port] 형식, 경로·끝 슬래시 없음)"
                )
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::parse_origins;

    #[test]
    fn 유효한_출처와_빈_항목() {
        assert!(parse_origins("").unwrap().is_empty());
        let v = parse_origins(" https://tauri.localhost,, tauri://localhost ,http://localhost:5173,")
            .unwrap();
        assert_eq!(
            v,
            ["https://tauri.localhost", "tauri://localhost", "http://localhost:5173"]
        );
    }

    #[test]
    fn 잘못된_출처는_그_항목을_밝혀_실패() {
        for bad in [
            "tauri.localhost",
            "https://a.example/",
            "https://a.example/x",
            "https://a.example?x=1",
            "://a",
            "https://",
            "https://a b",
        ] {
            let err = parse_origins(&format!("https://ok.example,{bad}"))
                .unwrap_err()
                .to_string();
            assert!(err.contains(&format!("{bad:?}")), "{bad}: {err}");
            assert!(!err.contains("ok.example"), "다른 항목은 노출하지 않는다: {err}");
        }
    }
}
