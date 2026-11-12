# RESM Rust 1단계(뼈대) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rust API 서버 + React SPA + Tauri 2 셸로 로그인과 매물 관리가 Windows 앱·Android 앱·브라우저 세 곳에서 동작하게 한다. 기존 PostgreSQL `real_es` 를 스키마 변경 없이 그대로 쓴다.

**Architecture:** `server/`(axum + sqlx)가 `/api/v1` JSON API와 SPA 정적 파일을 제공한다. `web/`(Vite + React)은 원본 `c:\Wspace\real-es` 의 컴포넌트를 복사해 데이터 접근만 API 호출로 바꾼다. `app/`(Tauri 2)은 같은 SPA 묶음을 Windows·Android로 포장하고 Bearer 토큰으로 서버에 붙는다.

**Tech Stack:** Rust 1.98 · axum · tokio · sqlx(postgres, 런타임 쿼리) · bcrypt · sha2 · cuid2 / Vite · React 19 · TypeScript · Tailwind v4 · react-router · TanStack Query · vitest · Playwright / Tauri 2 · tauri-plugin-store / pnpm 10 · Node 24

**Spec:** `docs/superpowers/specs/2026-11-12-resm-rust-stage-1-design.md` — 실행자는 계획과 함께 반드시 읽는다.

## Global Constraints

- 원본 `c:\Wspace\real-es` 와 `c:\Wspace\real-es-desktop` 은 **읽기만** 한다. 파일을 고치지 않는다.
- 새 서버는 DB 마이그레이션을 실행하지 않는다. `CREATE`/`ALTER`/`DROP` 금지. 스키마의 주인은 원본 `prisma/`.
- 테이블·컬럼 이름은 따옴표로 감싼다(`"Property"`, `"userId"`). 시각 컬럼은 `TIMESTAMP(3)`(timezone 없음, UTC) → Rust `chrono::NaiveDateTime`.
- 기본키는 서버가 `cuid2::create_id()` 로 만든다. `updatedAt` 은 UPDATE마다 서버가 `now() at time zone 'utc'` 로 넣는다.
- sqlx는 컴파일 시점 매크로(`query!`)를 쓰지 않는다. `sqlx::query` / `query_as` + `FromRow`.
- 세션: 무작위 32바이트 base64url(패딩 없음) 토큰, DB에는 SHA-256 소문자 16진, 만료 30일. bcrypt cost 12. 쿠키 이름 `session`(httpOnly, SameSite=Lax, path `/`).
- 모든 매물 쿼리는 `"userId" = 현재 사용자`. 남의 id는 404.
- API 금액은 **원 단위 십진 문자열**. 만원 환산은 `web/` 에서만.
- 오류 응답은 `{ "error": "<한국어 메시지>" }`. 가입·로그인 오류 문구는 원본 `src/app/(auth)/actions.ts` 와 글자까지 같게.
- UI는 원본 컴포넌트를 복사해 쓴다. 프리미티브 자작 금지, 임의 `px`·hex 금지, 네이티브 `confirm`/`alert` 금지(`ConfirmDialog`).
- 사용자와의 응답은 한국어. `git push` 는 사용자가 명시적으로 지시할 때만.
- 서버 포트 `3104`. 운영 주소 `https://resm2.approid.team`. Tauri 식별자 `team.approid.resm`.
- Zorin 배포·Cloudflare 설정은 실행 전에 사용자 확인을 받는다.

## Review Focus

각 줄의 테스트는 해당 Task에 들어 있다.

1. 세션이 만료·삭제된 상태에서 화면을 쓰는 경우 → 서버 401, 화면은 토큰을 지우고 `/login` 으로 보낸다. 빈 화면·무한 로딩 금지. (Task 3, Task 6)
2. 금액 입력이 소수 만원(`1.5`)·콤마 포함(`12,000`)·JS 안전 정수 밖의 큰 값·숫자가 아닌 값인 경우 → 각각 15000원, 120000000원, 손실 없이 왕복, 400 오류. 500 금지. (Task 5, Task 8)
3. 같은 이메일로 동시에 가입하는 경우(unique 위반) → "이미 가입된 이메일입니다" 400. 500 금지. (Task 4)
4. 브라우저에서 `/properties/abc/edit` 를 새로고침하는 경우 → SPA가 뜬다. 반대로 없는 API 경로 `/api/v1/nope` 는 `index.html` 이 아니라 JSON 404. (Task 9)
5. 앱에서 서버에 닿지 않는 경우(네트워크 끊김·서버 중지) → 한국어 오류 토스트와 재시도 가능 상태. 흰 화면 금지. (Task 6)

## File Structure

```
resm/
├── Cargo.toml                     # workspace: server, app/src-tauri
├── pnpm-workspace.yaml            # web, app
├── package.json                   # 루트 스크립트
├── CLAUDE.md  README.md  .gitignore  .env.example
├── docs/PROJECT_GUIDE.md
├── server/
│   ├── Cargo.toml
│   ├── src/main.rs                # 설정 로드, 풀 생성, serve
│   ├── src/lib.rs                 # build_router(AppState) — 테스트가 쓰는 진입점
│   ├── src/config.rs              # Config::from_env
│   ├── src/error.rs               # ApiError → {error}
│   ├── src/auth/{mod,password,session,extract,handlers}.rs
│   ├── src/notify.rs              # notify_superadmins
│   ├── src/properties/{mod,model,repo,handlers}.rs
│   ├── src/static_files.rs        # SPA 제공 + fallback
│   └── tests/{common/mod.rs,health.rs,auth.rs,properties.rs,static_files.rs}
├── web/
│   ├── package.json  vite.config.ts  tsconfig.json  index.html  components.json
│   ├── src/main.tsx  src/router.tsx  src/globals.css
│   ├── src/lib/api.ts             # fetch 래퍼, ApiError, 토큰 처리
│   ├── src/lib/platform.ts        # isTauri, API_BASE, tokenStore
│   ├── src/lib/auth.tsx           # useMe, RequireAuth, RedirectIfAuthed
│   ├── src/lib/nav.ts
│   ├── src/lib/properties/{fields,format,color-tags,money,api}.ts
│   ├── src/lib/naver/{trade-types,property-types}.ts
│   ├── src/components/{ui,layout,data-grid}/…  logo.tsx  empty-illustration.tsx  postcode-search.tsx
│   ├── src/routes/{login,register,terms,privacy}.tsx
│   ├── src/routes/properties/{list-page,new-page,edit-page,property-list,property-form}.tsx
│   └── e2e/smoke.spec.ts
├── app/
│   ├── package.json
│   └── src-tauri/{Cargo.toml,tauri.conf.json,build.rs,src/{main,lib}.rs,capabilities/default.json,icons/,gen/android/}
└── infra/{Dockerfile,compose.zorin.yml}
```

---

### Task 0: 저장소 뼈대

