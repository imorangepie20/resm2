# RESM 프로젝트 가이드

개발자와 AI 코딩 세션이 이 저장소에서 작업할 때 먼저 읽는 문서다. 작업 규칙은 [CLAUDE.md](../CLAUDE.md).

## 개요

중개사무소용 부동산 업무 도구(매물 관리, 계약 진행, 고객, 일정, 실거래가, 네이버 매물 수집, 엑셀 입출력). 원본 `c:\Wspace\real-es`(Next.js + Prisma + PostgreSQL, 읽기 전용 참고)를 Rust 서버 + React SPA + Tauri 2 앱으로 재작성한다.

## 구조

- `server/` axum API 서버 (`/api/v1`, JSON). 기존 `real_es` DB를 sqlx로 쓴다. 스키마 주인은 원본 `real-es/prisma/` 이며 이 서버는 마이그레이션을 실행하지 않는다.
- `web/` React SPA. 원본의 shadcn UI 컴포넌트를 복사해 쓰고 데이터는 `/api/v1` 로 받는다.
- `app/` Tauri 2 셸. 같은 화면 묶음을 Windows·Android 앱에 담는다.
- `docs/superpowers/specs/`, `docs/superpowers/plans/` 단계별 설계·계획 문서.

## 로드맵

| 단계 | 내용 | 비고 |
|---|---|---|
| 1 | 뼈대: Rust 서버 + 로그인 + 매물 관리, 세 플랫폼에서 동작 | 설계: `docs/superpowers/specs/2026-11-12-resm-rust-stage-1-design.md` |
| 2 | 계약 워크플로·양식 인쇄, 고객(당사자 포함), 일정 | |
| 3 | 엑셀 입출력, 실거래가(통계·지도) | 카카오 지도 키의 앱 출처 등록 확인 필요 |
| 4 | 네이버 매물 수집 | 가장 위험. 착수 전에 Rust에서 봇 차단 우회가 되는지 작은 실험부터 |
| 5 | 설정·회원·알림 화면·홈 대시보드, 이후 도메인 전환 | |

각 단계는 따로 설계 문서를 쓰고 승인받는다.

## 현재 참고 상태

