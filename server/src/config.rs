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
        let allowed_origins = std::env::var("RESM_ALLOWED_ORIGINS")
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();
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