**Files:**
- Create: `Cargo.toml`, `pnpm-workspace.yaml`, `package.json`, `.gitignore`, `.env.example`, `CLAUDE.md`, `README.md`, `docs/PROJECT_GUIDE.md`

- [ ] **Step 1:** `c:\Wspace\resm` 에서 `git init -b main`.
- [ ] **Step 2:** 루트 `Cargo.toml` 작성 — `[workspace] resolver = "3"`, `members = ["server"]` (`app/src-tauri` 는 Task 1에서 추가).
- [ ] **Step 3:** `pnpm-workspace.yaml` 에 `packages: ["web", "app"]`. 루트 `package.json` 은 `"private": true`, `"packageManager": "pnpm@10.34.5"`.
- [ ] **Step 4:** `.gitignore` — `target/`, `node_modules/`, `dist/`, `.env`, `.env.*`(단 `!.env.example`), `app/src-tauri/gen/android/**/build/`, `*.keystore`, `test-results/`, `playwright-report/`.
- [ ] **Step 5:** `.env.example` 작성:

```
DATABASE_URL="postgresql://real_es_app:CHANGE_ME@127.0.0.1:5432/real_es"
TEST_DATABASE_URL="postgresql://real_es_migrator:CHANGE_ME@127.0.0.1:5432/real_es_test"
RESM_BIND="127.0.0.1:3104"
RESM_ALLOWED_ORIGINS="http://tauri.localhost,https://tauri.localhost,tauri://localhost,http://localhost:5173"
RESM_COOKIE_SECURE="false"
RESM_STATIC_DIR="web/dist"
```

- [ ] **Step 6:** `CLAUDE.md` — 원본 `c:\Wspace\real-es\CLAUDE.md` 를 복사한 뒤 §6의 경로를 `web/src/components/…` 로 고치고, "UI를 새로 짜기 전에 원본 `c:\Wspace\real-es\src` 에 같은 화면이 있는지 먼저 본다" 한 줄을 §6에 추가한다.
- [ ] **Step 7:** `README.md`(개요, 디렉토리 구조, 실행 방법 자리, "현재 반영 상태" 절)와 `docs/PROJECT_GUIDE.md`("현재 참고 상태" 절 포함) 작성. 로드맵 표는 spec §4를 옮긴다.
- [ ] **Step 8:** Commit — `chore: 저장소 뼈대와 1단계 설계·계획 문서`.

---

### Task 1: 앱 셸 실험 — 우편번호 검색이 Tauri 웹뷰에서 되는가

spec이 "구현 초반에 실험"을 요구한다. 이 Task가 Android 빌드 환경도 함께 마련한다. 결과에 따라 Task 7의 구현 방식이 정해진다.

**Files:**
- Create: `app/package.json`, `app/src-tauri/{Cargo.toml,tauri.conf.json,build.rs,src/main.rs,src/lib.rs,capabilities/default.json}`, `app/src-tauri/icons/*`, `app/spike/index.html`
- Modify: 루트 `Cargo.toml`(members에 `app/src-tauri`), `docs/PROJECT_GUIDE.md`

**Interfaces:**
- Produces: `app/` Tauri 2 프로젝트(식별자 `team.approid.resm`, productName `RESM`), 그리고 `docs/PROJECT_GUIDE.md` 의 "우편번호 검색 방식" 결정 기록(`embed` | `server-iframe` | `tauri-window` 중 하나와 플랫폼별 실제 웹뷰 출처 문자열).

- [ ] **Step 1:** `app/` 에 Tauri 2 프로젝트 생성(`@tauri-apps/cli@^2`, `@tauri-apps/api@^2`). `tauri.conf.json`: `productName: "RESM"`, `identifier: "team.approid.resm"`, 창 제목 `RESM`, 기본 크기 1664×1040, `bundle.targets: ["nsis"]`. 임시로 `frontendDist: "../spike"`.
- [ ] **Step 2:** `c:\Wspace\real-es-desktop\src-tauri\icons\` 전체를 `app/src-tauri/icons/` 로 복사.
- [ ] **Step 3:** `app/spike/index.html` — 버튼 하나와 `<div id="wrap">`. 버튼을 누르면 `https://t1.daumcdn.net/mapjsapi/bundle/postcode/prod/postcode.v2.js` 를 동적으로 로드하고 `new daum.Postcode({ oncomplete }).embed(wrap)` 를 호출, 결과의 `zonecode`·`roadAddress` 와 `location.origin` 을 화면에 출력. (원본 `src/components/postcode-search.tsx` 의 로드·embed 방식과 같게.)
- [ ] **Step 4:** Windows 확인 — `pnpm --dir app tauri dev`. 기대: 검색창이 뜨고, 주소를 고르면 우편번호·도로명주소가 출력된다. 출력된 origin을 기록.
- [ ] **Step 5:** Android 빌드 환경 설치 — Android Studio(SDK Platform 최신, Build-Tools, NDK, Command-line Tools, 에뮬레이터 이미지 폰 1개·태블릿 1개). 환경 변수 `ANDROID_HOME`, `NDK_HOME` 설정. `rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android`. (JDK 21은 이미 있음.) 설치는 사용자 PC에 큰 프로그램을 까는 일이므로 **시작 전에 사용자에게 알린다.**
- [ ] **Step 6:** `pnpm --dir app tauri android init` 후 `pnpm --dir app tauri android dev` 로 에뮬레이터에서 같은 확인. 출력된 origin을 기록.
- [ ] **Step 7:** 한 곳이라도 안 되면 spec §5.4의 순서로 시도한다: (1) `server-iframe` — 검색 페이지를 임시 HTTPS 정적 호스트(또는 `tauri dev` 의 원격 URL)에 두고 iframe + `postMessage`, (2) `tauri-window` — 별도 웹뷰 창 + 이벤트. 세 플랫폼(브라우저 포함) 모두에서 되는 방식 하나로 정한다.
- [ ] **Step 8:** 결정한 방식, 플랫폼별 origin, Android 환경 설치 내용을 `docs/PROJECT_GUIDE.md` "현재 참고 상태"에 기록. `RESM_ALLOWED_ORIGINS` 예시 값이 실제 origin과 다르면 `.env.example` 을 고친다.
- [ ] **Step 9:** Commit — `feat(app): Tauri 2 셸과 우편번호 검색 실험`.

---

### Task 2: 서버 뼈대 — 설정·오류·health·테스트 DB

**Files:**
- Create: `server/Cargo.toml`, `server/src/{main,lib,config,error}.rs`, `server/tests/common/mod.rs`, `server/tests/health.rs`

