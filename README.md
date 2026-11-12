# RESM

중개사무소용 부동산 업무 도구. 기존 Next.js 웹앱(`c:\Wspace\real-es`)을 Rust(axum) API 서버 + React SPA + Tauri 2 앱(Windows, Android)으로 다시 만드는 프로젝트다. 브라우저 접속도 계속 지원하고, 같은 PostgreSQL `real_es` DB를 쓴다.

## 디렉토리 구조

```
resm/
├── Cargo.toml            # cargo workspace (server, 이후 app/src-tauri)
├── pnpm-workspace.yaml   # web, app
├── server/               # axum API 서버
├── web/                  # React SPA
├── app/                  # Tauri 2 셸 (src-tauri/, Windows + Android)
├── docs/                 # PROJECT_GUIDE.md, superpowers/specs·plans
├── CLAUDE.md             # 작업 규칙
└── README.md
```

`server/`, `web/`(화면 뼈대까지) 를 만들었다.

## 기술 스택

- 서버: Rust, axum + tokio + sqlx(PostgreSQL)
- 화면: Vite + React 19 + TypeScript + Tailwind v4, react-router, TanStack Query
- 앱: Tauri 2 (Windows NSIS, Android APK)

## 실행 방법

서버·화면·앱이 만들어지면 여기에 적는다. 환경변수 목록은 `.env.example` 참고.

### 개발 DB (서버 테스트용)

원본(`c:\Wspace\real-es`)의 로컬 PostgreSQL 컨테이너(`property-manager-postgres`)가 꺼져 있어, 이 저장소 전용 컨테이너를 따로 쓴다. 원본 파일은 바꾸지 않는다.

```bash
docker run -d --name resm-dev-pg --restart unless-stopped \
  -e POSTGRES_PASSWORD=<임의 비밀번호> -p 127.0.0.1:5433:5432 \
  -v resm-dev-pg-data:/var/lib/postgresql/data postgres:16
docker exec resm-dev-pg psql -U postgres -c "CREATE DATABASE real_es" -c "CREATE DATABASE real_es_test"
# 테스트 DB 한 번만: 기본 시간대를 Asia/Seoul 로. 서버가 세션을 UTC 로 고정하는 처리가 테스트에서 실제로 검증되게 한다.
docker exec resm-dev-pg psql -U postgres -c "ALTER DATABASE real_es_test SET timezone TO 'Asia/Seoul'"
# 원본 마이그레이션 적용 (원본 폴더에서, DATABASE_URL 만 덮어쓴다)
cd c:\Wspace\real-es
DATABASE_URL="postgresql://postgres:<비밀번호>@127.0.0.1:5433/real_es?schema=public" pnpm prisma migrate deploy
DATABASE_URL="postgresql://postgres:<비밀번호>@127.0.0.1:5433/real_es_test?schema=public" pnpm prisma migrate deploy
```

`.env.example` 을 `.env` 로 복사해 `DATABASE_URL`·`TEST_DATABASE_URL` 을 위 주소(`127.0.0.1:5433`)로 맞춘다. `.env` 는 git에 올리지 않는다.

### 환경변수

이름만 적는다(값은 `.env`, 예시는 `.env.example`): `DATABASE_URL`, `TEST_DATABASE_URL`, `RESM_BIND`, `RESM_ALLOWED_ORIGINS`, `RESM_COOKIE_SECURE`, `RESM_STATIC_DIR`.

- 서버 실행에는 `DATABASE_URL` 이 필요하고, 서버 테스트와 E2E 에는 `TEST_DATABASE_URL` 이 필요하다.
- `RESM_STATIC_DIR` 를 `web/dist` 로 두면 서버가 SPA 를 같이 제공한다(없는 경로는 `index.html`, `/api/...` 는 JSON 404). 비워 두면 API 만 제공한다.

### 서버

```bash
pnpm dev:server                   # cargo run -p resm-server, 127.0.0.1:3104
pnpm build:web                    # 서버가 SPA 도 제공하게 하려면 RESM_STATIC_DIR=web/dist 로 실행
```

### 웹 (React SPA)

```bash
pnpm install
pnpm dev:web      # http://localhost:5173 (/api 는 127.0.0.1:3104 로 프록시, 서버를 같이 띄운다)
pnpm build:web    # web/dist
```

### Windows 앱 (Tauri)

```bash
pnpm dev:server     # 서버는 따로 먼저 띄운다 (app:dev 는 서버를 띄우지 않는다)
pnpm app:dev        # 로컬 서버(127.0.0.1:3104)에 붙는 앱 창 (web/.env.tauri-dev, origin http://localhost:5173)
pnpm app:build:win  # 운영 주소(resm2.approid.team)로 NSIS 설치본: target/release/bundle/nsis/RESM_0.1.0_x64-setup.exe
```

- 설치된 앱의 origin 은 `https://tauri.localhost` 다(`useHttpsScheme`). 서버의 `RESM_ALLOWED_ORIGINS` 에 이 값이 있어야 한다.

