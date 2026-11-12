# resm-server 배포 이미지. 빌드 컨텍스트는 저장소 루트: docker build -f infra/Dockerfile .

# (a) web 빌드 — 브라우저 모드(plain vite build). VITE_API_BASE 가 없으므로 /api/v1 상대 경로를 쓴다.
FROM node:22-bookworm-slim AS web
ENV CI=true
WORKDIR /repo
RUN corepack enable
# 의존성 레이어: 매니페스트와 lock 만 먼저 복사해 캐시한다.
COPY package.json pnpm-workspace.yaml pnpm-lock.yaml ./
COPY web/package.json web/package.json
COPY app/package.json app/package.json
RUN pnpm install --frozen-lockfile --filter web...
COPY web web
RUN pnpm --filter web exec vite build

# (b) resm-server release 빌드 — Cargo.lock 을 그대로 지킨다(--locked).
FROM rust:1.98-bookworm AS server
WORKDIR /repo
COPY Cargo.toml Cargo.lock ./
# workspace 멤버 app/src-tauri 는 매니페스트만 두고 빈 lib 로 대체한다.
# -p resm-server 빌드는 Tauri 크레이트·시스템 라이브러리를 컴파일하지 않으므로 Cargo.lock 은 그대로 유효하다.
COPY app/src-tauri/Cargo.toml app/src-tauri/Cargo.toml
RUN mkdir -p app/src-tauri/src && touch app/src-tauri/src/lib.rs
# 의존성 레이어: server 소스 없이 먼저 의존성만 컴파일한다.
COPY server/Cargo.toml server/Cargo.toml
RUN mkdir -p server/src && echo 'fn main() {}' > server/src/main.rs && touch server/src/lib.rs \
    && cargo build --release --locked -p resm-server \
    && rm -rf server/src
COPY server/src server/src
# 더미 빌드 산출물이 최신으로 오인되지 않도록 mtime 을 갱신한다.
RUN touch server/src/main.rs server/src/lib.rs \
    && cargo build --release --locked -p resm-server

# (c) 런타임 — sqlx 는 TLS 기능 없이 빌드되므로 OpenSSL·CA 인증서가 필요 없다.
FROM debian:bookworm-slim AS runtime
RUN useradd --system --uid 10001 --no-create-home --shell /usr/sbin/nologin resm
COPY --from=server /repo/target/release/resm-server /usr/local/bin/resm-server
COPY --from=web /repo/web/dist /srv/resm/web
ENV RESM_BIND=0.0.0.0:3104 \
    RESM_STATIC_DIR=/srv/resm/web
USER resm
EXPOSE 3104
ENTRYPOINT ["/usr/local/bin/resm-server"]