**Interfaces:**
- Produces:
  - `pub struct Config { pub database_url: String, pub bind: SocketAddr, pub allowed_origins: Vec<String>, pub cookie_secure: bool, pub static_dir: Option<PathBuf> }`, `Config::from_env() -> anyhow::Result<Config>`
  - `#[derive(Clone)] pub struct AppState { pub db: PgPool, pub cfg: Arc<Config> }`
  - `pub fn build_router(state: AppState) -> axum::Router`
  - `pub enum ApiError { BadRequest(String), Unauthorized, Forbidden, NotFound, Internal(anyhow::Error) }` + `IntoResponse`(400/401/403/404/500, 본문 `{error}`; 401 "인증이 필요합니다", 404 "찾을 수 없습니다", 500 "서버 오류가 발생했습니다"·원인은 로그에만) + `impl From<sqlx::Error>`
  - `pub type ApiResult<T> = Result<T, ApiError>`
  - 테스트 헬퍼 `common::spawn_app() -> TestApp { pub addr: String, pub db: PgPool, pub client: reqwest::Client }` (임의 포트에 실제 서버를 띄움, `cookie_store` 끔), `common::unique_email() -> String`

- [ ] **Step 1: 테스트 DB 준비.** 로컬 PostgreSQL에 `real_es_test` DB를 만들고(로컬 superuser로, 소유자 `real_es_migrator`), 원본의 마이그레이션을 적용한다: `c:\Wspace\real-es` 에서 `DATABASE_URL=<TEST_DATABASE_URL> pnpm prisma migrate deploy`. 원본 파일은 바뀌지 않는다. 로컬 DB 접속 방법은 원본 `scripts/provision-local-db.mjs`·`.env.migration` 을 읽어 확인. 절차를 `README.md` 에 적는다.
- [ ] **Step 2: 실패하는 테스트** `server/tests/health.rs`:

```rust
#[tokio::test]
async fn health_reports_db_ok() {
    let app = common::spawn_app().await;
    let res = app.client.get(format!("{}/api/v1/health", app.addr)).send().await.unwrap();
    assert_eq!(res.status(), 200);
    assert_eq!(res.json::<serde_json::Value>().await.unwrap()["status"], "ok");
}

#[tokio::test]
async fn unknown_api_path_is_json_404() {
    let app = common::spawn_app().await;
    let res = app.client.get(format!("{}/api/v1/nope", app.addr)).send().await.unwrap();
    assert_eq!(res.status(), 404);
    assert!(res.json::<serde_json::Value>().await.unwrap()["error"].is_string());
}
```

- [ ] **Step 3:** `cargo test -p resm-server --test health` → 컴파일 실패(FAIL) 확인.
- [ ] **Step 4:** 위 Interfaces를 구현. 크레이트 이름 `resm-server`. `GET /api/v1/health` 는 `SELECT 1` 후 `{"status":"ok"}`. `/api/v1` 아래 미등록 경로는 `ApiError::NotFound`. 요청 로그는 `tower-http` TraceLayer.
- [ ] **Step 5:** `cargo test -p resm-server --test health` → PASS.
- [ ] **Step 6:** Commit — `feat(server): 설정·오류 응답·health`.

---

### Task 3: 인증 코어 — 비밀번호·세션·현재 사용자

**Files:**
- Create: `server/src/auth/{mod,password,session,extract}.rs`
- Test: `server/src/auth/password.rs`·`session.rs` 내 단위 테스트, `server/tests/auth.rs`(세션 부분)

**Interfaces:**
- Consumes: `AppState`, `ApiError`
- Produces:
  - `password::hash(pw: &str) -> anyhow::Result<String>` (cost 12), `password::verify(pw: &str, hash: &str) -> bool`
  - `session::new_token() -> String`, `session::hash_token(token: &str) -> String`
  - `session::create(db: &PgPool, user_id: &str) -> sqlx::Result<(String /*token*/, NaiveDateTime /*expires_at*/)>`
  - `session::invalidate(db: &PgPool, token: &str) -> sqlx::Result<()>`
  - `pub struct CurrentUser { pub id: String, pub agency_id: String, pub email: String, pub name: Option<String>, pub role: String }` — axum 추출기. `Authorization: Bearer` 가 있으면 그것, 없으면 `session` 쿠키. 세션 없음·만료 → `ApiError::Unauthorized`. 만료된 행은 삭제.
  - `pub struct SessionToken(pub String)` 추출기(로그아웃용, 같은 우선순위)

- [ ] **Step 1: 실패하는 단위 테스트.** 호환 고정값을 원본에서 만든다 — `c:\Wspace\real-es` 에서 `node -e "require('bcryptjs').hash('fixture-pw-1234',12).then(console.log)"` 의 출력을 테스트 상수 `BCRYPTJS_HASH` 로 넣는다.

```rust
#[test] fn verifies_bcryptjs_hash() { assert!(verify("fixture-pw-1234", BCRYPTJS_HASH)); assert!(!verify("wrong", BCRYPTJS_HASH)); }
#[test] fn own_hash_roundtrips() { let h = hash("abcd1234").unwrap(); assert!(h.starts_with("$2")); assert!(verify("abcd1234", &h)); }
#[test] fn token_is_43_char_base64url() { let t = new_token(); assert_eq!(t.len(), 43); assert!(t.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')); }
#[test] fn token_hash_matches_node_sha256_hex() {
    // node: crypto.createHash("sha256").update("abc").digest("hex")
    assert_eq!(hash_token("abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
}
```

- [ ] **Step 2:** `cargo test -p resm-server auth::` → FAIL.
- [ ] **Step 3:** `password.rs`, `session.rs` 구현.
- [ ] **Step 4: 실패하는 통합 테스트** `server/tests/auth.rs` — 테스트 전용 라우트 대신 Task 4의 `GET /auth/me` 를 쓰므로, 여기서는 DB 수준만: `common::insert_user(&db, email, pw) -> String /*user id*/` 헬퍼(Agency+User 직접 INSERT)를 `common` 에 추가하고,

```rust
#[tokio::test]
async fn session_row_uses_hashed_token_and_30_day_expiry() {
    let app = common::spawn_app().await;
    let uid = common::insert_user(&app.db, &common::unique_email(), "abcd1234").await;
    let (token, expires) = resm_server::auth::session::create(&app.db, &uid).await.unwrap();
    let hash: String = sqlx::query_scalar(r#"SELECT "tokenHash" FROM "Session" WHERE "userId" = $1"#).bind(&uid).fetch_one(&app.db).await.unwrap();
    assert_eq!(hash, resm_server::auth::session::hash_token(&token));
    let days = (expires - chrono::Utc::now().naive_utc()).num_days();
    assert!((29..=30).contains(&days));
}
```

- [ ] **Step 5:** `extract.rs` 구현(추출기 검증은 Task 4의 `/auth/me` 테스트가 맡는다). `cargo test -p resm-server` → PASS.
- [ ] **Step 6:** Commit — `feat(server): 인증 코어(bcrypt·세션·CurrentUser)`.

---

### Task 4: 인증 API — 가입·로그인·로그아웃·me, 알림 기록, CORS