- 앱 빌드는 `web/dist` 를 앱 모드(`vite build --mode tauri-prod`)로 덮어쓴다. 브라우저 화면은 실행 시점에 `/api/v1` 을 쓰므로 그대로 서버가 제공해도 동작하지만, 헷갈리지 않게 브라우저용은 `pnpm build:web` 으로 다시 빌드한다.

### Android 앱 (Tauri)

전제: Android Studio·SDK·NDK, Rust Android 타깃, 에뮬레이터(AVD `resm_phone_api36`·`resm_tablet_api36`)가 설치돼 있고 Windows 개발자 모드가 켜져 있다(`.so` 심볼릭 링크). `adb` 는 `%LOCALAPPDATA%\Android\Sdk\platform-tools\adb.exe`.

```bash
pnpm dev:server                       # 서버는 따로 먼저 띄운다
emulator -avd resm_phone_api36        # 에뮬레이터 부팅 (-no-boot-anim 은 쓰지 않는다)
adb reverse tcp:3104 tcp:3104         # 에뮬레이터의 127.0.0.1:3104 → PC 의 로컬 서버
pnpm app:android:dev                  # 에뮬레이터에 설치·실행, 로컬 서버(127.0.0.1:3104)에 붙는다 (web/.env.tauri-dev)
pnpm app:build:android                # 운영 주소(resm2.approid.team)용 APK (디버그 키 서명)
```

- 로컬 서버 연결은 `adb reverse` 로 한다. 앱 화면은 `https://tauri.localhost` 라서 `http://10.0.2.2:3104` 같은 평문 주소는 혼합 콘텐츠로 막히고, `127.0.0.1` 은 허용된다(실측). `RESM_ALLOWED_ORIGINS` 에 추가할 값은 없다(`app:android:dev` 도 origin 이 `https://tauri.localhost`).
- `app:android:dev` 는 Vite 를 `127.0.0.1:5173` 에 띄우고 `adb reverse tcp:5173` 은 Tauri 가 직접 건다. 에뮬레이터를 `-no-boot-anim` 으로 띄우면 Tauri 가 부팅 완료를 못 알아채 빌드 뒤 설치 단계에서 멈춘다.
- 평문 HTTP 는 디버그 빌드에서만 허용된다(Tauri 기본값). `app:build:android` 는 release 빌드라 평문을 허용하지 않고, 배포 키가 아직 없어 디버그 키로 서명한다. 결과물: `app/src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk` (arm64-v8a·armeabi-v7a·x86·x86_64 포함).
- 로컬 서버에 붙는 패키지 APK 가 필요하면 Windows 앱과 같은 방식으로 `--config` 덮어쓰기(before 명령 `--mode tauri-dev`, CSP `connect-src` 에 `http://127.0.0.1:3104`)를 주고 `pnpm --dir app tauri android build --debug --apk --target x86_64` 로 만든다.
- 상태바·내비게이션 바·키보드 영역은 `MainActivity.kt` 가 화면 바깥 여백으로 처리한다(웹 CSS 변경 없음).
- Android 빌드도 `web/dist` 를 앱 모드로 덮어쓴다(위 Windows 와 같음).

### 테스트 3종

```bash
cargo test -p resm-server        # 서버 통합 테스트 (TEST_DATABASE_URL 필요)
pnpm test:web                    # 웹 단위 테스트 (vitest)
pnpm build:web && pnpm test:e2e  # 브라우저 E2E (Playwright)
```

- E2E 최초 1회: `pnpm --dir web exec playwright install chromium`.
- `pnpm test:e2e` 는 서버를 직접 띄운다(`web/dist` 제공, `TEST_DATABASE_URL` 을 서버의 `DATABASE_URL` 로 넘김). 개발 DB 는 건드리지 않으며 `TEST_DATABASE_URL` 이 없으면 시작 전에 실패한다. 3104 포트가 이미 쓰이고 있으면 실패하므로 개발 서버를 먼저 끈다.
- 서버 테스트·E2E 는 둘 다 테스트 DB 를 쓰지만 테이블을 비우지 않고 실행마다 새 이메일을 쓰므로 동시에 돌려도 서로를 깨지 않는다. 그래도 한 번에 하나씩 실행하길 권한다(E2E 가 만든 사용자·매물은 테스트 DB 에 남는다).

## 로드맵

| 단계 | 내용 | 비고 |
|---|---|---|
| 1 | 뼈대: Rust 서버 + 로그인 + 매물 관리, 세 플랫폼에서 동작 | 설계: `docs/superpowers/specs/2026-11-12-resm-rust-stage-1-design.md` |
| 2 | 계약 워크플로·양식 인쇄, 고객(당사자 포함), 일정 | |
| 3 | 엑셀 입출력, 실거래가(통계·지도) | 카카오 지도 키의 앱 출처 등록 확인 필요 |
| 4 | 네이버 매물 수집 | 가장 위험. 착수 전에 Rust에서 봇 차단 우회가 되는지 작은 실험부터 |
| 5 | 설정·회원·알림 화면·홈 대시보드, 이후 도메인 전환 | |

## 현재 반영 상태

