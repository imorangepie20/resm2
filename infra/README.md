# 배포 (Zorin, `resm2.approid.team`)

resm Rust 서버(`resm-server`)를 기존 `real-es` 운영 스택 옆에 **별도 compose project**로 올리는 구성이다. 서버는 마이그레이션을 하지 않으며 기존 운영 DB(`real_es`)에 `real_es_app` 역할로 접속만 한다. 기존 스택의 컨테이너·볼륨·compose 파일은 건드리지 않는다.

> **상태: 2026-11-12 Zorin 에 배포했고 `https://resm2.approid.team` 으로 공개됐다(호스트 포트 3105, 이미지는 개발 PC 에서 빌드해 옮김).** 아래 "운영 변경" 표시가 붙은 단계는 소유자 승인 후에만 실행한다.

## 파일

| 파일 | 설명 |
| --- | --- |
| `infra/Dockerfile` | 다단계 빌드: Node/pnpm 으로 `web` → Rust 로 `resm-server`(`--locked`) → debian-slim 런타임(비루트 uid 10001, `EXPOSE 3104`) |
| `infra/compose.zorin.yml` | project `resm-rust`, 서비스 `resm-server` 하나. `127.0.0.1:3105:3104`(호스트 3104 는 다른 서비스가 사용 중), 기존 네트워크에 `external` 참여 |
| `infra/resm.env.example` | 환경 파일 견본(이름과 비밀 아닌 예시 값). 실제 `infra/resm.env` 는 git 에 올리지 않는다 |
| `.dockerignore` | `.env`, `target`, `node_modules`, `.git` 등을 빌드 컨텍스트에서 제외 |

## 기존 스택과의 접점

| 항목 | 값 | 근거 |
| --- | --- | --- |
| 네트워크 | `real-es-frontend`(앱·터널), `real-es-backend`(`internal: true`, DB) | `real-es/infra/compose.zorin.yml` 의 `networks:` `name:` 값. 이름이 명시돼 있어 project 접두어가 붙지 않는다 |
| DB | 서비스명 `postgres`(backend 망 안 호스트명), 포트 5432, DB `real_es`, 역할 `real_es_app` | 같은 파일 `postgres` 서비스, `postgres/init/10-application-role.sh` |
| DB 비밀번호 | 파일 `real-es/infra/secrets/app.env` 의 `DATABASE_URL` 안(환경변수 방식, docker secret 아님). 접속 문자열 끝에 `?schema=public`(Prisma 용)이 붙어 있다 | `infra/secrets/app.env.example` |
| cloudflared | 같은 compose 의 `tunnel` 서비스(profile `tunnel`), `frontend` 망만 연결, `TUNNEL_TOKEN`(`infra/secrets/tunnel.env`) 방식 = 대시보드에서 관리하는 터널 `real-es-zorin`. ingress 설정 파일 없음 | 같은 파일 `tunnel`, 배포 설계 문서 |
| 위치 | Zorin `~/apps/real-es`, project 이름 `real-es` | 배포 설계 문서 |

새 컨테이너는 `real-es-backend`(DB 접속)와 `real-es-frontend`(터널이 `resm-server:3104` 로 도달) **둘 다** 참여한다.

## 사전 조건 (Zorin)