**Files:**
- Create: `server/src/auth/handlers.rs`, `server/src/notify.rs`
- Modify: `server/src/lib.rs`(라우트, CORS, Origin 검사), `server/tests/auth.rs`

**Interfaces:**
- Consumes: Task 3 전부
- Produces:
  - `POST /api/v1/auth/signup` 본문 `{agencyName, agencyZipcode?, agencyAddress?, agencyPhone?, name, phone?, email, password, passwordConfirm, agree: bool}` → 200 `{token, user}` + `Set-Cookie`
  - `POST /api/v1/auth/login` 본문 `{email, password}` → 200 `{token, user}` + `Set-Cookie`
  - `POST /api/v1/auth/logout` → 204, 쿠키 제거
  - `GET /api/v1/auth/me` → 200 `user`
  - `user` = `{id, email, name, role, agencyId}`
  - `notify::notify_superadmins(db: &PgPool, kind: &str, title: &str, description: Option<&str>, link: Option<&str>) -> sqlx::Result<()>` — `role = 'superadmin'` 인 모든 사용자에게 `"Notification"` 행 INSERT
- 검증 순서와 문구(원본과 동일): 상호 비었음 "상호명을 입력하세요" → 이름 비었음 "이름을 입력하세요" → 이메일 형식 "올바른 이메일을 입력하세요" → 비밀번호 8자 미만 "비밀번호는 8자 이상이어야 합니다" → 불일치 "비밀번호가 일치하지 않습니다" → 미동의 "이용약관에 동의해주세요" → 중복 "이미 가입된 이메일입니다". 로그인 실패는 "이메일 또는 비밀번호가 올바르지 않습니다"(401).
- 가입 트랜잭션: `SELECT pg_advisory_xact_lock(824201409141::bigint)` → `"User"` 수가 0이면 `superadmin`, 아니면 `member` → Agency, User INSERT. 첫 가입자가 아니면 `notify_superadmins("member", "새 회원 가입: {name}", "{email} · {agencyName}", "/settings/members")`.
- CORS: `cfg.allowed_origins` 만 허용, 메서드 GET/POST/PATCH, 헤더 `authorization`·`content-type`, credentials 없음(앱은 Bearer). Origin 검사: 요청에 `Authorization` 헤더가 없고 메서드가 GET/HEAD/OPTIONS가 아닌데 `Origin` 헤더가 있으면, 그 값이 요청의 `Host` 와 같은 출처일 때만 통과(아니면 403).

- [ ] **Step 1: 실패하는 테스트** `server/tests/auth.rs` 에 추가(각 테스트는 `unique_email()` 사용):
  - `signup_then_me_with_bearer` — 가입 200, 응답 `token` 으로 `GET /auth/me` → 같은 이메일.
  - `signup_sets_session_cookie` — `Set-Cookie` 에 `session=`, `HttpOnly`, `SameSite=Lax`, `Path=/`.
  - `login_accepts_cookie_auth` — 로그인 응답 쿠키를 `Cookie` 헤더로 보내 `GET /auth/me` 200.
  - `login_wrong_password_401_message` — 본문 `error == "이메일 또는 비밀번호가 올바르지 않습니다"`.
  - `signup_validation_messages` — 위 문구 6종을 표로 돌려 각각 400과 문구 일치 확인.
  - `duplicate_email_is_400_not_500` — 같은 이메일로 `tokio::join!` 동시 가입 2건 → 하나 200, 하나 400 "이미 가입된 이메일입니다". *(Review Focus 3)*
  - `non_first_signup_is_member_and_notifies` — 가입자 role `member`, 가입 전후 `"Notification"` 중 `title = '새 회원 가입: {name}'` 행 수가 superadmin 수만큼 증가.
  - `logout_invalidates_token` — 로그아웃 204 뒤 같은 토큰으로 `/auth/me` → 401.
  - `expired_session_is_401_and_row_deleted` — `"Session"."expiresAt"` 을 과거로 UPDATE 후 `/auth/me` 401, 행 수 0. *(Review Focus 1)*
  - `garbage_bearer_is_401` — `Authorization: Bearer xxx` → 401.
  - `cross_origin_cookie_post_is_403` — 쿠키 인증 + `Origin: https://evil.example` 로 `POST /auth/logout` → 403.
  - `preflight_allows_configured_origin` — `OPTIONS /api/v1/auth/login` + `Origin: http://tauri.localhost` → `access-control-allow-origin` 이 그 값.
- [ ] **Step 2:** `cargo test -p resm-server --test auth` → FAIL.
- [ ] **Step 3:** `handlers.rs`, `notify.rs`, 라우트·CORS·Origin 검사 구현. 중복 이메일은 사전 조회와 unique 위반(SQLSTATE `23505`) 둘 다 같은 400으로 바꾼다.
- [ ] **Step 4:** `cargo test -p resm-server` → PASS.
- [ ] **Step 5: 원본과 교차 확인(수동).** 로컬 `real_es` DB를 가리켜 `cargo run -p resm-server` 실행, 원본 웹에서 쓰던 계정으로 `POST /auth/login` → 200. 받은 쿠키 값을 원본 웹(로컬 `pnpm dev`)의 `session` 쿠키로 넣었을 때 로그인 상태가 되는지 확인. 결과를 PROJECT_GUIDE에 한 줄.
- [ ] **Step 6:** Commit — `feat(server): 인증 API와 알림 기록`.

---

### Task 5: 매물 API

**Files:**
- Create: `server/src/properties/{mod,model,repo,handlers}.rs`, `server/tests/properties.rs`
- Modify: `server/src/lib.rs`

**Interfaces:**
- Consumes: `CurrentUser`, `ApiError`, `notify_superadmins`
- Produces (JSON 키는 DB 컬럼명과 같은 camelCase):
  - `PropertyDto`: `id`, `isFavorite: bool`, `colorTag: string|null`, `status`, `source`, `createdAt`, 그리고 아래 필드 전부(없으면 null)
    - 문자열: `name articleNo complexName address realEstateType tradeType heating customerName customerPhone partnerName partnerPhone partnerManager manager contractHopeDate contractDate moveInHopeDate moveInDate interim1Date interim2Date balanceDate approvalDate note memo`
    - 실수: `siteArea areaExclusive areaSupply landArea buildingArea area`
    - 정수: `totalHouseholds parkingCount`
    - 불리언: `isPreSale`
    - 금액(원, 십진 문자열): `price rentPrice`
  - `"dealAmount"`·`"contractChecklist"` 컬럼은 읽지도 쓰지도 않는다.
  - `GET /api/v1/properties?view=all|favorites|in-progress|contracted`(기본 `all`) → `PropertyDto[]`, `"createdAt" DESC`. `favorites` = `"isFavorite"`, `in-progress` = `status='계약진행'`, `contracted` = `status='계약완료'`. 알 수 없는 view는 400.
  - `GET /api/v1/properties/{id}` → `PropertyDto` | 404
  - `POST /api/v1/properties` 본문 = 수정 가능한 필드의 부분집합 → 201 `{id}`. `source` 기본 "수기", `status` 기본 "진행". 이후 `notify_superadmins("property", "새 매물 등록: {name ?? complexName ?? "매물"}", "{user.name ?? user.email}님이 등록", "/properties/{id}/edit")`.
  - `PATCH /api/v1/properties/{id}` → 204 | 404. 키 없음 = 그대로, `null` = 비움. `status` 가 null이면 무시. 수정 가능: 위 필드 + `status` + `isFavorite` + `colorTag`. `id`·`userId`·`source`·`createdAt` 은 무시.
  - `POST /api/v1/properties/delete` 본문 `{ids: string[]}` → 200 `{count}`(본인 것만 지워진 수)