- 2026-11-12: 저장소 뼈대(cargo/pnpm workspace, .gitignore, .env.example, 작업 규칙)와 1단계 설계·계획 문서를 추가했다. `app/` 외 코드는 이후 작업에서 추가한다.
- 2026-11-12: `app/` Tauri 2 셸(RESM, `team.approid.resm`)을 만들고, Daum 우편번호 검색을 원본 방식(`embed`)으로 Windows 앱·Android 폰/태블릿 에뮬레이터·브라우저에서 확인했다. 이 PC에 Android 빌드 환경을 설치했다.
- 2026-11-12: `server/` axum API 서버 뼈대를 추가했다. 환경변수 설정, `{ "error": "..." }` 오류 응답, `GET /api/v1/health`(DB 확인)가 동작한다.
- 2026-11-12: 서버 인증 코어를 추가했다. 원본 bcryptjs 해시(cost 12)를 그대로 검증하고, 세션 토큰(32바이트 base64url, DB에는 SHA-256 hex)·`CurrentUser` 추출기(Bearer 우선, 없으면 `session` 쿠키)가 기존 Next.js 앱의 `Session` 행과 호환된다.
- 2026-11-12: 서버 인증 API를 추가했다. 가입·로그인·로그아웃·me API(`/api/v1/auth/*`), 가입 시 superadmin 알림, CORS·쿠키 인증 Origin 검사를 추가했다. 원본 bcryptjs 계정으로 로그인되고 발급 세션의 `tokenHash` 가 원본 계산과 일치함을 확인했다.
- 2026-11-12: 서버 매물 API를 추가했다. 목록(view: all·favorites·in-progress·contracted)·단건·등록·부분 수정(PATCH, null=비움)·일괄 삭제(`/api/v1/properties*`)가 본인 매물만 대상으로 동작하고, 금액(price·rentPrice)은 원 단위 십진 문자열로 주고받으며 등록 시 superadmin 알림이 생성된다.
- 2026-11-12: `web/` React SPA 뼈대를 추가했다. 원본 `ui` 프리미티브·사이드바·헤더·푸터·테마 토글을 그대로 옮기고, API 클라이언트(쿠키/Bearer 겸용)와 인증 가드(비로그인 → `/login`, 로그인 → `/properties`)를 갖췄다. 서버에 닿지 않으면 "서버에 연결할 수 없습니다" 화면과 다시 시도 버튼이 나온다. 로그인·가입·매물 화면은 자리 표시 상태다.
- 2026-11-12: `web/` 로그인·회원가입·이용약관·개인정보처리방침 화면을 원본 그대로 옮겼다. 회원가입의 주소 검색은 Daum 우편번호 `embed`(스크립트 주소는 `t1.kakaocdn.net`)로 동작하고, 서버 오류 문구는 폼 안에 한 번만 표시된다. 원본 웹과 계정이 서로 공유됨을 확인했다.
- 2026-11-12: `web/` 매물 화면(전체·관심·계약진행·계약완료 목록, 등록, 수정)을 원본 그대로 옮겼다. 표/카드 반응형, 검색·필터·50개 페이징, 선택 삭제(확인 다이얼로그), 더블클릭 인라인 편집, 관심 별·색상 태그가 동작한다. 금액은 API가 원 문자열, 화면이 만원이며 변환은 `web/src/lib/properties/money.ts` 한 곳에서만 한다. 엑셀·계약진행 전환·계약 링크·미등록 표시·당사자 섹션은 이후 단계에서 되살린다.
- 2026-11-12: 서버가 `web/dist` 를 제공한다(`RESM_STATIC_DIR`). 새로고침·딥링크는 SPA 로, 없는 `/api/...` 는 JSON 404 로 응답하고 `assets/` 는 영구 캐시된다. Playwright 브라우저 E2E(가입 → 매물 등록·수정·삭제 → 로그아웃, 모바일 카드)를 테스트 DB 로 실행하고 README 에 개발 실행법을 정리했다.
- 2026-11-12: Windows 앱이 실제 화면(`web/`)을 띄운다. 로그인 토큰을 앱 저장소에 보관해 껐다 켜도 로그인이 유지되고, 우편번호 검색·매물 등록/수정/삭제·서버 불통 화면과 다시 시도가 앱에서 동작한다. CSP 를 켰고 `pnpm app:build:win` 으로 운영 주소용 NSIS 설치본을 만든다.
- 2026-11-12: 앱 origin 을 `https://tauri.localhost` 로 바꿔 우편번호 검색 창이 처음부터 https 로 열린다. CSP 에 `base-uri`·`form-action` 을 추가하고 style-src 를 Tauri 가 바꾸지 않게 고정했다.
- 2026-11-12: Android 앱이 폰·태블릿 에뮬레이터에서 동작한다. 가입(우편번호 검색)·로그인 유지·매물 카드/표·등록/수정/삭제·서버 불통 화면을 확인했고, 상태바·내비게이션 바·키보드가 화면을 가리지 않는다. `pnpm app:android:dev`(로컬 서버, `adb reverse`)와 `pnpm app:build:android`(운영 주소 APK)를 추가했다.