- 기존 스택 실행 중: `docker compose` 로 `real-es` 의 `postgres`·`app`·`tunnel` 이 떠 있고 `docker network ls` 에 `real-es-frontend`·`real-es-backend` 가 있다.
- 호스트 포트 `127.0.0.1:3105` 가 비어 있다(`ss -ltn | grep 3105`).
- `real-es/infra/secrets/app.env` 를 읽을 수 있다(비밀번호 출처).
- `resm-server` 라는 컨테이너가 없다(`docker ps -a --filter name=resm-server`).
- 시작 전 기준 상태를 적어 둔다: `docker ps --format '{{.Names}} {{.Status}}'`, `curl -i https://resm.approid.team/api/health`, `free -h`, `nproc`.
- **스키마 일치 확인(읽기 전용)**: 운영 DB 에 적용된 마이그레이션이 이 서버의 테스트가 기반한 목록과 같은지 첫 기동 전에 본다. 기존 스택의 postgres 컨테이너 안에서 컨테이너 환경변수(`REAL_ES_MIGRATION_USER`, `POSTGRES_DB`)를 그대로 쓰므로 비밀번호를 입력·출력하지 않는다(컨테이너 안 로컬 소켓 접속).

  ```sh
  docker exec "$(docker ps -q --filter label=com.docker.compose.project=real-es --filter label=com.docker.compose.service=postgres)"     sh -c 'psql -U "$REAL_ES_MIGRATION_USER" -d "$POSTGRES_DB" -At -c "SELECT migration_name FROM _prisma_migrations ORDER BY started_at, migration_name"'
  ```

  `real-es/prisma/migrations` 기준으로 아래 22개가 같은 순서로 나와야 한다.

  ```
20260623004428_init_agency
20260623031510_init_auth
20260623061711_init_legal_division
20260623063116_init_complex_article
20260623090005_complex_coords
20260623215848_add_property_type
20260624054035_add_favorite
20260624111441_add_property
20260624231004_add_property_name
20260625103221_add_contract_workflow
20260625131133_add_property_address
20260625173209_add_realprice_cache
20260625222306_add_customer
20260626061319_add_event
20260626113712_add_customer_zipcode
20260626225446_add_property_party
20260627031904_add_property_color_tag
20260627195103_add_agency_user_contact
20260627200042_add_agency_zipcode
20260701053331_add_system_config
20260703002615_add_notification
20260703014754_add_user_address
  ```

  다르면(빠졌거나 더 있으면) **새 서버를 시작하지 말고** 차이를 소유자에게 보고한다.
- 모든 compose 명령에 `-p resm-rust` 를 붙인다. 셸에 `COMPOSE_PROJECT_NAME=real-es` 가 남아 있으면 파일의 `name:` 보다 우선해 기존 스택을 건드릴 수 있다.

## 배포 절차

**운영 변경**: 1~6 단계는 Zorin 에서 실행한다. 승인 전에는 아무것도 하지 않는다.

1. 코드를 `~/apps/resm` 에 둔다(예: `git clone` 후 배포할 커밋 체크아웃). 배포 SHA 를 기록한다.
2. 환경 파일을 만든다. 비밀번호는 화면에 출력하지 않는다.

   ```sh
   cd ~/apps/resm
   cp infra/resm.env.example infra/resm.env
   chmod 600 infra/resm.env
   # infra/resm.env 의 DATABASE_URL 한 줄을 편집기로 채운다.
   # real-es/infra/secrets/app.env 의 DATABASE_URL 에서 "?schema=public" 만 빼고 같은 값을 쓴다.
   ```
3. 빌드·기동 (Rust 빌드는 처음 수 분 걸린다).
   **주의**: 빌드는 모든 코어와 수 GB 메모리를 쓴다. 같은 호스트에서 운영 중인 기존 앱·DB 가 그동안 느려질 수 있고, 메모리가 모자라면 운영 컨테이너가 죽을 수 있다. `free -h` 의 여유가 4GB 미만이면 여기서 빌드하지 말고, 같은 아키텍처의 다른 PC 에서 `docker build -f infra/Dockerfile -t resm-server:local .` → `docker save resm-server:local | gzip | ssh <zorin> 'gunzip | docker load'` 로 옮긴 뒤 `--build` 없이 `up -d` 한다.

   ```sh
   docker compose -p resm-rust -f infra/compose.zorin.yml up -d --build
   docker compose -p resm-rust -f infra/compose.zorin.yml ps      # healthy 확인
   ```