- 검증(위반 시 400): `status ∈ {진행, 계약진행, 계약완료}` 아니면 "상태 값이 올바르지 않습니다"; `colorTag ∈ {purple, blue, orange, gray}` 또는 null, 아니면 "색상 값이 올바르지 않습니다"; 금액은 `^\d{1,18}$` 문자열(또는 JSON 정수) 아니면 "금액 형식이 올바르지 않습니다"; 타입이 안 맞는 JSON(문자열 자리에 객체 등)은 "요청 형식이 올바르지 않습니다".
- "없음/null" 구분은 `Option<Option<T>>` + `#[serde(default, deserialize_with = …)]`(double option) 패턴으로 한다. 동적 UPDATE는 `sqlx::QueryBuilder`.

- [ ] **Step 1: 실패하는 테스트** `server/tests/properties.rs` (`common::signup(&app) -> (token, user_id)` 헬퍼 추가):
  - `create_defaults_and_get` — `{name:"테스트"}` 등록 → GET에서 `status=="진행"`, `source=="수기"`, `isFavorite==false`, 나머지 null.
  - `money_roundtrips_as_string` — `price:"9007199254740993"`(2^53+1) → GET에서 같은 문자열. *(Review Focus 2)*
  - `money_rejects_non_numeric` — `price:"1.5"`, `"abc"`, `"-1"` → 400 "금액 형식이 올바르지 않습니다". *(Review Focus 2)*
  - `patch_absent_keeps_null_clears` — `name`·`memo` 있는 매물에 `{memo:null}` → `memo==null`, `name` 유지.
  - `patch_null_status_ignored_bad_status_400`.
  - `patch_color_tag_validated` — `"red"` 400, `"blue"` 204, `null` 204.
  - `patch_sets_updated_at` — PATCH 전후 `"updatedAt"` 증가.
  - `views_filter` — 관심 1, 계약진행 1, 계약완료 1, 일반 1 만든 뒤 view별 개수 1/1/1, all 4. `view=xyz` 400.
  - `list_is_created_desc`.
  - `other_users_property_is_404` — 사용자 B가 A의 매물 GET·PATCH → 404, delete → `count==0`, A의 매물은 그대로.
  - `unauthenticated_is_401`.
  - `create_notifies_superadmins` — title `새 매물 등록: 테스트` 행 증가.
  - `unknown_keys_ignored_wrong_types_400` — `{bogus:1, name:"a"}` 201; `{name:{}}` 400.
- [ ] **Step 2:** `cargo test -p resm-server --test properties` → FAIL.
- [ ] **Step 3:** `model.rs`(Dto, 입력 구조체, 검증) → `repo.rs`(쿼리) → `handlers.rs` 구현. BIGINT ↔ 문자열 변환은 `model.rs` 한 곳.
- [ ] **Step 4:** `cargo test -p resm-server` → PASS. `cargo clippy -p resm-server -- -D warnings` 통과.
- [ ] **Step 5:** Commit — `feat(server): 매물 API`.

---

### Task 6: 화면 뼈대 — Vite·컴포넌트 이식·API 클라이언트·인증 가드·셸

**Files:**
- Create: `web/` 전체 뼈대(File Structure 참고), `web/src/lib/{api,platform,auth,nav}.ts(x)`, `web/src/lib/api.test.ts`
- Copy from 원본: `src/components/ui/` → `web/src/components/ui/`, `src/components/layout/{app-sidebar,app-header,app-footer,theme-provider,theme-toggle,breadcrumbs}.tsx`, `src/components/{logo,empty-illustration}.tsx`, `src/components/data-grid/editable-cell.tsx`, `src/hooks/`, `src/lib/utils.ts`, `src/app/globals.css`, `src/app/icon.svg`(→ `web/public/`), `components.json`

**Interfaces:**
- Produces:
  - `platform.ts`: `isTauri(): boolean`, `API_BASE: string`(브라우저 `"/api/v1"`, 앱 `import.meta.env.VITE_API_BASE`), `tokenStore: { get(): Promise<string|null>; set(t: string): Promise<void>; clear(): Promise<void> }`(앱: `@tauri-apps/plugin-store` 의 `auth.json` 키 `token`; 브라우저: 아무것도 안 함 — 쿠키가 맡음)
  - `api.ts`: `class ApiError extends Error { status: number }`, `api<T>(path: string, init?: { method?: string; body?: unknown }): Promise<T>` — 앱이면 `Authorization: Bearer`, 브라우저면 `credentials: "same-origin"`. 응답이 실패면 본문 `error` 를 메시지로 `ApiError`. 401이면 `tokenStore.clear()` 후 `onUnauthorized` 콜백 호출. `fetch` 자체가 실패(네트워크)하면 `ApiError(status 0, "서버에 연결할 수 없습니다")`.
  - `auth.tsx`: `type Me = { id: string; email: string; name: string|null; role: string; agencyId: string }`, `useMe()`(TanStack Query, 키 `["me"]`, 401이면 `null`), `login(email, password)`, `signup(input)`, `logout()`, `<RequireAuth>`(비로그인 → `/login`), `<RedirectIfAuthed>`(로그인 → `/properties`)
  - `router.tsx`: spec §5.4의 경로 전부. `/` → `/properties`. 보호 경로는 `<RequireAuth>` + 셸 레이아웃(원본 `(dashboard)/layout.tsx` 와 같은 구조: `SidebarProvider` / `AppSidebar` / `SidebarInset` / `AppHeader` / `main.flex.flex-1.flex-col.gap-4.p-4` / `AppFooter`).
  - `nav.ts`: 원본 `navGroups` 중 "매물 관리" 그룹만.