- 2026-11-12: 저장소 뼈대와 1단계 설계·계획 문서가 있고, 서버 뼈대(Task 2)까지 끝났다. 다음은 계획서의 Task 3부터.
- 2026-11-12 (Task 1): `app/` Tauri 2 셸 생성(식별자 `team.approid.resm`, productName `RESM`, 아이콘은 `real-es-desktop` 세트 복사). 화면은 임시로 `app/spike/index.html`(우편번호 실험 페이지). cargo workspace members는 `server`, `app/src-tauri`.
- 2026-11-12 (Task 2): `server/` axum 서버 뼈대. `GET /api/v1/health`, 오류는 `{ "error": "한국어 메시지" }`, 미등록 `/api/v1` 경로는 JSON 404. 테스트 DB는 전용 Docker 컨테이너(`resm-dev-pg`, 127.0.0.1:5433)의 `real_es_test`.
- 2026-11-12 (Task 3): `server/src/auth/` 인증 코어. bcrypt cost 12(원본 bcryptjs `$2b$` 해시 검증 호환), 세션 토큰 = 랜덤 32바이트 base64url(패딩 없음, 43자)·`"Session"."tokenHash"` = SHA-256 소문자 hex·만료 30일(UTC naive). `CurrentUser`/`SessionToken` 추출기는 `Authorization: Bearer` 가 있으면 그것만(무효여도 쿠키 폴백 없음), 없으면 `session` 쿠키; 만료 행은 조회 시 삭제. HTTP 검증은 Task 4의 `/auth/me` 에서. dev 프로필에서 bcrypt/blowfish만 opt-level 3.
- 2026-11-12 (Task 4): `POST /api/v1/auth/{signup,login,logout}`, `GET /auth/me`. 응답 `{token,user:{id,email,name,role,agencyId}}` + `session` 쿠키(HttpOnly, SameSite=Lax, Path=/, Expires=세션 만료, Secure는 `RESM_COOKIE_SECURE`). 가입은 `pg_advisory_xact_lock(824201409141)` 트랜잭션에서 첫 사용자=superadmin, 아니면 member + 모든 superadmin에게 `Notification`(`/settings/members`); 중복 이메일은 사전 조회·23505 모두 400. bcrypt는 `spawn_blocking`, 비밀번호는 bcryptjs처럼 앞 72바이트만 사용. CORS는 `RESM_ALLOWED_ORIGINS` 만(GET/POST/PATCH, authorization·content-type, credentials 없음); Authorization 없는 비안전 메서드 요청의 `Origin` 이 `Host` 와 같은 출처이거나 `RESM_ALLOWED_ORIGINS` 항목과 정확히 같을 때만 통과, 그 외(`null` 포함)는 403(`server/src/http.rs`; 앱의 로그인·가입 POST 허용). 교차 확인: 원본 bcryptjs(cost 12) 해시 계정으로 `/auth/login` 200, Rust 발급 토큰의 sha256 hex == `"tokenHash"`. (원본 웹 앱 미기동이라 쿠키 값 주입 확인은 생략)
- 2026-11-12 (Task 5): `server/src/properties/` 매물 API. `GET /properties?view=`(all|favorites|in-progress|contracted, 그 외 400)·`GET /properties/{id}`·`POST /properties`(201 `{id}`, source 기본 "수기", status 기본 "진행", superadmin 알림 `/properties/{id}/edit`)·`PATCH /properties/{id}`(키 없음=유지, null=NULL, `status:null` 무시, 빈 패치는 `updatedAt` 불변 204)·`POST /properties/delete {ids}`(200 `{count}`). 모든 쿼리에 `"userId"` 조건, 타인 매물은 404. 요청 본문은 `serde_json::Value` 로 받아 고정 컬럼 표(`model.rs` FIELDS)로 검증·변환(컬럼명은 요청에서 오지 않음). 금액은 BIGINT↔십진 문자열(`^\d{1,18}$` 또는 음이 아닌 정수, 아니면 400), 목록 정렬 `"createdAt" DESC, "id" DESC`, `createdAt` 은 `...Z` ISO 문자열. `dealAmount`·`contractChecklist` 는 읽고 쓰지 않음. 테스트 헬퍼 `common::signup`(사용자+세션 직접 INSERT).
- 2026-11-12 (Task 5 보정): `server/src/db.rs` 의 `connect` 가 풀의 모든 연결에 `SET TIME ZONE 'UTC'` 를 실행한다. Prisma 가 시간대 없는 `TIMESTAMP(3)` 에 UTC 를 저장하므로 `DEFAULT CURRENT_TIMESTAMP` 도 UTC 로 기록되게 하기 위함이며, 서버 main 과 테스트가 같은 함수로 풀을 만든다. 매물 INSERT 는 `createdAt`·`updatedAt` 을 모두 `now() at time zone 'utc'` 로 명시한다.
- **우편번호 검색 방식: `embed`** — 원본처럼 Daum `postcode.v2.js` 를 동적으로 로드해 `new daum.Postcode({...}).embed(el)`. 세 플랫폼 모두 `판교역로 235` 검색 → 첫 결과 선택 → 우편번호 `13494`·도로명주소 출력 확인. `server-iframe`·`tauri-window` 대안은 필요 없었다.
  - 플랫폼별 실제 출처(`location.origin`): Windows 앱(내장 자산) `http://tauri.localhost`, Android 앱(폰·태블릿 에뮬레이터, 디버그 APK) `http://tauri.localhost`, 브라우저 `http://localhost:<port>`. `tauri dev`(devUrl 없음)일 때 Windows는 `http://127.0.0.1:1430`. `.env.example` 의 `RESM_ALLOWED_ORIGINS` 는 이미 `http://tauri.localhost` 를 포함해 고치지 않았다.
  - 주의: 우편번호 검색 iframe이 "t1.daumcdn.net 도메인 종료 안내 - 2026년 11월 30일까지 t1.kakaocdn.net으로 변경" 공지를 띄운다. `https://t1.kakaocdn.net/mapjsapi/bundle/postcode/prod/postcode.v2.js` 는 같은 파일을 준다. 실제 컴포넌트(Task 7)는 kakaocdn 주소를 써야 한다.
- Android 빌드 환경(이 PC): Android Studio 2026.2.2.8(winget `Google.AndroidStudio`, `C:\Program Files\Android\Android Studio`), SDK `%LOCALAPPDATA%\Android\Sdk` 에 cmdline-tools 22.0(`cmdline-tools\latest`), platform-tools 37.0.1, platforms android-36(빌드 중 Gradle이 android-37.0·build-tools 36.0.0 추가 설치), build-tools 36.1.0, NDK 29.0.14206865, emulator 37.2.13, 시스템 이미지 `android-36;google_apis;x86_64`. 사용자 환경 변수 `ANDROID_HOME=%LOCALAPPDATA%\Android\Sdk`, `NDK_HOME=%ANDROID_HOME%\ndk\29.0.14206865`. Rust 타깃 aarch64/armv7/i686/x86_64-linux-android 추가. AVD `resm_phone_api36`(pixel_9), `resm_tablet_api36`(pixel_tablet). 가속은 WHPX.
  - 제약: Windows 개발자 모드가 꺼져 있어 `tauri android dev`/`tauri android build` 가 `.so` 심볼릭 링크 단계에서 실패한다("Creation symbolic link is not allowed for this system"). 켜려면 관리자 권한으로 설정 → 시스템 → 개발자용 → 개발자 모드. 그 전까지 우회: `tauri android build --debug --apk --target x86_64` 로 `.so` 를 만든 뒤(링크 단계에서 실패해도 됨) `target/x86_64-linux-android/debug/libresm_app_lib.so` 를 `gen/android/app/src/main/jniLibs/x86_64/` 에 복사하고 `gen/android` 에서 `gradlew assembleX86_64Debug -x rustBuildX86_64Debug`.
