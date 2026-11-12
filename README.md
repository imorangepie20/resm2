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

`server/`, `web/`, `app/` 는 이후 작업에서 만든다.

## 기술 스택

- 서버: Rust, axum + tokio + sqlx(PostgreSQL)
- 화면: Vite + React 19 + TypeScript + Tailwind v4, react-router, TanStack Query
- 앱: Tauri 2 (Windows NSIS, Android APK)

## 실행 방법

서버·화면·앱이 만들어지면 여기에 적는다. 환경변수 목록은 `.env.example` 참고.

## 로드맵

| 단계 | 내용 | 비고 |
|---|---|---|
| 1 | 뼈대: Rust 서버 + 로그인 + 매물 관리, 세 플랫폼에서 동작 | 설계: `docs/superpowers/specs/2026-11-12-resm-rust-stage-1-design.md` |
| 2 | 계약 워크플로·양식 인쇄, 고객(당사자 포함), 일정 | |
| 3 | 엑셀 입출력, 실거래가(통계·지도) | 카카오 지도 키의 앱 출처 등록 확인 필요 |
| 4 | 네이버 매물 수집 | 가장 위험. 착수 전에 Rust에서 봇 차단 우회가 되는지 작은 실험부터 |
| 5 | 설정·회원·알림 화면·홈 대시보드, 이후 도메인 전환 | |

## 현재 반영 상태

- 2026-11-12: 저장소 뼈대(cargo/pnpm workspace, .gitignore, .env.example, 작업 규칙)와 1단계 설계·계획 문서를 추가했다. 코드는 아직 없다.
- 2026-11-12: `app/` Tauri 2 셸(RESM, `team.approid.resm`)을 만들고, Daum 우편번호 검색을 원본 방식(`embed`)으로 Windows 앱·Android 폰/태블릿 에뮬레이터·브라우저에서 확인했다. 이 PC에 Android 빌드 환경을 설치했다.