- [ ] **Step 1:** `web/` 생성 — Vite(React + TS), Tailwind v4(`@tailwindcss/vite`), 경로 별칭 `@` → `web/src`. 의존 버전은 원본 `package.json` 과 같은 메이저로 맞춘다(react 19, `@base-ui/react`, lucide-react, sonner, cmdk, date-fns, react-day-picker, tw-animate-css 등 복사한 컴포넌트가 import하는 것만). 추가: `react-router`, `@tanstack/react-query`, `@tauri-apps/api`, `@tauri-apps/plugin-store`. dev 서버 포트 5173, `/api` 를 `http://127.0.0.1:3104` 로 프록시.
- [ ] **Step 2:** 위 "Copy from 원본" 목록을 복사. Next 전용 import를 바꾼다: `next/link` → `react-router` 의 `Link`(`href` → `to`), `next/navigation` 의 `usePathname`/`useRouter` → `useLocation`/`useNavigate`, `next-themes` → 그대로 사용(Next 없이 동작), `next/font` → 제거하고 원본 `globals.css` 의 시스템 폰트 스택 유지, `"use client"` 지시문 삭제. `app-header.tsx` 는 서버 컴포넌트였으므로 `useMe()` 로 이니셜을 얻도록 고치고 알림 벨과 `/profile` 링크를 뺀다(이니셜 아바타는 링크 없이 표시, 로그아웃 메뉴 포함). 사이드바에서 ⌘K 팔레트 진입은 뺀다.
- [ ] **Step 3: 실패하는 테스트** `web/src/lib/api.test.ts`(vitest, `fetch` 모킹):
  - `returns parsed json on 200`
  - `throws ApiError with server message on 400` — 본문 `{error:"상호명을 입력하세요"}` → `message` 일치, `status===400`
  - `401 clears token and calls onUnauthorized` *(Review Focus 1)*
  - `network failure becomes ApiError status 0 with "서버에 연결할 수 없습니다"` *(Review Focus 5)*
  - `204 returns undefined`
- [ ] **Step 4:** `pnpm --dir web test` → FAIL → `platform.ts`, `api.ts` 구현 → PASS.
- [ ] **Step 5:** `auth.tsx`, `router.tsx`, `main.tsx`(QueryClient, ThemeProvider `attribute="class" defaultTheme="system"`, `Toaster richColors closeButton`, `<html lang="ko">`) 구현. `onUnauthorized` 는 `["me"]` 캐시를 `null` 로 만들고 `/login` 으로 이동. 쿼리·뮤테이션의 `ApiError` 는 전역에서 `toast.error(message)` 로 보여 준다. `useMe` 가 status 0으로 실패하면 로그인으로 보내지 말고 "서버에 연결할 수 없습니다" 문구와 "다시 시도" 버튼이 있는 화면(원본 `Empty` 컴포넌트)을 보인다. *(Review Focus 5)*
- [ ] **Step 6:** 보호 경로에 임시 자리 표시 페이지를 두고 확인 — `cargo run -p resm-server` + `pnpm --dir web dev`. 비로그인 `/properties` → `/login` 이동. `pnpm --dir web build` 와 `tsc --noEmit` 통과.
- [ ] **Step 7:** Commit — `feat(web): SPA 뼈대·컴포넌트 이식·API 클라이언트`.

---

### Task 7: 인증 화면 — 로그인·회원가입·약관·우편번호

**Files:**
- Create: `web/src/routes/{login,register,terms,privacy}.tsx`, `web/src/components/postcode-search.tsx`
- Copy from 원본: `src/app/(auth)/{layout.tsx,login/page.tsx,register/page.tsx}`, `src/app/{terms,privacy}/page.tsx`, `src/components/postcode-search.tsx`, `src/components/auth/` 중 두 페이지가 import하는 것

**Interfaces:**
- Consumes: `login`, `signup`, `RedirectIfAuthed`, `formatTel`(Task 8에서 복사하는 `format.ts` — 이 Task에서 먼저 복사한다), Task 1의 우편번호 방식 결정
- Produces: `<PostcodeSearch onComplete={({ zonecode, address }) => …} />` — 원본과 같은 props. 내부 구현은 Task 1에서 정한 방식(`embed` 면 원본 그대로).

- [ ] **Step 1:** 원본 로그인·회원가입 페이지를 복사해 `useActionState`/FormData 서버 액션을 `login()`/`signup()` 호출로 바꾼다. 오류는 원본과 같은 자리에 서버의 `error` 문구를 표시. 성공 시 `["me"]` 갱신 후 `/properties`. 로그인 화면의 "비밀번호 찾기" 링크는 뺀다. 회원가입은 원본 동작 유지: 전화 자동 하이픈, 우편번호 검색, 비밀번호 확인, 약관 미동의 시 버튼 비활성, `/terms`·`/privacy` 새 탭 링크.
- [ ] **Step 2:** 약관 두 페이지를 복사(본문 그대로, "돌아가기" 링크 `/register`).
- [ ] **Step 3:** `PostcodeSearch` 를 Task 1 결정대로 구현. `server-iframe` 방식이면 서버에 정적 페이지 `GET /postcode.html` 을 추가하고(`server/src/static_files.rs` 는 Task 9에서 만들므로 이 경우 Task 9의 Step 1~4를 먼저 한다) `postMessage` 의 origin을 검사한다.
- [ ] **Step 4: 브라우저 수동 확인.** 가입(우편번호 검색 → 선택 → 우편번호·주소 자동 입력 포함) → `/properties` 도착 → 로그아웃 → 로그인. 원본 웹과 나란히 띄워 로그인·가입 화면이 같은지 라이트/다크에서 비교.
- [ ] **Step 5:** `tsc --noEmit`, `pnpm --dir web build` 통과. Commit — `feat(web): 로그인·회원가입·약관`.

---

### Task 8: 매물 화면 — 목록 4뷰·등록·수정

**Files:**
- Create: `web/src/lib/properties/{money,api}.ts`, `web/src/lib/properties/money.test.ts`, `web/src/routes/properties/{list-page,new-page,edit-page}.tsx`
- Copy from 원본: `src/lib/properties/{fields,format,color-tags}.ts`(+`fields.test.ts`, `format.test.ts`), `src/lib/naver/{trade-types,property-types}.ts`(+`property-types.test.ts`), `src/app/(dashboard)/properties/{property-list,property-form}.tsx` → `web/src/routes/properties/`

**Interfaces:**
- Consumes: `api`, 셸 레이아웃, Task 5의 API
- Produces:
  - `money.ts`: `wonToManwon(won: string | null): string`(원 문자열 → 만원 표시값, 예 `"15000"` → `"1.5"`, `"120000000"` → `"12000"`, null → `""`), `manwonToWon(input: string): string | null`(콤마·공백 제거, 예 `"12,000"` → `"120000000"`, `"1.5"` → `"15000"`, `""` → `null`; 숫자가 아니면 `throw new Error("금액 형식이 올바르지 않습니다")`). 큰 값도 손실 없게 `BigInt`/문자열 연산으로 한다(`Number` 곱셈 금지).
  - `api.ts`: `type PropertyRow`(Task 5의 `PropertyDto` 와 같은 키), `type PropertyView = "all" | "favorites" | "in-progress" | "contracted"`, `listProperties(view)`, `getProperty(id)`, `createProperty(input): Promise<{id: string}>`, `updateProperty(id, patch)`, `deleteProperties(ids): Promise<{count: number}>`. 쿼리 키 `["properties", view]`, `["property", id]`. 변경 뒤에는 `["properties"]` 전체 무효화.