4. Zorin 에서 로컬 확인: `curl -i http://127.0.0.1:3105/api/v1/health` → 200 `{"status":"ok"}`. `docker logs resm-server` 에 오류가 없어야 한다.
5. 기존 스택 영향 확인: `curl -i https://resm.approid.team/api/health` 와 `docker ps` 로 `real-es` 컨테이너가 그대로인지.
6. **Cloudflare 대시보드(소유자 작업, 운영 변경)**: Zero Trust → Networks → Tunnels → `real-es-zorin` → Public Hostname 추가.
   - Subdomain/Domain: `resm2.approid.team`
   - Service: Type `HTTP`, URL `resm-server:3104`
   - Additional settings 의 **HTTP Host Header 는 비워 둔다**(아래 "Host/Origin" 참고).
   - 이 터널은 기존 `resm.approid.team` 과 공유한다. 기존 호스트명 행은 건드리지 않고, 저장 직후 `curl -i https://resm.approid.team/api/health` 로 기존 사이트를 다시 확인한다.
   - 터널 컨테이너가 `real-es-frontend` 망에 있고 `resm-server` 도 같은 망에 있으므로 이름으로 해석된다.

## 환경변수

| 이름 | 값 | 이유 |
| --- | --- | --- |
| `DATABASE_URL` | `postgresql://real_es_app:<URL인코딩 비밀번호>@postgres:5432/real_es` | backend 망 안의 `postgres` 호스트. 비밀은 파일에만 |
| `RESM_BIND` | `0.0.0.0:3104` | 컨테이너 안에서 호스트 포트 매핑·터널이 닿도록 |
| `RESM_COOKIE_SECURE` | `true` | 공개 주소가 https |
| `RESM_ALLOWED_ORIGINS` | `https://tauri.localhost` | Windows 앱·Android WebView 의 Origin(`useHttpsScheme` 라 둘 다 https)(CORS 와 Origin 검사). 브라우저의 `https://resm2.approid.team` 은 Origin 과 Host 가 같으므로(`server/src/http.rs` `origin_check`) 넣지 않아도 된다 |
| `RESM_STATIC_DIR` | `/srv/resm/web` | 이미지에 복사된 `web/dist` |

## 배포 후 확인 목록

1. `https://resm2.approid.team/api/v1/health` 200, `https://resm.approid.team/api/health` 200(기존 웹 정상).
2. 기존 운영 계정으로 `https://resm2.approid.team` 로그인.
3. 브라우저 / Windows 설치본 / Android APK(운영 주소) 각각에서 로그인 → 목록 → 검색·필터 → 등록 → 수정 → 삭제. 앱은 실제 API 호출이 되고 DevTools(WebView)에 CSP 위반이 0건이어야 한다.
4. 새 앱에서 등록한 매물이 `resm.approid.team` 에 보이고 반대도 된다. 기존 웹 관리자 알림에 "새 매물 등록" 이 찍힌다.
5. 세 곳 모두 원본과 같은 디자인(라이트/다크).
6. 폰 폭은 카드, 넓은 폭은 표.
7. 세 곳 모두 회원가입의 우편번호 검색 동작.
8. 터널 경유 Host/Origin: 브라우저(`https://resm2.approid.team`)에서 로그인이 403 없이 성공해야 한다.
9. 앱 origin 의 OPTIONS preflight 가 Cloudflare 를 통과한다:
   `curl -i -X OPTIONS https://resm2.approid.team/api/v1/auth/login -H 'Origin: https://tauri.localhost' -H 'Access-Control-Request-Method: POST' -H 'Access-Control-Request-Headers: content-type,authorization'` → 200 과 `access-control-allow-origin: https://tauri.localhost`.
10. 쿠키: 서버는 `session` 쿠키를 `Path=/; HttpOnly; SameSite=Lax; Secure` 로 내리고 `Domain` 속성이 없다(`server/src/auth/handlers.rs`). 따라서 호스트 전용 쿠키이며 `resm.` 과 `resm2.` 사이에 **공유되지 않는다**. 두 주소에서 각각 로그인해야 한다. 세션 행은 같은 DB `Session` 테이블이고 토큰 해시 방식이 같으므로, 어느 서버가 발급한 토큰이든 두 서버 모두 DB 로 검증한다(Bearer 로는 상호 호환). 쿠키 이름이 같아도 호스트가 달라 충돌하지 않는다.
11. **권장(소유자 작업, Cloudflare 대시보드)**: 서버 자체에는 요청 제한(rate limiting)이 없으므로 `/api/v1/auth/*` 에 Cloudflare rate-limiting 규칙을 건다(로그인·가입 폭주 방어).