- 2026-11-12 (Task 6): `web/` Vite+React SPA 뼈대. 원본 `components/ui`(carousel·chart 제외)·레이아웃·`globals.css` 를 Next 의존만 걷어 내고 복사. `lib/api.ts` 가 토큰·credentials·오류 모양을 전담(`skipUnauthorized` 로 `/auth/me`·로그인의 401 은 전역 리다이렉트 제외, 본문 없는 5xx 는 프록시 불통으로 보고 status 0). `useMe()` 는 401 이면 `null`. `globals.css` 의 `--font-geist-*` 참조는 next/font 가 없어 제거(시스템 폰트 스택만 사용). 로그인·가입·약관·매물 라우트는 `routes/placeholder.tsx` 자리 표시이며 Task 7~8 에서 교체한다.
- 2026-11-12 (Task 7): `web/src/routes/{login,register,terms,privacy,auth-layout}.tsx`·`components/postcode-search.tsx`·`components/auth/auth-card.tsx`·`lib/properties/format.ts`. 오류는 `useMutation`(`meta.silent`)의 `error.message` 를 폼에 인라인 1회만 표시(전역 토스트·401 리다이렉트 없음). 로그인·가입 성공 후 이동은 `RedirectIfAuthed` 가 `["me"]` 갱신을 보고 한 번만 수행. 로그인 화면의 "비밀번호 찾기" 링크는 뺐다. 테스트: `routes/auth-pages.test.tsx`(Testing Library 도입).
- 2026-11-12 (Task 8): `web/src/routes/properties/{list-page,new-page,edit-page,property-list,property-form}.tsx`, `lib/properties/{fields,color-tags,money,api}.ts`, `lib/naver/{trade-types,property-types}.ts`. 금액 규칙: API 원 문자열, 화면 만원, 변환은 `money.ts` 한 곳(BigInt 없이 문자열 자릿수 연산, 소수 4자리 초과는 원본처럼 버림, 음수 입력은 거부). 매물 엔드포인트 호출과 요청 본문 타입 변환(`toApiValue`·`buildPropertyBody`: float·int는 JSON 숫자, bool, 빈 값은 null, 빈 status 생략)은 `lib/properties/api.ts` 만 한다. 쿼리 키 `["properties", view]`·`["property", id]`, 변경 성공 시 `["properties"]` 무효화. 목록의 인라인 편집·별·색상은 낙관적 반영 후 실패하면 바꾼 필드만 되돌리고 전역 토스트. 4개 목록 경로는 `key={view}` 로 상태를 분리. 수정 화면은 `gcTime: 0`(열 때마다 최신 값). 수정 폼의 금액 입력은 의도적으로 원본과 다르다: 원본은 `groupDigits`/`stripDigits` 로 소수점을 버려 1.5만원(15,000원)을 15로 보여 주고 손대면 저장값이 바뀌었으므로, 포트는 `money.ts` 의 `formatManwonInput` 으로 소수 4자리까지 유지하고 사용자가 고치지 않은 금액 필드는 요청에 넣지 않는다(저장 금액이 조용히 바뀌지 않게). 목록의 낙관적 갱신·롤백은 `useMutation` 의 onMutate/onError(행·필드 단위 복원)로 옮기고, 다른 매물 변경이 진행 중이면 재조회를 미룬다.
- 2026-11-12 (Task 9): `server/src/static_files.rs`(`ServeDir` + `index.html` 폴백, `/assets` 영구 캐시, 나머지 `no-cache`, `/api`·`/api/*` JSON 404), `web/playwright.config.ts`·`web/e2e/smoke.spec.ts`. E2E 는 `TEST_DATABASE_URL` 로만 서버를 띄우며 `reuseExistingServer: false`. 루트 스크립트 `dev:server`·`build:web`·`test:e2e`.