- [ ] **Step 1: 실패하는 테스트** `money.test.ts`:

```ts
expect(wonToManwon("120000000")).toBe("12000");
expect(wonToManwon("15000")).toBe("1.5");
expect(wonToManwon(null)).toBe("");
expect(manwonToWon("12,000")).toBe("120000000");
expect(manwonToWon("1.5")).toBe("15000");
expect(manwonToWon("")).toBeNull();
expect(manwonToWon("900719925474.0993")).toBe("9007199254740993");
expect(() => manwonToWon("abc")).toThrow("금액 형식이 올바르지 않습니다");
```

- [ ] **Step 2:** `pnpm --dir web test` → FAIL → `money.ts` 구현 → 복사한 원본 테스트 포함 전부 PASS. *(Review Focus 2)*
- [ ] **Step 3:** `api.ts` 구현.
- [ ] **Step 4:** `property-list.tsx` 이식. 서버 액션 호출을 `api.ts` 로 바꾸고 `router.refresh()` 는 쿼리 무효화로 바꾼다. 금액 표시·인라인 편집은 `money.ts` 로(원본의 `÷10000`·`×10000` 계산을 대체). 유지: 표(lg↑)/카드(lg↓), 넘버링, 검색, 유형·거래·상태 필터, 50개 페이징, 체크박스·선택 삭제(`ConfirmDialog`), 더블클릭 인라인 편집, 관심 별 토글, 색상 태그·색 필터(`view === "all"` 일 때만), 빈 상태 일러스트. 제거(코드째 삭제하고 2·3단계에서 원본을 다시 참고): "엑셀 입력"·"엑셀" 버튼, "계약진행 전환" 버튼, 행별 "계약" 링크, 고객명 옆 "미등록" 아이콘.
- [ ] **Step 5:** `list-page.tsx` — 경로별로 `view` 를 정해 `<PropertyList view rows />` 렌더(4개 경로가 한 컴포넌트 공유). 로딩은 원본 `Skeleton`.
- [ ] **Step 6:** `property-form.tsx` 이식 + `new-page.tsx`·`edit-page.tsx`. 금액은 `manwonToWon` 으로 바꿔 보낸다. 저장 후 `/properties`. 수정 화면에서 "당사자" 섹션과 "고객으로 등록" 버튼은 뺀다. 없는 id면 원본 `Empty` 로 "매물을 찾을 수 없습니다" + 목록 링크.
- [ ] **Step 7: 브라우저 수동 확인.** 등록 → 목록에 표시 → 인라인 편집 → 폼 수정 → 관심·색상 → 4개 뷰 → 검색·필터·페이징 → 선택 삭제. 창 폭을 줄여 카드 목록 전환 확인. 원본 웹과 나란히 비교(라이트/다크). 원본 웹에서 같은 계정으로 봤을 때 새 앱에서 만든 매물이 같은 금액으로 보이는지 확인.
- [ ] **Step 8:** `tsc --noEmit`, `pnpm --dir web test`, `pnpm --dir web build` 통과. Commit — `feat(web): 매물 목록·등록·수정`.

---

### Task 9: 서버가 SPA를 제공 + 브라우저 E2E

**Files:**
- Create: `server/src/static_files.rs`, `server/tests/static_files.rs`, `web/e2e/smoke.spec.ts`, `web/playwright.config.ts`
- Modify: `server/src/lib.rs`, 루트 `package.json`(스크립트 `build:web`, `dev:server`, `test:e2e`)

**Interfaces:**
- Consumes: `Config.static_dir`
- Produces: `static_files::service(dir: &Path) -> axum::Router` — 파일이 있으면 그 파일, 없으면 `index.html`(200). `/api/` 로 시작하는 경로에는 적용되지 않는다. `assets/` 아래는 `Cache-Control: public, max-age=31536000, immutable`, `index.html` 은 `no-cache`.

- [ ] **Step 1: 실패하는 테스트** `server/tests/static_files.rs` — 임시 디렉터리에 `index.html`(`<title>RESM</title>`)과 `assets/a.js` 를 만들고 `static_dir` 로 지정:
  - `deep_link_serves_index` — `GET /properties/abc/edit` → 200, 본문에 `RESM`. *(Review Focus 4)*
  - `asset_is_served_with_long_cache`
  - `unknown_api_path_still_json_404` — `GET /api/v1/nope` → 404, `content-type` 이 `application/json`. *(Review Focus 4)*
  - `no_static_dir_means_404_for_pages` — `static_dir: None` 이면 `GET /` 404.
- [ ] **Step 2:** FAIL 확인 → 구현(`tower-http` `ServeDir` + fallback) → `cargo test -p resm-server` PASS.
- [ ] **Step 3:** `web/e2e/smoke.spec.ts`(Playwright, `baseURL http://127.0.0.1:3104`, 서버는 `TEST_DATABASE_URL` 로 띄움):
  - 가입 → `/properties` 도착
  - 매물 등록(매물명 `E2E 매물`, 가격 `12,000`) → 목록에 `E2E 매물` 과 `12,000` 표시
  - 수정 폼에서 매물명을 `E2E 매물 수정` 으로 → 목록 반영
  - `/properties/<id>/edit` 에서 새로고침 → 폼이 다시 뜸
  - 선택 삭제 → 빈 상태
  - 로그아웃 → `/login`, 이후 `/properties` 직접 접근 → `/login`
  - 뷰포트 390×844에서 목록이 카드로 보임(표 숨김)
- [ ] **Step 4:** `pnpm build:web && pnpm test:e2e` → PASS.
- [ ] **Step 5:** `README.md` 에 개발 실행법(서버, 웹 dev, 테스트 3종) 기록. Commit — `feat(server): SPA 정적 제공과 브라우저 E2E`.

---

### Task 10: Windows 앱

**Files:**
- Modify: `app/src-tauri/{tauri.conf.json,Cargo.toml,src/lib.rs,capabilities/default.json}`, `app/package.json`
- Delete: `app/spike/`

**Interfaces:**
- Consumes: `web/dist`, `tokenStore`(Task 6), Task 1에서 기록한 Windows origin
- Produces: `app/src-tauri/target/release/bundle/nsis/RESM_0.1.0_x64-setup.exe`