## Host / Origin 동작

Origin 검사는 `Authorization` 없는 상태 변경 요청에서 `Origin` 의 호스트가 요청 `Host` 와 같거나 `RESM_ALLOWED_ORIGINS` 에 있어야 통과시킨다. 로컬에서 확인한 결과(로컬 컨테이너, 로그인 잘못된 자격):

| 요청 | 결과 |
| --- | --- |
| `Host: resm2.approid.team`, `Origin: https://resm2.approid.team` | 401(검사 통과) |
| `Host: resm2.approid.team`, `Origin: https://evil.example` | 403 |
| `Origin: https://tauri.localhost` | 401 |
| `Host: resm-server:3104`, `Origin: https://resm2.approid.team` | **403** |

마지막 줄이 핵심이다. 프록시가 Host 를 업스트림 이름(`resm-server:3104`)으로 바꾸면 브라우저 로그인이 막힌다. cloudflared 의 Host 설정(`httpHostHeader`, 대시보드 "HTTP Host Header")은 기본이 빈 값이며, 공식 문서는 빈 값일 때 무엇을 보내는지 적지 않는다. 알려진 동작은 원래 요청의 Host(공개 호스트명)를 그대로 전달하는 것이다. 따라서 이 칸을 비워 두고 확인 목록 8번으로 실제 검증한다. 만약 403 이 나오면 Host Header 칸을 `resm2.approid.team` 으로 채운다(권장: 서버 설정·재배포가 필요 없고 출처 검사가 그대로 유지된다). 그럴 수 없을 때만 차선책으로 `RESM_ALLOWED_ORIGINS` 에 `https://resm2.approid.team` 을 추가하고 `docker compose ... up -d` 로 재기동한다.

## 롤백

스키마는 건드리지 않았다(서버는 마이그레이션을 하지 않는다). 새 서버로 로그인·등록한 세션·매물·알림 행은 공유 DB 에 그대로 남는다(의도된 동작). **운영 변경**:

1. Cloudflare 대시보드에서 `resm2.approid.team` 공개 호스트명 삭제(DNS 에 CNAME 이 남으면 그것도 삭제).
2. `cd ~/apps/resm && docker compose -p resm-rust -f infra/compose.zorin.yml down`.
3. `resm.approid.team`(기존 스택)은 영향이 없다. 확인: `curl -i https://resm.approid.team/api/health`.

## 알려진 제한

- 서버 시작 시 DB 에 연결하지 못하면 즉시 종료하고, `restart: unless-stopped` 가 재시도한다.
- 서버는 SIGTERM 핸들러가 없다. compose 의 `init: true`(tini) 덕에 `docker stop` 이 1초 안에 끝난다. `docker run` 으로 직접 실행할 때는 `--init` 을 붙인다.
- 배포 전 DB 백업은 하지 않는다(현재 운영 데이터는 가상 데이터라는 소유자 결정, 설계 §5.6). 실데이터가 들어오면 첫 쓰기 전에 `pg_dump` 를 먼저 한다.
- 로컬 DB 와 운영 DB 의 데이터는 다르다. 로컬 검증은 임시 계정을 만들었다.
- 컨테이너 메모리 상한 256M, CPU 상한 1.0코어, 로그 10MB x 3 회전.
- Windows 설치본은 코드 서명이 없고 자동 업데이트가 없다.
- 이미지는 Zorin 에서 빌드한다(레지스트리 없음). 호스트 아키텍처가 로컬(amd64)과 다르면 처음부터 다시 빌드된다.