- [ ] **Step 1:** `tauri.conf.json` — `frontendDist: "../../web/dist"`, `devUrl: "http://localhost:5173"`, `beforeDevCommand`/`beforeBuildCommand` 로 web의 dev/build 실행. `tauri-plugin-store` 등록, capability에 `store:default`. CSP는 서버 주소와 Task 1에서 정한 우편번호 방식에 필요한 출처(`t1.daumcdn.net`, `postcode.map.daum.net` 등 실제 필요한 것)만 허용하도록 설정한다.
- [ ] **Step 2:** 앱 빌드용 환경 파일 — `web/.env.tauri-dev`(`VITE_API_BASE=http://127.0.0.1:3104/api/v1`), `web/.env.tauri-prod`(`VITE_API_BASE=https://resm2.approid.team/api/v1`). 루트 스크립트 `app:dev`, `app:build:win`(prod 주소로 빌드).
- [ ] **Step 3:** `pnpm app:dev` 로 로컬 서버에 붙여 수동 확인 — 가입(우편번호 포함)·로그인 → 앱을 껐다 켜도 로그인 유지(토큰 저장) → 매물 등록·수정·삭제 → 로그아웃. 서버를 끄고 앱을 다시 열어 "서버에 연결할 수 없습니다" 화면과 "다시 시도" 동작 확인. *(Review Focus 5)*
- [ ] **Step 4:** CORS가 막으면 Task 1의 기록과 실제 origin을 대조해 `RESM_ALLOWED_ORIGINS` 를 고친다(서버 코드에 origin을 하드코딩하지 않는다).
- [ ] **Step 5:** `pnpm app:build:win` → NSIS 설치본 생성 확인. (운영 서버가 아직 없으므로 설치본 실행 확인은 Task 12에서.)
- [ ] **Step 6:** `app/spike/` 삭제. Commit — `feat(app): Windows 앱`.

---

### Task 11: Android 앱

**Files:**
- Modify: `app/src-tauri/gen/android/` (필요한 설정만), `app/src-tauri/tauri.conf.json`, 루트 `package.json`

**Interfaces:**
- Consumes: Task 1의 Android 환경·origin, Task 10의 설정
- Produces: 디버그 APK(`app/src-tauri/gen/android/app/build/outputs/apk/**/app-*-debug.apk`), 스크립트 `app:android:dev`, `app:build:android`

- [ ] **Step 1:** 디버그 빌드에서만 평문 HTTP 허용 — `gen/android/app/src/debug/` 의 매니페스트/네트워크 보안 설정으로 `10.0.2.2` cleartext 허용. 배포(release) 설정은 건드리지 않는다. 디버그용 `web/.env.tauri-android-dev`(`VITE_API_BASE=http://10.0.2.2:3104/api/v1`).
- [ ] **Step 2:** 로컬 서버를 `RESM_BIND=0.0.0.0:3104` 로 띄우고 `pnpm app:android:dev` — **폰 에뮬레이터**에서 가입(우편번호 포함)·로그인·앱 재시작 후 로그인 유지·매물 카드 목록·카드 눌러 수정·삭제 확인.
- [ ] **Step 3:** **태블릿 에뮬레이터**(가로)에서 표 레이아웃과 사이드바 확인. 화면 잘림·소프트 키보드가 입력 칸을 가리는 문제·시스템 바 겹침(safe area)이 있으면 원본 토큰·유틸 클래스 범위 안에서 고친다.
- [ ] **Step 4:** Android 웹뷰의 실제 origin이 `RESM_ALLOWED_ORIGINS` 에 있는지 확인, 없으면 추가.
- [ ] **Step 5:** `pnpm app:build:android`(prod 주소, 디버그 서명 APK) → APK 생성 확인.
- [ ] **Step 6:** `README.md` 에 Android 빌드·실행법 기록. Commit — `feat(app): Android 앱`.

---

### Task 12: Zorin 배포와 전 플랫폼 확인

**Files:**
- Create: `infra/Dockerfile`, `infra/compose.zorin.yml`, `infra/README.md`
- Modify: `README.md`, `docs/PROJECT_GUIDE.md`

**Interfaces:**
- Consumes: 전부
- Produces: `https://resm2.approid.team` 에서 동작하는 새 서버

- [ ] **Step 1:** `infra/Dockerfile` — 다단계: (a) Node로 `web` 빌드, (b) Rust로 `resm-server` release 빌드, (c) 슬림 런타임 이미지에 바이너리 + `web/dist`, 비루트 사용자, `EXPOSE 3104`. `infra/compose.zorin.yml` — 서비스 `resm-server` 하나: `127.0.0.1:3104:3104`, env 파일(`DATABASE_URL` 은 기존 스택의 `real_es_app` 접속 정보, `RESM_BIND=0.0.0.0:3104`, `RESM_COOKIE_SECURE=true`, `RESM_STATIC_DIR`, `RESM_ALLOWED_ORIGINS`), 기존 네트워크 `real-es-backend`·`real-es-frontend` 에 `external: true` 로 참여, healthcheck `GET /api/v1/health`. 기존 스택의 컨테이너·볼륨·compose 파일은 수정하지 않는다. 비밀 값은 저장소에 넣지 않는다.
- [ ] **Step 2:** 로컬에서 `docker build` 와 컨테이너 기동(로컬 DB 대상)으로 health 200 확인.
- [ ] **Step 3: 사용자 확인.** 배포 절차(Zorin `~/apps/resm` 에 올림, 기존 PostgreSQL 컨테이너에 접속, 기존 cloudflared 터널에 `resm2.approid.team` → `http://resm-server:3104` 공개 호스트명 추가)를 사용자에게 보여 주고 승인받는다. Cloudflare 대시보드 작업은 사용자가 하거나 사용자가 지시한 방법으로 한다. 승인 전에는 Zorin에 아무것도 하지 않는다.
- [ ] **Step 4:** 배포 후 `https://resm2.approid.team/api/v1/health` 200, `https://resm.approid.team` 기존 웹 정상(health 200) 확인.
- [ ] **Step 5: 완료 기준 확인(spec §5.1).**
  1. 기존 운영 계정으로 `https://resm2.approid.team` 로그인.
  2. 브라우저 / Windows 설치본(Task 10의 exe 설치·실행) / Android APK(실기 또는 에뮬레이터, prod 주소) 각각에서 로그인 → 목록 → 검색·필터 → 등록 → 수정 → 삭제.
  3. 새 앱에서 등록한 매물이 `resm.approid.team` 에 보이고, 기존 웹에서 등록한 매물이 새 앱에 보인다. 기존 웹 관리자 계정의 알림에 "새 매물 등록" 이 찍힌다.
  4. 세 곳 모두 원본과 같은 디자인(라이트/다크).
  5. 폰 폭 카드 / 넓은 폭 표.
  6. 세 곳 모두 회원가입의 우편번호 검색 동작.
  실패 항목은 해당 Task로 돌아가 고친다.
- [ ] **Step 6:** `README.md` "현재 반영 상태"와 `docs/PROJECT_GUIDE.md` "현재 참고 상태"에 1단계 결과를 기록(배포 구조, 실행법, 알려진 제한: 서명 없음·자동 업데이트 없음). `infra/README.md` 에 배포·롤백 절차.
- [ ] **Step 7:** Commit — `feat(deploy): Zorin 배포 구성과 1단계 완료 기록`.
